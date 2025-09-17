use crate::{AssertThat, prelude::Mode};

pub(crate) struct NumberOfAssertions(usize);

impl NumberOfAssertions {
    pub(crate) const fn new() -> Self {
        Self(0)
    }
}

impl Drop for NumberOfAssertions {
    fn drop(&mut self) {
        if self.0 == 0 {
            panic!("An AssertThat was dropped without performing any actual assertions on it!");
        }
    }
}

pub(crate) trait AssertionTracking {
    fn track_assertion(&self);
}

impl<T, M: Mode> AssertionTracking for AssertThat<'_, T, M> {
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
    fn panics_on_drop_when_no_assertions_were_made() {
        assert_that_panic_by(|| 42.must().with_location(false))
            .has_type::<&str>()
            .is_equal_to(
                "An AssertThat was dropped without performing any actual assertions on it!",
            );
    }

    #[test]
    fn number_of_assertions_are_tracked() {
        let initial_assertions = 42.must().be_equal_to(42).be_positive();

        initial_assertions
            .number_of_assertions
            .borrow()
            .0
            .must()
            .be_equal_to(2);

        let derived_assertions = initial_assertions.derive(|it| it * 2).be_equal_to(84);

        initial_assertions
            .number_of_assertions
            .borrow()
            .0
            .must()
            .be_equal_to(3);
        derived_assertions
            .number_of_assertions
            .borrow()
            .0
            .must()
            .be_equal_to(1);
    }
}
