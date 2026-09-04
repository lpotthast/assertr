//! Tests for the structured-failure capture API:
//!
//! - `AssertThat::capture` runs assertions in capture mode inside a closure and returns the
//!   collected failures, making a forgotten capture structurally impossible.
//! - Captured failures are structured `AssertionFailure` values whose fields (location, subject
//!   name, asserted expression, subject type, description, actual, relation, expected, unexpected,
//!   facts, chain-level messages, children, kind) can be inspected without parsing formatted text.
//! - Rendering happens only at the panic or display boundary, via `Display`.

use assertr::FailureKind;
use assertr::prelude::*;
use indoc::formatdoc;

#[test]
fn capture_returns_structured_failures_with_separated_fields() {
    let failures = assert_that!(42)
        .with_subject_name("answer")
        .capture(|it| it.with_detail_message("user context").is_equal_to(43));

    assert_that!(&failures).has_length(1);
    let failure = &failures[0];

    assert_that!(failure.subject_name.as_deref()).is_equal_to(Some("answer"));
    assert_that!(failure.expression).is_equal_to(Some("42"));
    assert_that!(failure.subject_type_name).is_equal_to(core::any::type_name::<i32>());
    assert_that!(failure.facts.as_slice()).is_empty();
    assert_that!(failure.messages.as_slice()).contains_exactly(["user context"]);
    // The description holds only the assertion-specific body; everything else lives in its own
    // field, allowing custom rendering without parsing.
    assert_that!(failure.description())
        .contains("Expected: 43")
        .contains("Actual: 42")
        .does_not_contain("-------- assertr --------")
        .does_not_contain("Assertion failed at")
        .does_not_contain("Subject:")
        .does_not_contain("Expression:")
        .does_not_contain("Details:");
    // Failures are plain values: cloneable and comparable.
    assert_that!(failures.clone() == failures).is_true();
}

#[test]
fn location_is_captured_by_default_and_absent_when_disabled() {
    let failures = assert_that!(1).capture(|it| it.is_equal_to(2));
    let location = failures[0].location.expect("location captured by default");
    assert_that!(location.file()).ends_with("structured_failures.rs");
    assert_that!(location.line() > 0).is_true();

    let failures = assert_that!(1)
        .with_location(false)
        .capture(|it| it.is_equal_to(2));
    assert_that!(failures[0].location.is_none()).is_true();
}

#[test]
fn failures_arrive_in_assertion_order_and_carry_the_messages_provided_up_to_them() {
    let failures = assert_that!(42).with_location(false).capture(|it| {
        it.with_detail_message("early")
            .is_greater_than(100)
            .with_detail_message("late")
            .is_equal_to(1)
    });

    assert_that!(&failures).has_length(2);
    assert_that!(failures[0].description()).contains("is not greater than");
    assert_that!(failures[1].description()).contains("Expected: 1");
    // A message only reaches the failures raised after it was provided.
    assert_that!(failures[0].messages.as_slice()).contains_exactly(["early"]);
    assert_that!(failures[1].messages.as_slice()).contains_exactly(["early", "late"]);
}

#[test]
fn display_renders_the_stable_human_readable_format() {
    let failures = assert_that!(42)
        .with_location(false)
        .capture(|it| it.is_equal_to(43));

    assert_that!(failures[0].to_string()).is_equal_to(formatdoc! {"
        -------- assertr --------
        Expression: `42`

        Expected: 43

          Actual: 42
        -------- assertr --------
    "});
}

#[test]
fn per_failure_diagnostics_are_exposed_as_details() {
    let failures = assert_that!([1, 2, 3])
        .with_location(false)
        .capture(|it| it.contains_exactly([1, 9]));

    let failure = &failures[0];
    assert_that!(failure.facts.is_empty()).is_false();
    assert_that!(failure.messages.as_slice()).is_empty();
    // The rendered form still shows them under `Details:`.
    assert_that!(failure.to_string()).contains("Details:\n  - ");
}

#[test]
fn messages_and_details_render_as_separate_plain_bullet_blocks() {
    let failures = assert_that!(42)
        .with_location(false)
        .with_detail_message("first message\ncontinued message")
        .capture(|it| {
            it.track_assertion();
            it.failure(FailureKind::Other)
                .relation("The assertion failed.")
                .note("first detail\ncontinued detail")
                .raise();
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
fn failures_from_derived_and_satisfies_assertions_reach_the_root() {
    let failures = assert_that!(("foo".to_owned(), 42))
        .with_location(false)
        .capture(|it| {
            let it = it.satisfies(
                |v| &v.0,
                |name| {
                    name.contains("xyz");
                },
            );
            {
                let len = it.derive_owned(|v| v.0.len());
                len.is_equal_to(9);
            }
            it
        });

    assert_that!(&failures).has_length(2);
    assert_that!(failures[0].description()).contains("xyz");
    assert_that!(failures[1].description()).contains("Expected: 9");
    assert_that!(failures[0].subject_type_name).is_equal_to(core::any::type_name::<String>());
    assert_that!(failures[1].subject_type_name).is_equal_to(core::any::type_name::<usize>());
}

#[test]
fn capture_on_a_derived_assertion_is_scoped_to_that_chain() {
    let value = ("foo".to_owned(), 42);
    let root = assert_that!(value)
        .with_location(false)
        .with_detail_message("root context");

    let failures = root.derive_owned(|v| v.1).capture(|it| it.is_equal_to(43));

    // The derived chain's failures are returned locally instead of propagating to the
    // panic-mode root, while ancestor detail messages are preserved.
    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].messages.as_slice()).contains_exactly(["root context"]);

    // The root stays in panic mode and remains usable.
    root.is_equal_to(("foo".to_owned(), 42));
}

#[test]
fn mapping_inside_the_capture_closure_is_supported() {
    let failures = assert_that!("foo")
        .with_location(false)
        .capture(|it| it.map(|v| v.borrowed().len().into()).is_equal_to(4));

    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].subject_type_name).is_equal_to(core::any::type_name::<usize>());
}

#[test]
fn a_capture_closure_performing_no_assertions_panics() {
    let result = std::panic::catch_unwind(|| {
        let _ = assert_that!(42).capture(|it| it);
    });

    let panic = result.expect_err("expected a panic");
    assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(
        &"The closure passed to `capture` / `verify` performed no assertions!",
    ));
}

