//! Execution of reusable expectations and one-use observations on assertion chains.

use core::panic::Location;

use crate::{
    AssertThat, Mode,
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    mode::Panic,
};

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Supplies the chain's rendering settings and location policy for expectation evaluation.
    pub(crate) fn assertion_context(&self) -> AssertionContext<'_, R> {
        self.state.settings.assertion_context(&self.state.renderer)
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
    /// In panic mode, a rejection panics, so [`require`](Self::require) returns the observation
    /// without an `Option`. Use this method in code generic over the mode.
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
    #[cfg(any(feature = "std", test))]
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
    /// Asserts a reusable expectation and returns its successful observation.
    ///
    /// Panic mode raises every rejection, so a returned value always exists. Use this to continue
    /// with what an expectation observed, such as a parsed value or a found element. In code
    /// generic over the mode, use [`test_assertion`](Self::test_assertion) instead.
    ///
    /// ```
    /// use assertr::{matchers::IsSome, prelude::*};
    ///
    /// let maybe_port = Some(8080);
    /// let chain = assert_that!(maybe_port);
    /// let port: &u16 = chain.require(&IsSome);
    /// assert_that!(port).is_equal_to(8080);
    /// ```
    ///
    /// This tracks one assertion, just like [`matches`](Self::matches). A method that delegates
    /// here must use `#[track_caller]` and must not track the assertion again.
    ///
    /// # Panics
    ///
    /// Panics with the explained failure if the expectation rejects the subject.
    #[track_caller]
    pub fn require<'a, D: Expectation<T, R>>(&'a self, definition: &'a D) -> D::Success<'a> {
        self.test_assertion(definition)
            .expect("panic mode raises rejected expectations")
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::*;
    use crate::{prelude::*, renderer::RenderingBudget, test_support::NoRenderer};

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::{matchers::eq, prelude::*};

        #[test]
        fn are_as_expected() {
            1.must().match_expectation(eq(1));
        }
    }

    mod matches {
        use indoc::indoc;

        use super::*;
        use crate::matchers::*;

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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), matches(eq(2)));
        }

        #[test]
        fn does_not_require_a_renderer() {
            assert_that!(())
                .with_renderer(NoRenderer)
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
        fn tracks_before_evaluation_can_panic() {
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
        fn uses_the_equality_failure_directly() {
            let failures = assert_that!(1)
                .with_location(false)
                .capture(|it| it.matches(eq(2)));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(FailureKind::Equality);
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

        #[test]
        fn retains_the_borrowed_rejection_without_evaluating_again() {
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

    mod test_assertion {
        use crate::{matchers::eq, prelude::*};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), test_assertion(&eq(2)));
        }
    }

    mod require {
        use crate::{matchers::IsSome, prelude::*};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(None::<i32>), require(&IsSome));
        }

        #[test]
        fn returns_the_successful_observation() {
            let chain = assert_that!(Some(7));
            assert_that!(chain.require(&IsSome)).is_equal_to(7);
        }
    }

    mod one_use {
        use core::cell::RefCell;

        use super::*;

        #[test]
        fn success_transfers_a_guard_without_renderer_support_or_repeated_observation() {
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
            assert_that!(value.try_borrow_mut()).is_err();
            drop(success);
            assert_that!(value.try_borrow_mut()).is_ok();
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
                        assert_that!(value.try_borrow_mut()).is_err();
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
}
