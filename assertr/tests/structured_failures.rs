//! Tests for the structured-failure capture API:
//!
//! - `AssertThat::capture` runs assertions in capture mode inside a closure and returns the
//!   collected failures, making a forgotten capture structurally impossible.
//! - Captured failures are structured `AssertionFailure` values whose fields (location, subject
//!   name, asserted expression, subject type, actual, relation, expected, unexpected, facts,
//!   chain-level messages, children, kind) can be inspected without parsing formatted text.
//! - Values become owned rendered trees when failures are built. Adapters decide how to use those
//!   trees at the panic boundary or after capture.

use assertr::prelude::*;
use assertr::{Fact, FailureKind, renderer::Rendered};
use indoc::formatdoc;

fn rendered_text(value: &Rendered) -> String {
    let mut text = String::new();
    value.write(&mut text, true).unwrap();
    text
}

fn text(value: &Rendered) -> &str {
    match &value.body {
        assertr::renderer::RenderedBody::Text { text, .. } => text,
        body => panic!("expected a text node, got {body:?}"),
    }
}

fn text_opt(value: Option<&Rendered>) -> Option<&str> {
    value.map(text)
}

#[test]
fn capture_returns_structured_failures_with_separated_fields() {
    let failures = assert_that!(42)
        .with_subject_name("answer")
        .capture(|it| it.with_detail_message("user context").is_equal_to(43));

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| value.subject_name.as_deref())
                .is_equal_to(Some("answer"));
            element
                .derive(|value| &value.expression)
                .is_equal_to(Some("42"));
            element
                .derive(|value| &value.subject_type_name)
                .is_equal_to(core::any::type_name::<i32>());
            element.derive_owned(AssertionFailure::facts).is_empty();
            element
                .derive_owned(AssertionFailure::messages)
                .contains_exactly(["user context"]);
            element.derive_owned(AssertionFailure::constraint).is_none();
            element
                .derive_owned(|value| text_opt(value.expected.as_ref()))
                .is_equal_to(Some("43"));
            element
                .derive_owned(|value| text_opt(value.actual.as_ref()))
                .is_equal_to(Some("42"));
        },
    ]);
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

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| ToHumanReadableText.render(value))
                .contains("is not greater than");
            element
                .derive_owned(|value| value.messages.as_slice())
                .contains_exactly(["early"]);
        },
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| ToHumanReadableText.render(value))
                .contains("Expected: 1");
            element
                .derive_owned(|value| value.messages.as_slice())
                .contains_exactly(["early", "late"]);
        },
    ]);
}

#[test]
fn the_human_readable_adapter_renders_the_stable_format() {
    let failures = assert_that!(42)
        .with_location(false)
        .capture(|it| it.is_equal_to(43));

    assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {"
        -------- assertr --------
        Expression: `42`

        Expected: 43

          Actual: 42
        -------- assertr --------
    "});
}

#[test]
fn per_failure_diagnostics_are_exposed_as_facts() {
    let failures = assert_that!([1, 2, 3])
        .with_location(false)
        .capture(|it| it.contains_exactly([1, 9]));

    let failure = &failures[0];
    assert_that!(failure.facts.is_empty()).is_false();
    assert_that!(failure.messages.as_slice()).is_empty();
    // The rendered form still shows them under `Details:`.
    assert_that!(ToHumanReadableText.render(failure)).contains("Details:\n  - ");
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
                .fact(Fact::note("first detail\ncontinued detail"))
                .raise();
            it
        });

    assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(indoc::indoc! {"
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

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| ToHumanReadableText.render(value))
                .contains("xyz");
            element
                .derive(|value| &value.subject_type_name)
                .is_equal_to(core::any::type_name::<String>());
        },
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| ToHumanReadableText.render(value))
                .contains("Expected: 9");
            element
                .derive(|value| &value.subject_type_name)
                .is_equal_to(core::any::type_name::<usize>());
        },
    ]);
}

