use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use crate::{
    Fact,
    condition::AssertrCondition,
    failure::{FailureBuilder, FailureKind},
};
use core::any::type_name;

/// An error-typed condition as a composable matcher.
pub struct Condition<C>(C);

/// Adapts a domain condition without requiring a subject renderer.
///
/// Diagnostic evaluation renders the original error through `ValueRenderer<C::Error>`. The
/// default renderer requires `Debug`. Probes never render condition errors.
pub fn condition<C>(condition: C) -> Condition<C> {
    Condition(condition)
}

impl<C> Condition<C> {
    pub(crate) fn test<A>(&self, actual: &A) -> Result<(), C::Error>
    where
        C: AssertrCondition<A>,
    {
        self.0.test(actual)
    }
}

impl<A, R, C> AssertrMatcher<A, R> for Condition<C>
where
    C: AssertrCondition<A>,
    R: crate::ValueRenderer<C::Error>,
{
    fn describe(&self, _: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("satisfies the condition").expected(type_name::<C>())
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let result = self.test(actual);
        let matched = result.is_ok();
        if matched != context.is_positive() {
            if let Err(error) = result {
                if context.is_diagnostic() {
                    context.record(
                        FailureBuilder::detached::<A>(FailureKind::Predicate)
                            .relation("does not match the condition")
                            .fact(Fact::note(context.render().value(&error)))
                            .build(),
                    );
                } else {
                    context.outcome(matched, |context| {
                        <Self as AssertrMatcher<A, R>>::describe(self, context)
                    });
                }
            } else {
                context.outcome(matched, |context| {
                    <Self as AssertrMatcher<A, R>>::describe(self, context)
                });
            }
        }
        MatchResult::new(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::condition;
    use crate::{prelude::*, test_support::CustomValueRenderer};

    struct IsEven;

    impl AssertrCondition<i32> for IsEven {
        type Error = &'static str;

        fn test(&self, value: &i32) -> Result<(), Self::Error> {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err("value is odd")
            }
        }
    }

    #[test]
    fn matches_without_a_subject_renderer() {
        assert_that!(2)
            .with_renderer(CustomValueRenderer)
            .with_location(false)
            .matches(condition(IsEven));
        assert_that!(1)
            .with_renderer(CustomValueRenderer)
            .with_location(false)
            .does_not_match(condition(IsEven));
    }

    #[test]
    fn retains_the_condition_error() {
        let failures = assert_that!(1)
            .with_renderer(CustomValueRenderer)
            .capture(|it| it.matches(condition(IsEven)));

        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.children[0].relation.as_deref())
                    .is_equal_to(Some("does not match the condition"));
                element
                    .derive_owned(|value| ToHumanReadableText.render(value))
                    .contains("value is odd");
            },
        ]);
    }
}
