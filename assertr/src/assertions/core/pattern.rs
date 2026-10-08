use crate::{
    AssertThat, Mode,
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    renderer::DebugRenderer,
    renderer::ValueRenderer,
};

/// A Rust pattern together with the predicate and source text needed to assert that it matches.
///
/// Create patterns with [`pattern!`](crate::pattern) rather than constructing this type directly.
/// It is `Clone` when the predicate is. `Debug` shows the pattern's source text.
#[derive(Clone)]
pub struct Pattern<P> {
    description: &'static str,
    predicate: P,
}

impl<P> core::fmt::Debug for Pattern<P> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Pattern")
            .field("pattern", &self.description)
            .finish_non_exhaustive()
    }
}

impl<P> Pattern<P> {
    /// Creates the runtime representation emitted by [`pattern!`](crate::pattern).
    pub(crate) fn new(description: &'static str, predicate: P) -> Self {
        Self {
            description,
            predicate,
        }
    }
}

/// Creates a pattern for use with [`PatternAssertions`] or as a reusable matcher.
///
/// The subject is matched by reference, so ordinary patterns benefit from Rust's match ergonomics
/// and do not consume the assertion subject. Pattern guards are supported. They must be reusable
/// (`Fn`), because the same pattern serves direct assertions and reusable matchers.
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug)]
/// enum Error {
///     Invalid(&'static str),
/// }
///
/// let result: Result<(), Error> = Err(Error::Invalid("invalid UTF-8"));
/// assert_that!(result)
///     .is_matching(pattern!(Err(Error::Invalid(reason)) if reason.contains("UTF-8")));
/// ```
#[macro_export]
macro_rules! pattern {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        $crate::__private::new_pattern(
            ::core::stringify!($pattern $(if $guard)?),
            |actual: &_| ::core::matches!(actual, $pattern $(if $guard)?),
        )
    };
}

impl<A: ?Sized, R, F> Expectation<A, R> for Pattern<F>
where
    F: Fn(&A) -> bool,
    R: ValueRenderer<A>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    fn evaluate(&self, actual: &A, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if (self.predicate)(actual) {
            Ok(())
        } else {
            Err(())
        }
    }

    const KIND: FailureKind = FailureKind::Predicate;
    fn explain(
        &self,
        rejected: Option<(&A, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "matches the pattern",
                "does not match the pattern",
            )
            .expected(self.description)
    }
}

/// Rejects subjects that match a Rust pattern, retaining the unwanted pattern in diagnostics.
///
/// Construct with [`new`](Self::new) and [`pattern!`](crate::pattern). Like `Pattern`, guards
/// must implement `Fn`.
///
/// ```
/// use assertr::{matchers::DoesNotMatchPattern, prelude::*};
/// assert_that!(Some(3)).matches(DoesNotMatchPattern::new(pattern!(None)));
/// ```
#[derive(Clone)]
pub struct DoesNotMatchPattern<P>(Pattern<P>);

impl<P> core::fmt::Debug for DoesNotMatchPattern<P> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("DoesNotMatchPattern")
            .field(&self.0)
            .finish()
    }
}

impl<P> DoesNotMatchPattern<P> {
    /// Owns the pattern that the subject must not match.
    #[must_use]
    pub const fn new(pattern: Pattern<P>) -> Self {
        Self(pattern)
    }
}

impl<T: ?Sized, R: ValueRenderer<T>, P: Fn(&T) -> bool> Expectation<T, R>
    for DoesNotMatchPattern<P>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if (self.0.predicate)(actual) {
            Err(())
        } else {
            Ok(())
        }
    }

    const KIND: FailureKind = FailureKind::Predicate;
    fn explain(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "does not match the pattern",
                "matches the pattern",
            )
            .unexpected(self.0.description)
    }
}

/// Assertions based on arbitrary Rust patterns.
///
/// Failure diagnostics include the pattern's source text and the subject rendered through the
/// active [`ValueRenderer`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PatternAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject matches `pattern`.
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that the subject does not match `pattern`.
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;
}

