/// The structural syntax used to render a group of diagnostic values.
///
/// Collection subjects obtain their syntax from [`CollectionPresentation`]. Custom assertions
/// and equality implementations pass this style directly when rendering an ad-hoc group.
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
/// This is a presentation choice, not a behavioral capability. Sorting uses the final rendered
/// text of each item. Assertions whose meaning depends on [`StableOrder`](crate::assertions::collection::StableOrder)
/// always render the subject in iteration order so displayed positions retain their meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderingOrder {
    /// Preserve the container's iteration order.
    PreserveIteration,
    /// Sort items by their rendered text and mark the diagnostic as sorted for rendering.
    SortByRenderedText,
}

/// Presentation metadata for a [`Collection`](crate::assertions::collection::Collection).
///
/// This value controls only diagnostic syntax, type-hint visibility, and rendering order.
/// Positional APIs are controlled independently by
/// [`StableOrder`](crate::assertions::collection::StableOrder) and
/// [`RandomAccess`](crate::assertions::collection::RandomAccess).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectionPresentation {
    style: GroupStyle,
    show_type_hint: bool,
    order: RenderingOrder,
}

impl CollectionPresentation {
    /// Creates presentation metadata using list syntax, with no type hint and preserved order.
    #[must_use]
    pub const fn list() -> Self {
        Self {
            style: GroupStyle::List,
            show_type_hint: false,
            order: RenderingOrder::PreserveIteration,
        }
    }

    /// Creates presentation metadata using set syntax, with no type hint and preserved order.
    #[must_use]
    pub const fn set() -> Self {
        Self {
            style: GroupStyle::Set,
            show_type_hint: false,
            order: RenderingOrder::PreserveIteration,
        }
    }

    /// Makes the rendered collection show its short Rust type hint.
    #[must_use]
    pub const fn with_type_hint(mut self) -> Self {
        self.show_type_hint = true;
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
        self.show_type_hint
    }

    /// Returns the order in which diagnostics render the collection's elements.
    #[must_use]
    pub const fn order(self) -> RenderingOrder {
        self.order
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::{CollectionPresentation, GroupStyle, RenderingOrder};

    #[test]
    fn list_and_set_defaults_preserve_order_and_hide_the_type_hint() {
        let list = CollectionPresentation::list();
        assert_that!(list.style()).is_equal_to(GroupStyle::List);
        assert_that!(list.shows_type_hint()).is_false();
        assert_that!(list.order()).is_equal_to(RenderingOrder::PreserveIteration);

        let set = CollectionPresentation::set();
        assert_that!(set.style()).is_equal_to(GroupStyle::Set);
        assert_that!(set.shows_type_hint()).is_false();
        assert_that!(set.order()).is_equal_to(RenderingOrder::PreserveIteration);
    }

    #[test]
    fn builders_change_only_the_selected_presentation_property() {
        let presentation = CollectionPresentation::list()
            .with_type_hint()
            .with_order(RenderingOrder::SortByRenderedText);

        assert_that!(presentation.style()).is_equal_to(GroupStyle::List);
        assert_that!(presentation.shows_type_hint()).is_true();
        assert_that!(presentation.order()).is_equal_to(RenderingOrder::SortByRenderedText);
    }
}
