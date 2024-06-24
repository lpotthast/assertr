use crate::{failure::ExpectedActualFailure, tracking::AssertionTracking, AssertThat, Mode};
use std::fmt::Display;

impl<'t, T: Display, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn has_display_value(self, expected: impl Display) -> Self {
        self.track_assertion();

        let actual = format!("{}", self.actual());
        let expected = format!("{}", expected);

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
