use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use crate::{
    ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};

/// A string prefix constraint.
pub struct StartsWith<E>(E);

/// Matches a string prefix through `AsRef<str>`.
pub fn starts_with<E>(expected: E) -> StartsWith<E>
where
    E: AsRef<str>,
{
    StartsWith(expected)
}

impl<A, R, E> AssertrMatcher<A, R> for StartsWith<E>
where
    A: AsRef<str> + ?Sized,
    R: ValueRenderer<str>,
    E: AsRef<str>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("starts with").expected(context.render().value(self.0.as_ref()))
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let matched = actual.as_ref().starts_with(self.0.as_ref());
        if matched != context.is_positive() && context.is_diagnostic() {
            context.record(
                FailureBuilder::detached::<A>(FailureKind::Membership)
                    .actual(context.render().value(actual.as_ref()))
                    .relation(if matched {
                        "starts with"
                    } else {
                        "does not start with"
                    })
                    .expected(context.render().value(self.0.as_ref()))
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
    use super::starts_with;
    use crate::{matchers::MatchContext, prelude::*};

    #[test]
    fn accepts_unsized_strings() {
        let matcher = starts_with("hel");
        let mut context = MatchContext::default();

        assert_that!(matcher.evaluate("hello", &mut context).matched).is_true();
    }
}
