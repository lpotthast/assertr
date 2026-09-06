use super::{AssertrMatcher, Description, MatchContext, MatchResult};
use crate::{
    ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};
use core::cmp::Ordering;

/// A partial-order lower bound.
pub struct GreaterOrEqual<E>(E);

/// Matches values greater than or equal to `expected`. Incomparable values do not match.
pub fn ge<E>(expected: E) -> GreaterOrEqual<E> {
    GreaterOrEqual(expected)
}

impl<A, R, E> AssertrMatcher<A, R> for GreaterOrEqual<E>
where
    A: PartialOrd<E> + ?Sized,
    R: ValueRenderer<A> + ValueRenderer<E>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        Description::new("is greater than or equal to").expected(context.render().value(&self.0))
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let matched = matches!(
            actual.partial_cmp(&self.0),
            Some(Ordering::Equal | Ordering::Greater)
        );
        if matched != context.is_positive() && context.is_diagnostic() {
            context.record(
                FailureBuilder::detached::<A>(FailureKind::Ordering)
                    .actual(context.render().value(actual))
                    .relation(if matched {
                        "is greater than or equal to"
                    } else {
                        "is not greater than or equal to"
                    })
                    .expected(context.render().value(&self.0))
                    .build(),
            );
        } else {
            context.outcome(matched, |context| {
                <Self as AssertrMatcher<A, R>>::describe(self, context)
            });
        }
        MatchResult::new(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::ge;
    use crate::prelude::*;

    #[test]
    fn incomparable_values_do_not_match() {
        assert_that!(f64::NAN).does_not_match(ge(0.0));
    }
}
