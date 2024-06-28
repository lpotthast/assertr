use crate::{failure::GenericFailure, AssertThat};
use crate::{tracking::AssertionTracking, Mode};
use std::fmt::Debug;
use std::sync::Mutex;

pub trait MutexAssertions {
    fn is_locked(self) -> Self;
    fn is_not_locked(self) -> Self;
}

impl<'t, T: Debug, M: Mode> MutexAssertions for AssertThat<'t, Mutex<T>, M> {
    #[track_caller]
    fn is_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if let Ok(guard) = actual.try_lock() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: Mutex {{ data: {guard:#?}, poisoned: {poisoned} }}\n\nto be locked, but it wasn't!",
                poisoned = actual.is_poisoned())
            });
        }
        self
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if let Err(_err) = actual.try_lock() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: Mutex {{ data: <locked>, poisoned: {poisoned} }}\n\nto not be locked, but it was!",
                poisoned = actual.is_poisoned())
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod is_locked {
        use indoc::formatdoc;
        use std::sync::Mutex;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock();
            assert_that(&mutex).is_locked();
            drop(guard);
        }

        #[test]
        fn panics_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that_panic_by(|| assert_that(mutex).with_location(false).is_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected: Mutex {{ data: 42, poisoned: false }}

                    to be locked, but it wasn't!
                    -------- assertr --------
                "});
        }
    }

    mod is_not_locked {
        use indoc::formatdoc;
        use std::sync::Mutex;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that(mutex).is_not_locked();
        }

        #[test]
        fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock();
            assert_that_panic_by(|| assert_that(&mutex).with_location(false).is_not_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected: Mutex {{ data: <locked>, poisoned: false }}

                    to not be locked, but it was!
                    -------- assertr --------
                "});
            drop(guard);
        }
    }
}
