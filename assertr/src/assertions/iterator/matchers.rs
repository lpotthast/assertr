//! Matcher execution over bounded, single-pass iterator scans.

use super::{
    AssertThat, AssertionContext, Borrow, ExpectationDiagnostics, Mode, PREVIEW_CAPACITY,
    PhantomData, PositionReporting, Scan, Tail, Vec, VecDeque, exact_size_hint, execute,
};
use crate::{
    Fact, ValueRenderer,
    expectation::{Evidence, MatcherList},
    failure::{FailureBuilder, FailureKind, PathSegment},
};

fn explain_scan<Target, R: ValueRenderer<usize>>(
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
    evidence: Evidence,
    relation: &'static str,
    consumed: usize,
) -> FailureBuilder<Target> {
    evidence.explain(failure.relation(relation).fact(Fact::labelled(
        "Consumed",
        context.render().value(&consumed),
    )))
}

struct ContainsMatching<'e, T, P> {
    expected: &'e P,
    item: PhantomData<fn() -> T>,
    positions: PositionReporting,
}

impl<T, P, I, R> Scan<I, R> for ContainsMatching<'_, T, P>
where
    I: Iterator,
    I::Item: Borrow<T>,
    P: ExpectationDiagnostics<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = (Evidence, usize);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut retained: VecDeque<Evidence> = VecDeque::new();
        let mut consumed = 0;
        let mut discarded = 0;
        for item in iterator {
            let mut child = context.isolated();
            let accepted = if let Some(index) = self.positions.index(consumed) {
                child.scoped(PathSegment::Index(index), |child| {
                    child.evaluate(item.borrow(), self.expected)
                })
            } else {
                child.evaluate(item.borrow(), self.expected)
            };
            consumed += 1;
            if accepted {
                return Ok(());
            }
            if retained.len() == PREVIEW_CAPACITY
                && let Some(old) = retained.pop_front()
            {
                discarded += old.children.len() + old.omitted;
            }
            retained.push_back(child.into_evidence());
        }
        let mut child = context.isolated();
        child.omitted = discarded;
        for evidence in retained {
            child.append(evidence);
        }
        child
            .finish(false, |child| child.describe(self.expected))
            .map_err(|evidence| (evidence, consumed))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let (evidence, consumed) = rejection;
        explain_scan(
            failure,
            context,
            evidence,
            "does not contain a matching element",
            consumed,
        )
    }
}

struct ContainsNoMatching<'e, T, P> {
    expected: &'e P,
    item: PhantomData<fn() -> T>,
    positions: PositionReporting,
}

impl<T, P, I, R> Scan<I, R> for ContainsNoMatching<'_, T, P>
where
    I: Iterator,
    I::Item: Borrow<T>,
    P: ExpectationDiagnostics<T, R>,
    R: ValueRenderer<usize>,
    R: ValueRenderer<T>,
{
    type Rejection = (Evidence, usize);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        for (index, item) in iterator.enumerate() {
            let mut child = context.isolated();
            if child.probe(item.borrow(), self.expected) {
                if child.is_diagnostic() {
                    child.record(
                        FailureBuilder::detached::<T>(FailureKind::Membership)
                            .actual(child.render().value(item.borrow()))
                            .relation("matches the unwanted constraint")
                            .constraint(child.describe(self.expected))
                            .path(self.positions.index(index).map(PathSegment::Index))
                            .build(),
                    );
                } else {
                    child.outcome(false, |child| child.describe(self.expected));
                }
                return Err((child.into_evidence(), index + 1));
            }
        }
        Ok(())
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let (evidence, consumed) = rejection;
        explain_scan(
            failure,
            context,
            evidence,
            "contains an unexpected matching element",
            consumed,
        )
    }
}

enum SequenceRejection {
    KnownLength {
        actual: usize,
        expected: usize,
    },
    Missing {
        evidence: Evidence,
        consumed: usize,
        expected: usize,
    },
    Mismatch {
        evidence: Evidence,
        consumed: usize,
    },
    Extra {
        evidence: Evidence,
        consumed: usize,
    },
}

struct ElementsAre<'e, T, L> {
    expected: &'e L,
    item: PhantomData<fn() -> T>,
    exact: bool,
}

