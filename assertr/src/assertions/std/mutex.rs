use core::fmt::Write;
use indoc::writedoc;
use std::sync::{Mutex, TryLockError};

use crate::{AssertThat, AssertionRenderer, Mode, tracking::AssertionTracking};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait MutexAssertions<T, R> {
    /// Asserts that this mutex is locked.
    /// Note that implementations may try to acquire the lock in order to check its state.
    fn is_locked(self) -> Self
    where
        R: AssertionRenderer<T>;

    /// Asserts that this mutex is not locked.
    /// Note that implementations may try to acquire the lock in order to check its state.
    #[cfg_attr(feature = "fluent", fluent_alias("not_be_locked"))]
    fn is_not_locked(self) -> Self;

    /// Asserts that this mutex is not locked.
    /// Note that implementations may try to acquire the lock in order to check its state.
    ///
    /// Synonym for [`Self::is_not_locked`].
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
        R: AssertionRenderer<T>,
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
                let data = self.render_value(&*guard);
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: Mutex {{ data: {data:#?}, poisoned: {poisoned} }}

                        to be locked, but it wasn't!
                    ", poisoned = actual.is_poisoned()}
                });
            }
        }
        self
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if matches!(actual.try_lock(), Err(TryLockError::WouldBlock)) {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Mutex {{ data: <locked>, poisoned: {poisoned} }}

                    to not be locked, but it was!
                ", poisoned = actual.is_poisoned()}
            });
        }
        self
    }

    #[track_caller]
    fn is_poisoned(self) -> Self {
        self.track_assertion();
        if !self.actual().is_poisoned() {
            self.fail(|w: &mut String| {
                writeln!(w, "Expected the mutex to be poisoned, but it was not!")
            });
        }
        self
    }

    #[track_caller]
    fn is_not_poisoned(self) -> Self {
        self.track_assertion();
        if self.actual().is_poisoned() {
            self.fail(|w: &mut String| {
                writeln!(w, "Expected the mutex to not be poisoned, but it was!")
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    fn poisoned_mutex() -> Mutex<i32> {
        let mutex = Mutex::new(42);
        std::thread::scope(|scope| {
            let panic = scope
                .spawn(|| {
                    let _guard = mutex.lock().expect("the mutex should initially be healthy");
                    panic!("poison the mutex");
                })
                .join();
            assert!(panic.is_err());
        });
        mutex
    }

    mod is_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

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
                    Expected: Mutex {{ data: 42, poisoned: false }}

                    to be locked, but it wasn't!
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
                    Expected: Mutex {{ data: 42, poisoned: true }}

                    to be locked, but it wasn't!
                    -------- assertr --------
                "});
        }
    }

    mod is_not_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

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
                    Expected: Mutex {{ data: <locked>, poisoned: false }}

                    to not be locked, but it was!
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

    mod is_free {
        use crate::prelude::*;
        use std::sync::Mutex;

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that!(mutex).is_free();
        }
    }

    mod is_poisoned {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        fn succeeds_when_poisoned() {
            assert_that!(super::poisoned_mutex()).is_poisoned();
        }

        #[test]
        fn panics_when_not_poisoned() {
            assert_that_panic_by(|| {
                assert_that!(Mutex::new(42))
                    .with_location(false)
                    .is_poisoned()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expected the mutex to be poisoned, but it was not!
                -------- assertr --------
            "});
        }
    }

    mod is_not_poisoned {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::sync::Mutex;

        #[test]
        fn succeeds_when_not_poisoned() {
            assert_that!(Mutex::new(42)).is_not_poisoned();
        }

        #[test]
        fn panics_when_poisoned() {
            assert_that_panic_by(|| {
                assert_that!(super::poisoned_mutex())
                    .with_location(false)
                    .is_not_poisoned()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expected the mutex to not be poisoned, but it was!
                -------- assertr --------
            "});
        }
    }
}
