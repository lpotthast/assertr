use alloc::{format, string::String, vec::Vec};
use core::{
    borrow::Borrow,
    fmt::{self, Debug, Write},
    marker::PhantomData,
};

use crate::assertions::{
    collection::{Collection, StableOrder},
    map::Map,
};

use super::{
    GroupStyle, RenderingOrder,
    budget::RenderingBudget,
    type_info::{TypeInfo, Typed},
    value::ValueRenderer,
};

/// Renders diagnostic values with an assertion chain's renderer and output budget.
///
/// Custom assertion implementations obtain this context through
/// [`AssertThat::render`](crate::AssertThat::render). Use [`value`](Self::value) for one leaf value
/// or [`values`](Self::values) for an ad-hoc list or set. A leaf adapter retains its complete Rust
/// type name and a configurable [`TypeHint`](super::TypeHint), whether or not text output shows
/// that hint. Synthetic groups retain the canonical types and default short hints of their items
/// rather than inventing an outer Rust type for the group.
pub struct RenderingContext<'r, R> {
    renderer: &'r R,
    budget: RenderingBudget,
}

impl<R> Clone for RenderingContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for RenderingContext<'_, R> {}

impl<'r, R> RenderingContext<'r, R> {
    pub(crate) const fn new(renderer: &'r R, budget: RenderingBudget) -> Self {
        Self { renderer, budget }
    }

    pub(crate) const fn max_items(self) -> usize {
        self.budget.max_items()
    }

