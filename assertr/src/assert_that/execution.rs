//! Execution of reusable expectations and one-use observations on assertion chains.

use crate::{
    AssertThat, Mode,
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    mode::Panic,
};
use core::panic::Location;

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Supplies the chain's rendering settings and location policy for expectation evaluation.
    pub(crate) fn assertion_context(&self) -> AssertionContext<'_, R> {
        AssertionContext::from_rendering(self.render(), self.state.include_location)
    }

    /// Asserts that the subject satisfies an expectation, then continues with the same subject.
    ///
    /// Any [matcher](mod@crate::matchers) works: a single check like [`eq`](crate::matchers::eq),
    /// a combination like [`all_of`](crate::matchers::all_of), a collection policy like
    /// [`elements_are!`](crate::elements_are), or a custom [`Expectation`]. The expectation is
    /// evaluated once and its failure is reported as is. Pass `&expected` to reuse it.
    ///
    /// ```
    /// use assertr::{matchers::{all_of, eq, ge, lt}, prelude::*};
    ///
    /// assert_that!(42).matches(all_of(matchers![ge(18), lt(65)]));
    /// assert_that!([1, 2]).matches(elements_are![eq(1), ge(2)]);
    /// ```
    ///
    /// Plain values are not matchers. Write `matches(eq(2))`, or use
    /// [`is_equal_to`](crate::assertions::PartialEqAssertions::is_equal_to).
    #[track_caller]
    #[allow(clippy::return_self_not_must_use)]
    pub fn matches<D: Expectation<T, R>>(self, expected: D) -> Self {
        drop(self.test_assertion(&expected));
        self
    }

    /// Fluent alias of [`AssertThat::matches`].
    #[cfg(feature = "fluent")]
    #[track_caller]
    #[allow(clippy::return_self_not_must_use)]
    pub fn match_expectation<D: Expectation<T, R>>(self, expected: D) -> Self {
        self.matches(expected)
    }

    /// Asserts a reusable expectation, returning its successful observation for a projection or
    /// callback. A rejected expectation raises its explained failure and returns `None` in capture
    /// mode. Evaluation and explanation share the original observation.
    ///
    /// This tracks one assertion, just like [`matches`](Self::matches). A method that delegates
    /// here must use `#[track_caller]` and must not track the assertion again.
    #[track_caller]
    pub fn test_assertion<'a, D: Expectation<T, R>>(
        &'a self,
        definition: &'a D,
    ) -> Option<D::Success<'a>> {
        self.track_assertion();
        self.test_observation_after_tracking(self.actual(), definition, Location::caller())
    }

    /// Applies an expectation to the subject after the adapter has tracked and captured its
    /// caller, for example before invoking user code or awaiting.
    #[cfg(feature = "std")]
    #[track_caller]
    pub(crate) fn apply_assertion_after_tracking<D: Expectation<T, R>>(
        self,
        definition: D,
        location: &'static Location<'static>,
    ) -> Self {
        drop(self.test_observation_after_tracking(self.actual(), &definition, location));
        self
    }

    /// Executes an expectation on a borrowed observation after the assertion was tracked, raising
    /// a rejection at `location`.
    #[track_caller]
    pub(crate) fn test_observation_after_tracking<'a, O: ?Sized, D: Expectation<O, R>>(
        &'a self,
        actual: &'a O,
        definition: &'a D,
        location: &'static Location<'static>,
    ) -> Option<D::Success<'a>> {
        self.test_once_after_tracking(
            D::KIND,
            location,
            |context| definition.evaluate(actual, context),
            |rejection, failure, context| {
                definition.explain(Some((actual, rejection)), failure, context)
            },
        )
    }

    /// Executes one observation after the adapter has tracked and captured its caller.
    ///
    /// Observation and explanation each run at most once. Rejection owns any resources needed
    /// for rendering. Explanation must release them before returning the populated builder,
    /// so failure routing never holds a temporary guard. Successful observations belong to the
    /// caller, just as they do with `test_assertion`.
    #[track_caller]
    pub(crate) fn test_once_after_tracking<Success, Rejection>(
        &self,
        kind: FailureKind,
        location: &'static Location<'static>,
        observe: impl FnOnce(&AssertionContext<'_, R>) -> Result<Success, Rejection>,
        explain: impl FnOnce(Rejection, FailureBuilder, &AssertionContext<'_, R>) -> FailureBuilder,
    ) -> Option<Success> {
        let context = self.assertion_context();
        match observe(&context) {
            Ok(success) => Some(success),
            Err(rejection) => {
                self.raise_at(explain(rejection, self.failure(kind), &context), location);
                None
            }
        }
    }
}

impl<T, R> AssertThat<'_, T, Panic, R> {
    /// Asserts an expectation whose successful observation an extraction continues with.
    ///
    /// Panic mode raises every rejection, so a returned value always exists. This tracks one
    /// assertion, just like [`test_assertion`](Self::test_assertion).
    #[track_caller]
    pub(crate) fn require<'a, D: Expectation<T, R>>(&'a self, definition: &'a D) -> D::Success<'a> {
        self.test_assertion(definition)
            .expect("panic mode raises rejected expectations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expectation::Expectation,
        failure::{FailureBuilder, FailureKind},
        prelude::*,
        renderer::RenderingBudget,
    };
    use core::cell::Cell;

    #[cfg(feature = "fluent")]
    mod matches_fluent_aliases {
        use crate::matchers::eq;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            1.must().match_expectation(eq(1));
        }
    }

    mod matches {
        use crate::{matchers::*, prelude::*};
        use indoc::indoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), matches(eq(2)));
        }

        #[test]
        fn does_not_require_a_renderer() {
            assert_that!(())
                .with_renderer(crate::test_support::NoRenderer)
                .matches(anything());
        }

        #[test]
        fn tracks_once_and_keeps_capture_assertions_isolated() {
            let subject = assert_that!(1).with_renderer(DebugRenderer);
            let subject = subject.matches(satisfying(|it| {
                it.is_equal_to(1).is_less_than(2);
            }));

            assert_that!(subject.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn uses_the_equality_failure_directly() {
            let failures = assert_that!(1)
                .with_location(false)
                .capture(|it| it.matches(eq(2)));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(crate::failure::FailureKind::Equality);
                    element.derive(|value| &value.children).is_empty();
                    element
                        .derive_owned(ToString::to_string)
                        .is_equal_to(indoc! {r"
                -------- assertr --------
                Expression: `1`

                Expected: 2

                  Actual: 1
                -------- assertr --------
            "});
                },
            ]);
        }
    }

    mod one_use {
        use super::*;
        use core::cell::RefCell;

        #[test]
        fn success_transfers_a_guard_without_renderer_support_or_repeated_observation() {
            struct NoRenderer;
            let value = RefCell::new(7);
            let calls = Cell::new(0);
            let chain = assert_that!(()).with_renderer(NoRenderer);
            chain.track_assertion();
            let guard = value.borrow_mut();
            let success = chain.test_once_after_tracking(
                FailureKind::Other,
                Location::caller(),
                |_| {
                    assert_that!(chain.state.records.assertion_count()).is_equal_to(1);
                    calls.set(calls.get() + 1);
                    Ok::<_, ()>(guard)
                },
                |(), _, _| panic!("a successful observation must not be explained"),
            );
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(value.try_borrow_mut().is_err()).is_true();
            drop(success);
            assert_that!(value.try_borrow_mut().is_ok()).is_true();
        }

        #[test]
        fn rejection_renders_the_original_guard_then_releases_it_before_continuation() {
            let value = RefCell::new(7);
            let calls = Cell::new(0);
            let explanations = Cell::new(0);
            let failures = assert_that!(()).capture(|chain| {
                chain.track_assertion();
                let guard = value.borrow_mut();
                let success = chain.test_once_after_tracking(
                    FailureKind::Other,
                    Location::caller(),
                    |_| {
                        assert_that!(chain.state.records.assertion_count()).is_equal_to(1);
                        calls.set(calls.get() + 1);
                        Err::<(), _>(guard)
                    },
                    |guard, failure, context| {
                        explanations.set(explanations.get() + 1);
                        assert_that!(value.try_borrow_mut().is_err()).is_true();
                        let failure = failure.actual(context.render().value(&*guard));
                        drop(guard);
                        failure
                    },
                );
                assert_that!(success).is_none();
                *value.borrow_mut() = 9;
                chain
            });
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(explanations.get()).is_equal_to(1);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains("Actual: 7");
        }
    }

    struct Expected<'e> {
        text: &'e str,
        calls: &'e Cell<usize>,
    }

    impl<R: ValueRenderer<str>> Expectation<String, R> for Expected<'_> {
        type Success<'a>
            = ()
        where
            Self: 'a,
            String: 'a;
        type Rejection<'a>
            = (&'a str, &'a str)
        where
            Self: 'a,
            String: 'a;

        fn evaluate<'a>(
            &'a self,
            actual: &'a String,
            context: &AssertionContext<'_, R>,
        ) -> Result<(), Self::Rejection<'a>> {
            self.calls.set(self.calls.get() + 1);
            assert_that!(context.include_location()).is_false();
            assert_that!(context.render().budget().max_leaf_characters()).is_equal_to(2);
            Err((actual.as_str(), self.text))
        }

        const KIND: FailureKind = FailureKind::Other;

        fn explain<'a>(
            &'a self,
            rejected: Option<(&'a String, Self::Rejection<'a>)>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            let render = context.render();
            match rejected {
                None => failure
                    .relation("has the expected text")
                    .expected(render.value(self.text)),
                Some((_, (actual, expected))) => failure
                    .actual(render.value(actual))
                    .expected(render.value(expected)),
            }
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn tracks_before_evaluation_can_panic() {
        use crate::matchers::predicate;

        let failures = assert_that!(1).capture(|it| {
            let child = it.derive(|value| value);
            let outcome = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                child.matches(predicate(|_: &i32| panic!("evaluation panicked")));
            }));
            assert_that!(outcome).is_err();
            it
        });
        assert_that!(failures).is_empty();
    }

    #[test]
    fn ordinary_and_matcher_execution_retain_borrowed_rejection_without_retesting() {
        let calls = Cell::new(0);
        let text = String::from("expected");
        let definition = Expected {
            text: &text,
            calls: &calls,
        };
        let actual = String::from("actual");
        let failures = assert_that!(actual)
            .with_location(false)
            .with_subject_name("subject")
            .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(2))
            .capture(|it| it.matches(&definition));
        assert_that!(calls.get()).is_equal_to(1);
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].kind).is_equal_to(FailureKind::Other);
        assert_that!(failures[0].subject_name.as_deref()).is_equal_to(Some("subject"));
        assert_that!(failures[0].expression).is_equal_to(Some("actual"));
        assert_that!(failures[0].location).is_none();
        assert_that!(failures[0].children).is_empty();

        for rendered in [&failures[0].actual, &failures[0].expected] {
            let crate::renderer::RenderedBody::Text {
                text,
                omitted_characters,
            } = &rendered.as_ref().unwrap().body
            else {
                panic!("expected a rendered leaf")
            };
            assert_that!(text.chars().count()).is_equal_to(2);
            assert_that!(*omitted_characters).is_greater_than(0);
        }
    }
}