impl<T, M: Mode, R> PatternAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.matches(pattern)
    }

    #[track_caller]
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.matches(DoesNotMatchPattern::new(pattern))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use indoc::formatdoc;

    #[derive(Debug)]
    enum TestError {
        MissingQueryParams,
        MissingTokenQueryParam,
        InvalidToken { reason: String },
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            Some(42).must().be_matching(pattern!(Some(42)));
            Some(42).must().not_be_matching(pattern!(Some(43)));
        }
    }

    mod matcher {
        use core::cell::Cell;

        use crate::failure::FailureKind;
        use crate::matchers::{DoesNotMatchPattern, dereferenced, elements_are};
        use crate::prelude::*;

        #[test]
        fn supports_pattern_guards() {
            assert_that!(Some(2)).matches(pattern!(Some(value) if *value > 0));
        }

        #[test]
        fn reusable_and_ordinary_patterns_report_the_same_failure() {
            let some = || assert_that!(Some(1)).with_location(false);
            let none = || assert_that!(None::<i32>).with_location(false);
            for (reusable, ordinary) in [
                (
                    some().capture(|it| it.matches(pattern!(None))),
                    some().capture(|it| it.is_matching(pattern!(None))),
                ),
                (
                    none().capture(|it| it.matches(DoesNotMatchPattern::new(pattern!(None)))),
                    none().capture(|it| it.is_not_matching(pattern!(None))),
                ),
            ] {
                assert_that!(&reusable).has_length(1);
                assert_that!(reusable[0].kind).is_equal_to(FailureKind::Predicate);
                assert_that!(reusable[0].to_string()).is_equal_to(ordinary[0].to_string());
            }
        }

        #[test]
        fn negative_patterns_accept_unsized_subjects() {
            let forbidden = DoesNotMatchPattern::new(pattern!("secret"));
            assert_that!(String::from("public")).matches(dereferenced(&forbidden));
            let failures = assert_that!(String::from("secret"))
                .capture(|it| it.matches(dereferenced(&forbidden)));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].kind).is_equal_to(FailureKind::Predicate);
        }

        #[test]
        fn missing_subject_keeps_the_unexpected_pattern_without_running_its_guard() {
            let calls = Cell::new(0);
            let forbidden = DoesNotMatchPattern::new(pattern!(Some(_) if {
                calls.set(calls.get() + 1);
                true
            }));
            let failures = assert_that!([] as [Option<i32>; 0])
                .capture(|it| it.matches(elements_are(matchers![&forbidden])));
            let description = failures[0].children[0].constraint.as_ref().unwrap();
            assert_that!(description.relation.as_deref())
                .is_equal_to(Some("does not match the pattern"));
            assert_that!(description.expected).is_none();
            assert_that!(description.unexpected).is_some();
            assert_that!(calls.get()).is_equal_to(0);

            let failures = assert_that!(Some(1)).capture(|it| it.matches(&forbidden));
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
        }
    }

    mod renderer_contract {
        use core::fmt;

        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PatternAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn both_methods_render_the_subject_with_the_active_renderer() {
            struct Opaque(u8);

            for negative in [false, true] {
                let failures = assert_that!(Opaque(1))
                    .with_debug_format(|actual: &Opaque, f: &mut fmt::Formatter<'_>| {
                        write!(f, "Opaque({})", actual.0)
                    })
                    .capture(|it| {
                        if negative {
                            it.is_not_matching(pattern!(Opaque(1)))
                        } else {
                            it.is_matching(pattern!(Opaque(2)))
                        }
                    });
                assert_that!(failures[0].to_string()).contains("Actual: Opaque(1)");
            }
        }
    }

    mod is_matching {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<(), TestError>::Err(
                    TestError::MissingTokenQueryParam
                )),
                is_matching(pattern!(Err(TestError::MissingQueryParams)))
            );
        }

        #[test]
        fn succeeds_when_pattern_matches() {
            assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams))
                .is_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn supports_patterns_with_guards() {
            assert_that!(Result::<(), TestError>::Err(TestError::InvalidToken {
                reason: "invalid UTF-8".to_owned(),
            }))
            .is_matching(pattern!(
                Err(TestError::InvalidToken { reason }) if reason.contains("UTF-8")
            ));
        }

        #[test]
        fn supports_or_patterns() {
            assert_that!(Result::<(), TestError>::Err(
                TestError::MissingTokenQueryParam
            ))
            .is_matching(pattern!(Err(
                TestError::MissingQueryParams | TestError::MissingTokenQueryParam
            )));
        }

        #[test]
        fn borrows_the_actual_value_while_matching() {
            let actual = Some(String::from("value"));

            assert_that!(&actual).is_matching(pattern!(Some(value) if value == "value"));

            assert_that!(actual).get_some().is_equal_to("value");
        }

        #[test]
        fn evaluates_the_actual_expression_once() {
            let mut evaluations = 0;

            assert_that!({
                evaluations += 1;
                Some(42)
            })
            .is_matching(pattern!(Some(42)));

            assert_that!(evaluations).is_equal_to(1);
        }

        #[test]
        fn panics_with_the_expected_pattern_and_rendered_actual_value() {
            assert_that_panic_by(|| {
                assert_that!(Result::<(), TestError>::Err(
                    TestError::MissingTokenQueryParam
                ))
                .with_location(false)
                .is_matching(pattern!(Err(TestError::MissingQueryParams)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Result::<(), TestError>::Err(TestError::MissingTokenQueryParam)`

                Actual: Err(
                    MissingTokenQueryParam,
                )

                does not match the pattern

                Expected: Err(TestError::MissingQueryParams)
                -------- assertr --------
            "});
        }
    }

    mod is_not_matching {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams)),
                is_not_matching(pattern!(Err(TestError::MissingQueryParams)))
            );
        }

        #[test]
        fn succeeds_when_pattern_does_not_match() {
            assert_that!(Result::<(), TestError>::Err(
                TestError::MissingTokenQueryParam
            ))
            .is_not_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn supports_patterns_with_guards() {
            assert_that!(Result::<(), TestError>::Err(TestError::InvalidToken {
                reason: "expired token".to_owned(),
            }))
            .is_not_matching(pattern!(
                Err(TestError::InvalidToken { reason }) if reason.contains("UTF-8")
            ));
        }

        #[test]
        fn panics_with_the_unexpected_pattern_and_rendered_actual_value() {
            assert_that_panic_by(|| {
                assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams))
                    .with_location(false)
                    .is_not_matching(pattern!(Err(TestError::MissingQueryParams)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Result::<(), TestError>::Err(TestError::MissingQueryParams)`

                Actual: Err(
                    MissingQueryParams,
                )

                matches the pattern

                Unexpected: Err(TestError::MissingQueryParams)
                -------- assertr --------
            "});
        }
    }
}
