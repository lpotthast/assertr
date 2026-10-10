//! Tests for the structured-failure capture API:
//!
//! - `AssertThat::capture` runs assertions in capture mode inside a closure and returns the
//!   collected failures, making a forgotten capture structurally impossible.
//! - Captured failures are structured `AssertionFailure` values whose fields (location, subject
//!   name, asserted expression, subject type, actual, relation, expected, unexpected, facts,
//!   chain-level messages, children, kind) can be inspected without parsing formatted text.
//! - Values become owned rendered trees when failures are built. Adapters decide how to use those
//!   trees at the panic boundary or after capture.

mod support;

use assertr::{
    failure::{Fact, FailureKind},
    prelude::*,
};

use self::support::{text, text_opt};

#[test]
fn failures_arrive_in_assertion_order_and_carry_the_messages_provided_up_to_them() {
    let failures = assert_that!(42).with_location(false).capture(|it| {
        it.with_detail_message("early")
            .is_greater_than(100)
            .with_detail_message("late")
            .is_equal_to(1)
    });

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(ToString::to_string)
                .contains("is not greater than");
            element
                .derive_owned(|value| value.messages.as_slice())
                .contains_exactly(["early"]);
        },
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(ToString::to_string)
                .contains("Expected: 1");
            element
                .derive_owned(|value| value.messages.as_slice())
                .contains_exactly(["early", "late"]);
        },
    ]);
}

#[test]
fn messages_and_details_render_as_separate_plain_bullet_blocks() {
    let failures = assert_that!(42)
        .with_location(false)
        .with_detail_message("first message\ncontinued message")
        .capture(|it| {
            it.track_assertion();
            it.raise(
                it.failure(FailureKind::Other)
                    .relation("The assertion failed.")
                    .fact(Fact::note("first detail\ncontinued detail")),
            );
            it
        });

    assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {"
        -------- assertr --------
        Expression: `42`

        The assertion failed.

        Messages:
          - first message
            continued message
        Details:
          - first detail
            continued detail
        -------- assertr --------
    "});
}

#[test]
fn mapping_inside_the_capture_closure_is_supported() {
    let failures = assert_that!("foo")
        .with_location(false)
        .capture(|it| it.map(|v| v.borrowed().len().into()).is_equal_to(4));

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive(|value| &value.subject_type_name)
                .is_equal_to(core::any::type_name::<usize>());
        },
    ]);
}

/// Failure-field routing stays here. Structural rendering behavior lives beside the renderer.
mod fields {
    use assertr::renderer::{Rendered, RenderedBody};

    use super::*;

    #[test]
    fn an_unexpected_map_entry_retains_a_structured_tuple() {
        use std::collections::BTreeMap;

        let failures = assert_that!(BTreeMap::from([("a", 1)]))
            .with_location(false)
            .capture(|it| it.does_not_contain_entry("a", 1));
        let unexpected = failures[0].unexpected.as_ref().unwrap();
        let RenderedBody::Tuple { items, .. } = &unexpected.body else {
            panic!("expected a tuple node, got {:?}", unexpected.body);
        };

        assert_that!(items.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("\"a\"");
            },
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("1");
            },
        ]);
    }

    #[test]
    fn a_structural_key_path_retains_its_tree_and_renders_inline() {
        use assertr::{
            failure::{FailureBuilder, PathSegment},
            renderer::RenderingOrder,
        };

        let failures = assert_that!([4, 6]).with_location(false).capture(|it| {
            it.track_assertion();
            it.raise(it.failure(FailureKind::Other).path(
                [PathSegment::Key(it.render().borrowed_values::<i32, _>(
                    it.actual(),
                    RenderingOrder::PreserveIteration,
                ))],
            ));
            it
        });
        let [PathSegment::Key(values)] = failures[0].path.as_slice() else {
            panic!("expected a key path");
        };
        assert_that!(failures[0].facts).is_empty();
        let RenderedBody::Group { items, .. } = &values.body else {
            panic!("expected a group node, got {:?}", values.body);
        };

        assert_that!(items.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("4");
            },
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("6");
            },
        ]);
        let parent = FailureBuilder::new::<()>(FailureKind::Other)
            .children(failures.into_vec())
            .build();
        assert_that!(parent.to_string()).contains("At [[4, 6]]:");
    }

    #[test]
    #[cfg(feature = "std")]
    fn children_of_an_order_free_subject_carry_no_index_and_are_sorted_by_rendered_text() {
        use std::collections::HashSet;

        let failures = assert_that!(HashSet::from([3, 1, 2]))
            .with_location(false)
            .capture(|it| {
                it.contains_satisfying(|element| {
                    element.is_equal_to(9);
                })
            });
        let children = &failures[0].children;

        assert_that!(
            children
                .iter()
                .map(|child| text_opt(child.actual.as_ref()))
                .collect::<Vec<_>>()
        )
        .contains_exactly([Some("1"), Some("2"), Some("3")]);
        assert_that!(children.iter().all(|child| child.facts.is_empty())).is_true();
    }

    #[test]
    fn a_downstream_failure_is_built_like_a_built_in_one() {
        let failures = assert_that!(1).with_location(false).capture(|it| {
            it.track_assertion();
            it.raise(
                it.failure(FailureKind::Other)
                    .relation("does not hold")
                    .fact(Fact::note("some evidence")),
            );
            it
        });
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Other);
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not hold"));
        assert_that!(text_opt(failure.actual.as_ref())).is_none();
        assert_that!(text_opt(failure.expected.as_ref())).is_none();
        assert_that!(failure.facts).contains_exactly([Fact::note("some evidence")]);
        let fact = assert_that!(failure.facts[0]);
        fact.derive(|fact| &fact.label).is_none();
        fact.derive(|fact| &fact.value)
            .derive(|value| &value.type_name)
            .is_none();
    }
}

mod matcher_metadata {
    use super::*;

    #[test]
    fn matcher_paths_and_constraints_preserve_metadata() {
        use assertr::{
            failure::PathSegment,
            matchers::{all_of, eq, predicate},
        };
        let failures = assert_that!([1])
            .with_subject_name("rows")
            .with_detail_message("request context")
            .capture(|it| {
                it.matches(elements_are![all_of(matchers![
                    eq(2),
                    predicate(|_: &i32| false)
                ])])
            });
        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.subject_name.as_deref())
                    .is_equal_to(Some("rows"));
                element
                    .derive(|value| &value.expression)
                    .is_equal_to(Some("[1]"));
                element
                    .derive(|value| &value.messages)
                    .contains_exactly(["request context"]);
                element
                    .derive_owned(|value| value.location.unwrap().file())
                    .ends_with("structured_failures.rs");
                element
                    .derive(|value| &value.children)
                    .contains_exactly_satisfying([false, true].map(|has_constraint| {
                        move |child: AssertThat<AssertionFailure, Capture>| {
                            child
                                .derive(|child| &child.path)
                                .contains_exactly([PathSegment::Index(0)]);
                            child.derive(|child| &child.location).is_none();
                            child.derive(|child| &child.expression).is_none();
                            child.derive(|child| &child.messages).is_empty();
                            if has_constraint {
                                child.derive(|child| &child.constraint).is_some_satisfying(
                                    |constraint| {
                                        constraint
                                            .derive_owned(|constraint| {
                                                constraint.relation.as_deref()
                                            })
                                            .is_equal_to(Some("satisfies the predicate"));
                                    },
                                );
                            }
                        }
                    }));
            },
        ]);
    }
}
