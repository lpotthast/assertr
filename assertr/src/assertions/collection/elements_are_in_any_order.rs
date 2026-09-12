//! Exact unordered assignment and its diagnostic evidence.
//!
//! The sparse candidate cache stores each pair's boolean result and owned evidence. It retains
//! no evaluation context or borrowed observation. A failed assignment completes unvisited pairs
//! involving unmatched actual elements or expected slots before sorting and truncating evidence.
//! This prevents search pruning from hiding diagnostic candidates. Passing matches, probes, and
//! zero-item budgets skip completion. The result is unchanged, while candidate work and storage
//! can approach the Cartesian product.
//!
//! Missing slots consume candidate rejections first and retain the complete missing-subject
//! constraint. Unexpected occurrences consume remaining rejections. A surplus occurrence that
//! satisfies an occupied expectation uses that expectation's description. Separate occurrence
//! groups preserve multiplicity without inventing actual collection indexes.

use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, Fact,
    assertions::collection::Collection,
    expectation::{Evidence, MatcherList},
    failure::{FailureBuilder, FailureKind},
    renderer::IntoRendered,
    util::matching::{BipartiteMatchResult, match_bipartite},
};
use alloc::{collections::BTreeMap, vec::Vec};

/// Exact one-to-one order-free matching, preserving multiplicity.
pub struct ElementsAreInAnyOrder<L>(L);

/// Matches every actual element to one distinct expectation using maximum bipartite matching.
///
/// Each actual/expectation pair is evaluated at most once. When explaining a mismatch, comparisons
/// involving unmatched elements or expectations are completed before evidence is sorted and
/// limited. Surplus occurrences that satisfy occupied expectations are explained through those
/// expectations' descriptions, without requiring an element renderer. Probes and zero-item budgets
/// skip diagnostic completion. Cached candidate evidence can require quadratic space.
///
/// Candidate rejections retain their complete nested failures. The `at slot` fact identifies a
/// zero-based expectation position, never an actual collection index.
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
        let matched = {
            let context = &mut context;
            let actual = actual.elements().collect::<Vec<_>>();
            let expected_length = self.0.len();
            // A sparse cache avoids allocating the full Cartesian product for easy exact matches.
            let mut cache = BTreeMap::new();
            let mut evaluate_pair = |index: usize, slot| {
                cache
                    .entry((index, slot))
                    .or_insert_with(|| {
                        let mut branch = context.isolated_for_order(C::PRESENTATION.order());
                        let matched = self.0.evaluate_at(slot, actual[index], &mut branch);
                        (matched, branch.into_evidence())
                    })
                    .0
            };
            let result = match_bipartite(actual.len(), expected_length, &mut evaluate_pair);
            let matched = result.is_exact();
            if !matched && context.is_diagnostic() {
                complete_unmatched_pairs(&result, actual.len(), expected_length, evaluate_pair);
            }
            if !matched {
                for slot in result.unmatched_expected {
                    let rejections = (0..actual.len()).filter_map(|index| {
                        cache
                            .remove(&(index, slot))
                            .and_then(|(matched, branch)| (!matched).then_some(branch))
                    });
                    record_missing::<C, _, _>(&self.0, slot, rejections, context);
                }
                if !result.unmatched_actual.is_empty() {
                    let mut unexpected = context.isolated_for_order(C::PRESENTATION.order());
                    for index in &result.unmatched_actual {
                        let occupied = result.matched_pairs.iter().filter_map(|&(_, slot)| {
                            cache
                                .get(&(*index, slot))
                                .and_then(|(matched, _)| matched.then_some(slot))
                        });
                        if record_surplus::<C::Item, _, _>(&self.0, occupied, &mut unexpected) {
                            continue;
                        }
                        for slot in 0..expected_length {
                            if let Some((false, branch)) = cache.remove(&(*index, slot)) {
                                unexpected.append(branch);
                            }
                        }
                    }
                    if context.is_diagnostic() {
                        context.record(
                            unexpected
                                .into_evidence()
                                .explain(
                                    FailureBuilder::detached::<C>(FailureKind::Matching)
                                        .relation("has unexpected elements")
                                        .fact(Fact::labelled(
                                            "unexpected count",
                                            context
                                                .render()
                                                .value(&result.unmatched_actual.len())
                                                .into_rendered(),
                                        )),
                                )
                                .build(),
                        );
                    } else {
                        context.outcome(false, |context| context.describe::<C, _>(self));
                    }
                }
            }
            matched
        };
        let evidence = context.into_evidence();
        if matched { Ok(()) } else { Err(evidence) }
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

