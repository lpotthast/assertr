use crate::failure::FailureKind;
use crate::{AssertThat, Mode, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use tokio::sync::Mutex;

/// Observes a locked Tokio mutex, retaining any acquired guard on rejection.
pub struct IsLocked;
impl<T, R> Expectation<Mutex<T>, R> for IsLocked {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mutex<T>: 'a;
    type Rejection<'a>
        = tokio::sync::MutexGuard<'a, T>
    where
        Self: 'a,
        Mutex<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.try_lock().map_or_else(|_| Ok(()), Err)
    }
}
impl<T, R> ExpectationDiagnostics<Mutex<T>, R> for IsLocked
where
    R: ValueRenderer<T>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is locked"),
            Some((actual, guard)) => failure
                .actual(render.struct_field(actual, "Mutex", "data", &*guard))
                .relation("is not locked"),
        }
    }
}
/// Acquires an available Tokio mutex and returns its guard.
pub struct IsNotLocked;
impl<T, R> Expectation<Mutex<T>, R> for IsNotLocked {
    type Success<'a>
        = tokio::sync::MutexGuard<'a, T>
    where
        Self: 'a,
        Mutex<T>: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Mutex<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.try_lock().map_err(|_| ())
    }
}
impl<T, R> ExpectationDiagnostics<Mutex<T>, R> for IsNotLocked {
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is not locked"),
            Some((actual, ())) => failure
                .actual(render.unavailable_struct_field(actual, "Mutex", "data", "<locked>"))
                .relation("is unexpectedly locked"),
        }
    }
}

/// Acquires a mutex and checks its value with a reusable assertion callback.
/// The callback runs in capture mode only after successful acquisition.
pub struct HasValueSatisfying<F>(F);

impl<F> HasValueSatisfying<F> {
    /// Owns a reusable callback whose failures become children of one mutex failure.
    #[must_use]
    pub const fn new(assertions: F) -> Self {
        Self(assertions)
    }
}

/// Evidence from a rejected mutex value check.
pub enum ValueRejection<'a, T> {
    /// The mutex could not be acquired.
    Locked,
    /// The original guard and the callback's captured failures.
    Rejected(tokio::sync::MutexGuard<'a, T>, crate::expectation::Evidence),
}

fn evaluate_value<'a, T, R: Clone>(
    actual: &'a Mutex<T>,
    assertions: impl for<'v> FnOnce(AssertThat<'v, T, crate::mode::Capture, R>),
    context: &AssertionContext<'_, R>,
) -> Result<(), ValueRejection<'a, T>> {
    let guard = actual.try_lock().map_err(|_| ValueRejection::Locked)?;
    let failures = crate::assert_that::collect_assertions(
        &*guard,
        context.render(),
        context.include_location(),
        assertions,
    );
    if failures.is_empty() {
        return Ok(());
    }
    let mut evidence = context.isolated();
    for failure in failures {
        evidence.record(failure);
    }
    Err(ValueRejection::Rejected(guard, evidence.into_evidence()))
}

fn explain_value<T, R: ValueRenderer<T>, Target>(
    rejected: Option<(&Mutex<T>, ValueRejection<'_, T>)>,
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target> {
    let render = context.render();
    match rejected {
        None => failure.relation("contains a value that satisfies the assertions"),
        Some((actual, ValueRejection::Locked)) => failure
            .actual(render.unavailable_struct_field(actual, "Mutex", "data", "<locked>"))
            .relation("is unexpectedly locked"),
        Some((actual, ValueRejection::Rejected(guard, evidence))) => evidence.explain(
            failure
                .actual(render.struct_field(actual, "Mutex", "data", &*guard))
                .relation("contains a value that does not satisfy the assertions"),
        ),
    }
}

impl<T, R: Clone, F> Expectation<Mutex<T>, R> for HasValueSatisfying<F>
where
    F: for<'a> Fn(AssertThat<'a, T, crate::mode::Capture, R>),
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = ValueRejection<'a, T>
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        evaluate_value(actual, &self.0, context)
    }
}
impl<T, R: Clone + ValueRenderer<T>, F> ExpectationDiagnostics<Mutex<T>, R>
    for HasValueSatisfying<F>
