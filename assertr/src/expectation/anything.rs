use crate::{
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
};

/// An unconstrained wildcard.
#[derive(Debug, Clone, Copy)]
pub struct Anything;

/// Always matches, without renderer requirements.
#[must_use]
pub const fn anything() -> Anything {
    Anything
}

impl<A: ?Sized, R> Expectation<A, R> for Anything {
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = core::convert::Infallible
    where
        Self: 'a,
        A: 'a;
    fn evaluate(
        &self,
        _: &A,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), core::convert::Infallible> {
        Ok(())
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&A, core::convert::Infallible)>,
        failure: FailureBuilder,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("is anything"),
            Some((_, rejection)) => match rejection {},
        }
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
