use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use alloc::borrow::Cow;

/// A boolean closure with an optional diagnostic description.
pub struct Predicate<F> {
    callback: F,
    description: Cow<'static, str>,
}

/// Adapts a reusable boolean closure. The subject needs no renderer.
pub fn predicate<A: ?Sized, F>(callback: F) -> Predicate<F>
where
    F: Fn(&A) -> bool,
{
    Predicate {
        callback,
        description: Cow::Borrowed("satisfies the predicate"),
    }
}

impl<F> Predicate<F> {
    /// Sets the relation describing the predicate, without embedding diagnostic values.
    #[must_use]
    pub fn described_as(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }
}

impl<A: ?Sized, R, F> AssertrMatcher<A, R> for Predicate<F>
where
    F: Fn(&A) -> bool,
{
    fn describe(&self, _: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new(self.description.clone())
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        context.outcome((self.callback)(actual), |context| {
            <Self as AssertrMatcher<A, R>>::describe(self, context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::predicate;
    use crate::prelude::*;

    #[test]
    fn propagates_user_panics() {
        assert_that_panic_by(|| {
            assert_that!(1).matches(predicate(|_: &i32| panic!("user panic")));
        })
        .has_type::<&str>()
        .is_equal_to("user panic");
    }
}
