use crate::Mode;
use crate::{failure::GenericFailure, AssertThat};
use std::cell::RefCell;
use std::fmt::Debug;

impl<'t, T: Debug, M: Mode> AssertThat<'t, RefCell<T>, M> {
    /// Check that the RefCell is immutably or mutably borrowed.
    #[track_caller]
    pub fn is_borrowed(self) -> Self {
        let actual = self.actual().borrowed();

        if actual.try_borrow_mut().is_ok() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?} is not borrowed.\n\nExpected: RefCell to be borrowed (immutably) at least once."
                ),
            });
        }
        self
    }

    /// Check that the RefCell is mutably borrowed.
    #[track_caller]
    pub fn is_mutably_borrowed(self) -> Self {
        let actual = self.actual().borrowed();

        if actual.try_borrow().is_ok() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?} is not mutably borrowed.\n\nExpected: RefCell to be borrowed mutably."
                ),
            });
        }
        self
    }

    /// Check that the RefCell is not mutably borrowed, wither by being not borrowed at all, or only borrowed immutably.
    #[track_caller]
    pub fn is_not_mutably_borrowed(self) -> Self {
        let actual = self.actual().borrowed();

        if actual.try_borrow_mut().is_ok() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?} is mutably borrowed.\n\nExpected: RefCell to not be borrowed mutably."
                ),
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::assert_that;

    #[test]
    fn is_borrowed_succeeds_when_borrowed() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow();
        assert_that(&cell).is_borrowed();
        drop(borrow);
    }

    #[test]
    fn is_borrowed_succeeds_when_mutably_borrowed() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow_mut();
        assert_that(&cell).is_borrowed();
        drop(borrow);
    }

    #[test]
    fn is_mutably_borrowed_succeeds_when_mutably_borrowed() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow_mut();
        assert_that(&cell).is_borrowed();
        assert_that(&cell).is_mutably_borrowed();
        drop(borrow);
    }

    #[test]
    fn is_not_mutably_borrowed_succeeds_when_not_borrowed_at_all() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow();
        assert_that(&cell).is_not_mutably_borrowed();
        drop(borrow);
    }

    #[test]
    fn is_not_mutably_borrowed_succeeds_when_immutably_borrowed() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow();
        assert_that(&cell).is_not_mutably_borrowed();
        drop(borrow);
    }
}
