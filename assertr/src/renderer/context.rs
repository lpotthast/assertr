use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::{
    borrow::Borrow,
    fmt::{self, Debug, Write},
};

use super::{
    GroupStyle, Rendered, RenderedBody, RenderingOrder, budget::RenderingBudget,
    value::ValueRenderer,
};
use crate::{
    assertions::{
        collection::{Collection, StableOrder},
        map::Map,
    },
    util::selection::select_smallest,
};

/// Renders diagnostic values with an assertion chain's renderer and output budget.
///
/// Custom assertion implementations obtain this context through
/// [`AssertThat::render`](crate::AssertThat::render) or
/// [`AssertionContext::render`](crate::expectation::AssertionContext::render). Every method renders
/// its leaves immediately and returns an owned [`Rendered`] tree, requiring a [`ValueRenderer`]
/// only for the leaf types it displays.
///
/// Use [`value`](Self::value) for one leaf value. Leaves retain their complete Rust type name and
/// hide their type hint by default. Show it with [`Rendered::with_type_hint`].
/// Use [`collection`](Self::collection) and [`map`](Self::map) for a subject's own structure and
/// presentation metadata. [`variant`](Self::variant) and [`struct_field`](Self::struct_field)
/// wrap one leaf without requiring a renderer for the owner. Synthetic groups created by
/// [`borrowed_values`](Self::borrowed_values) and [`entry_list`](Self::entry_list) retain the
/// canonical types of their items rather than inventing an outer Rust type for the group.
///
/// Leaves use pretty formatting (`f.alternate() == true`) unless the context was made
/// [`compact`](Self::compact).
///
/// Sorted groups rank items by their rendered text, including leaf truncation, before applying
/// the item limit. Ties keep encounter order. With a nonzero item limit, every item is rendered
/// once, including both leaves of map entries that are left out. A zero item limit renders
/// nothing.
pub struct RenderingContext<'r, R> {
    renderer: &'r R,
    budget: RenderingBudget,
    pretty: bool,
}

impl<R> Debug for RenderingContext<'_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderingContext")
            .field("budget", &self.budget)
            .field("pretty", &self.pretty)
            .finish_non_exhaustive()
    }
}

impl<R> Clone for RenderingContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for RenderingContext<'_, R> {}

/// Identity evidence is an address, independent of any ability to render the pointee.
pub(crate) struct IdentityRenderer;

impl<T: ?Sized> ValueRenderer<T> for IdentityRenderer {
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print only the memory address, using the same format for all output styles and Rust
        // versions. Pointers may also carry extra information, such as a slice's length. If that
        // information differs despite equal addresses, the assertion explains the mismatch in a
        // separate note.
        write!(f, "{:p}", core::ptr::from_ref(value).cast::<()>())
    }
}

impl<'r, R> RenderingContext<'r, R> {
    /// Creates a context with an explicit renderer and budget, rendering pretty leaves.
    pub const fn new(renderer: &'r R, budget: RenderingBudget) -> Self {
        Self {
            renderer,
            budget,
            pretty: true,
        }
    }

