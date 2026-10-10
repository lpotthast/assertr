//! Assertr's human-readable failure report, produced by the `Display` implementations of
//! [`AssertionFailure`] and [`AssertionFailures`](crate::failure::AssertionFailures).
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
//! `Nested failures:` (its children), each child indented one level and introduced by the
//! typed path it was raised for, when present. When any child report spans several lines, an
//! empty line separates the children.

use alloc::{format, string::String, vec::Vec};
use core::fmt::{self, Display, Write};

use crate::{
    failure::{AssertionFailure, BANNER, Fact, PathSegment},
    renderer::Rendered,
};

/// Writes the complete report of one failure, enclosed in banners.
pub(crate) fn write_failure(failure: &AssertionFailure, w: &mut dyn Write) -> fmt::Result {
    w.write_str(BANNER)?;
    write_report(failure, w)?;
    w.write_str(BANNER)
}

/// The report of a failure displayed inside a parent failure, including its path heading.
pub(crate) fn child_text(failure: &AssertionFailure) -> String {
    let mut report = String::new();
    write_children(&mut report, core::slice::from_ref(failure))
        .expect("writing a text report to a String cannot fail");
    report
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
    output.push_str(format!("{value:#}").trim_end_matches('\n'));
}

/// Writes everything between the banners.
fn write_report(failure: &AssertionFailure, w: &mut dyn Write) -> fmt::Result {
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

    let description = body(
        failure.actual.as_ref(),
        failure.relation.as_deref(),
        failure.expected.as_ref(),
        failure.unexpected.as_ref(),
    );
    let has_body = !description.is_empty() || failure.constraint.is_some();
    let omission_note = (failure.omitted_children > 0).then(|| {
        Fact::note(crate::renderer::omission(
            failure.omitted_children,
            "nested failure",
        ))
    });
    let facts = failure
        .facts
        .iter()
        .chain(omission_note.iter())
        .collect::<Vec<_>>();
    let has_blocks =
        !failure.messages.is_empty() || !facts.is_empty() || !failure.children.is_empty();

    // Separate the subject header only from content that follows it.
    let has_header = failure.subject_name.is_some() || failure.expression.is_some();
    if has_header && (has_body || has_blocks) {
        w.write_str("\n")?;
    }

    w.write_str(&description)?;
    if let Some(constraint) = &failure.constraint {
        if !description.is_empty() {
            w.write_str("\n")?;
        }
        w.write_str("Constraint:\n")?;
        write_report(constraint, &mut Indented::at_line_start(w))?;
    }

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
        f.write_str(format!("{:#}", self.0.value).trim_end_matches('\n'))
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

/// Writes the `Nested failures:` list. Like a loose Markdown list, an empty line separates the
/// items when any of them spans several lines, so each child report reads as its own block.
fn write_children(w: &mut dyn Write, children: &[AssertionFailure]) -> fmt::Result {
    if children.is_empty() {
        return Ok(());
    }

    let items = children
        .iter()
        .map(child_item)
        .collect::<Result<Vec<_>, _>>()?;
    let loose = items
        .iter()
        .any(|item| item.trim_end_matches('\n').contains('\n'));

    writeln!(w, "Nested failures:")?;
    for (index, item) in items.iter().enumerate() {
        if loose && index != 0 {
            w.write_str("\n")?;
        }
        w.write_str(item)?;
    }
    Ok(())
}

/// One `Nested failures:` item: the bullet, the child's path heading when present, and its
/// indented report.
fn child_item(child: &AssertionFailure) -> Result<String, fmt::Error> {
    let mut item = String::from("  - ");
    if child.path.is_empty() {
        write_report(child, &mut Indented::continuing(&mut item))?;
    } else {
        item.push_str("At ");
        for segment in &child.path {
            match segment {
                PathSegment::Field(name) => write!(item, ".{name}")?,
                PathSegment::TupleIndex(index) => write!(item, ".{index}")?,
                PathSegment::Variant(name) => write!(item, "::{name}")?,
                PathSegment::Index(index) => write!(item, "[{index}]")?,
                PathSegment::Key(key) => write!(item, "[{key}]")?,
            }
        }
        item.push_str(":\n");
        write_report(child, &mut Indented::at_line_start(&mut item))?;
    }
    Ok(item)
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
    use super::body;
    use crate::prelude::*;

    mod child_locations {
        use indoc::formatdoc;

        use super::*;
        use crate::{
            failure::{Fact, FailureBuilder, FailureKind, PathSegment},
            test_support::FailureReportAssertions,
        };

        #[test]
        fn index_and_key_facts_remain_evidence_with_and_without_paths() {
            for path in [alloc::vec![], alloc::vec![PathSegment::Index(1)]] {
                let heading = if path.is_empty() {
                    "  - does not match"
                } else {
                    "  - At [1]:\n    does not match"
                };
                let child = FailureBuilder::new::<()>(FailureKind::Matching)
                    .path(path)
                    .relation("does not match")
                    .facts([
                        Fact::labelled("index", "7"),
                        Fact::labelled("key", "evidence"),
                    ])
                    .build();
                let root = FailureBuilder::new::<()>(FailureKind::Matching)
                    .relation("does not hold")
                    .facts([
                        Fact::labelled("index", "8"),
                        Fact::labelled("key", "root evidence"),
                    ])
                    .child(child)
                    .build();

                assert_that!(root).has_text_report(formatdoc! {"
                    -------- assertr --------
                    does not hold

                    Details:
                      - index: 8
                      - key: root evidence
                    Nested failures:
                    {heading}

                        Details:
                          - index: 7
                          - key: evidence
                    -------- assertr --------
                "});
            }
        }

        #[test]
        fn every_path_segment_contributes_to_the_child_heading() {
            let child = FailureBuilder::new::<()>(FailureKind::Matching)
                .path([
                    PathSegment::Field("rows"),
                    PathSegment::Index(1),
                    PathSegment::Variant("Some"),
                    PathSegment::TupleIndex(0),
                    PathSegment::Key("\"id\"".into()),
                ])
                .relation("does not match")
                .build();
            let root = FailureBuilder::new::<()>(FailureKind::Matching)
                .child(child)
                .build();

            assert_that!(root).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Nested failures:
                  - At .rows[1]::Some.0["id"]:
                    does not match
                -------- assertr --------
            "#});
        }
    }

    mod nested_failure_items {
        use super::*;
        use crate::failure::{FailureBuilder, FailureKind, PathSegment};

        #[test]
        fn are_separated_by_empty_lines_when_any_item_spans_several_lines() {
            let root = FailureBuilder::new::<()>(FailureKind::Matching)
                .relation("does not match")
                .child(
                    FailureBuilder::new::<&str>(FailureKind::Equality)
                        .path([PathSegment::Field("name")])
                        .actual("\"Bob\"")
                        .expected("\"Ada\"")
                        .build(),
                )
                .child(
                    FailureBuilder::new::<u32>(FailureKind::Predicate)
                        .path([PathSegment::Field("active")])
                        .relation("is not active")
                        .build(),
                )
                .build();

            assert_that!(root.to_string()).is_equal_to(indoc::indoc! {r#"
                -------- assertr --------
                does not match

                Nested failures:
                  - At .name:
                    Expected: "Ada"

                      Actual: "Bob"

                  - At .active:
                    is not active
                -------- assertr --------
            "#});
        }

        #[test]
        fn stay_compact_when_every_item_fits_on_one_line() {
            let root = FailureBuilder::new::<()>(FailureKind::Matching)
                .relation("does not match")
                .child(
                    FailureBuilder::new::<u32>(FailureKind::Predicate)
                        .relation("is not even")
                        .build(),
                )
                .child(
                    FailureBuilder::new::<u32>(FailureKind::Predicate)
                        .relation("is not positive")
                        .build(),
                )
                .build();

            assert_that!(root.to_string()).is_equal_to(indoc::indoc! {"
                -------- assertr --------
                does not match

                Nested failures:
                  - is not even
                  - is not positive
                -------- assertr --------
            "});
        }
    }

    mod constraint_fields {
        use super::*;
        use crate::failure::{Fact, FailureBuilder, FailureKind};

        #[test]
        fn descriptions_render_facts_and_unexpected_values_through_the_common_grammar() {
            let description = FailureBuilder::new::<i32>(FailureKind::Equality)
                .relation("is not equal to")
                .unexpected("3")
                .fact(Fact::labelled("Reason", "reserved value"))
                .child(
                    FailureBuilder::new::<i32>(FailureKind::Other)
                        .relation("has a valid identifier")
                        .build(),
                )
                .omitted_children(1)
                .build();
            let failure = FailureBuilder::new::<[i32]>(FailureKind::Matching)
                .constraint(description)
                .build();
            assert_that!(failure.to_string()).is_equal_to(indoc::indoc! {r"
                -------- assertr --------
                Constraint:
                    is not equal to

                    Unexpected: 3

                    Details:
                      - Reason: reserved value
                      - ... 1 more nested failure ...
                    Nested failures:
                      - has a valid identifier
                -------- assertr --------
            "});
        }
    }

    mod expression_line {
        use super::*;

        #[test]
        fn is_capped_to_one_line_and_one_hundred_characters() {
            const LONG_EXPRESSION: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvw";

            let failures = assert_that!(42)
                .with_expression("first line\nsecond line")
                .with_location(false)
                .capture(|it| it.is_equal_to(43));
            assert_that!(failures[0].to_string()).contains("Expression: `first line...`\n\n");

            let failures = assert_that!(42)
                .with_expression(LONG_EXPRESSION)
                .with_location(false)
                .capture(|it| it.is_equal_to(43));
            let rendered = failures[0].to_string();
            let expression_line = rendered
                .lines()
                .find(|line| line.starts_with("Expression:"))
                .expect("expression line");
            assert_that!(expression_line.chars().count()).is_equal_to("Expression: ``".len() + 100);
            assert_that!(expression_line).ends_with("...`");
            assert_that!(failures[0].expression).is_equal_to(Some(LONG_EXPRESSION));
        }
    }

    mod header_separator {
        use super::*;
        use crate::failure::{FailureBuilder, FailureKind};

        #[test]
        fn is_omitted_when_nothing_follows_the_header() {
            let mut failure = FailureBuilder::new::<i32>(FailureKind::Other).build();
            failure.subject_name = Some("answer".into());
            failure.expression = Some("value");

            assert_that!(failure.to_string()).is_equal_to(indoc::indoc! {"
                -------- assertr --------
                Subject: answer
                Expression: `value`
                -------- assertr --------
            "});
        }

        #[test]
        fn separates_the_header_from_a_following_block() {
            let mut failure = FailureBuilder::new::<i32>(FailureKind::Other).build();
            failure.expression = Some("value");
            failure.messages.push("context".into());

            assert_that!(failure.to_string()).is_equal_to(indoc::indoc! {"
                -------- assertr --------
                Expression: `value`

                Messages:
                  - context
                -------- assertr --------
            "});
        }
    }

    mod body_grammar {
        use super::*;

        fn rendered(text: &str) -> crate::renderer::Rendered {
            text.into()
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
        use crate::failure::{Fact, FailureBuilder, FailureKind, PathSegment};

        #[test]
        fn nested_failures_are_indented_one_level_per_depth_with_empty_lines_left_empty() {
            let grandchild = FailureBuilder::new::<i32>(FailureKind::Ordering)
                .actual(1)
                .relation("is not greater than")
                .expected(5)
                .path([PathSegment::Index(0)])
                .build();
            let child = FailureBuilder::new::<[i32; 1]>(FailureKind::Predicate)
                .actual(format_args!("[1]"))
                .relation("does not exactly satisfy the assertions")
                .child(grandchild)
                .build();
            let failures = assert_that!(1).with_location(false).capture(|it| {
                it.track_assertion();
                it.raise(
                    it.failure(FailureKind::Predicate)
                        .relation("does not hold")
                        .fact(Fact::note("first note\nsecond line"))
                        .child(child),
                );
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
                      - At [0]:
                        Actual: 1

                        is not greater than

                        Expected: 5
                -------- assertr --------
            "});
        }
    }
}
