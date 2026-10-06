use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};
use alloc::borrow::Cow;
use core::fmt;

/// A boolean closure with an optional diagnostic description.
///
/// It is `Clone` when the closure is. `Debug` shows the description without the closure.
#[derive(Clone)]
pub struct Predicate<F> {
    callback: F,
    description: Cow<'static, str>,
}

impl<F> fmt::Debug for Predicate<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Predicate")
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

/// Adapts a reusable boolean closure.
///
/// Evaluation needs no renderer. Diagnostics render a rejected subject, so explaining a failure
/// requires the active renderer to support the subject type.
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
    R: ValueRenderer<A>,
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
            Some((actual, ())) => failure
                .actual(context.render().value(actual))
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
    fn renders_the_rejected_subject_through_the_active_renderer() {
        use crate::test_support::{CustomValueRenderer, assert_custom_value};

        let failures = assert_that!([1, 7])
            .with_renderer(CustomValueRenderer)
            .capture(|it| it.matches(matchers::each(predicate(|x: &i32| *x < 5))));
        assert_that!(failures[0].children).has_length(1);
        let child = &failures[0].children[0];
        assert_custom_value(child.actual.as_ref().unwrap(), &7);
        assert_that!(child.relation.as_deref())
            .is_equal_to(Some("does not satisfy the constraint"));
        assert_that!(child.constraint.as_ref().unwrap().relation.as_deref())
            .is_equal_to(Some("satisfies the predicate"));
    }

    #[test]
    fn propagates_user_panics() {
        assert_that_panic_by(|| {
            assert_that!(1).matches(predicate(|_: &i32| panic!("user panic")));
        })
        .has_type::<&str>()
        .is_equal_to("user panic");
    }
}