    pub(crate) const fn renderer(self) -> &'r R {
        self.renderer
    }

    /// Returns a copy of the active diagnostic retention limits.
    ///
    /// Custom evidence collectors can use these limits to bound retained children and record
    /// omissions. They must still determine the complete assertion result independently of the
    /// budget. Changing the returned copy does not change this context or its assertion chain.
    #[must_use]
    pub const fn budget(self) -> RenderingBudget {
        self.budget
    }

    pub(crate) const fn max_items(self) -> usize {
        self.budget.max_items()
    }

    /// Returns a context rendering compact leaves, passing a formatter with
    /// `f.alternate() == false` to the renderer.
    ///
    /// Use this for evidence embedded inline in surrounding text, where multi-line pretty output
    /// would break the layout. Assertr renders map keys in
    /// [`PathSegment::Key`](crate::failure::PathSegment::Key) path segments this way, for example
    /// `entry["key"]`.
    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.pretty = false;
        self
    }

    /// Formats identity evidence with the same budget, without rendering subject contents or
    /// changing the renderer carried by the assertion chain.
    pub(crate) fn identities(self) -> RenderingContext<'static, IdentityRenderer> {
        RenderingContext {
            renderer: &IdentityRenderer,
            budget: self.budget,
            pretty: self.pretty,
        }
    }

    /// Renders one leaf value using the chain's renderer and leaf budget.
    ///
    /// The result retains `T`'s complete Rust type name. Text output hides its type hint unless
    /// enabled with [`Rendered::with_type_hint`].
    pub fn value<T: ?Sized>(self, value: &T) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        let (text, omitted_characters) = self.leaf(value);
        Rendered::typed::<T>(RenderedBody::Text {
            text,
            omitted_characters,
        })
    }

    /// Renders a typed `owner` as a one-field tuple variant, such as `Err(value)`.
    ///
    /// The result retains the owner's type, while the inner value retains its own. The owner
    /// supplies only its type. Only `value` is rendered, and the leaf budget applies to it. The
    /// variant name is structural text.
    pub fn variant<O: ?Sized, T: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        value: &T,
    ) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        Rendered::typed::<O>(RenderedBody::Variant {
            name,
            value: Box::new(self.value(value)),
        })
    }

    /// Renders a typed `owner` as the named one-field struct and field.
    ///
    /// The result retains the owner's type, while the field value retains its own. The owner
    /// supplies only its type. Only `value` is rendered, and the leaf budget applies to it. The
    /// struct and field names are structural text.
    pub fn struct_field<O: ?Sized, T: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        field: &'static str,
        value: &T,
    ) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        Rendered::typed::<O>(RenderedBody::Struct {
            name,
            fields: alloc::vec![(field, self.value(value))],
        })
    }

    /// Renders a typed `owner` with a field whose contents cannot be inspected.
    ///
    /// The result retains the owner's type. No field type is inferred when there is no concrete
    /// field value to inspect. No renderer capability is required. Names and the `unavailable`
    /// marker, such as `"<locked>"`, are structural text and do not consume the leaf budget.
    #[allow(clippy::unused_self)] // Keep every structural method on the rendering entry point.
    pub fn unavailable_struct_field<O: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        field: &'static str,
        unavailable: &'static str,
    ) -> Rendered {
        Rendered::typed::<O>(RenderedBody::Struct {
            name,
            fields: alloc::vec![(
                field,
                Rendered::untyped(RenderedBody::Placeholder(unavailable))
            )],
        })
    }

    /// Renders the elements of a [`Collection`] according to its presentation metadata.
    ///
    /// Retains the canonical collection and item types. [`Collection::PRESENTATION`] selects the
    /// group syntax, outer type-hint visibility, and rendering order. The item and leaf limits
    /// apply. Sorted rendering selects items after sorting rendered text. Use
    /// [`stable_collection`](Self::stable_collection) when diagnostics refer to positions.
    pub fn collection<C: Collection + ?Sized>(self, collection: &C) -> Rendered
    where
        R: ValueRenderer<C::Item>,
    {
        self.presented(collection, collection.elements(), C::PRESENTATION.order())
    }

    /// Renders the elements of a stable-order collection, always in their semantic order.
    ///
    /// Positional diagnostics use this method instead of [`collection`](Self::collection) so a
    /// displayed index always refers to the element shown at that position. Otherwise it follows
    /// `collection`, including syntax, type metadata, and budget limits.
    pub fn stable_collection<C: StableOrder + ?Sized>(self, collection: &C) -> Rendered
    where
        R: ValueRenderer<C::Item>,
    {
        self.presented(
            collection,
            collection.elements(),
            RenderingOrder::PreserveIteration,
        )
    }

    /// Already observed collection targets, retaining the source collection's presentation and
    /// type. Using the retained references avoids repeating custom `Borrow` conversions during
    /// diagnostics. Elements beyond `observed` count as omitted.
    pub(crate) fn observed_collection<T: ?Sized, C: Collection + ?Sized>(
        self,
        collection: &C,
        observed: &[&T],
        order: RenderingOrder,
    ) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        self.presented(collection, observed.iter().copied(), order)
    }

    fn presented<'a, T: ?Sized + 'a, C: Collection + ?Sized>(
        self,
        collection: &C,
        items: impl Iterator<Item = &'a T>,
        order: RenderingOrder,
    ) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        let presentation = C::PRESENTATION;
        Rendered::typed::<C>(self.group(presentation.style(), items, collection.length(), order))
            .with_type_hint(presentation.shows_type_hint())
    }

    /// Renders the entries of a [`Map`] with its type hint and rendering order.
    ///
    /// The canonical map, key, and value types are retained, with the map's short type hint shown.
    /// The budget limits retained entries and each leaf independently. [`Map::RENDERING_ORDER`]
    /// selects iteration order or sorting by rendered key text, then rendered value text, before
    /// applying the item limit. No [`MapLookup`](crate::assertions::MapLookup) is required.
    pub fn map<M: Map + ?Sized>(self, map: &M) -> Rendered
    where
        R: ValueRenderer<M::Key> + ValueRenderer<M::Value>,
    {
        Rendered::typed::<M>(self.map_body(map, M::RENDERING_ORDER)).with_type_hint(true)
    }

    fn map_body<M: Map + ?Sized>(self, map: &M, order: RenderingOrder) -> RenderedBody
    where
        R: ValueRenderer<M::Key> + ValueRenderer<M::Value>,
    {
        let candidates = map
            .entries()
            .map(|(key, value)| (self.value(key), self.value(value)));
        let (entries, omitted, sorted) =
            self.select(candidates, map.length(), order, |(key, value)| {
                (self.text(key), self.text(value))
            });
        RenderedBody::Map {
            entries,
            omitted,
            sorted,
        }
    }

    /// Renders values which borrow `T` as a synthetic list in the given order.
    ///
    /// Each item is passed to the renderer as `&T`, such as rendering `String` items as `str`, and
    /// retains the type information of `T`. The group has no invented outer Rust type and uses the
    /// chain's per-group item limit. Sorting uses rendered text, including leaf truncation, before
    /// applying the item limit and marks the group as sorted. The order grants no positional
    /// capability.
    pub fn borrowed_values<T: ?Sized, C: Collection + ?Sized>(
        self,
        values: &C,
        order: RenderingOrder,
    ) -> Rendered
    where
        C::Item: Borrow<T>,
        R: ValueRenderer<T>,
    {
        Rendered::untyped(self.group(
            GroupStyle::List,
            values.elements().map(Borrow::borrow),
            values.length(),
            order,
        ))
    }

    /// Renders key/value entries as a synthetic list of tuples in the given order.
    ///
    /// Keys and values are rendered through their respective `Borrow` views and retain their type
    /// information. The list has no invented outer Rust type. `order` selects iteration order or
    /// sorting by rendered tuple text before applying the item limit. Each key and value also
    /// receives the leaf limit.
    pub fn entry_list<K: ?Sized, V: ?Sized, BK, BV, C: Collection<Item = (BK, BV)> + ?Sized>(
        self,
        entries: &C,
        order: RenderingOrder,
    ) -> Rendered
    where
        BK: Borrow<K>,
        BV: Borrow<V>,
        R: ValueRenderer<K> + ValueRenderer<V>,
    {
        let candidates = entries.elements().map(|(key, value)| {
            Rendered::from((
                self.value::<K>(key.borrow()),
                self.value::<V>(value.borrow()),
            ))
        });
        Rendered::untyped(self.selected_group(
            GroupStyle::List,
            candidates,
            entries.length(),
            order,
        ))
    }

    fn group<'a, T: ?Sized + 'a>(
        self,
        style: GroupStyle,
        items: impl Iterator<Item = &'a T>,
        length: usize,
        order: RenderingOrder,
    ) -> RenderedBody
    where
        R: ValueRenderer<T>,
    {
        self.selected_group(style, items.map(|item| self.value(item)), length, order)
    }

    fn selected_group(
        self,
        style: GroupStyle,
        candidates: impl Iterator<Item = Rendered>,
        length: usize,
        order: RenderingOrder,
    ) -> RenderedBody {
        let (items, omitted, sorted) =
            self.select(candidates, length, order, |item| self.text(item));
        RenderedBody::Group {
            style,
            items,
            omitted,
            sorted,
        }
    }

    /// Retains at most the item limit of `candidates`, sorted by `key` when requested. Every
    /// element of a group of `length` elements that is not retained counts as omitted.
    fn select<X, K: Ord>(
        self,
        candidates: impl Iterator<Item = X>,
        length: usize,
        order: RenderingOrder,
        key: impl FnMut(&X) -> K,
    ) -> (Vec<X>, usize, bool) {
        let maximum = self.max_items();
        let sorted = order == RenderingOrder::SortByRenderedText;
        let items: Vec<X> = if sorted {
            select_smallest(candidates, maximum, key)
        } else {
            candidates.take(maximum).collect()
        };
        let omitted = length.saturating_sub(items.len());
        (items, omitted, sorted)
    }

    /// The text a rendered value is sorted by, in this context's layout.
    fn text(self, rendered: &Rendered) -> String {
        if self.pretty {
            format!("{rendered:#}")
        } else {
            format!("{rendered}")
        }
    }

    /// Renders one leaf within the leaf budget, returning the retained text and omitted character
    /// count.
    ///
    /// [`BoundedOutput`] never fails, so an error can only originate from the value renderer. The
    /// text written before the error is kept and followed by [`RENDERER_ERROR`], so a faulty
    /// renderer never turns a diagnostic into a panic.
    fn leaf<T: ?Sized>(self, value: &T) -> (String, usize)
    where
        R: ValueRenderer<T>,
    {
        let value = FormatterFn(|f: &mut fmt::Formatter<'_>| self.renderer.fmt(value, f));
        let mut output = BoundedOutput::new(self.budget.max_leaf_characters());
        let result = if self.pretty {
            write!(output, "{value:#?}")
        } else {
            write!(output, "{value:?}")
        };
        let (mut text, omitted) = output.finish();
        if result.is_err() {
            text.push_str(RENDERER_ERROR);
        }
        (text, omitted)
    }
}

