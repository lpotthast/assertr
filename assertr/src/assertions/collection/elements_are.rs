use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, Fact,
    assertions::collection::StableOrder,
    expectation::{Evidence, MatcherList},
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};
use alloc::vec::Vec;

/// Positional policy for a matcher sequence.
#[derive(Clone, Copy)]
enum Position {
    Exact,
    Prefix,
    Suffix,
    Contiguous,
}

/// A sequence constraint requiring stable element order.
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

/// Matches the final positions.
pub fn ends_with_elements<L>(list: L) -> ElementsAre<L> {
    ElementsAre {
        list,
        position: Position::Suffix,
    }
}

/// Matches a contiguous window.
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
        let context = settings.isolated();
        let actual_length = actual.length();
        let expected_length = self.list.len();
        let mut elements = actual.elements();
        // Only contiguous searches revisit elements across multiple candidate windows.
        let buffered = if matches!(self.position, Position::Contiguous) {
            elements.by_ref().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let starts = match self.position {
            Position::Exact | Position::Prefix => 0..1,
            Position::Suffix => {
                actual_length.saturating_sub(expected_length)
                    ..actual_length.saturating_sub(expected_length) + 1
            }
            Position::Contiguous => 0..actual_length.saturating_sub(expected_length) + 1,
        };
        let mut alternatives = context.isolated();
        for start in starts {
            let mut window = context.isolated();
            let mut matched = actual_length >= expected_length;
            if matches!(self.position, Position::Exact) {
                matched &= actual_length == expected_length;
            }
            let mut window_elements = elements.by_ref().skip(start);
            for index in 0..expected_length {
                let item = if matches!(self.position, Position::Contiguous) {
                    buffered.get(start + index).copied()
                } else {
                    window_elements.next()
                };
                if let Some(item) = item {
                    matched &= window.scoped(PathSegment::Index(start + index), |context| {
                        self.list.evaluate_at(index, item, context)
                    });
                } else {
                    window.scoped(PathSegment::Index(start + index), |context| {
                        context.outcome(false, |context| self.list.describe_at(index, context));
                    });
                }
            }
            if actual_length < expected_length
                || (matches!(self.position, Position::Exact) && actual_length != expected_length)
            {
                if window.is_diagnostic() {
                    window.record(
                        FailureBuilder::detached::<C>(FailureKind::Matching)
                            .relation("does not have the required sequence")
                            .fact(Fact::labelled(
                                "actual length",
                                window.render().value(&actual_length).into_rendered(),
                            ))
                            .fact(Fact::labelled(
                                "expected length",
                                window.render().value(&expected_length).into_rendered(),
                            ))
                            .build(),
                    );
                } else {
                    window.outcome(false, |context| context.describe::<C, _>(self));
                }
            }
            if matched {
                return Ok(());
            }
            alternatives.append(window.into_evidence());
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
                    Position::Exact => "has exactly these positions",
                    Position::Prefix => "starts with these positions",
                    Position::Suffix => "ends with these positions",
                    Position::Contiguous => "contains these contiguous positions",
                }),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
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
    fn supports_explicit_equality() {
        assert_that!([1, 2]).matches(elements_are![eq(1), eq(2)]);
    }

    #[test]
    fn supports_heterogeneous_equality() {
        assert_that!([String::from("hello")]).matches(elements_are![eq("hello")]);
        let failures = assert_that!([String::from("hello")])
            .capture(|it| it.matches(elements_are![eq("world")]));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children[0].kind)
            .is_equal_to(crate::failure::FailureKind::Equality);
    }

    #[test]
    fn matches_empty_sequences() {
        assert_that!([0; 0]).matches(elements_are![]);
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
                  - actual length: cus... 6 more characters ...
                  - expected length: cus... 6 more characters ...
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
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = AssertionContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are![])).is_false();
            assert_that!(context.into_evidence().children).is_empty();
        }
    }
}
