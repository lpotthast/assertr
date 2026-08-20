use crate::{AssertThat, prelude::Mode};

pub(crate) struct NumberOfAssertions(usize);

impl NumberOfAssertions {
    pub(crate) const fn new() -> Self {
        Self(0)
    }
}

impl Drop for NumberOfAssertions {
    fn drop(&mut self) {
        if crate::enforce_drop_contracts() {
            assert!(
                self.0 != 0,
                "An AssertThat was dropped without performing any actual assertions on it!"
            );
        }
    }
}

pub(crate) trait AssertionTracking {
    fn track_assertion(&self);
}

impl<T, M: Mode, R> AssertionTracking for AssertThat<'_, T, M, R> {
    /// Track that a single assertion was made / is about to be checked.
    fn track_assertion(&self) {
        self.number_of_assertions.borrow_mut().0 += 1;

        // If we don't propagate to our parent that an assertion was made, we could drop a parent
        // `AssertThat` value, which was only used to derive another `AssertThat` on which then
        // assertions were made.
        // We would unexpectedly panic because we think nothing was asserted.
        if let Some(parent) = self.parent {
            parent.track_assertion();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    #[cfg(feature = "std")]
    fn panics_on_drop_when_no_assertions_were_made() {
        assert_that_panic_by(|| assert_that!(42).with_location(false))
            .has_type::<&str>()
            .is_equal_to(
                "An AssertThat was dropped without performing any actual assertions on it!",
            );
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn panics_on_drop_when_no_assertions_were_made_async() {
        assert_that_panic_by_async(async || assert_that!(42).with_location(false))
            .await
            .has_type::<&str>()
            .is_equal_to(
                "An AssertThat was dropped without performing any actual assertions on it!",
            );
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

        assert_that!(initial_assertions.number_of_assertions.borrow().0).is_equal_to(2);

        let derived_assertions = initial_assertions.derive(|it| it * 2).is_equal_to(84);

        assert_that!(initial_assertions.number_of_assertions.borrow().0).is_equal_to(3);
        assert_that!(derived_assertions.number_of_assertions.borrow().0).is_equal_to(1);
    }
}