#[test]
fn capture_on_a_derived_assertion_is_scoped_to_that_chain() {
    let value = ("foo".to_owned(), 42);
    let root = assert_that!(value)
        .with_location(false)
        .with_detail_message("root context");

    let failures = root.derive_owned(|v| v.1).capture(|it| it.is_equal_to(43));

    // The derived chain's failures are returned locally instead of propagating to the panic-mode
    // root, while ancestor detail messages are preserved.
    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| value.messages.as_slice())
                .contains_exactly(["root context"]);
        },
    ]);

    // The root stays in panic mode and remains usable.
    root.is_equal_to(("foo".to_owned(), 42));
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
fn a_side_effect_only_adapter_can_consume_a_captured_failure() {
    use core::cell::Cell;
    use core::convert::Infallible;

    use assertr::failure::adapter::Adapter;

    struct Observer<'a>(&'a Cell<Option<FailureKind>>);

    impl Adapter<AssertionFailure> for Observer<'_> {
        type Output = ();
        type Error = Infallible;

        fn adapt(&self, failure: &AssertionFailure) -> Result<(), Self::Error> {
            self.0.set(Some(failure.kind));
            Ok(())
        }
    }

    let failures = assert_that!(1)
        .with_location(false)
        .capture(|it| it.is_equal_to(2));
    let observed = Cell::new(None);

    Observer(&observed).adapt(&failures[0]).unwrap();

    assert_that!(observed.get()).is_equal_to(Some(FailureKind::Equality));
}

#[test]
// The `if` around the panic keeps the closure's return type inferable; an `assert!` would change
// the panic payload.
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
    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|value| ToHumanReadableText.render(value))
                .contains("Expected: 43");
        },
    ]);

    assert_that!(42.verify(|it| it.be_equal_to(42))).is_empty();

    let failures = String::from("foo").verify_owned(|it| it.have_length(9));
    assert_that!(&failures).has_length(1);
}

/// One field-level test per assertion family: the fields carry everything the text carries.
mod fields {
    use super::{rendered_text, text, text_opt};
    use assertr::prelude::*;
    use assertr::renderer::{Rendered, RenderedBody, TypeHint};
    use assertr::{Fact, FailureKind};
    use core::{cell::RefCell, fmt};

