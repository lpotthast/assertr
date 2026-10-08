//! Exact unordered assignment and its diagnostic evidence.
//!
//! The assignment search only probes pairs. A sparse cache stores each probed pair's truth, so
//! the search evaluates a pair at most once. When a diagnostic rejection is explained, only the
//! pairs involving a missing slot or an unexpected occurrence are evaluated again with
//! diagnostics. Their evidence is recorded directly into the group of that slot or occurrence,
//! through the normal budgeted context scopes.

use crate::expectation::composite_items;
use crate::{
    assertions::collection::Collection,
    expectation::AssertionContext,
    expectation::Evidence,
    expectation::Expectation,
    failure::AssertionFailure,
    failure::Fact,
    failure::{FailureBuilder, FailureKind},
    matchers::MatcherList,
    renderer::RenderingOrder,
    renderer::ValueRenderer,
    util::matching::{BipartiteMatchResult, assign_exactly},
};
use alloc::{collections::BTreeMap, vec::Vec};

/// Exact one-to-one order-free matching, preserving multiplicity.
#[derive(Debug, Clone)]
pub struct ElementsAreInAnyOrder<L>(L);

/// Matches every actual element to one distinct expectation using maximum bipartite matching.
///
/// The assignment search probes each actual/expectation pair at most once. Probes also reject
/// unequal lengths without comparisons and stop at the first occurrence that cannot be assigned.
///
/// A diagnostic rejection reports each missing expectation with the failures of every element
/// against it, and each unexpected element with its failures against the satisfied expectations.
/// These pairs are evaluated again to explain them, so their matchers can run twice. An unexpected
/// element that satisfies an already satisfied expectation is reported through that expectation's
/// description instead, without requiring an element renderer. Groups beyond the rendering budget
/// are only counted and evaluate no pairs. Budgets bound neither the search nor its sparse truth
/// cache, which can require quadratic space.
///
/// The `at slot` fact identifies a zero-based expectation position, never an actual collection
/// index.
pub fn elements_are_in_any_order<L>(list: L) -> ElementsAreInAnyOrder<L> {
    ElementsAreInAnyOrder(list)
}

