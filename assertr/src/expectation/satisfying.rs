use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics,
    assert_that::collect_assertions,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind},
    mode::Capture,
};

/// A matcher that checks a value with existing assertion methods. Construct it with [`satisfying`].
pub struct Satisfying<F>(F);

/// Uses existing assertion methods as a matcher, including within a `partial!` field expectation.
///
/// The built-in matchers cover a selective set of constraints. Use this adapter when a field
/// needs a check from an [assertion family](crate::assertions) or a custom assertion trait.
/// The closure receives an assertion context for the field. All its assertions must pass for
/// the matcher to match, and their structured failures retain the enclosing field's path.
///
/// ```rust
/// # #[cfg(feature = "matchers")]
/// # {
/// use assertr::prelude::*;
/// use assertr::matchers::satisfying;
///
/// struct User {
///     name: String,
///     age: u32,
/// }
/// let user = User { name: "Alice".into(), age: 30 };
/// assert_that!(user).matches(partial!(User {
///     age: satisfying(|age| {
///         age.is_greater_or_equal_to(18).is_less_than(65);
///     }),
///     ..
/// }));
/// # }
/// ```
///
/// End the final assertion with a semicolon so the synchronous closure returns `()`. It runs in
/// capture mode, using the active renderer and rendering budget. This adapter requires a
/// cloneable renderer and a reusable (`Fn`) callback. It can also be used wherever a matcher is
/// accepted, without enabling the `matchers` feature.
///
/// The similarly named [`AssertThat::satisfies`] projects a subject and continues its original
/// assertion chain. `satisfying` constructs an expectation for a value that will be matched later.
///
/// # Type inference
///
/// If several assertion traits provide the same method, such as `contains`, annotate the closure
/// parameter so Rust can select the assertion family:
///
/// ```rust
/// use assertr::prelude::*;
/// use assertr::matchers::satisfying;
///
/// let editor_roles = satisfying(|roles: AssertThat<'_, Vec<&str>, Capture>| {
///     roles.contains("editor").has_length(2);
/// });
/// assert_that!(vec!["reader", "editor"]).matches(&editor_roles);
/// ```
///
/// This annotation uses the default [`crate::DebugRenderer`]. Supply `AssertThat`'s fourth type
/// parameter when using a custom renderer. The matcher can also be used as a `partial!` field
/// expectation, for example `roles: &editor_roles`.
///
/// # Panics
///
/// Panics during evaluation if the closure performs no assertions. User panics propagate.
pub fn satisfying<A, R, F>(callback: F) -> Satisfying<F>
where
    F: for<'a> Fn(AssertThat<'a, A, Capture, R>),
{
    Satisfying(callback)
}

impl<A, R, F> Expectation<A, R> for Satisfying<F>
where
    R: Clone,
    F: for<'a> Fn(AssertThat<'a, A, Capture, R>),
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
        let failures = collect_assertions(
            actual,
            context.render(),
            context.include_location(),
            &self.0,
        );
        let matched = failures.is_empty();
        if !matched {
            for failure in failures {
                context.record(failure);
            }
            if context.evidence.is_empty() {
                context.outcome(false, |context| context.describe::<A, _>(self));
            }
        }
        let evidence = context.into_evidence();
        if matched { Ok(()) } else { Err(evidence) }
    }
}
impl<A, R, F> ExpectationDiagnostics<A, R> for Satisfying<F>
where
    R: Clone,
    F: for<'a> Fn(AssertThat<'a, A, Capture, R>),
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("satisfies the assertions"),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::satisfying;
    use crate::prelude::*;

    #[test]
    fn adapts_assertion_closures() {
        assert_that!(2).matches(satisfying(|it| {
            it.is_equal_to(2);
        }));
    }

    #[test]
    fn rejects_empty_assertion_closures() {
        assert_that_panic_by(|| {
            assert_that!(1).matches(satisfying(|_| {}));
        })
        .has_type::<&str>()
        .is_equal_to("The closure passed to satisfying performed no assertions!");
    }
}
