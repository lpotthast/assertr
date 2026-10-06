//! Exact unordered assignment and its diagnostic evidence.
//!
//! The sparse cache stores each pair's boolean result and total direct failure count, never failure
//! evidence or borrowed observations. Cache misses evaluate and explain eligible rejections
//! immediately, releasing guards before another pair runs. Hits never repeat evaluation,
//! explanation, or sample offers. Search and completion keep their existing order. Passing matches,
//! probes, and zero allowances skip completion.
//!
//! Shared candidate sampling and destination ownership live in `expectation::assignment`. This
//! assertion supplies the pair cache, drives search and diagnostic completion, and then assembles
//! missing constraints, unexpected groups, and surplus descriptions. Assembly starts after
//! completion so retained failures cannot change later pair budgets or explanation eligibility.

use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, Fact,
    assertions::collection::Collection,
    expectation::{
        Evidence, MatcherList,
        assignment::{AssignedEvidence, CandidateSamples},
    },
    failure::{FailureBuilder, FailureKind},
    renderer::IntoRendered,
    util::matching::{BipartiteMatchResult, match_bipartite, matches_exactly},
};
use alloc::{collections::BTreeMap, vec::Vec};

/// The sparse cache owns scalar outcomes only, never failure evidence, report keys, or
/// observations.
struct PairOutcome {
    matched: bool,
    failures: usize,
}

fn evaluate_pair(
    cache: &mut BTreeMap<(usize, usize), PairOutcome>,
    pair: (usize, usize),
    evaluate: impl FnOnce() -> (bool, Evidence),
    retain: impl FnOnce(Evidence),
) -> bool {
    cache
        .entry(pair)
        .or_insert_with(|| {
            let (matched, evidence) = evaluate();
            let failures = if matched {
                0
            } else {
                evidence.children.len() + evidence.omitted
            };
            if !matched {
                retain(evidence);
            }
            PairOutcome { matched, failures }
        })
        .matched
}

/// Exact one-to-one order-free matching, preserving multiplicity.
#[derive(Debug, Clone)]
pub struct ElementsAreInAnyOrder<L>(L);

/// Matches every actual element to one distinct expectation using maximum bipartite matching.
///
/// Each actual/expectation pair is evaluated at most once. Requested rejection diagnostics are
/// built immediately from that observation.
/// A mismatch completes unvisited comparisons involving unmatched elements or expectations. Probes
/// and zero evidence allowances skip completion. Probes also reject unequal lengths without
/// comparisons and stop at the first occurrence that cannot be assigned. Surplus occurrences that
/// satisfy occupied expectations use those expectations' descriptions, without requiring an element
/// renderer.
///
/// During assignment, each occurrence and expectation slot samples at most the inherited item
/// allowance of direct child failures. Each sampled item is one owned
/// [`AssertionFailure`](crate::AssertionFailure), including its rendered values and any constraint
/// description or nested child failures. Those nested failures do not take additional sample
/// positions.
///
/// The union of the samples is routed after assignment, and completed comparisons feed their final
/// groups directly. Missing expectations own their rejections exclusively. Finite samples may
/// remain underfilled when assignment disqualifies retained children and eligible replacements were
/// discarded. Omission counts include those lost
/// candidates. Unlimited budgets retain all evidence. Budgets bound neither comparison work nor
/// total memory, and the scalar comparison cache can require quadratic space.
///
/// The `at slot` fact identifies a zero-based expectation position, never an actual collection
/// index.
pub fn elements_are_in_any_order<L>(list: L) -> ElementsAreInAnyOrder<L> {
    ElementsAreInAnyOrder(list)
}

