use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use crate::failure::PathSegment;
use core::marker::PhantomData;

#[doc(hidden)]
pub struct Field<A: ?Sized, T: ?Sized, P, M> {
    projection: P,
    matcher: M,
    path: PathSegment,
    types: PhantomData<fn(&A, &T)>,
}

#[doc(hidden)]
pub fn field<A: ?Sized, T: ?Sized, P, M>(
    projection: P,
    matcher: M,
    path: PathSegment,
) -> Field<A, T, P, M>
where
    P: for<'a> Fn(&'a A) -> Option<&'a T>,
{
    Field {
        projection,
        matcher,
        path,
        types: PhantomData,
    }
}

impl<A: ?Sized, T: ?Sized, R, P, M> AssertrMatcher<A, R> for Field<A, T, P, M>
where
    P: for<'a> Fn(&'a A) -> Option<&'a T>,
    M: AssertrMatcher<T, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        self.matcher.describe(context)
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        if let Some(value) = (self.projection)(actual) {
            context.scoped(self.path.clone(), |context| {
                self.matcher.evaluate(value, context)
            })
        } else {
            context.outcome(false, |_| {
                ConstraintDescription::new("has the required structure")
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::field;
    use crate::{
        failure::{FailureKind, PathSegment},
        matchers::{equal_to, predicate},
        prelude::*,
    };
    use core::cell::Cell;

    #[test]
    fn scopes_projected_evidence_to_the_field() {
        let matcher = field(
            |value: &(i32,)| Some(&value.0),
            equal_to(2),
            PathSegment::TupleIndex(0),
        );
        let failures = assert_that!((1,)).capture(|it| it.matches(matcher));

        assert_that!(failures[0].children[0].path).is_equal_to([PathSegment::TupleIndex(0)]);
        assert_that!(failures[0].children[0].kind).is_equal_to(FailureKind::Equality);
    }

    #[test]
    fn missing_fields_do_not_evaluate_the_inner_matcher() {
        let calls = Cell::new(0);
        let matcher = field(
            |value: &Option<i32>| value.as_ref(),
            predicate(|_: &i32| {
                calls.set(calls.get() + 1);
                true
            }),
            PathSegment::TupleIndex(0),
        );

        assert_that!(None::<i32>).does_not_match(matcher);
        assert_that!(calls.get()).is_equal_to(0);
    }
}