where
    F: for<'a> Fn(AssertThat<'a, T, crate::mode::Capture, R>),
{
    const KIND: FailureKind = FailureKind::Predicate;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        explain_value(rejected, failure, context)
    }
}

/// Non-blocking assertions for Tokio's [`Mutex`] type.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioMutexAssertions<T, R> {
    /// Asserts that `try_lock` cannot acquire the mutex.
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that `try_lock` can acquire the mutex.
    fn is_not_locked(self) -> Self;

    /// Alias of [`TokioMutexAssertions::is_not_locked`].
    #[track_caller]
    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }

    /// Tries to acquire the mutex and runs assertions against its contained value.
    ///
    /// Fails the assertion if the mutex is currently locked. The closure receives a capture-mode
    /// assertion, and any failures it raises are attached to one mutex-level failure.
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, crate::mode::Capture, R>),
        R: ValueRenderer<T> + Clone;
}

impl<T, M: Mode, R> TokioMutexAssertions<T, R> for AssertThat<'_, Mutex<T>, M, R> {
    #[track_caller]
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsLocked)
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.apply_assertion(IsNotLocked)
    }

    #[track_caller]
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, crate::mode::Capture, R>),
        R: ValueRenderer<T> + Clone,
    {
        let assertions = core::cell::Cell::new(Some(assertions));
        self.apply_assertion(HasValueSatisfying::new(
            |it: AssertThat<'_, T, crate::mode::Capture, R>| {
                let callback = assertions.take().expect("callback runs once");
                callback(it);
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    mod observations {
        use super::super::{HasValueSatisfying, IsNotLocked};
        use crate::{RenderingBudget, matchers::all_of, prelude::*, test_support::NoRenderer};
        use core::cell::Cell;
        use tokio::sync::Mutex;

        #[test]
        fn successful_acquisitions_are_released_between_siblings_without_a_renderer() {
            assert_that!(Mutex::new(7))
                .with_renderer(NoRenderer)
                .matches(all_of((IsNotLocked, IsNotLocked)));
        }

        #[test]
        fn callback_rejections_retain_truth_and_omissions_with_zero_child_budget() {
            let lock = Mutex::new(7);
            let budget = RenderingBudget::default().with_max_items(0);
            let failures = assert_that!(lock)
                .with_rendering_budget(budget)
                .capture(|it| {
                    it.has_value_satisfying(|value| {
                        value.is_equal_to(8).is_equal_to(9);
                    })
                });
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).is_empty();
            assert_that!(failures[0].omitted_children).is_equal_to(2);
            let definition = HasValueSatisfying::new(|value: AssertThat<'_, i32, Capture>| {
                value.is_equal_to(8).is_equal_to(9);
            });
            let composed = assert_that!(lock)
                .with_rendering_budget(budget)
                .capture(|it| it.matches(definition));
            assert_that!(composed[0].omitted_children).is_equal_to(2);
            assert_that!(lock.try_lock().is_ok()).is_true();
        }

        #[test]
        fn reusable_value_callback_skips_contention_and_releases_guarded_rejections() {
            let calls = Cell::new(0);
            let lock = Mutex::new(7);
            let definition = HasValueSatisfying::new(|value: AssertThat<'_, i32, Capture>| {
                calls.set(calls.get() + 1);
                value.is_equal_to(8);
            });
            let guard = lock.try_lock().unwrap();
            let failures = assert_that!(lock).capture(|it| it.matches(&definition));
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(0);
            drop(guard);
            let failures =
                assert_that!(lock).capture(|it| it.matches(all_of((&definition, &definition))));
            assert_that!(failures[0].children).has_length(2);
            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(lock.try_lock().is_ok()).is_true();
        }
    }

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};
        use tokio::sync::Mutex;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Mutex<i32>, Panic, NoRenderer>
                    => TokioMutexAssertions<i32, NoRenderer>
            );
        }
    }

    mod is_locked {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            mutex.must().be_locked();
            drop(guard);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            assert_caller_location!(assert_that!(mutex), is_locked());
        }

        #[tokio::test]
        async fn succeeds_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that!(&mutex).is_locked();
            drop(guard);
        }

        #[test]
        fn panics_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that_panic_by(|| assert_that!(mutex).with_location(false).is_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `mutex`

                    Actual: Mutex {{
                        data: 42,
                    }}

                    is not locked
                    -------- assertr --------
                "});
        }
    }

    mod is_not_locked {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().not_be_locked();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            let _guard = mutex.try_lock().unwrap();
            assert_caller_location!(assert_that!(mutex), is_not_locked());
        }

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that!(mutex).is_not_locked();
        }

        #[tokio::test]
        async fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that_panic_by(|| assert_that!(&mutex).with_location(false).is_not_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `&mutex`

                    Actual: Mutex {{
                        data: <locked>,
                    }}

                    is unexpectedly locked
                    -------- assertr --------
                "});
            drop(guard);
        }
    }

    /// Synonym of `is_not_locked`. The fluent name and caller location are pinned here. The
    /// behavior is covered by that module.
    mod is_free {
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().be_free();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let lock = Mutex::new(42);
            let _guard = lock.try_lock().unwrap();
            assert_caller_location!(assert_that!(lock), is_free());
        }
    }

    mod has_value_satisfying {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().have_value_satisfying(|value| {
                value.is_equal_to(42);
            });
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            assert_caller_location!(
                assert_that!(mutex),
                has_value_satisfying(|value| {
                    value.is_equal_to(43);
                })
            );
        }

        #[test]
        fn succeeds_when_the_mutex_is_available_and_the_value_satisfies_the_assertions() {
            let mutex = Mutex::new(String::from("value"));

            assert_that!(mutex).has_value_satisfying(|value| {
                value.contains("alu");
            });
        }

        #[test]
        fn accepts_an_fn_once_assertion_callback() {
            let mutex = Mutex::new(42);
            let captured = String::from("consumed");

            assert_that!(mutex).has_value_satisfying(move |value| {
                drop(captured);
                value.is_equal_to(42);
            });
        }

        #[test]
        fn panics_with_the_contained_failures_when_the_value_does_not_satisfy_the_assertions() {
            let mutex = Mutex::new(42);

            assert_that_panic_by(|| {
                assert_that!(mutex)
                    .with_location(false)
                    .has_value_satisfying(|value| {
                        value.is_equal_to(43);
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `mutex`

                    Actual: Mutex {{
                        data: 42,
                    }}

                    contains a value that does not satisfy the assertions

                    Nested failures:
                      - Expected: 43

                          Actual: 42
                    -------- assertr --------
                "});
        }

        #[tokio::test]
        async fn panics_when_the_mutex_is_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;

            assert_that_panic_by(|| {
                assert_that!(mutex)
                    .with_location(false)
                    .has_value_satisfying(|value| {
                        value.is_equal_to(42);
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `mutex`

                    Actual: Mutex {{
                        data: <locked>,
                    }}

                    is unexpectedly locked
                    -------- assertr --------
                "});

            drop(guard);
        }

        #[test]
        fn works_in_capture_mode() {
            let mutex = Mutex::new(42);

            let failures = assert_that!(mutex).with_location(false).capture(|it| {
                it.has_value_satisfying(|value| {
                    value.is_equal_to(43);
                })
            });

            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .satisfies_owned(
                            |failure| ToHumanReadableText.render(failure),
                            |description| {
                                description.contains(formatdoc! {r"
                                    Actual: Mutex {{
                                        data: 42,
                                    }}

                                    contains a value that does not satisfy the assertions
                                "});
                            },
                        )
                        .satisfies(
                            |failure| &failure.children,
                            |children| {
                                children.contains_exactly_satisfying([
                                    |child: AssertThat<AssertionFailure, Capture>| {
                                        child.satisfies_owned(
                                            |failure| ToHumanReadableText.render(failure),
                                            |description| {
                                                description.contains(formatdoc! {r"
                                                    Expected: 43

                                                      Actual: 42
                                                "});
                                            },
                                        );
                                    },
                                ]);
                            },
                        );
                },
            ]);
        }
    }
}
