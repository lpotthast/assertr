use crate::failure::{FailureBuilder, FailureKind};
use crate::{AssertThat, Mode, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics};

/// A Rust pattern together with the predicate and source text needed to assert that it matches.
///
/// Create patterns with [`pattern!`](crate::pattern) rather than constructing this type directly.
pub struct Pattern<P> {
    description: &'static str,
    predicate: P,
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
/// and do not consume the assertion subject. Pattern guards are supported. Reusable matchers
/// require `Fn` guards. Direct pattern assertions also accept `FnOnce` guards.
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
}
impl<A: ?Sized, R, F> ExpectationDiagnostics<A, R> for Pattern<F>
where
    F: Fn(&A) -> bool,
{
    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejected: Option<(&A, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("matches the pattern")
                .expected(self.description),
            Some((_, ())) => failure
                .relation("does not satisfy the constraint")
                .constraint(context.describe(&self)),
        }
    }
}

// Ordinary checks render the subject. Reusable positive patterns instead describe a constraint.
fn explain_pattern_rejection<T, Target, R: ValueRenderer<T>>(
    actual: &T,
    description: &'static str,
    negative: bool,
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target> {
    let failure = if negative {
        failure
            .unexpected(description)
            .relation("matches the pattern")
    } else {
        failure
            .expected(description)
            .relation("does not match the pattern")
    };
    failure.actual(context.render().value(actual))
}

/// Rejects subjects that match a Rust pattern, retaining the unwanted pattern in diagnostics.
///
/// Construct with [`new`](Self::new) and [`pattern!`](crate::pattern). Like `Pattern`, reusable
/// guards must implement `Fn`. Ordinary [`PatternAssertions::is_not_matching`] also accepts
/// consuming `FnOnce` guards through its execution adapter.
///
/// ```
/// use assertr::{matchers::DoesNotMatchPattern, prelude::*};
/// assert_that!(Some(3)).matches(DoesNotMatchPattern::new(pattern!(None)));
/// ```
pub struct DoesNotMatchPattern<P>(Pattern<P>);

impl<P> DoesNotMatchPattern<P> {
    /// Owns the pattern that the subject must not match.
    #[must_use]
    pub const fn new(pattern: Pattern<P>) -> Self {
        Self(pattern)
    }
}

impl<T, R, P: Fn(&T) -> bool> Expectation<T, R> for DoesNotMatchPattern<P> {
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
}

impl<T, R: ValueRenderer<T>, P: Fn(&T) -> bool> ExpectationDiagnostics<T, R>
    for DoesNotMatchPattern<P>
{
    const KIND: FailureKind = FailureKind::Predicate;
    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .unexpected(self.0.description)
                .relation("does not match the pattern"),
            Some((actual, ())) => {
                explain_pattern_rejection(actual, self.0.description, true, failure, context)
            }
        }
    }
}

/// Assertions based on arbitrary Rust patterns.
///
/// Failure diagnostics include the pattern's source text and the subject rendered through the
/// active [`ValueRenderer`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PatternAssertions<T, R> {
    /// Asserts that the subject matches `pattern`.
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that the subject does not match `pattern`.
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>;
}

impl<T, M: Mode, R> PatternAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>,
    {
        assert_pattern(self, pattern, false)
    }

    #[track_caller]
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>,
    {
        assert_pattern(self, pattern, true)
    }
}

#[track_caller]
fn assert_pattern<T, M: Mode, R, P>(
    this: AssertThat<'_, T, M, R>,
    pattern: Pattern<P>,
    negative: bool,
) -> AssertThat<'_, T, M, R>
where
    P: FnOnce(&T) -> bool,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    this.test_once_after_tracking(
        FailureKind::Predicate,
        core::panic::Location::caller(),
        |_| {
            if (pattern.predicate)(this.actual()) == negative {
                Err(())
            } else {
                Ok(())
            }
        },
        |(), failure, context| {
            explain_pattern_rejection(
                this.actual(),
                pattern.description,
                negative,
                failure,
                context,
            )
        },
    );
    this
}

