use super::{AssertrMatcher, Description, MatchContext, MatchResult};
use crate::{
    ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};

/// Ordinary heterogeneous equality.
pub struct EqualTo<E>(E);

/// Matches through the actual value's ordinary `PartialEq` implementation.
pub fn equal_to<E>(expected: E) -> EqualTo<E> {
    EqualTo(expected)
}

impl<A, R, E> AssertrMatcher<A, R> for EqualTo<E>
where
    A: PartialEq<E> + ?Sized,
    R: ValueRenderer<A> + ValueRenderer<E>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        Description::new("is equal to").expected(context.render().value(&self.0))
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let matched = equals(actual, &self.0);
        if matched != context.is_positive() {
            if context.is_diagnostic() {
                let failure = FailureBuilder::detached::<A>(FailureKind::Equality)
                    .actual(context.render().value(actual));
                context.record(
                    if matched {
                        failure
                            .relation("is equal to")
                            .unexpected(context.render().value(&self.0))
                    } else {
                        failure.expected(context.render().value(&self.0))
                    }
                    .build(),
                );
            } else {
                context.outcome(matched, |context| {
                    <Self as AssertrMatcher<A, R>>::describe(self, context)
                });
            }
        }
        MatchResult::new(matched)
    }
}

pub(crate) fn equals<A, E: ?Sized>(actual: &A, expected: &E) -> bool
where
    A: PartialEq<E> + ?Sized,
{
    actual.eq(expected)
}

#[cfg(test)]
mod tests {
    use super::equal_to;
    use crate::prelude::*;

    #[test]
    fn preserves_nan_equality() {
        assert_that!(f64::NAN).does_not_match(equal_to(f64::NAN));
        assert_that!(f64::NAN).is_not_equal_to(f64::NAN);
    }
}