    /// Adapts one typed leaf value to [`Debug`] using the chain's renderer and output budget.
    ///
    /// The returned adapter always retains `T`'s complete Rust type name and a short type hint.
    /// Text output hides the hint by default; customize the metadata with
    /// [`Typed::with_type_hint`] and its visibility with [`Typed::show_type_hint`].
    pub fn value<'a, T: ?Sized>(self, value: &'a T) -> Typed<RenderedValue<'a, T, R>>
    where
        'r: 'a,
    {
        self.value_with_type_info(value, TypeInfo::of::<T>())
    }

    fn value_with_type_info<'a, T: ?Sized>(
        self,
        value: &'a T,
        info: TypeInfo,
    ) -> Typed<RenderedValue<'a, T, R>>
    where
        'r: 'a,
    {
        let body = RenderedValue {
            value,
            rendering: self,
        };
        Typed::from_info(body, info)
    }

    /// A typed `owner` rendered as a one-field tuple variant, such as `Err(value)`.
    ///
    /// The returned adapter retains the owner's type information, while the inner value retains
    /// its own independently.
    pub(crate) fn variant<'a, O: ?Sized, T: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        value: &'a T,
    ) -> Typed<Variant<'a, T, R>>
    where
        'r: 'a,
    {
        let body = Variant {
            name,
            value: self.value(value),
        };
        Typed::new::<O>(body)
    }

    /// A typed `owner` rendered as the named one-field struct and field.
    ///
    /// The returned adapter retains the owner's type information, while the field value retains
    /// its own independently.
    pub(crate) fn struct_field<'a, O: ?Sized, T: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        field: &'static str,
        value: &'a T,
    ) -> Typed<StructField<'a, T, R>>
    where
        'r: 'a,
    {
        let body = StructField {
            name,
            field,
            value: self.value(value),
        };
        Typed::new::<O>(body)
    }

    /// A typed `owner` rendered with a field whose contents cannot be inspected.
    ///
    /// The returned adapter retains the owner's type information. No field type is inferred when
    /// there is no concrete field value to inspect.
    #[allow(clippy::unused_self)] // Keep every structural adapter on the rendering entry point.
    pub(crate) fn unavailable_struct_field<O: ?Sized>(
        self,
        _owner: &O,
        name: &'static str,
        field: &'static str,
        unavailable: &'static str,
    ) -> Typed<UnavailableStructField> {
        let body = UnavailableStructField {
            name,
            field,
            unavailable,
        };
        Typed::new::<O>(body)
    }

    /// Adapts a [`Collection`]'s own item values to [`Debug`] as a list or set in iteration order.
    ///
    /// Every value uses the chain's renderer and leaf limit and retains the item type's canonical
    /// information and default short hint. The synthetic group has no invented outer Rust type and
    /// uses the chain's per-group item limit. The adapter retains the collection by reference and
    /// obtains its elements only when formatted.
    ///
    /// Use [`borrowed_values`](Self::borrowed_values) when each collection item borrows a different
    /// type that the renderer should receive, such as rendering `String` items as `str`.
    #[must_use]
    pub fn values<'a, C: Collection + ?Sized>(
        self,
        values: &'a C,
        style: GroupStyle,
    ) -> RenderedValues<'a, C::Item, C, R>
    where
        'r: 'a,
    {
        self.borrowed_values::<C::Item, C>(values, style)
    }

    /// Adapts values which borrow `T` to [`Debug`] as a list or set in iteration order.
    ///
    /// This is the explicit borrowed-view counterpart to [`values`](Self::values). Each item is
    /// passed to the renderer as `&T`, and the retained child type information describes `T`.
    #[must_use]
    pub fn borrowed_values<'a, T: ?Sized, C: Collection + ?Sized>(
        self,
        values: &'a C,
        style: GroupStyle,
    ) -> RenderedValues<'a, T, C, R>
    where
        'r: 'a,
        C::Item: Borrow<T>,
    {
        RenderedValues {
            items: values,
            item: PhantomData,
            item_type: TypeInfo::of::<T>(),
            style,
            sorted_for_rendering: false,
            rendering: self,
        }
    }

    /// The elements of a [`Collection`], rendered according to its presentation metadata.
    pub(crate) fn collection<'a, C>(
        self,
        collection: &'a C,
    ) -> Typed<RenderedValues<'a, C::Item, C, R>>
    where
        'r: 'a,
        C: Collection + ?Sized,
    {
        let presentation = C::PRESENTATION;
        let body = RenderedValues {
            items: collection,
            item: PhantomData,
            item_type: TypeInfo::of::<C::Item>(),
            style: presentation.style(),
            sorted_for_rendering: presentation.order() == RenderingOrder::SortByRenderedText,
            rendering: self,
        };
        Typed::new::<C>(body).show_type_hint(presentation.shows_type_hint())
    }

    /// The elements of a stable-order collection, always rendered in their semantic order.
    ///
    /// Positional diagnostics use this adapter instead of the collection's ordinary rendering
    /// order so a displayed index always refers to the element shown at that position.
    pub(crate) fn stable_collection<'a, C>(
        self,
        collection: &'a C,
    ) -> Typed<RenderedValues<'a, C::Item, C, R>>
    where
        'r: 'a,
        C: StableOrder + ?Sized,
    {
        let presentation = C::PRESENTATION;
        let body = RenderedValues {
            items: collection,
            item: PhantomData,
            item_type: TypeInfo::of::<C::Item>(),
            style: presentation.style(),
            sorted_for_rendering: false,
            rendering: self,
        };
        Typed::new::<C>(body).show_type_hint(presentation.shows_type_hint())
    }

    /// The entries of a [`Map`], rendered with its type hint and iteration-order policy.
    ///
    /// The adapter retains the map by reference and obtains its entries only when formatted.
    pub(crate) fn map<'a, M>(self, map: &'a M) -> Typed<MapEntries<'a, M, R>>
    where
        'r: 'a,
        M: Map + ?Sized,
    {
        let body = MapEntries {
            map,
            key_type: TypeInfo::of::<M::Key>(),
            value_type: TypeInfo::of::<M::Value>(),
            sorted_for_rendering: M::RENDERING_ORDER == RenderingOrder::SortByRenderedText,
            rendering: self,
        };
        Typed::new::<M>(body).show_type_hint(true)
    }

    /// A synthetic list of rendered key/value tuples with an explicit iteration-order policy.
    ///
    /// Every key and value retains its respective type information. The list itself has no
    /// invented outer Rust type. The adapter retains the collection by reference and obtains its
    /// entries only when formatted.
    pub(crate) fn entry_list<
        'a,
        K: ?Sized,
        V: ?Sized,
        BK,
        BV,
        C: Collection<Item = (BK, BV)> + ?Sized,
    >(
        self,
        entries: &'a C,
        sort_for_rendering: bool,
    ) -> EntryList<'a, K, V, C, R>
    where
        'r: 'a,
        BK: Borrow<K>,
        BV: Borrow<V>,
    {
        EntryList {
            entries,
            key_value: PhantomData,
            key_type: TypeInfo::of::<K>(),
            value_type: TypeInfo::of::<V>(),
            sorted_for_rendering: sort_for_rendering,
            rendering: self,
        }
    }
}

