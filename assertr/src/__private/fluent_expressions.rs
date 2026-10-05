//! Macro-only expression-aware support for terminal fluent entry points.

use core::panic::Location;

use crate::AssertionFailures;

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

/// Fallback for results from unrelated methods with fluent entry names.
pub trait AttachExpressionFallback {
    /// Leaves an unrelated result unchanged.
    fn attach(self, expression: &'static str, location: &'static Location<'static>);
}

impl<T> AttachExpressionFallback for AttachExpression<'_, T> {
    fn attach(self, _: &'static str, _: &'static Location<'static>) {}
}
