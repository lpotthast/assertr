use crate::renderer::{Rendered, RenderingContext};
use crate::{
    AssertThat, Mode, failure::Fact, failure::FailureKind, renderer::DebugRenderer,
    renderer::ValueRenderer,
};
use crate::{expectation::AssertionContext, expectation::Expectation, failure::FailureBuilder};
use std::sync::MutexGuard;
use std::sync::{Mutex, TryLockError};

property_expectation! {
    /// Checks whether a mutex is poisoned.
    pub struct IsPoisoned for<T> Mutex<T>;
    kind Other;
    check |actual| actual.is_poisoned();
    relations "is poisoned", "is not poisoned";
    hidden;
}

property_expectation! {
    /// Checks whether a mutex is not poisoned.
    pub struct IsNotPoisoned for<T> Mutex<T>;
    kind Other;
    check |actual| !actual.is_poisoned();
    relations "is not poisoned", "is unexpectedly poisoned";
    hidden;
}

/// Renders lock data that another guard holds, so the lock cannot be observed.
pub(crate) fn locked_data<O: ?Sized, R>(
    render: RenderingContext<'_, R>,
    actual: &O,
    lock: &'static str,
) -> Rendered {
    render.unavailable_struct_field(actual, lock, "data", "<locked>")
}

/// Explains a rejected "is locked" check with the data observed through an acquired guard.
///
/// Shared by the standard and Tokio lock expectations. The caller releases the guard after
/// this returns.
pub(crate) fn explain_acquired_lock<O: ?Sized, T: ?Sized, R: ValueRenderer<T>>(
    actual: &O,
    lock: &'static str,
    data: &T,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    failure
        .actual(context.render().struct_field(actual, lock, "data", data))
        .relation("is not locked")
}

/// Explains a rejected "is not locked" check whose lock is held elsewhere.
///
/// Shared by the standard and Tokio lock expectations.
pub(crate) fn explain_held_lock<O: ?Sized, R>(
    actual: &O,
    lock: &'static str,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    failure
        .actual(locked_data(context.render(), actual, lock))
        .relation("is unexpectedly locked")
}

fn with_poison_note(failure: FailureBuilder, poisoned: bool) -> FailureBuilder {
    if poisoned {
        failure.fact(Fact::note("The mutex is poisoned."))
    } else {
        failure
    }
}

/// Observes whether a mutex is locked, retaining an acquired guard when the check rejects.
#[derive(Debug, Clone, Copy)]
pub struct IsLocked;

/// An acquired guard and the observed poison state from a rejected lock expectation.
/// Explanation releases the guard before the executor raises or evaluates another child.
pub struct UnlockedRejection<'a, T> {
    guard: MutexGuard<'a, T>,
    poisoned: bool,
}

impl<T, R: ValueRenderer<T>> Expectation<Mutex<T>, R> for IsLocked {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mutex<T>: 'a;
    type Rejection<'a>
        = UnlockedRejection<'a, T>
    where
        Self: 'a,
        Mutex<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        match actual.try_lock() {
            Ok(guard) => Err(UnlockedRejection {
                guard,
                poisoned: false,
            }),
            Err(TryLockError::Poisoned(error)) => Err(UnlockedRejection {
                guard: error.into_inner(),
                poisoned: true,
            }),
            Err(TryLockError::WouldBlock) => Ok(()),
        }
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
            Some((actual, UnlockedRejection { guard, poisoned })) => {
                let failure = explain_acquired_lock(actual, "Mutex", &*guard, failure, context);
                // Release before raising or observing the next composed assertion.
                drop(guard);
                with_poison_note(failure, poisoned)
            }
        }
    }
}

/// Observes whether a mutex can be acquired, returning the acquired guard on success.
/// Poisoned acquisition also counts as unlocked, matching [`MutexAssertions::is_not_locked`].
#[derive(Debug, Clone, Copy)]
pub struct IsNotLocked;

impl<T, R> Expectation<Mutex<T>, R> for IsNotLocked {
    type Success<'a>
        = MutexGuard<'a, T>
    where
        Self: 'a,
        Mutex<T>: 'a;
    type Rejection<'a>
        = bool
    where
        Self: 'a,
        Mutex<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Mutex<T>,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, bool> {
        match actual.try_lock() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(TryLockError::WouldBlock) => Err(actual.is_poisoned()),
        }
    }

    const KIND: FailureKind = FailureKind::Other;
    fn explain(
        &self,
        rejected: Option<(&Mutex<T>, bool)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("is not locked"),
            Some((actual, poisoned)) => with_poison_note(
                explain_held_lock(actual, "Mutex", failure, context),
                poisoned,
            ),
        }
    }
}

/// Assertions for the lock and poison state of [`Mutex`].
///
/// Lock state is observed with [`Mutex::try_lock`]. A successful or poisoned acquisition means
/// unlocked. [`TryLockError::WouldBlock`] means locked.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MutexAssertions<T, R = DebugRenderer> {
    /// Asserts that this mutex is locked.
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that this mutex is not locked.
    fn is_not_locked(self) -> Self;

    /// Alias of [`MutexAssertions::is_not_locked`].
    #[track_caller]
    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }

    /// Asserts that this mutex is poisoned.
    fn is_poisoned(self) -> Self;

    /// Asserts that this mutex is not poisoned.
    fn is_not_poisoned(self) -> Self;
}

