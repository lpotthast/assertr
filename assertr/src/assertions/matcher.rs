//! Assertions using expected-side matchers.
//!
//! [`MatcherAssertions::matches`] applies any reusable expectation to a subject, such as
//! [`eq`](crate::matchers::eq), a composition like [`all_of`](crate::matchers::all_of), or a
//! collection policy like [`elements_are!`](crate::elements_are). The
//! [matcher catalog](mod@crate::matchers) lists the built-in expectations. With the `partial`
//! feature, `partial!` also checks selected fields of structs and enums.

use crate::{AssertThat, DebugRenderer, Expectation, Mode};

/// Assertions against reusable expected-side constraints.
///
/// Matchers are ordinary expectations, so the same definition works for direct assertions,
/// `*_matching` methods, and nested composition. [`crate::expectation::satisfying`] brings
/// existing assertion methods into an expectation when no built-in matcher fits. See the
/// [matcher catalog](mod@crate::matchers) for examples and feature requirements.
///
/// ```
/// use assertr::{matchers::{all_of, eq, ge, lt}, prelude::*};
///
/// assert_that!(42).matches(all_of(matchers![ge(18), lt(65)]));
/// assert_that!([1, 2]).matches(elements_are![eq(1), ge(2)]);
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MatcherAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject satisfies a matcher.
    ///
    /// Delegates to [`AssertThat::apply_assertion`], retaining the expectation's failure kind and
    /// fields. For example, `matches(eq(2))` reports the equality failure directly, and
    /// `.matches(partial!(User { name: eq("Alice"), .. }))` checks only the selected field.
    ///
    /// Pass `&matcher` to reuse an expectation. Both this method and `partial!` fields require
    /// explicit matchers. Use [`eq`](crate::matchers::eq) for an equality matcher. For an ordinary
    /// equality assertion, use
    /// [`is_equal_to`](crate::assertions::core::partial_eq::PartialEqAssertions::is_equal_to).
    #[track_caller]
    fn matches<E: Expectation<T, R>>(self, expected: E) -> Self;
}

impl<T, M: Mode, R> MatcherAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn matches<E: Expectation<T, R>>(self, expected: E) -> Self {
        self.apply_assertion(expected)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::matchers::equal_to;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            1.must().match_expectation(equal_to(1));
        }
    }

    use crate::{matchers::*, prelude::*};

    mod matches {
        use super::*;
        use indoc::indoc;

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
            let subject = assert_that!(1).with_renderer(DebugRenderer);
            let subject = subject.matches(satisfying(|it| {
                it.is_equal_to(1).is_less_than(2);
            }));

            assert_that!(subject.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn uses_the_equality_failure_directly() {
            let failures = assert_that!(1)
                .with_location(false)
                .capture(|it| it.matches(equal_to(2)));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(crate::FailureKind::Equality);
                    element.derive(|value| &value.children).is_empty();
                    element
                        .derive_owned(ToString::to_string)
                        .is_equal_to(indoc! {r"
                -------- assertr --------
                Expression: `1`

                Expected: 2

                  Actual: 1
                -------- assertr --------
            "});
                },
            ]);
        }
    }
}
