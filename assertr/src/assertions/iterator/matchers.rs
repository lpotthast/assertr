//! Matcher adapters over bounded, single-pass iterator scans.

use super::{
    AssertThat, Borrow, Mode, PREVIEW_CAPACITY, PositionReporting, Vec, VecDeque, exact_size_hint,
};
use crate::{
    Fact,
    failure::{Attached, FailureBuilder, FailureKind, PathSegment},
    matchers::{AssertrMatcher, ConstraintDescription, MatchContext, MatcherList},
};

#[track_caller]
fn failure<'a, S, M: Mode, R>(
    this: &'a AssertThat<'_, S, M, R>,
    context: MatchContext<'_, R>,
    relation: &'static str,
    consumed: usize,
) -> FailureBuilder<Attached<'a>>
where
    R: crate::ValueRenderer<usize>,
{
    this.failure(FailureKind::Matching)
        .relation(relation)
        .fact(Fact::labelled("Consumed", this.render().value(&consumed)))
        .fact(Fact::labelled(
            "Preview starts at",
            consumed.saturating_sub(PREVIEW_CAPACITY),
        ))
        .omitted_children(context.omitted_children())
        .children(context.into_failures())
}

#[track_caller]
pub(crate) fn membership<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    matcher: &P,
    positive: bool,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: AssertrMatcher<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let mut retained: VecDeque<MatchContext<'_, R>> = VecDeque::new();
    let mut consumed = 0;
    let mut discarded = 0;
    for item in iterator {
        let mut context = MatchContext::for_assertion(this);
        context.set_positive(positive);
        let accepted = if let Some(index) = positions.index(consumed) {
            context
                .scoped(PathSegment::Index(index), |context| {
                    matcher.evaluate(item.borrow(), context)
                })
                .matched
        } else {
            matcher.evaluate(item.borrow(), &mut context).matched
        };
        consumed += 1;
        if accepted {
            if !positive {
                failure(
                    this,
                    context,
                    "contains an unexpected matching element",
                    consumed,
                )
                .raise();
            }
            return;
        }
        if retained.len() == PREVIEW_CAPACITY
            && let Some(old) = retained.pop_front()
        {
            discarded += old.evidence.len() + old.omitted;
        }
        retained.push_back(context);
    }
    if positive {
        let mut context = MatchContext::for_assertion(this);
        context.omitted = discarded;
        for child in retained {
            context.append(child);
        }
        if context.evidence.is_empty() {
            context.outcome(false, |context| matcher.describe(context));
        }
        failure(
            this,
            context,
            "does not contain a matching element",
            consumed,
        )
        .raise();
    }
}

#[track_caller]
pub(crate) fn exact_or_prefix<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    list: &L,
    exact: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    let mut context = MatchContext::for_assertion(this);
    if let Some(actual) = exact_size_hint(&iterator)
        && ((exact && actual != expected_length) || (!exact && actual < expected_length))
    {
        this.failure(FailureKind::Matching)
            .relation("does not have the required sequence length")
            .fact(Fact::labelled(
                "Reported length",
                this.render().value(&actual),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_length),
            ))
            .raise();
        return;
    }
    for index in 0..expected_length {
        let Some(item) = iterator.next() else {
            context.scoped(PathSegment::Index(index), |context| {
                context.outcome(false, |context| list.describe_at(index, context));
            });
            failure(this, context, "is missing a matching position", index)
                .fact(Fact::labelled(
                    "Expected length",
                    this.render().value(&expected_length),
                ))
                .raise();
            return;
        };
        if !context
            .scoped(PathSegment::Index(index), |context| {
                list.evaluate_at(index, item.borrow(), context)
            })
            .matched
        {
            failure(
                this,
                context,
                "does not match the required position",
                index + 1,
            )
            .raise();
            return;
        }
    }
    if exact && iterator.next().is_some() {
        failure(this, context, "has an extra element", expected_length + 1).raise();
    }
}

