use std::sync::{Mutex, TryLockError};

use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};

/// Assertions for the lock and poison state of [`Mutex`].
///
/// Lock state is observed with [`Mutex::try_lock`]. A successful or poisoned acquisition means
/// unlocked. [`TryLockError::WouldBlock`] means locked.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait MutexAssertions<T, R> {
    /// Asserts that this mutex is locked.
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that this mutex is not locked.
    fn is_not_locked(self) -> Self;

    /// Alias of [`MutexAssertions::is_not_locked`].
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
        self.track_assertion();
        let actual = self.actual();
        {
            let acquired = match actual.try_lock() {
                Ok(guard) => Some(guard),
                Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
                Err(TryLockError::WouldBlock) => None,
            };
            if let Some(guard) = acquired {
                let mut failure = self
                    .failure(FailureKind::Other)
                    .actual(self.render().struct_field(actual, "Mutex", "data", &*guard))
                    .relation("is not locked");
                if actual.is_poisoned() {
                    failure = failure.note("The mutex is poisoned.");
                }
                // Release the lock before raising, so a panic does not poison the mutex.
                drop(guard);
                failure.raise();
            }
        }
        self
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if matches!(actual.try_lock(), Err(TryLockError::WouldBlock)) {
            let mut failure = self
                .failure(FailureKind::Other)
                .actual(
                    self.render()
                        .unavailable_struct_field(actual, "Mutex", "data", "<locked>"),
                )
                .relation("is unexpectedly locked");
            if actual.is_poisoned() {
                failure = failure.note("The mutex is poisoned.");
            }
            failure.raise();
        }
        self
    }

    #[track_caller]
    fn is_poisoned(self) -> Self {
        self.track_assertion();
        if !self.actual().is_poisoned() {
            self.failure(FailureKind::Other)
                .relation("is not poisoned")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_not_poisoned(self) -> Self {
        self.track_assertion();
        if self.actual().is_poisoned() {
            self.failure(FailureKind::Other)
                .relation("is unexpectedly poisoned")
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::sync::Mutex;

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
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

            assert_that!(failures[0].description()).contains(format!("data: {SENTINEL},"));
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock();
            mutex.must().be_locked();
            drop(guard);
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().not_be_locked();
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

    /// Synonym of `is_not_locked`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_free {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;
        #[cfg(feature = "fluent")]
        use std::sync::Mutex;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().be_free();
        }
    }

    mod is_poisoned {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            super::poisoned_mutex().must().be_poisoned();
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
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().not_be_poisoned();
        }

        #[test]
        fn succeeds_when_not_poisoned() {
            assert_that!(Mutex::new(42)).is_not_poisoned();
        }

        #[test]
        fn panics_when_poisoned() {
            assert_that_panic_by(|| {
                assert_that_owned!(super::poisoned_mutex())
                    .with_location(false)
                    .is_not_poisoned()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `super::poisoned_mutex()`

                is unexpectedly poisoned
                -------- assertr --------
            "});
        }
    }
}
