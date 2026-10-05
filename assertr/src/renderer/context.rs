use alloc::{boxed::Box, format, string::String};
use core::{
    borrow::Borrow,
    fmt::{self, Debug, Write},
    marker::PhantomData,
};

use crate::{
    assertions::{
        collection::{Collection, StableOrder},
        map::Map,
    },
    util::selection::select_smallest,
};

use super::{
    GroupStyle, IntoRendered, Rendered, RenderedBody, RenderingOrder,
    budget::RenderingBudget,
    rendered::tuple_text,
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
///
/// Use [`collection`](Self::collection) and [`map`](Self::map) for a subject's own structure and
/// presentation metadata. [`variant`](Self::variant) and [`struct_field`](Self::struct_field)
/// wrap one leaf without requiring a renderer for the owner. Adapters are lazy: construction
/// requires no renderer capability, and each formatting or [`IntoRendered`] conversion renders
/// their leaves with this context's budget. Converting once retains an owned tree for reuse.
///
/// Sorted collection, map, and entry-list adapters inspect every entry for a nonzero item limit.
/// They retain at most that limit plus the incoming entry during selection, with cached text keys
/// and encounter order for ties. Groups exceeding the limit use heap selection. Smaller groups sort
/// once. Zero limits do not advance or render entries, and unlimited limits collect and sort all.
/// Every inspected leaf uses the renderer once per conversion, including both leaves of map
/// entries. Selection preserves full-sort output in the requested compact or pretty style.
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
    /// Creates a context with an explicit renderer and budget.
    pub const fn new(renderer: &'r R, budget: RenderingBudget) -> Self {
        Self { renderer, budget }
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

    /// Formats identity evidence with the same budget, without rendering subject contents or
    /// changing the renderer carried by the assertion chain.
    pub(crate) fn identities(self) -> RenderingContext<'static, IdentityRenderer> {
        RenderingContext::new(&IdentityRenderer, self.budget)
    }

    /// Adapts one typed leaf value to [`Debug`] using the chain's renderer and output budget.
    ///
    /// The returned adapter always retains `T`'s complete Rust type name and a short type hint.
    /// Text output hides the hint by default; customize the metadata with [`Typed::with_type_hint`]
    /// and its visibility with [`Typed::show_type_hint`].
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
    /// The returned adapter retains the owner's type information, while the inner value retains its
    /// own independently. Type hints are hidden by default. The owner supplies only its type and
    /// is not retained. Formatting requires `R: ValueRenderer<T>` and applies the leaf budget to
    /// `value`. The variant name is structural text.
    pub fn variant<'a, O: ?Sized, T: ?Sized>(
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
    /// The returned adapter retains the owner's type information, while the field value retains its
    /// own independently. Type hints are hidden by default. The owner supplies only its type and
    /// is not retained. Formatting requires `R: ValueRenderer<T>` and applies the leaf budget to
    /// `value`. The struct and field names are structural text.
    pub fn struct_field<'a, O: ?Sized, T: ?Sized>(
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
    /// there is no concrete field value to inspect. The owner's type hint is hidden by default.
    /// No renderer capability is required. Names and the `unavailable` marker, such as
    /// `"<locked>"`, are structural text and do not consume the leaf budget.
    #[allow(clippy::unused_self)] // Keep every structural adapter on the rendering entry point.
    pub fn unavailable_struct_field<O: ?Sized>(
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
    ///
    /// Retains the canonical collection and item types. [`Collection::PRESENTATION`] selects the
    /// group syntax, outer type-hint visibility, and rendering order. Formatting requires only
    /// `R: ValueRenderer<C::Item>`. Each formatting traverses the borrowed collection lazily and
    /// applies the item and leaf limits. Sorted rendering selects items after sorting rendered
    /// text. Use [`stable_collection`](Self::stable_collection) when diagnostics refer to
    /// positions.
    pub fn collection<'a, C>(self, collection: &'a C) -> Typed<RenderedValues<'a, C::Item, C, R>>
    where
        'r: 'a,
        C: Collection + ?Sized,
    {
        self.borrowed_collection::<C::Item, C>(collection)
    }

    /// A collection's borrowed item view, retaining its presentation and outer type information.
    ///
    /// Follows [`collection`](Self::collection), but formats each item through `Borrow<T>` and
    /// requires only `R: ValueRenderer<T>`. Child metadata describes `T`, including unsized views
    /// such as `str`. Borrowing happens each time the adapter is formatted.
    pub fn borrowed_collection<'a, T: ?Sized, C: Collection + ?Sized>(
        self,
        collection: &'a C,
    ) -> Typed<RenderedValues<'a, T, C, R>>
    where
        'r: 'a,
        C::Item: Borrow<T>,
    {
        let presentation = C::PRESENTATION;
        let body = self
            .borrowed_values::<T, C>(collection, presentation.style())
            .with_order(presentation.order());
        Typed::new::<C>(body).show_type_hint(presentation.shows_type_hint())
    }

    /// The elements of a stable-order collection, always rendered in their semantic order.
    ///
    /// Positional diagnostics use this adapter instead of the collection's ordinary rendering order
    /// so a displayed index always refers to the element shown at that position.
    /// Otherwise it follows [`collection`](Self::collection), including syntax, type metadata,
    /// leaf-renderer requirements, and budget limits. Construction requires [`StableOrder`],
    /// independently of the collection's presentation metadata.
    pub fn stable_collection<'a, C>(
        self,
        collection: &'a C,
    ) -> Typed<RenderedValues<'a, C::Item, C, R>>
    where
        'r: 'a,
        C: StableOrder + ?Sized,
    {
        self.stable_borrowed_collection::<C::Item, C>(collection)
    }

    /// A stable-order collection's borrowed item view, always in semantic order.
    ///
    /// Combines [`borrowed_collection`](Self::borrowed_collection)'s leaf rendering and metadata
    /// with [`stable_collection`](Self::stable_collection)'s positional order and capability bound.
    pub fn stable_borrowed_collection<'a, T: ?Sized, C: StableOrder + ?Sized>(
        self,
        collection: &'a C,
    ) -> Typed<RenderedValues<'a, T, C, R>>
    where
        'r: 'a,
        C::Item: Borrow<T>,
    {
        let mut rendered = self.borrowed_collection::<T, C>(collection);
        rendered.body.sorted_for_rendering = false;
        rendered
    }

    /// Already observed collection targets, retaining the source collection's presentation and
    /// type. Using the retained references avoids repeating custom `Borrow` conversions during
    /// diagnostics.
    pub(crate) fn observed_collection<T: ?Sized, C: Collection + ?Sized>(
        self,
        _collection: &C,
        elements: &[&T],
        total: usize,
        order: RenderingOrder,
    ) -> Rendered
    where
        R: ValueRenderer<T>,
    {
        let presentation = C::PRESENTATION;
        let body = self
            .borrowed_values::<T, _>(elements, presentation.style())
            .with_order(order);
        let mut rendered = Typed::new::<C>(body)
            .show_type_hint(presentation.shows_type_hint())
            .into_rendered();
        if let RenderedBody::Group { items, omitted, .. } = &mut rendered.body {
            *omitted = total.saturating_sub(items.len());
        }
        rendered
    }

    /// The entries of a [`Map`], rendered with its type hint and iteration-order policy.
    ///
    /// The adapter retains the map by reference and obtains its entries only when formatted.
    /// The canonical map, key, and value types are retained, with the map's short type hint shown
    /// by default. Formatting requires only `R: ValueRenderer<M::Key> + ValueRenderer<M::Value>`.
    /// The budget limits retained entries and each leaf independently. [`Map::RENDERING_ORDER`]
    /// selects iteration order or sorting by rendered key text, then rendered value text, before
    /// applying the item limit. No [`MapLookup`](crate::assertions::map::MapLookup) is required.
    pub fn map<'a, M>(self, map: &'a M) -> Typed<MapEntries<'a, M, R>>
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
    /// Every key and value retains its respective type information. The list itself has no invented
    /// outer Rust type. The adapter retains the collection by reference and obtains its entries
    /// only when formatted. Keys and values are rendered through their respective `Borrow` views,
    /// requiring only `R: ValueRenderer<K> + ValueRenderer<V>`.
    ///
    /// `order` selects iteration order or sorting by rendered tuple text before applying the item
    /// limit. Each key and value also receives the leaf limit. This synthetic evidence uses the
    /// explicit order, independently of the source collection's presentation metadata.
    #[must_use]
    pub fn entry_list<'a, K: ?Sized, V: ?Sized, BK, BV, C: Collection<Item = (BK, BV)> + ?Sized>(
        self,
        entries: &'a C,
        order: RenderingOrder,
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
            sorted_for_rendering: order == RenderingOrder::SortByRenderedText,
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
        write_body(f, self.rendered_body(f.alternate()))
    }
}

