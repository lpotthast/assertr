use std::{any::Any, cell::RefCell, fmt::Debug, mem::ManuallyDrop, panic::UnwindSafe};

use actual::Actual;
use failure::Failure;
use indoc::formatdoc;

pub mod actual;
pub mod assertions;
pub mod condition;
pub mod failure;
pub mod util;

pub mod prelude {
    pub use crate::assert_that;
    pub use crate::assert_that_panic_by;
    pub use crate::assertions::array;
    pub use crate::assertions::array::ArrayAssertions;
    pub use crate::assertions::bool;
    pub use crate::assertions::boxed;
    pub use crate::assertions::debug;
    pub use crate::assertions::display;
    pub use crate::assertions::eq;
    pub use crate::assertions::eq::EqualityAssertions;
    pub use crate::assertions::hashmap;
    pub use crate::assertions::iter;
    pub use crate::assertions::iter::IntoIteratorAssertions;
    pub use crate::assertions::mutex;
    pub use crate::assertions::option;
    pub use crate::assertions::ord;
    pub use crate::assertions::panic_value;
    pub use crate::assertions::panic_value::PanicValueAssertions;
    pub use crate::assertions::path;
    pub use crate::assertions::range;
    pub use crate::assertions::ref_cell;
    pub use crate::assertions::result;
    pub use crate::assertions::slice;
    pub use crate::assertions::str_slice;
    pub use crate::assertions::string;
    pub use crate::condition::Condition;
    pub use crate::condition::ConditionAssertions;
    pub use crate::failure::Failure;
    pub use crate::AssertThat;
}

pub struct PanicValue(Box<dyn Any + Send>);

struct DetailMessages<'a>(&'a [String]);

struct DisplayString<'a>(&'a str);

impl<'a> Debug for DisplayString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl<'a> Debug for DetailMessages<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|it| DisplayString(it)))
            .finish()
    }
}

#[track_caller]
pub fn assert_that<'t, T, A: Into<Actual<'t, T>>>(actual: A) -> AssertThat<'t, T> {
    AssertThat::new(actual.into())
}

#[track_caller]
pub fn assert_that_panic_by<'t, F: FnOnce() -> R + UnwindSafe, R>(
    fun: F,
) -> AssertThat<'t, PanicValue> {
    assert_that(std::panic::catch_unwind(move || {
        fun();
    }))
    .with_detail_message("Function did not panic as expected!")
    .is_err()
    .map(|it| PanicValue(it.unwrap_owned()).into())
}

pub struct AssertThat<'t, T> {
    actual: ManuallyDrop<Actual<'t, T>>,

    subject_name: ManuallyDrop<Option<String>>,
    detail_messages: ManuallyDrop<RefCell<Vec<String>>>,
    print_location: bool,
    capture: bool,
    failures: ManuallyDrop<RefCell<Vec<String>>>,
    failures_captured: bool,
}

impl<'t, T> Drop for AssertThat<'t, T> {
    fn drop(&mut self) {
        if self.capture && !self.failures_captured {
            // Note: We cannot print the actual value, as we cannot add bounds to T,
            // as this would render this Drop implementation not being called for all AssertThat's!
            panic!("{}", String::from("You dropped an `assert_that(..)` value, on which `.with_capture(true)` was called, without actually capturing the assertions failures using `.capture_failures()`!"));
        }
    }
}

