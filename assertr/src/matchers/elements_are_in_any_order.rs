use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, MatcherList};
use crate::{
    AssertionFailure, Fact,
    assertions::collection::Collection,
    failure::{FailureBuilder, FailureKind},
    renderer::{GroupStyle, Rendered, RenderedBody, RenderingOrder, TypeHint},
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
/// Plain equality candidates are summarized as non-matching elements. Rejections with paths,
/// messages, or other evidence retain their detailed failures. The `at slot` fact identifies a
/// zero-based expectation position, never an actual collection index.
pub fn elements_are_in_any_order<L>(list: L) -> ElementsAreInAnyOrder<L> {
    ElementsAreInAnyOrder(list)
}

impl<C, R, L> AssertrMatcher<C, R> for ElementsAreInAnyOrder<L>
where
    R: crate::ValueRenderer<usize>,
    C: Collection + ?Sized,
    L: MatcherList<C::Item, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("has exactly these elements in any order")
            .omitted_children(self.0.len().saturating_sub(context.render().max_items()))
            .children(
                (0..self.0.len().min(context.render().max_items()))
                    .map(|index| self.0.describe_at(index, context)),
            )
    }

    fn evaluate(&self, actual: &C, context: &mut MatchContext<'_, R>) -> MatchResult {
        let actual = actual.elements().collect::<Vec<_>>();
        let expected_length = self.0.len();
        // A sparse cache avoids allocating the full Cartesian product for easy exact matches.
        let mut cache = BTreeMap::new();
        let mut evaluate_pair = |index: usize, slot| {
            cache
                .entry((index, slot))
                .or_insert_with(|| {
                    let mut branch = context.isolated_for_order(C::PRESENTATION.order());
                    let matched = self.0.evaluate_at(slot, actual[index], &mut branch).matched;
                    (matched, branch)
                })
                .0
        };
        let result = match_bipartite(actual.len(), expected_length, &mut evaluate_pair);
        let matched = result.is_exact();
        if !matched && context.is_positive() && context.is_diagnostic() {
            complete_unmatched_pairs(&result, actual.len(), expected_length, evaluate_pair);
        }
        if matched != context.is_positive() {
            if matched {
                // Cached successful constraints explain negative matching without replaying code.
                let mut successes = context.isolated_for_order(C::PRESENTATION.order());
                for pair in result.matched_pairs {
                    if let Some((_, branch)) = cache.remove(&pair) {
                        successes.append(branch);
                    }
                }
                context.append(successes);
                if context.evidence.is_empty() && context.omitted == 0 {
                    context.outcome(true, |context| {
                        <Self as AssertrMatcher<C, R>>::describe(self, context)
                    });
                }
            } else {
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
                        context.record_group(
                            FailureBuilder::detached::<C>(FailureKind::Matching)
                                .relation("has unexpected elements")
                                .fact(Fact::labelled(
                                    "unexpected count",
                                    context.render().value(&result.unmatched_actual.len()),
                                ))
                                .children(unexpected.evidence)
                                .omitted_children(unexpected.omitted)
                                .build(),
                        );
                    } else {
                        context.outcome(false, |context| {
                            <Self as AssertrMatcher<C, R>>::describe(self, context)
                        });
                    }
                }
            }
        }
        MatchResult::new(matched)
    }
}

/// Summarizes plain equality candidates without discarding richer matcher or callback evidence.
fn record_missing<
    'r,
    C: Collection + ?Sized,
    R: crate::ValueRenderer<usize>,
    L: MatcherList<C::Item, R>,