#[cfg(test)]
mod tests {
    fn assert_one_use_guard_lifetime(negative: bool) {
        use crate::prelude::*;
        use core::cell::{Cell, RefCell};

        for (actual, matches) in [(Some(1), false), (Some(1), true), (None, true)] {
            let resource = RefCell::new(());
            let calls = Cell::new(0);
            let renders = Cell::new(0);
            let guard = resource.borrow_mut();
            let pattern = pattern!(Some(_) if {
                let _guard = guard;
                calls.set(calls.get() + 1);
                matches
            });
            let failures = assert_that!(actual)
                .with_debug_format(|value: &Option<i32>, f: &mut core::fmt::Formatter<'_>| {
                    assert_that!(resource.try_borrow_mut().is_ok()).is_true();
                    renders.set(renders.get() + 1);
                    write!(f, "{value:?}")
                })
                .capture(|it| {
                    let it = if negative {
                        it.is_not_matching(pattern)
                    } else {
                        it.is_matching(pattern)
                    };
                    assert_that!(resource.try_borrow_mut().is_ok()).is_true();
                    it
                });
            assert_that!(calls.get()).is_equal_to(usize::from(actual.is_some()));
            let failed = (actual.is_some() && matches) == negative;
            assert_that!(failures.len()).is_equal_to(usize::from(failed));
            assert_that!(renders.get()).is_equal_to(usize::from(failed));
        }
    }

    mod matcher {
        use core::cell::Cell;

        use crate::matchers::{DoesNotMatchPattern, elements_are};
        use crate::prelude::*;

        #[test]
        fn supports_pattern_guards() {
            assert_that!(Some(2)).matches(pattern!(Some(value) if *value > 0));
        }

        #[test]
        fn missing_subject_keeps_the_unexpected_pattern_without_running_its_guard() {
            let calls = Cell::new(0);
            let forbidden = DoesNotMatchPattern::new(pattern!(Some(_) if {
                calls.set(calls.get() + 1);
                true
            }));
            let failures = assert_that!([] as [Option<i32>; 0])
                .capture(|it| it.matches(elements_are((&forbidden,))));
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
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PatternAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(
                crate::matchers::DoesNotMatchPattern<fn(&i32) -> bool>
                    => crate::Expectation<i32, NoRenderer>
            );
            assert_trait_impl!(
                crate::matchers::Pattern<fn(&i32) -> bool>
                    => crate::ExpectationDiagnostics<i32, NoRenderer>
            );
        }
    }

    mod is_matching {
        use core::fmt;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Some(42).must().be_matching(pattern!(Some(42)));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<(), TestError>::Err(
                    TestError::MissingTokenQueryParam
                )),
                is_matching(pattern!(Err(TestError::MissingQueryParams)))
            );
        }

        #[derive(Debug)]
        enum TestError {
            MissingQueryParams,
            MissingTokenQueryParam,
            InvalidToken { reason: String },
        }

        #[test]
        fn succeeds_when_pattern_matches() {
            assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams))
                .is_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn supports_consuming_one_shot_guards() {
            let token = alloc::string::String::from("token");
            assert_that!(Some(1)).is_matching(pattern!(Some(_) if { drop(token); true }));
        }

        #[test]
        fn one_use_guard_is_released_before_rendering_or_continuation() {
            super::assert_one_use_guard_lifetime(false);
        }

        #[test]
        fn rejecting_guard_runs_once_after_tracking() {
            let calls = core::cell::Cell::new(0);
            let token = String::from("token");
            let failures = assert_that!(Some(1)).capture(|root| {
                let child = root.derive(|value| value);
                child.is_matching(pattern!(Some(_) if {
                    assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    calls.set(calls.get() + 1);
                    drop(token);
                    false
                }));
                root
            });
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
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

        #[test]
        fn works_in_capture_mode() {
            let failures = assert_that!(Some(42))
                .with_location(false)
                .capture(|it| it.is_matching(pattern!(None)));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Some(42)`

                        Actual: Some(
                            42,
                        )

                        does not match the pattern

                        Expected: None
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn uses_the_active_renderer() {
            struct Opaque(u8);

            assert_that_panic_by(|| {
                assert_that!(Opaque(1))
                    .with_debug_format(|actual: &Opaque, f: &mut fmt::Formatter<'_>| {
                        write!(f, "Opaque({})", actual.0)
                    })
                    .with_location(false)
                    .is_matching(pattern!(Opaque(2)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Opaque(1)`

                Actual: Opaque(1)

                does not match the pattern

                Expected: Opaque(2)
                -------- assertr --------
            "});
        }
    }

    mod is_not_matching {
        use core::fmt;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Some(42).must().not_be_matching(pattern!(Some(43)));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams)),
                is_not_matching(pattern!(Err(TestError::MissingQueryParams)))
            );
        }

        #[derive(Debug)]
        enum TestError {
            MissingQueryParams,
            MissingTokenQueryParam,
            InvalidToken { reason: String },
        }

        #[test]
        fn succeeds_when_pattern_does_not_match() {
            assert_that!(Result::<(), TestError>::Err(
                TestError::MissingTokenQueryParam
            ))
            .is_not_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn one_use_guard_is_released_before_rendering_or_continuation() {
            super::assert_one_use_guard_lifetime(true);
        }

        #[test]
        fn rejecting_guard_runs_once_after_tracking() {
            let calls = core::cell::Cell::new(0);
            let token = String::from("token");
            let failures = assert_that!(Some(1)).capture(|root| {
                let child = root.derive(|value| value);
                child.is_not_matching(pattern!(Some(_) if {
                    assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    calls.set(calls.get() + 1);
                    drop(token);
                    true
                }));
                root
            });
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
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

        #[test]
        fn works_in_capture_mode() {
            let failures = assert_that!(Some(42))
                .with_location(false)
                .capture(|it| it.is_not_matching(pattern!(Some(42))));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Some(42)`

                        Actual: Some(
                            42,
                        )

                        matches the pattern

                        Unexpected: Some(42)
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn uses_the_active_renderer() {
            struct Opaque(u8);

            assert_that_panic_by(|| {
                assert_that!(Opaque(1))
                    .with_debug_format(|actual: &Opaque, f: &mut fmt::Formatter<'_>| {
                        write!(f, "Opaque({})", actual.0)
                    })
                    .with_location(false)
                    .is_not_matching(pattern!(Opaque(1)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Opaque(1)`

                Actual: Opaque(1)

                matches the pattern

                Unexpected: Opaque(1)
                -------- assertr --------
            "});
        }
    }
}
