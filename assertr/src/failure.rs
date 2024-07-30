use std::fmt::{Arguments, Debug};

use indoc::formatdoc;

use crate::{
    details::{DetailMessages, WithDetail},
    prelude::Mode,
    AssertThat,
};

pub(crate) trait Fallible {
    fn store_failure(&self, failure: String);
}

pub trait Failure {
    fn format(&self) -> Arguments<'_>;
}

pub struct GenericFailure<'a> {
    pub arguments: std::fmt::Arguments<'a>,
}

impl<'a> Failure for GenericFailure<'a> {
    fn format(&self) -> Arguments<'_> {
        self.arguments
    }
}

pub struct ExpectedActualFailure<'e, 'a, E: Debug, A: Debug> {
    pub expected: &'e E,
    pub actual: &'a A,
}

impl<'a> From<GenericFailure<'a>> for String {
    fn from(value: GenericFailure<'a>) -> Self {
        format!("{}", value.arguments)
    }
}

impl<'e, 'a, E: Debug, A: Debug> From<ExpectedActualFailure<'e, 'a, E, A>> for String {
    fn from(value: ExpectedActualFailure<'e, 'a, E, A>) -> Self {
        format!(
            "Expected: {:#?}\n\n  Actual: {:#?}",
            value.expected, value.actual
        )
    }
}

impl<'t, T, M: Mode> Fallible for AssertThat<'t, T, M> {
    fn store_failure(&self, failure: String) {
        match &self.parent {
            Some(parent) => parent.store_failure(failure),
            None => self.failures.borrow_mut().push(failure),
        };
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn fail(&self, failure: impl Into<String>) {
        self.fail_with_arguments(format_args!("{}", failure.into()));
    }

    #[track_caller]
    pub fn fail_using<'a>(&self, failure_provider: impl FnOnce(&Self) -> Arguments<'a>) {
        self.fail_with_arguments(failure_provider(self));
    }

    /// Final.
    #[track_caller]
    pub fn fail_with_arguments(&self, failure: Arguments<'_>) {
        let mut detail_messages = Vec::new();
        self.collect_messages(&mut detail_messages);

        // TODO: Compute should_print_location by reading root!
        let err = match (self.print_location, detail_messages.len()) {
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
                detail_messages = DetailMessages(detail_messages.as_ref()),
            },
            (true, 0) => {
                let caller_location = std::panic::Location::caller();
                formatdoc! {"
                    -------- assertr --------
                    Assertion failed at {file}:{line}:{column}

                    {failure}
                    -------- assertr --------
                ",
                    file = caller_location.file(),
                    line = caller_location.line(),
                    column = caller_location.column(),
                }
            }
            (true, _) => {
                let caller_location = std::panic::Location::caller();
                formatdoc! {"
                    -------- assertr --------
                    Assertion failed at {file}:{line}:{column}

                    {failure}

                    Details: {detail_messages:#?}
                    -------- assertr --------
                ",
                    file = caller_location.file(),
                    line = caller_location.line(),
                    column = caller_location.column(),
                    detail_messages = DetailMessages(detail_messages.as_ref()),
                }
            }
        };

        // TODO: Check is_capture in root! Do not allow with_capture() on derived asserts.
        match self.mode.borrow().is_capture() {
            true => self.store_failure(err),
            false => panic!("{err}"),
        };
    }
}
