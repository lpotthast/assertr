use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    failure::{FailureBuilder, FailureKind},
};
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

impl<A: ?Sized, R, F> Expectation<A, R> for Predicate<F>
where
    F: Fn(&A) -> bool,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    fn evaluate(&self, actual: &A, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if (self.callback)(actual) {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<A: ?Sized, R, F> ExpectationDiagnostics<A, R> for Predicate<F>
where
    F: Fn(&A) -> bool,
{
    const KIND: FailureKind = FailureKind::Matching;
    fn explain<Target>(
        &self,
        rejected: Option<(&A, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation(self.description.clone()),
            Some((_, ())) => failure
                .relation("does not satisfy the constraint")
                .constraint(context.describe(&self)),
        }
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