/// Adapts a formatting closure to `Debug`, so the formatter's alternate flag can be selected.
struct FormatterFn<F>(F);

impl<F> Debug for FormatterFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

/// The marker appended to a leaf whose value renderer reported a formatting error.
const RENDERER_ERROR: &str = "<renderer error>";

/// Retains at most `maximum` characters of the text written to it and counts the rest.
///
/// A leaf renderer's complete output never has to be held in memory: characters beyond the limit
/// are counted for the omission marker and dropped as they arrive.
struct BoundedOutput {
    retained: String,
    remaining: usize,
    omitted: usize,
}

impl BoundedOutput {
    fn new(maximum: usize) -> Self {
        Self {
            retained: String::new(),
            remaining: maximum,
            omitted: 0,
        }
    }

    fn finish(self) -> (String, usize) {
        (self.retained, self.omitted)
    }
}

impl Write for BoundedOutput {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut retained_end = text.len();
        let mut retained_characters = 0;
        for (index, _) in text.char_indices() {
            if retained_characters == self.remaining {
                retained_end = index;
                break;
            }
            retained_characters += 1;
        }
        self.retained.push_str(&text[..retained_end]);
        self.remaining -= retained_characters;
        self.omitted += text[retained_end..].chars().count();
        Ok(())
    }
}