impl<C: Collection + ?Sized, R, L> Expectation<C, R> for ElementsAreInAnyOrder<L>
where
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        C: 'a;
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let actual = actual.elements().collect::<Vec<_>>();
        let expected_length = self.0.len();
        // A sparse cache avoids allocating the full Cartesian product for easy exact matches.
        let mut cache = BTreeMap::new();
        let branch_settings = context.isolated_for_order(C::PRESENTATION.order());
        let (limit, order) = branch_settings.evidence_policy();
        let mut samples = CandidateSamples::new(actual.len(), expected_length, limit, order);
        let evaluate = |index: usize, slot| {
            let mut branch = branch_settings.isolated();
            let matched = self.0.evaluate_at(slot, actual[index], &mut branch);
            (matched, branch.into_evidence())
        };
        if context.is_probe() {
            // A probe discards evidence. Unequal lengths and the first unassignable occurrence
            // already decide the outcome, so it neither completes pairs nor assembles groups.
            let accepted = matches_exactly(actual.len(), expected_length, |index, slot| {
                evaluate_pair(&mut cache, (index, slot), || evaluate(index, slot), drop)
            });
            return if accepted {
                Ok(())
            } else {
                Err(context.into_evidence())
            };
        }
        let result = match_bipartite(actual.len(), expected_length, |index, slot| {
            evaluate_pair(
                &mut cache,
                (index, slot),
                || evaluate(index, slot),
                |evidence| {
                    samples.offer((index, slot), evidence);
                },
            )
        });
        if result.is_exact() {
            return Ok(());
        }
        let mut destinations = samples.resolve(&result);
        if context.is_diagnostic() {
            complete_unmatched_pairs(&result, actual.len(), expected_length, |index, slot| {
                evaluate_pair(
                    &mut cache,
                    (index, slot),
                    || evaluate(index, slot),
                    |evidence| {
                        destinations.offer((index, slot), evidence);
                    },
                )
            });
        }
        for &slot in &result.unmatched_expected {
            let total = (0..actual.len())
                .filter_map(|index| cache.get(&(index, slot)))
                .map(|outcome| outcome.failures)
                .sum();
            record_missing::<C, _, _>(
                &self.0,
                slot,
                destinations.take_missing(slot, total),
                &mut context,
            );
        }
        record_unexpected::<C, _, _>(&self.0, &result, &cache, &mut destinations, &mut context);
        Err(context.into_evidence())
    }
}
impl<C: Collection + ?Sized, R, L> ExpectationDiagnostics<C, R> for ElementsAreInAnyOrder<L>
where
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => context.describe_list::<C::Item, _, _>(
                &self.0,
                failure.relation("has exactly these elements in any order"),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

fn record_unexpected<
    C: Collection + ?Sized,
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
>(
    list: &L,
    result: &BipartiteMatchResult,
    cache: &BTreeMap<(usize, usize), PairOutcome>,
    destinations: &mut AssignedEvidence,
    context: &mut AssertionContext<'_, R>,
) {
    if !result.unmatched_actual.is_empty() {
        let mut unexpected = context.isolated_for_order(C::PRESENTATION.order());
        for index in &result.unmatched_actual {
            let occupied = result.matched_pairs.iter().filter_map(|&(_, slot)| {
                cache
                    .get(&(*index, slot))
                    .and_then(|outcome| outcome.matched.then_some(slot))
            });
            if record_surplus::<C::Item, _, _>(list, occupied, &mut unexpected) {
                destinations.discard_surplus(*index);
                continue;
            }
            let total = (0..list.len())
                .filter(|slot| result.unmatched_expected.binary_search(slot).is_err())
                .filter_map(|slot| cache.get(&(*index, slot)))
                .map(|outcome| outcome.failures)
                .sum();
            unexpected.append(destinations.take_unexpected(*index, total));
        }
        context.record_with(|context| {
            unexpected
                .into_evidence()
                .explain(
                    FailureBuilder::detached::<C>(FailureKind::Matching)
                        .relation("has unexpected elements")
                        .fact(Fact::labelled(
                            "Unexpected count",
                            context
                                .render()
                                .value(&result.unmatched_actual.len())
                                .into_rendered(),
                        )),
                )
                .build()
        });
    }
}

/// Retains the complete missing-subject constraint and bounded candidate rejections.
fn record_missing<
    C: Collection + ?Sized,
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
>(
    list: &L,
    slot: usize,
    rejections: Evidence,
    context: &mut AssertionContext<'_, R>,
) {
    let mut alternatives = context.isolated_for_order(C::PRESENTATION.order());
    alternatives.append(rejections);
    context.record_with(|context| {
        let failure = FailureBuilder::detached::<C>(FailureKind::Matching)
            .fact(Fact::labelled(
                "At slot",
                context.render().value(&slot).into_rendered(),
            ))
            .relation("is missing an element matching this expectation")
            .constraint(list.describe_at(slot, context));
        alternatives.into_evidence().explain(failure).build()
    });
}

/// Search marks can skip pairs needed for evidence. Complete both unmatched sides before consuming
/// cached evidence, so the caller's cached evaluator never replays a pair.
fn complete_unmatched_pairs(
    result: &BipartiteMatchResult,
    actual_length: usize,
    expected_length: usize,
    mut evaluate_pair: impl FnMut(usize, usize) -> bool,
) {
    for &index in &result.unmatched_actual {
        for slot in 0..expected_length {
            evaluate_pair(index, slot);
        }
    }
    for &slot in &result.unmatched_expected {
        for index in 0..actual_length {
            evaluate_pair(index, slot);
        }
    }
}

/// Explains one surplus occurrence using cached truth and independently described constraints.
/// Returns false when the occurrence satisfies none of the occupied expectations.
fn record_surplus<A: ?Sized, R, L>(
    list: &L,
    occupied: impl Iterator<Item = usize>,
    context: &mut AssertionContext<'_, R>,
) -> bool
where
    R: crate::ValueRenderer<usize>,
    L: MatcherList<A, R>,
{
    let mut occupied = occupied.peekable();
    let Some(first) = occupied.next() else {
        return false;
    };
    context.record_with(|context| {
        if occupied.peek().is_none() {
            return FailureBuilder::detached::<A>(FailureKind::Matching)
                .relation("has an extra occurrence matching an already satisfied expectation")
                .constraint(list.describe_at(first, context))
                .fact(Fact::labelled(
                    "At slot",
                    context.render().value(&first).into_rendered(),
                ))
                .build();
        }
        let mut constraints = context.isolated();
        for slot in core::iter::once(first).chain(occupied) {
            constraints.record_with(|constraints| {
                FailureBuilder::detached::<A>(FailureKind::Matching)
                    .fact(Fact::labelled(
                        "At slot",
                        constraints.render().value(&slot).into_rendered(),
                    ))
                    .constraint(list.describe_at(slot, constraints))
                    .build()
            });
        }
        constraints
            .into_evidence()
            .explain(
                FailureBuilder::detached::<A>(FailureKind::Matching)
                    .relation("has an extra occurrence matching already satisfied expectations"),
            )
            .build()
    });
    true
}

/// Exact unordered matcher list with explicit expectations and duplicate preservation.
///
/// Use [`eq`](crate::matchers::eq) or [`equal_to`](crate::matchers::equal_to) for equality.
#[macro_export]
macro_rules! elements_are_in_any_order {
    ($($value:expr),* $(,)?) => {
        $crate::assertions::collection::elements_are_in_any_order($crate::matchers![$($value),*])
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
        assert_that!(ToHumanReadableText.render(&actual[0]))
            .is_equal_to(ToHumanReadableText.render(&expected[0]));
    }

    #[test]
    fn supports_overlapping_constraints() {
        assert_that!([1, 2]).matches(elements_are_in_any_order![ge(1), eq(1)]);
    }

    mod evaluate {
        use super::*;
        use crate::expectation::{ExpectationDiagnostics, predicate, satisfying};
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
                Expectation, Fact,
                failure::{FailureBuilder, FailureKind},
                renderer::IntoRendered,
            };

            #[test]
            fn formatting_constraints_remain_distinguishable_in_candidate_failures() {
                use crate::assertions::core::{debug::HasDebugString, display::HasDisplayValue};
                let debug = assert_that!([1]).with_location(false).capture(|it| {
                    it.matches(crate::assertions::collection::elements_are_in_any_order((
                        HasDebugString::new("2"),
                    )))
                });
                let display = assert_that!([1]).with_location(false).capture(|it| {
                    it.matches(crate::assertions::collection::elements_are_in_any_order((
                        HasDisplayValue::new("2"),
                    )))
                });
                for (failures, relation) in [
                    (&debug, "has the expected Debug representation"),
                    (&display, "has the expected Display representation"),
                ] {
                    let missing = &failures[0].children[0];
                    let constraint = missing.constraint.as_ref().unwrap();
                    assert_that!(constraint.relation.as_deref()).is_equal_to(Some(relation));
                    assert_that!(rendered_text(constraint.expected.as_ref().unwrap()))
                        .is_equal_to("\"2\"");
                    assert_that!(missing.children).has_length(1);
                    assert_that!(missing.facts).has_length(1);
                    assert_that!(missing.children[0].actual).is_some();
                    assert_that!(ToHumanReadableText.render(&failures[0])).contains(relation);
                }
                assert_that!(ToHumanReadableText.render(&debug[0]))
                    .is_not_equal_to(ToHumanReadableText.render(&display[0]));
            }

            struct EqualityDescription {
                relation: &'static str,
                detailed: bool,
            }
            impl Expectation<i32> for EqualityDescription {
                type Success<'a> = ();
                type Rejection<'a> = ();
                fn evaluate(&self, actual: &i32, _: &AssertionContext<'_>) -> Result<(), ()> {
                    if *actual == 99 { Ok(()) } else { Err(()) }
                }
            }
            impl ExpectationDiagnostics<i32> for EqualityDescription {
                const KIND: FailureKind = FailureKind::Equality;
                fn explain<'a, Target>(
                    &'a self,
                    rejected: Option<(&'a i32, ())>,
                    failure: FailureBuilder<Target>,
                    context: &AssertionContext<'_>,
                ) -> FailureBuilder<Target> {
                    let render = context.render();
                    let failure = failure.expected(render.value(&99));
                    if let Some((actual, ())) = rejected {
                        failure.actual(render.value(actual))
                    } else {
                        let failure = failure.relation(self.relation);
                        if self.detailed {
                            failure.fact(Fact::labelled(
                                "requirement",
                                render.value(&"retained").into_rendered(),
                            ))
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
                assert_that!(rendered_text(&description.facts[0].value))
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
            use crate::{
                __private::field::field, assertions::core::partial_eq::equal_to,
                failure::PathSegment,
            };
            struct Row {
                id: i32,
            }
            let matcher = field(
                |row: &Row| Some(&row.id),
                equal_to(99),
                PathSegment::Field("id"),
            );
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
            use crate::{expectation::all_of, test_support::UnorderedSet};
            let matcher =
                elements_are_in_any_order![all_of((eq(99), predicate(|value: &i32| *value != 2)))];
            for values in [[1, 2], [2, 1]] {
                for prefix in [0, 1] {
                    let mut context = AssertionContext::new(
                        &DebugRenderer,
                        RenderingBudget::default().with_max_items(prefix + 1),
                    );
                    if prefix > 0 {
                        context.evaluate(&0, &eq(1));
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
                    assert_that!(ToHumanReadableText.render(&actual[0]))
                        .is_equal_to(ToHumanReadableText.render(&expected[0]));
                }
                if limit == 0 {
                    assert_that!(expected[0].children).is_empty();
                    assert_that!(expected[0].omitted_children).is_equal_to(1);
                } else {
                    let unexpected = &expected[0].children[0];
                    assert_that!(unexpected.children).has_length(limit.min(2));
                    assert_that!(unexpected.omitted_children).is_equal_to(2 - limit.min(2));
                    assert_that!(rendered_text(
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
                assert_that!(ToHumanReadableText.render(&actual[0]))
                    .is_equal_to(ToHumanReadableText.render(&expected[0]));
                if limit == 0 {
                    assert_that!(expected[0].children).is_empty();
                    assert_that!(expected[0].omitted_children).is_equal_to(1);
                } else {
                    let missing = &expected[0].children[0];
                    assert_that!(missing.children).has_length(limit.min(2));
                    assert_that!(missing.omitted_children).is_equal_to(2 - limit.min(2));
                    assert_that!(rendered_text(missing.children[0].actual.as_ref().unwrap()))
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
                assert_that!(rendered_text(&unexpected.facts[0].value)).is_equal_to("2");
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
                        .map(|child| rendered_text(&child.facts[0].value))
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
            assert_that!(rendered_text(&missing.facts[0].value)).is_equal_to("count(1)");
            let unexpected = &failures[0].children[1];
            assert_that!(rendered_text(&unexpected.facts[0].value)).is_equal_to("count(2)");
            assert_that!(unexpected.children).has_length(2);
            for surplus in &unexpected.children {
                assert_that!(surplus.children).is_empty();
                assert_that!(rendered_text(&surplus.facts[0].value)).is_equal_to("count(0)");
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
        fn completes_both_unmatched_sides_without_replaying_pairs() {
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
            assert_that!(*calls.borrow()).is_equal_to([[1, 1], [1, 1]]);
            // Rejections against the missing expectation are retained there first.
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
            assert_that!(rendered_text(&unexpected.facts[0].value)).is_equal_to("2");
            assert_that!(unexpected.children).is_empty();
            assert_that!(unexpected.omitted_children).is_equal_to(0);
        }

        #[test]
        fn only_completes_pairs_when_diagnostics_can_retain_evidence() {
            for (limit, expected_calls) in [(0, [1, 1, 0]), (usize::MAX, [1, 1, 1])] {
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
            assert_that!(context.omitted).is_equal_to(1);
            assert_that!(context.into_evidence().children).is_empty();
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
        ) -> impl crate::expectation::MatcherList<usize, DebugRenderer> + 'a {
            use crate::expectation::{all_of, predicate};
            (0..M)
                .map(|slot| {
                    all_of((
                        predicate(move |index: &usize| {
                            calls.borrow_mut().push((*index, slot));
                            matrix[*index][slot]
                        }),
                        predicate(move |index: &usize| matrix[*index][slot]),
                    ))
                })
                .collect::<Vec<_>>()
        }

        #[test]
        fn assignment_can_starve_an_unexpected_sample_without_replaying_discarded_pairs() {
            use crate::renderer::RenderingOrder;
            let matrix = [
                [false, false, true],
                [false, true, false],
                [false, false, false],
                [false, true, false],
            ];
            for order in [
                RenderingOrder::PreserveIteration,
                RenderingOrder::SortByRenderedText,
            ] {
                for (limit, missing_count, unexpected_count, omitted) in
                    [(2, 2, 1, 4), (usize::MAX, 8, 5, 0)]
                {
                    let calls = RefCell::new(Vec::new());
                    let matcher = crate::assertions::collection::elements_are_in_any_order(
                        matrix_matchers(&matrix, &calls),
                    );
                    let mut context = AssertionContext::new(
                        &DebugRenderer,
                        RenderingBudget::unlimited().with_max_items(limit),
                    )
                    .isolated_for_order(order);
                    assert_that!(context.evaluate(&[0, 1, 2, 3], &matcher)).is_false();
                    assert_that!(*calls.borrow()).is_equal_to([
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (1, 0),
                        (1, 1),
                        (2, 0),
                        (2, 1),
                        (2, 2),
                        (3, 0),
                        (3, 1),
                        (1, 2),
                        (3, 2),
                    ]);
                    let failures = context.into_evidence().children;
                    let missing = failures
                        .iter()
                        .find(|failure| failure.constraint.is_some())
                        .unwrap();
                    assert_that!(missing.children).has_length(missing_count);
                    assert_that!(missing.omitted_children).is_equal_to(8 - missing_count);
                    let unexpected = failures
                        .iter()
                        .find(|failure| failure.constraint.is_none())
                        .unwrap();
                    // Every pair was visited. Four eligible children of occurrence 2 have been
                    // discarded, so k = 2 retains only the surplus description for occurrence 3.
                    assert_that!(unexpected.children).has_length(unexpected_count);
                    assert_that!(unexpected.omitted_children).is_equal_to(omitted);
                }
            }
        }

        #[test]
        fn truth_and_pair_traces_match_the_original_search_and_completion_across_budgets() {
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
                        let mut reference = Vec::new();
                        let mut seen = alloc::collections::BTreeSet::new();
                        let mut evaluate = |index: usize, slot: usize| {
                            if seen.insert((index, slot)) {
                                reference.push((index, slot));
                            }
                            matrix[index][slot]
                        };
                        let result = crate::util::matching::match_bipartite(n, 2, &mut evaluate);
                        if limit > 0 && !result.is_exact() {
                            super::super::complete_unmatched_pairs(&result, n, 2, evaluate);
                        }
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
                        assert_that!(*calls.borrow()).is_equal_to(reference);
                    }
                }
            }
        }
    }

    mod probe {
        use super::*;

        #[test]
        fn surplus_search_does_not_complete_unvisited_pairs_or_render() {
            use crate::{assertions::core::partial_eq::equal_to, expectation::predicate};
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
            assert_that!(context.probe(&[1, 1, 99], &elements_are_in_any_order![equal_to(1)]))
                .is_false();
            assert_that!(context.omitted).is_equal_to(0);
            assert_that!(context.into_evidence().children).is_empty();
        }

        #[test]
        fn stops_at_the_first_unassignable_occurrence() {
            use crate::expectation::predicate_list;
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
