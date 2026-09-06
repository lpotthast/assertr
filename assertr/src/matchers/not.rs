use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};

/// A negated constraint.
pub struct Not<M>(M);

/// Reverses truth and diagnostic polarity without replaying the inner matcher.
pub fn not<M>(matcher: M) -> Not<M> {
    Not(matcher)
}

impl<A: ?Sized, R, M> AssertrMatcher<A, R> for Not<M>
where
    M: AssertrMatcher<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("does not match").children([self.0.describe(context)])
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut fork = context.fork();
        let positive = fork.is_positive();
        fork.set_positive(!positive);
        let result = self.0.evaluate(actual, &mut fork);
        fork.commit();
        MatchResult::new(!result.matched)
    }
}

#[cfg(test)]
mod tests {
    use super::not;
    use crate::{matchers::equal_to, prelude::*};

    #[test]
    fn reverses_nested_polarity() {
        assert_that!(2).matches(not(not(equal_to(2))));
        assert_that!(2).does_not_match(not(equal_to(2)));
    }
}
