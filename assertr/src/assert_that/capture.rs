use alloc::{string::String, vec::Vec};
use core::{cell::RefCell, marker::PhantomData, panic::AssertUnwindSafe};

use crate::{
    AssertThat, AssertionFailures, ChainRecords, ChainState,
    actual::Actual,
    details::WithDetail,
    mode::{Capture, Panic},
    renderer::RenderingContext,
    tracking::NumberOfAssertions,
};

/// Runs a callback on an isolated capture chain with the supplied rendering and location settings.
///
/// Descendant assertions contribute to the capture root. User panics propagate, and an empty
/// callback is rejected before its failures are returned.
pub(crate) fn collect_assertions<A, R, F>(
    actual: &A,
    rendering: RenderingContext<'_, R>,
    include_location: bool,
    assertions: F,
) -> AssertionFailures
where
    R: Clone,
    F: for<'a> FnOnce(AssertThat<'a, A, Capture, R>),
{
    let sink = AssertThat::new_capturing(Actual::Borrowed(actual))
        .with_renderer(rendering.renderer().clone())
        .with_rendering_budget(rendering.budget())
        .with_location(include_location);
    assertions(sink.derive(|value| value));
    assert!(
        sink.state.records.assertion_count() != 0,
        "The closure passed to satisfying performed no assertions!"
    );
    sink.state.records.failures.take()
}

impl<'t, R> ChainState<'t, Panic, R> {
    fn into_capturing(self, messages: Vec<String>) -> ChainState<'t, Capture, R> {
        ChainState {
            records: ChainRecords {
                parent: None,
                detail_messages: AssertUnwindSafe(RefCell::new(messages)),
                // Validate work performed by the capture closure, not preceding panic-mode work.
                number_of_assertions: AssertUnwindSafe(RefCell::new(NumberOfAssertions::new())),
                failures: self.records.failures,
            },
            subject_name: self.subject_name,
            expression: self.expression,
            include_location: self.include_location,
            rendering_budget: self.rendering_budget,
            panic_presentation: self.panic_presentation,
            mode: PhantomData,
            renderer: self.renderer,
        }
    }
}

impl<'t, T, R> AssertThat<'t, T, Panic, R> {
    /// Runs the given assertions in capture mode and returns the collected failures as structured
    /// [`crate::AssertionFailure`] values. An empty result means every assertion passed.
    ///
    /// Use this when a test or validation step should report several failed checks together. The
    /// closure receives this chain in capture mode and must return it, or a mapped continuation,
    /// so its failures can be extracted. Keep the final assertion as the closure's return value.
    /// The returned [`AssertionFailures`] supports slice access, iteration, collection assertions,
    /// and conversion to a vector through [`AssertionFailures::into_vec`].
    ///
    /// ```rust
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!(42).capture(|it| it.is_less_than(0).is_equal_to(43));
    ///
    /// assert_that!(failures).contains_exactly_satisfying([
    ///     |failure: AssertThat<AssertionFailure, Capture>| {
    ///         failure.derive_owned(AssertionFailure::kind).is_equal_to(assertr::FailureKind::Ordering);
    ///         failure.derive_owned(|failure| ToHumanReadableText.render(failure))
    ///             .contains("is not less than");
    ///     },
    ///     |failure: AssertThat<AssertionFailure, Capture>| {
    ///         failure.derive_owned(AssertionFailure::kind).is_equal_to(assertr::FailureKind::Equality);
    ///     },
    /// ]);
    /// ```
    ///
    /// Each [`crate::AssertionFailure`] exposes its values, relation, facts, and nested failures as
    /// data. Inspect those fields directly or pass the failure to an
    /// [adapter](crate::failure::adapter).
    /// [`ToHumanReadableText`](crate::failure::adapter::ToHumanReadableText) produces the default
    /// report. Capture mode never invokes the chain's
    /// [panic presentation](Self::with_panic_presentation).
    ///
    /// Assertions on [projections](Self::derive) within the closure contribute their failures
    /// to the same result. Calling `capture` on an already-derived assertion instead starts a
    /// separate collection for that child. Its failures do not propagate to the panic-mode parent,
    /// and existing ancestor detail messages remain attached.
    ///
    /// With the `fluent` feature, `value.verify(...)` and `value.verify_owned(...)` enter capture
    /// mode directly. `capture` itself needs no optional feature. It collects assertion failures,
    /// but does not catch panics from user code.
    ///
    /// # Panics
    ///
    /// Panics if the closure performed no assertions.
    #[track_caller]
    #[must_use = "the captured failures must be inspected; chain assertions without `capture` to panic on failure instead"]
    pub fn capture<F, U: 't, R2>(self, assertions: F) -> AssertionFailures
    where
        F: FnOnce(AssertThat<'t, T, Capture, R>) -> AssertThat<'t, U, Capture, R2>,
    {
        self.into_capturing().run_and_collect(assertions)
    }

    fn into_capturing(self) -> AssertThat<'t, T, Capture, R> {
        // Sever the parent link: `capture` scopes failure collection to this chain, so failures
        // must not propagate to (and get lost in) a panic-mode ancestor. Ancestor detail messages
        // are preserved by flattening them into this chain.
        let mut messages = Vec::new();
        self.collect_messages(&mut messages);

        let AssertThat { actual, state } = self;
        AssertThat {
            actual,
            state: state.into_capturing(messages),
        }
    }
}

