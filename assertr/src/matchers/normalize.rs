use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, equals};
use crate::{
    ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};
use core::marker::PhantomData;

#[doc(hidden)]
pub struct EqualityKind;

#[doc(hidden)]
pub struct MatcherKind;

#[doc(hidden)]
pub struct Normalize<E, K>(E, PhantomData<K>);

#[doc(hidden)]
pub fn normalize<E, K>(expected: E) -> Normalize<E, K> {
    Normalize(expected, PhantomData)
}

impl<A, R, E> AssertrMatcher<A, R> for Normalize<E, EqualityKind>
where
    A: PartialEq<E> + ?Sized,
    R: ValueRenderer<A> + ValueRenderer<E>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("is equal to").expected(context.render().value(&self.0))
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        // Borrowing the expected value must preserve PartialEq<E>, not add PartialEq<&E>.
        let matched = equals(actual, &self.0);
        if matched != context.is_positive() && context.is_diagnostic() {
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
        MatchResult::new(matched)
    }
}

impl<A: ?Sized, R, E> AssertrMatcher<A, R> for Normalize<E, MatcherKind>
where
    E: AssertrMatcher<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        self.0.describe(context)
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        self.0.evaluate(actual, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::{matchers::ge, prelude::*};
    use alloc::string::String;

    #[test]
    fn preserves_heterogeneous_equality_in_macro_shorthand() {
        assert_that!([String::from("hello")]).matches(elements_are!["hello"]);
        assert_that!([String::from("hello")]).does_not_match(elements_are!["world"]);
    }

    #[test]
    fn accepts_matchers_alongside_plain_values() {
        let matcher = elements_are![1, ge(2)];

        assert_that!([1, 3]).matches(&matcher);
        assert_that!([1, 0]).does_not_match(&matcher);
    }
}
