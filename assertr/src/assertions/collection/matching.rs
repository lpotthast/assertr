//! Reusable collection element matcher expectations.

use super::Collection;
use crate::{
    expectation::{AssertionContext, Evidence, Expectation, composite_items, evidence_items},
    failure::{FailureBuilder, FailureKind},
    renderer::{RenderingOrder, ValueRenderer},
};

/// Relations describing a search for one matching item, such as a collection element or map value.
pub(crate) struct MatchingItem {
    /// The relation of the unmet expectation, for example `contains a matching element`.
    pub(crate) contains: &'static str,
    /// The relation of the rejection, for example `does not contain a matching element`.
    pub(crate) does_not_contain: &'static str,
}

impl MatchingItem {
    /// Evaluates `matcher` against each item until one matches, retaining every rejected branch
    /// in `order`. A subject without items rejects with the described expectation as evidence.
    ///
    /// A diagnostic search first probes the items, so a passing search explains no rejected
    /// candidate. Only a failing search evaluates them again to collect evidence.
    pub(crate) fn find<'i, T, R, M, I>(
        &self,
        items: impl Fn() -> I,
        matcher: &M,
        order: RenderingOrder,
        settings: &AssertionContext<'_, R>,
    ) -> Result<(), Evidence>
    where
        T: ?Sized + 'i,
        M: Expectation<T, R>,
        I: Iterator<Item = &'i T>,
    {
        if !settings.is_probe() {
            let probe = settings.probing();
            if items().any(|item| probe.probe(item, matcher)) {
                return Ok(());
            }
        }
        let mut context = settings.isolated_for_order(order);
        for item in items() {
            let mut branch = context.isolated();
            if branch.evaluate(item, matcher) {
                return Ok(());
            }
            context.append(branch.into_evidence());
        }
        context.finish(false, |context| {
            self.describe::<T, _, _>(
                matcher,
                FailureBuilder::new::<()>(FailureKind::Matching),
                context,
            )
            .build()
        })
    }

    /// Explains a rejection from [`Self::find`], or describes the unmet expectation.
    pub(crate) fn explain<T, R, M>(
        &self,
        matcher: &M,
        rejection: Option<Evidence>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder
    where
        T: ?Sized,
        M: Expectation<T, R>,
    {
        match rejection {
            None => self.describe::<T, _, _>(matcher, failure, context),
            Some(evidence) => failure.relation(self.does_not_contain).evidence(evidence),
        }
    }

    fn describe<T, R, M>(
        &self,
        matcher: &M,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder
    where
        T: ?Sized,
        M: Expectation<T, R>,
    {
        failure
            .relation(self.contains)
            .children([context.describe::<T, _>(matcher)])
    }
}

pub(crate) const MATCHING_ELEMENT: MatchingItem = MatchingItem {
    contains: "contains a matching element",
    does_not_contain: "does not contain a matching element",
};

/// A reusable collection membership assertion requiring at least one matching element.
///
/// Evaluation stops at the first match. On rejection, it retains the original child failures,
/// ordered and limited according to the collection's presentation and active rendering budget.
/// Empty collections reject the assertion. No collection or element renderer is required beyond
/// the capabilities of the supplied matcher.
/// [`CollectionAssertions::contains_matching`](super::CollectionAssertions::contains_matching)
/// executes this same definition.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::matchers::ContainsMatching;
/// use assertr::matchers::eq;
///
/// let expected = ContainsMatching::new(eq(2));
/// assert_that!([1, 2, 3]).matches(&expected);
/// ```
#[derive(Debug, Clone)]
pub struct ContainsMatching<M>(M);

impl<M> ContainsMatching<M> {
    /// Owns the element matcher. Pass a reference to reuse a borrowed matcher.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self(matcher)
    }
}

/// Matches collections containing at least one matching element.
///
/// This is a convenience constructor for [`ContainsMatching::new`].
#[must_use]
pub const fn contains_matching<M>(matcher: M) -> ContainsMatching<M> {
    ContainsMatching::new(matcher)
}

impl<C: Collection + ?Sized, R, M> Expectation<C, R> for ContainsMatching<M>
where
    M: Expectation<C::Item, R>,
{
    evidence_items!(C);

    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        MATCHING_ELEMENT.find(
            || actual.elements(),
            &self.0,
            C::PRESENTATION.order(),
            settings,
        )
    }

    const KIND: FailureKind = FailureKind::Matching;

    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        MATCHING_ELEMENT.explain::<C::Item, _, _>(
            &self.0,
            rejected.map(|(_, evidence)| evidence),
            failure,
            context,
        )
    }
}

/// A reusable collection assertion rejecting every element that matches the supplied matcher.
///
/// Each element is probed once. The rejection identifies every unwanted element, limited by the
/// active rendering budget.
/// [`CollectionAssertions::does_not_contain_matching`](super::CollectionAssertions::does_not_contain_matching)
/// executes this same definition.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::matchers::DoesNotContainMatching;
/// use assertr::matchers::eq;
///
/// let unexpected = DoesNotContainMatching::new(eq(4));
/// assert_that!([1, 2, 3]).matches(&unexpected);
/// ```
#[derive(Debug, Clone)]
pub struct DoesNotContainMatching<M>(M);

impl<M> DoesNotContainMatching<M> {
    /// Owns the element matcher. Pass a reference to reuse a borrowed matcher.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self(matcher)
    }
}

/// Matches collections in which no element matches.
///
/// This is a convenience constructor for [`DoesNotContainMatching::new`].
#[must_use]
pub const fn does_not_contain_matching<M>(matcher: M) -> DoesNotContainMatching<M> {
    DoesNotContainMatching::new(matcher)
}

