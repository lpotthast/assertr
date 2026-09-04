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
            print_location: self.print_location,
            rendering_budget: self.rendering_budget,
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
    /// Runs the given assertions in capture mode and returns the collected failures as
    /// structured [`AssertionFailure`] values. An empty result means every assertion passed.
    ///
    /// The closure receives this assertion in capture mode and returns it, or a mapped
    /// continuation, so its failures can be extracted.
    ///
    /// On a derived assertion, `capture` returns that child's failures instead of propagating them
    /// to its panic-mode parent. Existing ancestor detail messages remain attached.
    ///
    /// ```rust
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!(42).capture(|it| it.is_less_than(0).is_equal_to(43));
    ///
    /// assert_that!(failures).has_length(2);
    /// ```
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
        // must not propagate to (and get lost in) a panic-mode ancestor. Ancestor detail
        // messages are preserved by flattening them into this chain.
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
    /// Runs the given assertion closure and extracts the collected failures from the assertion
    /// it returns. Shared implementation of [`AssertThat::capture`] and the fluent `verify`
    /// entry points.
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