impl<'t, T, R> AssertThat<'t, T, Capture, R> {
    /// Runs the given assertion closure and extracts the collected failures from the assertion it
    /// returns. Shared implementation of [`AssertThat::capture`] and the fluent `verify` entry
    /// points.
    #[track_caller]
    pub(crate) fn run_and_collect<F, U: 't, R2>(self, assertions: F) -> AssertionFailures
    where
        F: FnOnce(Self) -> AssertThat<'t, U, Capture, R2>,
    {
        let completed = assertions(self);
        assert!(
            completed.state.records.assertion_count() != 0,
            "The closure passed to `capture` / `verify` performed no assertions!"
        );
        completed.state.records.failures.take()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use indoc::formatdoc;

    #[test]
    fn returned_context_collects_projections_and_renderer_changes_once() {
        let failures = assert_that!([1, 2])
            .with_detail_message("root detail")
            .capture(|it| {
                it.derive(|values| &values[0]).is_equal_to(3);
                it.with_renderer(DebugRenderer).contains(4)
            });
        assert_that!(failures).contains_exactly_satisfying(
            [|failure: AssertThat<crate::AssertionFailure, Capture>| {
                failure
                    .derive(|failure| &failure.messages)
                    .contains("root detail");
            }; 2],
        );
    }

    #[test]
    fn owned_iterator_returns_a_context_that_finishes_capture() {
        let failures = assert_that_owned!([1, 2].into_iter())
            .capture(|it| -> AssertThat<'_, (), Capture> { it.contains(3) });
        assert_that!(failures).has_length(1);
        assert_that!(assert_that_owned!(0..).capture(|it| it.starts_with([0, 1]))).is_empty();
    }

    #[test]
    #[cfg(feature = "fluent")]
    #[crate::fluent_expressions]
    fn fluent_verification_collects_returned_contexts() {
        let failures = 1.verify(|it| it.be_equal_to(2));
        assert_that!(failures[0].expression).is_equal_to(Some("1"));
        assert_that!([1, 2].into_iter().verify_owned(|it| it.contain(3))).has_length(1);
        assert_that!(1.verify(|it| it.be_equal_to(1))).is_empty();
    }

    #[test]
    fn capture_yields_failures_and_does_not_panic() {
        let failures = assert_that!(42)
            .with_location(false)
            .capture(|it| it.is_greater_than(100).is_equal_to(1));

        assert_that!(failures.as_slice()).contains_exactly_satisfying([
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_text_report(formatdoc! {"
                    -------- assertr --------
                    Expression: `42`

                    Actual: 42

                    is not greater than

                    Expected: 100
                    -------- assertr --------
                "});
            },
            |it: AssertThat<AssertionFailure, Capture>| {
                it.has_text_report(formatdoc! {"
                    -------- assertr --------
                    Expression: `42`

                    Expected: 1

                      Actual: 42
                    -------- assertr --------
                "});
            },
        ]);
    }
}