#[test]
fn assertions_before_capture_do_not_satisfy_the_capture_closure_check() {
    let result = std::panic::catch_unwind(|| {
        let _ = assert_that!(42).is_equal_to(42).capture(|it| it);
    });

    let panic = result.expect_err("expected a panic");
    assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(
        &"The closure passed to `capture` / `verify` performed no assertions!",
    ));
}

#[test]
fn dropping_an_unused_panic_mode_assertion_no_longer_panics() {
    let result = std::panic::catch_unwind(|| {
        let _unused = assert_that!(42);
    });

    assert_that!(result.is_ok()).is_true();
}

#[test]
// The `if` around the panic keeps the closure's return type inferable; an `assert!` would
// change the panic payload.
#[allow(clippy::manual_assert)]
fn a_panic_inside_the_capture_closure_propagates_without_a_double_panic() {
    let result = std::panic::catch_unwind(|| {
        let _ = assert_that!(42).capture(|it| {
            // Record a failure first, so unwinding happens while failures are held.
            let it = it.is_equal_to(43);
            if it.actual() == &42 {
                panic!("original panic");
            }
            it
        });
    });

    let panic = result.expect_err("expected a panic");
    assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_verify_and_verify_owned_return_structured_failures() {
    let failures = 42.verify(|it| it.with_location(false).be_equal_to(43));
    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].description()).contains("Expected: 43");

    assert_that!(42.verify(|it| it.be_equal_to(42))).is_empty();

    let failures = String::from("foo").verify_owned(|it| it.have_length(9));
    assert_that!(&failures).has_length(1);
}

/// One field-level test per assertion family: the fields carry everything the text carries.
mod fields {
    use assertr::prelude::*;
    use assertr::{Fact, FailureKind};
    use core::fmt;

