//! Assertions about reports and compile-time trait availability.

use alloc::string::ToString;

use crate::{
    AssertThat, Mode,
    failure::{AssertionFailure, FailureKind},
};

pub(crate) trait FailureReportAssertions {
    fn has_text_report(self, expected: impl AsRef<str>) -> Self;
}

impl<M: Mode, R> FailureReportAssertions for AssertThat<'_, AssertionFailure, M, R> {
    #[track_caller]
    fn has_text_report(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let report = self.actual().to_string();
        let expected = expected.as_ref();
        if report != expected {
            self.raise(
                self.failure(FailureKind::Equality)
                    .actual(format_args!("{report:?}"))
                    .expected(format_args!("{expected:?}")),
            );
        }
        self
    }
}

macro_rules! assert_trait_impl {
    ($type:ty => $trait:path) => {{
        fn assert_implemented<T: $trait>() {}
        assert_implemented::<$type>();
    }};
}

pub(crate) use assert_trait_impl;
