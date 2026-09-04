//! Assertr's stable human-readable failure adapter.
//!
//! The body grammar is:
//!
//! ```text
//! Actual: <actual>
//!
//! <relation>
//!
//! Expected: <expected>
//!
//! Unexpected: <unexpected>
//! ```
//!
//! with every absent part left out. A failure that has an actual and an expected value but no
//! relation is a direct comparison and renders as the aligned pair
//!
//! ```text
//! Expected: <expected>
//!
//!   Actual: <actual>
//! ```
//!
//! The body is followed by the chain's `Messages:`, the failure's `Details:` (its facts), and its
//! `Nested failures:` (its children), each child indented one level and introduced by the element
//! index or map key it was raised for.

use alloc::string::String;
use core::{
    convert::Infallible,
    fmt::{self, Display, Write},
    ops::Deref,
};

use super::Adapter;
use crate::{AssertionFailure, Fact, failure::BANNER, renderer::Rendered};

/// The typed human-readable representation of an assertion failure.
///
/// Keeping this distinct from an arbitrary [`String`] prevents a machine-oriented stage from
/// accidentally accepting human-readable text. It can be borrowed through [`AsRef<str>`] or
/// consumed with [`into_string`](Self::into_string).
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HumanReadableText(String);

impl HumanReadableText {
    /// Borrows the generated text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes this value and returns its string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for HumanReadableText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for HumanReadableText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for HumanReadableText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<HumanReadableText> for String {
    fn from(text: HumanReadableText) -> Self {
        text.into_string()
    }
}

impl PartialEq<str> for HumanReadableText {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for HumanReadableText {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for HumanReadableText {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

/// Converts an [`AssertionFailure`] to assertr's stable human-readable text.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToHumanReadableText;

impl ToHumanReadableText {
    /// Converts one failure without the `Result` wrapper needed by generic adapter composition.
    ///
    /// # Panics
    ///
    /// Panics if Rust's infallible [`String`] formatter unexpectedly reports an error.
    #[must_use]
    pub fn render(self, failure: &AssertionFailure) -> HumanReadableText {
        let mut report = String::new();
        report.push_str(BANNER);
        write_report(failure, &mut report, false)
            .expect("writing a text report to a String cannot fail");
        report.push_str(BANNER);
        HumanReadableText(report)
    }
}

impl Adapter<AssertionFailure> for ToHumanReadableText {
    type Output = HumanReadableText;
    type Error = Infallible;

    fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
        Ok(self.render(failure))
    }
}

/// Renders the description of a failure from its fields.
fn body(
    actual: Option<&Rendered>,
    relation: Option<&str>,
    expected: Option<&Rendered>,
    unexpected: Option<&Rendered>,
) -> String {
    let mut body = String::new();

    if let (Some(actual), None, Some(expected), None) = (actual, relation, expected, unexpected) {
        body.push_str("Expected: ");
        write_value(&mut body, expected);
        body.push_str("\n\n  Actual: ");
        write_value(&mut body, actual);
        body.push('\n');
        return body;
    }

    if let Some(actual) = actual {
        write_body_value(&mut body, "Actual: ", actual);
    }
    if let Some(relation) = relation {
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(relation.trim_end_matches('\n'));
        body.push('\n');
    }
    if let Some(expected) = expected {
        write_body_value(&mut body, "Expected: ", expected);
    }
    if let Some(unexpected) = unexpected {
        write_body_value(&mut body, "Unexpected: ", unexpected);
    }
    body
}

fn write_body_value(body: &mut String, label: &str, value: &Rendered) {
    if !body.is_empty() {
        body.push('\n');
    }
    body.push_str(label);
    write_value(body, value);
    body.push('\n');
}

fn write_value(output: &mut String, value: &Rendered) {
    let mut rendered = String::new();
    value
        .write(&mut rendered, true)
        .expect("writing a rendered value to a String cannot fail");
    output.push_str(rendered.trim_end_matches('\n'));
}

/// Writes everything between the banners.
///
/// `located` is set for a child whose position was already written as its heading, so the fact
/// that carries the position is not repeated among its details.
fn write_report(failure: &AssertionFailure, w: &mut dyn Write, located: bool) -> fmt::Result {
    if let Some(location) = failure.location {
        write!(
            w,
            "Assertion failed at {file}:{line}:{column}\n\n",
            file = location.file(),
            line = location.line(),
            column = location.column(),
        )?;
    }

    if let Some(subject_name) = &failure.subject_name {
        writeln!(w, "Subject: {subject_name}")?;
    }
    if let Some(expression) = failure.expression {
        w.write_str("Expression: `")?;
        write_expression(w, expression)?;
        w.write_str("`\n")?;
    }
    if failure.subject_name.is_some() || failure.expression.is_some() {
        w.write_str("\n")?;
    }

    let description = body(
        failure.actual.as_ref(),
        failure.relation.as_deref(),
        failure.expected.as_ref(),
        failure.unexpected.as_ref(),
    );
    let has_body = !description.is_empty();
    w.write_str(&description)?;

    let facts = failure
        .facts
        .iter()
        .filter(|fact| !(located && fact.is_location()))
        .collect::<alloc::vec::Vec<_>>();
    let has_blocks =
        !failure.messages.is_empty() || !facts.is_empty() || !failure.children.is_empty();
    if has_body && has_blocks {
        w.write_str("\n")?;
    }

    write_entries(w, "Messages", failure.messages.iter().map(String::as_str))?;
    write_entries(w, "Details", facts.iter().map(|fact| FactText(fact)))?;
    write_children(w, &failure.children)
}

/// A fact as `Display` text without a trailing newline.
struct FactText<'a>(&'a Fact);

impl Display for FactText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.label.is_empty() {
            f.write_str(&self.0.label)?;
            f.write_str(": ")?;
        }
        let mut value = String::new();
        self.0.value.write(&mut value, true)?;
        f.write_str(value.trim_end_matches('\n'))
    }
}

fn write_entries<E: Display>(
    w: &mut dyn Write,
    label: &str,
    entries: impl Iterator<Item = E>,
) -> fmt::Result {
    let mut entries = entries.peekable();
    if entries.peek().is_none() {
        return Ok(());
    }

    writeln!(w, "{label}:")?;
    for entry in entries {
        w.write_str("  - ")?;
        write!(Indented::continuing(w), "{entry}")?;
        w.write_str("\n")?;
    }
    Ok(())
}

fn write_children(w: &mut dyn Write, children: &[AssertionFailure]) -> fmt::Result {
    if children.is_empty() {
        return Ok(());
    }

    writeln!(w, "Nested failures:")?;
    for child in children {
        w.write_str("  - ")?;
        let heading = child.facts.iter().find(|fact| fact.is_location());
        if let Some(heading) = heading {
            write!(w, "At {} ", heading.label)?;
            heading.value.write(w, false)?;
            writeln!(w, ":")?;
            write_report(child, &mut Indented::at_line_start(w), true)?;
        } else {
            write_report(child, &mut Indented::continuing(w), false)?;
        }
    }
    Ok(())
}

const MAX_EXPRESSION_CHARS: usize = 100;
const ELLIPSIS: &str = "...";

fn write_expression(w: &mut dyn Write, expression: &str) -> fmt::Result {
    let line_end = expression.find(['\r', '\n']).unwrap_or(expression.len());
    let first_line = &expression[..line_end];
    let truncated =
        line_end != expression.len() || first_line.chars().count() > MAX_EXPRESSION_CHARS;

    if truncated {
        for character in first_line
            .chars()
            .take(MAX_EXPRESSION_CHARS - ELLIPSIS.len())
        {
            w.write_char(character)?;
        }
        w.write_str(ELLIPSIS)
    } else {
        w.write_str(first_line)
    }
}

