use crate::Mode;
use crate::{failure::GenericFailure, AssertThat};
use std::fmt::Debug;
use std::sync::Mutex;

impl<'t, T: Debug, M: Mode> AssertThat<'t, Mutex<T>, M> {
    #[track_caller]
    pub fn is_locked(self) -> Self {
        let actual = self.actual().borrowed();
        if let Ok(guard) = actual.try_lock() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: Mutex {{ data: {guard:#?}, poisoned: {poisoned} }} \n\nto be locked, but it wasn't!",
                poisoned = actual.is_poisoned())
            });
        }
        self
    }

    #[track_caller]
    pub fn is_not_locked(self) -> Self {
        let actual = self.actual().borrowed();
        if let Err(_err) = actual.try_lock() {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: Mutex {{ data: <locked>, poisoned: {poisoned} }} \n\nto not be locked, but it was!",
                poisoned = actual.is_poisoned())
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::assert_that;

    #[test]
    fn is_locked_succeeds_when_locked() {
        let mutex = Mutex::new(42);
        let guard = mutex.lock();
        assert_that(&mutex).is_locked();
        drop(guard);
    }

    #[test]
    fn is_not_locked_succeeds_when_not_locked() {
        let mutex = Mutex::new(42);
        assert_that(mutex).is_not_locked();
    }
}
