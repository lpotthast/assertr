//! Borrowed-target identity algorithms and diagnostics for finite collections.

use alloc::vec::Vec;
use core::{borrow::Borrow, ptr};

use super::{Collection, StableOrder};
use crate::{
    AssertThat, Mode,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{GroupStyle, RenderingOrder},
    util::matching::match_bipartite,
};

const METADATA_NOTE: &str = "Some pointers have equal data addresses but different metadata.";

#[track_caller]
pub(super) fn assert_contains_same_instance_as<C, U: ?Sized, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &U,
) where
    C: Collection,
    C::Item: Borrow<U>,
    M: Mode,
{
    this.track_assertion();
    let actual = this.actual();
    let mut same_address = false;
    if !actual.elements().any(|element| {
        let element = <C::Item as Borrow<U>>::borrow(element);
        same_address |= ptr::addr_eq(element, expected);
        ptr::eq(element, expected)
    }) {
        let rendering = this.render().identities();
        let mut failure = this
            .failure(FailureKind::Membership)
            .actual(rendering.borrowed_collection::<U, _>(actual))
            .relation("does not contain the same instance as")
            .expected(rendering.value(expected));
        if same_address {
            failure = failure.fact(Fact::note(METADATA_NOTE));
        }
        failure.raise();
    }
}

#[track_caller]
pub(super) fn assert_does_not_contain_same_instance_as<C, U: ?Sized, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &U,
) where
    C: Collection,
    C::Item: Borrow<U>,
    M: Mode,
{
    this.track_assertion();
    let actual = this.actual();
    if actual
        .elements()
        .any(|element| ptr::eq(<C::Item as Borrow<U>>::borrow(element), expected))
    {
        let rendering = this.render().identities();
        this.failure(FailureKind::Membership)
            .actual(rendering.borrowed_collection::<U, _>(actual))
            .relation("contains the same instance as")
            .unexpected(rendering.value(expected))
            .raise();
    }
}

#[track_caller]
pub(super) fn assert_contains_exactly_same_instances<C, U: ?Sized, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[&U],
) where
    C: StableOrder,
    C::Item: Borrow<U>,
    M: Mode,
    R: crate::ValueRenderer<usize>,
{
    this.track_assertion();
    let actual = this.actual();
    let mismatch = actual
        .elements()
        .map(<C::Item as Borrow<U>>::borrow)
        .zip(expected.iter().copied())
        .enumerate()
        .find(|(_, (element, expected))| !ptr::eq(*element, *expected));
    if actual.length() != expected.len() || mismatch.is_some() {
        let rendering = this.render().identities();
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(rendering.stable_borrowed_collection::<U, _>(actual))
            .relation("does not contain exactly the same instances in order")
            .expected(rendering.borrowed_values::<U, _>(expected, GroupStyle::List));
        if actual.length() != expected.len() {
            failure = failure
                .fact(Fact::labelled(
                    "Actual length",
                    this.render().value(&actual.length()),
                ))
                .fact(Fact::labelled(
                    "Expected length",
                    this.render().value(&expected.len()),
                ));
        }
        if let Some((index, (element, expected))) = mismatch {
            if ptr::addr_eq(element, expected) {
                failure = failure.fact(Fact::note(METADATA_NOTE));
            }
            if rendering.max_items() == 0 {
                failure = failure.omitted(1, "unmatched element");
            } else {
                failure = failure.child(
                    FailureBuilder::detached::<U>(FailureKind::Equality)
                        .actual(rendering.value(element))
                        .relation("is not the same instance as")
                        .expected(rendering.value(expected))
                        .build()
                        .located_at(Fact::index(index)),
                );
            }
        }
        failure.raise();
    }
}

