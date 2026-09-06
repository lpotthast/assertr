use alloc::{string::String, vec::Vec};
use core::{cell::RefCell, marker::PhantomData};

use crate::{
    AssertThat, AssertionFailure, ChainState,
    details::WithDetail,
    mode::{Capture, Panic},
    tracking::NumberOfAssertions,
};

impl<'t, R> ChainState<'t, Panic, R> {
    fn into_capturing(self, messages: Vec<String>) -> ChainState<'t, Capture, R> {
        ChainState {
            parent: None,
            subject_name: self.subject_name,
            expression: self.expression,
            detail_messages: RefCell::new(messages),
            include_location: self.include_location,
            rendering_budget: self.rendering_budget,
            panic_presentation: self.panic_presentation,
            // `capture` validates the assertions performed by its closure, not work completed on
            // the panic-mode chain before capture began.
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: self.failures,
            mode: PhantomData,
            renderer: self.renderer,
        }
    }
}

impl<'t, T, R> AssertThat<'t, T, Panic, R> {
    /// Runs the given assertions in capture mode and returns the collected failures as structured
    /// [`AssertionFailure`] values. An empty result means every assertion passed.
    ///
    /// Use this when a test or validation step should report several failed checks together. The
    /// closure receives this chain in capture mode and must return it, or a mapped continuation,
    /// so its failures can be extracted. Keep the final assertion as the closure's return value.
    ///
    /// ```rust
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!(42).capture(|it| it.is_less_than(0).is_equal_to(43));
    ///
    /// assert_that!(failures).has_length(2);
    /// assert_eq!(failures[0].kind, assertr::FailureKind::Ordering);
    ///
    /// let report = ToHumanReadableText.render(&failures[0]);
    /// assert!(report.contains("is not less than"));
    /// ```
    ///
    /// Each [`AssertionFailure`] exposes its values, relation, facts, and nested failures as data.
    /// Inspect those fields directly or pass the failure to an [adapter](crate::failure::adapter).
    /// [`ToHumanReadableText`](crate::failure::adapter::ToHumanReadableText) produces the default
    /// report. Capture mode never invokes the chain's
    /// [panic presentation](Self::with_panic_presentation).
    ///
    /// Assertions on [projections](Self::satisfies) within the closure contribute their failures
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
    pub fn capture<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
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
    pub(crate) fn run_and_collect<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(Self) -> AssertThat<'t, U, Capture, R2>,
    {
        let completed = assertions(self);
        assert!(
            completed.state.number_of_assertions.borrow().0 != 0,
            "The closure passed to `capture` / `verify` performed no assertions!"
        );
        completed.state.failures.take()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use indoc::formatdoc;

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
