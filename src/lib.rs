use std::{
    any::{Any, TypeId},
    cell::RefCell,
    fmt::Debug,
    panic::UnwindSafe,
};

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

pub struct PanicValue(Box<dyn Any>);

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

pub trait Mode: Default + Clone + 'static {
    fn is_capture(&self) -> bool {
        TypeId::of::<Self>() == TypeId::of::<Capture>()
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FailFast;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FailLate;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Capture {
    captured: bool,
}

impl Mode for FailFast {}
impl Mode for FailLate {}
impl Mode for Capture {}

#[track_caller]
pub fn assert_that<'t, T, A: Into<Actual<'t, T>>>(actual: A) -> AssertThat<'t, T, FailFast> {
    AssertThat::new(actual.into())
}

#[track_caller]
pub fn assert_that_panic_by<'t, F: FnOnce() -> R + UnwindSafe, R>(
    fun: F,
) -> AssertThat<'t, PanicValue, FailFast> {
    assert_that(std::panic::catch_unwind(move || {
        fun();
    }))
    .with_detail_message("Function did not panic as expected!")
    .is_err()
    .map(|it| PanicValue(it.unwrap_owned()).into())
}

pub struct AssertThat<'t, T, M: Mode> {
    actual: Actual<'t, T>,

    subject_name: Option<String>,
    detail_messages: RefCell<Vec<String>>,
    print_location: bool,
    failures: RefCell<Vec<String>>,

    mode: M,
}

// Drop cannot be specialized...
impl Drop for Capture {
    fn drop(&mut self) {
        if !self.captured {
            // Note: We cannot print the actual value, as we cannot add bounds to T,
            // as this would render this Drop implementation not being called for all AssertThat's!
            panic!("{}", String::from("You dropped an `assert_that(..)` value, on which `.with_capture(true)` was called, without actually capturing the assertions failures using `.capture_failures()`!"));
        }
    }
}

impl<'t, T> AssertThat<'t, T, FailFast> {
    #[track_caller]
    pub(crate) fn new(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            subject_name: None,
            detail_messages: RefCell::new(Vec::new()),
            print_location: true,
            failures: RefCell::new(Vec::new()),
            mode: FailFast,
        }
    }
}

impl<'t, T> AssertThat<'t, T, Capture> {
    #[must_use]
    pub fn capture_failures(mut self) -> Vec<String> {
        self.mode.captured = true;
        self.failures.take()
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    pub fn actual(&self) -> &Actual<T> {
        &self.actual
    }

    pub fn actual_ref(&self) -> &T {
        self.actual().borrowed()
    }

    pub fn derive<'u, U>(&'t self, mapper: impl FnOnce(&'t T) -> U) -> AssertThat<'u, U, M> {
        AssertThat {
            actual: Actual::Owned(mapper(self.actual().borrowed())),
            subject_name: None, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: RefCell::new(Vec::new()), // TODO: keep messages?
            print_location: self.print_location,
            failures: RefCell::new(Vec::new()), // TODO: keep failures?
            mode: self.mode.clone(),            // Clone safe?
        }
    }

    pub(crate) fn map<U>(
        self,
        mapper: impl FnOnce(Actual<T>) -> Actual<U>,
    ) -> AssertThat<'t, U, M> {
        AssertThat {
            actual: mapper(self.actual),
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            failures: self.failures,
            mode: self.mode,
        }
    }

    /// Gives the `actual` value contain in this assertion a descriptive name.
    /// This name will be part of panic messages when set.
    #[allow(dead_code)]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.subject_name = Some(subject_name.into());
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
    pub fn with_conditional_detail_message<DM: Into<String> + 'static>(
        self,
        condition: bool,
        message_provider: impl Fn(&Self) -> DM,
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
    pub fn with_capture(self) -> AssertThat<'t, T, Capture> {
        AssertThat {
            actual: self.actual,
            subject_name: self.subject_name,
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            failures: self.failures,
            mode: Capture { captured: false },
        }
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

        match self.mode.is_capture() {
            true => self.failures.borrow_mut().push(err),
            false => panic!("{err}"),
        };
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
            .with_capture()
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
            .with_capture()
            .is_equal_to(43);

        assert_that_panic_by(move || drop(assert))
            .has_type::<String>()
            .is_equal_to(format!("You dropped an `assert_that(..)` value, on which `.with_capture(true)` was called, without actually capturing the assertions failures using `.capture_failures()`!"));
    }
}
