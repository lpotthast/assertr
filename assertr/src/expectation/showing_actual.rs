use crate::{
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    renderer::ValueRenderer,
};

/// Renders the subject when the wrapped matcher rejects it. Construct it with [`showing_actual`].
#[derive(Debug, Clone)]
pub struct ShowingActual<M>(M);

/// Adds the rendered subject to a rejection of `matcher` that does not already show it.
///
/// Composite matchers such as `partial!`, [`each`](crate::matchers::each), and
/// [`all_of`](crate::matchers::all_of) render only the parts they check, so they work without a
/// renderer for the whole subject. A rejected `partial!` variant therefore names the expected
/// variant but not the actual value. Wrap the matcher to show the actual value too, which
/// requires a renderer for the subject:
///
/// ```
/// # #[cfg(feature = "partial")]
/// # {
/// use assertr::{matchers::{eq, showing_actual}, prelude::*};
///
/// let result: Result<i32, &str> = Err("timed out");
/// let failures = assert_that!(result)
///     .capture(|it| it.matches(showing_actual(partial!(variant Ok(eq(1))))));
/// assert_that!(failures[0].to_string())
///     .contains("Actual: Err(")
///     .contains("timed out");
/// # }
/// ```
///
/// The wrapped failure is reported whole, never flattened into an enclosing matcher, so the
/// actual value stays attached to it.
#[must_use]
pub const fn showing_actual<M>(matcher: M) -> ShowingActual<M> {
    ShowingActual(matcher)
}

impl<T: ?Sized, R, M> Expectation<T, R> for ShowingActual<M>
where
    M: Expectation<T, R>,
    R: ValueRenderer<T>,
{
    type Success<'a>
        = M::Success<'a>
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = M::Rejection<'a>
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        self.0.evaluate(actual, context)
    }

    const KIND: FailureKind = M::KIND;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let actual = rejected.as_ref().map(|(actual, _)| *actual);
        let failure = self.0.explain(rejected, failure, context);
        match actual {
            Some(actual) if !failure.has_actual() => failure.actual(context.render().value(actual)),
            _ => failure,
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use super::showing_actual;
    use crate::{
        matchers::{all_of, each, eq},
        prelude::*,
    };

    #[test]
    fn caller_location_is_as_expected() {
        assert_caller_location!(assert_that!([1, 2]), matches(showing_actual(each(eq(1)))));
    }

    #[test]
    fn passes_like_the_wrapped_matcher() {
        assert_that!([1, 1]).matches(showing_actual(each(eq(1))));
    }

    #[test]
    fn renders_the_subject_of_a_composite_rejection() {
        assert_that!(|| {
            assert_that!([1, 2])
                .with_location(false)
                .matches(showing_actual(each(eq(1))));
        })
        .panics()
        .message()
        .is_equal_to(formatdoc! {"
            -------- assertr --------
            Expression: `[1, 2]`

            Actual: [
                1,
                2,
            ]

            does not match

            Nested failures:
              - At [1]:
                Expected: 1

                  Actual: 2
            -------- assertr --------
        "});
    }

    #[test]
    fn keeps_an_actual_value_the_matcher_already_shows() {
        let failures = assert_that!(2).capture(|it| it.matches(showing_actual(eq(1))));
        assert_that!(&failures[0].actual)
            .is_equal_to(Some(AssertionContext::default().render().value(&2)));
    }

    #[test]
    fn stays_whole_inside_compositions() {
        let failures = assert_that!([1, 2])
            .capture(|it| it.matches(all_of(matchers![showing_actual(each(eq(1)))])));
        assert_that!(failures[0].children).has_length(1);
        assert_that!(failures[0].children[0].actual.is_some()).is_true();
    }
}
