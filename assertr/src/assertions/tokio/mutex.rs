use tokio::sync::Mutex;

use crate::{
    AssertThat, Mode,
    assertions::std::mutex::{explain_acquired_lock, explain_held_lock, lock_data},
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    renderer::{DebugRenderer, ValueRenderer},
};

/// Observes a locked Tokio mutex, retaining any acquired guard on rejection.
#[derive(Debug, Clone, Copy)]
pub struct IsLocked;

impl<T, R> Expectation<Mutex<T>, R> for IsLocked
where
    R: ValueRenderer<T>,
{
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

    const KIND: FailureKind = FailureKind::Other;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("is locked"),
            Some((actual, guard)) => {
                let failure = explain_acquired_lock(actual, "Mutex", &*guard, failure, context);
                // Release before raising or observing the next composed assertion.
                drop(guard);
                failure
            }
        }
    }
}

/// Acquires an available Tokio mutex and returns its guard.
#[derive(Debug, Clone, Copy)]
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

    const KIND: FailureKind = FailureKind::Other;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("is not locked"),
            Some((actual, ())) => explain_held_lock(actual, "Mutex", failure, context),
        }
    }
}

/// Acquires a mutex and checks its value with a reusable assertion callback.
/// The callback runs in capture mode only after successful acquisition.
///
/// It is `Clone` when the callback is. `Debug` omits the callback.
#[derive(Clone)]
pub struct HasValueSatisfying<F>(F);

impl<F> core::fmt::Debug for HasValueSatisfying<F> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("HasValueSatisfying")
            .finish_non_exhaustive()
    }
}

impl<F> HasValueSatisfying<F> {
    /// Owns a reusable callback whose failures become children of one mutex failure.
    #[must_use]
    pub const fn new(assertions: F) -> Self {
        Self(assertions)
    }
}

/// The rejection of [`HasValueSatisfying`]: why the guarded value could not be accepted.
///
/// The `Rejected` variant holds the guard acquired for the callback, so explanation renders the
/// same value the callback saw, and the callback's captured failures as [`Evidence`]. Explanation
/// releases the guard after rendering it. Drop the rejection promptly if a composite does not
/// explain it, because the guard keeps the mutex locked.
///
/// [`Evidence`]: crate::expectation::Evidence
#[derive(Debug)]
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
    let mut evidence = context.isolated();
    if evidence.run_assertions(&*guard, assertions) {
        return Ok(());
    }
    Err(ValueRejection::Rejected(guard, evidence.into_evidence()))
}

fn explain_value<T, R: ValueRenderer<T>>(
    rejected: Option<(&Mutex<T>, ValueRejection<'_, T>)>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    match rejected {
        None => failure.relation("contains a value that satisfies the assertions"),
        Some((actual, ValueRejection::Locked)) => {
            explain_held_lock(actual, "Mutex", failure, context)
        }
        Some((actual, ValueRejection::Rejected(guard, evidence))) => failure
            .actual(lock_data(context.render(), actual, "Mutex", &*guard))
            .relation("contains a value that does not satisfy the assertions")
            .evidence(evidence),
    }
}

impl<T, R: Clone + ValueRenderer<T>, F> Expectation<Mutex<T>, R> for HasValueSatisfying<F>
where
    F: for<'a> Fn(AssertThat<'a, T, crate::mode::Capture, R>),
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mutex<T>: 'a;
    type Rejection<'a>
        = ValueRejection<'a, T>
    where
        Self: 'a,
        Mutex<T>: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        evaluate_value(actual, &self.0, context)
    }

    const KIND: FailureKind = FailureKind::Predicate;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Mutex<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_value(rejected, failure, context)
    }
}

/// Non-blocking assertions for Tokio's [`Mutex`] type.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioMutexAssertions<T, M: Mode, R = DebugRenderer> {
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
    /// Fails the assertion if the mutex is currently locked, without running the callback.
    /// Otherwise, the callback receives an assertion over the value in this chain's mode while
    /// the guard is held.
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
        R: Clone;
}

