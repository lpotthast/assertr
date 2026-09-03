//! Structured assertion failures and their descriptions.
//!
//! [`AssertionFailure`] is what capture mode hands back. Implement [`Failure`] to describe a
//! failure without building a `String` first, then pass it to [`AssertThat::fail`].

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Arguments, Display, Write};

use crate::{AssertThat, details::WithDetail, prelude::Mode};

/// A description accepted by [`AssertThat::fail`] without first allocating a `String`.
pub trait Failure {
    /// Writes the failure message to the target string.
    ///
    /// # Errors
    ///
    /// Returns a `core::fmt::Error` if writing to the target string fails.
    fn write_to(self, target: &mut String) -> core::fmt::Result;
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

/// Delimiter opening and closing every rendered failure message.
pub(crate) const BANNER: &str = "-------- assertr --------\n";

/// A single structured assertion failure.
///
/// Capture-mode assertions (see [`AssertThat::capture`]) collect these instead of panicking.
/// Every part of a failure is exposed as its own field, so consumers can inspect failures
/// programmatically or compose their own rendering without parsing formatted text.
///
/// The complete human-readable form is produced by the [`Display`] implementation. In panic mode,
/// it becomes the panic message. Capture mode retains the fields without formatting that complete
/// message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssertionFailure {
    /// Where the failing assertion was invoked. `None` when location printing was disabled via
    /// `with_location(false)`.
    pub location: Option<&'static core::panic::Location<'static>>,

    /// The name given to the assertion's subject via `with_subject_name`, if any.
    pub subject_name: Option<String>,

    /// The source expression that produced the assertion subject, if the entry point captured it.
    /// Derived child chains start a new diagnostic subject and do not inherit their parent's
    /// expression.
    pub expression: Option<&'static str>,

    /// The Rust type name of the assertion subject that raised this failure.
    ///
    /// This is produced by [`core::any::type_name`] and is intended for diagnostics. A failure
    /// raised by a derived child records the child's subject type rather than the root's.
    pub subject_type_name: &'static str,

    /// The assertion-specific description, such as the rendered subject and expected value.
    /// Location, subject name, expression, and detail messages live in their own fields.
    pub description: String,

    /// Diagnostics attached by the failing assertion itself and scoped to exactly this failure,
    /// e.g. the `Differences` of an equality assertion or the per-element diagnostics of a
    /// collection assertion.
    pub details: Vec<String>,

    /// User-provided detail messages (`with_detail_message` / `add_detail_message`) collected
    /// from the assertion chain. Contains only the messages provided up to the point this
    /// failure was raised. A message added later appears only in the failures raised after it.
    pub messages: Vec<String>,
}

impl Display for AssertionFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(BANNER)?;

        if let Some(location) = self.location {
            f.write_fmt(format_args!(
                "Assertion failed at {file}:{line}:{column}\n\n",
                file = location.file(),
                line = location.line(),
                column = location.column(),
            ))?;
        }

        if let Some(subject_name) = &self.subject_name {
            f.write_fmt(format_args!("Subject: {subject_name}\n"))?;
        }
        if let Some(expression) = self.expression {
            f.write_str("Expression: `")?;
            write_expression(f, expression)?;
            f.write_str("`\n")?;
        }
        if self.subject_name.is_some() || self.expression.is_some() {
            f.write_str("\n")?;
        }

        f.write_str(&self.description)?;
        if !self.description.ends_with('\n') {
            f.write_str("\n")?;
        }

        if !self.messages.is_empty() || !self.details.is_empty() {
            f.write_str("\n")?;
            write_labeled_entries(f, "Messages", &self.messages)?;
            write_labeled_entries(f, "Details", &self.details)?;
        }

        f.write_str(BANNER)
    }
}