impl<T, L, I, R> Scan<I, R> for ElementsAre<'_, T, L>
where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = SequenceRejection;
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let expected_length = self.expected.len();
        let mut child = context.isolated();
        if let Some(actual) = exact_size_hint(&iterator)
            && ((self.exact && actual != expected_length)
                || (!self.exact && actual < expected_length))
        {
            return Err(SequenceRejection::KnownLength {
                actual,
                expected: expected_length,
            });
        }
        for index in 0..expected_length {
            let Some(item) = iterator.next() else {
                child.scoped(PathSegment::Index(index), |child| {
                    child.outcome(false, |child| self.expected.describe_at(index, child));
                });
                return Err(SequenceRejection::Missing {
                    evidence: child.into_evidence(),
                    consumed: index,
                    expected: expected_length,
                });
            };
            if !child.scoped(PathSegment::Index(index), |child| {
                self.expected.evaluate_at(index, item.borrow(), child)
            }) {
                return Err(SequenceRejection::Mismatch {
                    evidence: child.into_evidence(),
                    consumed: index + 1,
                });
            }
        }
        if self.exact && iterator.next().is_some() {
            return Err(SequenceRejection::Extra {
                evidence: child.into_evidence(),
                consumed: expected_length + 1,
            });
        }
        Ok(())
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejection {
            SequenceRejection::KnownLength { actual, expected } => failure
                .relation("does not have the required sequence length")
                .fact(Fact::labelled("Reported length", render.value(&actual)))
                .fact(Fact::labelled("Expected length", render.value(&expected))),
            SequenceRejection::Missing {
                evidence,
                consumed,
                expected,
            } => explain_scan(
                failure,
                context,
                evidence,
                "is missing a matching position",
                consumed,
            )
            .fact(Fact::labelled("Expected length", render.value(&expected))),
            SequenceRejection::Mismatch { evidence, consumed } => explain_scan(
                failure,
                context,
                evidence,
                "does not match the required position",
                consumed,
            ),
            SequenceRejection::Extra { evidence, consumed } => {
                explain_scan(failure, context, evidence, "has an extra element", consumed)
            }
        }
    }
}

struct MatchingWindow<'e, T, L> {
    expected: &'e L,
    item: PhantomData<fn() -> T>,
    suffix: bool,
}

impl<T, L, I, R> Scan<I, R> for MatchingWindow<'_, T, L>
where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = (Evidence, usize, usize);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let expected_length = self.expected.len();
        if expected_length == 0 {
            return Ok(());
        }
        let mut window = Tail::new(expected_length);
        let mut final_window = Evidence::default();
        let evaluate_window = |window: &VecDeque<I::Item>, consumed: usize| {
            let mut child = context.isolated();
            let mut matched = true;
            for (slot, item) in window.iter().enumerate() {
                matched &= child.scoped(
                    PathSegment::Index(consumed - expected_length + slot),
                    |child| self.expected.evaluate_at(slot, item.borrow(), child),
                );
            }
            if matched {
                Ok(())
            } else {
                Err(child.into_evidence())
            }
        };
        for item in iterator {
            window.push(item);
            if !self.suffix && window.items.len() == expected_length {
                match evaluate_window(&window.items, window.consumed) {
                    Ok(()) => return Ok(()),
                    Err(evidence) => final_window = evidence,
                }
            }
        }
        if self.suffix && window.items.len() == expected_length {
            match evaluate_window(&window.items, window.consumed) {
                Ok(()) => return Ok(()),
                Err(evidence) => final_window = evidence,
            }
        }
        Err((final_window, window.consumed, expected_length))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let relation = if self.suffix {
            "ends with these positions"
        } else {
            "contains these contiguous positions"
        };
        let (evidence, consumed, expected) = rejection;
        let failure = explain_scan(
            failure,
            context,
            evidence,
            if self.suffix {
                "does not end with matching positions"
            } else {
                "does not contain matching contiguous positions"
            },
            consumed,
        );
        if consumed < expected {
            // Describe incomplete windows without evaluating their matchers or inventing positions.
            failure
                .fact(Fact::labelled(
                    "Expected length",
                    context.render().value(&expected),
                ))
                .constraint(
                    context
                        .describe_list::<T, _, _>(
                            self.expected,
                            FailureBuilder::detached::<()>(FailureKind::Matching)
                                .relation(relation),
                        )
                        .build(),
                )
        } else {
            failure
        }
    }
}

struct Items<'a, T, I> {
    items: &'a [I],
    view: PhantomData<T>,
}

impl<T, I> crate::assertions::HasLength for Items<'_, T, I> {
    fn length(&self) -> usize {
        self.items.len()
    }
}

impl<T, I: Borrow<T>> crate::assertions::collection::Collection for Items<'_, T, I> {
    type Item = T;
    const PRESENTATION: crate::renderer::CollectionPresentation =
        crate::renderer::CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(Borrow::borrow)
    }
}

enum UnorderedRejection {
    KnownLength { actual: usize, expected: usize },
    Mismatch { evidence: Evidence, consumed: usize },
}

