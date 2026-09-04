// A `no_std` crate does not receive the standard prelude in its unit-test modules even though
// the hosted test harness links `std`. Re-export the alloc prelude pieces those tests use,
// without changing the production prelude or feature surface.
#[cfg(all(test, not(feature = "std")))]
pub(crate) use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "derive")]
pub use assertr_derive::AssertrEq;

pub use crate::any;
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
#[cfg(feature = "num")]
pub use crate::assertions::num::NumAssertions;
#[cfg(feature = "program")]
pub use crate::assertions::program::Program;
#[cfg(feature = "program")]
pub use crate::assertions::program::ProgramAssertions;
#[cfg(feature = "program")]
pub use crate::assertions::program::ProgramAssertionsRequiringPanicMode;
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
#[cfg(feature = "serde-json")]
pub use crate::conversion::json;
#[cfg(feature = "serde-toml")]
pub use crate::conversion::toml;
#[cfg(all(test, not(feature = "std")))]
pub(crate) use crate::entry::assert_that_panic_by;
pub use crate::eq;
pub use crate::mode::{Capture, Mode, Panic};
pub use crate::pattern;
pub use crate::report::{FailureReporter, TextReporter};
#[cfg(test)]
pub(crate) use crate::test_support::FailureReportAssertions;
#[cfg(test)]
pub(crate) use crate::test_support::rendered_text;
pub use crate::{AssertThat, AssertionFailure, DebugRenderer, RenderingBudget, ValueRenderer};
#[cfg(feature = "fluent")]
pub use crate::{IntoAssertContext, IntoOwnedAssertContext};
