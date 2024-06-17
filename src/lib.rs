use std::{any::Any, fmt::Debug};

use actual::Actual;
use failure::Failure;
use indoc::formatdoc;

pub mod actual;
pub mod assertions;
pub mod condition;
pub mod failure;

pub mod prelude {
    pub use crate::assert_that;
    pub use crate::assert_that_panic_by;
    pub use crate::assertions::any;
    pub use crate::assertions::array;
    pub use crate::assertions::bool;
    pub use crate::assertions::debug;
    pub use crate::assertions::display;
    pub use crate::assertions::eq;
    pub use crate::assertions::hashmap;
    pub use crate::assertions::iter;
    pub use crate::assertions::iter::IntoIteratorAssertions;
    pub use crate::assertions::option;
    pub use crate::assertions::ord;
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

#[track_caller]
pub fn assert_that<'t, T, A: Into<Actual<'t, T>>>(actual: A) -> AssertThat<'t, T> {
    AssertThat::new(actual.into())
}

#[track_caller]
pub fn assert_that_panic_by<'t, R, F: Fn() -> R>(fun: F) -> AssertThat<'t, Box<dyn Any + Send>> {
    let result: Result<(), Box<dyn Any + Send>> =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            fun();
        }));
    assert_that(Actual::Owned(result))
        .with_additional_message("Function did not panic as expected!")
        .is_err()
}

pub struct AssertThat<'t, T> {
    actual: Actual<'t, T>,
    print_location: bool,
    additional_messages: Vec<Box<dyn Debug>>,
}

impl<'t, T> AssertThat<'t, T> {
    #[track_caller]
    pub(crate) fn new(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            print_location: true,
            additional_messages: Vec::new(),
        }
    }

    fn derive<U>(&'t self, mapper: impl Fn(&'t T) -> U) -> AssertThat<'t, U> {
        AssertThat {
            actual: Actual::Owned(mapper(self.actual.borrowed())),
            print_location: self.print_location,
            additional_messages: Vec::new(), // TODO: keep messages?
        }
    }

    fn map<U>(self, mapper: impl Fn(Actual<T>) -> Actual<U>) -> AssertThat<'t, U> {
        AssertThat {
            actual: mapper(self.actual),
            print_location: self.print_location,
            additional_messages: self.additional_messages,
        }
    }
}

impl<'t, T> AssertThat<'t, T> {
    pub fn actual(&'t self) -> &'t T {
        self.actual.borrowed()
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

    /// Specify an additional messages to be displayed on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_additional_message(mut self, message: impl std::fmt::Debug + 'static) -> Self {
        self.additional_messages.push(Box::new(message));
        self
    }

    #[track_caller]
    pub fn fail_with(&self, failure: impl Failure) -> ! {
        let caller_location = std::panic::Location::caller();

        let err = match (self.print_location, self.additional_messages.len()) {
            (false, 0) => formatdoc! {"
                    -------- assertr --------
                    {failure}
                    -------- assertr --------
                "
            },
            (false, _) => formatdoc! {"
                    -------- assertr --------
                    {failure}

                    Details: {additional_messages:#?}
                    -------- assertr --------
                ",
                additional_messages = self.additional_messages,
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

                    Details: {additional_messages:#?}
                    -------- assertr --------
                ",
                file = caller_location.file(),
                line = caller_location.line(),
                column = caller_location.column(),
                additional_messages = self.additional_messages,
            },
        };

        panic!("{err}")
    }
}
