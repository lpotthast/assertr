use alloc::string::String;

use crate::{AssertThat, mode::Mode};

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Sets the subject name shown in failure messages.
    #[must_use]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.state.subject_name = Some(subject_name.into());
        self
    }

    /// Sets the source expression shown in the backticked `Expression:` field of failure messages.
    ///
    /// [`assert_that!`](crate::assert_that) and [`assert_that_owned!`](crate::assert_that_owned)
    /// set this automatically to the tokens used in the macro parenthesis.
    ///
    /// Derived child chains start a new diagnostic subject and do not inherit their parent
    /// expression.
    #[must_use]
    pub fn with_expression(mut self, expression: &'static str) -> Self {
        self.state.expression = Some(expression);
        self
    }

    /// Controls whether failures record the source file, line, and column.
    ///
    /// Disable locations when comparing a rendered failure exactly in a test.
    ///
    /// Assertions derived from this one (through `satisfies` and friends) inherit the setting.
    #[must_use]
    pub fn with_location(mut self, value: bool) -> Self {
        self.state.print_location = value;
        self
    }
}
