use crate::{
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    renderer::ValueRenderer,
};
use alloc::borrow::Cow;
use core::fmt;

/// A boolean closure with optional diagnostic relations.
///
/// It is `Clone` when the closure is. `Debug` shows the relations without the closure.
#[derive(Clone)]
pub struct Predicate<F> {
    callback: F,
    description: Cow<'static, str>,
    rejection: Option<Cow<'static, str>>,
}

impl<F> fmt::Debug for Predicate<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Predicate")
            .field("description", &self.description)
            .field("rejection", &self.rejection)
            .finish_non_exhaustive()
    }
}

/// Adapts a reusable boolean closure.
///
/// Diagnostics render a rejected subject, so the active renderer must support the subject type.
/// Name the check with [`described_as`](Predicate::described_as) and its rejection with
/// [`rejected_as`](Predicate::rejected_as) to define a domain check without implementing
/// [`Expectation`]:
///
/// ```
/// use assertr::{matchers::predicate, prelude::*};
///
/// fn is_even<R: ValueRenderer<i32>>() -> impl Expectation<i32, R> + Clone {
///     predicate(|value: &i32| value % 2 == 0)
///         .described_as("is even")
///         .rejected_as("is odd")
/// }
///
/// assert_that!(4).matches(is_even());
/// let failures = assert_that!(3).with_location(false).capture(|it| it.matches(is_even()));
/// assert_that!(failures[0].to_string()).contains("Actual: 3\n\nis odd");
/// ```
pub fn predicate<A: ?Sized, F>(callback: F) -> Predicate<F>
where
    F: Fn(&A) -> bool,
{
    Predicate {
        callback,
        description: Cow::Borrowed("satisfies the predicate"),
        rejection: None,
    }
}

impl<F> Predicate<F> {
    /// Sets the relation describing the predicate, without embedding diagnostic values.
    #[must_use]
    pub fn described_as(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the relation reported next to a rejected subject, such as `is odd`.
    ///
    /// Without it, a rejection reports that the subject does not satisfy the constraint and
    /// nests the [`described_as`](Self::described_as) description.
    #[must_use]
    pub fn rejected_as(mut self, relation: impl Into<Cow<'static, str>>) -> Self {
        self.rejection = Some(relation.into());
        self
    }
}

impl<A: ?Sized, R, F> Expectation<A, R> for Predicate<F>
where
    F: Fn(&A) -> bool,
    R: ValueRenderer<A>,
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

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&A, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match (rejected, &self.rejection) {
            (None, _) => failure.relation(self.description.clone()),
            (Some((actual, ())), Some(rejection)) => failure
                .actual(context.render().value(actual))
                .relation(rejection.clone()),
            (Some((actual, ())), None) => failure
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
    fn a_rejection_relation_replaces_the_nested_constraint() {
        let failures = assert_that!(3).with_location(false).capture(|it| {
            it.matches(
                predicate(|value: &i32| value % 2 == 0)
                    .described_as("is even")
                    .rejected_as("is odd"),
            )
        });
        assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {"
            -------- assertr --------
            Expression: `3`

            Actual: 3

            is odd
            -------- assertr --------
        "});
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