/// A rendered value inside a one-field tuple variant, such as `Some(value)`.
///
/// Created inside [`Typed`] by [`RenderingContext::variant`]. The outer adapter retains the
/// owner's type, while this body renders only the field through `R: ValueRenderer<T>`.
pub struct Variant<'a, T: ?Sized, R> {
    name: &'static str,
    value: Typed<RenderedValue<'a, T, R>>,
}

impl<T: ?Sized, R: ValueRenderer<T>> Debug for Variant<'_, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_body(f, self.rendered_body(f.alternate()))
    }
}

/// A rendered value inside a one-field struct, such as `RefCell { value: 1 }`.
///
/// Created inside [`Typed`] by [`RenderingContext::struct_field`]. The outer adapter retains the
/// owner's type, while this body renders only the field through `R: ValueRenderer<T>`.
pub struct StructField<'a, T: ?Sized, R> {
    name: &'static str,
    field: &'static str,
    value: Typed<RenderedValue<'a, T, R>>,
}

impl<T: ?Sized, R: ValueRenderer<T>> Debug for StructField<'_, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_body(f, self.rendered_body(f.alternate()))
    }
}

/// A one-field struct whose field is temporarily inaccessible, such as `Mutex { data: <locked> }`.
///
/// Created inside [`Typed`] by [`RenderingContext::unavailable_struct_field`]. It requires no
/// leaf renderer and retains no inferred type for the inaccessible field.
pub struct UnavailableStructField {
    name: &'static str,
    field: &'static str,
    unavailable: &'static str,
}

