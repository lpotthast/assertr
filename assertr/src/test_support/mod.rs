//! Shared test assertions, renderers, and capability fixtures.

mod assertions;
mod caller_location;
mod collections;
#[cfg(not(feature = "std"))]
mod panic;
mod rendering;

pub(crate) use assertions::{FailureReportAssertions, assert_trait_impl};
#[cfg(feature = "std")]
pub(crate) use caller_location::block_on;
pub(crate) use caller_location::{assert_caller_location, check_caller_location};
pub(crate) use collections::{PreservedBag, UnorderedMap, UnorderedSet};
#[cfg(not(feature = "std"))]
pub(crate) use panic::assert_that_panic_by;
pub(crate) use rendering::{
    ComparisonRenderer, CustomValueRenderer, NoRenderer, NumericRenderer, RedactingRenderer,
    RendererActual, RendererExpected, SENTINEL, SentinelRenderer, assert_custom_fact,
    assert_custom_value, assert_redacted, rendered_text,
};

/// Calls the observer whenever a user operand is resolved.
pub(crate) struct BorrowSpy<T, F> {
    pub(crate) value: T,
    pub(crate) observe: F,
}
impl<T, F: Fn()> core::borrow::Borrow<T> for BorrowSpy<T, F> {
    fn borrow(&self) -> &T {
        (self.observe)();
        &self.value
    }
}
impl<T, F: Fn()> crate::borrow_for::BorrowFor<T> for BorrowSpy<T, F> {
    type View = T;
}

/// A non-Debug operand whose unsized string view is observed on every borrow.
pub(crate) struct StrOperand<F> {
    pub(crate) value: &'static str,
    pub(crate) observe: F,
}
impl<F: Fn()> core::borrow::Borrow<str> for StrOperand<F> {
    fn borrow(&self) -> &str {
        (self.observe)();
        self.value
    }
}
impl<F: Fn()> crate::borrow_for::BorrowFor<alloc::string::String> for StrOperand<F> {
    type View = str;
}

/// Renders the string comparison leaves and indexes, without a wrapper renderer.
#[derive(Clone)]
pub(crate) struct StringRenderer;
impl crate::ValueRenderer<str> for StringRenderer {
    fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{value:?}")
    }
}
impl crate::ValueRenderer<alloc::string::String> for StringRenderer {
    fn fmt(
        &self,
        value: &alloc::string::String,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        crate::ValueRenderer::<str>::fmt(self, value, f)
    }
}
impl crate::ValueRenderer<usize> for StringRenderer {
    fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{value}")
    }
}