>(
    list: &L,
    slot: usize,
    rejections: impl Iterator<Item = MatchContext<'r, R>>,
    context: &mut MatchContext<'r, R>,
) {
    let description = context
        .is_diagnostic()
        .then(|| list.describe_at(slot, context));
    let expected = description
        .as_ref()
        .filter(|description| {
            description.relation == "is equal to"
                && description.children.is_empty()
                && description.omitted_children == 0
        })
        .and_then(|description| description.expected.as_ref());
    let mut summarize = expected.is_some();
    let mut alternatives = context.isolated_for_order(C::PRESENTATION.order());
    for branch in rejections {
        // Check every candidate before limiting the group. An omitted complex rejection must
        // not be counted as an omitted element in a simple value summary.
        summarize &= branch.omitted == 0
            && branch.evidence.len() == 1
            && expected
                .is_some_and(|expected| is_plain_equality_rejection(&branch.evidence[0], expected));
        alternatives.append(branch);
    }
    if let Some(description) = description {
        let failure = FailureBuilder::detached::<C>(FailureKind::Matching)
            .fact(Fact::labelled("at slot", context.render().value(&slot)));
        let failure = if summarize && !alternatives.evidence.is_empty() {
            let sorted = alternatives.evidence_order() == RenderingOrder::SortByRenderedText;
            let candidates = Rendered {
                body: RenderedBody::Group {
                    style: GroupStyle::List,
                    items: alternatives
                        .evidence
                        .into_iter()
                        .map(|failure| failure.actual.unwrap())
                        .collect(),
                    omitted: alternatives.omitted,
                    sorted,
                },
                type_name: None,
                hint: TypeHint::Short,
                shows_type_hint: false,
                compact: true,
            };
            failure
                .relation("is missing an expected element")
                .expected(description.expected.unwrap())
                .fact(Fact::labelled("non-matching elements", candidates))
        } else {
            failure
                .relation("is missing an element matching this expectation")
                .constraint(description)
                .children(alternatives.evidence)
                .omitted_children(alternatives.omitted)
        };
        context.record_group(failure.build());
    } else {
        context.outcome(false, |_| {
            ConstraintDescription::new("is missing a matching element")
        });
    }
}

fn is_plain_equality_rejection(failure: &AssertionFailure, expected: &Rendered) -> bool {
    failure.kind == FailureKind::Equality
        && failure.actual.is_some()
        && failure.expected.as_ref() == Some(expected)
        && failure.relation.is_none()
        && failure.unexpected.is_none()
        && failure.constraint.is_none()
        && failure.path.is_empty()
        && failure.location.is_none()
        && failure.expression.is_none()
        && failure.subject_name.is_none()
        && failure.messages.is_empty()
        && failure.facts.is_empty()
        && failure.children.is_empty()
        && failure.omitted_children == 0
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
    context: &mut MatchContext<'_, R>,
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
                    .fact(Fact::labelled("at slot", context.render().value(&first)))
                    .build(),
            );
            return true;
        }
        let mut constraints = context.isolated();
        for slot in core::iter::once(first).chain(occupied) {
            if constraints.is_diagnostic() {
                constraints.record(
                    FailureBuilder::detached::<A>(FailureKind::Matching)
                        .fact(Fact::labelled("at slot", constraints.render().value(&slot)))
                        .constraint(list.describe_at(slot, &constraints))
                        .build(),
                );
            } else {
                constraints.outcome(false, |_| {
                    ConstraintDescription::new("already has a matching element")
                });
            }
        }
        context.record_group(
            FailureBuilder::detached::<A>(FailureKind::Matching)
                .relation("has an extra occurrence matching already satisfied expectations")
                .children(constraints.evidence)
                .omitted_children(constraints.omitted)
                .build(),
        );
    } else {
        context.outcome(false, |_| {
            ConstraintDescription::new(
                "has an extra occurrence matching an already satisfied expectation",
            )
        });
    }
    true
}

