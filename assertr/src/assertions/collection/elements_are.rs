use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, Fact,
    assertions::collection::StableOrder,
    expectation::{Evidence, MatcherList},
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};
use alloc::vec::Vec;

/// Positional policy for a matcher sequence.
#[derive(Debug, Clone, Copy)]
enum Position {
    Exact,
    Prefix,
    Suffix,
    Contiguous,
}

/// A sequence constraint requiring stable element order.
#[derive(Debug, Clone)]
pub struct ElementsAre<L> {
    list: L,
    position: Position,
}

/// Matches exactly the listed constraints in positional order.
pub fn elements_are<L>(list: L) -> ElementsAre<L> {
    ElementsAre {
        list,
        position: Position::Exact,
    }
}

/// Matches the initial positions.
pub fn starts_with_elements<L>(list: L) -> ElementsAre<L> {
    ElementsAre {
        list,
        position: Position::Prefix,
    }
}

/// Matches the final positions. Constraints align with the subject's end, so a shorter subject
/// reports the leading constraints as missing.
pub fn ends_with_elements<L>(list: L) -> ElementsAre<L> {
    ElementsAre {
        list,
        position: Position::Suffix,
    }
}

/// Matches a contiguous window.
///
/// A rejection retains one group per rejected candidate window. Each group carries a `Window start`
/// fact with the window's zero-based starting index.
pub fn contains_contiguous_elements<L>(list: L) -> ElementsAre<L> {
    ElementsAre {
        list,
        position: Position::Contiguous,
    }
}

impl<C: StableOrder + ?Sized, R, L> Expectation<C, R> for ElementsAre<L>
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
        let actual_length = actual.length();
        let expected_length = self.list.len();
        let mut elements = actual.elements();
        // Only contiguous searches revisit elements across multiple candidate windows.
        let buffered = if matches!(self.position, Position::Contiguous) {
            elements.by_ref().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        // A suffix aligns from the right. Leading slots of a longer suffix precede the subject.
        let unaligned = if matches!(self.position, Position::Suffix) {
            expected_length.saturating_sub(actual_length)
        } else {
            0
        };
        let starts = match self.position {
            Position::Exact | Position::Prefix => 0..1,
            Position::Suffix => {
                actual_length.saturating_sub(expected_length)
                    ..actual_length.saturating_sub(expected_length) + 1
            }
            Position::Contiguous => 0..actual_length.saturating_sub(expected_length) + 1,
        };
        let length_matches = if matches!(self.position, Position::Exact) {
            actual_length == expected_length
        } else {
            actual_length >= expected_length
        };
        let mut alternatives = settings.isolated();
        for start in starts {
            let mut window = alternatives.isolated();
            let mut matched = length_matches;
            let mut window_elements = elements.by_ref().skip(start);
            for index in 0..expected_length {
                if index < unaligned {
                    window.outcome(false, |context| self.list.describe_at(index, context));
                    continue;
                }
                let position = start + index - unaligned;
                let item = if matches!(self.position, Position::Contiguous) {
                    buffered.get(position).copied()
                } else {
                    window_elements.next()
                };
                window.scoped(PathSegment::Index(position), |context| {
                    if let Some(item) = item {
                        matched &= self.list.evaluate_at(index, item, context);
                    } else {
                        context.outcome(false, |context| self.list.describe_at(index, context));
                    }
                });
            }
            if !length_matches {
                window.record_with(|window| {
                    FailureBuilder::detached::<C>(FailureKind::Matching)
                        .relation("does not have the required sequence")
                        .fact(Fact::labelled(
                            "Actual length",
                            window.render().value(&actual_length).into_rendered(),
                        ))
                        .fact(Fact::labelled(
                            "Expected length",
                            window.render().value(&expected_length).into_rendered(),
                        ))
                        .build()
                });
            }
            if matched {
                return Ok(());
            }
            let evidence = window.into_evidence();
            if matches!(self.position, Position::Contiguous) {
                // Each rejected window stays one group, so its evidence does not interleave with
                // the evidence of overlapping windows.
                alternatives.record_with(|alternatives| {
                    evidence
                        .explain(
                            FailureBuilder::detached::<C>(FailureKind::Matching)
                                .relation("does not match in this window")
                                .fact(Fact::labelled(
                                    "Window start",
                                    alternatives.render().value(&start).into_rendered(),
                                )),
                        )
                        .build()
                });
            } else {
                alternatives.append(evidence);
            }
        }
        Err(alternatives.into_evidence())
    }
}
impl<C: StableOrder + ?Sized, R, L> ExpectationDiagnostics<C, R> for ElementsAre<L>
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
                &self.list,
                failure.relation(match self.position {
                    Position::Exact => "has exactly these elements in order",
                    Position::Prefix => "starts with these elements",
                    Position::Suffix => "ends with these elements",
                    Position::Contiguous => "contains these elements contiguously",
                }),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation(match self.position {
                Position::Contiguous => "does not contain these elements contiguously",
                Position::Exact | Position::Prefix | Position::Suffix => "does not match",
            })),
        }
    }
}