    #[test]
    fn an_equality_failure_carries_expected_and_actual_without_a_relation() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_equal_to(43));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
        assert_that!(failure.actual.as_deref()).is_equal_to(Some("42"));
        assert_that!(failure.relation.as_deref()).is_none();
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("43"));
        assert_that!(failure.unexpected.as_deref()).is_none();
        assert_that!(failure.facts.as_slice()).is_empty();
        assert_that!(failure.children.as_slice()).is_empty();
        assert_that!(failure.description()).is_equal_to("Expected: 43\n\n  Actual: 42\n");
    }

    #[test]
    fn a_negated_assertion_carries_the_unexpected_value_instead_of_an_expected_one() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_not_equal_to(42));
        let failure = &failures[0];

        assert_that!(failure.relation.as_deref()).is_equal_to(Some("is equal to"));
        assert_that!(failure.expected.as_deref()).is_none();
        assert_that!(failure.unexpected.as_deref()).is_equal_to(Some("42"));
        assert_that!(failure.description())
            .is_equal_to("Actual: 42\n\nis equal to\n\nUnexpected: 42\n");
    }

    #[test]
    fn an_ordering_failure_carries_the_relation_between_actual_and_expected() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_greater_than(100));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Ordering);
        assert_that!(failure.actual.as_deref()).is_equal_to(Some("42"));
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("is not greater than"));
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("100"));
        assert_that!(failure.description())
            .is_equal_to("Actual: 42\n\nis not greater than\n\nExpected: 100\n");
    }

    #[test]
    fn a_length_failure_carries_the_actual_length_as_a_labeled_fact() {
        let failures = assert_that!([1])
            .with_location(false)
            .capture(|it| it.has_length(2));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Length);
        assert_that!(failure.relation.as_deref())
            .is_equal_to(Some("does not have the expected length"));
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("2"));
        assert_that!(failure.facts.as_slice()).contains_exactly([Fact::new("Actual length", "1")]);
        assert_that!(failure.facts[0].to_string()).is_equal_to("Actual length: 1");
    }

    #[test]
    fn a_membership_failure_carries_the_missing_element_as_expected() {
        let failures = assert_that!([1, 2])
            .with_location(false)
            .capture(|it| it.contains(3));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Membership);
        assert_that!(failure.actual.as_deref()).is_equal_to(Some("[\n    1,\n    2,\n]"));
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not contain"));
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("3"));
    }

    #[test]
    fn a_variant_failure_names_the_expected_variant() {
        let failures = assert_that!(Option::<i32>::None)
            .with_location(false)
            .capture(OptionAssertions::is_some);
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Variant);
        assert_that!(failure.actual.as_deref()).is_equal_to(Some("None"));
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("is not the expected variant"));
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("Option::Some"));
    }

    #[test]
    fn nested_failures_of_a_positional_subject_are_children_located_by_index() {
        let failures = assert_that!([1, 2]).with_location(false).capture(|it| {
            it.contains_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                },
            ])
        });
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Predicate);
        assert_that!(failure.relation.as_deref())
            .is_equal_to(Some("does not exactly satisfy the assertions"));
        // The parent's own text does not embed the child.
        assert_that!(failure.description()).does_not_contain("Expected: 3");
        assert_that!(failure.facts.as_slice()).is_empty();

        assert_that!(failure.children.as_slice()).has_length(1);
        let child = &failure.children[0];
        assert_that!(child.kind).is_equal_to(FailureKind::Equality);
        assert_that!(child.actual.as_deref()).is_equal_to(Some("2"));
        assert_that!(child.expected.as_deref()).is_equal_to(Some("3"));
        assert_that!(child.facts.as_slice()).contains_exactly([Fact::new(Fact::INDEX, "1")]);
        assert_that!(child.subject_type_name).is_equal_to(core::any::type_name::<i32>());

        assert_that!(failure.to_string()).ends_with(indoc::indoc! {"
            does not exactly satisfy the assertions

            Nested failures:
              - At index 1:
                Expected: 3

                  Actual: 2
            -------- assertr --------
        "});
    }

    #[test]
    fn rejected_elements_of_a_matching_assertion_are_children_too() {
        let failures = assert_that!([1, 2, 3]).with_location(false).capture(|it| {
            it.contains_exactly_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 9,
                |it: &i32| *it == 3,
            ])
        });
        let child = &failures[0].children[0];

        assert_that!(child.kind).is_equal_to(FailureKind::Predicate);
        assert_that!(child.actual.as_deref()).is_equal_to(Some("2"));
        assert_that!(child.relation.as_deref()).is_equal_to(Some("does not match its predicate"));
        assert_that!(child.facts.as_slice()).contains_exactly([Fact::new(Fact::INDEX, "1")]);
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
                .map(|child| child.actual.as_deref())
                .collect::<Vec<_>>()
        )
        .contains_exactly([Some("1"), Some("2"), Some("3")]);
        assert_that!(children.iter().all(|child| child.facts.is_empty())).is_true();
    }

    #[test]
    fn a_downstream_failure_is_built_like_a_built_in_one() {
        let failures = assert_that!(1).with_location(false).capture(|it| {
            it.track_assertion();
            it.failure(FailureKind::Other)
                .relation("does not hold")
                .note("some evidence")
                .raise();
            it
        });
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Other);
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not hold"));
        assert_that!(failure.description()).is_equal_to("does not hold\n");
        assert_that!(failure.actual.as_deref()).is_none();
        assert_that!(failure.expected.as_deref()).is_none();
        assert_that!(failure.facts.as_slice()).contains_exactly([Fact::note("some evidence")]);
    }

    #[derive(PartialEq)]
    struct Secret(u32);

    #[derive(Clone, Copy)]
    struct SecretRenderer;

    impl ValueRenderer<Secret> for SecretRenderer {
        fn fmt(&self, value: &Secret, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Secret#{}", value.0)
        }
    }

    #[test]
    fn expected_and_actual_are_rendered_through_the_active_renderer() {
        let failures = assert_that!(Secret(1))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.is_equal_to(Secret(2)));
        let failure = &failures[0];

        assert_that!(failure.actual.as_deref()).is_equal_to(Some("Secret#1"));
        assert_that!(failure.expected.as_deref()).is_equal_to(Some("Secret#2"));

        let failures = assert_that!(Secret(1))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.is_not_equal_to(Secret(1)));
        assert_that!(failures[0].unexpected.as_deref()).is_equal_to(Some("Secret#1"));
    }

    #[test]
    fn children_inherit_the_renderer_and_the_location_setting() {
        let failures = assert_that!([Secret(1)])
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| {
                it.contains_satisfying(|element| {
                    element.is_equal_to(Secret(2));
                })
            });
        let child = &failures[0].children[0];

        assert_that!(child.actual.as_deref()).is_equal_to(Some("Secret#1"));
        assert_that!(child.expected.as_deref()).is_equal_to(Some("Secret#2"));
        assert_that!(child.location.is_none()).is_true();
    }
}
