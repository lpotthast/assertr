//! Assertions using expected-side matchers.
//!
//! Apply a `partial!` expectation to a subject with [`MatcherAssertions::matches`]. The
//! [partial matching guide](mod@crate::matchers) explains selected fields, nested structures, and
//! using existing assertion methods within a field through [`crate::expectation::satisfying`].

use crate::{AssertThat, DebugRenderer, ExpectationDiagnostics, Mode};

/// Assertions against reusable expected-side constraints.
///
/// Use `partial!` to describe selected fields and nested values. Its built-in field matchers
/// cover a selective set of constraints. [`crate::expectation::satisfying`] brings existing
/// assertion methods into an expectation when another check is needed. See the
/// [partial matching guide](mod@crate::matchers) for examples and feature requirements.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MatcherAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject satisfies a matcher.
    ///
    /// Delegates to [`AssertThat::apply_assertion`], retaining the expectation's failure kind and
    /// fields. For example, `matches(equal_to(2))` reports the equality failure directly.
    ///
    /// For example, `.matches(partial!(User { name: eq("Alice"), .. }))` checks only the selected
    /// field. Pass `&matcher` to reuse an expectation. Both this method and `partial!` fields
    /// require explicit matchers. Use [`eq`](crate::matchers::eq) for an equality matcher.
    /// For an ordinary equality assertion, use
    /// [`is_equal_to`](crate::assertions::core::partial_eq::PartialEqAssertions::is_equal_to).
    #[track_caller]
    fn matches<E: ExpectationDiagnostics<T, R>>(self, expected: E) -> Self;
}

impl<T, M: Mode, R> MatcherAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn matches<E: ExpectationDiagnostics<T, R>>(self, expected: E) -> Self {
        self.apply_assertion(expected)
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
                        .derive_owned(|value| ToHumanReadableText.render(value))
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
