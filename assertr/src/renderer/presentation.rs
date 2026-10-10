/// The structural syntax used to render a group of diagnostic values.
///
/// Collection subjects obtain their syntax from [`CollectionPresentation`]. Synthetic groups use
/// list syntax.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GroupStyle {
    /// Render the values with list delimiters, e.g. `[1, 2]`.
    List,
    /// Render the values with set delimiters, e.g. `{1, 2}`.
    Set,
}

/// The order in which repeated items are shown in diagnostics.
///
/// This is a presentation choice, not a behavioral capability. Sorting uses the final rendered text
/// of each item. Assertions whose meaning depends on
/// [`StableOrder`](crate::assertions::StableOrder) always render the subject in
/// iteration order so displayed positions retain their meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderingOrder {
    /// Preserve the container's iteration order.
    PreserveIteration,
    /// Sort items by their rendered text and mark the diagnostic as sorted for rendering.
    SortByRenderedText,
}

/// Presentation metadata for a [`Collection`](crate::assertions::Collection).
///
/// This value controls only diagnostic syntax, type-hint visibility, and rendering order.
/// Positional APIs are controlled independently by
/// [`StableOrder`](crate::assertions::StableOrder) and
/// [`RandomAccess`](crate::assertions::RandomAccess).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectionPresentation {
    style: GroupStyle,
    shows_type_hint: bool,
    order: RenderingOrder,
}

impl CollectionPresentation {
    /// Creates presentation metadata using list syntax, with no type hint and preserved order.
    #[must_use]
    pub const fn list() -> Self {
        Self {
            style: GroupStyle::List,
            shows_type_hint: false,
            order: RenderingOrder::PreserveIteration,
        }
    }

    /// Creates presentation metadata using set syntax, with no type hint and preserved order.
    #[must_use]
    pub const fn set() -> Self {
        Self {
            style: GroupStyle::Set,
            ..Self::list()
        }
    }

    /// Selects whether the rendered collection shows its short Rust type hint, such as `BTreeSet`.
    ///
    /// Query the setting with [`shows_type_hint`](Self::shows_type_hint).
    ///
    /// ```
    /// use assertr::{prelude::*, renderer::CollectionPresentation};
    ///
    /// const PRESENTATION: CollectionPresentation = CollectionPresentation::set().with_type_hint(true);
    /// assert_that!(PRESENTATION.shows_type_hint()).is_true();
    /// ```
    #[must_use]
    pub const fn with_type_hint(mut self, show: bool) -> Self {
        self.shows_type_hint = show;
        self
    }

    /// Selects whether rendering preserves iteration order or sorts by rendered text.
    #[must_use]
    pub const fn with_order(mut self, order: RenderingOrder) -> Self {
        self.order = order;
        self
    }

    /// Returns the collection's diagnostic group syntax.
    #[must_use]
    pub const fn style(self) -> GroupStyle {
        self.style
    }

    /// Returns whether diagnostics show the collection's short Rust type hint.
    #[must_use]
    pub const fn shows_type_hint(self) -> bool {
        self.shows_type_hint
    }

    /// Returns the order in which diagnostics render the collection's elements.
    #[must_use]
    pub const fn order(self) -> RenderingOrder {
        self.order
    }
}
