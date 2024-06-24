use crate::{failure::ExpectedActualFailure, tracking::AssertionTracking, AssertThat, Mode};
use std::fmt::Debug;

impl<'t, T: Debug, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn has_debug_value(self, expected: impl Debug) -> Self {
        self.track_assertion();

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
mod tests {
    // TODO
}
