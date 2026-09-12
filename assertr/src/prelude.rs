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

#[cfg(feature = "std")]
pub use crate::assert_that_panic_by;
#[cfg(feature = "std")]
pub use crate::assert_that_panic_by_async;
#[cfg(feature = "http")]
pub use crate::assertions::http::prelude::*;
#[cfg(feature = "jiff")]
pub use crate::assertions::jiff::prelude::*;
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
#[cfg(feature = "std")]
pub use crate::assertions::std::prelude::*;
#[cfg(feature = "tokio")]
pub use crate::assertions::tokio::prelude::*;
#[cfg(feature = "matchers")]
pub use crate::partial;
#[cfg(test)]
pub(crate) use crate::test_support::FailureReportAssertions;
#[cfg(test)]
pub(crate) use crate::test_support::assert_caller_location;
pub use crate::{
    assert_that, assert_that_owned, assert_that_type,
    assertions::{
        HasLength,
        alloc::prelude::*,
        collection::{
            CollectionAssertions, RandomAccessExtractAssertions, StableOrderAssertions,
            StableOrderExtractAssertions,
        },
        condition::{ConditionAssertions, IterableConditionAssertions},
        core::prelude::*,
        map::MapAssertions,
        matcher::MatcherAssertions,
        set::SetAssertions,
    },
    condition::AssertrCondition,
    elements_are, elements_are_in_any_order, entries_are,
    expectation::{Expectation, ExpectationDiagnostics},
    failure::adapter::ToHumanReadableText,
    matchers,
    mode::{Capture, Mode, Panic},
    pattern,
};
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

#[cfg(test)]
pub(crate) use crate::AssertionContext;