/// Retains the complete missing-subject constraint and bounded candidate rejections.
fn record_missing<
    C: Collection + ?Sized,
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
>(
    list: &L,
    slot: usize,
    rejections: impl Iterator<Item = Evidence>,
    context: &mut AssertionContext<'_, R>,
) {
    let description = context
        .is_diagnostic()
        .then(|| list.describe_at(slot, context));
    let mut alternatives = context.isolated_for_order(C::PRESENTATION.order());
    for branch in rejections {
        alternatives.append(branch);
    }
    if let Some(description) = description {
        let failure = FailureBuilder::detached::<C>(FailureKind::Matching).fact(Fact::labelled(
            "at slot",
            context.render().value(&slot).into_rendered(),
        ));
        let failure = alternatives.into_evidence().explain(
            failure
                .relation("is missing an element matching this expectation")
                .constraint(description),
        );
        context.record(failure.build());
    } else {
        context.outcome(false, |_| {
            FailureBuilder::detached::<()>(FailureKind::Matching)
                .relation("is missing a matching element")
                .build()
        });
    }
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
    if context.is_diagnostic() {
        if occupied.peek().is_none() {
            context.record(
                FailureBuilder::detached::<A>(FailureKind::Matching)
                    .relation("has an extra occurrence matching an already satisfied expectation")
                    .constraint(list.describe_at(first, context))
                    .fact(Fact::labelled(
                        "at slot",
                        context.render().value(&first).into_rendered(),
                    ))
                    .build(),
            );
            return true;
        }
        let mut constraints = context.isolated();
        for slot in core::iter::once(first).chain(occupied) {
            if constraints.is_diagnostic() {
                constraints.record(
                    FailureBuilder::detached::<A>(FailureKind::Matching)
                        .fact(Fact::labelled(
                            "at slot",
                            constraints.render().value(&slot).into_rendered(),
                        ))
                        .constraint(list.describe_at(slot, &constraints))
                        .build(),
                );
            } else {
                constraints.outcome(false, |_| {
                    FailureBuilder::detached::<()>(FailureKind::Matching)
                        .relation("already has a matching element")
                        .build()
                });
            }
        }
        context.record(
            constraints
                .into_evidence()
                .explain(
                    FailureBuilder::detached::<A>(FailureKind::Matching).relation(
                        "has an extra occurrence matching already satisfied expectations",
                    ),
                )
                .build(),
        );
    } else {
        context.outcome(false, |_| {
            FailureBuilder::detached::<()>(FailureKind::Matching)
                .relation("has an extra occurrence matching an already satisfied expectation")
                .build()
        });
    }
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
                      - unexpected count: 2
                    Nested failures:
                      - Expected: 1

                          Actual: 99
                      - has an extra occurrence matching an already satisfied expectation

                        Constraint:
                            is equal to

                            Expected: 1

                        Details:
                          - at slot: 0
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
                      - at slot: 2
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
                assert_that!(description.facts[0].label).is_equal_to("requirement");
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
            use crate::assertions::core::partial_eq::equal_to;
            struct DetailedMatcher;
            use crate::{
                AssertionContext, Expectation,
                expectation::Evidence,
                failure::{FailureBuilder, FailureKind},
            };
            impl Expectation<i32> for DetailedMatcher {
                type Success<'a> = ();
                type Rejection<'a> = Evidence;
                fn evaluate(
                    &self,
                    actual: &i32,
                    settings: &AssertionContext<'_>,
                ) -> Result<(), Evidence> {
                    let mut context = settings.isolated();
                    let result = context.evaluate(actual, &equal_to(99));
                    if *actual == 2 {
                        context.record(
                            FailureBuilder::detached::<i32>(FailureKind::Matching)
                                .relation("also fails a second requirement")
                                .build(),
                        );
                    }
                    if result {
                        Ok(())
                    } else {
                        Err(context.into_evidence())
                    }
                }
            }
            impl ExpectationDiagnostics<i32> for DetailedMatcher {
                const KIND: FailureKind = FailureKind::Equality;
                const FLATTEN: bool = true;
                fn explain<Target>(
                    &self,
                    rejected: Option<(&i32, Evidence)>,
                    failure: FailureBuilder<Target>,
                    context: &AssertionContext<'_, DebugRenderer>,
                ) -> FailureBuilder<Target> {
                    let render = context.render();
                    match rejected {
                        None => failure.relation("is equal to").expected(render.value(&99)),
                        Some((_, evidence)) => evidence.explain(failure),
                    }
                }
            }
            for values in [[1, 2], [2, 1]] {
                let failures =
                    bounded_failures(&values, &elements_are_in_any_order![DetailedMatcher], 1);
                let missing = &failures[0].children[0];
                assert_that!(missing.constraint.is_some()).is_true();
                assert_that!(missing.facts).has_length(1);
                assert_that!(missing.children).has_length(1);
                assert_that!(missing.omitted_children).is_equal_to(2);
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
                        predicate(|_: &Element| true),
                        predicate(|_: &Element| false)
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
                assert_that!(surplus.facts[0].label.as_ref()).is_equal_to("at slot");
                assert_that!(
                    surplus
                        .constraint
                        .as_ref()
                        .unwrap()
                        .relation
                        .as_deref()
                        .unwrap()
                )
                .is_equal_to("satisfies the predicate");
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
            assert_that!(context.evidence.omitted).is_equal_to(1);
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
            assert_that!(*calls.borrow()).is_equal_to([1, 1, 0]);
            assert_that!(context.probe(&[1, 1, 99], &elements_are_in_any_order![equal_to(1)]))
                .is_false();
            assert_that!(context.evidence.omitted).is_equal_to(0);
            assert_that!(context.into_evidence().children).is_empty();
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