#[track_caller]
pub(super) fn assert_contains_exactly_same_instances_in_any_order<C, U: ?Sized, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[&U],
) where
    C: Collection,
    C::Item: Borrow<U>,
    M: Mode,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual
        .elements()
        .map(<C::Item as Borrow<U>>::borrow)
        .collect::<Vec<_>>();
    let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
        ptr::eq(elements[a], expected[e])
    });
    if !matched.is_exact() {
        let missing = matched
            .unmatched_expected
            .iter()
            .map(|index| expected[*index])
            .collect::<Vec<_>>();
        let unexpected = matched
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect::<Vec<_>>();
        let rendering = this.render().identities();
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(rendering.borrowed_collection::<U, _>(actual))
            .relation("does not contain exactly the same instances in any order")
            .expected(rendering.borrowed_values::<U, _>(expected, GroupStyle::List));
        if !missing.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Instances not found",
                rendering.borrowed_values::<U, _>(&missing, GroupStyle::List),
            ));
        }
        if !unexpected.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Instances not expected",
                rendering
                    .borrowed_values::<U, _>(&unexpected, GroupStyle::List)
                    .sort_for_rendering(
                        C::PRESENTATION.order() == RenderingOrder::SortByRenderedText,
                    ),
            ));
        }
        // Unmatched pairs cannot have full pointer equality, so a shared address here proves that
        // pointer metadata accounts for at least one difference.
        if unexpected.iter().any(|actual| {
            missing
                .iter()
                .any(|expected| ptr::addr_eq(*actual, *expected))
        }) {
            failure = failure.fact(Fact::note(METADATA_NOTE));
        }
        failure.raise();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::renderer::{CollectionPresentation, Rendered, RenderedBody, RenderingContext};
    use crate::test_support::{
        NoRenderer, NumericRenderer, PreservedBag, UnorderedSet, rendered_text,
    };
    use indoc::formatdoc;

    struct Opaque {
        _byte: u8,
    }

    fn keys() -> [Opaque; 3] {
        [
            Opaque { _byte: 1 },
            Opaque { _byte: 1 },
            Opaque { _byte: 1 },
        ]
    }

    fn items(value: &Rendered) -> &[Rendered] {
        let RenderedBody::Group { items, .. } = &value.body else {
            panic!("expected an identity group");
        };
        items
    }

    mod contains_same_instance_as {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0]].must().contain_same_instance_as(&keys[0]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_same_instance_as(&keys[1])
            );
        }

        #[test]
        fn finds_the_borrowed_pointee_instead_of_the_reference_slot() {
            let keys = keys();
            let actual = [&keys[0], &keys[1]];
            let assertion = assert_that!(actual)
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&keys[1]);
            assert_eq!(assertion.state.number_of_assertions.borrow().0, 1);
        }

        #[test]
        fn rejects_distinct_equal_instances_with_an_exact_report() {
            let keys = keys();
            let actual = [&keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| it.contains_same_instance_as(&keys[1]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(FailureKind::Membership);
                    element.derive(|value| value).has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                ]

                does not contain the same instance as

                Expected: {e:p}
                -------- assertr --------
            ", a = &keys[0], e = &keys[1]});
                },
            ]);
        }

        #[test]
        fn rejects_an_empty_collection() {
            let keys = keys();
            let failures = assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(&keys[0]));
            assert_eq!(failures.len(), 1);
        }

        #[test]
        fn explains_a_slice_metadata_mismatch() {
            let data = [1, 2];
            let short = &data[..1];
            let long = &data[..];
            let failures = assert_that!([short])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(long));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.facts)
                        .is_equal_to([Fact::note(METADATA_NOTE)]);
                    element
                        .derive_owned(|value| {
                            rendered_text(&items(value.actual.as_ref().unwrap())[0])
                        })
                        .is_equal_to(rendered_text(element.actual().expected.as_ref().unwrap()));
                },
            ]);
        }

        #[test]
        fn panic_mode_raises_the_identity_failure() {
            let keys = keys();
            assert_that_panic_by(|| {
                assert_that!([&keys[0]])
                    .with_renderer(NoRenderer)
                    .contains_same_instance_as(&keys[1]);
            })
            .has_type::<String>()
            .contains("does not contain the same instance as");
        }
    }

    mod does_not_contain_same_instance_as {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0]].must().not_contain_same_instance_as(&keys[1]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                does_not_contain_same_instance_as(&keys[0])
            );
        }

        #[test]
        fn accepts_missing_instances_and_empty_collections() {
            let keys = keys();
            assert_that!([&keys[0]])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&keys[1]);
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&keys[0]);
        }

        #[test]
        fn reports_the_unexpected_instance_without_an_index() {
            let keys = keys();
            let actual = [&keys[0], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| it.does_not_contain_same_instance_as(&keys[0]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.expected.is_none())
                        .is_true();
                    element
                        .derive_owned(|value| value.facts.is_empty())
                        .is_true();
                    element
                        .derive_owned(|value| value.children.is_empty())
                        .is_true();
                    element.derive(|value| value).has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                    {a:p},
                ]

                contains the same instance as

                Unexpected: {a:p}
                -------- assertr --------
            ", a = &keys[0]});
                },
            ]);
        }

        #[test]
        fn same_address_with_different_metadata_is_not_a_match() {
            let data = [1, 2];
            assert_that!([&data[..1]])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&data[..]);
        }
    }

    mod contains_exactly_same_instances {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[1], &keys[0]]
                .must()
                .contain_exactly_same_instances([&keys[1], &keys[0]]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_exactly_same_instances([&keys[1]])
            );
        }

        #[test]
        fn accepts_the_requested_candidate_order_and_duplicate_references() {
            let keys = keys();
            let candidates = [&keys[1], &keys[2], &keys[0]];
            assert_that!(candidates)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([&keys[1], &keys[2], &keys[0]]);
            let repeated = [&keys[0], &keys[0]];
            let assertion = assert_that!(repeated)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([&keys[0], &keys[0]]);
            assert_eq!(assertion.state.number_of_assertions.borrow().0, 1);
        }

        #[test]
        fn reports_the_first_reordered_pair_as_an_indexed_child() {
            let keys = keys();
            let actual = [&keys[1], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly_same_instances([&keys[0], &keys[1]]));
            assert_that!(failures).contains_exactly_satisfying([
                |item: AssertThat<AssertionFailure, Capture>| {
                    item.derive(|subject| &subject.children)
                        .contains_exactly_satisfying([
                            |element: AssertThat<AssertionFailure, Capture>| {
                                element
                                    .derive(|value| &value.facts)
                                    .is_equal_to([Fact::index(0)]);
                            },
                        ]);
                    item.derive(|subject| subject).has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {b:p},
                    {a:p},
                ]

                does not contain exactly the same instances in order

                Expected: [
                    {a:p},
                    {b:p},
                ]

                Nested failures:
                  - At index 0:
                    Actual: {b:p}

                    is not the same instance as

                    Expected: {a:p}
                -------- assertr --------
            ", a = &keys[0], b = &keys[1]});
                },
            ]);
        }

        #[test]
        fn checks_lengths_even_when_the_common_prefix_matches() {
            let keys = keys();
            for (actual, expected) in [
                (vec![&keys[0]], vec![]),
                (vec![], vec![&keys[0]]),
                (vec![&keys[0], &keys[0]], vec![&keys[0]]),
                (vec![&keys[0]], vec![&keys[0], &keys[0]]),
            ] {
                let failures = assert_that!(actual)
                    .with_renderer(NumericRenderer)
                    .capture(|it| it.contains_exactly_same_instances(&expected));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| &item.facts).is_equal_to([
                            Fact::labelled(
                                "Actual length",
                                RenderingContext::new(&DebugRenderer, RenderingBudget::default())
                                    .value(&actual.len()),
                            ),
                            Fact::labelled(
                                "Expected length",
                                RenderingContext::new(&DebugRenderer, RenderingBudget::default())
                                    .value(&expected.len()),
                            ),
                        ]);
                        element
                            .derive_owned(|item| item.children.is_empty())
                            .is_true();
                    },
                ]);
            }
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([] as [&Opaque; 0]);
        }

        #[test]
        fn checks_metadata_and_keeps_the_borrowed_child_type() {
            let data = [1, 2];
            let failures = assert_that!([&data[..1]])
                .with_renderer(NumericRenderer)
                .capture(|it| it.contains_exactly_same_instances([&data[..]]));
            assert_eq!(failures[0].facts, [Fact::note(METADATA_NOTE)]);
            assert_eq!(failures[0].children[0].subject_type_name, "[i32]");
        }

        #[test]
        fn length_facts_need_only_a_numeric_renderer() {
            use crate::test_support::assert_custom_value;
            struct Numbers;
            impl ValueRenderer<usize> for Numbers {
                fn fmt(
                    &self,
                    value: &usize,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "custom({value})")
                }
            }
            let keys = keys();
            let subject = [&keys[0]];
            let failures = assert_that!(subject)
                .with_renderer(Numbers)
                .with_location(false)
                .capture(|it| it.contains_exactly_same_instances([] as [&Opaque; 0]));

            assert_custom_value(&failures[0].facts[0].value, &1_usize);
            assert_custom_value(&failures[0].facts[1].value, &0_usize);
            let pointer = format!("{:p}", subject[0]);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: [
                    {pointer},
                ]

                does not contain exactly the same instances in order

                Expected: []

                Details:
                  - Actual length: custom(1)
                  - Expected length: custom(0)
                -------- assertr --------
            "});
        }
    }

    mod contains_exactly_same_instances_in_any_order {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0], &keys[1]]
                .must()
                .contain_exactly_same_instances_in_any_order([&keys[1], &keys[0]]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_exactly_same_instances_in_any_order([&keys[1]])
            );
        }

        #[test]
        fn accepts_reordering_and_matching_duplicate_counts() {
            let keys = keys();
            let repeated = [&keys[0], &keys[1], &keys[0]];
            let assertion = assert_that!(repeated)
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([&keys[0], &keys[0], &keys[1]]);
            assert_eq!(assertion.state.number_of_assertions.borrow().0, 1);
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([] as [&Opaque; 0]);
        }

        #[test]
        fn reports_missing_and_unexpected_duplicate_occurrences() {
            let keys = keys();
            let actual = [&keys[0], &keys[0], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| {
                    it.contains_exactly_same_instances_in_any_order([&keys[0], &keys[1], &keys[1]])
                });
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.children.is_empty())
                        .is_true();
                    element.derive(|value| value).has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                    {a:p},
                    {a:p},
                ]

                does not contain exactly the same instances in any order

                Expected: [
                    {a:p},
                    {b:p},
                    {b:p},
                ]

                Details:
                  - Instances not found: [
                        {b:p},
                        {b:p},
                    ]
                  - Instances not expected: [
                        {a:p},
                        {a:p},
                    ]
                -------- assertr --------
            ", a = &keys[0], b = &keys[1]});
                },
            ]);
        }

        #[test]
        fn rejects_missing_and_extra_occurrences_including_empty_sides() {
            let keys = keys();
            for (actual, expected, label) in [
                (vec![&keys[0]], vec![], "Instances not expected"),
                (vec![], vec![&keys[0]], "Instances not found"),
                (
                    vec![&keys[0], &keys[0]],
                    vec![&keys[0]],
                    "Instances not expected",
                ),
                (
                    vec![&keys[0]],
                    vec![&keys[0], &keys[0]],
                    "Instances not found",
                ),
            ] {
                let failures = assert_that!(actual)
                    .with_renderer(NoRenderer)
                    .capture(|it| it.contains_exactly_same_instances_in_any_order(&expected));
                assert_that!(failures).contains_exactly_satisfying([
                    |item: AssertThat<AssertionFailure, Capture>| {
                        item.derive(|subject| &subject.facts)
                            .contains_exactly_satisfying([
                                |element: AssertThat<crate::Fact, Capture>| {
                                    element.derive(|value| &value.label).is_equal_to(label);
                                },
                            ]);
                    },
                ]);
            }
        }

        #[test]
        fn distinguishes_slice_metadata_when_matching_instances() {
            let data = [1, 2];
            let short = &data[..1];
            let long = &data[..];
            assert_that!([short, long])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([long, short]);
            let failures = assert_that!([short, short])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_exactly_same_instances_in_any_order([short, long]));
            assert_eq!(
                failures[0].facts.last().unwrap(),
                &Fact::note(METADATA_NOTE)
            );
        }
    }

    mod adapters {
        use super::*;
        use alloc::{
            collections::{BTreeSet, BinaryHeap, LinkedList, VecDeque},
            rc::Rc,
        };

        fn check_ordered<C: StableOrder + ?Sized>(actual: &C, expected: [&Opaque; 3])
        where
            C::Item: Borrow<Opaque>,
        {
            assert_that!(actual)
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(expected[1])
                .contains_exactly_same_instances(expected)
                .contains_exactly_same_instances_in_any_order([
                    expected[2],
                    expected[0],
                    expected[1],
                ]);
        }

        #[test]
        fn ordered_adapters_borrow_owned_values_and_references() {
            let keys = keys();
            let expected = [&keys[0], &keys[1], &keys[2]];
            check_ordered(&keys, expected);
            check_ordered(keys.as_slice(), expected);
            check_ordered(&expected, expected);
            check_ordered(&expected.to_vec(), expected);
            check_ordered(&VecDeque::from(expected), expected);
            check_ordered(&LinkedList::from(expected), expected);
        }

        #[test]
        fn smart_pointers_borrow_their_opaque_targets() {
            let boxed = Box::new(Opaque { _byte: 1 });
            assert_that!([&boxed])
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&boxed);
            let actual = [boxed];
            assert_that!(actual)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([actual[0].as_ref()]);
            let shared = Rc::new(Opaque { _byte: 1 });
            assert_that!([Rc::clone(&shared), Rc::clone(&shared)])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([shared.as_ref(), shared.as_ref()]);
            #[cfg(target_has_atomic = "ptr")]
            {
                let shared = alloc::sync::Arc::new(Opaque { _byte: 1 });
                assert_that!([alloc::sync::Arc::clone(&shared)])
                    .with_renderer(NoRenderer)
                    .contains_same_instance_as(shared.as_ref());
            }
        }

        #[test]
        fn order_free_adapters_expose_instance_membership() {
            let values = [1, 2];
            let expected = [&values[1], &values[0]];
            assert_that!(BTreeSet::from(expected))
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&values[0])
                .contains_exactly_same_instances_in_any_order(expected);
            assert_that!(BinaryHeap::from(expected))
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order(expected);
            #[cfg(feature = "std")]
            assert_that!(std::collections::HashSet::from(expected))
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order(expected);
        }

        #[test]
        fn supports_str_and_trait_object_targets() {
            trait Marker {}
            impl Marker for Opaque {}

            let text = String::from("text");
            assert_that!([text.as_str()])
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(text.as_str())
                .contains_exactly_same_instances([text.as_str()]);
            let key = Opaque { _byte: 1 };
            let object: &dyn Marker = &key;
            assert_that!([object])
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(object)
                .contains_exactly_same_instances([object])
                .contains_exactly_same_instances_in_any_order([object]);
        }
    }

    mod rendering {
        use super::*;

        #[test]
        fn sorts_order_free_evidence_using_presentation_metadata() {
            let actual = UnorderedSet(vec![1, 2]);
            let expected = [9, 10];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .capture(|it| {
                    it.contains_exactly_same_instances_in_any_order([&expected[1], &expected[0]])
                });
            let mut addresses = actual
                .0
                .iter()
                .map(|value| format!("{value:p}"))
                .collect::<Vec<_>>();
            addresses.sort();
            for rendered in [
                failures[0].actual.as_ref().unwrap(),
                &failures[0].facts[1].value,
            ] {
                assert!(matches!(
                    rendered.body,
                    RenderedBody::Group { sorted: true, .. }
                ));
                assert_eq!(
                    items(rendered)
                        .iter()
                        .map(rendered_text)
                        .collect::<Vec<_>>(),
                    addresses
                );
            }
            let actual = PreservedBag(vec![2, 1]);
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(&expected[0]));
            assert!(matches!(
                failures[0].actual.as_ref().unwrap().body,
                RenderedBody::Group { sorted: false, .. }
            ));
        }

        #[test]
        fn positional_evidence_ignores_presentation_sorting() {
            struct SortedPresentation<'a>([&'a Opaque; 2]);
            impl HasLength for SortedPresentation<'_> {
                fn length(&self) -> usize {
                    2
                }
            }
            impl<'a> Collection for SortedPresentation<'a> {
                type Item = &'a Opaque;
                const PRESENTATION: CollectionPresentation =
                    CollectionPresentation::list().with_order(RenderingOrder::SortByRenderedText);
                fn elements(&self) -> impl Iterator<Item = &Self::Item> {
                    self.0.iter()
                }
            }
            impl StableOrder for SortedPresentation<'_> {}
            let keys = keys();
            let mut references = [&keys[0], &keys[1]];
            references
                .sort_by_key(|value| core::cmp::Reverse(format!("{value:p}", value = *value)));
            let actual = SortedPresentation(references);
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .capture(|it| it.contains_exactly_same_instances([references[1], references[0]]));
            let rendered = failures[0].actual.as_ref().unwrap();
            assert!(matches!(
                rendered.body,
                RenderedBody::Group { sorted: false, .. }
            ));
            assert_eq!(
                items(rendered)[0].type_name,
                Some(core::any::type_name::<Opaque>())
            );
            assert_eq!(
                rendered_text(&items(rendered)[0]),
                format!("{:p}", references[0])
            );
            assert_eq!(failures[0].children[0].facts, [Fact::index(0)]);
        }

        #[test]
        fn budgets_limit_evidence_without_limiting_comparison() {
            let keys = keys();
            let actual = [&keys[0], &keys[1]];
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_rendering_budget(
                    RenderingBudget::builder()
                        .max_items(1)
                        .max_leaf_characters(2)
                        .build(),
                )
                .capture(|it| it.contains_exactly_same_instances([&keys[0], &keys[2]]));
            let rendered = failures[0].actual.as_ref().unwrap();
            assert!(matches!(
                rendered.body,
                RenderedBody::Group { omitted: 1, .. }
            ));
            assert!(
                matches!(&items(rendered)[0].body, RenderedBody::Text { text, omitted_characters } if text == "0x" && *omitted_characters > 0)
            );
            assert_eq!(failures[0].children[0].facts, [Fact::index(1)]);

            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_rendering_budget(
                    RenderingBudget::builder()
                        .max_items(0)
                        .max_leaf_characters(0)
                        .build(),
                )
                .capture(|it| {
                    it.contains_same_instance_as(&keys[1])
                        .contains_exactly_same_instances([&keys[0], &keys[2]])
                        .contains_exactly_same_instances_in_any_order([&keys[0], &keys[2]])
                });
            assert_that!(failures).contains_exactly_satisfying([false, true].map(|unordered| {
                move |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive(|failure| &failure.actual)
                        .is_some_satisfying(|actual| {
                            actual.derive_owned(items).is_empty();
                        });
                    failure.derive(|failure| &failure.children).is_empty();
                    let facts = failure.derive(|failure| &failure.facts);
                    if unordered {
                        facts.contains_exactly_satisfying(
                            [|fact: AssertThat<Fact, Capture>| {
                                fact.derive_owned(|fact| items(&fact.value)).is_empty();
                            }; 2],
                        );
                    } else {
                        facts.contains_exactly([Fact::note("... 1 more unmatched element ...")]);
                    }
                }
            }));
        }
    }
}
