use alloc::{string::String, sync::Arc};

use crate::{AssertThat, mode::Mode};

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Sets the subject name shown in failure messages.
    #[must_use]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.state.settings.subject_name = Some(subject_name.into());
        self
    }

    /// Sets the source expression shown in the backticked `Expression:` field of failure messages.
    ///
    /// The entry macros and `#[fluent_expressions]` record it through
    /// `__private::with_expression`. Derived child chains start a new diagnostic subject and do not
    /// inherit their parent expression.
    #[must_use]
    pub(crate) fn with_expression(mut self, expression: &'static str) -> Self {
        self.state.settings.expression = crate::Expression::Explicit(expression);
        self
    }

    /// Controls whether failures record the source file, line, and column.
    ///
    /// Disable locations when comparing a rendered failure exactly in a test.
    ///
    /// Assertions derived from this one (through `satisfies` and friends) inherit the setting.
    #[must_use]
    pub fn with_location(mut self, value: bool) -> Self {
        self.state.settings.include_location = value;
        self
    }

    /// Selects the closure that produces this chain's panic text.
    ///
    /// The default panic text is the failure's `Display` report. The closure receives the
    /// already-built [`AssertionFailure`](crate::failure::AssertionFailure) and returns the text
    /// displayed by the panic. It can wrap the default report, which `failure.to_string()`
    /// produces, or build any other representation from the failure's structured fields.
    ///
    /// The closure must be `'static`, so it cannot borrow stack-local data. Move data into it,
    /// clone owned values such as [`String`], or share owned data through [`Arc`]. This bound does
    /// not require the closure to live forever. It is dropped when the last chain using it is
    /// dropped, and the subject's borrow can still end at the chain's last use.
    ///
    /// It must be `Send` and `Sync`, because an [eventual
    /// assertion](crate::assertions::EventualAssertions) keeps it while awaiting, possibly on
    /// another thread. It needs no `Clone`: mapped and derived assertions share the closure
    /// through an internal [`Arc`]. Calling this method again replaces the selected
    /// presentation for this chain.
    ///
    /// The closure must implement [`RefUnwindSafe`](core::panic::RefUnwindSafe), since its
    /// concrete type is erased and shared by chains that may cross a `catch_unwind` boundary. A
    /// closure capturing unprotected shared mutable state is rejected.
    ///
    /// Use [`with_renderer`](Self::with_renderer) to customize individual diagnostic values and
    /// [`with_rendering_budget`](Self::with_rendering_budget) to limit them before presentation.
    ///
    /// Capture mode stores structured failures without invoking presentation. In panic mode with
    /// `std`, a panicking closure falls back to the built-in report with a presentation
    /// diagnostic. Without `std`, its panics propagate because unwind catching is unavailable.
    /// Assertr never logs the report to stdout automatically.
    ///
    /// ```should_panic
    /// use assertr::prelude::*;
    ///
    /// let context = String::from("Integration check failed:");
    /// assert_that!(1)
    ///     .with_panic_presentation(move |failure| format!("{context}\n{failure}"))
    ///     .is_equal_to(2);
    /// ```
    #[must_use]
    pub fn with_panic_presentation(
        mut self,
        presentation: impl Fn(&crate::failure::AssertionFailure) -> String
        + core::panic::RefUnwindSafe
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.state.settings.panic_presentation = Some(Arc::new(presentation));
        self
    }
}
