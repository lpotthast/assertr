use alloc::{
    rc::Rc,
    string::{String, ToString},
};

use crate::{
    AssertThat,
    failure::adapter::{Adapter, AdapterExt, HumanReadableText},
    mode::Mode,
};

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
        self.state.include_location = value;
        self
    }

    /// Selects the adapter that produces this context's panic text.
    ///
    /// The default is [`ToHumanReadableText`](crate::failure::adapter::ToHumanReadableText).
    /// This method takes ownership of an adapter returning [`HumanReadableText`], displayed by
    /// the panic. The adapter must be `'static`, so it cannot borrow stack-local data. Move data
    /// into the adapter, clone owned values such as [`String`], or share owned data through [`Rc`].
    /// This bound does not require the adapter to live forever. It is dropped when the last
    /// context using it is dropped, and the subject's borrow can still end at the context's last use.
    ///
    /// The adapter's error can be any type implementing [`core::fmt::Display`]. This method
    /// wraps the adapter to convert its errors to [`String`] only when presentation runs.
    ///
    /// It runs on the asserting thread and needs neither `Send`, `Sync`, nor `Clone`. Mapped and
    /// derived assertions share the adapter through an internal [`Rc`]. Calling this method
    /// again replaces the selected presentation for this context.
    ///
    /// Presentation receives an already-built [`AssertionFailure`](crate::AssertionFailure).
    /// Use [`with_renderer`](Self::with_renderer) to customize individual diagnostic values and
    /// [`with_rendering_budget`](Self::with_rendering_budget) to limit them before presentation.
    ///
    /// Capture mode stores structured failures without invoking presentation. In panic mode a
    /// returned adapter error falls back to the built-in report with a presentation diagnostic.
    /// With `std`, an unwinding adapter panic also uses this fallback. Without `std`, adapter
    /// panics propagate because unwind catching is unavailable. Assertr never logs the report
    /// to stdout automatically.
    ///
    /// ```no_run
    /// use core::convert::Infallible;
    /// use assertr::failure::adapter::{
    ///     Adapter, AdapterExt, HumanReadableText, ToHumanReadableText,
    /// };
    /// use assertr::prelude::*;
    ///
    /// struct AddContext(String);
    ///
    /// impl Adapter<HumanReadableText> for AddContext {
    ///     type Output = HumanReadableText;
    ///     type Error = Infallible;
    ///
    ///     fn adapt(&self, text: &HumanReadableText) -> Result<HumanReadableText, Infallible> {
    ///         Ok(HumanReadableText::new(format!("{}\n{text}", self.0)))
    ///     }
    /// }
    ///
    /// let context = String::from("Integration check failed:");
    /// let presentation = ToHumanReadableText.then(AddContext(context.clone()));
    /// assert_that!(1)
    ///     .with_panic_presentation(presentation)
    ///     .is_equal_to(2);
    /// ```
    #[must_use]
    pub fn with_panic_presentation<A>(mut self, adapter: A) -> Self
    where
        A: Adapter<crate::AssertionFailure, Output = HumanReadableText> + 'static,
        A::Error: core::fmt::Display,
    {
        let adapter = adapter.map_err(|error| error.to_string());
        self.state.panic_presentation = Some(Rc::new(adapter));
        self
    }
}