impl Debug for UnavailableStructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_body(f, self.rendered_body(f.alternate()))
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

impl<T: ?Sized, C: ?Sized, R> RenderedValues<'_, T, C, R> {
    /// Selects the diagnostic order for this synthetic group.
    ///
    /// The default is [`RenderingOrder::PreserveIteration`]. Sorting uses rendered text, including
    /// leaf truncation, before applying the item limit and marks the resulting group as sorted.
    /// This setting grants no positional capability. Use a stable collection adapter for evidence
    /// whose positions have semantic meaning.
    #[must_use]
    pub fn with_order(mut self, order: RenderingOrder) -> Self {
        self.sorted_for_rendering = order == RenderingOrder::SortByRenderedText;
        self
    }
}

impl<T: ?Sized, C: Collection + ?Sized, R: ValueRenderer<T>> Debug for RenderedValues<'_, T, C, R>
where
    C::Item: Borrow<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_body(f, self.rendered_body(f.alternate()))
    }
}

/// Rendered key/value entries in map syntax, including the key and value type information.
///
/// Created inside [`Typed`] by [`RenderingContext::map`]. Formatting traverses the borrowed map
/// with its rendering order and budget, requiring renderers only for its keys and values.
pub struct MapEntries<'a, M: ?Sized, R> {
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
        write_body(f, self.rendered_body(f.alternate()))
    }
}

/// Rendered key/value entries as a synthetic list of tuples, including the child type information.
///
/// Created by [`RenderingContext::entry_list`]. Formatting traverses the borrowed collection with
/// the selected rendering order and budget. The list has no invented outer Rust type.
pub struct EntryList<'a, K: ?Sized, V: ?Sized, C: ?Sized, R> {
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
        write_body(f, self.rendered_body(f.alternate()))
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

/// The marker appended to a leaf whose value renderer reported a formatting error.
const RENDERER_ERROR: &str = "<renderer error>";

/// Renders one leaf within the leaf budget, returning the retained text and omitted character
/// count.
///
/// [`BoundedOutput`] never fails, so an error can only originate from the value renderer. The text
/// written before the error is kept and followed by [`RENDERER_ERROR`], so a faulty renderer never
/// turns a diagnostic into a panic.
fn render_leaf<T: ?Sized, R: ValueRenderer<T>>(
    value: &T,
    rendering: RenderingContext<'_, R>,
    alternate: bool,
) -> (String, usize) {
    let value = FormatterFn(|f: &mut fmt::Formatter<'_>| rendering.renderer.fmt(value, f));
    let mut output = BoundedOutput::new(rendering.budget.max_leaf_characters());
    let result = if alternate {
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

trait BuildRenderedBody {
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody;
}

trait BuildRendered {
    fn rendered(&self, pretty_leaves: bool) -> Rendered;
}

impl<D: BuildRenderedBody> BuildRendered for Typed<D> {
    fn rendered(&self, pretty_leaves: bool) -> Rendered {
        Rendered::typed(
            self.body.rendered_body(pretty_leaves),
            self.info.type_name,
            self.info.hint,
            self.show_type_hint,
        )
    }
}

impl<D: BuildRenderedBody> IntoRendered for Typed<D> {
    fn into_rendered(self) -> Rendered {
        self.rendered(true)
    }

    fn into_rendered_compact(self) -> Rendered {
        self.rendered(false)
    }
}

impl<T: ?Sized, R: ValueRenderer<T>> BuildRenderedBody for RenderedValue<'_, T, R> {
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        let (text, omitted_characters) = render_leaf(self.value, self.rendering, pretty_leaves);
        RenderedBody::Text {
            text,
            omitted_characters,
        }
    }
}

impl<T: ?Sized, R: ValueRenderer<T>> BuildRenderedBody for Variant<'_, T, R> {
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        RenderedBody::Variant {
            name: self.name,
            value: Box::new(self.value.rendered(pretty_leaves)),
        }
    }
}

