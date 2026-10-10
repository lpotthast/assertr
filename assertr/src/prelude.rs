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

#[cfg(feature = "num")]
pub use crate::assertions::NumAssertions;
#[cfg(feature = "thirtyfour")]
pub use crate::assertions::ThirtyfourWebElementAssertions;
#[cfg(any(feature = "std", test))]
pub use crate::assertions::{AsyncFnOnceAssertions, FnOnceAssertions};
#[cfg(feature = "std")]
pub use crate::assertions::{
    CommandAssertions, EventualAssertions, MutexAssertions, PathAssertions, Patience,
};
#[cfg(feature = "http")]
pub use crate::assertions::{HttpHeaderValueAssertions, HttpHeaderValueExtractAssertions};
#[cfg(feature = "program")]
pub use crate::assertions::{Program, ProgramAssertions, ProgramExtractAssertions};
#[cfg(feature = "reqwest")]
pub use crate::assertions::{ReqwestResponseAssertions, ReqwestResponseExtractAssertions};
#[cfg(feature = "rootcause")]
pub use crate::assertions::{
    RootcauseDynamicReportAssertions, RootcauseDynamicReportExtractAssertions,
    RootcauseReportAssertions,
};
#[cfg(feature = "jiff")]
pub use crate::assertions::{SignedDurationAssertions, SpanAssertions, ZonedAssertions};
#[cfg(feature = "tokio")]
pub use crate::assertions::{
    TokioMutexAssertions, TokioRwLockAssertions, TokioWatchReceiverAssertions,
};
#[cfg(test)]
pub(crate) use crate::expectation::AssertionContext;
#[cfg(feature = "partial")]
pub use crate::partial;
#[cfg(test)]
pub(crate) use crate::test_support::FailureReportAssertions;
#[cfg(test)]
pub(crate) use crate::test_support::assert_caller_location;
pub use crate::{
    AssertThat, assert_that, assert_that_owned, assert_that_type,
    assertions::{
        BoolAssertions, BoxAssertions, BoxExtractAssertions, CharAssertions, CollectionAssertions,
        DebugAssertions, DisplayAssertions, ExactSizeIteratorAssertions, IdentityAssertions,
        IntoIteratorAssertions, IteratorAssertions, LengthAssertions, MapAssertions, MemAssertions,
        OptionAssertions, OptionExtractAssertions, PartialEqAssertions, PartialOrdAssertions,
        PatternAssertions, PollAssertions, PollExtractAssertions, RandomAccessExtractAssertions,
        RangeAssertions, RangeBoundAssertions, RefCellAssertions, ResultAssertions,
        ResultExtractAssertions, SetAssertions, StableOrderAssertions,
        StableOrderExtractAssertions, StrAssertions,
    },
    elements_are, elements_are_in_any_order, entries_are,
    expectation::Expectation,
    failure::{AssertionFailure, AssertionFailures},
    matchers,
    mode::{Capture, Mode, Panic},
    pattern,
    renderer::{DebugRenderer, RenderingBudget, ValueRenderer},
};
#[cfg(feature = "fluent")]
pub use crate::{FluentEntry, OwnedFluentEntry};
