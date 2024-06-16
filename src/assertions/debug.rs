use crate::{failure::ExpectedActualFailure, AssertThat};
use std::fmt::Debug;

// General Assertions
impl<'t, T: Debug> AssertThat<'t, T> {
    #[track_caller]
    pub fn has_debug_value(self, expected: impl Debug) -> Self {
        let actual = format!("{:?}", self.actual.borrowed());
        let expected = format!("{:?}", expected);

        if actual != expected {
            self.fail_with(ExpectedActualFailure {
                expected: &expected,
                actual: &actual,
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {}
