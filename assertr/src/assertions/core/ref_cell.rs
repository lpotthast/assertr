use crate::AssertThat;
use crate::{Mode, tracking::AssertionTracking};
use core::cell::RefCell;
use core::fmt::Debug;
use indoc::writedoc;
use std::fmt::Write;

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RefCellAssertions {
    /// Check that the `RefCell` is immutably or mutably borrowed.
    fn is_borrowed(self) -> Self;

    /// Check that the `RefCell` is mutably borrowed.
    fn is_mutably_borrowed(self) -> Self;

    /// Check that the `RefCell` is not mutably borrowed, wither by being not borrowed at all, or only borrowed immutably.
    #[cfg_attr(feature = "fluent", fluent_alias("not_be_mutably_borrowed"))]
    fn is_not_mutably_borrowed(self) -> Self;
}

impl<T: Debug, M: Mode> RefCellAssertions for AssertThat<'_, RefCell<T>, M> {
    #[track_caller]
    fn is_borrowed(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if actual.try_borrow_mut().is_ok() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is not borrowed.

                    Expected: RefCell to be borrowed (immutably) at least once.
                ",actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_mutably_borrowed(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if actual.try_borrow().is_ok() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is not mutably borrowed.

                    Expected: RefCell to be borrowed mutably.
                ",actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_mutably_borrowed(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if actual.try_borrow().is_err() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is mutably borrowed.

                    Expected: RefCell to not be borrowed mutably.
                ",actual = self.actual()}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod is_borrowed {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::cell::RefCell;

        #[test]
        fn succeeds_when_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            assert_that!(&cell).is_borrowed();
            drop(borrow);
        }

        #[test]
        fn succeeds_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(&cell).is_borrowed();
            drop(borrow);
        }

        #[test]
        fn panics_when_not_borrowed() {
            let cell = RefCell::new(42);
            assert_that_panic_by(|| assert_that!(&cell).with_location(false).is_borrowed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RefCell {{
                        value: 42,
                    }} is not borrowed.

                    Expected: RefCell to be borrowed (immutably) at least once.
                    -------- assertr --------
                "#});
        }
    }

    mod is_mutably_borrowed {
        use crate::prelude::*;
        use std::cell::RefCell;

        #[test]
        fn succeeds_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(&cell).is_borrowed();
            assert_that!(&cell).is_mutably_borrowed();
            drop(borrow);
        }
    }

    mod is_not_mutably_borrowed {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::cell::RefCell;

        #[test]
        fn succeeds_when_not_borrowed_at_all() {
            let cell = RefCell::new(42);
            assert_that!(&cell).is_not_mutably_borrowed();
        }

        #[test]
        fn succeeds_when_immutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            assert_that!(&cell).is_not_mutably_borrowed();
            drop(borrow);
        }

        #[test]
        fn panics_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that_panic_by(|| {
                assert_that!(&cell)
                    .with_location(false)
                    .is_not_mutably_borrowed()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RefCell {{
                        value: <borrowed>,
                    }} is mutably borrowed.

                    Expected: RefCell to not be borrowed mutably.
                    -------- assertr --------
                "#});
            drop(borrow);
        }
    }
}
