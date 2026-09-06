use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, MatcherList};
use crate::{
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
            if (actual_length < expected_length
                || (matches!(self.position, Position::Exact) && actual_length != expected_length))
                && window.is_diagnostic()
            {
                window.record(
                    FailureBuilder::detached::<C>(FailureKind::Matching)
                        .relation("does not have the required sequence")
                        .fact("actual length", actual_length)
                        .fact("expected length", expected_length)
                        .build(),
                );
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
}