impl<T, M: Mode, R> MutexAssertions<T, R> for AssertThat<'_, Mutex<T>, M, R> {
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
    fn is_poisoned(self) -> Self {
        self.matches(IsPoisoned)
    }

    #[track_caller]
    fn is_not_poisoned(self) -> Self {
        self.matches(IsNotPoisoned)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;
        use std::sync::Mutex;

        #[test]
        fn are_as_expected() {
            {
                let mutex = Mutex::new(42);
                let guard = mutex.lock();
                mutex.must().be_locked();
                drop(guard);
            }
            Mutex::new(42).must().not_be_locked();
            Mutex::new(42).must().be_free();
            super::poisoned_mutex().must().be_poisoned();
            Mutex::new(42).must().not_be_poisoned();
        }
    }

    use crate::prelude::*;
    use std::sync::Mutex;

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };
        use std::sync::Mutex;

        struct Secret;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Mutex<i32>, Panic, NoRenderer>
                    => MutexAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn failures_render_the_inner_value_with_the_active_renderer() {
            let failures = assert_that!(Mutex::new(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(MutexAssertions::is_locked);

            assert_that!(failures[0].to_string()).contains(format!("data: {SENTINEL},"));
        }
    }

    mod shared_observations {
        use super::*;
        use crate::{
            assertions::std::mutex::{IsLocked, IsNotLocked},
            matchers::{all_of, any_of},
            test_support::NoRenderer,
        };

        #[test]
        fn compositions_release_guards_before_the_next_child_and_before_raising() {
            let mutex = Mutex::new(7);
            for limit in [0, 2] {
                let failures = assert_that!(mutex)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| it.matches(any_of(matchers![IsLocked, IsLocked])));
                assert_that!(failures).has_length(1);
                assert_that!(mutex.is_poisoned()).is_false();
            }
            assert_that!(mutex)
                .with_renderer(NoRenderer)
                .matches(all_of(matchers![IsNotLocked, IsNotLocked]));
            assert_that!(mutex.try_lock().is_ok()).is_true();
        }
    }

    fn poisoned_mutex() -> Mutex<i32> {
        let mutex = Mutex::new(42);
        std::thread::scope(|scope| {
            let panic = scope
                .spawn(|| {
                    let _guard = mutex.lock().expect("the mutex should initially be healthy");
                    panic!("poison the mutex");
                })
                .join();
            assert_that!(panic).is_err();
        });
        mutex
    }

    mod is_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            assert_caller_location!(assert_that!(mutex), is_locked());
        }

        #[test]
        fn succeeds_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock();
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

        #[test]
        fn treats_a_poisoned_but_available_mutex_as_not_locked() {
            let mutex = super::poisoned_mutex();
            assert_that_panic_by(|| assert_that!(mutex).with_location(false).is_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `mutex`

                    Actual: Mutex {{
                        data: 42,
                    }}

                    is not locked

                    Details:
                      - The mutex is poisoned.
                    -------- assertr --------
                "});
        }
    }

    mod is_not_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        fn caller_location_is_as_expected() {
            let mutex = Mutex::new(42);
            let _guard = mutex.lock().unwrap();
            assert_caller_location!(assert_that!(mutex), is_not_locked());
        }

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that!(mutex).is_not_locked();
        }

        #[test]
        fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock();
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

        #[test]
        fn succeeds_when_poisoned_but_not_locked() {
            let mutex = super::poisoned_mutex();
            assert_that!(mutex).is_not_locked();
        }
    }

    /// Synonym of `is_not_locked`. The fluent name and caller location are pinned here. The
    /// behavior is covered by that module.
    mod is_free {
        use crate::prelude::*;
        use std::sync::Mutex;

        #[test]
        fn caller_location_is_as_expected() {
            let lock = Mutex::new(42);
            let _guard = lock.lock().unwrap();
            assert_caller_location!(assert_that!(lock), is_free());
        }
    }

    mod is_poisoned {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!(Mutex::new(42)), is_poisoned());
        }

        #[test]
        fn succeeds_when_poisoned() {
            assert_that!(super::poisoned_mutex()).is_poisoned();
        }

        #[test]
        fn panics_when_not_poisoned() {
            assert_that_panic_by(|| {
                assert_that_owned!(Mutex::new(42))
                    .with_location(false)
                    .is_poisoned()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `Mutex::new(42)`

                is not poisoned
                -------- assertr --------
            "});
        }
    }

    mod is_not_poisoned {
        use crate::prelude::*;

        use std::sync::Mutex;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!(super::poisoned_mutex()),
                is_not_poisoned()
            );
        }

        #[test]
        fn succeeds_when_not_poisoned() {
            assert_that!(Mutex::new(42)).is_not_poisoned();
        }

        #[test]
        fn reports_the_negated_relation() {
            let failures = assert_that_owned!(super::poisoned_mutex())
                .capture(MutexAssertions::is_not_poisoned);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is unexpectedly poisoned"));
        }
    }
}
