use alloc::string::String;
use core::fmt::Write;
use indoc::writedoc;

use crate::{AssertThat, Mode, tracking::AssertionTracking};

/// Non-consuming assertions for the exact number of elements remaining in an iterator.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ExactSizeIteratorAssertions {
    /// Asserts that `ExactSizeIterator::len()` equals `expected` without advancing the iterator.
    fn has_remaining_count(self, expected: usize) -> Self;
    /// Asserts that no elements remain without advancing the iterator.
    fn has_no_remaining_elements(self) -> Self;
    /// Asserts that at least one element remains without advancing the iterator.
    fn has_remaining_elements(self) -> Self;
}

impl<I: ExactSizeIterator, M: Mode, R> ExactSizeIteratorAssertions for AssertThat<'_, I, M, R> {
    #[track_caller]
    fn has_remaining_count(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual().len();
        if actual != expected {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                Expected remaining element count: {expected}
                  Actual remaining element count: {actual}
            "}
            });
        }
        self
    }

    #[track_caller]
    fn has_no_remaining_elements(self) -> Self {
        self.has_remaining_count(0)
    }

    #[track_caller]
    fn has_remaining_elements(self) -> Self {
        self.track_assertion();
        if self.actual().len() == 0 {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                Expected the iterator to have remaining elements, but it has none.
            "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod has_remaining_count {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_count_matches_without_advancing() {
            let mut iterator = [1, 2, 3].into_iter();
            assert_that!(iterator.next()).is_equal_to(Some(1));
            assert_that!(iterator)
                .has_remaining_count(2)
                .has_remaining_count(2);
        }

        #[test]
        fn panics_when_count_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].into_iter())
                    .with_location(false)
                    .has_remaining_count(3);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected remaining element count: 3
                      Actual remaining element count: 2
                    -------- assertr --------
                "});
        }
    }

    mod has_no_remaining_elements {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_no_elements_remain() {
            assert_that!([1].into_iter().skip(1)).has_no_remaining_elements();
        }

        #[test]
        fn panics_when_elements_remain() {
            assert_that_panic_by(|| {
                assert_that!([1].into_iter())
                    .with_location(false)
                    .has_no_remaining_elements();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected remaining element count: 0
                      Actual remaining element count: 1
                    -------- assertr --------
                "});
        }
    }

    mod has_remaining_elements {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_elements_remain_without_advancing() {
            assert_that!([1, 2].into_iter())
                .has_remaining_elements()
                .has_remaining_count(2);
        }

        #[test]
        fn panics_when_no_elements_remain() {
            assert_that_panic_by(|| {
                assert_that!([1_i32; 0].into_iter())
                    .with_location(false)
                    .has_remaining_elements();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected the iterator to have remaining elements, but it has none.
                    -------- assertr --------
                "});
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn assertions_have_fluent_aliases() {
            assert_that!([1, 2].into_iter())
                .have_remaining_count(2)
                .have_remaining_elements();
            assert_that!([1_i32; 0].into_iter()).have_no_remaining_elements();
        }
    }
}
