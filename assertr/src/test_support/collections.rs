//! Capability fixtures available with or without the library's `std` feature.

use crate::{
    assertions::{HasLength, collection::Collection, map::Map, set::SetLookup},
    renderer::{CollectionPresentation, RenderingOrder},
};
use alloc::vec::Vec;

/// A set without a deterministic iteration order, like a `HashSet`, that is available in every
/// feature configuration.
pub(crate) struct UnorderedSet(pub(crate) Vec<i32>);

impl HasLength for UnorderedSet {
    fn length(&self) -> usize {
        self.0.len()
    }
}

impl Collection for UnorderedSet {
    type Item = i32;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set()
        .with_type_hint()
        .with_order(RenderingOrder::SortByRenderedText);

    fn elements(&self) -> impl Iterator<Item = &i32> {
        self.0.iter()
    }
}

impl SetLookup for UnorderedSet {
    fn contains_element(&self, element: &i32) -> bool {
        self.0.contains(element)
    }
}

/// An order-free collection whose diagnostic presentation preserves iteration order.
pub(crate) struct PreservedBag(pub(crate) Vec<i32>);

impl HasLength for PreservedBag {
    fn length(&self) -> usize {
        self.0.len()
    }
}

impl Collection for PreservedBag {
    type Item = i32;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list().with_type_hint();

    fn elements(&self) -> impl Iterator<Item = &i32> {
        self.0.iter()
    }
}

/// A map without a deterministic iteration order, like a `HashMap`, that is available in every
/// feature configuration.
pub(crate) struct UnorderedMap(pub(crate) Vec<(i32, i32)>);

impl HasLength for UnorderedMap {
    fn length(&self) -> usize {
        self.0.len()
    }
}

impl Map for UnorderedMap {
    type Key = i32;
    type Value = i32;
    const RENDERING_ORDER: RenderingOrder = RenderingOrder::SortByRenderedText;

    fn entries(&self) -> impl Iterator<Item = (&i32, &i32)> {
        self.0.iter().map(|(key, value)| (key, value))
    }
}
