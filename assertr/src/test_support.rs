use alloc::vec::Vec;
use core::fmt;

use crate::{
    AssertThat, AssertionFailure, AssertrPartialEq, EqContext, Mode, ValueRenderer,
    assertions::{HasLength, collection::Collection, map::Map, set::SetLookup},
    failure::{FailureKind, adapter::ToHumanReadableText},
    renderer::Rendered,
    renderer::{CollectionPresentation, RenderingOrder},
};

pub(crate) fn rendered_text(value: &Rendered) -> alloc::string::String {
    let mut text = alloc::string::String::new();
    value
        .write(&mut text, true)
        .expect("writing a rendered value to a String cannot fail");
    text
}

pub(crate) trait FailureReportAssertions {
    fn has_text_report(self, expected: impl AsRef<str>) -> Self;
}

impl<M: Mode, R> FailureReportAssertions for AssertThat<'_, AssertionFailure, M, R> {
    #[track_caller]
    fn has_text_report(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let report = ToHumanReadableText.render(self.actual());
        let expected = expected.as_ref();
        if report != expected {
            self.failure(FailureKind::Equality)
                .actual(format_args!("{report:?}"))
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }
}

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

pub(crate) struct NoRenderer;

pub(crate) const SENTINEL: &str = "<rendered>";

#[derive(Clone, Copy)]
pub(crate) struct SentinelRenderer;

impl<T: ?Sized> ValueRenderer<T> for SentinelRenderer {
    fn fmt(&self, _value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(SENTINEL)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RendererActual(pub(crate) u32);

#[derive(Clone, Copy)]
pub(crate) struct RendererExpected(pub(crate) u32);

impl AssertrPartialEq<RendererExpected, SentinelRenderer> for RendererActual {
    fn eq(
        &self,
        other: &RendererExpected,
        _ctx: Option<&mut EqContext<'_, SentinelRenderer>>,
    ) -> bool {
        self.0 == other.0
    }
}

macro_rules! assert_trait_impl {
    ($type:ty => $trait:path) => {{
        fn assert_implemented<T: $trait>() {}
        assert_implemented::<$type>();
    }};
}

pub(crate) use assert_trait_impl;
