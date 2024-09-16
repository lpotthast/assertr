use crate::{prelude::Mode, AssertThat};

pub(crate) struct NumAssertions(usize);

impl NumAssertions {
    pub(crate) const fn new() -> Self {
        Self(0)
    }
}

impl Drop for NumAssertions {
    fn drop(&mut self) {
        if self.0 == 0 {
            panic!("An AssertThat was dropped without performing any actual assertions on it!");
        }
    }
}

pub(crate) trait AssertionTracking {
    fn track_assertion(&self);
}

impl<'t, T, M: Mode> AssertionTracking for AssertThat<'t, T, M> {
    /// Track that a single assertion was made / is about to be checked.
    fn track_assertion(&self) {
        self.num_assertions.borrow_mut().0 += 1;
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
        assert_that_panic_by(|| assert_that(42).with_location(false))
            .has_type::<&str>()
            .is_equal_to(
                "An AssertThat was dropped without performing any actual assertions on it!",
            );
    }
}
