// A `no_std` crate does not receive the standard prelude in its unit-test modules even though the
// hosted test harness links `std`. Re-export the alloc prelude pieces those tests use, without
// changing the production prelude or feature surface.
#[cfg(all(test, not(feature = "std")))]
pub(crate) use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

pub use crate::assert_that;
pub use crate::assert_that_owned;
#[cfg(feature = "std")]
pub use crate::assert_that_panic_by;
#[cfg(feature = "std")]
pub use crate::assert_that_panic_by_async;
pub use crate::assert_that_type;
pub use crate::assertions::HasLength;
pub use crate::assertions::alloc::prelude::*;
pub use crate::assertions::collection::CollectionAssertions;
pub use crate::assertions::collection::RandomAccessExtractAssertions;
pub use crate::assertions::collection::StableOrderAssertions;
pub use crate::assertions::collection::StableOrderExtractAssertions;
pub use crate::assertions::condition::ConditionAssertions;
pub use crate::assertions::condition::IterableConditionAssertions;
pub use crate::assertions::core::prelude::*;
#[cfg(feature = "http")]
pub use crate::assertions::http::prelude::*;
#[cfg(feature = "jiff")]
pub use crate::assertions::jiff::prelude::*;
pub use crate::assertions::map::MapAssertions;
pub use crate::assertions::matcher::MatcherAssertions;
#[cfg(feature = "num")]
pub use crate::assertions::num::NumAssertions;
#[cfg(feature = "program")]
pub use crate::assertions::program::Program;
#[cfg(feature = "program")]
pub use crate::assertions::program::ProgramAssertions;
#[cfg(feature = "program")]
pub use crate::assertions::program::ProgramExtractAssertions;
#[cfg(feature = "reqwest")]
pub use crate::assertions::reqwest::prelude::*;
#[cfg(feature = "rootcause")]
pub use crate::assertions::rootcause::prelude::*;
pub use crate::assertions::set::SetAssertions;
#[cfg(feature = "std")]
pub use crate::assertions::std::prelude::*;
#[cfg(feature = "tokio")]
pub use crate::assertions::tokio::prelude::*;
pub use crate::condition::AssertrCondition;
pub use crate::elements_are;
pub use crate::elements_are_in_any_order;
pub use crate::entries_are;
pub use crate::failure::adapter::ToHumanReadableText;
pub use crate::matchers;
pub use crate::matchers::AssertrMatcher;
pub use crate::mode::{Capture, Mode, Panic};
#[cfg(feature = "matchers")]
pub use crate::partial;
pub use crate::pattern;
#[cfg(test)]
pub(crate) use crate::test_support::FailureReportAssertions;
#[cfg(test)]
pub(crate) use crate::test_support::assert_caller_location;
// Without the `std` feature, unit tests use a private helper backed by the hosted test harness.
#[cfg(all(test, not(feature = "std")))]
pub(crate) use crate::test_support::assert_that_panic_by;
#[cfg(test)]
pub(crate) use crate::test_support::rendered_text;
pub use crate::{
    AssertThat, AssertionFailure, AssertionFailures, DebugRenderer, RenderingBudget, ValueRenderer,
};
#[cfg(feature = "fluent")]
pub use crate::{IntoAssertContext, IntoOwnedAssertContext};