#[track_caller]
pub(crate) fn suffix_or_contiguous<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    list: &L,
    suffix: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    if expected_length == 0 {
        return;
    }
    let mut window = VecDeque::new();
    let mut consumed = 0;
    let mut final_window = MatchContext::for_assertion(this);
    for item in iterator {
        if window.len() == expected_length {
            window.pop_front();
        }
        window.push_back(item);
        consumed += 1;
        if !suffix && window.len() == expected_length {
            let mut context = MatchContext::for_assertion(this);
            let mut matched = true;
            for (slot, item) in window.iter().enumerate() {
                matched &= context
                    .scoped(
                        PathSegment::Index(consumed - expected_length + slot),
                        |context| list.evaluate_at(slot, item.borrow(), context),
                    )
                    .matched;
            }
            if matched {
                return;
            }
            final_window = context;
        }
    }
    if suffix && window.len() == expected_length {
        let mut matched = true;
        for (slot, item) in window.iter().enumerate() {
            matched &= final_window
                .scoped(
                    PathSegment::Index(consumed - expected_length + slot),
                    |context| list.evaluate_at(slot, item.borrow(), context),
                )
                .matched;
        }
        if matched {
            return;
        }
    }
    let mut failure = failure(
        this,
        final_window,
        if suffix {
            "does not end with matching positions"
        } else {
            "does not contain matching contiguous positions"
        },
        consumed,
    );
    if consumed < expected_length {
        // No complete candidate was available. Describe the expectations without evaluating
        // matchers against an incomplete window or inventing actual iterator positions.
        let context = MatchContext::for_assertion(this);
        let maximum = context.render().max_items();
        failure = failure
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_length),
            ))
            .constraint(
                ConstraintDescription::new(if suffix {
                    "ends with these positions"
                } else {
                    "contains these contiguous positions"
                })
                .omitted_children(expected_length.saturating_sub(maximum))
                .children(
                    (0..expected_length.min(maximum))
                        .map(|index| list.describe_at(index, &context)),
                ),
            );
    }
    failure.raise();
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

#[track_caller]
pub(crate) fn unordered<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    list: &L,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    if let Some(actual) = exact_size_hint(&iterator)
        && actual != expected_length
    {
        this.failure(FailureKind::Matching)
            .relation("does not have the required number of elements")
            .fact(Fact::labelled(
                "Reported length",
                this.render().value(&actual),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_length),
            ))
            .raise();
        return;
    }
    let items = iterator
        .take(expected_length.saturating_add(1))
        .collect::<Vec<_>>();
    let actual = Items {
        items: &items,
        view: core::marker::PhantomData::<T>,
    };
    let mut context = MatchContext::for_assertion(this);
    if !crate::matchers::elements_are_in_any_order(list)
        .evaluate(&actual, &mut context)
        .matched
    {
        failure(
            this,
            context,
            "does not match exactly in any order",
            items.len(),
        )
        .raise();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Fact,
        matchers::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult},
        prelude::*,
        renderer::{Rendered, RenderedBody},
        test_support::CustomValueRenderer,
    };
    use core::cell::Cell;

    struct DescriptionOnly<'a> {
        expected: i32,
        descriptions: &'a Cell<usize>,
    }

    impl<R: ValueRenderer<i32>> AssertrMatcher<i32, R> for DescriptionOnly<'_> {
        fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
            self.descriptions.set(self.descriptions.get() + 1);
            ConstraintDescription::new("is equal to")
                .expected(context.render().value(&self.expected))
        }

        fn evaluate(&self, _: &i32, _: &mut MatchContext<'_, R>) -> MatchResult {
            panic!("an incomplete window must not evaluate matchers")
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
                    crate::matchers::predicate(|actual: &i32| {
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
                        RenderingBudget::builder()
                            .max_items(maximum)
                            .max_leaf_characters(3)
                            .build(),
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
                                                        .derive(|constraint| &constraint.relation)
                                                        .is_equal_to("is equal to");
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
                        RenderingBudget::builder()
                            .max_items(maximum)
                            .max_leaf_characters(3)
                            .build(),
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
                                                    ConstraintDescription,
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
