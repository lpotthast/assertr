use core::{cell::Cell, marker::PhantomData, panic::AssertUnwindSafe};

use crate::{
    AssertThat, AssertionFailures, ChainRecords, ChainState,
    mode::{Capture, Panic},
};

impl<'t, R> ChainState<'t, Panic, R> {
    fn into_capturing(self) -> ChainState<'t, Capture, R> {
        // Sever the parent link: `capture` scopes failure collection to this chain, so failures
        // must not propagate to (and get lost in) a panic-mode ancestor. Ancestor detail messages
        // are preserved as inherited messages, which keep following every local message.
        let mut inherited_messages = self.records.inherited_messages;
        if let Some(parent) = self.records.parent {
            parent.collect_messages(&mut inherited_messages);
        }
        ChainState {
            records: ChainRecords {
                parent: None,
                detail_messages: self.records.detail_messages,
                // Validate work performed by the capture closure, not preceding panic-mode work.
                number_of_assertions: AssertUnwindSafe(Cell::new(0)),
                failures: self.records.failures,
                inherited_messages,
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
    ///         failure.derive(|failure| &failure.kind).is_equal_to(assertr::FailureKind::Ordering);
    ///         failure.derive_owned(|failure| failure.to_string())
    ///             .contains("is not less than");
    ///     },
    ///     |failure: AssertThat<AssertionFailure, Capture>| {
    ///         failure.derive(|failure| &failure.kind).is_equal_to(assertr::FailureKind::Equality);
    ///     },
    /// ]);
    /// ```
    ///
    /// Each [`crate::AssertionFailure`] exposes its values, relation, facts, and nested failures as
    /// data. Inspect those fields directly. Its `Display` implementation produces the default
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
        self.into_capturing().collect_failures(assertions)
    }

    fn into_capturing(self) -> AssertThat<'t, T, Capture, R> {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual,
            state: state.into_capturing(),
        }
    }
}

impl<'t, T, R> AssertThat<'t, T, Capture, R> {
    /// Runs an assertion callback on this capture root and takes the failures collected by the
    /// chain it returns. Shared by [`AssertThat::capture`], the fluent `verify` entry points, and
    /// assertion-callback matchers.
    ///
    /// # Panics
    ///
    /// Panics if the callback performed no assertions.
    #[track_caller]
    pub(crate) fn collect_failures<F, U: 't, R2>(self, assertions: F) -> AssertionFailures
    where
        F: FnOnce(Self) -> AssertThat<'t, U, Capture, R2>,
    {
        let completed = assertions(self);
        assert!(
            completed.state.records.assertion_count() != 0,
            "the assertion callback performed no assertions"
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
            [|failure: AssertThat<AssertionFailure, Capture>| {
                failure
                    .derive(|failure| &failure.messages)
                    .contains("root detail");
            }; 2],
        );
    }

    mod detail_messages {
        use super::*;

        #[test]
        fn capture_on_a_derived_chain_keeps_local_messages_before_ancestor_messages() {
            let root = assert_that!(5).with_detail_message("parent");
            let child = root.derive(|it| it).with_detail_message("child-1");
            let failures = child.capture(|it| it.with_detail_message("child-2").is_equal_to(6));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].messages).contains_exactly(["child-1", "child-2", "parent"]);
        }

        #[test]
        fn capture_on_a_root_chain_keeps_insertion_order() {
            let failures = assert_that!(5)
                .with_detail_message("first")
                .capture(|it| it.with_detail_message("second").is_equal_to(6));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].messages).contains_exactly(["first", "second"]);
        }
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

        assert_that!(&failures[..]).contains_exactly_satisfying([
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
