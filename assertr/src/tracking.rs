use crate::{AssertThat, prelude::Mode};

/// Counts the assertions performed on an assertion chain.
///
/// [`AssertThat::capture`] uses the count to reject capture closures that perform no assertions.
/// In panic mode, unused assertion contexts are caught at compile time instead, by the
/// `#[must_use]` annotations on the entry points.
pub(crate) struct NumberOfAssertions(pub(crate) usize);

impl NumberOfAssertions {
    pub(crate) const fn new() -> Self {
        Self(0)
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Records that one assertion was performed on this chain.
    ///
    /// Every assertion method must call this as its first statement, whether it ends up passing
    /// or failing. [`AssertThat::capture`] and the fluent `verify` use the count to reject a
    /// closure that performed no assertions at all, so an assertion that forgets to
    /// track makes a passing capture closure panic as if it had been empty.
    ///
    /// A handwritten leaf assertion calls this before raising a failure through
    /// [`AssertThat::failure`]. Assertions built by composing existing ones (through
    /// [`AssertThat::satisfies`] and friends) are tracked by the assertions they delegate to and
    /// must not call this in addition. See [custom assertions](crate#custom-assertions) for how to
    /// shape the trait around either kind of method.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// use assertr::failure::FailureKind;
    ///
    /// trait EvenAssertions<R = DebugRenderer> {
    ///     fn is_even(self) -> Self
    ///     where
    ///         R: ValueRenderer<u32>;
    /// }
    ///
    /// impl<M: Mode, R> EvenAssertions<R> for AssertThat<'_, u32, M, R> {
    ///     #[track_caller]
    ///     fn is_even(self) -> Self
    ///     where
    ///         R: ValueRenderer<u32>,
    ///     {
    ///         self.track_assertion();
    ///         if self.actual() % 2 != 0 {
    ///             self.failure(FailureKind::Predicate)
    ///                 .actual(self.render().value(self.actual()))
    ///                 .relation("is not even")
    ///                 .raise();
    ///         }
    ///         self
    ///     }
    /// }
    ///
    /// assert_that!(42).is_even();
    /// ```
    ///
    pub fn track_assertion(&self) {
        self.state.number_of_assertions.borrow_mut().0 += 1;

        // Propagate to the parent, so that assertions made on a derived assertion also count
        // for the chain it was derived from.
        if let Some(parent) = self.state.parent {
            parent.track_assertion_on_chain();
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
        assert_that!(result.is_ok()).is_true();
    }

    #[test]
    fn dropping_an_unused_assert_during_unwinding_preserves_the_original_panic() {
        assert_that_panic_by(|| {
            let _assert = assert_that!(42);
            panic!("original panic");
        })
        .has_type::<&str>()
        .is_equal_to("original panic");
    }

    #[test]
    fn number_of_assertions_are_tracked() {
        let initial_assertions = assert_that!(42).is_equal_to(42).is_not_equal_to(43);

        assert_that!(initial_assertions.state.number_of_assertions.borrow().0).is_equal_to(2);

        let derived_assertions = initial_assertions.derive_owned(|it| it * 2).is_equal_to(84);

        assert_that!(initial_assertions.state.number_of_assertions.borrow().0).is_equal_to(3);
        assert_that!(derived_assertions.state.number_of_assertions.borrow().0).is_equal_to(1);
    }
}