impl<T, M: Mode, R> TokioMutexAssertions<T, M, R> for AssertThat<'_, Mutex<T>, M, R> {
    #[track_caller]
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsLocked)
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.matches(IsNotLocked)
    }

    #[track_caller]
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
        R: Clone,
    {
        {
            // The guard is held while the callback runs and released before the chain continues.
            let guard = self.test_assertion(&IsNotLocked);
            self.satisfy_success(guard.as_deref(), assertions);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[tokio::test]
        async fn are_as_expected() {
            {
                let mutex = Mutex::new(42);
                let guard = mutex.lock().await;
                mutex.must().be_locked();
                drop(guard);
            }
            Mutex::new(42).must().not_be_locked();
            Mutex::new(42).must().be_free();
            Mutex::new(42).must().have_value_satisfying(|value| {
                value.is_equal_to(42);
            });
        }
    }

    mod observations {
        use core::cell::Cell;

        use tokio::sync::Mutex;

        use super::super::{HasValueSatisfying, IsNotLocked};
        use crate::{
            matchers::all_of, prelude::*, renderer::RenderingBudget, test_support::NoRenderer,
        };

        #[test]
        fn successful_acquisitions_are_released_between_siblings_without_a_renderer() {
            assert_that!(Mutex::new(7))
                .with_renderer(NoRenderer)
                .matches(all_of(matchers![IsNotLocked, IsNotLocked]));
        }

        #[test]
        fn callback_rejections_retain_truth_and_omissions_with_zero_child_budget() {
            let lock = Mutex::new(7);
            let budget = RenderingBudget::default().with_max_items(0);
            // The chain method runs the callback in the chain's mode, so each failure is its own.
            let failures = assert_that!(lock)
                .with_rendering_budget(budget)
                .capture(|it| {
                    it.has_value_satisfying(|value| {
                        value.is_equal_to(8).is_equal_to(9);
                    })
                });
            assert_that!(failures).has_length(2);
            // The reusable expectation nests its callback failures within the budget.
            let definition = HasValueSatisfying::new(|value: AssertThat<'_, i32, Capture>| {
                value.is_equal_to(8).is_equal_to(9);
            });
            let composed = assert_that!(lock)
                .with_rendering_budget(budget)
                .capture(|it| it.matches(definition));
            assert_that!(composed[0].omitted_children).is_equal_to(2);
            assert_that!(lock.try_lock()).is_ok();
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
            let failures = assert_that!(lock)
                .capture(|it| it.matches(all_of(matchers![&definition, &definition])));
            assert_that!(failures[0].children).has_length(2);
            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(lock.try_lock()).is_ok();
        }
    }

    mod renderer_contract {
        use tokio::sync::Mutex;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Mutex<i32>, Panic, NoRenderer>
                    => TokioMutexAssertions<i32, Panic, NoRenderer>
            );
        }
    }

    mod is_locked {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

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
            assert_that!(|| assert_that!(mutex).with_location(false).is_locked())
                .panics()
                .has_message()
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
            assert_that!(|| assert_that!(&mutex).with_location(false).is_not_locked())
                .panics()
                .has_message()
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
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            let guard = mutex.try_lock().unwrap();
            assert_caller_location!(
                assert_that!(mutex),
                has_value_satisfying(|value| {
                    value.is_equal_to(42);
                })
            );
            drop(guard);
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
        fn panics_with_the_callback_failure_when_the_value_does_not_satisfy_the_assertions() {
            let mutex = Mutex::new(42);

            assert_that!(|| {
                assert_that!(mutex)
                    .with_location(false)
                    .has_value_satisfying(|value| {
                        value.is_equal_to(43);
                    });
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: 43

                      Actual: 42
                    -------- assertr --------
                "});
        }

        #[tokio::test]
        async fn rejects_a_locked_mutex_without_running_the_callback() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            let failures = assert_that!(mutex)
                .capture(|it| it.has_value_satisfying(|_| panic!("assertions should not run")));
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is unexpectedly locked"));
            drop(guard);
        }
    }
}