fn write_labeled_entries(
    f: &mut core::fmt::Formatter<'_>,
    label: &str,
    entries: &[String],
) -> core::fmt::Result {
    if entries.is_empty() {
        return Ok(());
    }

    f.write_fmt(format_args!("{label}:\n"))?;
    for entry in entries {
        let mut lines = entry.split('\n');
        f.write_str("  - ")?;
        f.write_str(lines.next().unwrap_or_default())?;
        for line in lines {
            f.write_str("\n    ")?;
            f.write_str(line)?;
        }
        f.write_str("\n")?;
    }
    Ok(())
}

const MAX_EXPRESSION_CHARS: usize = 100;
const ELLIPSIS: &str = "...";

fn write_expression(f: &mut core::fmt::Formatter<'_>, expression: &str) -> core::fmt::Result {
    let line_end = expression.find(['\r', '\n']).unwrap_or(expression.len());
    let first_line = &expression[..line_end];
    let truncated =
        line_end != expression.len() || first_line.chars().count() > MAX_EXPRESSION_CHARS;

    if truncated {
        for character in first_line
            .chars()
            .take(MAX_EXPRESSION_CHARS - ELLIPSIS.len())
        {
            f.write_char(character)?;
        }
        f.write_str(ELLIPSIS)
    } else {
        f.write_str(first_line)
    }
}

pub(crate) trait Fallible {
    fn store_failure(&self, failure: AssertionFailure);
}

impl<T, M: Mode, R> Fallible for AssertThat<'_, T, M, R> {
    fn store_failure(&self, failure: AssertionFailure) {
        match &self.state.parent {
            Some(parent) => parent.store_failure(failure),
            None => self.state.failures.borrow_mut().push(failure),
        }
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Records or raises a failure message.
    ///
    /// This is the failure path of a hand-written leaf assertion, called after
    /// [`AssertThat::track_assertion`]. See [custom assertions](crate#custom-assertions) for the
    /// shape such an assertion takes.
    ///
    /// # Panics
    ///
    /// Panics with the formatted failure message when not in capture mode.
    #[track_caller]
    pub fn fail(&self, failure: impl Failure) {
        self.fail_with_details(core::iter::empty(), failure);
    }

    #[track_caller]
    #[cfg(feature = "std")]
    pub(crate) fn fail_at(
        &self,
        location: &'static core::panic::Location<'static>,
        failure: impl Failure,
    ) {
        self.fail_with_details_at(location, core::iter::empty(), failure);
    }

    /// Records or raises a failure with diagnostics scoped to that failure.
    ///
    /// Assertion implementations use this for evidence specific to one failure. The details are
    /// stored in [`AssertionFailure::details`] and cannot reappear in later failures. Use
    /// [`AssertThat::add_detail_message`] or a `with_detail_message` method for user context that
    /// should apply to subsequent failures. See [custom assertions](crate#custom-assertions) for
    /// the shape of a hand-written assertion.
    ///
    /// # Panics
    ///
    /// Panics with the formatted failure message when not in capture mode.
    #[track_caller]
    pub fn fail_with_details(
        &self,
        details: impl IntoIterator<Item = String>,
        failure: impl Failure,
    ) {
        self.fail_with_details_at(core::panic::Location::caller(), details, failure);
    }

    #[track_caller]
    pub(crate) fn fail_with_details_at(
        &self,
        location: &'static core::panic::Location<'static>,
        details: impl IntoIterator<Item = String>,
        failure: impl Failure,
    ) {
        let location = if self.state.print_location {
            Some(location)
        } else {
            None
        };

        let mut messages = Vec::new();
        self.collect_messages(&mut messages);

        let mut description = String::new();
        failure.write_to(&mut description).expect("no write error");

        let failure = AssertionFailure {
            location,
            subject_name: self.state.subject_name.clone(),
            expression: self.state.expression,
            subject_type_name: core::any::type_name::<T>(),
            description,
            details: details.into_iter().collect(),
            messages,
        };

        if M::CAPTURES {
            self.store_failure(failure);
        } else {
            panic!("{failure}");
        }
    }
}
