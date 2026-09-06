use super::{AssertrMatcher, Description, MatchContext, MatchResult};

/// An explicit matcher selection for macro shorthand.
pub struct AsMatcher<M>(M);

/// Selects matcher interpretation when a value also supports equality.
pub fn as_matcher<M>(matcher: M) -> AsMatcher<M> {
    AsMatcher(matcher)
}

impl<A: ?Sized, R, M> AssertrMatcher<A, R> for AsMatcher<M>
where
    M: AssertrMatcher<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        self.0.describe(context)
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        self.0.evaluate(actual, context)
    }
}

#[cfg(test)]
mod tests {
    use super::as_matcher;
    use crate::{
        matchers::{MatchContext, predicate},
        prelude::*,
        test_support::NoRenderer,
    };

    #[test]
    fn preserves_inner_evidence_without_requiring_a_renderer() {
        let matcher = as_matcher(predicate(|value: &i32| *value > 0).described_as("is positive"));
        let mut context = MatchContext::new(&NoRenderer, RenderingBudget::default());
        let description = matcher.describe(&context);
        let result = matcher.evaluate(&0, &mut context);
        let failures = context.into_failures();

        assert_that!(description.relation).is_equal_to("is positive");
        assert_that!(result.matched).is_false();
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].constraint.as_ref().unwrap().relation).is_equal_to("is positive");
    }
}
