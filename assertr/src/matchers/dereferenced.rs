use super::{AssertrMatcher, Description, MatchContext, MatchResult};

/// Explicitly dereferences an actual reference before matching. No actual values are moved.
pub struct Dereferenced<M>(M);

/// Adapts a matcher for owned reference subjects or reference-valued fields.
pub fn dereferenced<M>(matcher: M) -> Dereferenced<M> {
    Dereferenced(matcher)
}

impl<A: ?Sized, R, M> AssertrMatcher<&A, R> for Dereferenced<M>
where
    M: AssertrMatcher<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        self.0.describe(context)
    }

    fn evaluate(&self, actual: &&A, context: &mut MatchContext<'_, R>) -> MatchResult {
        self.0.evaluate(*actual, context)
    }
}

#[cfg(test)]
mod tests {
    use super::dereferenced;
    use crate::{matchers::equal_to, prelude::*};

    #[test]
    fn accepts_owned_references() {
        assert_that_owned!(&42).matches(dereferenced(equal_to(42)));
    }
}
