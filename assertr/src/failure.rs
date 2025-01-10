use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Arguments, Write};

use crate::{
    details::{DetailMessages, WithDetail},
    prelude::Mode,
    AssertThat,
};

pub trait Failure {
    fn write_to(self, target: &mut String) -> std::fmt::Result;
}

impl Failure for &str {
    fn write_to(self, target: &mut String) -> core::fmt::Result {
        target.write_str(self)
    }
}

impl Failure for Arguments<'_> {
    fn write_to(self, target: &mut String) -> core::fmt::Result {
        target.write_fmt(self)
    }
}

impl<F> Failure for F
where
    F: FnOnce(&mut String) -> core::fmt::Result,
{
    fn write_to(self, target: &mut String) -> core::fmt::Result {
        self(target)
    }
}

pub(crate) trait Fallible {
    fn store_failure(&self, failure: String);
}

impl<T, M: Mode> Fallible for AssertThat<'_, T, M> {
    fn store_failure(&self, failure: String) {
        match &self.parent {
            Some(parent) => parent.store_failure(failure),
            None => self.failures.borrow_mut().push(failure),
        };
    }
}

impl<T, M: Mode> AssertThat<'_, T, M> {
    #[track_caller]
    pub fn fail(&self, failure: impl Failure) {
        let mut detail_messages = Vec::new();
        self.collect_messages(&mut detail_messages);

        let msg = build_failure_message(self.print_location, detail_messages, failure)
            .expect("no write error");

        // TODO: Check is_capture in root! Do not allow with_capture() on derived asserts.
        match self.mode.borrow().is_capture() {
            true => self.store_failure(msg),
            false => panic!("{msg}"),
        };
    }
}

#[track_caller]
fn build_failure_message(
    print_location: bool,
    detail_messages: Vec<String>,
    failure: impl Failure,
) -> Result<String, core::fmt::Error> {
    let mut err = String::new();

    err.write_str("-------- assertr --------\n")?;

    if print_location {
        let caller_location = core::panic::Location::caller();
        let _ = err.write_fmt(format_args!(
            "Assertion failed at {file}:{line}:{column}\n\n",
            file = caller_location.file(),
            line = caller_location.line(),
            column = caller_location.column(),
        ));
    }

    failure.write_to(&mut err)?;

    if !detail_messages.is_empty() {
        err.write_str("\n")?;
        err.write_fmt(format_args!(
            "Details: {detail_messages:#?}\n",
            detail_messages = DetailMessages(detail_messages.as_ref())
        ))?;
    }

    err.write_str("-------- assertr --------\n")?;

    Ok(err)
}