/// The rendered body of one typed diagnostic value under an assertion chain's rendering policy.
///
/// [`RenderingContext::value`] returns it inside [`Typed`], which supplies the value's type
/// metadata independently of this leaf-rendering body.
pub struct RenderedValue<'a, T: ?Sized, R> {
    value: &'a T,
    rendering: RenderingContext<'a, R>,
}

impl<T: ?Sized, R: ValueRenderer<T>> Debug for RenderedValue<'_, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = render_leaf(self.value, self.rendering, f.alternate())?;
        f.write_str(&output)
    }
}

/// A rendered value inside a one-field tuple variant, such as `Some(value)`.
pub(crate) struct Variant<'a, T: ?Sized, R> {
    name: &'static str,
    value: Typed<RenderedValue<'a, T, R>>,
}

impl<T: ?Sized, R: ValueRenderer<T>> Debug for Variant<'_, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.name).field(&self.value).finish()
    }
}

/// A rendered value inside a one-field struct, such as `RefCell { value: 1 }`.
pub(crate) struct StructField<'a, T: ?Sized, R> {
    name: &'static str,
    field: &'static str,
    value: Typed<RenderedValue<'a, T, R>>,
}

impl<T: ?Sized, R: ValueRenderer<T>> Debug for StructField<'_, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.name)
            .field(self.field, &self.value)
            .finish()
    }
}

/// A one-field struct whose field is temporarily inaccessible, such as `Mutex { data: <locked> }`.
pub(crate) struct UnavailableStructField {
    name: &'static str,
    field: &'static str,
    unavailable: &'static str,
}

impl Debug for UnavailableStructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.name)
            .field(self.field, &Verbatim(self.unavailable))
            .finish()
    }
}

/// A [`Debug`] adapter for an ad-hoc list or set under an assertion chain's rendering policy.
///
/// Create it with [`RenderingContext::values`] or [`RenderingContext::borrowed_values`]. Each item
/// retains its type information. This synthetic group does not claim to represent a concrete outer
/// Rust value. The adapter retains its source collection by reference and re-iterates it each time
/// it is formatted.
pub struct RenderedValues<'a, T: ?Sized, C: ?Sized, R> {
    items: &'a C,
    item: PhantomData<&'a T>,
    item_type: TypeInfo,
    style: GroupStyle,
    sorted_for_rendering: bool,
    rendering: RenderingContext<'a, R>,
}

impl<T: ?Sized, C: Collection + ?Sized, R: ValueRenderer<T>> Debug for RenderedValues<'_, T, C, R>
where
    C::Item: Borrow<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maximum = self.rendering.max_items();

        if self.sorted_for_rendering {
            let rendered = sorted_and_truncated(
                self.items.elements().map(|value| {
                    render_debug(
                        &self
                            .rendering
                            .value_with_type_info(value.borrow(), self.item_type),
                        f.alternate(),
                    )
                }),
                maximum,
            )?;
            write_values(f, self.style, rendered.iter().map(|value| Verbatim(value)))?;
        } else {
            let rendered = self.items.elements().take(maximum).map(|value| {
                self.rendering
                    .value_with_type_info(value.borrow(), self.item_type)
            });
            write_values(f, self.style, rendered)?;
        }

        write_suffix(
            f,
            self.items.length().saturating_sub(maximum),
            "element",
            self.sorted_for_rendering,
        )
    }
}

/// Rendered key/value entries in map syntax, including the key and value type information.
pub(crate) struct MapEntries<'a, M: ?Sized, R> {
    map: &'a M,
    key_type: TypeInfo,
    value_type: TypeInfo,
    sorted_for_rendering: bool,
    rendering: RenderingContext<'a, R>,
}