    #[test]
    fn an_equality_failure_carries_expected_and_actual_without_a_relation() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_equal_to(43));
        let failure = &failures[0];

        assert_that!(failure.kind()).is_equal_to(FailureKind::Equality);
        assert_that!(text_opt(failure.actual())).is_equal_to(Some("42"));
        assert_that!(failure.actual().unwrap().type_name())
            .is_equal_to(Some(core::any::type_name::<i32>()));
        assert_that!(failure.actual.as_ref().unwrap().hint).is_equal_to(TypeHint::Short);
        assert_that!(failure.actual.as_ref().unwrap().shows_type_hint).is_false();
        assert_that!(failure.relation()).is_none();
        assert_that!(text_opt(failure.expected())).is_equal_to(Some("43"));
        assert_that!(text_opt(failure.unexpected())).is_none();
        assert_that!(failure.facts()).is_empty();
        assert_that!(failure.children()).is_empty();
        assert_that!(ToHumanReadableText.render(failure))
            .contains("Expected: 43\n\n  Actual: 42\n");
    }

    #[test]
    fn a_negated_assertion_carries_the_unexpected_value_instead_of_an_expected_one() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_not_equal_to(42));
        let failure = &failures[0];

        assert_that!(failure.relation()).is_equal_to(Some("is equal to"));
        assert_that!(text_opt(failure.expected())).is_none();
        assert_that!(text_opt(failure.unexpected())).is_equal_to(Some("42"));
        assert_that!(ToHumanReadableText.render(failure))
            .contains("Actual: 42\n\nis equal to\n\nUnexpected: 42\n");
    }

    #[test]
    fn an_ordering_failure_carries_the_relation_between_actual_and_expected() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_greater_than(100));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Ordering);
        assert_that!(text_opt(failure.actual.as_ref())).is_equal_to(Some("42"));
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("is not greater than"));
        assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("100"));
        assert_that!(ToHumanReadableText.render(failure))
            .contains("Actual: 42\n\nis not greater than\n\nExpected: 100\n");
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
        assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("2"));
        assert_that!(failure.facts.as_slice()).contains_exactly([Fact::labelled(
            "Actual length",
            assertr::renderer::RenderingContext::new(&DebugRenderer, RenderingBudget::default())
                .value(&1_usize),
        )]);
        assert_that!(failure.facts[0].label.as_ref()).is_equal_to("Actual length");
        assert_that!(text(&failure.facts[0].value)).is_equal_to("1");
    }

    #[test]
    fn a_membership_failure_carries_the_missing_element_as_expected() {
        let failures = assert_that!([1, 2])
            .with_location(false)
            .capture(|it| it.contains(3));
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Membership);
        assert_that!(rendered_text(failure.actual.as_ref().unwrap()))
            .is_equal_to("[\n    1,\n    2,\n]");
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not contain"));
        assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("3"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn an_order_free_group_retains_types_omissions_and_its_sorted_flag() {
        use std::collections::HashSet;

        let failures = assert_that!(HashSet::from([3, 1, 2]))
            .with_rendering_budget(RenderingBudget::builder().max_items(2).build())
            .with_location(false)
            .capture(|it| it.contains(9));
        let actual = failures[0].actual.as_ref().unwrap();

        assert_that!(actual.type_name).is_equal_to(Some(core::any::type_name::<HashSet<i32>>()));
        assert_that!(actual.shows_type_hint).is_true();
        let RenderedBody::Group {
            style,
            items,
            omitted,
            sorted,
        } = &actual.body
        else {
            panic!("expected a group node, got {:?}", actual.body);
        };
        assert_that!(*style).is_equal_to(assertr::renderer::GroupStyle::Set);
        assert_that!(*omitted).is_equal_to(1);
        assert_that!(*sorted).is_true();
        assert_that!(items.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<Rendered, Capture>| {
                element
                    .derive(|value| &value.type_name)
                    .is_equal_to(Some(core::any::type_name::<i32>()));
                element.derive_owned(text).is_equal_to("1");
            },
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("2");
            },
        ]);
    }

    #[test]
    fn a_map_retains_typed_key_value_entries_and_its_omission_count() {
        use std::collections::BTreeMap;

        let failures = assert_that!(BTreeMap::from([(1, 10), (2, 20)]))
            .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
            .with_location(false)
            .capture(|it| it.contains_key(&9));
        let actual = failures[0].actual.as_ref().unwrap();
        let RenderedBody::Map {
            entries,
            omitted,
            sorted,
        } = &actual.body
        else {
            panic!("expected a map node, got {:?}", actual.body);
        };

        assert_that!(*omitted).is_equal_to(1);
        assert_that!(*sorted).is_false();
        assert_that!(entries.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<(Rendered, Rendered), Capture>| {
                element
                    .derive(|value| &value.0.type_name)
                    .is_equal_to(Some(core::any::type_name::<i32>()));
                element
                    .derive(|value| &value.1.type_name)
                    .is_equal_to(Some(core::any::type_name::<i32>()));
                element
                    .derive_owned(|value| text(&value.0))
                    .is_equal_to("1");
                element
                    .derive_owned(|value| text(&value.1))
                    .is_equal_to("10");
            },
        ]);
    }

    #[test]
    fn an_expected_entry_list_retains_each_key_and_value_as_a_node() {
        use std::collections::BTreeMap;

        let failures = assert_that!(BTreeMap::from([("a", 1)]))
            .with_location(false)
            .capture(|it| it.contains_exactly_entries([("b", 2)]));
        let expected = failures[0].expected.as_ref().unwrap();
        let RenderedBody::EntryList {
            entries,
            omitted,
            sorted,
        } = &expected.body
        else {
            panic!("expected an entry-list node, got {:?}", expected.body);
        };

        assert_that!(*omitted).is_equal_to(0);
        assert_that!(*sorted).is_false();
        assert_that!(entries.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<(Rendered, Rendered), Capture>| {
                element
                    .derive_owned(|value| text(&value.0))
                    .is_equal_to("\"b\"");
                element
                    .derive_owned(|value| text(&value.1))
                    .is_equal_to("2");
            },
        ]);
    }

    #[test]
    fn an_unexpected_map_entry_retains_a_structured_tuple() {
        use std::collections::BTreeMap;

        let failures = assert_that!(BTreeMap::from([("a", 1)]))
            .with_location(false)
            .capture(|it| it.does_not_contain_entry("a", 1));
        let unexpected = failures[0].unexpected.as_ref().unwrap();
        let RenderedBody::Tuple { items } = &unexpected.body else {
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
    fn a_compact_structural_key_retains_its_tree_and_inline_layout() {
        use assertr::renderer::GroupStyle;

        let failures = assert_that!([4, 6]).with_location(false).capture(|it| {
            it.track_assertion();
            it.failure(FailureKind::Other)
                .fact(Fact::key(it.render().values(it.actual(), GroupStyle::List)))
                .raise();
            it
        });
        let values = &failures[0]
            .facts
            .iter()
            .find(|fact| fact.label == Fact::KEY)
            .unwrap()
            .value;
        let RenderedBody::Group { items, .. } = &values.body else {
            panic!("expected a group node, got {:?}", values.body);
        };

        assert_that!(values.compact).is_true();
        assert_that!(items.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("4");
            },
            |element: AssertThat<Rendered, Capture>| {
                element.derive_owned(text).is_equal_to("6");
            },
        ]);
        assert_that!(ToHumanReadableText.render(&failures[0])).contains("key: [4, 6]");
    }

    #[test]
    fn a_result_variant_retains_the_owner_and_inner_value_as_nodes() {
        let failures = assert_that!(Result::<i32, &str>::Err("boom"))
            .with_location(false)
            .capture(ResultAssertions::is_ok);
        let actual = failures[0].actual.as_ref().unwrap();

        assert_that!(actual.type_name)
            .is_equal_to(Some(core::any::type_name::<Result<i32, &str>>()));
        let RenderedBody::Variant { name, value } = &actual.body else {
            panic!("expected a variant node, got {:?}", actual.body);
        };
        assert_that!(*name).is_equal_to("Err");
        assert_that!(value.type_name).is_equal_to(Some(core::any::type_name::<&str>()));
        assert_that!(text(value)).is_equal_to("\"boom\"");
    }

    #[test]
    fn a_struct_adapter_retains_its_field_as_a_node() {
        let cell = RefCell::new(42);
        let failures = assert_that!(cell)
            .with_location(false)
            .capture(RefCellAssertions::is_borrowed);
        let actual = failures[0].actual.as_ref().unwrap();

        assert_that!(actual.type_name).is_equal_to(Some(core::any::type_name::<RefCell<i32>>()));
        let RenderedBody::Struct { name, fields } = &actual.body else {
            panic!("expected a struct node, got {:?}", actual.body);
        };
        assert_that!(*name).is_equal_to("RefCell");
        assert_that!(fields.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<(&str, Rendered), Capture>| {
                element.derive(|value| &value.0).is_equal_to("value");
                element
                    .derive(|value| &value.1.type_name)
                    .is_equal_to(Some(core::any::type_name::<i32>()));
                element
                    .derive_owned(|value| text(&value.1))
                    .is_equal_to("42");
            },
        ]);
    }

    #[test]
    fn an_inaccessible_struct_field_remains_a_placeholder_node() {
        let cell = RefCell::new(42);
        let borrow = cell.borrow_mut();
        let failures = assert_that!(&cell)
            .with_location(false)
            .capture(RefCellAssertions::is_not_mutably_borrowed);
        drop(borrow);

        let RenderedBody::Struct { fields, .. } = &failures[0].actual.as_ref().unwrap().body else {
            panic!("expected a struct node");
        };
        assert_that!(&fields[0].1.body).is_equal_to(RenderedBody::Placeholder("<borrowed>"));
    }

    #[test]
    fn a_variant_failure_names_the_expected_variant() {
        let failures = assert_that!(Option::<i32>::None)
            .with_location(false)
            .capture(OptionAssertions::is_some);
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Variant);
        assert_that!(text_opt(failure.actual.as_ref())).is_equal_to(Some("None"));
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("is not the expected variant"));
        assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("Option::Some"));
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

        assert_that!(failure.kind).is_equal_to(FailureKind::Matching);
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not match"));
        assert_that!(failure.facts.as_slice()).is_empty();

        assert_that!(failure.children.as_slice()).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(AssertionFailure::kind)
                    .is_equal_to(FailureKind::Equality);
                element
                    .derive_owned(|value| text_opt(value.actual.as_ref()))
                    .is_equal_to(Some("2"));
                element
                    .derive_owned(|value| text_opt(value.expected.as_ref()))
                    .is_equal_to(Some("3"));
                element
                    .derive(|value| &value.path)
                    .is_equal_to([assertr::failure::PathSegment::Index(1)]);
                element.derive(|value| &value.facts).is_empty();
                element
                    .derive(|value| &value.subject_type_name)
                    .is_equal_to(core::any::type_name::<i32>());
            },
        ]);

        assert_that!(ToHumanReadableText.render(failure)).ends_with(indoc::indoc! {"
            does not match

            Nested failures:
              - At [1]:
                Expected: 3

                  Actual: 2
            -------- assertr --------
        "});
    }

    #[test]
    fn rejected_elements_of_a_matching_assertion_are_children_too() {
        let failures = assert_that!([1, 2, 3]).with_location(false).capture(|it| {
            it.contains_exactly_matching(assertr::matchers::predicate_list([
                |it: &i32| *it == 1,
                |it: &i32| *it == 9,
                |it: &i32| *it == 3,
            ]))
        });
        let child = &failures[0].children[0];

        assert_that!(child.kind).is_equal_to(FailureKind::Matching);
        assert_that!(child.actual).is_none();
        assert_that!(child.constraint.as_ref().unwrap().relation)
            .is_equal_to("satisfies the predicate");
        assert_that!(child.relation.as_deref())
            .is_equal_to(Some("does not satisfy the constraint"));
        assert_that!(child.path).is_equal_to([assertr::failure::PathSegment::Index(1)]);
        assert_that!(child.facts).is_empty();
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
            it.failure(FailureKind::Other)
                .relation("does not hold")
                .fact(Fact::note("some evidence"))
                .raise();
            it
        });
        let failure = &failures[0];

        assert_that!(failure.kind).is_equal_to(FailureKind::Other);
        assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not hold"));
        assert_that!(ToHumanReadableText.render(failure)).contains("does not hold\n");
        assert_that!(text_opt(failure.actual())).is_none();
        assert_that!(text_opt(failure.expected())).is_none();
        assert_that!(failure.facts()).contains_exactly([Fact::note("some evidence")]);
        let fact = assert_that!(failure.facts()[0]);
        fact.derive_owned(Fact::label).is_empty();
        fact.derive(Fact::value)
            .derive_owned(Rendered::type_name)
            .is_none();
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

        assert_that!(text_opt(failure.actual.as_ref())).is_equal_to(Some("Secret#1"));
        assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("Secret#2"));
        assert_that!(failure.actual.as_ref().unwrap().type_name)
            .is_equal_to(Some(core::any::type_name::<Secret>()));

        let failures = assert_that!(Secret(1))
            .with_renderer(SecretRenderer)
            .with_location(false)
            .capture(|it| it.is_not_equal_to(Secret(1)));
        assert_that!(text_opt(failures[0].unexpected.as_ref())).is_equal_to(Some("Secret#1"));
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

        assert_that!(text_opt(child.actual.as_ref())).is_equal_to(Some("Secret#1"));
        assert_that!(text_opt(child.expected.as_ref())).is_equal_to(Some("Secret#2"));
        assert_that!(child.location.is_none()).is_true();
    }
}

