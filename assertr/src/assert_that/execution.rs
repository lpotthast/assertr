//! Execution of reusable expectations and one-use observations on assertion chains.

use crate::{
    AssertThat, AssertionContext, ExpectationDiagnostics, Mode,
    failure::{Attached, FailureBuilder, FailureKind},
};
use core::panic::Location;

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Supplies the chain's rendering settings and location policy for expectation evaluation.
    pub(crate) fn assertion_context(&self) -> AssertionContext<'_, R> {
        AssertionContext::from_rendering(self.render(), self.state.include_location)
    }

    /// Asserts a reusable expectation and continues with the original subject and settings.
    ///
    /// The expectation is evaluated once. Rejections are explained and raised through the chain's
    /// active failure mode. Successful observations are dropped before returning the chain.
    ///
    /// ```
    /// use assertr::{matchers::EqualTo, prelude::*};
    /// assert_that!(3).apply_assertion(EqualTo::new(3)).is_greater_than(2);
    /// ```
    #[track_caller]
    #[allow(clippy::return_self_not_must_use)]
    pub fn apply_assertion<D: ExpectationDiagnostics<T, R>>(self, definition: D) -> Self {
        drop(self.test_assertion(&definition));
        self
    }

    /// Asserts a reusable expectation, returning its successful observation for a projection or
    /// callback. A rejected expectation raises its explained failure and returns `None` in capture
    /// mode. Evaluation and explanation share the original observation.
    ///
    /// This tracks one assertion, just like [`apply_assertion`](Self::apply_assertion). A method
    /// that delegates here must use `#[track_caller]` and must not track the assertion again.
    #[track_caller]
    pub fn test_assertion<'a, D: ExpectationDiagnostics<T, R>>(
        &'a self,
        definition: &'a D,
    ) -> Option<D::Success<'a>> {
        self.test_assertion_with_failure(definition, |_, failure| failure)
    }

    #[track_caller]
    pub(crate) fn apply_assertion_with_failure<D, F>(self, definition: D, prepare: F) -> Self
    where
        D: ExpectationDiagnostics<T, R>,
        F: for<'a> FnOnce(&'a Self, FailureBuilder<Attached<'a>>) -> FailureBuilder<Attached<'a>>,
    {
        drop(self.test_assertion_with_failure(&definition, prepare));
        self
    }

    #[track_caller]
    fn test_assertion_with_failure<'a, D, F>(
        &'a self,
        definition: &'a D,
        prepare: F,
    ) -> Option<D::Success<'a>>
    where
        D: ExpectationDiagnostics<T, R>,
        F: FnOnce(&'a Self, FailureBuilder<Attached<'a>>) -> FailureBuilder<Attached<'a>>,
    {
        self.track_assertion();
        self.test_assertion_after_tracking(definition, prepare)
    }

    // Consuming adapters track before invoking user code, then execute the observed or collected
    // subject without tracking a second time.
    #[track_caller]
    pub(crate) fn apply_assertion_after_tracking<D: ExpectationDiagnostics<T, R>>(
        self,
        definition: D,
    ) -> Self {
        self.apply_assertion_after_tracking_at(definition, Location::caller())
    }

    #[track_caller]
    pub(crate) fn apply_assertion_after_tracking_at<D: ExpectationDiagnostics<T, R>>(
        self,
        definition: D,
        location: &'static Location<'static>,
    ) -> Self {
        drop(self.test_observation_after_tracking(self.actual(), &definition, location));
        self
    }

    #[track_caller]
    fn test_assertion_after_tracking<'a, D, F>(
        &'a self,
        definition: &'a D,
        prepare: F,
    ) -> Option<D::Success<'a>>
    where
        D: ExpectationDiagnostics<T, R>,
        F: FnOnce(&'a Self, FailureBuilder<Attached<'a>>) -> FailureBuilder<Attached<'a>>,
    {
        self.test_observation_with_failure(self.actual(), definition, Location::caller(), prepare)
    }

    /// Executes an adapter's borrowed observation on the original chain. The adapter has already
    /// tracked, and may execute multiple observations or resume after awaiting at `location`.
    #[track_caller]
    pub(crate) fn test_observation_after_tracking<'a, O: ?Sized, D>(
        &'a self,
        actual: &'a O,
        definition: &'a D,
        location: &'static Location<'static>,
    ) -> Option<D::Success<'a>>
    where
        D: ExpectationDiagnostics<O, R>,
    {
        self.test_observation_with_failure(actual, definition, location, |_, failure| failure)
    }

    #[track_caller]
    fn test_observation_with_failure<'a, O: ?Sized, D, F>(
        &'a self,
        actual: &'a O,
        definition: &'a D,
        location: &'static Location<'static>,
        prepare: F,
    ) -> Option<D::Success<'a>>
    where
        D: ExpectationDiagnostics<O, R>,
        F: FnOnce(&'a Self, FailureBuilder<Attached<'a>>) -> FailureBuilder<Attached<'a>>,
    {
        self.test_once_after_tracking(
            D::KIND,
            location,
            |context| definition.evaluate(actual, context),
            |rejection, failure, context| {
                definition.explain(Some((actual, rejection)), prepare(self, failure), context)
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
    pub(crate) fn test_once_after_tracking<'a, Success, Rejection>(
        &'a self,
        kind: FailureKind,
        location: &'static Location<'static>,
        observe: impl FnOnce(&AssertionContext<'_, R>) -> Result<Success, Rejection>,
        explain: impl FnOnce(
            Rejection,
            FailureBuilder<Attached<'a>>,
            &AssertionContext<'_, R>,
        ) -> FailureBuilder<Attached<'a>>,
    ) -> Option<Success> {
        let context = self.assertion_context();
        match observe(&context) {
            Ok(success) => Some(success),
            Err(rejection) => {
                explain(rejection, self.failure_at(kind, location), &context).raise();
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Expectation, RenderingBudget,
        failure::{FailureBuilder, FailureKind},
        prelude::*,
    };
    use core::cell::Cell;

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
            assert_that!(ToHumanReadableText.render(&failures[0]).as_str()).contains("Actual: 7");
        }
    }

    struct Expected<'e> {
        text: &'e str,
        calls: &'e Cell<usize>,
    }

    impl<R> Expectation<String, R> for Expected<'_> {
        type Success<'a>
            = ()
        where
            Self: 'a,
            String: 'a;
        type Rejection<'a>
            = (&'a str, &'a str)
        where
            Self: 'a;

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
    }

    impl<R: ValueRenderer<str>> ExpectationDiagnostics<String, R> for Expected<'_> {
        const KIND: FailureKind = FailureKind::Other;

        fn explain<'a, Target>(
            &'a self,
            rejected: Option<(&'a String, Self::Rejection<'a>)>,
            failure: FailureBuilder<Target>,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder<Target> {
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
            .capture(|it| it.apply_assertion(&definition));
        assert_that!(calls.get()).is_equal_to(1);
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].kind).is_equal_to(FailureKind::Other);
        assert_that!(failures[0].subject_name.as_deref()).is_equal_to(Some("subject"));
        assert_that!(failures[0].expression).is_equal_to(Some("actual"));
        assert_that!(failures[0].location).is_none();

        let matcher_failures = assert_that!(actual)
            .with_location(false)
            .with_subject_name("subject")
            .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(2))
            .capture(|it| it.matches(&definition));
        assert_that!(calls.get()).is_equal_to(2);
        assert_that!(matcher_failures).has_length(1);
        assert_that!(matcher_failures[0]).is_equal_to(failures[0].clone());
        assert_that!(matcher_failures[0].children).is_empty();

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
