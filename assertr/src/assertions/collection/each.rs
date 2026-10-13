use crate::{
    assertions::collection::{Collection, StableOrder},
    expectation::{AssertionContext, Evidence, Expectation, composite_items},
    failure::{FailureBuilder, FailureKind, PathSegment},
};

/// Applies one matcher to every element of a collection with a stable order, locating each
/// failure at its element's position. Construct it with [`each`].
#[derive(Debug, Clone)]
pub struct Each<M>(M);

/// Matches collections in which every element matches `matcher`. An empty collection matches.
///
/// Requires [`StableOrder`], so a failure names the position of every element that does not
/// match, such as `At [2]:`. Use [`each_in_any_order`] for collections without a guaranteed
/// order, such as a `HashSet` or a `BinaryHeap`.
///
/// ```
/// use assertr::{matchers::{each, ge}, prelude::*};
///
/// assert_that!([18, 30]).matches(each(ge(18)));
/// let failures = assert_that!([18, 3, 30]).capture(|it| it.matches(each(ge(18))));
/// assert_that!(failures[0].to_string()).contains("At [1]:");
/// ```
#[must_use]
pub const fn each<M>(matcher: M) -> Each<M> {
    Each(matcher)
}

impl<C: StableOrder + ?Sized, R, M> Expectation<C, R> for Each<M>
where
    M: Expectation<C::Item, R>,
{
    composite_items!(C);

    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let mut matched = true;
        for (index, item) in actual.elements().enumerate() {
            matched &= context.scoped(PathSegment::Index(index), |context| {
                context.evaluate(item, &self.0)
            });
        }
        context.finish(matched, |context| context.describe::<C, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;

    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_every_element(
            &self.0,
            rejected.map(|(_, evidence)| evidence),
            failure,
            context,
        )
    }
}

/// Applies one matcher to every element of any collection, without positional meaning. Construct
/// it with [`each_in_any_order`].
#[derive(Debug, Clone)]
pub struct EachInAnyOrder<M>(M);

/// Matches collections in which every element matches `matcher`, in any order. An empty
/// collection matches.
///
/// Works for every [`Collection`]. Failures show each rejected element's own evidence but no
/// position, and follow the collection's rendering order, so a hash set reports deterministically.
/// Prefer [`each`] when the collection has a stable order.
///
/// ```
/// use assertr::{matchers::{each_in_any_order, ge}, prelude::*};
/// use std::collections::BinaryHeap;
///
/// assert_that!(BinaryHeap::from([18, 30])).matches(each_in_any_order(ge(18)));
/// ```
#[must_use]
pub const fn each_in_any_order<M>(matcher: M) -> EachInAnyOrder<M> {
    EachInAnyOrder(matcher)
}

impl<C: Collection + ?Sized, R, M> Expectation<C, R> for EachInAnyOrder<M>
where
    M: Expectation<C::Item, R>,
{
    composite_items!(C);

    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated_for_order(C::PRESENTATION.order());
        let mut matched = true;
        for item in actual.elements() {
            matched &= context.evaluate(item, &self.0);
        }
        context.finish(matched, |context| context.describe::<C, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;

    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_every_element(
            &self.0,
            rejected.map(|(_, evidence)| evidence),
            failure,
            context,
        )
    }
}

/// Explains a rejection of [`Each`] or [`EachInAnyOrder`], or describes the unmet expectation.
fn explain_every_element<T: ?Sized, R, M: Expectation<T, R>>(
    matcher: &M,
    rejection: Option<Evidence>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder {
    match rejection {
        None => failure
            .relation("has every element matching")
            .children([context.describe(matcher)]),
        Some(evidence) => failure.relation("does not match").evidence(evidence),
    }
}

#[cfg(test)]
mod tests {
    mod each {
        use indoc::formatdoc;

        use super::super::each;
        use crate::{
            assertions::core::partial_eq::eq, failure::PathSegment, matchers::ge, prelude::*,
        };

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2]), matches(each(eq(1))));
        }

        #[test]
        fn matches_every_element_and_empty_collections() {
            assert_that!([18, 30]).matches(each(ge(18)));
            assert_that!([] as [i32; 0]).matches(each(ge(18)));
            assert_that!(alloc::collections::BTreeSet::from([18, 30])).matches(each(ge(18)));
        }

        #[test]
        fn locates_each_rejected_element_at_its_position() {
            let failures = assert_that!([1, 9, 9])
                .with_location(false)
                .capture(|it| it.matches(each(eq(9))));
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].path).is_equal_to([PathSegment::Index(0)]);

            assert_that!(|| {
                assert_that!([9, 1, 2])
                    .with_location(false)
                    .matches(each(eq(9)));
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `[9, 1, 2]`

                does not match

                Nested failures:
                  - At [1]:
                    Expected: 9

                      Actual: 1

                  - At [2]:
                    Expected: 9

                      Actual: 2
                -------- assertr --------
            "});
        }
    }

    mod each_in_any_order {
        use super::super::each_in_any_order;
        use crate::{
            assertions::core::partial_eq::eq, matchers::ge, prelude::*,
            test_support::assert_bounded_order,
        };

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2]), matches(each_in_any_order(eq(1))));
        }

        #[test]
        fn matches_every_element_of_any_collection() {
            assert_that!([18, 30]).matches(each_in_any_order(ge(18)));
            assert_that!(alloc::collections::BinaryHeap::from([18, 30]))
                .matches(each_in_any_order(ge(18)));
            #[cfg(feature = "std")]
            assert_that!(std::collections::HashSet::from([18, 30]))
                .matches(each_in_any_order(ge(18)));
        }

        #[test]
        fn rejected_elements_carry_no_position() {
            let failures = assert_that!([1, 9])
                .with_location(false)
                .capture(|it| it.matches(each_in_any_order(eq(9))));
            assert_that!(failures[0].children[0].path).is_empty();
        }

        #[test]
        fn bounded_evidence_is_independent_of_iteration_order() {
            assert_bounded_order(&each_in_any_order(eq(9)));
        }
    }
}
