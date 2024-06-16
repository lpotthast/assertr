use crate::{failure::ExpectedActualFailure, AssertThat};
use std::fmt::Display;

// General Assertions
impl<'t, T: Display> AssertThat<'t, T> {
    #[track_caller]
    pub fn has_display_value(self, expected: impl Display) -> Self {
        let actual = format!("{}", self.actual.borrowed());
        let expected = format!("{}", expected);

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