impl<C: Collection + ?Sized, R: ValueRenderer<C::Item>, M: Expectation<C::Item, R>>
    Expectation<C, R> for DoesNotContainMatching<M>
{
    composite_items!(C);

    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated_for_order(C::PRESENTATION.order());
        let mut found = false;
        for item in actual.elements() {
            if context.probe(item, &self.0) {
                found = true;
                context.record(|context| {
                    FailureBuilder::new::<C::Item>(FailureKind::Membership)
                        .actual(context.render().value(item))
                        .relation("matches the unwanted constraint")
                        .constraint(context.describe(&self.0))
                        .build()
                });
            }
        }
        if found {
            Err(context.into_evidence())
        } else {
            Ok(())
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure
                .relation("contains no matching elements")
                .children([context.describe(&self.0)]),
            Some((_, evidence)) => failure
                .relation("contains matching elements")
                .evidence(evidence),
        }
    }
}

#[cfg(test)]
mod tests {
    mod contains_matching {
        use core::{cell::Cell, fmt};

        use crate::{
            assertions::{collection::contains_matching, core::partial_eq::eq},
            matchers::{all_of, predicate},
            prelude::*,
            test_support::{NoRenderer, UnorderedSet, assert_bounded_order, bounded_failures},
        };

        struct ReverseRenderer<'a>(&'a Cell<usize>);

        impl ValueRenderer<i32> for ReverseRenderer<'_> {
            fn fmt(&self, value: &i32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(formatter, "{}", 10 - value)
            }
        }

        #[test]
        fn is_implemented_without_renderer_support() {
            assert_that!([1, 2])
                .with_renderer(NoRenderer)
                .matches(contains_matching(matchers::anything()));
        }

        #[test]
        fn a_passing_search_explains_no_rejected_candidate() {
            let renders = Cell::new(0);
            let values = UnorderedSet((0..100).collect());
            assert_that!(&values)
                .with_renderer(ReverseRenderer(&renders))
                .matches(contains_matching(eq(99)));
            assert_that!(renders.get()).is_equal_to(0);

            let failures = assert_that!(&values)
                .with_renderer(ReverseRenderer(&renders))
                .capture(|it| it.matches(contains_matching(eq(100))));
            assert_that!(failures).has_length(1);
            assert_that!(renders.get()).is_greater_than(0);
        }

        #[test]
        fn bounded_evidence_is_independent_of_iteration_order() {
            assert_bounded_order(&contains_matching(eq(9)));
        }

        #[test]
        fn stays_one_nested_group_inside_compositions() {
            let failures = assert_that!(vec![vec![2, 3], vec![4]])
                .with_location(false)
                .capture(|it| {
                    it.matches(matchers::each(contains_matching(predicate(|it: &i32| {
                        *it == 1
                    }))))
                });
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure.has_text_report(indoc::indoc! {r"
                    -------- assertr --------
                    Expression: `vec![vec![2, 3], vec![4]]`

                    does not match

                    Nested failures:
                      - does not contain a matching element

                        Nested failures:
                          - Actual: 2

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate

                          - Actual: 3

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate

                      - does not contain a matching element

                        Nested failures:
                          - Actual: 4

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate
                    -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn sorts_nested_branches_before_limiting_them() {
            let matcher = contains_matching(all_of(matchers![eq(9), eq(0)]));
            let failures = bounded_failures(&[3, 2, 1], &matcher, 1);

            assert_that!(failures[0].omitted_children).is_equal_to(5);
            assert_that!(failures[0].children).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|item| item.expected.as_ref())
                        .is_equal_to(Some(&AssertionContext::default().render().value(&0)));
                },
            ]);
            assert_bounded_order(&matcher);
        }

        #[test]
        fn selects_evidence_using_the_active_renderer_and_skips_zero_budget_rendering() {
            for limit in [0, 1] {
                let renders = Cell::new(0);
                let failures = assert_that!(UnorderedSet(vec![1, 2, 3]))
                    .with_renderer(ReverseRenderer(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| it.matches(contains_matching(eq(9))));

                assert_that!(renders.get()).is_equal_to(if limit == 0 { 0 } else { 6 });
                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive(|failure| &failure.omitted_children)
                            .is_equal_to(3 - limit);
                        failure
                            .derive(|failure| &failure.children)
                            .contains_exactly_satisfying(vec![
                                |child: AssertThat<
                                    AssertionFailure,
                                    Capture,
                                >| {
                                    child.derive(|child| &child.actual).is_some_satisfying(
                                        |actual| {
                                            actual
                                                .derive_owned(|value| format!("{value:#}"))
                                                .is_equal_to("7");
                                        },
                                    );
                                };
                                limit
                            ]);
                    },
                ]);
            }
        }
    }

    mod does_not_contain_matching {
        use crate::{
            assertions::{collection::does_not_contain_matching, core::partial_eq::eq},
            prelude::*,
        };

        #[test]
        fn rejects_every_matching_element_and_counts_omitted_ones_at_zero_budget() {
            for limit in [0, 1, usize::MAX] {
                let failures = assert_that!([1, 2, 1])
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| it.matches(does_not_contain_matching(eq(1))));

                assert_that!(failures).has_length(1);
                let retained = limit.min(2);
                assert_that!(failures[0].children).has_length(retained);
                assert_that!(failures[0].omitted_children).is_equal_to(2 - retained);
            }
        }

        #[test]
        fn probes_record_no_evidence() {
            let context = AssertionContext::default();
            assert_that!(context.probe(&[1, 2], &does_not_contain_matching(eq(1)))).is_false();
            assert_that!(context.probe(&[1, 2], &does_not_contain_matching(eq(3)))).is_true();
        }
    }
}