/// Exact unordered matcher list with equality shorthand and duplicate preservation.
#[macro_export]
macro_rules! elements_are_in_any_order {
    ($($value:expr),* $(,)?) => {
        $crate::matchers::elements_are_in_any_order($crate::matchers![$($value),*])
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{
            ge,
            test_support::{assert_bounded_order, bounded_failures},
        },
        prelude::*,
    };

    #[test]
    fn bounded_candidate_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&elements_are_in_any_order![9], true);
        assert_bounded_order(&elements_are_in_any_order![ge(0), ge(0), ge(0)], false);
    }

    #[test]
    fn sorts_unexpected_evidence_before_limiting_it() {
        let matcher = elements_are_in_any_order![9];
        let expected = bounded_failures(&[9, 1, 2, 3], &matcher, true, 1);
        let actual = bounded_failures(&[9, 3, 2, 1], &matcher, true, 1);

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
    fn negative_unordered_evidence_uses_each_assigned_element_once() {
        let failures = assert_that!([1, 2])
            .capture(|it| it.does_not_match(elements_are_in_any_order![ge(0), ge(0)]));

        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| {
                        value
                            .children
                            .iter()
                            .map(|child| rendered_text(child.actual.as_ref().unwrap()))
                            .collect::<Vec<_>>()
                    })
                    .contains_exactly_in_any_order(["1", "2"]);
            },
        ]);
    }

    #[test]
    fn supports_overlapping_constraints() {
        assert_that!([1, 2]).matches(elements_are_in_any_order![ge(1), 1]);
    }

    mod evaluate {
        use super::*;
        use crate::matchers::{AssertrMatcher, MatchContext, predicate, satisfying};
        use core::cell::{Cell, RefCell};
        use indoc::indoc;

        #[test]
        fn reports_rejected_and_surplus_occurrences() {
            let failures = bounded_failures(
                &[1, 1, 99],
                &elements_are_in_any_order![1],
                true,
                usize::MAX,
            );
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
                &elements_are_in_any_order![ge(0), 1, 99],
                true,
                usize::MAX,
            );
            assert_that!(failures[0]).has_text_report(indoc! {r"
                -------- assertr --------
                Expression: `actual`

                does not match

                Nested failures:
                  - is missing an expected element

                    Expected: 99

                    Details:
                      - at slot: 2
                      - non-matching elements: [1, 2] (sorted for rendering)
                -------- assertr --------
            "});
        }

        #[test]
        fn candidate_summaries_reuse_rendered_leaves_and_inherited_order() {
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
            let budget = RenderingBudget::builder()
                .max_items(1)
                .max_leaf_characters(7)
                .build();
            let mut context = MatchContext::new(&renderer, budget)
                .isolated_for_order(RenderingOrder::SortByRenderedText);
            assert_that!(
                elements_are_in_any_order![ge(0), 1, 99]
                    .evaluate(&[2, 1], &mut context)
                    .matched
            )
            .is_false();
            // Two actual/expected comparisons and one expectation description. The summary
            // moves rendered leaves without invoking the renderer again.
            assert_that!(renderer.0.get()).is_equal_to(5);
            let failures = context.into_failures();
            let RenderedBody::Group {
                items,
                omitted,
                sorted,
                ..
            } = &failures[0].facts[1].value.body
            else {
                panic!("missing the candidate value summary")
            };
            assert_that!(*sorted).is_true();
            assert_that!(*omitted).is_equal_to(1);
            assert_that!(items).has_length(1);
            assert_that!(items[0].type_name).is_equal_to(Some("i32"));
            let RenderedBody::Text {
                text,
                omitted_characters,
            } = &items[0].body
            else {
                panic!("missing the rendered candidate")
            };
            assert_that!(text).is_equal_to("custom-");
            assert_that!(*omitted_characters).is_equal_to(1);
        }

        #[test]
        fn candidate_summaries_preserve_field_rejection_paths() {
            use crate::{
                failure::PathSegment,
                matchers::{equal_to, field::field},
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
        fn candidate_summaries_do_not_hide_complex_omitted_rejections() {
            use crate::{
                failure::{FailureBuilder, FailureKind},
                matchers::{ConstraintDescription, MatchResult, equal_to},
            };
            struct DetailedMatcher;
            impl AssertrMatcher<i32> for DetailedMatcher {
                fn describe(&self, context: &MatchContext<'_>) -> ConstraintDescription {
                    ConstraintDescription::new("is equal to").expected(context.render().value(&99))
                }
                fn evaluate(&self, actual: &i32, context: &mut MatchContext<'_>) -> MatchResult {
                    let result = equal_to(99).evaluate(actual, context);
                    if *actual == 2 {
                        context.record(
                            FailureBuilder::detached::<i32>(FailureKind::Matching)
                                .relation("also fails a second requirement")
                                .build(),
                        );
                    }
                    result
                }
            }
            for values in [[1, 2], [2, 1]] {
                let failures = bounded_failures(
                    &values,
                    &elements_are_in_any_order![DetailedMatcher],
                    true,
                    1,
                );
                let missing = &failures[0].children[0];
                assert_that!(missing.constraint.is_some()).is_true();
                assert_that!(missing.facts).has_length(1);
                assert_that!(missing.children).has_length(1);
                assert_that!(missing.omitted_children).is_equal_to(2);
            }
        }

        #[test]
        fn completes_unexpected_evidence_before_sorting_and_limiting() {
            let matcher = elements_are_in_any_order![1];
            for limit in [0, 1, 2, usize::MAX] {
                let expected = bounded_failures(&[1, 1, 99], &matcher, true, limit);
                for values in [[1, 99, 1], [99, 1, 1]] {
                    let actual = bounded_failures(&values, &matcher, true, limit);
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
            let matcher = elements_are_in_any_order![ge(0), 1, 99];
            for limit in [0, 1, 2, usize::MAX] {
                let expected = bounded_failures(&[1, 2], &matcher, true, limit);
                let actual = bounded_failures(&[2, 1], &matcher, true, limit);
                assert_that!(ToHumanReadableText.render(&actual[0]))
                    .is_equal_to(ToHumanReadableText.render(&expected[0]));
                if limit == 0 {
                    assert_that!(expected[0].children).is_empty();
                    assert_that!(expected[0].omitted_children).is_equal_to(1);
                } else {
                    let missing = &expected[0].children[0];
                    assert_that!(missing.children).is_empty();
                    assert_that!(missing.omitted_children).is_equal_to(0);
                    let crate::renderer::RenderedBody::Group { items, omitted, .. } =
                        &missing.facts[1].value.body
                    else {
                        panic!("missing the candidate value summary")
                    };
                    assert_that!(items).has_length(limit.min(2));
                    assert_that!(*omitted).is_equal_to(2 - limit.min(2));
                    assert_that!(rendered_text(&items[0])).is_equal_to("1");
                }
            }
        }

        #[test]
        fn preserves_surplus_multiplicity_and_all_occupied_constraints_within_budget() {
            let matcher = elements_are_in_any_order![ge(0), 1];
            for limit in [1, 2, usize::MAX] {
                let failures = bounded_failures(&[1, 1, 1, 1], &matcher, true, limit);
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
                            .as_ref()
                    )
                    .is_equal_to("is equal to");
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
                assert_that!(surplus.constraint.as_ref().unwrap().relation.as_ref())
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
            let failures = assert_that!([(0, 1), (1, 99)])
                .capture(|it| it.matches(crate::matchers::elements_are_in_any_order(list)));
            assert_that!(*calls.borrow()).is_equal_to([[1, 1], [1, 1]]);
            // Rejections against the missing expectation are retained there first.
            assert_that!(failures[0].children).has_length(2);
            assert_that!(failures[0].children[0].children).has_length(2);
            assert_that!(failures[0].children[1].children).has_length(1);
        }

        #[test]
        fn surplus_groups_preserve_the_enclosing_path_once() {
            use crate::failure::PathSegment;
            let mut context = MatchContext::default();
            let result = context.scoped(PathSegment::Field("items"), |context| {
                elements_are_in_any_order![1].evaluate(&[1, 1], context)
            });
            assert_that!(result.matched).is_false();
            let failures = context.into_failures();
            assert_that!(failures[0].path).is_equal_to([PathSegment::Field("items")]);
            let surplus = &failures[0].children[0];
            assert_that!(surplus.path).is_empty();
            assert_that!(surplus.constraint.is_some()).is_true();
        }

        #[test]
        fn empty_expectations_report_the_unexpected_count() {
            let failures =
                bounded_failures(&[1, 2], &elements_are_in_any_order![], true, usize::MAX);
            let unexpected = &failures[0].children[0];
            assert_that!(unexpected.relation.as_deref())
                .is_equal_to(Some("has unexpected elements"));
            assert_that!(rendered_text(&unexpected.facts[0].value)).is_equal_to("2");
            assert_that!(unexpected.children).is_empty();
            assert_that!(unexpected.omitted_children).is_equal_to(0);
        }

        #[test]
        fn only_completes_pairs_when_positive_diagnostics_can_retain_evidence() {
            for (positive, limit, expected_calls) in [
                (true, 0, [1, 1, 0]),
                (true, usize::MAX, [1, 1, 1]),
                (false, usize::MAX, [1, 1, 0]),
            ] {
                let calls = RefCell::new([0; 3]);
                let matcher = elements_are_in_any_order![predicate(|index: &usize| {
                    calls.borrow_mut()[*index] += 1;
                    *index < 2
                })];
                let mut context = MatchContext::new(
                    &DebugRenderer,
                    RenderingBudget::builder().max_items(limit).build(),
                );
                context.set_positive(positive);
                assert_that!(matcher.evaluate(&[0, 1, 2], &mut context).matched).is_false();
                assert_that!(*calls.borrow()).is_equal_to(expected_calls);
                if !positive || limit == 0 {
                    assert_that!(context.into_failures()).is_empty();
                }
            }
        }

        #[test]
        fn exact_matches_keep_sparse_evaluation_under_both_polarities() {
            for positive in [true, false] {
                let calls = Cell::new(0);
                let matcher = predicate(|_: &i32| {
                    calls.set(calls.get() + 1);
                    true
                });
                let matcher = elements_are_in_any_order![&matcher, &matcher];
                let mut context = MatchContext::default();
                context.set_positive(positive);
                assert_that!(matcher.evaluate(&[1, 1], &mut context).matched).is_true();
                assert_that!(calls.get()).is_equal_to(2);
                assert_that!(context.into_failures()).has_length(if positive { 0 } else { 2 });
            }
        }

        #[test]
        fn a_zero_item_budget_does_not_render_completed_or_surplus_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("rendered omitted evidence")
                }
            }
            let mut context = MatchContext::new(
                &NeverRender,
                RenderingBudget::builder().max_items(0).build(),
            );
            assert_that!(
                elements_are_in_any_order![1]
                    .evaluate(&[1, 1, 99], &mut context)
                    .matched
            )
            .is_false();
            assert_that!(context.omitted_children()).is_equal_to(1);
            assert_that!(context.into_failures()).is_empty();
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
                .with_rendering_budget(RenderingBudget::builder().max_items(0).build())
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
        use crate::matchers::MatchContext;

        #[test]
        fn surplus_search_does_not_complete_unvisited_pairs_or_render() {
            use crate::matchers::{equal_to, predicate};
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
            let context = MatchContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[0, 1, 2], &matcher)).is_false();
            assert_that!(*calls.borrow()).is_equal_to([1, 1, 0]);
            assert_that!(context.probe(&[1, 1, 99], &elements_are_in_any_order![equal_to(1)]))
                .is_false();
            assert_that!(context.omitted_children()).is_equal_to(0);
            assert_that!(context.into_failures()).is_empty();
        }

        #[test]
        fn length_mismatches_do_not_render_numeric_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = MatchContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are_in_any_order![])).is_false();
            assert_that!(context.into_failures()).is_empty();
        }
    }
}
