//! Shared test assertions, expectations, operands, renderers, and capability fixtures.

mod assertions;
mod caller_location;
mod collections;
mod evidence;
mod expectations;
mod operands;
mod rendering;

pub(crate) use assertions::{FailureReportAssertions, assert_trait_impl, rejected_kind};
#[cfg(feature = "std")]
pub(crate) use caller_location::{LocationRecorder, raised_failure, recording_presentation};
pub(crate) use caller_location::{assert_caller_location, block_on, check_caller_location};
pub(crate) use collections::{PreservedBag, UnorderedMap, UnorderedSet};
pub(crate) use evidence::{assert_bounded_order, bounded_failures};
pub(crate) use expectations::opaque_predicate;
pub(crate) use operands::{BorrowSpy, StrOperand};
pub(crate) use rendering::{
    ComparisonRenderer, CustomValueRenderer, NoRenderer, NumericRenderer, PanickingRenderer,
    RedactingRenderer, RendererActual, RendererExpected, SENTINEL, SentinelRenderer,
    StringRenderer, assert_custom_fact, assert_custom_value, assert_redacted,
};