impl<T: ?Sized, R: ValueRenderer<T>> BuildRenderedBody for StructField<'_, T, R> {
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        RenderedBody::Struct {
            name: self.name,
            fields: alloc::vec![(self.field, self.value.rendered(pretty_leaves))],
        }
    }
}

impl BuildRenderedBody for UnavailableStructField {
    fn rendered_body(&self, _pretty_leaves: bool) -> RenderedBody {
        RenderedBody::Struct {
            name: self.name,
            fields: alloc::vec![(
                self.field,
                Rendered {
                    body: RenderedBody::Placeholder(self.unavailable),
                    type_name: None,
                    hint: super::TypeHint::Short,
                    shows_type_hint: false,
                    compact: false,
                },
            )],
        }
    }
}

impl<T: ?Sized, C: Collection + ?Sized, R: ValueRenderer<T>> BuildRenderedBody
    for RenderedValues<'_, T, C, R>
where
    C::Item: Borrow<T>,
{
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        let maximum = self.rendering.max_items();
        let candidates = self.items.elements().map(|value| {
            self.rendering
                .value_with_type_info(value.borrow(), self.item_type)
                .rendered(pretty_leaves)
        });
        let items = if self.sorted_for_rendering {
            select_smallest(candidates, maximum, |item| item.text(pretty_leaves))
        } else {
            candidates.take(maximum).collect()
        };
        RenderedBody::Group {
            style: self.style,
            items,
            omitted: self.items.length().saturating_sub(maximum),
            sorted: self.sorted_for_rendering,
        }
    }
}

impl<T: ?Sized, C: Collection + ?Sized, R: ValueRenderer<T>> IntoRendered
    for RenderedValues<'_, T, C, R>
where
    C::Item: Borrow<T>,
{
    fn into_rendered(self) -> Rendered {
        untyped(self.rendered_body(true))
    }

    fn into_rendered_compact(self) -> Rendered {
        untyped(self.rendered_body(false))
    }
}

impl<M: Map + ?Sized, R> BuildRenderedBody for MapEntries<'_, M, R>
where
    R: ValueRenderer<M::Key> + ValueRenderer<M::Value>,
{
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        let maximum = self.rendering.max_items();
        let candidates = self.map.entries().map(|(key, value)| {
            (
                self.rendering
                    .value_with_type_info(key, self.key_type)
                    .rendered(pretty_leaves),
                self.rendering
                    .value_with_type_info(value, self.value_type)
                    .rendered(pretty_leaves),
            )
        });
        let entries = if self.sorted_for_rendering {
            select_smallest(candidates, maximum, |(key, value)| {
                (key.text(pretty_leaves), value.text(pretty_leaves))
            })
        } else {
            candidates.take(maximum).collect()
        };
        RenderedBody::Map {
            entries,
            omitted: self.map.length().saturating_sub(maximum),
            sorted: self.sorted_for_rendering,
        }
    }
}

impl<K: ?Sized, V: ?Sized, BK, BV, C: Collection<Item = (BK, BV)> + ?Sized, R> BuildRenderedBody
    for EntryList<'_, K, V, C, R>
where
    BK: Borrow<K>,
    BV: Borrow<V>,
    R: ValueRenderer<K> + ValueRenderer<V>,
{
    fn rendered_body(&self, pretty_leaves: bool) -> RenderedBody {
        let maximum = self.rendering.max_items();
        let candidates = self.entries.elements().map(|(key, value)| {
            (
                self.rendering
                    .value_with_type_info(key.borrow(), self.key_type)
                    .rendered(pretty_leaves),
                self.rendering
                    .value_with_type_info(value.borrow(), self.value_type)
                    .rendered(pretty_leaves),
            )
        });
        let entries = if self.sorted_for_rendering {
            select_smallest(candidates, maximum, |(key, value)| {
                tuple_text(key, value, pretty_leaves)
            })
        } else {
            candidates.take(maximum).collect()
        };
        RenderedBody::EntryList {
            entries,
            omitted: self.entries.length().saturating_sub(maximum),
            sorted: self.sorted_for_rendering,
        }
    }
}

impl<K: ?Sized, V: ?Sized, BK, BV, C: Collection<Item = (BK, BV)> + ?Sized, R> IntoRendered
    for EntryList<'_, K, V, C, R>
where
    BK: Borrow<K>,
    BV: Borrow<V>,
    R: ValueRenderer<K> + ValueRenderer<V>,
{
    fn into_rendered(self) -> Rendered {
        untyped(self.rendered_body(true))
    }

    fn into_rendered_compact(self) -> Rendered {
        untyped(self.rendered_body(false))
    }
}

fn untyped(body: RenderedBody) -> Rendered {
    Rendered {
        body,
        type_name: None,
        hint: super::TypeHint::Short,
        shows_type_hint: false,
        compact: false,
    }
}

