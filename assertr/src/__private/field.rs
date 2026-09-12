use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind, PathSegment},
};
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

impl<A: ?Sized, T: ?Sized, R, F, M> Expectation<A, R> for Field<A, T, F, M>
where
    F: for<'a> Fn(&'a A) -> Option<&'a T>,
    M: ExpectationDiagnostics<T, R>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        A: 'a;
    fn evaluate(&self, actual: &A, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let matched = if let Some(value) = (self.projection)(actual) {
            context.scoped(self.path.clone(), |context| {
                context.evaluate(value, &self.matcher)
            })
        } else {
            context.outcome(false, |_| {
                FailureBuilder::detached::<()>(FailureKind::Matching)
                    .relation("has the required structure")
                    .build()
            })
        };
        let evidence = context.into_evidence();
        if matched { Ok(()) } else { Err(evidence) }
    }
}

impl<A: ?Sized, T: ?Sized, R, F, M> ExpectationDiagnostics<A, R> for Field<A, T, F, M>
where
    F: for<'a> Fn(&'a A) -> Option<&'a T>,
    M: ExpectationDiagnostics<T, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => self.matcher.explain(
                None,
                failure
                    .path([self.path.clone()])
                    .subject_type::<T>()
                    .kind(M::KIND),
                context,
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::field;
    use crate::{
        assertions::core::partial_eq::equal_to,
        expectation::predicate,
        failure::{FailureKind, PathSegment},
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

        let failures = assert_that!(None::<i32>).capture(|it| it.matches(matcher));
        assert_that!(failures).has_length(1);
        assert_that!(calls.get()).is_equal_to(0);
    }
}
