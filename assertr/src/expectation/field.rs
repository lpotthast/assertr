use core::fmt;

use crate::{
    expectation::{AssertionContext, Evidence, Expectation},
    failure::{FailureBuilder, FailureKind, PathSegment},
};

/// Applies a matcher to one field of the subject, locating its evidence at that field.
///
/// Construct it with [`field`].
pub struct Field<A: ?Sized, T: ?Sized, F, M> {
    projection: F,
    // Selects the value through `projection`, so required and optional projections share one type.
    select: for<'a> fn(&F, &'a A) -> Option<&'a T>,
    matcher: M,
    path: PathSegment,
}

impl<A: ?Sized, T: ?Sized, F: Clone, M: Clone> Clone for Field<A, T, F, M> {
    fn clone(&self) -> Self {
        Self {
            projection: self.projection.clone(),
            select: self.select,
            matcher: self.matcher.clone(),
            path: self.path.clone(),
        }
    }
}

impl<A: ?Sized, T: ?Sized, F, M: fmt::Debug> fmt::Debug for Field<A, T, F, M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Field")
            .field("path", &self.path)
            .field("matcher", &self.matcher)
            .finish_non_exhaustive()
    }
}

/// Matches the field selected by `projection` against `matcher`.
///
/// Failures show the matcher's explanation at the named field, such as `At .name:`. Combine it
/// with [`predicate`](crate::matchers::predicate) or any other matcher to check a domain type
/// without implementing [`Expectation`]:
///
/// ```
/// use assertr::{matchers::{field, predicate}, prelude::*};
///
/// struct Person {
///     name: String,
/// }
///
/// fn has_name<R: ValueRenderer<String>>() -> impl Expectation<Person, R> + Clone {
///     field(
///         "name",
///         |person: &Person| &person.name,
///         predicate(|name: &String| !name.is_empty())
///             .described_as("is not empty")
///             .rejected_as("is empty"),
///     )
/// }
///
/// assert_that!(Person { name: "Ada".into() }).matches(has_name());
/// let failures = assert_that!(Person { name: String::new() }).capture(|it| it.matches(has_name()));
/// assert_that!(failures[0].to_string()).contains("At .name:");
/// ```
pub fn field<A: ?Sized, T: ?Sized, F, M>(
    name: &'static str,
    projection: F,
    matcher: M,
) -> Field<A, T, F, M>
where
    F: for<'a> Fn(&'a A) -> &'a T,
{
    // Matches the selector type shared with optional `partial!` projections.
    #[allow(clippy::unnecessary_wraps)]
    fn select<'a, A: ?Sized, T: ?Sized, F: for<'b> Fn(&'b A) -> &'b T>(
        projection: &F,
        actual: &'a A,
    ) -> Option<&'a T> {
        Some(projection(actual))
    }
    Field {
        projection,
        select,
        matcher,
        path: PathSegment::Field(name),
    }
}

/// Matches an optional projection at an explicit path. `None` rejects without evaluating the
/// matcher. Generated `partial!` code reaches this through `__private::field`.
pub(crate) fn projected<A: ?Sized, T: ?Sized, F, M>(
    projection: F,
    matcher: M,
    path: PathSegment,
) -> Field<A, T, F, M>
where
    F: for<'a> Fn(&'a A) -> Option<&'a T>,
{
    fn select<'a, A: ?Sized, T: ?Sized, F: for<'b> Fn(&'b A) -> Option<&'b T>>(
        projection: &F,
        actual: &'a A,
    ) -> Option<&'a T> {
        projection(actual)
    }
    Field {
        projection,
        select,
        matcher,
        path,
    }
}

impl<A: ?Sized, T: ?Sized, R, F, M> Expectation<A, R> for Field<A, T, F, M>
where
    M: Expectation<T, R>,
{
    composite_items!(A);
    fn evaluate(&self, actual: &A, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let matched = (self.select)(&self.projection, actual).is_some_and(|value| {
            context.scoped(self.path.clone(), |context| {
                context.evaluate(value, &self.matcher)
            })
        });
        context.finish(matched, |_| {
            FailureBuilder::new::<()>(FailureKind::Matching)
                .relation("has the required structure")
                .build()
        })
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
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
    use core::cell::Cell;

    use super::{field, projected};
    use crate::{
        assertions::core::partial_eq::eq,
        failure::{FailureKind, PathSegment},
        matchers::predicate,
        prelude::*,
    };

    #[test]
    fn scopes_projected_evidence_to_the_field() {
        let matcher = projected(
            |value: &(i32,)| Some(&value.0),
            eq(2),
            PathSegment::TupleIndex(0),
        );
        let failures = assert_that!((1,)).capture(|it| it.matches(matcher));

        assert_that!(failures[0].children[0].path).is_equal_to([PathSegment::TupleIndex(0)]);
        assert_that!(failures[0].children[0].kind).is_equal_to(FailureKind::Equality);
    }

    #[test]
    fn missing_fields_do_not_evaluate_the_inner_matcher() {
        let calls = Cell::new(0);
        let matcher = projected(
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

    #[test]
    fn named_fields_locate_rejections_and_describe_the_matcher() {
        struct Person {
            name: &'static str,
        }
        let matcher = field("name", |person: &Person| &person.name, eq("Ada"));
        let failures = assert_that!(Person { name: "Bob" })
            .with_location(false)
            .capture(|it| it.matches(&matcher));

        assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {r#"
            -------- assertr --------
            Expression: `Person { name: "Bob" }`

            does not match

            Nested failures:
              - At .name:
                Expected: "Ada"

                  Actual: "Bob"
            -------- assertr --------
        "#});
    }
}
