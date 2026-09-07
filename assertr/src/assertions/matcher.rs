//! Assertions using expected-side matchers.
//!
//! Apply a `partial!` expectation to a subject with [`MatcherAssertions::matches`]. The
//! [partial matching guide](mod@crate::matchers) explains selected fields, nested structures, and
//! using existing assertion methods within a field through [`crate::matchers::satisfying`].

use crate::{
    AssertThat, DebugRenderer, Mode,
    failure::FailureKind,
    matchers::{AssertrMatcher, MatchContext},
};

/// Assertions against reusable expected-side constraints.
///
/// Use `partial!` to describe selected fields and nested values. Its built-in field matchers
/// cover a selective set of constraints. [`crate::matchers::satisfying`] brings existing
/// assertion methods into an expectation when another check is needed. See the
/// [partial matching guide](mod@crate::matchers) for examples and feature requirements.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MatcherAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject satisfies a matcher.
    ///
    /// For example, `.matches(partial!(User { name: "Alice", .. }))` checks only the selected
    /// field. Pass `&matcher` to reuse an expectation. Plain values are accepted as equality
    /// expectations inside `partial!`, but this method requires a matcher. For an ordinary
    /// equality assertion, use
    /// [`is_equal_to`](crate::assertions::core::partial_eq::PartialEqAssertions::is_equal_to).
    #[track_caller]
    fn matches<E: AssertrMatcher<T, R>>(self, expected: E) -> Self;

    /// Asserts that the subject does not satisfy a matcher.
    ///
    /// Negates the entire expectation. For a `partial!` expectation, a mismatch in any selected
    /// field is enough for this assertion to pass. Ignored fields do not affect the result.
    #[track_caller]
    fn does_not_match<E: AssertrMatcher<T, R>>(self, unexpected: E) -> Self;
}

impl<T, M: Mode, R> MatcherAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn matches<E: AssertrMatcher<T, R>>(self, expected: E) -> Self {
        self.track_assertion();

        self.assert_matcher(&expected, true);

        self
    }

    #[track_caller]
    fn does_not_match<E: AssertrMatcher<T, R>>(self, unexpected: E) -> Self {
        self.track_assertion();

        self.assert_matcher(&unexpected, false);

        self
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    #[track_caller]
    pub(crate) fn assert_matcher<E: AssertrMatcher<T, R>>(&self, matcher: &E, positive: bool) {
        let mut context = MatchContext::for_assertion(self);
        context.set_positive(positive);
        if matcher.evaluate(self.actual(), &mut context).matched != positive {
            if context.evidence.is_empty() && context.omitted == 0 {
                context.outcome(!positive, |context| matcher.describe(context));
            }
            self.failure(FailureKind::Matching)
                .relation(if positive {
                    "does not match"
                } else {
                    "matches unexpectedly"
                })
                .omitted_children(context.omitted_children())
                .children(context.into_failures())
                .raise();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{matchers::*, prelude::*};

    mod matches {
        use indoc::indoc;

        use super::*;

        #[cfg(feature = "fluent")]
        #[test]
        fn fluent_alias_is_as_expected() {
            1.must().r#match(equal_to(1));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), matches(equal_to(2)));
        }

        #[test]
        fn trait_does_not_require_a_renderer() {
            crate::test_support::assert_trait_impl!(
                AssertThat<'static, (), Panic, crate::test_support::NoRenderer>
                    => MatcherAssertions<(), crate::test_support::NoRenderer>
            );
        }

        #[test]
        fn tracks_once_and_keeps_capture_assertions_isolated() {
            let subject = assert_that!(1).with_renderer(crate::DebugRenderer);
            let subject = subject.matches(satisfying(|it| {
                it.is_equal_to(1).is_less_than(2);
            }));

            assert_that!(subject.state.number_of_assertions.borrow().0).is_equal_to(1);
        }

        #[test]
        fn retains_one_outer_failure_and_equality_evidence() {
            let failures = assert_that!(1)
                .with_location(false)
                .capture(|it| it.matches(equal_to(2)));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(crate::FailureKind::Matching);
                    element
                        .derive(|value| &value.children[0].kind)
                        .is_equal_to(crate::FailureKind::Equality);
                    element
                        .derive_owned(|value| ToHumanReadableText.render(value))
                        .is_equal_to(indoc! {r"
                -------- assertr --------
                Expression: `1`

                does not match

                Nested failures:
                  - Expected: 2

                      Actual: 1
                -------- assertr --------
            "});
                },
            ]);
        }
    }

    mod does_not_match {
        use super::*;

        #[cfg(feature = "fluent")]
        #[test]
        fn fluent_alias_is_as_expected() {
            1.must().not_match(equal_to(2));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), does_not_match(equal_to(1)));
        }

        #[test]
        fn successful_constraint_explains_negation() {
            let failures = assert_that!(1).capture(|it| it.does_not_match(equal_to(1)));

            assert_that!(failures[0].actual).is_none();
            assert_that!(failures[0].children[0].unexpected).is_some();
        }
    }
}
