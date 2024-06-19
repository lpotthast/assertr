use crate::{failure::ExpectedActualFailure, AssertThat, Mode};
use std::fmt::Debug;

// General Assertions
impl<'t, T: Debug, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn has_debug_value(self, expected: impl Debug) -> Self {
        let actual = format!("{:?}", self.actual());
        let expected = format!("{:?}", expected);

        if actual != expected {
            self.fail(ExpectedActualFailure {
                expected: &expected,
                actual: &actual,
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {}
