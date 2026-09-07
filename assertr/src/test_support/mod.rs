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
    CustomValueRenderer, NoRenderer, NumericRenderer, RedactingRenderer, RendererActual,
    RendererExpected, SENTINEL, SentinelRenderer, assert_custom_fact, assert_custom_value,
    assert_redacted, rendered_text,
};