impl<M: Map + ?Sized, R> Debug for MapEntries<'_, M, R>
where
    R: ValueRenderer<M::Key> + ValueRenderer<M::Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maximum = self.rendering.max_items();

        if self.sorted_for_rendering {
            let rendered = sorted_and_truncated(
                self.map.entries().map(|(key, value)| {
                    Ok((
                        render_debug(
                            &self.rendering.value_with_type_info(key, self.key_type),
                            f.alternate(),
                        )?,
                        render_debug(
                            &self.rendering.value_with_type_info(value, self.value_type),
                            f.alternate(),
                        )?,
                    ))
                }),
                maximum,
            )?;
            f.debug_map()
                .entries(
                    rendered
                        .iter()
                        .map(|(key, value)| (Verbatim(key), Verbatim(value))),
                )
                .finish()?;
        } else {
            f.debug_map()
                .entries(self.map.entries().take(maximum).map(|(key, value)| {
                    (
                        self.rendering.value_with_type_info(key, self.key_type),
                        self.rendering.value_with_type_info(value, self.value_type),
                    )
                }))
                .finish()?;
        }

        write_suffix(
            f,
            self.map.length().saturating_sub(maximum),
            "entry",
            self.sorted_for_rendering,
        )
    }
}

/// Rendered key/value entries as a synthetic list of tuples, including the child type information.
pub(crate) struct EntryList<'a, K: ?Sized, V: ?Sized, C: ?Sized, R> {
    entries: &'a C,
    key_value: PhantomData<(&'a K, &'a V)>,
    key_type: TypeInfo,
    value_type: TypeInfo,
    sorted_for_rendering: bool,
    rendering: RenderingContext<'a, R>,
}

impl<K: ?Sized, V: ?Sized, BK, BV, C: Collection<Item = (BK, BV)> + ?Sized, R> Debug
    for EntryList<'_, K, V, C, R>
where
    BK: Borrow<K>,
    BV: Borrow<V>,
    R: ValueRenderer<K> + ValueRenderer<V>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maximum = self.rendering.max_items();
        let tuples = self.entries.elements().map(|(key, value)| {
            (
                self.rendering
                    .value_with_type_info(key.borrow(), self.key_type),
                self.rendering
                    .value_with_type_info(value.borrow(), self.value_type),
            )
        });

        if self.sorted_for_rendering {
            let rendered = sorted_and_truncated(
                tuples.map(|tuple| render_debug(&tuple, f.alternate())),
                maximum,
            )?;
            f.debug_list()
                .entries(rendered.iter().map(|tuple| Verbatim(tuple)))
                .finish()?;
        } else {
            f.debug_list().entries(tuples.take(maximum)).finish()?;
        }

        write_suffix(
            f,
            self.entries.length().saturating_sub(maximum),
            "entry",
            self.sorted_for_rendering,
        )
    }
}

/// Writes pre-rendered text without `Debug` escaping.
struct Verbatim<'a>(&'a str);

impl Debug for Verbatim<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Adapts a formatting closure to `Debug`.
struct FormatterFn<F>(F);

impl<F> Debug for FormatterFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

