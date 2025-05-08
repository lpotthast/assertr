use crate::AssertThat;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use indoc::writedoc;
use jiff::Span;
use std::fmt::Write;

pub trait SpanAssertions {
    fn is_zero(self) -> Self;

    fn is_negative(self) -> Self;

    /// Unlike the is_positive assertions on numerics, this fails for a span of 0!
    fn is_positive(self) -> Self;
}

impl<M: Mode> SpanAssertions for AssertThat<'_, Span, M> {
    #[track_caller]
    fn is_zero(self) -> Self {
        self.track_assertion();

        if !self.actual().is_zero() {
            self.add_detail_message("Actual was not zero.");

            let expected = Span::new();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "#, actual = self.actual()}
            });
        }

        self
    }

    #[track_caller]
    fn is_negative(self) -> Self {
        self.track_assertion();

        if !self.actual().is_negative() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value to be negative. But was

                      Actual: {actual:#?}
                "#, actual = self.actual()}
            });
        }

        self
    }

    #[track_caller]
    fn is_positive(self) -> Self {
        self.track_assertion();

        if !self.actual().is_positive() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value to be positive. But was

                      Actual: {actual} ({actual:#?})
                "#, actual = self.actual()}
            });
        }

        self
    }
}

#[cfg(test)]
mod tests {
    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::{Span, ToSpan};

        #[test]
        fn succeeds_when_zero() {
            assert_that(Span::new()).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: Span = 2.hours().minutes(30);

            assert_that_panic_by(|| assert_that(duration).with_location(false).is_zero())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 0s

                      Actual: 2h 30m

                    Details: [
                        Actual was not zero.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::ToSpan;

        #[test]
        fn succeeds_when_zero() {
            assert_that(-2.hours().minutes(30)).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that(0.seconds()).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 0s
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| {
                assert_that(2.hours().minutes(30))
                    .with_location(false)
                    .is_negative()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 2h 30m
                    -------- assertr --------
                "#});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::ToSpan;

        #[test]
        fn succeeds_when_positive() {
            assert_that(2.hours().minutes(30)).is_positive();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that(0.seconds()).with_location(false).is_positive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be positive. But was

                      Actual: PT0S (0s)
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| {
                assert_that(-2.hours().minutes(30))
                    .with_location(false)
                    .is_positive()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be positive. But was

                      Actual: -PT2H30M (2h 30m ago)
                    -------- assertr --------
                "#});
        }
    }
}