struct ElementsAreInAnyOrder<'e, T, L> {
    expected: &'e L,
    item: PhantomData<fn() -> T>,
}

impl<T, L, I, R> Scan<I, R> for ElementsAreInAnyOrder<'_, T, L>
where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = UnorderedRejection;
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let expected_length = self.expected.len();
        if let Some(actual) = exact_size_hint(&iterator)
            && actual != expected_length
        {
            return Err(UnorderedRejection::KnownLength {
                actual,
                expected: expected_length,
            });
        }
        let items = iterator
            .take(expected_length.saturating_add(1))
            .collect::<Vec<_>>();
        let actual = Items {
            items: &items,
            view: PhantomData::<T>,
        };
        let mut child = context.isolated();
        if child.evaluate(
            &actual,
            &crate::assertions::collection::elements_are_in_any_order(self.expected),
        ) {
            Ok(())
        } else {
            Err(UnorderedRejection::Mismatch {
                evidence: child.into_evidence(),
                consumed: items.len(),
            })
        }
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejection {
            UnorderedRejection::KnownLength { actual, expected } => failure
                .relation("does not have the required number of elements")
                .fact(Fact::labelled(
                    "Reported length",
                    context.render().value(&actual),
                ))
                .fact(Fact::labelled(
                    "Expected length",
                    context.render().value(&expected),
                )),
            UnorderedRejection::Mismatch { evidence, consumed } => explain_scan(
                failure,
                context,
                evidence,
                "does not match exactly in any order",
                consumed,
            ),
        }
    }
}

#[track_caller]
pub(crate) fn membership<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &P,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: ExpectationDiagnostics<T, R>,
    R: ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ContainsMatching::<T, P> {
            expected,
            item: PhantomData,
            positions,
        },
    );
}

#[track_caller]
pub(crate) fn no_membership<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &P,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: ExpectationDiagnostics<T, R>,
    R: ValueRenderer<usize> + ValueRenderer<T>,
{
    execute(
        this,
        iterator,
        &ContainsNoMatching::<T, P> {
            expected,
            item: PhantomData,
            positions,
        },
    );
}

#[track_caller]
pub(crate) fn exact_or_prefix<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &L,
    exact: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ElementsAre::<T, L> {
            expected,
            item: PhantomData,
            exact,
        },
    );
}

#[track_caller]
pub(crate) fn suffix_or_contiguous<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &L,
    suffix: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &MatchingWindow::<T, L> {
            expected,
            item: PhantomData,
            suffix,
        },
    );
}

#[track_caller]
pub(crate) fn unordered<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &L,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ElementsAreInAnyOrder::<T, L> {
            expected,
            item: PhantomData,
        },
    );
}

#[cfg(test)]
mod tests {
    use crate::{
        expectation::ExpectationDiagnostics,
        prelude::*,
        renderer::{Rendered, RenderedBody},
        test_support::CustomValueRenderer,
    };
    use core::cell::Cell;

    use crate::failure::{FailureBuilder, FailureKind};

    mod evidence_budget {
        use super::*;
        use crate::{failure::PathSegment, matchers::eq};

        #[test]
        fn membership_completion_counts_only_rejected_candidates_or_the_empty_fallback() {
            for count in [0, 1, 2, 3, 20] {
                for budget in [0, 1, 32] {
                    let failures = assert_that_owned!(0..count)
                        .with_rendering_budget(RenderingBudget::default().with_max_items(budget))
                        .capture(|it| it.contains_matching(eq(99)));
                    let retained = count.clamp(1, 16).min(budget);
                    assert_that!(failures).has_length(1);
                    assert_that!(failures[0].children).has_length(retained);
                    assert_that!(failures[0].omitted_children).is_equal_to(count.max(1) - retained);
                    assert_that!(ToHumanReadableText.render(&failures[0]).as_str())
                        .does_not_contain("Preview starts at");
                }
            }
        }