fn render_leaf<T: ?Sized, R: ValueRenderer<T>>(
    value: &T,
    rendering: RenderingContext<'_, R>,
    alternate: bool,
) -> Result<String, fmt::Error> {
    let value = FormatterFn(|f: &mut fmt::Formatter<'_>| rendering.renderer.fmt(value, f));
    let mut output = BoundedOutput::new(rendering.budget.max_leaf_characters());
    if alternate {
        write!(output, "{value:#?}")?;
    } else {
        write!(output, "{value:?}")?;
    }
    Ok(output.finish())
}

fn render_debug<T: Debug + ?Sized>(value: &T, alternate: bool) -> Result<String, fmt::Error> {
    let mut output = String::new();
    if alternate {
        write!(output, "{value:#?}")?;
    } else {
        write!(output, "{value:?}")?;
    }
    Ok(output)
}

/// Retains at most `maximum` characters of the text written to it and counts the rest.
///
/// A leaf renderer's complete output never has to be held in memory: characters beyond the
/// limit are counted for the omission marker and dropped as they arrive.
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

    fn finish(mut self) -> String {
        if self.omitted != 0 {
            self.retained.push_str(&omission(self.omitted, "character"));
        }
        self.retained
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

/// Renders every item so the items can be sorted by their text, then keeps the first `maximum`.
///
/// A zero budget renders nothing, so leaf renderers are never invoked for omitted output.
fn sorted_and_truncated<T: Ord>(
    rendered: impl Iterator<Item = Result<T, fmt::Error>>,
    maximum: usize,
) -> Result<Vec<T>, fmt::Error> {
    if maximum == 0 {
        return Ok(Vec::new());
    }
    let mut items = rendered.collect::<Result<Vec<_>, _>>()?;
    items.sort_unstable();
    items.truncate(maximum);
    Ok(items)
}

fn write_values<I>(f: &mut fmt::Formatter<'_>, style: GroupStyle, values: I) -> fmt::Result
where
    I: Iterator,
    I::Item: Debug,
{
    match style {
        GroupStyle::List => f.debug_list().entries(values).finish(),
        GroupStyle::Set => f.debug_set().entries(values).finish(),
    }
}

fn write_suffix(
    f: &mut fmt::Formatter<'_>,
    omitted: usize,
    noun: &str,
    sorted_for_rendering: bool,
) -> fmt::Result {
    if omitted != 0 {
        write!(f, " ({})", omission(omitted, noun))?;
    }
    if sorted_for_rendering {
        f.write_str(" (sorted for rendering)")?;
    }
    Ok(())
}

/// The marker standing in for output the budget omitted, such as `... 1_200 more elements ...`.
///
/// `noun` is singular. It is pluralized for every count other than one.
pub(crate) fn omission(omitted: usize, noun: &str) -> String {
    let count = grouped_count(omitted);
    if omitted == 1 {
        format!("... {count} more {noun} ...")
    } else if let Some(stem) = noun.strip_suffix('y') {
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
    use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap};
    use core::{any::type_name, cell::RefCell, fmt};

    use crate::prelude::*;
    use crate::renderer::{GroupStyle, RenderingOrder, TypeHint};
    use crate::test_support::{PreservedBag, UnorderedMap, UnorderedSet};

    use super::{RenderingContext, grouped_count, omission};

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
        RenderingContext::new(
            renderer,
            RenderingBudget::builder().max_items(maximum).build(),
        )
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
            assert_that!(omission(2, "entry")).is_equal_to("... 2 more entries ...");
        }

        #[test]
        fn use_the_singular_noun_for_one_omitted_item() {
            assert_that!(omission(1, "element")).is_equal_to("... 1 more element ...");
            assert_that!(omission(1, "entry")).is_equal_to("... 1 more entry ...");
        }
    }

    mod value {
        use super::*;

        #[test]
        fn forwards_compact_and_pretty_formatting() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let value = 7;
            let adapted_value = rendering.value(&value);

            assert_that!(format!("{adapted_value:?}")).is_equal_to("compact(7)");
            assert_that!(format!("{adapted_value:#?}")).is_equal_to("pretty(7)");
        }

        #[test]
        fn type_hints_preserve_compact_and_pretty_formatting() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let value = 7;
            let adapted_value = rendering
                .value(&value)
                .with_type_hint(TypeHint::Label("Number"))
                .show_type_hint(true);

            assert_that!(format!("{adapted_value:?}")).is_equal_to("Number compact(7)");
            assert_that!(format!("{adapted_value:#?}")).is_equal_to("Number pretty(7)");
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
            let value = [1, 2, 3];

            assert_that!(format!("{:?}", rendering.value(&value)))
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
                RenderingBudget::builder().max_leaf_characters(2).build(),
            );

            assert_that!(format!("{:?}", rendering.value("é😊x")))
                .is_equal_to("é😊... 1 more character ...");
        }

        #[test]
        fn does_not_count_the_type_hint_as_part_of_the_leaf() {
            let renderer = RawRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::builder().max_leaf_characters(2).build(),
            );

            let value = rendering.value("é😊x").show_type_hint(true);
            assert_that!(format!("{value:?}")).is_equal_to("str é😊... 1 more character ...");
        }

        #[test]
        fn applies_to_each_collection_value() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::builder()
                    .max_items(2)
                    .max_leaf_characters(3)
                    .build(),
            );
            let values = [123_456, 234_567, 345_678];

            assert_that!(format!(
                "{:#?}",
                rendering.values(&values, GroupStyle::List)
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
                RenderingBudget::builder().max_leaf_characters(2).build(),
            );
            let map = BTreeMap::from([(1_234, 5_678)]);
            let list_entries = [(1_234, 5_678)];

            assert_that!(format!("{:?}", rendering.map(&map)))
                .is_equal_to("BTreeMap {12... 2 more characters ...: 56... 2 more characters ...}");
            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(&list_entries, false)
            ))
            .is_equal_to("[(12... 2 more characters ..., 56... 2 more characters ...)]");
        }
    }

    mod wrappers {
        use super::*;

        #[test]
        fn compose_leaf_output_into_debug_structures() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let value = 7;
            let result = Result::<(), i32>::Err(value);
            let cell = RefCell::new(value);

            assert_that!(format!("{:?}", rendering.variant(&result, "Err", &value)))
                .is_equal_to("Err(compact(7))");
            assert_that!(format!(
                "{:?}",
                rendering.struct_field(&cell, "RefCell", "value", &value)
            ))
            .is_equal_to("RefCell { value: compact(7) }");
            assert_that!(format!(
                "{:?}",
                rendering.unavailable_struct_field(&cell, "RefCell", "value", "<borrowed>")
            ))
            .is_equal_to("RefCell { value: <borrowed> }");
        }

        #[test]
        fn retain_the_outer_and_inner_type_information() {
            let renderer = AlternateAwareRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let value = 7;
            let result = Result::<(), i32>::Err(value);
            let cell = RefCell::new(value);

            let variant = rendering.variant(&result, "Err", &value);
            assert_that!(variant.info.type_name).is_equal_to(type_name::<Result<(), i32>>());
            assert_that!(variant.info.hint).is_equal_to(TypeHint::Short);
            assert_that!(variant.body.value.info.type_name).is_equal_to(type_name::<i32>());

            let field = rendering.struct_field(&cell, "RefCell", "value", &value);
            assert_that!(field.info.type_name).is_equal_to(type_name::<RefCell<i32>>());
            assert_that!(field.body.value.info.type_name).is_equal_to(type_name::<i32>());

            let unavailable =
                rendering.unavailable_struct_field(&cell, "RefCell", "value", "<borrowed>");
            assert_that!(unavailable.info.type_name).is_equal_to(type_name::<RefCell<i32>>());
        }
    }

    mod values {
        use super::*;
        use crate::assertions::{HasLength, collection::Collection};
        use crate::renderer::CollectionPresentation;
        use core::cell::Cell;

        struct ObservedCollection {
            items: Vec<i32>,
            iterations: Cell<usize>,
        }

        impl HasLength for ObservedCollection {
            fn length(&self) -> usize {
                self.items.len()
            }
        }

        impl Collection for ObservedCollection {
            type Item = i32;
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

            fn elements(&self) -> impl Iterator<Item = &Self::Item> {
                self.iterations.set(self.iterations.get() + 1);
                self.items.iter()
            }
        }

        #[test]
        fn apply_style_order_and_item_budget() {
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);
            let values = [3, 1, 2];

            assert_that!(format!("{:?}", rendering.values(&values, GroupStyle::List)))
                .is_equal_to("[3, 1] (... 1 more element ...)");
            assert_that!(format!("{:?}", rendering.values(&values, GroupStyle::Set)))
                .is_equal_to("{3, 1} (... 1 more element ...)");
        }

        #[test]
        fn infer_the_direct_item_type_when_it_has_multiple_borrowed_views() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = vec![String::from("alpha"), String::from("beta")];

            assert_that!(format!("{:?}", rendering.values(&values, GroupStyle::List)))
                .is_equal_to(r#"["alpha", "beta"]"#);
        }

        #[test]
        fn render_items_through_an_explicit_borrowed_view() {
            let renderer = RawRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = vec![String::from("alpha"), String::from("beta")];

            assert_that!(format!(
                "{:?}",
                rendering.borrowed_values::<str, _>(&values, GroupStyle::List)
            ))
            .is_equal_to("[alpha, beta]");
        }

        #[test]
        fn retain_the_item_type_information() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = [1, 2];

            let adapted = rendering.values(&values, GroupStyle::List);

            assert_that!(adapted.item_type.type_name).is_equal_to(type_name::<i32>());
            assert_that!(adapted.item_type.hint).is_equal_to(TypeHint::Short);
        }

        #[test]
        fn retain_and_reiterate_any_collection_without_collecting_its_elements() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = ObservedCollection {
                items: vec![1, 2],
                iterations: Cell::new(0),
            };

            let adapted = rendering.values(&values, GroupStyle::List);
            assert_that!(values.iterations.get()).is_equal_to(0);

            assert_that!(format!("{adapted:?}")).is_equal_to("[1, 2]");
            assert_that!(values.iterations.get()).is_equal_to(1);

            assert_that!(format!("{adapted:?}")).is_equal_to("[1, 2]");
            assert_that!(values.iterations.get()).is_equal_to(2);
        }
    }

    mod collections {
        use super::*;

        #[test]
        fn apply_type_hint_style_order_and_item_budget() {
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);

            assert_that!(format!("{:?}", rendering.collection(&vec![3, 1, 2])))
                .is_equal_to("[3, 1] (... 1 more element ...)");
            assert_that!(format!(
                "{:?}",
                rendering.collection(&BTreeSet::from([3, 1, 2]))
            ))
            .is_equal_to("BTreeSet {1, 2} (... 1 more element ...)");
            assert_that!(format!(
                "{:?}",
                rendering.collection(&UnorderedSet(vec![3, 1, 2]))
            ))
            .is_equal_to("UnorderedSet {1, 2} (... 1 more element ...) (sorted for rendering)");
            assert_that!(format!(
                "{:?}",
                rendering.collection(&PreservedBag(vec![3, 1, 2]))
            ))
            .is_equal_to("PreservedBag [3, 1] (... 1 more element ...)");
            assert_that!(format!(
                "{:?}",
                rendering.collection(&BinaryHeap::from([3, 1, 2]))
            ))
            .is_equal_to("BinaryHeap [1, 2] (... 1 more element ...) (sorted for rendering)");
        }

        #[test]
        fn retain_the_collection_and_item_type_information() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = BTreeSet::from([1, 2]);

            let adapted = rendering.collection(&values);

            assert_that!(adapted.info.type_name).is_equal_to(type_name::<BTreeSet<i32>>());
            assert_that!(adapted.body.item_type.type_name).is_equal_to(type_name::<i32>());
        }
    }

    mod maps {
        use super::*;
        use crate::assertions::{HasLength, map::Map};
        use core::cell::Cell;

        struct ObservedMap {
            entries: Vec<(i32, i32)>,
            iterations: Cell<usize>,
        }

        impl HasLength for ObservedMap {
            fn length(&self) -> usize {
                self.entries.len()
            }
        }

        impl Map for ObservedMap {
            type Key = i32;
            type Value = i32;
            const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

            fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
                self.iterations.set(self.iterations.get() + 1);
                self.entries.iter().map(|(key, value)| (key, value))
            }
        }

        #[test]
        fn apply_type_hint_order_and_item_budget() {
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);

            assert_that!(format!(
                "{:?}",
                rendering.map(&BTreeMap::from([(3, 30), (1, 10), (2, 20)]))
            ))
            .is_equal_to("BTreeMap {1: 10, 2: 20} (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering.map(&UnorderedMap(vec![(3, 30), (1, 10), (2, 20)]))
            ))
            .is_equal_to(
                "UnorderedMap {1: 10, 2: 20} (... 1 more entry ...) (sorted for rendering)",
            );
        }

        #[test]
        fn retain_the_map_key_and_value_type_information() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let values = BTreeMap::from([(1, "one")]);

            let adapted = rendering.map(&values);

            assert_that!(adapted.info.type_name).is_equal_to(type_name::<BTreeMap<i32, &str>>());
            assert_that!(adapted.body.key_type.type_name).is_equal_to(type_name::<i32>());
            assert_that!(adapted.body.value_type.type_name).is_equal_to(type_name::<&str>());
        }

        #[test]
        fn retain_and_reiterate_the_map_without_collecting_its_entries() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let map = ObservedMap {
                entries: vec![(1, 10), (2, 20)],
                iterations: Cell::new(0),
            };

            let adapted = rendering.map(&map);
            assert_that!(map.iterations.get()).is_equal_to(0);

            let first = format!("{adapted:?}");
            assert_that!(map.iterations.get()).is_equal_to(1);

            let second = format!("{adapted:?}");
            assert_that!(map.iterations.get()).is_equal_to(2);
            assert_that!(first).is_equal_to(second);
        }
    }

    mod entry_lists {
        use super::*;
        use crate::assertions::{HasLength, collection::Collection};
        use crate::renderer::CollectionPresentation;
        use core::cell::Cell;

        struct ObservedEntries {
            entries: Vec<(i32, i32)>,
            iterations: Cell<usize>,
        }

        impl HasLength for ObservedEntries {
            fn length(&self) -> usize {
                self.entries.len()
            }
        }

        impl Collection for ObservedEntries {
            type Item = (i32, i32);
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

            fn elements(&self) -> impl Iterator<Item = &Self::Item> {
                self.iterations.set(self.iterations.get() + 1);
                self.entries.iter()
            }
        }

        #[test]
        fn apply_order_and_item_budget() {
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);
            let entries = [(3, 30), (1, 10), (2, 20)];

            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(&entries, false)
            ))
            .is_equal_to("[(3, 30), (1, 10)] (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(&entries, true)
            ))
            .is_equal_to("[(1, 10), (2, 20)] (... 1 more entry ...) (sorted for rendering)");
        }

        #[test]
        fn retain_the_key_and_value_type_information() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let entries = [(1, "one")];

            let adapted = rendering.entry_list::<i32, str, _, _, _>(&entries, false);

            assert_that!(adapted.key_type.type_name).is_equal_to(type_name::<i32>());
            assert_that!(adapted.value_type.type_name).is_equal_to(type_name::<str>());
        }

        #[test]
        fn retain_and_reiterate_any_collection_without_collecting_its_entries() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let entries = ObservedEntries {
                entries: vec![(1, 10), (2, 20)],
                iterations: Cell::new(0),
            };

            let adapted = rendering.entry_list::<i32, i32, _, _, _>(&entries, false);
            assert_that!(entries.iterations.get()).is_equal_to(0);

            let first = format!("{adapted:?}");
            assert_that!(entries.iterations.get()).is_equal_to(1);

            let second = format!("{adapted:?}");
            assert_that!(entries.iterations.get()).is_equal_to(2);
            assert_that!(first).is_equal_to(second);
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

            assert_that!(format!("{:?}", rendering.values(&values, GroupStyle::List)))
                .is_equal_to("[] (... 2 more elements ...)");
            assert_that!(format!(
                "{:?}",
                rendering.collection(&UnorderedSet(vec![1, 2]))
            ))
            .is_equal_to("UnorderedSet {} (... 2 more elements ...) (sorted for rendering)");
            assert_that!(format!("{:?}", rendering.map(&BTreeMap::from(entries))))
                .is_equal_to("BTreeMap {} (... 1 more entry ...)");
            assert_that!(format!("{:?}", rendering.map(&UnorderedMap(vec![(1, 2)]))))
                .is_equal_to("UnorderedMap {} (... 1 more entry ...) (sorted for rendering)");
            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(&entries, false)
            ))
            .is_equal_to("[] (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(&entries, true)
            ))
            .is_equal_to("[] (... 1 more entry ...) (sorted for rendering)");
        }
    }
}
