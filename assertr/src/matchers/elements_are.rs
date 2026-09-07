use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, MatcherList};
use crate::{
    Fact,
    assertions::collection::StableOrder,
    failure::{FailureBuilder, FailureKind, PathSegment},
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

impl<C, R, L> AssertrMatcher<C, R> for ElementsAre<L>
where
    R: crate::ValueRenderer<usize>,
    C: StableOrder + ?Sized,
    L: MatcherList<C::Item, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new(match self.position {
            Position::Exact => "has exactly these positions",
            Position::Prefix => "starts with these positions",
            Position::Suffix => "ends with these positions",
            Position::Contiguous => "contains these contiguous positions",
        })
        .omitted_children(self.list.len().saturating_sub(context.render().max_items()))
        .children(
            (0..self.list.len().min(context.render().max_items()))
                .map(|index| self.list.describe_at(index, context)),
        )
    }

    fn evaluate(&self, actual: &C, context: &mut MatchContext<'_, R>) -> MatchResult {
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
                    matched &= window
                        .scoped(PathSegment::Index(start + index), |context| {
                            self.list.evaluate_at(index, item, context)
                        })
                        .matched;
                } else if context.is_positive() {
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
                                window.render().value(&actual_length),
                            ))
                            .fact(Fact::labelled(
                                "expected length",
                                window.render().value(&expected_length),
                            ))
                            .build(),
                    );
                } else if context.is_positive() {
                    window.outcome(false, |context| {
                        <Self as AssertrMatcher<C, R>>::describe(self, context)
                    });
                }
            }
            if matched {
                if !context.is_positive() {
                    if window.evidence.is_empty() && window.omitted == 0 {
                        window.outcome(true, |context| {
                            <Self as AssertrMatcher<C, R>>::describe(self, context)
                        });
                    }
                    context.append(window);
                }
                return MatchResult::new(true);
            }
            alternatives.append(window);
        }
        if context.is_positive() {
            context.append(alternatives);
        }
        MatchResult::new(false)
    }
}

/// Exact positional matcher list with equality shorthand.
#[macro_export]
macro_rules! elements_are {
    ($($value:expr),* $(,)?) => {
        $crate::matchers::elements_are($crate::matchers![$($value),*])
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
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
    fn supports_equality_shorthand() {
        assert_that!([1, 2]).matches(elements_are![1, 2]);
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
            .with_rendering_budget(RenderingBudget::builder().max_items(0).build())
            .capture(|it| it.matches(elements_are![9, 9, 9]));

        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children).is_empty();
        assert_that!(failures[0].omitted_children).is_greater_or_equal_to(3);
        assert_that!(renders.get()).is_equal_to(0);
    }

    #[test]
    fn small_budget_renders_only_retained_leaves() {
        let renders = Cell::new(0);
        let failures = assert_that!([1, 2, 3])
            .with_renderer(CountingRenderer(&renders))
            .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
            .capture(|it| it.matches(elements_are![9, 9, 9]));

        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children).has_length(1);
        assert_that!(renders.get()).is_equal_to(2);
    }

    mod evaluate {
        use super::*;
        use crate::{matchers::all_of, renderer::RenderedBody, test_support::CustomValueRenderer};
        #[test]
        fn preserves_sequence_length_metadata_and_budget_in_nested_failures() {
            use indoc::formatdoc;

            let failures = assert_that!([1, 2])
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(3).build())
                .capture(|it| it.matches(all_of((elements_are![1],))));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
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

            let child = &failures[0].children[0];
            assert_eq!(child.facts.len(), 2);
            for fact in &child.facts {
                assert_eq!(fact.value.type_name, Some("usize"));
                assert_eq!(
                    fact.value.body,
                    RenderedBody::Text {
                        text: "cus".into(),
                        omitted_characters: 6
                    }
                );
            }
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
                .capture(|it| it.matches(elements_are![]));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
        -------- assertr --------
        Expression: `[1, 2]`

        does not match

        Details:
          - ... 1 more nested failure ...
        -------- assertr --------
    "});
            assert_that!(failures[0].omitted_children).is_equal_to(1);
        }
    }

    mod probe {
        use super::*;
        use crate::matchers::MatchContext;
        #[test]
        fn length_mismatches_do_not_render_numeric_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = MatchContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are![])).is_false();
            assert_that!(context.into_failures()).is_empty();
        }
    }
}
