use super::{AssertrMatcher, Description, MatchContext, MatchResult};

/// An unconstrained wildcard.
pub struct Anything;

/// Always matches, without renderer requirements.
#[must_use]
pub const fn anything() -> Anything {
    Anything
}

impl<A: ?Sized, R> AssertrMatcher<A, R> for Anything {
    fn describe(&self, _: &MatchContext<'_, R>) -> Description {
        Description::new("is anything")
    }

    fn evaluate(&self, _: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        context.outcome(true, |context| {
            <Self as AssertrMatcher<A, R>>::describe(self, context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::anything;
    use crate::{prelude::*, test_support::NoRenderer};

    #[test]
    fn requires_no_renderer() {
        assert_that!(1)
            .with_renderer(NoRenderer)
            .matches(anything());
    }
}