/// The marker standing in for output the budget omitted, such as `... 1_200 more elements ...`.
///
/// `noun` is singular. It is pluralized for every count other than one. A `y` following a
/// consonant becomes `ies`, as in `entries`. Every other noun appends `s`, as in `keys`.
pub(crate) fn omission(omitted: usize, noun: &str) -> String {
    let count = grouped_count(omitted);
    if omitted == 1 {
        format!("... {count} more {noun} ...")
    } else if let Some(stem) = noun.strip_suffix('y')
        && stem
            .chars()
            .next_back()
            .is_some_and(|last| last.is_alphabetic() && !"aeiouAEIOU".contains(last))
    {
        format!("... {count} more {stem}ies ...")
    } else {
        format!("... {count} more {noun}s ...")
    }
}

fn grouped_count(value: usize) -> String {
    let decimal = format!("{value}");
    let mut grouped = String::with_capacity(decimal.len() + decimal.len() / 3);
    for (index, character) in decimal.chars().enumerate() {
        if index != 0 && (decimal.len() - index).is_multiple_of(3) {
            grouped.push('_');
        }
        grouped.push(character);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use alloc::{
        boxed::Box,
        collections::{BTreeMap, BTreeSet, BinaryHeap},
    };
    use core::{any::type_name, cell::RefCell, fmt};

    use super::{RenderingContext, grouped_count, omission};
    use crate::{
        prelude::*,
        renderer::{GroupStyle, Rendered, RenderedBody, RenderingOrder},
        test_support::{PreservedBag, UnorderedMap, UnorderedSet},
    };

    struct AlternateAwareRenderer;

    impl ValueRenderer<i32> for AlternateAwareRenderer {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                write!(f, "pretty({value})")
            } else {
                write!(f, "compact({value})")
            }
        }
    }

    struct RawRenderer;

    impl ValueRenderer<str> for RawRenderer {
        fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(value)
        }
    }

    struct PanickingRenderer;

    impl ValueRenderer<i32> for PanickingRenderer {
        fn fmt(&self, _value: &i32, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            panic!("a zero-item budget must not render leaf values")
        }
    }

    fn with_max_items<R>(renderer: &R, maximum: usize) -> RenderingContext<'_, R> {
        RenderingContext::new(renderer, RenderingBudget::default().with_max_items(maximum))
    }

    fn check_type<T: ?Sized>(value: &Rendered, shown: bool) {
        assert_that!(value.type_name).is_equal_to(Some(type_name::<T>()));
        assert_that!(value.shows_type_hint).is_equal_to(shown);
    }

    fn leaf<T: ?Sized>(text: &str) -> Rendered {
        Rendered::typed::<T>(RenderedBody::Text {
            text: text.into(),
            omitted_characters: 0,
        })
    }

    fn text(rendered: &Rendered, pretty: bool) -> String {
        if pretty {
            format!("{rendered:#}")
        } else {
            format!("{rendered}")
        }
    }

    mod bounded_sorting {
        use core::cell::Cell;

        use super::*;

        // A full sort, applied to trees built by iteration-preserving paths.
        fn full_sort(mut tree: Rendered, limit: usize, pretty: bool) -> Rendered {
            match &mut tree.body {
                RenderedBody::Group {
                    items,
                    omitted,
                    sorted,
                    ..
                } => {
                    items.sort_by_cached_key(|item| text(item, pretty));
                    *omitted = items.len().saturating_sub(limit);
                    items.truncate(limit);
                    *sorted = true;
                }
                RenderedBody::Map {
                    entries,
                    omitted,
                    sorted,
                } => {
                    entries.sort_by_cached_key(|(key, value)| {
                        (text(key, pretty), text(value, pretty))
                    });
                    *omitted = entries.len().saturating_sub(limit);
                    entries.truncate(limit);
                    *sorted = true;
                }
                _ => panic!("expected a structural group"),
            }
            tree
        }

        pub(super) struct StatefulRenderer(Cell<usize>);

        impl ValueRenderer<i32> for StatefulRenderer {
            fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let call = self.0.get();
                self.0.set(call + 1);
                write!(f, "{value}:{call}:{}", f.alternate())
            }
        }

        pub(super) fn assert_full_sort(
            leaves: usize,
            build: impl Fn(RenderingContext<'_, StatefulRenderer>, RenderingOrder) -> Rendered,
        ) {
            fn context(
                renderer: &StatefulRenderer,
                budget: RenderingBudget,
                pretty: bool,
            ) -> RenderingContext<'_, StatefulRenderer> {
                let context = RenderingContext::new(renderer, budget);
                if pretty { context } else { context.compact() }
            }

            for leaf_limit in [0, 1, 2, usize::MAX] {
                for limit in [0, 1, 2, 4, 5, 6, usize::MAX] {
                    for pretty in [false, true] {
                        let budget =
                            RenderingBudget::unlimited().with_max_leaf_characters(leaf_limit);
                        let renderer = StatefulRenderer(Cell::new(0));
                        let actual = build(
                            context(&renderer, budget.with_max_items(limit), pretty),
                            RenderingOrder::SortByRenderedText,
                        );
                        // In particular, losing map entries still render both leaves in order.
                        assert_that!(renderer.0.get()).is_equal_to(if limit == 0 {
                            0
                        } else {
                            leaves
                        });
                        let renderer = StatefulRenderer(Cell::new(0));
                        let reference = build(
                            context(&renderer, budget, pretty),
                            RenderingOrder::PreserveIteration,
                        );
                        let reference = full_sort(reference, limit, pretty);
                        assert_that!(actual).is_equal_to(&reference);
                    }
                }
            }
        }
    }

    mod omissions {
        use super::*;

        #[test]
        fn use_readable_digit_grouping() {
            assert_that!(grouped_count(0)).is_equal_to("0");
            assert_that!(grouped_count(999)).is_equal_to("999");
            assert_that!(grouped_count(1_000)).is_equal_to("1_000");
            assert_that!(grouped_count(99_950)).is_equal_to("99_950");
            assert_that!(grouped_count(1_234_567)).is_equal_to("1_234_567");
        }

        #[test]
        fn name_the_omitted_items() {
            assert_that!(omission(1_200, "element")).is_equal_to("... 1_200 more elements ...");
        }

        #[test]
        fn use_the_singular_noun_for_one_omitted_item() {
            assert_that!(omission(1, "element")).is_equal_to("... 1 more element ...");
            assert_that!(omission(1, "entry")).is_equal_to("... 1 more entry ...");
        }

        #[test]
        fn replace_y_with_ies_only_after_a_consonant() {
            assert_that!(omission(2, "entry")).is_equal_to("... 2 more entries ...");
            assert_that!(omission(2, "key")).is_equal_to("... 2 more keys ...");
            assert_that!(omission(2, "day")).is_equal_to("... 2 more days ...");
        }
    }

    mod renderer_errors {
        use super::*;

        struct PartialRenderer;

        impl ValueRenderer<i32> for PartialRenderer {
            fn fmt(&self, _value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("partial")?;
                Err(fmt::Error)
            }
        }

        #[test]
        fn keep_the_text_written_before_the_error_and_mark_the_leaf() {
            let renderer = PartialRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::default());

            assert_that!(rendering.value(&1).body).is_equal_to(RenderedBody::Text {
                text: "partial<renderer error>".into(),
                omitted_characters: 0,
            });
        }

        #[test]
        fn do_not_panic_while_capturing_a_failure() {
            let failures = assert_that!(1)
                .with_debug_format(|_, _| Err(fmt::Error))
                .capture(|it| it.is_equal_to(2));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains("<renderer error>");
        }
    }

    mod value {
        use super::*;

        #[test]
        fn forwards_compact_and_pretty_formatting() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());

            assert_that!(rendering.value(&7).to_string()).is_equal_to("pretty(7)");
            assert_that!(rendering.compact().value(&7).to_string()).is_equal_to("compact(7)");
        }

        #[test]
        fn shows_the_type_hint_on_request() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());

            let value = rendering.value(&7);
            check_type::<i32>(&value, false);
            assert_that!(value.with_type_hint(true).to_string()).is_equal_to("i32 pretty(7)");
        }

        #[test]
        fn keeps_collection_looking_output_opaque() {
            struct CollectionLookingRenderer;

            impl ValueRenderer<[i32; 3]> for CollectionLookingRenderer {
                fn fmt(&self, _value: &[i32; 3], f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("[first, second, third]")
                }
            }

            let renderer = CollectionLookingRenderer;
            let rendering = with_max_items(&renderer, 1);

            assert_that!(rendering.value(&[1, 2, 3]).to_string())
                .is_equal_to("[first, second, third]");
        }
    }

    mod leaf_budget {
        use super::*;

        #[test]
        fn counts_unicode_characters_without_splitting_them() {
            let renderer = RawRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(2),
            );

            assert_that!(rendering.value("é😊x").to_string())
                .is_equal_to("é😊... 1 more character ...");
        }

        #[test]
        fn does_not_count_the_type_hint_as_part_of_the_leaf() {
            let renderer = RawRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(2),
            );

            let value = rendering.value("é😊x").with_type_hint(true);
            assert_that!(value.to_string()).is_equal_to("str é😊... 1 more character ...");
        }

        #[test]
        fn applies_to_each_collection_value() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default()
                    .with_max_items(2)
                    .with_max_leaf_characters(3),
            );
            let values = [123_456, 234_567, 345_678];

            assert_that!(format!(
                "{:#}",
                rendering.borrowed_values::<i32, _>(&values, RenderingOrder::PreserveIteration)
            ))
            .is_equal_to(indoc::indoc! {"
                [
                    123... 3 more characters ...,
                    234... 3 more characters ...,
                ] (... 1 more element ...)"});
        }

        #[test]
        fn applies_to_each_key_and_value() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(2),
            );
            let map = BTreeMap::from([(1_234, 5_678)]);
            let list_entries = [(1_234, 5_678)];

            assert_that!(rendering.map(&map).to_string())
                .is_equal_to("BTreeMap {12... 2 more characters ...: 56... 2 more characters ...}");
            assert_that!(
                rendering
                    .entry_list::<i32, i32, _, _, _>(
                        &list_entries,
                        RenderingOrder::PreserveIteration
                    )
                    .to_string()
            )
            .is_equal_to("[(12... 2 more characters ..., 56... 2 more characters ...)]");
        }
    }

    mod wrappers {
        use super::*;

        #[test]
        fn limit_unicode_leaves_without_limiting_structural_names() {
            let rendering = RenderingContext::new(
                &RawRenderer,
                RenderingBudget::unlimited()
                    .with_max_items(0)
                    .with_max_leaf_characters(2),
            );
            let owner = Some("é😊x");
            assert_that!(rendering.variant(&owner, "Some", "é😊x").to_string())
                .is_equal_to("Some(é😊... 1 more character ...)");
            assert_that!(
                rendering
                    .struct_field(&owner, "Wrapper", "value", "é😊x")
                    .to_string()
            )
            .is_equal_to("Wrapper { value: é😊... 1 more character ... }");
        }

        #[test]
        fn retain_owner_and_leaf_metadata() {
            let rendering =
                RenderingContext::new(&AlternateAwareRenderer, RenderingBudget::unlimited());
            let value = 7;
            let result = Result::<(), i32>::Err(value);
            let cell = RefCell::new(value);

            let variant = rendering.variant(&result, "Err", &value);
            check_type::<Result<(), i32>>(&variant, false);
            assert_that!(variant.body).is_equal_to(RenderedBody::Variant {
                name: "Err",
                value: Box::new(leaf::<i32>("pretty(7)")),
            });

            let field = rendering.struct_field(&cell, "RefCell", "value", &value);
            check_type::<RefCell<i32>>(&field, false);
            assert_that!(field.body).is_equal_to(RenderedBody::Struct {
                name: "RefCell",
                fields: vec![("value", leaf::<i32>("pretty(7)"))],
            });

            let unavailable =
                rendering.unavailable_struct_field(&cell, "RefCell", "value", "<borrowed>");
            check_type::<RefCell<i32>>(&unavailable, false);
            assert_that!(unavailable.to_string()).is_equal_to("RefCell { value: <borrowed> }");
            let RenderedBody::Struct { fields, .. } = &unavailable.body else {
                panic!("expected a struct");
            };
            assert_that!(fields[0].1.type_name).is_none();
        }
    }

    mod borrowed_values {
        use super::*;

        #[test]
        fn apply_order_and_item_budget() {
            bounded_sorting::assert_full_sort(5, |rendering, order| {
                rendering.borrowed_values::<i32, _>(&[11, 10, 2, 10, -1], order)
            });
            let rendering = with_max_items(&DebugRenderer, 2);
            let values = [30, 2, 10];

            assert_that!(
                rendering
                    .borrowed_values::<i32, _>(&values, RenderingOrder::PreserveIteration)
                    .to_string()
            )
            .is_equal_to("[30, 2] (... 1 more element ...)");
            assert_that!(
                rendering
                    .borrowed_values::<i32, _>(&values, RenderingOrder::SortByRenderedText)
                    .to_string()
            )
            .is_equal_to("[10, 2] (... 1 more element ...) (sorted for rendering)");
        }

        #[test]
        fn sort_the_budgeted_leaf_text_including_omission_markers() {
            let rendering = RenderingContext::new(
                &RawRenderer,
                RenderingBudget::unlimited()
                    .with_max_items(1)
                    .with_max_leaf_characters(1),
            );
            // Complete text would put "aaz" first. Its larger omission count sorts after "ab"
            // once both leaves have been truncated to one character.
            let values = ["aaz", "ab"];
            let rendered =
                rendering.borrowed_values::<str, _>(&values, RenderingOrder::SortByRenderedText);

            assert_that!(rendered.to_string()).is_equal_to(
                "[a... 1 more character ...] (... 1 more element ...) (sorted for rendering)",
            );
        }

        #[test]
        fn render_items_through_the_borrowed_view_without_an_outer_type() {
            let rendering = RenderingContext::new(&RawRenderer, RenderingBudget::unlimited());
            let values = vec![String::from("alpha"), String::from("beta")];

            let rendered =
                rendering.borrowed_values::<str, _>(&values, RenderingOrder::PreserveIteration);
            assert_that!(rendered.type_name).is_none();
            assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::List,
                items: vec![leaf::<str>("alpha"), leaf::<str>("beta")],
                omitted: 0,
                sorted: false,
            });
        }
    }

    mod collections {
        use super::*;
        use crate::assertions::HasLength;

        #[test]
        fn apply_type_hint_style_order_and_item_budget() {
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);

            assert_that!(rendering.collection(&vec![3, 1, 2]).to_string())
                .is_equal_to("[3, 1] (... 1 more element ...)");
            assert_that!(rendering.collection(&BTreeSet::from([3, 1, 2])).to_string())
                .is_equal_to("BTreeSet {1, 2} (... 1 more element ...)");
            assert_that!(
                rendering
                    .collection(&UnorderedSet(vec![3, 1, 2]))
                    .to_string()
            )
            .is_equal_to("UnorderedSet {1, 2} (... 1 more element ...) (sorted for rendering)");
            assert_that!(
                rendering
                    .collection(&PreservedBag(vec![3, 1, 2]))
                    .to_string()
            )
            .is_equal_to("PreservedBag [3, 1] (... 1 more element ...)");
            assert_that!(
                rendering
                    .collection(&BinaryHeap::from([3, 1, 2]))
                    .to_string()
            )
            .is_equal_to("BinaryHeap [1, 2] (... 1 more element ...) (sorted for rendering)");
        }

        #[test]
        fn retain_collection_and_leaf_metadata() {
            let rendering = with_max_items(&DebugRenderer, 2);
            let tree = rendering.collection(&BTreeSet::from([3, 1, 2]));
            check_type::<BTreeSet<i32>>(&tree, true);
            assert_that!(tree.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::Set,
                items: vec![leaf::<i32>("1"), leaf::<i32>("2")],
                omitted: 1,
                sorted: false,
            });
            let unordered = rendering.collection(&UnorderedSet(vec![3, 1, 2]));
            check_type::<UnorderedSet>(&unordered, true);
            assert_that!(unordered.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::Set,
                items: vec![leaf::<i32>("1"), leaf::<i32>("2")],
                omitted: 1,
                sorted: true,
            });
            #[cfg(feature = "std")]
            {
                let hashed = rendering.collection(&std::collections::HashSet::from([3, 1, 2]));
                check_type::<std::collections::HashSet<i32>>(&hashed, true);
                assert_that!(hashed.body).is_equal_to(&unordered.body);
            }
        }

        struct SortedSequence<T>(Vec<T>);

        impl<T> HasLength for SortedSequence<T> {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl<T> crate::assertions::collection::Collection for SortedSequence<T> {
            type Item = T;
            const PRESENTATION: crate::renderer::CollectionPresentation =
                crate::renderer::CollectionPresentation::list()
                    .with_type_hint(true)
                    .with_order(RenderingOrder::SortByRenderedText);

            fn elements(&self) -> impl Iterator<Item = &T> {
                self.0.iter()
            }
        }

        impl<T> crate::assertions::collection::StableOrder for SortedSequence<T> {}

        #[test]
        fn stable_views_preserve_metadata_but_keep_iteration_order() {
            let rendering = with_max_items(&DebugRenderer, 2);
            let values = SortedSequence(vec![3, 1, 2]);
            for (rendered, expected, sorted) in [
                (rendering.collection(&values), ["1", "2"], true),
                (rendering.stable_collection(&values), ["3", "1"], false),
            ] {
                check_type::<SortedSequence<i32>>(&rendered, true);
                assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                    style: GroupStyle::List,
                    items: expected.map(leaf::<i32>).into(),
                    omitted: 1,
                    sorted,
                });
            }
        }

        #[test]
        fn observed_targets_count_unobserved_elements_as_omitted() {
            let rendering = with_max_items(&DebugRenderer, 5);
            let values = PreservedBag(vec![3, 1, 2]);
            let rendered = rendering.observed_collection(
                &values,
                &[&values.0[0]],
                RenderingOrder::PreserveIteration,
            );

            check_type::<PreservedBag>(&rendered, true);
            assert_that!(rendered.to_string())
                .is_equal_to("PreservedBag [3] (... 2 more elements ...)");
        }
    }

    mod maps {
        use super::*;

        #[test]
        fn apply_type_hint_order_and_item_budget() {
            bounded_sorting::assert_full_sort(10, |rendering, order| {
                let map = UnorderedMap(vec![(11, 2), (10, 20), (10, 11), (10, 11), (-1, 9)]);
                Rendered::untyped(rendering.map_body(&map, order))
            });
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);

            assert_that!(
                rendering
                    .map(&BTreeMap::from([(3, 30), (1, 10), (2, 20)]))
                    .to_string()
            )
            .is_equal_to("BTreeMap {1: 10, 2: 20} (... 1 more entry ...)");
            assert_that!(
                rendering
                    .map(&UnorderedMap(vec![(3, 30), (1, 10), (2, 20)]))
                    .to_string()
            )
            .is_equal_to(
                "UnorderedMap {1: 10, 2: 20} (... 1 more entry ...) (sorted for rendering)",
            );
        }

        #[test]
        fn retain_map_and_entry_metadata() {
            let rendering = with_max_items(&DebugRenderer, 1);
            let tree = rendering.map(&BTreeMap::from([(2, "two"), (1, "one")]));
            check_type::<BTreeMap<i32, &str>>(&tree, true);
            assert_that!(tree.body).is_equal_to(RenderedBody::Map {
                entries: vec![(leaf::<i32>("1"), leaf::<&str>("\"one\""))],
                omitted: 1,
                sorted: false,
            });
        }
    }

    mod entry_lists {
        use super::*;

        #[test]
        fn apply_order_and_item_budget() {
            bounded_sorting::assert_full_sort(10, |rendering, order| {
                rendering.entry_list::<i32, i32, _, _, _>(
                    &[(11, 2), (10, 20), (10, 11), (10, 11), (-1, 9)],
                    order,
                )
            });
            // Tuple punctuation participates in ordering, unlike a map's separate key/value keys.
            let punctuation = [("a", "z"), ("a!", "z"), ("a", "b")];
            assert_that!(
                with_max_items(&RawRenderer, 2)
                    .entry_list::<str, str, _, _, _>(
                        &punctuation,
                        RenderingOrder::SortByRenderedText,
                    )
                    .to_string()
            )
            .is_equal_to("[(a!, z), (a, b)] (... 1 more element ...) (sorted for rendering)");
            let rendering = with_max_items(&DebugRenderer, 2);
            let entries = [(3, 30), (1, 10), (2, 20)];

            assert_that!(
                rendering
                    .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::PreserveIteration)
                    .to_string()
            )
            .is_equal_to("[(3, 30), (1, 10)] (... 1 more element ...)");
        }

        #[test]
        fn retain_borrowed_entry_metadata_without_an_outer_type() {
            let rendering = with_max_items(&DebugRenderer, 1);
            let entries = [(2, String::from("two")), (1, String::from("one"))];
            for (order, key, value) in [
                (RenderingOrder::PreserveIteration, "2", "\"two\""),
                (RenderingOrder::SortByRenderedText, "1", "\"one\""),
            ] {
                let rendered = rendering.entry_list::<i32, str, _, _, _>(&entries, order);
                assert_that!(rendered.type_name).is_none();
                assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                    style: GroupStyle::List,
                    items: vec![Rendered::from((leaf::<i32>(key), leaf::<str>(value)))],
                    omitted: 1,
                    sorted: order == RenderingOrder::SortByRenderedText,
                });
            }
        }
    }

    mod zero_item_budget {
        use super::*;

        #[test]
        fn does_not_render_structural_leaves() {
            let renderer = PanickingRenderer;
            let rendering = with_max_items(&renderer, 0);
            let values = [1, 2];
            let entries = [(1, 2)];

            for order in [
                RenderingOrder::PreserveIteration,
                RenderingOrder::SortByRenderedText,
            ] {
                assert_that!(rendering.borrowed_values::<i32, _>(&values, order).body).is_equal_to(
                    RenderedBody::Group {
                        style: GroupStyle::List,
                        items: Vec::new(),
                        omitted: 2,
                        sorted: order == RenderingOrder::SortByRenderedText,
                    },
                );
                assert_that!(
                    rendering
                        .entry_list::<i32, i32, _, _, _>(&entries, order)
                        .to_string()
                )
                .is_equal_to("[] (... 1 more element ...)");
            }
            assert_that!(rendering.collection(&UnorderedSet(vec![1, 2])).to_string())
                .is_equal_to("UnorderedSet {} (... 2 more elements ...) (sorted for rendering)");
            assert_that!(rendering.map(&BTreeMap::from(entries)).to_string())
                .is_equal_to("BTreeMap {} (... 1 more entry ...)");
            assert_that!(rendering.map(&UnorderedMap(vec![(1, 2)])).to_string())
                .is_equal_to("UnorderedMap {} (... 1 more entry ...)");
        }
    }
}