/// Indents every non-empty line written through it by one level. Empty lines stay empty.
struct Indented<'w> {
    inner: &'w mut dyn Write,
    at_line_start: bool,
}

impl<'w> Indented<'w> {
    const INDENT: &'static str = "    ";

    /// The next character starts a new line and receives the indentation.
    fn at_line_start(inner: &'w mut dyn Write) -> Self {
        Self {
            inner,
            at_line_start: true,
        }
    }

    /// The next character continues the current line, such as the line holding a bullet.
    fn continuing(inner: &'w mut dyn Write) -> Self {
        Self {
            inner,
            at_line_start: false,
        }
    }
}

impl Write for Indented<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for (index, line) in text.split('\n').enumerate() {
            if index != 0 {
                self.inner.write_str("\n")?;
                self.at_line_start = true;
            }
            if line.is_empty() {
                continue;
            }
            if self.at_line_start {
                self.inner.write_str(Self::INDENT)?;
                self.at_line_start = false;
            }
            self.inner.write_str(line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::renderer::IntoRendered;

    use super::{ToHumanReadableText, body};

    mod body_grammar {
        use super::*;

        fn rendered(text: &str) -> crate::renderer::Rendered {
            text.into_rendered()
        }

        #[test]
        fn renders_a_direct_comparison_as_the_aligned_pair() {
            assert_that!(body(
                Some(&rendered("42")),
                None,
                Some(&rendered("43")),
                None
            ))
            .is_equal_to("Expected: 43\n\n  Actual: 42\n");
        }

        #[test]
        fn renders_a_relation_between_actual_and_expected() {
            assert_that!(body(
                Some(&rendered("42")),
                Some("is not greater than"),
                Some(&rendered("43")),
                None
            ))
            .is_equal_to("Actual: 42\n\nis not greater than\n\nExpected: 43\n");
        }

        #[test]
        fn renders_an_unexpected_value_after_the_relation() {
            assert_that!(body(
                Some(&rendered("[1, 2]")),
                Some("contains"),
                None,
                Some(&rendered("2"))
            ))
            .is_equal_to("Actual: [1, 2]\n\ncontains\n\nUnexpected: 2\n");
        }

        #[test]
        fn leaves_absent_parts_out() {
            assert_that!(body(
                Some(&rendered("[]")),
                Some("is unexpectedly empty"),
                None,
                None
            ))
            .is_equal_to("Actual: []\n\nis unexpectedly empty\n");
            assert_that!(body(None, Some("did not panic"), None, None))
                .is_equal_to("did not panic\n");
            assert_that!(body(None, None, None, None)).is_equal_to("");
        }
    }

    mod indentation {
        use super::*;
        use crate::failure::{Fact, FailureBuilder, FailureKind};

        #[test]
        fn nested_failures_are_indented_one_level_per_depth_with_empty_lines_left_empty() {
            let grandchild = FailureBuilder::detached::<i32>(FailureKind::Ordering)
                .actual(1)
                .relation("is not greater than")
                .expected(5)
                .build()
                .located_at(Fact::index(0));
            let child = FailureBuilder::detached::<[i32; 1]>(FailureKind::Predicate)
                .actual(format_args!("[1]"))
                .relation("does not exactly satisfy the assertions")
                .child(grandchild)
                .build();
            let failures = assert_that!(1).with_location(false).capture(|it| {
                it.track_assertion();
                it.failure(FailureKind::Predicate)
                    .relation("does not hold")
                    .note("first note\nsecond line")
                    .child(child)
                    .raise();
                it
            });

            assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(indoc::indoc! {"
                -------- assertr --------
                Expression: `1`

                does not hold

                Details:
                  - first note
                    second line
                Nested failures:
                  - Actual: [1]

                    does not exactly satisfy the assertions

                    Nested failures:
                      - At index 0:
                        Actual: 1

                        is not greater than

                        Expected: 5
                -------- assertr --------
            "});
        }
    }
}
