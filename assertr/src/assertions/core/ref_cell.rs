use crate::{AssertThat, Mode, ValueRenderer};
use alloc::string::String;
use core::cell::RefCell;
use core::fmt::Write;
use indoc::writedoc;

/// Assertions for the dynamic borrow state of a [`RefCell`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RefCellAssertions<T, R> {
    /// Asserts that the `RefCell` has an active shared or mutable borrow.
    fn is_borrowed(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the `RefCell` has an active mutable borrow.
    fn is_mutably_borrowed(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the `RefCell` has no active mutable borrow.
    ///
    /// Immutable borrows are allowed.
    fn is_not_mutably_borrowed(self) -> Self;
}

impl<T, M: Mode, R> RefCellAssertions<T, R> for AssertThat<'_, RefCell<T>, M, R> {
    #[track_caller]
    fn is_borrowed(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_borrow_mut().is_ok() {
            let value = self
                .actual()
                .try_borrow()
                .expect("the borrow check already succeeded");
            let actual = self.render_struct_field("RefCell", "value", &*value);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is not borrowed.

                    Expected: RefCell to have an active borrow.
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_mutably_borrowed(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_borrow().is_ok() {
            let value = self
                .actual()
                .try_borrow()
                .expect("the borrow check already succeeded");
            let actual = self.render_struct_field("RefCell", "value", &*value);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is not mutably borrowed.

                    Expected: RefCell to be borrowed mutably.
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_mutably_borrowed(self) -> Self {
        self.track_assertion();
        if self.actual().try_borrow().is_err() {
            let actual = Self::render_unavailable_struct_field("RefCell", "value", "<borrowed>");
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?} is mutably borrowed.

                    Expected: RefCell to not be borrowed mutably.
                "}
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            cell.must().be_borrowed();
            drop(borrow);
        }

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
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: RefCell {{
                        value: 42,
                    }} is not borrowed.

                    Expected: RefCell to have an active borrow.
                    -------- assertr --------
                "});
        }
    }

    mod is_mutably_borrowed {
        use crate::prelude::*;
        use std::cell::RefCell;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            cell.must().be_mutably_borrowed();
            drop(borrow);
        }

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            RefCell::new(42).must().not_be_mutably_borrowed();
        }

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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: RefCell {{
                        value: <borrowed>,
                    }} is mutably borrowed.

                    Expected: RefCell to not be borrowed mutably.
                    -------- assertr --------
                "});
            drop(borrow);
        }
    }
}