impl<C: Collection + ?Sized, R, L> Expectation<C, R> for ElementsAreInAnyOrder<L>
where
    R: ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
{
    composite_items!(C);
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let actual = actual.elements().collect::<Vec<_>>();
        let mut pairs = Pairs {
            list: &self.0,
            actual: &actual,
            settings,
            cache: BTreeMap::new(),
        };
        let probe = context.is_probe();
        let result = match assign_exactly(probe, actual.len(), self.0.len(), |index, slot| {
            pairs.matches(index, slot)
        }) {
            Ok(()) => return Ok(()),
            Err(None) => return Err(context.into_evidence()),
            Err(Some(result)) => result,
        };
        let order = C::PRESENTATION.order();
        for &slot in &result.unmatched_expected {
            context.record(|context| pairs.missing::<C>(slot, order, context));
        }
        if !result.unmatched_actual.is_empty() {
            context.record(|context| pairs.unexpected::<C>(&result, order, context));
        }
        Err(context.into_evidence())
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => context.describe_list::<C::Item, _>(
                &self.0,
                failure.relation("has exactly these elements in any order"),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

/// The pairs of actual occurrences and expectation slots, with the truth of each probed pair.
struct Pairs<'a, 'r, A: ?Sized, L, R> {
    list: &'a L,
    actual: &'a [&'a A],
    settings: &'a AssertionContext<'r, R>,
    // A sparse cache avoids allocating the full Cartesian product for easy exact matches.
    cache: BTreeMap<(usize, usize), bool>,
}

impl<A: ?Sized, L, R> Pairs<'_, '_, A, L, R>
where
    L: MatcherList<A, R>,
    R: ValueRenderer<usize>,
{
    /// Probes a pair once, without diagnostics.
    fn matches(&mut self, index: usize, slot: usize) -> bool {
        let (list, actual, settings) = (self.list, self.actual, self.settings);
        *self.cache.entry((index, slot)).or_insert_with(|| {
            let mut probe = settings.isolated().with_diagnostics(false);
            list.evaluate_at(slot, actual[index], &mut probe)
        })
    }

    /// Evaluates a pair with diagnostics, recording its rejection into `group`.
    fn explain(&self, index: usize, slot: usize, group: &mut AssertionContext<'_, R>) {
        self.list.evaluate_at(slot, self.actual[index], group);
    }

    /// Describes a missing slot with the rejection of every element against it. An element known
    /// to satisfy the slot is assigned elsewhere and contributes no evidence.
    fn missing<C: ?Sized>(
        &self,
        slot: usize,
        order: RenderingOrder,
        context: &AssertionContext<'_, R>,
    ) -> AssertionFailure {
        let mut candidates = context.isolated_for_order(order);
        for index in 0..self.actual.len() {
            if self.cache.get(&(index, slot)) != Some(&true) {
                self.explain(index, slot, &mut candidates);
            }
        }
        let failure = FailureBuilder::new::<C>(FailureKind::Matching)
            .fact(Fact::labelled("At slot", context.render().value(&slot)))
            .relation("is missing an element matching this expectation")
            .constraint(self.list.describe_at(slot, context));
        candidates.into_evidence().explain(failure).build()
    }

    /// Describes every unexpected element. A surplus element satisfying an occupied slot is
    /// described through that slot. Any other element reports its rejection by every occupied
    /// slot. Rejections by missing slots belong to those slots.
    fn unexpected<C: ?Sized>(
        &mut self,
        result: &BipartiteMatchResult,
        order: RenderingOrder,
        context: &AssertionContext<'_, R>,
    ) -> AssertionFailure {
        let mut unexpected = context.isolated_for_order(order);
        let occupied = || result.matched_pairs.iter().map(|&(_, slot)| slot);
        for &index in &result.unmatched_actual {
            let satisfied = occupied()
                .filter(|&slot| self.matches(index, slot))
                .collect::<Vec<_>>();
            if satisfied.is_empty() {
                for slot in occupied() {
                    self.explain(index, slot, &mut unexpected);
                }
            } else {
                unexpected.record(|context| self.surplus(&satisfied, context));
            }
        }
        unexpected
            .into_evidence()
            .explain(
                FailureBuilder::new::<C>(FailureKind::Matching)
                    .relation("has unexpected elements")
                    .fact(Fact::labelled(
                        "Unexpected count",
                        context.render().value(&result.unmatched_actual.len()),
                    )),
            )
            .build()
    }

    /// Describes a surplus element through the occupied slots it satisfies.
    fn surplus(&self, satisfied: &[usize], context: &AssertionContext<'_, R>) -> AssertionFailure {
        if let &[slot] = satisfied {
            return FailureBuilder::new::<A>(FailureKind::Matching)
                .relation("has an extra occurrence matching an already satisfied expectation")
                .constraint(self.list.describe_at(slot, context))
                .fact(Fact::labelled("At slot", context.render().value(&slot)))
                .build();
        }
        let mut constraints = context.isolated();
        for &slot in satisfied {
            constraints.record(|constraints| {
                FailureBuilder::new::<A>(FailureKind::Matching)
                    .fact(Fact::labelled("At slot", constraints.render().value(&slot)))
                    .constraint(self.list.describe_at(slot, constraints))
                    .build()
            });
        }
        constraints
            .into_evidence()
            .explain(
                FailureBuilder::new::<A>(FailureKind::Matching)
                    .relation("has an extra occurrence matching already satisfied expectations"),
            )
            .build()
    }
}

/// Exact unordered matcher list with explicit expectations and duplicate preservation.
///
/// Use [`eq`](crate::matchers::eq) or [`eq`](crate::matchers::eq) for equality.
#[macro_export]
macro_rules! elements_are_in_any_order {
    ($($value:expr),* $(,)?) => {
        $crate::matchers::elements_are_in_any_order($crate::matchers![$($value),*])
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        assertions::core::partial_ord::ge,
        expectation::test_support::{assert_bounded_order, bounded_failures},
        matchers::eq,
        prelude::*,
    };

    #[test]
    fn bounded_candidate_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&elements_are_in_any_order![eq(9)]);
    }

    #[test]
    fn sorts_unexpected_evidence_before_limiting_it() {
        let matcher = elements_are_in_any_order![eq(9)];
        let expected = bounded_failures(&[9, 1, 2, 3], &matcher, 1);
        let actual = bounded_failures(&[9, 3, 2, 1], &matcher, 1);

        assert_that!(actual[0].children).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element.derive(|value| &value.children).has_length(1);
                element
                    .derive(|value| &value.omitted_children)
                    .is_equal_to(2);
            },
        ]);
        assert_that!(actual[0].to_string()).is_equal_to(expected[0].to_string());
    }

    #[test]
    fn supports_overlapping_constraints() {
        assert_that!([1, 2]).matches(elements_are_in_any_order![ge(1), eq(1)]);
    }

    mod evaluate {
        use super::*;
        use crate::matchers::{predicate, satisfying};
        use core::cell::{Cell, RefCell};
        use indoc::indoc;

        #[test]
        fn reports_rejected_and_surplus_occurrences() {
            let failures =
                bounded_failures(&[1, 1, 99], &elements_are_in_any_order![eq(1)], usize::MAX);
            assert_that!(failures[0]).has_text_report(indoc! {r"
                -------- assertr --------
                Expression: `actual`

                does not match

                Nested failures:
                  - has unexpected elements

                    Details:
                      - Unexpected count: 2
                    Nested failures:
                      - Expected: 1

                          Actual: 99

                      - has an extra occurrence matching an already satisfied expectation

                        Constraint:
                            is equal to

                            Expected: 1

                        Details:
                          - At slot: 0
                -------- assertr --------
            "});
        }

        #[test]
        fn reports_every_candidate_for_a_missing_expectation() {
            let failures = bounded_failures(
                &[1, 2],
                &elements_are_in_any_order![ge(0), eq(1), eq(99)],
                usize::MAX,
            );
            assert_that!(failures[0]).has_text_report(indoc! {r"
                -------- assertr --------
                Expression: `actual`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        is equal to

                        Expected: 99

                    Details:
                      - At slot: 2
                    Nested failures:
                      - Expected: 99

                          Actual: 1

                      - Expected: 99

                          Actual: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn candidate_failures_reuse_rendered_leaves_and_inherited_order() {
            use crate::renderer::{RenderedBody, RenderingOrder};
            struct Renderer(Cell<usize>);
            impl ValueRenderer<i32> for Renderer {
                fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.0.set(self.0.get() + 1);
                    write!(f, "custom-{value}")
                }
            }
            impl ValueRenderer<usize> for Renderer {
                fn fmt(
                    &self,
                    value: &usize,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "{value}")
                }
            }
            let renderer = Renderer(Cell::new(0));
            let budget = RenderingBudget::default()
                .with_max_items(1)
                .with_max_leaf_characters(7);
            let mut context = AssertionContext::new(&renderer, budget)
                .isolated_for_order(RenderingOrder::SortByRenderedText);
            assert_that!(
                context.evaluate(&[2, 1], &elements_are_in_any_order![ge(0), eq(1), eq(99)])
            )
            .is_false();
            // Two actual/expected comparisons and one expectation description. Retaining
            // their failures does not invoke the renderer again.
            assert_that!(renderer.0.get()).is_equal_to(5);
            let failures = context.into_evidence().children;
            assert_that!(failures[0].omitted_children).is_equal_to(1);
            assert_that!(failures[0].children).has_length(1);
            let candidate = failures[0].children[0].actual.as_ref().unwrap();
            assert_that!(candidate.type_name).is_equal_to(Some("i32"));
            let RenderedBody::Text {
                text,
                omitted_characters,
            } = &candidate.body
            else {
                panic!("missing the rendered candidate")
            };
            assert_that!(text).is_equal_to("custom-");
            assert_that!(*omitted_characters).is_equal_to(1);
        }

        mod description_fidelity {
            use super::*;
            use crate::{
                expectation::Expectation,
                failure::Fact,
                failure::{FailureBuilder, FailureKind},
            };

            #[test]
            fn formatting_constraints_remain_distinguishable_in_candidate_failures() {
                use crate::assertions::core::{debug::HasDebugString, display::HasDisplayValue};
                let debug = assert_that!([1]).with_location(false).capture(|it| {
                    it.matches(crate::assertions::collection::elements_are_in_any_order(
                        matchers![HasDebugString::new("2")],
                    ))
                });
                let display = assert_that!([1]).with_location(false).capture(|it| {
                    it.matches(crate::assertions::collection::elements_are_in_any_order(
                        matchers![HasDisplayValue::new("2")],
                    ))
                });
                for (failures, relation) in [
                    (&debug, "has the expected Debug representation"),
                    (&display, "has the expected Display representation"),
                ] {
                    let missing = &failures[0].children[0];
                    let constraint = missing.constraint.as_ref().unwrap();
                    assert_that!(constraint.relation.as_deref()).is_equal_to(Some(relation));
                    assert_that!(format!("{:#}", constraint.expected.as_ref().unwrap()))
                        .is_equal_to("\"2\"");
                    assert_that!(missing.children).has_length(1);
                    assert_that!(missing.facts).has_length(1);
                    assert_that!(missing.children[0].actual).is_some();
                    assert_that!(failures[0].to_string()).contains(relation);
                }
                assert_that!(debug[0].to_string()).is_not_equal_to(display[0].to_string());
            }

            struct EqualityDescription {
                relation: &'static str,
                detailed: bool,
            }
            impl Expectation<i32> for EqualityDescription {
                type Success<'a>
                    = ()
                where
                    Self: 'a,
                    i32: 'a;
                type Rejection<'a>
                    = ()
                where
                    Self: 'a,
                    i32: 'a;
                fn evaluate(&self, actual: &i32, _: &AssertionContext<'_>) -> Result<(), ()> {
                    if *actual == 99 { Ok(()) } else { Err(()) }
                }

                const KIND: FailureKind = FailureKind::Equality;
                fn explain<'a>(
                    &'a self,
                    rejected: Option<(&'a i32, ())>,
                    failure: FailureBuilder,
                    context: &AssertionContext<'_>,
                ) -> FailureBuilder {
                    let render = context.render();
                    let failure = failure.expected(render.value(&99));
                    if let Some((actual, ())) = rejected {
                        failure.actual(render.value(actual))
                    } else {
                        let failure = failure.relation(self.relation);
                        if self.detailed {
                            failure.fact(Fact::labelled("requirement", render.value(&"retained")))
                        } else {
                            failure
                        }
                    }
                }
            }

            #[test]
            fn retains_facts_on_a_missing_subject_description() {
                let matcher = elements_are_in_any_order![EqualityDescription {
                    relation: "is equal to",
                    detailed: true,
                }];
                let failures = assert_that!([1]).capture(|it| it.matches(matcher));
                let missing = &failures[0].children[0];
                let description = missing.constraint.as_ref().unwrap();
                assert_that!(description.relation.as_deref()).is_equal_to(Some("is equal to"));
                assert_that!(description.facts).has_length(1);
                assert_that!(description.facts[0].label)
                    .is_equal_to(alloc::borrow::Cow::Borrowed("requirement"));
                assert_that!(format!("{:#}", description.facts[0].value))
                    .is_equal_to("\"retained\"");
                assert_that!(missing.children).has_length(1);
                assert_that!(missing.children[0].kind).is_equal_to(FailureKind::Equality);
            }

            #[test]
            fn retains_candidate_failures_and_the_complete_description() {
                let matcher = elements_are_in_any_order![EqualityDescription {
                    relation: "equals the required value",
                    detailed: false,
                }];
                let failures = assert_that!([1]).capture(|it| it.matches(matcher));
                let missing = &failures[0].children[0];
                assert_that!(missing.constraint.as_ref().unwrap().relation.as_deref())
                    .is_equal_to(Some("equals the required value"));
                assert_that!(missing.expected).is_none();
                assert_that!(missing.children).has_length(1);
                assert_that!(missing.children[0].kind).is_equal_to(FailureKind::Equality);
                assert_that!(missing.facts).has_length(1);
            }
        }

        #[test]
        fn candidate_failures_preserve_field_rejection_paths() {
            use crate::{__private::field, assertions::core::partial_eq::eq, failure::PathSegment};
            struct Row {
                id: i32,
            }
            let matcher = field(|row: &Row| Some(&row.id), eq(99), PathSegment::Field("id"));
            let failures = assert_that!([Row { id: 1 }, Row { id: 2 }])
                .with_location(false)
                .capture(|it| it.matches(elements_are_in_any_order![matcher]));
            let missing = &failures[0].children[0];
            assert_that!(missing.constraint.is_some()).is_true();
            assert_that!(missing.facts).has_length(1);
            assert_that!(missing.children).has_length(2);
            for child in &missing.children {
                assert_that!(child.path).is_equal_to([PathSegment::Field("id")]);
            }
        }

        #[test]
        fn candidate_failures_count_all_omitted_rejections() {
            use crate::{matchers::all_of, test_support::UnorderedSet};
            let matcher = elements_are_in_any_order![all_of(matchers![
                eq(99),
                predicate(|value: &i32| *value != 2)
            ])];
            for values in [[1, 2], [2, 1]] {
                for prefix in [0, 1] {
                    let mut context = AssertionContext::new(
                        &DebugRenderer,
                        RenderingBudget::default().with_max_items(prefix + 1),
                    );
                    if prefix > 0 {
                        assert_that!(context.evaluate(&0, &eq(1))).is_false();
                    }
                    assert_that!(context.evaluate(&UnorderedSet(values.to_vec()), &matcher))
                        .is_false();
                    let failures = context.into_evidence().children;
                    let missing = &failures[prefix];
                    assert_that!(missing.constraint.is_some()).is_true();
                    assert_that!(missing.facts).has_length(1);
                    assert_that!(missing.children).has_length(1);
                    assert_that!(missing.omitted_children).is_equal_to(2);
                }
            }
        }

        #[test]
        fn completes_unexpected_evidence_before_sorting_and_limiting() {
            let matcher = elements_are_in_any_order![eq(1)];
            for limit in [0, 1, 2, usize::MAX] {
                let expected = bounded_failures(&[1, 1, 99], &matcher, limit);
                for values in [[1, 99, 1], [99, 1, 1]] {
                    let actual = bounded_failures(&values, &matcher, limit);
                    assert_that!(actual[0].to_string()).is_equal_to(expected[0].to_string());
                }
                if limit == 0 {
                    assert_that!(expected[0].children).is_empty();
                    assert_that!(expected[0].omitted_children).is_equal_to(1);
                } else {
                    let unexpected = &expected[0].children[0];
                    assert_that!(unexpected.children).has_length(limit.min(2));
                    assert_that!(unexpected.omitted_children).is_equal_to(2 - limit.min(2));
                    assert_that!(format!(
                        "{:#}",
                        unexpected.children[0].actual.as_ref().unwrap()
                    ))
                    .is_equal_to("99");
                }
            }
        }

        #[test]
        fn completes_missing_expectation_evidence_before_sorting_and_limiting() {
            let matcher = elements_are_in_any_order![ge(0), eq(1), eq(99)];
            for limit in [0, 1, 2, usize::MAX] {
                let expected = bounded_failures(&[1, 2], &matcher, limit);
                let actual = bounded_failures(&[2, 1], &matcher, limit);
                assert_that!(actual[0].to_string()).is_equal_to(expected[0].to_string());
                if limit == 0 {
                    assert_that!(expected[0].children).is_empty();
                    assert_that!(expected[0].omitted_children).is_equal_to(1);
                } else {
                    let missing = &expected[0].children[0];
                    assert_that!(missing.children).has_length(limit.min(2));
                    assert_that!(missing.omitted_children).is_equal_to(2 - limit.min(2));
                    assert_that!(format!(
                        "{:#}",
                        missing.children[0].actual.as_ref().unwrap()
                    ))
                    .is_equal_to("1");
                }
            }
        }

        #[test]
        fn preserves_surplus_multiplicity_and_all_occupied_constraints_within_budget() {
            let matcher = elements_are_in_any_order![ge(0), eq(1)];
            for limit in [1, 2, usize::MAX] {
                let failures = bounded_failures(&[1, 1, 1, 1], &matcher, limit);
                let unexpected = &failures[0].children[0];
                assert_that!(format!("{:#}", unexpected.facts[0].value)).is_equal_to("2");
                assert_that!(unexpected.children).has_length(limit.min(2));
                assert_that!(unexpected.omitted_children).is_equal_to(2 - limit.min(2));
                for surplus in &unexpected.children {
                    assert_that!(surplus.relation.as_deref()).is_equal_to(Some(
                        "has an extra occurrence matching already satisfied expectations",
                    ));
                    assert_that!(surplus.children).has_length(limit.min(2));
                    assert_that!(surplus.omitted_children).is_equal_to(2 - limit.min(2));
                    let slots = surplus
                        .children
                        .iter()
                        .map(|child| format!("{:#}", child.facts[0].value))
                        .collect::<Vec<_>>();
                    if limit >= 2 {
                        assert_that!(slots).contains_exactly_in_any_order(["0", "1"]);
                    }
                    assert_that!(
                        surplus.children[0]
                            .constraint
                            .as_ref()
                            .unwrap()
                            .relation
                            .as_deref()
                    )
                    .is_equal_to(Some("is equal to"));
                }
            }
        }

        #[test]
        fn explains_surplus_through_constraints_without_an_element_renderer() {
            struct Element;
            struct CountRenderer;
            impl ValueRenderer<usize> for CountRenderer {
                fn fmt(
                    &self,
                    value: &usize,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "count({value})")
                }
            }
            let failures = assert_that!([Element, Element, Element])
                .with_renderer(CountRenderer)
                .capture(|it| {
                    it.matches(elements_are_in_any_order![
                        crate::test_support::opaque_predicate(|_: &Element| true),
                        crate::test_support::opaque_predicate(|_: &Element| false)
                    ])
                });
            let missing = &failures[0].children[0];
            assert_that!(format!("{:#}", missing.facts[0].value)).is_equal_to("count(1)");
            let unexpected = &failures[0].children[1];
            assert_that!(format!("{:#}", unexpected.facts[0].value)).is_equal_to("count(2)");
            assert_that!(unexpected.children).has_length(2);
            for surplus in &unexpected.children {
                assert_that!(surplus.children).is_empty();
                assert_that!(format!("{:#}", surplus.facts[0].value)).is_equal_to("count(0)");
                assert_that!(surplus.facts[0].label.as_ref()).is_equal_to("At slot");
                assert_that!(
                    surplus
                        .constraint
                        .as_ref()
                        .unwrap()
                        .relation
                        .as_deref()
                        .unwrap()
                )
                .is_equal_to("satisfies the opaque predicate");
            }
        }

        #[test]
        fn explains_both_unmatched_sides_by_evaluating_their_pairs_again() {
            let calls = RefCell::new([[0; 2]; 2]);
            let calls_ref = &calls;
            let list = [1, 2]
                .into_iter()
                .enumerate()
                .map(|(slot, expected)| {
                    satisfying(move |it: AssertThat<'_, (usize, i32), Capture>| {
                        calls_ref.borrow_mut()[it.actual().0][slot] += 1;
                        it.derive(|actual| &actual.1).is_equal_to(expected);
                    })
                })
                .collect::<Vec<_>>();
            let failures = assert_that!([(0, 1), (1, 99)]).capture(|it| {
                it.matches(crate::assertions::collection::elements_are_in_any_order(
                    list,
                ))
            });
            // The search probes three pairs. Explaining the missing expectation and the unexpected
            // element evaluates the rejected pairs again.
            assert_that!(*calls.borrow()).is_equal_to([[1, 1], [2, 2]]);
            assert_that!(failures[0].children).has_length(2);
            assert_that!(failures[0].children[0].children).has_length(2);
            assert_that!(failures[0].children[1].children).has_length(1);
        }

        #[test]
        fn surplus_groups_preserve_the_enclosing_path_once() {
            use crate::failure::PathSegment;
            let mut context = AssertionContext::default();
            let result = context.scoped(PathSegment::Field("items"), |context| {
                context.evaluate(&[1, 1], &elements_are_in_any_order![eq(1)])
            });
            assert_that!(result).is_false();
            let failures = context.into_evidence().children;
            assert_that!(failures[0].path).is_equal_to([PathSegment::Field("items")]);
            let surplus = &failures[0].children[0];
            assert_that!(surplus.path).is_empty();
            assert_that!(surplus.constraint.is_some()).is_true();
        }

        #[test]
        fn empty_expectations_report_the_unexpected_count() {
            let failures = bounded_failures(&[1, 2], &elements_are_in_any_order![], usize::MAX);
            let unexpected = &failures[0].children[0];
            assert_that!(unexpected.relation.as_deref())
                .is_equal_to(Some("has unexpected elements"));
            assert_that!(format!("{:#}", unexpected.facts[0].value)).is_equal_to("2");
            assert_that!(unexpected.children).is_empty();
            assert_that!(unexpected.omitted_children).is_equal_to(0);
        }

        #[test]
        fn only_explains_pairs_when_diagnostics_can_retain_evidence() {
            for (limit, expected_calls) in [(0, [1, 1, 0]), (usize::MAX, [1, 1, 2])] {
                let calls = RefCell::new([0; 3]);
                let matcher = elements_are_in_any_order![predicate(|index: &usize| {
                    calls.borrow_mut()[*index] += 1;
                    *index < 2
                })];
                let mut context = AssertionContext::new(
                    &DebugRenderer,
                    RenderingBudget::default().with_max_items(limit),
                );
                assert_that!(context.evaluate(&[0, 1, 2], &matcher)).is_false();
                assert_that!(*calls.borrow()).is_equal_to(expected_calls);
                if limit == 0 {
                    assert_that!(context.into_evidence().children).is_empty();
                }
            }
        }

        #[test]
        fn exact_matches_keep_sparse_evaluation() {
            let calls = Cell::new(0);
            let matcher = predicate(|_: &i32| {
                calls.set(calls.get() + 1);
                true
            });
            let matcher = elements_are_in_any_order![&matcher, &matcher];
            let mut context = AssertionContext::default();
            assert_that!(context.evaluate(&[1, 1], &matcher)).is_true();
            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(context.into_evidence().children).is_empty();
        }

        #[test]
        fn a_zero_item_budget_does_not_render_completed_or_surplus_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("rendered omitted evidence")
                }
            }
            let mut context =
                AssertionContext::new(&NeverRender, RenderingBudget::default().with_max_items(0));
            assert_that!(context.evaluate(&[1, 1, 99], &elements_are_in_any_order![eq(1)]))
                .is_false();
            let evidence = context.into_evidence();
            assert_that!(evidence.omitted).is_equal_to(1);
            assert_that!(evidence.children).is_empty();
        }

        #[test]
        fn a_zero_item_budget_preserves_length_failure_without_rendering_evidence() {
            use indoc::formatdoc;
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("rendered omitted evidence")
                }
            }
            let failures = assert_that!([1, 2])
                .with_renderer(NeverRender)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.matches(elements_are_in_any_order![]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
        -------- assertr --------
        Expression: `[1, 2]`

        does not match

        Details:
          - ... 1 more nested failure ...
        -------- assertr --------
    "});
                    element
                        .derive(|value| &value.omitted_children)
                        .is_equal_to(1);
                },
            ]);
        }

        fn matrix_matchers<'a, const M: usize>(
            matrix: &'a [[bool; M]],
            calls: &'a RefCell<Vec<(usize, usize)>>,
        ) -> impl matchers::MatcherList<usize, DebugRenderer> + 'a {
            use crate::matchers::{all_of, predicate};
            (0..M)
                .map(|slot| {
                    all_of(matchers![
                        predicate(move |index: &usize| {
                            calls.borrow_mut().push((*index, slot));
                            matrix[*index][slot]
                        }),
                        predicate(move |index: &usize| matrix[*index][slot])
                    ])
                })
                .collect::<Vec<_>>()
        }

        #[test]
        fn the_search_probes_each_pair_once_and_diagnostics_follow_it() {
            for n in [2, 3] {
                // Exhaust small relations, including exact matches, duplicates and both unmatched
                // sides.
                for bits in 0..(1 << (n * 2)) {
                    let matrix: Vec<_> = (0..n)
                        .map(|index| {
                            core::array::from_fn::<_, 2, _>(|slot| {
                                bits & (1 << (index * 2 + slot)) != 0
                            })
                        })
                        .collect();
                    for limit in [0, 1, 2, usize::MAX] {
                        let mut search = Vec::new();
                        let mut seen = alloc::collections::BTreeSet::new();
                        let result = crate::util::matching::match_bipartite(n, 2, |index, slot| {
                            if seen.insert((index, slot)) {
                                search.push((index, slot));
                            }
                            matrix[index][slot]
                        });
                        let calls = RefCell::new(Vec::new());
                        let matcher = crate::assertions::collection::elements_are_in_any_order(
                            matrix_matchers(&matrix, &calls),
                        );
                        let mut context = AssertionContext::new(
                            &DebugRenderer,
                            RenderingBudget::unlimited().with_max_items(limit),
                        );
                        assert_that!(context.evaluate(&(0..n).collect::<Vec<_>>(), &matcher))
                            .is_equal_to(result.is_exact());
                        let calls = calls.borrow();
                        assert_that!(&calls[..search.len()]).is_equal_to(search.as_slice());
                        if limit == 0 || result.is_exact() {
                            assert_that!(calls.len()).is_equal_to(search.len());
                        }
                    }
                }
            }
        }
    }

    mod probe {
        use super::*;

        #[test]
        fn surplus_search_does_not_complete_unvisited_pairs_or_render() {
            use crate::{assertions::core::partial_eq::eq, matchers::predicate};
            use core::cell::RefCell;
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let calls = RefCell::new([0; 3]);
            let matcher = elements_are_in_any_order![predicate(|index: &usize| {
                calls.borrow_mut()[*index] += 1;
                *index < 2
            })];
            let context = AssertionContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[0, 1, 2], &matcher)).is_false();
            // Unequal lengths decide a probe before any comparison.
            assert_that!(*calls.borrow()).is_equal_to([0, 0, 0]);
            assert_that!(context.probe(&[1, 1, 99], &elements_are_in_any_order![eq(1)])).is_false();
            let evidence = context.into_evidence();
            assert_that!(evidence.omitted).is_equal_to(0);
            assert_that!(evidence.children).is_empty();
        }

        #[test]
        fn stops_at_the_first_unassignable_occurrence() {
            use crate::matchers::predicate_list;
            use core::cell::Cell;
            let calls = Cell::new(0);
            let counted = |expected: usize| {
                let calls = &calls;
                move |actual: &usize| {
                    calls.set(calls.get() + 1);
                    *actual == expected
                }
            };
            let matcher =
                super::super::elements_are_in_any_order(predicate_list([counted(1), counted(2)]));
            let context = AssertionContext::default();

            assert_that!(context.probe(&[0, 1], &matcher)).is_false();
            assert_that!(calls.get()).is_equal_to(2);

            calls.set(0);
            assert_that!(context.probe(&[1, 2, 3], &matcher)).is_false();
            assert_that!(calls.get()).is_equal_to(0);
        }

        #[test]
        fn length_mismatches_do_not_render_numeric_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = AssertionContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are_in_any_order![])).is_false();
            assert_that!(context.into_evidence().children).is_empty();
        }
    }
}