fn write_body(f: &mut fmt::Formatter<'_>, body: RenderedBody) -> fmt::Result {
    let pretty = f.alternate();
    untyped(body).write(f, pretty)
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

    use crate::{
        prelude::*,
        renderer::{GroupStyle, IntoRendered, Rendered, RenderedBody, RenderingOrder, TypeHint},
        test_support::{PreservedBag, UnorderedMap, UnorderedSet},
    };

    use super::{
        BuildRendered, BuildRenderedBody, RenderingContext, grouped_count, omission, untyped,
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
        assert_that!(value.type_name()).is_equal_to(Some(type_name::<T>()));
        assert_that!(value.hint).is_equal_to(TypeHint::Short);
        assert_that!(value.shows_type_hint).is_equal_to(shown);
    }

    fn leaf<T: ?Sized>(text: &str) -> Rendered {
        Rendered::typed(
            RenderedBody::Text {
                text: text.into(),
                omitted_characters: 0,
            },
            type_name::<T>(),
            TypeHint::Short,
            false,
        )
    }

    mod bounded_sorting {
        use super::super::tuple_text;
        use super::*;
        use crate::renderer::Rendered;
        use core::cell::Cell;

        // The old adapter algorithm, applied to trees built by iteration-preserving paths.
        fn full_sort(mut tree: Rendered, limit: usize, pretty: bool) -> Rendered {
            match &mut tree.body {
                RenderedBody::Group {
                    items,
                    omitted,
                    sorted,
                    ..
                } => {
                    items.sort_by_cached_key(|item| item.text(pretty));
                    *omitted = items.len().saturating_sub(limit);
                    items.truncate(limit);
                    *sorted = true;
                }
                RenderedBody::Map {
                    entries,
                    omitted,
                    sorted,
                } => {
                    entries
                        .sort_by_cached_key(|(key, value)| (key.text(pretty), value.text(pretty)));
                    *omitted = entries.len().saturating_sub(limit);
                    entries.truncate(limit);
                    *sorted = true;
                }
                RenderedBody::EntryList {
                    entries,
                    omitted,
                    sorted,
                } => {
                    entries.sort_by_cached_key(|(key, value)| tuple_text(key, value, pretty));
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
            build: impl Fn(RenderingContext<'_, StatefulRenderer>, RenderingOrder, bool) -> Rendered,
        ) {
            for leaf_limit in [0, 1, 2, usize::MAX] {
                for limit in [0, 1, 2, 4, 5, 6, usize::MAX] {
                    for pretty in [false, true] {
                        let budget =
                            RenderingBudget::unlimited().with_max_leaf_characters(leaf_limit);
                        let renderer = StatefulRenderer(Cell::new(0));
                        let actual = build(
                            RenderingContext::new(&renderer, budget.with_max_items(limit)),
                            RenderingOrder::SortByRenderedText,
                            pretty,
                        );
                        // In particular, losing map entries still render both leaves in order.
                        assert_that!(renderer.0.get()).is_equal_to(if limit == 0 {
                            0
                        } else {
                            leaves
                        });
                        let renderer = StatefulRenderer(Cell::new(0));
                        let reference = build(
                            RenderingContext::new(&renderer, budget),
                            RenderingOrder::PreserveIteration,
                            pretty,
                        );
                        let reference = full_sort(reference, limit, pretty);
                        assert_that!(actual).is_equal_to(&reference);
                        for style in [false, true] {
                            assert_that!(actual.text(style)).is_equal_to(reference.text(style));
                        }
                    }
                }
            }
        }
    }

    mod budget {
        use super::*;

        #[test]
        fn returns_an_independent_copy_of_both_limits() {
            const RENDERING: RenderingContext<'_, DebugRenderer> = RenderingContext::new(
                &DebugRenderer,
                RenderingBudget::unlimited()
                    .with_max_items(2)
                    .with_max_leaf_characters(3),
            );
            const BUDGET: RenderingBudget = RENDERING.budget();
            let changed = BUDGET.with_max_items(9).with_max_leaf_characters(10);

            assert_that!(RENDERING.budget()).is_equal_to(BUDGET);
            assert_that!(BUDGET.max_items()).is_equal_to(2);
            assert_that!(BUDGET.max_leaf_characters()).is_equal_to(3);
            assert_that!(changed.max_items()).is_equal_to(9);
            assert_that!(changed.max_leaf_characters()).is_equal_to(10);
        }
    }

    mod with_order {
        use super::*;

        #[test]
        fn selects_rendered_order_before_the_item_limit_and_can_restore_iteration() {
            let rendering = with_max_items(&DebugRenderer, 2);
            let values = [30, 2, 10];
            let sorted = rendering
                .values(&values, GroupStyle::Set)
                .with_order(RenderingOrder::SortByRenderedText);

            assert_that!(format!("{sorted:?}"))
                .is_equal_to("{10, 2} (... 1 more element ...) (sorted for rendering)");
            assert_that!(format!(
                "{:?}",
                sorted.with_order(RenderingOrder::PreserveIteration)
            ))
            .is_equal_to("{30, 2} (... 1 more element ...)");
        }

        #[test]
        fn sorts_the_budgeted_leaf_text_including_omission_markers() {
            let rendering = RenderingContext::new(
                &RawRenderer,
                RenderingBudget::unlimited()
                    .with_max_items(1)
                    .with_max_leaf_characters(1),
            );
            // Complete text would put "aaz" first. Its larger omission count sorts after "ab"
            // once both leaves have been truncated to one character.
            let values = ["aaz", "ab"];
            let rendered = rendering
                .borrowed_values::<str, _>(&values, GroupStyle::List)
                .with_order(RenderingOrder::SortByRenderedText)
                .into_rendered();

            assert_that!(rendered.text(false)).is_equal_to(
                "[a... 1 more character ...] (... 1 more element ...) (sorted for rendering)",
            );
        }

        #[test]
        fn empty_and_zero_limit_groups_do_not_render_leaves() {
            let rendering = with_max_items(&PanickingRenderer, 0);
            for values in [&[][..], &[1, 2][..]] {
                let rendered = rendering
                    .values(values, GroupStyle::List)
                    .with_order(RenderingOrder::SortByRenderedText)
                    .into_rendered();
                let RenderedBody::Group {
                    items,
                    omitted,
                    sorted,
                    ..
                } = rendered.body
                else {
                    panic!("expected a group");
                };
                assert_that!(items).is_empty();
                assert_that!(omitted).is_equal_to(values.len());
                assert_that!(sorted).is_true();
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
            assert_that!(omission(2, "entry")).is_equal_to("... 2 more entries ...");
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

            assert_that!(rendering.value(&1).into_rendered().body).is_equal_to(
                RenderedBody::Text {
                    text: "partial<renderer error>".into(),
                    omitted_characters: 0,
                },
            );
        }

        #[test]
        fn do_not_panic_while_capturing_a_failure() {
            let failures = assert_that!(1)
                .with_debug_format(|_, _| Err(fmt::Error))
                .capture(|it| it.is_equal_to(2));

            assert_that!(failures).has_length(1);
            assert_that!(ToHumanReadableText.render(&failures[0])).contains("<renderer error>");
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
                RenderingBudget::default().with_max_leaf_characters(2),
            );

            assert_that!(format!("{:?}", rendering.value("é😊x")))
                .is_equal_to("é😊... 1 more character ...");
        }

        #[test]
        fn does_not_count_the_type_hint_as_part_of_the_leaf() {
            let renderer = RawRenderer;
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(2),
            );

            let value = rendering.value("é😊x").show_type_hint(true);
            assert_that!(format!("{value:?}")).is_equal_to("str é😊... 1 more character ...");
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
                RenderingBudget::default().with_max_leaf_characters(2),
            );
            let map = BTreeMap::from([(1_234, 5_678)]);
            let list_entries = [(1_234, 5_678)];

            assert_that!(format!("{:?}", rendering.map(&map)))
                .is_equal_to("BTreeMap {12... 2 more characters ...: 56... 2 more characters ...}");
            assert_that!(format!(
                "{:?}",
                rendering.entry_list::<i32, i32, _, _, _>(
                    &list_entries,
                    RenderingOrder::PreserveIteration
                )
            ))
            .is_equal_to("[(12... 2 more characters ..., 56... 2 more characters ...)]");
        }
    }

    mod wrappers {
        use super::*;

        #[test]
        fn limits_unicode_leaves_without_limiting_structural_names() {
            let rendering = RenderingContext::new(
                &RawRenderer,
                RenderingBudget::unlimited()
                    .with_max_items(0)
                    .with_max_leaf_characters(2),
            );
            let owner = Some("é😊x");
            assert_that!(format!("{:?}", rendering.variant(&owner, "Some", "é😊x")))
                .is_equal_to("Some(é😊... 1 more character ...)");
            assert_that!(format!(
                "{:?}",
                rendering.struct_field(&owner, "Wrapper", "value", "é😊x")
            ))
            .is_equal_to("Wrapper { value: é😊... 1 more character ... }");
            assert_that!(format!(
                "{:?}",
                rendering.unavailable_struct_field(&owner, "Wrapper", "value", "<unavailable>")
            ))
            .is_equal_to("Wrapper { value: <unavailable> }");
        }

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
        fn retain_owner_and_leaf_metadata_in_owned_nodes() {
            let rendering =
                RenderingContext::new(&AlternateAwareRenderer, RenderingBudget::unlimited());
            let value = 7;
            let result = Result::<(), i32>::Err(value);
            let cell = RefCell::new(value);

            let variant = rendering.variant(&result, "Err", &value).into_rendered();
            check_type::<Result<(), i32>>(&variant, false);
            assert_that!(variant.body).is_equal_to(RenderedBody::Variant {
                name: "Err",
                value: Box::new(leaf::<i32>("pretty(7)")),
            });

            let field = rendering
                .struct_field(&cell, "RefCell", "value", &value)
                .into_rendered();
            check_type::<RefCell<i32>>(&field, false);
            assert_that!(field.body).is_equal_to(RenderedBody::Struct {
                name: "RefCell",
                fields: vec![("value", leaf::<i32>("pretty(7)"))],
            });

            let unavailable = rendering
                .unavailable_struct_field(&cell, "RefCell", "value", "<borrowed>")
                .into_rendered();
            check_type::<RefCell<i32>>(&unavailable, false);
            let RenderedBody::Struct { name, fields } = &unavailable.body else {
                panic!("expected a struct");
            };
            assert_that!(*name).is_equal_to("RefCell");
            assert_that!(fields).has_length(1);
            assert_that!(fields[0].0).is_equal_to("value");
            assert_that!(fields[0].1.type_name()).is_none();
            assert_that!(fields[0].1.shows_type_hint).is_false();
            assert_that!(&fields[0].1.body).is_equal_to(RenderedBody::Placeholder("<borrowed>"));
        }
    }

    mod values {
        use super::*;
        use crate::{
            assertions::{HasLength, collection::Collection},
            renderer::CollectionPresentation,
        };
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
            bounded_sorting::assert_full_sort(5, |rendering, order, pretty| {
                untyped(
                    rendering
                        .values(&[11, 10, 2, 10, -1], GroupStyle::Set)
                        .with_order(order)
                        .rendered_body(pretty),
                )
            });
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
            let rendering = RenderingContext::new(&RawRenderer, RenderingBudget::unlimited());
            let values = vec![String::from("alpha"), String::from("beta")];

            let rendered = rendering
                .borrowed_values::<str, _>(&values, GroupStyle::List)
                .into_rendered();
            assert_that!(rendered.type_name()).is_none();
            assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::List,
                items: vec![leaf::<str>("alpha"), leaf::<str>("beta")],
                omitted: 0,
                sorted: false,
            });
        }

        #[test]
        fn synthetic_groups_retain_leaf_metadata_without_an_outer_type() {
            let rendering = with_max_items(&DebugRenderer, 1);
            let values = [2, 1];
            let rendered = rendering
                .values(&values, GroupStyle::Set)
                .with_order(RenderingOrder::SortByRenderedText)
                .into_rendered();

            assert_that!(rendered.type_name()).is_none();
            assert_that!(rendered.shows_type_hint).is_false();
            assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::Set,
                items: vec![leaf::<i32>("1")],
                omitted: 1,
                sorted: true,
            });
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
        fn retain_collection_and_leaf_metadata_in_owned_groups() {
            let rendering = with_max_items(&DebugRenderer, 2);
            let tree = rendering
                .collection(&BTreeSet::from([3, 1, 2]))
                .into_rendered();
            check_type::<BTreeSet<i32>>(&tree, true);
            assert_that!(tree.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::Set,
                items: vec![leaf::<i32>("1"), leaf::<i32>("2")],
                omitted: 1,
                sorted: false,
            });
            let unordered = rendering
                .collection(&UnorderedSet(vec![3, 1, 2]))
                .into_rendered();
            check_type::<UnorderedSet>(&unordered, true);
            assert_that!(unordered.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::Set,
                items: vec![leaf::<i32>("1"), leaf::<i32>("2")],
                omitted: 1,
                sorted: true,
            });
            #[cfg(feature = "std")]
            {
                let hashed = rendering
                    .collection(&std::collections::HashSet::from([3, 1, 2]))
                    .into_rendered();
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
                    .show_type_hint(true)
                    .with_order(RenderingOrder::SortByRenderedText);

            fn elements(&self) -> impl Iterator<Item = &T> {
                self.0.iter()
            }
        }

        impl<T> crate::assertions::collection::StableOrder for SortedSequence<T> {}

        #[test]
        fn borrowed_and_stable_views_preserve_metadata_but_select_their_own_order() {
            let rendering = with_max_items(&RawRenderer, 2);
            let values = SortedSequence(vec![
                String::from("z"),
                String::from("a"),
                String::from("m"),
            ]);
            for (rendered, expected, sorted) in [
                (
                    rendering
                        .borrowed_collection::<str, _>(&values)
                        .into_rendered(),
                    ["a", "m"],
                    true,
                ),
                (
                    rendering
                        .stable_borrowed_collection::<str, _>(&values)
                        .into_rendered(),
                    ["z", "a"],
                    false,
                ),
            ] {
                check_type::<SortedSequence<String>>(&rendered, true);
                assert_that!(rendered.body).is_equal_to(RenderedBody::Group {
                    style: GroupStyle::List,
                    items: expected.map(leaf::<str>).into(),
                    omitted: 1,
                    sorted,
                });
            }
            let rendering = with_max_items(&DebugRenderer, 2);
            let values = SortedSequence(vec![3, 1, 2]);
            let stable = rendering
                .stable_collection(&values)
                .show_type_hint(false)
                .into_rendered();
            check_type::<SortedSequence<i32>>(&stable, false);
            assert_that!(stable.body).is_equal_to(RenderedBody::Group {
                style: GroupStyle::List,
                items: vec![leaf::<i32>("3"), leaf::<i32>("1")],
                omitted: 1,
                sorted: false,
            });
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
            bounded_sorting::assert_full_sort(10, |rendering, order, pretty| {
                let map = UnorderedMap(vec![(11, 2), (10, 20), (10, 11), (10, 11), (-1, 9)]);
                let mut adapter = rendering.map(&map);
                adapter.body.sorted_for_rendering = order == RenderingOrder::SortByRenderedText;
                adapter.rendered(pretty)
            });
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
        fn retain_map_and_entry_metadata_in_owned_nodes() {
            let rendering = with_max_items(&DebugRenderer, 1);
            let tree = rendering
                .map(&BTreeMap::from([(2, "two"), (1, "one")]))
                .into_rendered();
            check_type::<BTreeMap<i32, &str>>(&tree, true);
            assert_that!(tree.body).is_equal_to(RenderedBody::Map {
                entries: vec![(leaf::<i32>("1"), leaf::<&str>("\"one\""))],
                omitted: 1,
                sorted: false,
            });
            let unordered = rendering
                .map(&UnorderedMap(vec![(2, 20), (1, 10)]))
                .into_rendered();
            check_type::<UnorderedMap>(&unordered, true);
            assert_that!(unordered.body).is_equal_to(RenderedBody::Map {
                entries: vec![(leaf::<i32>("1"), leaf::<i32>("10"))],
                omitted: 1,
                sorted: true,
            });
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
        use crate::{
            assertions::{HasLength, collection::Collection},
            renderer::CollectionPresentation,
        };
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
            bounded_sorting::assert_full_sort(10, |rendering, order, pretty| {
                untyped(
                    rendering
                        .entry_list::<i32, i32, _, _, _>(
                            &[(11, 2), (10, 20), (10, 11), (10, 11), (-1, 9)],
                            order,
                        )
                        .rendered_body(pretty),
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
                    .into_rendered()
                    .text(false)
            )
            .is_equal_to("[(a!, z), (a, b)] (... 1 more entry ...) (sorted for rendering)");
            let renderer = DebugRenderer;
            let rendering = with_max_items(&renderer, 2);
            let entries = [(3, 30), (1, 10), (2, 20)];

            assert_that!(format!(
                "{:?}",
                rendering
                    .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::PreserveIteration)
            ))
            .is_equal_to("[(3, 30), (1, 10)] (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering
                    .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::SortByRenderedText)
            ))
            .is_equal_to("[(1, 10), (2, 20)] (... 1 more entry ...) (sorted for rendering)");
        }

        #[test]
        fn retain_borrowed_entry_metadata_without_an_outer_type() {
            let rendering = with_max_items(&DebugRenderer, 1);
            let entries = [(2, String::from("two")), (1, String::from("one"))];
            for (order, key, value) in [
                (RenderingOrder::PreserveIteration, "2", "\"two\""),
                (RenderingOrder::SortByRenderedText, "1", "\"one\""),
            ] {
                let rendered = rendering
                    .entry_list::<i32, str, _, _, _>(&entries, order)
                    .into_rendered();
                assert_that!(rendered.type_name()).is_none();
                assert_that!(rendered.shows_type_hint).is_false();
                assert_that!(rendered.body).is_equal_to(RenderedBody::EntryList {
                    entries: vec![(leaf::<i32>(key), leaf::<str>(value))],
                    omitted: 1,
                    sorted: order == RenderingOrder::SortByRenderedText,
                });
            }
        }

        #[test]
        fn retain_and_reiterate_any_collection_without_collecting_its_entries() {
            let renderer = DebugRenderer;
            let rendering = RenderingContext::new(&renderer, RenderingBudget::unlimited());
            let entries = ObservedEntries {
                entries: vec![(1, 10), (2, 20)],
                iterations: Cell::new(0),
            };

            let adapted = rendering
                .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::PreserveIteration);
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
                .is_equal_to("UnorderedMap {} (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering
                    .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::PreserveIteration)
            ))
            .is_equal_to("[] (... 1 more entry ...)");
            assert_that!(format!(
                "{:?}",
                rendering
                    .entry_list::<i32, i32, _, _, _>(&entries, RenderingOrder::SortByRenderedText)
            ))
            .is_equal_to("[] (... 1 more entry ...)");
        }
    }
}
