//! Matcher execution over bounded, single-pass iterator scans.

use super::{
    AssertThat, AssertionContext, Borrow, ExpectationDiagnostics, Mode, PREVIEW_CAPACITY,
    PhantomData, PositionReporting, Scan, Vec, VecDeque, exact_size_hint, execute,
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
    evidence.explain(
        failure
            .relation(relation)
            .fact(Fact::labelled(
                "Consumed",
                context.render().value(&consumed),
            ))
            .fact(Fact::labelled(
                "Preview starts at",
                context
                    .render()
                    .value(&consumed.saturating_sub(PREVIEW_CAPACITY)),
            )),
    )
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
        child.evidence.omitted = discarded;
        for evidence in retained {
            child.append(evidence);
        }
        if child.evidence.children.is_empty() {
            child.outcome(false, |child| child.describe(self.expected));
        }
        Err((child.into_evidence(), consumed))
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
        let mut window = VecDeque::new();
        let mut consumed = 0;
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
            if window.len() == expected_length {
                window.pop_front();
            }
            window.push_back(item);
            consumed += 1;
            if !self.suffix && window.len() == expected_length {
                match evaluate_window(&window, consumed) {
                    Ok(()) => return Ok(()),
                    Err(evidence) => final_window = evidence,
                }
            }
        }
        if self.suffix && window.len() == expected_length {
            match evaluate_window(&window, consumed) {
                Ok(()) => return Ok(()),
                Err(evidence) => final_window = evidence,
            }
        }
        Err((final_window, consumed, expected_length))
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
    view: core::marker::PhantomData<T>,
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
        Fact,
        expectation::ExpectationDiagnostics,
        prelude::*,
        renderer::{Rendered, RenderedBody},
        test_support::CustomValueRenderer,
    };
    use core::cell::Cell;

    use crate::failure::{FailureBuilder, FailureKind};

    struct DescriptionOnly<'a> {
        expected: i32,
        descriptions: &'a Cell<usize>,
    }

    impl<R> crate::Expectation<i32, R> for DescriptionOnly<'_> {
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

    fn assert_truncated_value(
        value: &AssertThat<Rendered, Capture>,
        type_name: &str,
        omitted_characters: usize,
    ) {
        value
            .derive_owned(Rendered::type_name)
            .is_some_satisfying(|name| {
                name.is_equal_to(type_name);
            });
        value
            .derive(Rendered::body)
            .is_equal_to(RenderedBody::Text {
                text: "cus".into(),
                omitted_characters,
            });
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
                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive(|failure| &failure.omitted_children)
                            .is_equal_to(1 - retained);
                        failure
                            .derive_owned(AssertionFailure::children)
                            .contains_exactly_satisfying(
                                (0..retained)
                                    .map(|_| {
                                        |child: AssertThat<AssertionFailure, Capture>| {
                                            child.derive(|child| &child.path).is_equal_to([
                                                crate::failure::PathSegment::Index(1),
                                            ]);
                                            child
                                                .derive_owned(AssertionFailure::constraint)
                                                .is_some_satisfying(|constraint| {
                                                    constraint
                                                        .derive_owned(|constraint| {
                                                            constraint.relation()
                                                        })
                                                        .is_equal_to(Some("is equal to"));
                                                    constraint
                                                        .derive(|constraint| &constraint.expected)
                                                        .is_some_satisfying(|expected| {
                                                            assert_truncated_value(
                                                                &expected, "i32", 11,
                                                            );
                                                        });
                                                });
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                            );
                        for label in ["Consumed", "Expected length"] {
                            failure
                                .derive_owned(AssertionFailure::facts)
                                .contains_satisfying(|fact| {
                                    fact.derive_owned(Fact::label).is_equal_to(label);
                                    assert_truncated_value(&fact.derive(Fact::value), "usize", 6);
                                });
                        }
                    },
                ]);
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
                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive_owned(AssertionFailure::constraint)
                            .is_some_satisfying(|constraint| {
                                constraint
                                    .derive(|constraint| &constraint.omitted_children)
                                    .is_equal_to(2 - maximum);
                                constraint
                                    .derive(|constraint| &constraint.children)
                                    .contains_exactly_satisfying(
                                        (0..maximum)
                                            .map(|index| {
                                                move |child: AssertThat<
                                                    crate::AssertionFailure,
                                                    Capture,
                                                >| {
                                                    child
                                                        .derive(|child| &child.expected)
                                                        .is_some_satisfying(|expected| {
                                                            assert_truncated_value(
                                                                &expected,
                                                                "i32",
                                                                6 + index,
                                                            );
                                                        });
                                                }
                                            })
                                            .collect::<Vec<_>>(),
                                    );
                            });
                        for label in ["Consumed", "Expected length"] {
                            failure
                                .derive_owned(AssertionFailure::facts)
                                .contains_satisfying(|fact| {
                                    fact.derive_owned(Fact::label).is_equal_to(label);
                                    assert_truncated_value(&fact.derive(Fact::value), "usize", 6);
                                });
                        }
                    },
                ]);
            }
        }
    }
}