impl<'t, T> AssertThat<'t, T> {
    #[track_caller]
    pub(crate) fn new(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual: ManuallyDrop::new(actual),
            subject_name: ManuallyDrop::new(None),
            detail_messages: ManuallyDrop::new(RefCell::new(Vec::new())),
            print_location: true,
            capture: false,
            failures: ManuallyDrop::new(RefCell::new(Vec::new())),
            failures_captured: false,
        }
    }

    pub fn derive<U>(&'t self, mapper: impl FnOnce(&'t T) -> U) -> AssertThat<'t, U> {
        AssertThat {
            actual: ManuallyDrop::new(Actual::Owned(mapper(self.actual.borrowed()))),
            subject_name: ManuallyDrop::new(None), // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: ManuallyDrop::new(RefCell::new(Vec::new())), // TODO: keep messages?
            print_location: self.print_location,
            capture: self.capture,
            failures: ManuallyDrop::new(RefCell::new(Vec::new())), // TODO: keep failures?
            failures_captured: false,
        }
    }

    pub(crate) fn map_with_actual_already_taken<U>(
        mut self,
        mapper: impl FnOnce() -> Actual<'t, U>,
    ) -> AssertThat<'t, U> {
        let failures_captured = self.failures_captured;
        self.failures_captured = true; // Avoid panic on drop of self!

        AssertThat {
            actual: ManuallyDrop::new(mapper()),
            subject_name: ManuallyDrop::new(None), // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(ManuallyDrop::take(&mut self.detail_messages))
            },
            print_location: self.print_location,
            capture: self.capture,
            failures: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(ManuallyDrop::take(&mut self.failures))
            },
            failures_captured,
        }
    }

    pub(crate) fn map<U>(
        mut self,
        mapper: impl FnOnce(Actual<T>) -> Actual<U>,
    ) -> AssertThat<'t, U> {
        let failures_captured = self.failures_captured;
        self.failures_captured = true; // Avoid panic on drop of self!

        AssertThat {
            actual: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(mapper(ManuallyDrop::take(&mut self.actual)))
            },
            subject_name: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(ManuallyDrop::take(&mut self.subject_name))
            }, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(ManuallyDrop::take(&mut self.detail_messages))
            },
            print_location: self.print_location,
            capture: self.capture,
            failures: unsafe {
                // Safety: AssertThat's Drop impl does not use this field.
                ManuallyDrop::new(ManuallyDrop::take(&mut self.failures))
            },
            failures_captured,
        }
    }

    pub fn actual(&'t self) -> &'t T {
        self.actual.borrowed()
    }

    /// Gives the `actual` value contain in this assertion a descriptive name.
    /// This name will be part of panic messages when set.
    #[allow(dead_code)]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        *self.subject_name = Some(subject_name.into());
        self
    }

    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn add_detail_message(&self, message: impl Into<String>) {
        self.detail_messages.borrow_mut().push(message.into());
    }

    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_detail_message(self, message: impl Into<String>) -> Self {
        self.detail_messages.borrow_mut().push(message.into());
        self
    }

    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_conditional_detail_message<M: Into<String> + 'static>(
        self,
        condition: bool,
        message_provider: impl Fn(&Self) -> M,
    ) -> Self {
        if condition {
            let message = message_provider(&self);
            self.detail_messages.borrow_mut().push(message.into());
        }
        self
    }

    /// Control wether the location is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_capture(mut self, value: bool) -> Self {
        self.capture = value;
        self
    }

    /// Control wether the location is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_location(mut self, value: bool) -> Self {
        self.print_location = value;
        self
    }

    pub fn fail_using<F: Failure<'t>>(&'t self, failure_provider: impl Fn(&Self) -> F) {
        let failure = failure_provider(self);
        self.fail(failure);
    }

    #[track_caller]
    pub fn fail(&self, failure: impl Failure<'t>) {
        let caller_location = std::panic::Location::caller();

        let err = match (self.print_location, self.detail_messages.borrow().len()) {
            (false, 0) => formatdoc! {"
                    -------- assertr --------
                    {failure}
                    -------- assertr --------
                "
            },
            (false, _) => formatdoc! {"
                    -------- assertr --------
                    {failure}

                    Details: {detail_messages:#?}
                    -------- assertr --------
                ",
                detail_messages = DetailMessages(self.detail_messages.borrow().as_ref()),
            },
            (true, 0) => formatdoc! {"
                    -------- assertr --------
                    Assertion failed at {file}:{line}:{column}

                    {failure}
                    -------- assertr --------
                ",
                file = caller_location.file(),
                line = caller_location.line(),
                column = caller_location.column(),
            },
            (true, _) => formatdoc! {"
                    -------- assertr --------
                    Assertion failed at {file}:{line}:{column}

                    {failure}

                    Details: {detail_messages:#?}
                    -------- assertr --------
                ",
                file = caller_location.file(),
                line = caller_location.line(),
                column = caller_location.column(),
                detail_messages = self.detail_messages.borrow(),
            },
        };

        match self.capture {
            true => self.failures.borrow_mut().push(err),
            false => panic!("{err}"),
        };
    }

    #[must_use]
    pub fn capture_failures(mut self) -> Vec<String> {
        self.failures_captured = true;
        self.failures.take()
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn with_capture_yields_failures_and_does_not_panic() {
        let failures = assert_that(42)
            .with_location(false)
            .with_capture(true)
            .is_greater_than(100)
            .is_equal_to(1)
            .capture_failures();

        assert_that(failures.as_slice())
            .has_length(2)
            .contains_exactly([
                formatdoc! {"
                    -------- assertr --------
                    Actual: 42

                    is not greater than

                    Expected: 100
                    -------- assertr --------
                "},
                formatdoc! {"
                    -------- assertr --------
                    Expected: 1

                      Actual: 42
                    -------- assertr --------
                "},
            ]);
    }

    #[test]
    fn dropping_a_capturing_assert_panics_when_failures_occurred_which_were_not_captured() {
        let assert = assert_that(42)
            .with_location(false)
            .with_capture(true)
            .is_equal_to(43);

        assert_that_panic_by(move || drop(assert))
            .has_type::<String>()
            .is_equal_to(format!("You dropped an `assert_that(..)` value, on which `.with_capture(true)` was called, without actually capturing the assertions failures using `.capture_failures()`!"));
    }
}