        #[test]
        fn contiguous_retains_the_final_window_and_keeps_evaluating_after_budget_exhaustion() {
            let failures = assert_that_owned!(0..30)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .capture(|it| it.contains_contiguous_matching([eq(99), eq(99)]));
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::Index(28)]);
            assert_that!(failures[0].omitted_children).is_equal_to(1);
            assert_that_owned!(0..)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .contains_contiguous_matching([eq(20), eq(21)]);
        }

        #[test]
        fn membership_limits_rejections_after_selecting_the_preview() {
            let failures = assert_that_owned!(0..20)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.contains_matching(eq(99)));

            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::Index(4)]);
            assert_that!(failures[0].omitted_children).is_equal_to(19);
        }

        #[test]
        fn suffix_limits_repeated_position_evidence() {
            let failures = assert_that_owned!([1, 2, 3].into_iter())
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.ends_with_matching([eq(0), eq(0), eq(0)]));

            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::Index(0)]);
            assert_that!(failures[0].omitted_children).is_equal_to(2);
        }
    }

    mod windows {
        use super::*;

        #[test]
        fn wraparound_and_overlapping_windows_preserve_matching_and_stopping_points() {
            for length in [1, 3, 16, 17, 40] {
                let expected = (0..length).map(matchers::eq).collect::<Vec<_>>();
                let consumed = Cell::new(0);
                let actual = core::iter::repeat_n(99, 77)
                    .chain(0..)
                    .inspect(|_| consumed.set(consumed.get() + 1));
                assert_that_owned!(actual).contains_contiguous_matching(&expected);
                assert_that!(consumed.get()).is_equal_to(77 + length);
                assert_that_owned!(core::iter::repeat_n(99, 77).chain(0..length))
                    .ends_with_matching(expected);
            }
            assert_that_owned!([1, 1, 1, 2].into_iter()).contains_contiguous_matching([
                matchers::eq(1),
                matchers::eq(1),
                matchers::eq(2),
            ]);
        }

        #[test]
        fn non_fused_windows_stop_at_first_exhaustion() {
            for suffix in [false, true] {
                let mut yielded = [Some(1), None, Some(2)].into_iter();
                let mut iterator = core::iter::from_fn(|| yielded.next().flatten());
                let expected = [matchers::eq(1), matchers::eq(2)];
                let scan = super::super::MatchingWindow::<i32, _> {
                    expected: &expected,
                    item: core::marker::PhantomData,
                    suffix,
                };
                let (_, consumed, _) =
                    super::super::Scan::observe(&scan, &mut iterator, &AssertionContext::default())
                        .err()
                        .unwrap();
                assert_that!(consumed).is_equal_to(1);
                assert_that!(iterator.next()).is_equal_to(Some(2));
            }
        }
    }

    struct DescriptionOnly<'a> {
        expected: i32,
        descriptions: &'a Cell<usize>,
    }

    impl<R> Expectation<i32, R> for DescriptionOnly<'_> {
        type Success<'a>
            = ()
        where
            Self: 'a;
        type Rejection<'a>
            = ()
        where
            Self: 'a;
        fn evaluate(&self, _: &i32, _: &AssertionContext<'_, R>) -> Result<(), ()> {
            panic!("an incomplete window must not evaluate matchers")
        }
    }
    impl<R: ValueRenderer<i32>> ExpectationDiagnostics<i32, R> for DescriptionOnly<'_> {
        const KIND: FailureKind = FailureKind::Matching;
        fn explain<Target>(
            &self,
            rejected: Option<(&i32, ())>,
            failure: FailureBuilder<Target>,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder<Target> {
            let render = context.render();
            match rejected {
                None => {
                    self.descriptions.set(self.descriptions.get() + 1);
                    failure
                        .relation("is equal to")
                        .expected(render.value(&self.expected))
                }
                Some((_, ())) => {
                    unreachable!("the test cannot return a rejection")
                }
            }
        }
    }

    fn assert_truncated_value(value: &Rendered, type_name: &str, omitted_characters: usize) {
        assert_that!(value.type_name()).is_equal_to(Some(type_name));
        assert_that!(value.body()).is_equal_to(RenderedBody::Text {
            text: "cus".into(),
            omitted_characters,
        });
    }

    fn assert_truncated_lengths(failure: &AssertionFailure) {
        for label in ["Consumed", "Expected length"] {
            let fact = failure
                .facts()
                .iter()
                .find(|fact| fact.label() == label)
                .unwrap();
            assert_truncated_value(fact.value(), "usize", 6);
        }
    }

    #[test]
    fn missing_positions_respect_renderer_and_budget_without_evaluating_matchers() {
        for exact in [true, false] {
            for maximum in 0..=2 {
                let evaluations = Cell::new(0);
                let descriptions = Cell::new(0);
                let later_descriptions = Cell::new(0);
                let matchers = (
                    crate::expectation::predicate(|actual: &i32| {
                        evaluations.set(evaluations.get() + 1);
                        *actual == 1
                    }),
                    DescriptionOnly {
                        expected: 987_654,
                        descriptions: &descriptions,
                    },
                    DescriptionOnly {
                        expected: 10,
                        descriptions: &later_descriptions,
                    },
                );
                let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                    .with_renderer(CustomValueRenderer)
                    .with_rendering_budget(
                        RenderingBudget::default()
                            .with_max_items(maximum)
                            .with_max_leaf_characters(3),
                    )
                    .capture(|it| {
                        if exact {
                            it.contains_exactly_matching(matchers)
                        } else {
                            it.starts_with_matching(matchers)
                        }
                    });

                let retained = maximum.min(1);
                assert_that!(evaluations.get()).is_equal_to(1);
                assert_that!(descriptions.get()).is_equal_to(retained);
                assert_that!(later_descriptions.get()).is_equal_to(0);
                assert_that!(failures).has_length(1);
                let failure = &failures[0];
                assert_that!(failure.omitted_children).is_equal_to(1 - retained);
                assert_that!(failure.children).has_length(retained);
                for child in &failure.children {
                    assert_that!(child.path).is_equal_to([crate::failure::PathSegment::Index(1)]);
                    let constraint = child.constraint().unwrap();
                    assert_that!(constraint.relation()).is_equal_to(Some("is equal to"));
                    assert_truncated_value(constraint.expected.as_ref().unwrap(), "i32", 11);
                }
                assert_truncated_lengths(failure);
            }
        }
    }

    #[test]
    fn short_windows_respect_renderer_and_budget_without_evaluating_matchers() {
        for suffix in [true, false] {
            for maximum in 0..=2 {
                let descriptions = Cell::new(0);
                let matchers = [9, 10].map(|expected| DescriptionOnly {
                    expected,
                    descriptions: &descriptions,
                });
                let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                    .with_renderer(CustomValueRenderer)
                    .with_rendering_budget(
                        RenderingBudget::default()
                            .with_max_items(maximum)
                            .with_max_leaf_characters(3),
                    )
                    .capture(|it| {
                        if suffix {
                            it.ends_with_matching(matchers)
                        } else {
                            it.contains_contiguous_matching(matchers)
                        }
                    });

                assert_that!(descriptions.get()).is_equal_to(maximum);
                assert_that!(failures).has_length(1);
                let failure = &failures[0];
                let constraint = failure.constraint().unwrap();
                assert_that!(constraint.omitted_children).is_equal_to(2 - maximum);
                assert_that!(constraint.children).has_length(maximum);
                for (index, child) in constraint.children.iter().enumerate() {
                    assert_truncated_value(child.expected.as_ref().unwrap(), "i32", 6 + index);
                }
                assert_truncated_lengths(failure);
            }
        }
    }

    #[test]
    fn short_scans_describe_the_missing_constraints_and_observed_length() {
        use crate::test_support::assert_custom_fact;

        // Prefix/exact retain the first missing slot. Suffix/contiguous describe the full
        // missing window. Both cases use the same diagnostics for empty and short iterators.
        for consumed in 0..=1 {
            for operation in 0..4 {
                let expected = [matchers::eq(1), matchers::eq(987_654)];
                let mut values = (0..consumed).map(|_| 1);
                let iterator = core::iter::from_fn(move || values.next());
                let failures = assert_that_owned!(iterator)
                    .with_renderer(CustomValueRenderer)
                    .capture(|it| match operation {
                        0 => it.starts_with_matching(expected),
                        1 => it.contains_exactly_matching(expected),
                        2 => it.ends_with_matching(expected),
                        _ => it.contains_contiguous_matching(expected),
                    });
                assert_that!(failures).has_length(1);
                let failure = &failures[0];
                assert_custom_fact(failure, "Consumed", consumed);
                assert_custom_fact(failure, "Expected length", 2);
                if operation < 2 {
                    assert_that!(failure.relation())
                        .is_equal_to(Some("is missing a matching position"));
                    assert_that!(failure.children).has_length(1);
                    let child = &failure.children[0];
                    assert_that!(child.path)
                        .is_equal_to([crate::failure::PathSegment::Index(consumed)]);
                    crate::test_support::assert_custom_value(
                        child.constraint().unwrap().expected.as_ref().unwrap(),
                        &if consumed == 0 { 1_i32 } else { 987_654_i32 },
                    );
                } else {
                    let constraint = failure.constraint().unwrap();
                    assert_that!(constraint.relation()).is_equal_to(Some(if operation == 2 {
                        "ends with these positions"
                    } else {
                        "contains these contiguous positions"
                    }));
                    assert_that!(constraint.children).has_length(2);
                    for (child, value) in constraint.children.iter().zip([1_i32, 987_654]) {
                        crate::test_support::assert_custom_value(
                            child.expected.as_ref().unwrap(),
                            &value,
                        );
                    }
                }
            }
        }
    }
}