/// Exact positional matcher list with explicit expectations.
///
/// Use [`eq`](crate::matchers::eq) or [`equal_to`](crate::matchers::equal_to) for equality.
#[macro_export]
macro_rules! elements_are {
    ($($value:expr),* $(,)?) => {
        $crate::assertions::collection::elements_are($crate::matchers![$($value),*])
    };
}

#[cfg(test)]
mod tests {
    use crate::{matchers::eq, prelude::*};
    use core::{cell::Cell, fmt};

    struct CountingRenderer<'a>(&'a Cell<usize>);

    impl ValueRenderer<i32> for CountingRenderer<'_> {
        fn fmt(&self, value: &i32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            write!(formatter, "{value}")
        }
    }

    impl ValueRenderer<usize> for CountingRenderer<'_> {
        fn fmt(&self, value: &usize, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            write!(formatter, "{value}")
        }
    }

    #[test]
    fn positional_policies_cover_empty_short_reordered_and_overlapping_sequences() {
        use super::{
            ElementsAre,
            Position::{Contiguous, Exact, Prefix, Suffix},
        };

        // Outcomes are exact, prefix, suffix, and contiguous respectively. Exercise the
        // shared algorithm here. Public adapters retain diagnostics and boundary checks.
        let cases: &[(&[i32], &[i32], [bool; 4])] = &[
            (&[], &[], [true, true, true, true]),
            (&[1], &[], [false, true, true, true]),
            (&[], &[1], [false, false, false, false]),
            (&[1], &[1, 2], [false, false, false, false]),
            (&[1, 2], &[1, 2], [true, true, true, true]),
            (&[1, 2], &[2, 1], [false, false, false, false]),
            (&[1, 2, 3], &[1, 2], [false, true, false, true]),
            (&[1, 2, 3], &[2, 3], [false, false, true, true]),
            (&[0, 1, 2, 3], &[1, 2], [false, false, false, true]),
            (&[1, 2, 3], &[1, 3], [false, false, false, false]),
            (&[1, 1, 2], &[1, 2], [false, false, true, true]),
            (&[1, 1, 2], &[1, 2, 2], [false, false, false, false]),
        ];
        for &(actual, expected, outcomes) in cases {
            for (position, accepted) in [Exact, Prefix, Suffix, Contiguous]
                .into_iter()
                .zip(outcomes)
            {
                let matcher = ElementsAre {
                    list: expected.iter().copied().map(eq).collect::<Vec<_>>(),
                    position,
                };
                let failures = assert_that!(actual).capture(|it| it.matches(matcher));
                assert_that!(failures.is_empty()).is_equal_to(accepted);
            }
        }
    }

    #[test]
    fn short_suffixes_align_from_the_right() {
        use crate::assertions::collection::ends_with_elements;
        let failures = assert_that!([2, 3])
            .with_location(false)
            .capture(|it| it.matches(ends_with_elements([eq(1), eq(2), eq(3)])));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0]).has_text_report(indoc::indoc! {r"
            -------- assertr --------
            Expression: `[2, 3]`

            does not match

            Nested failures:
              - does not satisfy the constraint

                Constraint:
                    is equal to

                    Expected: 1
              - does not have the required sequence

                Details:
                  - Actual length: 2
                  - Expected length: 3
            -------- assertr --------
        "});
    }

    #[test]
    fn short_suffixes_report_aligned_mismatches_at_their_actual_positions() {
        use crate::assertions::collection::ends_with_elements;
        let failures = assert_that!([2, 4])
            .with_location(false)
            .capture(|it| it.matches(ends_with_elements([eq(1), eq(2), eq(3)])));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0]).has_text_report(indoc::indoc! {r"
            -------- assertr --------
            Expression: `[2, 4]`

            does not match

            Nested failures:
              - does not satisfy the constraint

                Constraint:
                    is equal to

                    Expected: 1
              - At [1]:
                Expected: 3

                  Actual: 4
              - does not have the required sequence

                Details:
                  - Actual length: 2
                  - Expected length: 3
            -------- assertr --------
        "});
    }

    #[test]
    fn supports_owned_string_equality() {
        assert_that!([String::from("hello")]).matches(elements_are![eq("hello")]);
        let failures = assert_that!([String::from("hello")])
            .capture(|it| it.matches(elements_are![eq("world")]));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children[0].kind)
            .is_equal_to(crate::failure::FailureKind::Equality);
    }

    #[test]
    fn zero_budget_preserves_truth_without_rendering_leaves() {
        let renders = Cell::new(0);
        let failures = assert_that!([1, 2, 3])
            .with_renderer(CountingRenderer(&renders))
            .with_rendering_budget(RenderingBudget::default().with_max_items(0))
            .capture(|it| it.matches(elements_are![eq(9), eq(9), eq(9)]));

        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element.derive(|value| &value.children).is_empty();
                element
                    .derive(|value| &value.omitted_children)
                    .is_greater_or_equal_to(3);
            },
        ]);
        assert_that!(renders.get()).is_equal_to(0);
    }

    #[test]
    fn small_budget_renders_only_retained_leaves() {
        let renders = Cell::new(0);
        let failures = assert_that!([1, 2, 3])
            .with_renderer(CountingRenderer(&renders))
            .with_rendering_budget(RenderingBudget::default().with_max_items(1))
            .capture(|it| it.matches(elements_are![eq(9), eq(9), eq(9)]));

        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element.derive(|value| &value.children).has_length(1);
            },
        ]);
        assert_that!(renders.get()).is_equal_to(2);
    }

    mod alternatives {
        use super::*;
        use crate::{assertions::collection::contains_contiguous_elements, failure::PathSegment};

        #[derive(Debug)]
        struct Compared<'a> {
            value: i32,
            comparisons: &'a Cell<usize>,
        }
        impl borrow_for::BorrowFor<Compared<'_>> for i32 {
            type View = i32;
        }
        impl PartialEq<i32> for Compared<'_> {
            fn eq(&self, other: &i32) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value == *other
            }
        }
        impl ValueRenderer<Compared<'_>> for CountingRenderer<'_> {
            fn fmt(&self, value: &Compared<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(f, "{}", value.value)
            }
        }

        #[test]
        fn completed_windows_share_the_remaining_allowance_without_changing_truth() {
            for later_success in [false, true] {
                let comparisons = Cell::new(0);
                let renders = Cell::new(0);
                let actual = (0..20)
                    .map(|index| Compared {
                        value: if later_success && index == 19 { 9 } else { 0 },
                        comparisons: &comparisons,
                    })
                    .collect::<Vec<_>>();
                let failures = assert_that!(actual)
                    .with_renderer(CountingRenderer(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                    .capture(|it| it.matches(contains_contiguous_elements([eq(9)])));
                assert_that!(comparisons.get()).is_equal_to(20);
                // Only the first window is retained. It renders its start, actual, and expected.
                assert_that!(renders.get()).is_equal_to(3);
                assert_that!(failures.len()).is_equal_to(usize::from(!later_success));
                if !later_success {
                    assert_that!(failures[0].children).has_length(1);
                    let window = &failures[0].children[0];
                    assert_that!(window.path).is_empty();
                    assert_that!(window.children).has_length(1);
                    assert_that!(window.children[0].path).contains_exactly([PathSegment::Index(0)]);
                    assert_that!(failures[0].omitted_children).is_equal_to(19);
                }
            }
        }
    }

    mod contiguous {
        use crate::{matchers::eq, prelude::*, test_support::FailureReportAssertions};

        #[test]
        fn groups_the_evidence_of_each_rejected_window() {
            let failures = assert_that!(vec![1, 3, 1, 4])
                .with_location(false)
                .capture(|it| it.contains_contiguous_matching(matchers![eq(1), eq(2)]));
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure.has_text_report(indoc::indoc! {r"
                    -------- assertr --------
                    Expression: `vec![1, 3, 1, 4]`

                    does not contain these elements contiguously

                    Nested failures:
                      - does not match in this window

                        Details:
                          - Window start: 0
                        Nested failures:
                          - At [1]:
                            Expected: 2

                              Actual: 3
                      - does not match in this window

                        Details:
                          - Window start: 1
                        Nested failures:
                          - At [1]:
                            Expected: 1

                              Actual: 3
                          - At [2]:
                            Expected: 2

                              Actual: 1
                      - does not match in this window

                        Details:
                          - Window start: 2
                        Nested failures:
                          - At [3]:
                            Expected: 2

                              Actual: 4
                    -------- assertr --------
                    "});
                },
            ]);
        }
    }

    mod evaluate {
        use super::*;
        use crate::{
            expectation::all_of, renderer::RenderedBody, test_support::CustomValueRenderer,
        };
        #[test]
        fn preserves_sequence_length_metadata_and_budget_in_nested_failures() {
            use indoc::formatdoc;

            let failures = assert_that!([1, 2])
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| it.matches(all_of((elements_are![eq(1)],))));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
            -------- assertr --------
            Expression: `[1, 2]`

            does not match

            Nested failures:
              - does not have the required sequence

                Details:
                  - Actual length: cus... 6 more characters ...
                  - Expected length: cus... 6 more characters ...
            -------- assertr --------
        "});
                },
            ]);

            let child = &failures[0].children[0];
            assert_that!(child.facts).contains_exactly_satisfying(
                [|fact: AssertThat<crate::Fact, Capture>| {
                    fact.derive(|fact| &fact.value.type_name)
                        .is_some_satisfying(|name| {
                            name.is_equal_to("usize");
                        });
                    fact.derive(|fact| &fact.value.body)
                        .is_equal_to(RenderedBody::Text {
                            text: "cus".into(),
                            omitted_characters: 6,
                        });
                }; 2],
            );
        }

        #[test]
        fn a_zero_item_budget_preserves_length_failure_without_rendering_evidence() {
            use indoc::formatdoc;
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                    panic!("rendered omitted evidence")
                }
            }
            let failures = assert_that!([1, 2])
                .with_renderer(NeverRender)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.matches(elements_are![]));
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
        use crate::AssertionContext;
        #[test]
        fn length_mismatches_do_not_render_numeric_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = AssertionContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are![])).is_false();
            assert_that!(context.into_evidence().children).is_empty();
        }
    }
}
