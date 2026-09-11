//! Macro-only expression-aware support for terminal fluent entry points.

use core::{marker::PhantomData, panic::Location};

use crate::{AssertThat, AssertionFailures, mode::Capture};

/// Retains a type-only callback probe while preserving the callback value and its coercions.
pub fn remember_callback<F>(callback: F, kind: &mut Option<Callback<F>>) -> F {
    *kind = Some(Callback(PhantomData));
    callback
}

/// A type-only probe that never invokes or changes the callback.
pub struct Callback<F>(PhantomData<fn() -> F>);

/// Recognizes a resolved callback that accepts a capture-mode assertion chain.
pub trait CaptureCallback<'t, T: 't, R, Output> {
    /// Permits expression attachment for an assertion callback.
    fn accepts_capture(self) -> bool;
}

impl<'t, T: 't, R, Output, F> CaptureCallback<'t, T, R, Output> for &Callback<F>
where
    F: FnOnce(AssertThat<'t, T, Capture, R>) -> Output,
{
    fn accepts_capture(self) -> bool {
        true
    }
}

/// Autoref fallback for unrelated callback inputs, other arities, and non-callable arguments.
pub trait CaptureCallbackFallback {
    /// Prevents unrelated arguments from granting expression attachment.
    fn accepts_capture(self) -> bool;
}

impl<F> CaptureCallbackFallback for &&Callback<F> {
    fn accepts_capture(self) -> bool {
        false
    }
}

/// Completes expression attachment without changing the original call's result type.
///
/// Keeping `T` on both sides supplies the generated closure's expected input type before its
/// specialized attachment is resolved. The macro places this call at the original method span,
/// so its caller location identifies the same entry as the runtime's tracked fluent method.
#[track_caller]
pub fn finish<T>(result: T, attach: impl FnOnce(T, &'static Location<'static>) -> T) -> T {
    attach(result, Location::caller())
}

/// Borrows a completed result for type-directed expression attachment.
pub struct AttachExpression<'a, T>(&'a mut T);

impl<'a, T> AttachExpression<'a, T> {
    /// Wraps a result without changing its type or taking ownership of it.
    #[must_use]
    pub fn new(result: &'a mut T) -> Self {
        Self(result)
    }
}

impl AttachExpression<'_, AssertionFailures> {
    /// Attaches the expression only to failures originating at this fluent entry.
    pub fn attach(self, expression: &'static str, location: &'static Location<'static>) {
        self.0.attach_expression(expression, location);
    }
}

impl<T, R> AttachExpression<'_, AssertThat<'_, T, Capture, R>> {
    /// Attaches an inline closure's receiver expression before it runs assertions.
    pub fn attach_to_input(self, expression: &'static str) {
        self.0.state.expression = crate::Expression::Explicit(expression);
    }
}

/// Fallback for results from unrelated methods with fluent entry names.
pub trait AttachExpressionFallback {
    /// Leaves an unrelated result unchanged.
    fn attach(self, expression: &'static str, location: &'static Location<'static>);

    /// Leaves an unrelated inline callback input unchanged.
    fn attach_to_input(self, expression: &'static str);
}

impl<T> AttachExpressionFallback for AttachExpression<'_, T> {
    fn attach(self, _: &'static str, _: &'static Location<'static>) {}

    fn attach_to_input(self, _: &'static str) {}
}