mod matcher_metadata {
    use super::*;

    #[test]
    fn matcher_paths_and_constraints_preserve_metadata() {
        use assertr::{
            failure::PathSegment,
            matchers::{all_of, equal_to, predicate},
        };
        let failures = assert_that!([1])
            .with_subject_name("rows")
            .with_detail_message("request context")
            .capture(|it| {
                it.matches(elements_are![all_of((
                    equal_to(2),
                    predicate(|_: &i32| false)
                ))])
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
                    .is_equal_to(["request context"]);
                element
                    .derive_owned(|value| value.location.unwrap().file())
                    .ends_with("structured_failures.rs");
                element
                    .derive_owned(AssertionFailure::children)
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
                                            .derive(|constraint| &constraint.relation)
                                            .is_equal_to("satisfies the predicate");
                                    },
                                );
                            }
                        }
                    }));
            },
        ]);
    }

    #[test]
    fn typed_paths_take_precedence_over_legacy_location_facts() {
        use assertr::failure::{FailureBuilder, PathSegment};
        let child = FailureBuilder::detached::<i32>(FailureKind::Matching)
            .path([
                PathSegment::Field("rows"),
                PathSegment::Index(1),
                PathSegment::Field("id"),
            ])
            .relation("does not match")
            .build()
            .located_at(Fact::index(1));
        let root = FailureBuilder::detached::<()>(FailureKind::Matching)
            .child(child)
            .build();
        let text = ToHumanReadableText.render(&root);
        assert_that!(text)
            .contains("At .rows[1].id:")
            .does_not_contain("At index");
    }
}
