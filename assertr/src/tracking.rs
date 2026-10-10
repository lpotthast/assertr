use crate::{AssertThat, ChainRecords, prelude::Mode};

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Records that one assertion was performed on this chain.
    ///
    /// Every assertion is tracked exactly once, before checking or invoking user code, whether it
    /// passes or fails. Reusable leaf checks implement
    /// [`Expectation`](crate::expectation::Expectation) and delegate to [`AssertThat::matches`]
    /// or [`AssertThat::test_assertion`], which track for them, as the example below does.
    /// Methods delegating to these or to other tracked assertions must not track again.
    /// [`AssertThat::capture`] and the fluent `verify` use the count to reject a closure that
    /// performed no assertions at all, so an assertion that forgets to track makes a passing
    /// capture closure panic as if it had been empty.
    ///
    /// Call this method directly only in an execution adapter, which owns an invocation,
    /// consumption, or polling step that the borrowed expectation protocol cannot express. Such an
    /// adapter tracks before its operation and builds any failure through [`AssertThat::failure`].
    /// See [custom assertions](crate#custom-assertions) for a complete reusable definition.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// use assertr::matchers::predicate;
    ///
    /// trait EvenAssertions {
    ///     fn is_even(self) -> Self;
    /// }
    ///
    /// impl<M: Mode, R: ValueRenderer<u32>> EvenAssertions for AssertThat<'_, u32, M, R> {
    ///     #[track_caller]
    ///     fn is_even(self) -> Self {
    ///         self.matches(predicate(|value: &u32| value % 2 == 0)
    ///             .described_as("is even"))
    ///     }
    /// }
    ///
    /// assert_that!(42).is_even();
    /// ```
    pub fn track_assertion(&self) {
        self.state.records.track_assertion();
    }
}

impl ChainRecords<'_> {
    pub(crate) fn assertion_count(&self) -> usize {
        self.number_of_assertions.get()
    }

    fn track_assertion(&self) {
        self.number_of_assertions
            .set(self.number_of_assertions.get() + 1);

        // Propagate to the parent, so that assertions made on a derived assertion also count for
        // the chain it was derived from.
        if let Some(parent) = self.parent {
            parent.track_assertion();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn dropping_an_unused_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _unused = assert_that!(42).with_location(false);
        });
        assert_that!(result).is_ok();
    }

    #[test]
    fn dropping_an_unused_assert_during_unwinding_preserves_the_original_panic() {
        assert_that!(|| {
            let _assert = assert_that!(42);
            panic!("original panic");
        })
        .panics()
        .has_type::<&str>()
        .is_equal_to("original panic");
    }

    #[test]
    fn number_of_assertions_are_tracked() {
        let initial_assertions = assert_that!(42).is_equal_to(42).is_not_equal_to(43);

        assert_that!(initial_assertions.state.records.assertion_count()).is_equal_to(2);

        let derived_assertions = initial_assertions.derive_owned(|it| it * 2).is_equal_to(84);

        assert_that!(initial_assertions.state.records.assertion_count()).is_equal_to(3);
        assert_that!(derived_assertions.state.records.assertion_count()).is_equal_to(1);
    }

    #[test]
    fn capture_counts_each_assertion_once_across_projections_and_renderer_changes() {
        let failures = assert_that!(42).is_equal_to(42).capture(|root| {
            assert_that!(root.state.records.assertion_count()).is_equal_to(0);

            let root = root.is_equal_to(42);
            let child = root.derive_owned(|it| it * 2).is_equal_to(84);
            assert_that!(root.state.records.assertion_count()).is_equal_to(2);
            assert_that!(child.state.records.assertion_count()).is_equal_to(1);

            let root = root.with_renderer(DebugRenderer).is_equal_to(43);
            assert_that!(root.state.records.assertion_count()).is_equal_to(3);
            root
        });
        assert_that!(failures).has_length(1);
    }
}
