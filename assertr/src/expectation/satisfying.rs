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
/// # #[cfg(feature = "partial")]
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
/// accepted, without enabling the `partial` feature.
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
        for failure in failures {
            context.record(failure);
        }
        context.finish(matched, |context| context.describe::<A, _>(self))
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
    use core::cell::Cell;

    #[test]
    fn adapts_assertion_closures() {
        let calls = Cell::new(0);
        let matcher = satisfying(|it| {
            calls.set(calls.get() + 1);
            it.is_equal_to(2).is_greater_than(0);
        });

        assert_that!(2).matches(&matcher).matches(&matcher);
        assert_that!(calls.get()).is_equal_to(2);
    }

    #[test]
    fn captures_every_failure_including_derived_assertions() {
        let failures = assert_that!(1).with_location(false).capture(|it| {
            it.matches(satisfying(|it| {
                let it = it.is_equal_to(2);
                it.derive_owned(|value| value + 1).is_equal_to(3);
            }))
        });

        assert_that!(failures).has_length(1);
        assert_that!(failures[0]).has_text_report(indoc::formatdoc! {r"
            -------- assertr --------
            Expression: `1`

            does not match

            Nested failures:
              - Expected: 2

                  Actual: 1
              - Expected: 3

                  Actual: 2
            -------- assertr --------
        "});
    }

    #[test]
    fn rejects_empty_assertion_closures() {
        assert_that_panic_by(|| {
            assert_that!(1).matches(satisfying(|_| {}));
        })
        .has_type::<&str>()
        .is_equal_to("the assertion callback performed no assertions");
    }

    #[test]
    fn propagates_user_panics_even_after_capturing_a_failure() {
        assert_that_panic_by(|| {
            assert_that!(1).matches(satisfying(|it| {
                it.is_equal_to(2);
                panic!("callback panic");
            }));
        })
        .has_type::<&str>()
        .is_equal_to("callback panic");
    }

    #[test]
    fn describes_missing_subjects_without_invoking_the_callback() {
        let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
        let matcher = satisfying(|_: AssertThat<i32, Capture>| panic!("callback invoked"));

        let description = context.describe::<i32, _>(&matcher);

        assert_that!(description.relation).is_equal_to(Some("satisfies the assertions".into()));
        assert_that!(description.children).is_empty();
    }

    #[test]
    fn scopes_captured_failures_to_the_enclosing_path_once() {
        let mut context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
        let matcher = satisfying(|it| {
            it.is_equal_to(2).is_equal_to(3);
        });
        let path = crate::failure::PathSegment::Index(4);

        let accepted = context.scoped(path.clone(), |context| context.evaluate(&1, &matcher));

        assert_that!(accepted).is_false();
        let evidence = context.into_evidence();
        assert_that!(evidence.children).has_length(2);
        for failure in evidence.children {
            assert_that!(failure.path).contains_exactly([path.clone()]);
        }
    }

    mod inherited_settings {
        use super::*;
        use crate::test_support::{CustomValueRenderer, assert_custom_value};

        #[test]
        fn renders_derived_values_without_requiring_a_subject_renderer() {
            struct Opaque(usize);

            let failures = assert_that!(Opaque(1))
                .with_renderer(CustomValueRenderer)
                .capture(|it| {
                    it.matches(satisfying(
                        |it: AssertThat<'_, Opaque, Capture, CustomValueRenderer>| {
                            it.satisfies(
                                |value| &value.0,
                                |value| {
                                    value.is_equal_to(9);
                                },
                            );
                        },
                    ))
                });

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            let child = &failures[0].children[0];
            assert_custom_value(child.actual.as_ref().unwrap(), &1_usize);
            assert_custom_value(child.expected.as_ref().unwrap(), &9_usize);
        }

        #[test]
        fn limits_captured_failures_and_their_rendered_leaves() {
            let failures = assert_that!(123_456)
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(1)
                        .with_max_leaf_characters(3),
                )
                .capture(|it| {
                    it.matches(satisfying(|it| {
                        it.is_equal_to(99).is_equal_to(100);
                    }))
                });

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].omitted_children).is_equal_to(1);
            assert_that!(rendered_text(
                failures[0].children[0].actual.as_ref().unwrap()
            ))
            .is_equal_to("123... 3 more characters ...");
        }

        #[test]
        fn preserves_the_location_policy_for_captured_assertions() {
            for include_location in [false, true] {
                let failures = assert_that!(1)
                    .with_location(include_location)
                    .capture(|it| {
                        it.matches(satisfying(|it| {
                            it.is_equal_to(2);
                        }))
                    });

                assert_that!(failures[0].children[0].location.is_some())
                    .is_equal_to(include_location);
            }
        }
    }
}
