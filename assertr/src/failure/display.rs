//! The one place that turns the fields of an [`AssertionFailure`] into text.
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
use core::fmt::{self, Display, Write};

use super::{AssertionFailure, BANNER, Fact};

/// Renders the description of a failure from its fields.
pub(crate) fn body(
    actual: Option<&str>,
    relation: Option<&str>,
    expected: Option<&str>,
    unexpected: Option<&str>,
) -> String {
    let mut body = String::new();

    if let (Some(actual), None, Some(expected), None) = (actual, relation, expected, unexpected) {
        let _ = write!(body, "Expected: {expected}\n\n  Actual: {actual}\n");
        return body;
    }

    let paragraphs = [
        actual.map(|actual| ("Actual: ", actual)),
        relation.map(|relation| ("", relation)),
        expected.map(|expected| ("Expected: ", expected)),
        unexpected.map(|unexpected| ("Unexpected: ", unexpected)),
    ];
    for (label, text) in paragraphs.into_iter().flatten() {
        if !body.is_empty() {
            body.push('\n');
        }
        let _ = writeln!(body, "{label}{}", text.trim_end_matches('\n'));
    }
    body
}

impl Display for AssertionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(BANNER)?;
        write_report(self, f, false)?;
        f.write_str(BANNER)
    }
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

    let description = failure.description();
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
        f.write_str(self.0.value.trim_end_matches('\n'))
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
            writeln!(w, "At {} {}:", heading.label, heading.value)?;
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

    use super::body;

    mod body_grammar {
        use super::*;

        #[test]
        fn renders_a_direct_comparison_as_the_aligned_pair() {
            assert_that!(body(Some("42"), None, Some("43"), None))
                .is_equal_to("Expected: 43\n\n  Actual: 42\n");
        }

        #[test]
        fn renders_a_relation_between_actual_and_expected() {
            assert_that!(body(
                Some("42"),
                Some("is not greater than"),
                Some("43"),
                None
            ))
            .is_equal_to("Actual: 42\n\nis not greater than\n\nExpected: 43\n");
        }

        #[test]
        fn renders_an_unexpected_value_after_the_relation() {
            assert_that!(body(Some("[1, 2]"), Some("contains"), None, Some("2")))
                .is_equal_to("Actual: [1, 2]\n\ncontains\n\nUnexpected: 2\n");
        }

        #[test]
        fn leaves_absent_parts_out() {
            assert_that!(body(Some("[]"), Some("is unexpectedly empty"), None, None))
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

            assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {"
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
