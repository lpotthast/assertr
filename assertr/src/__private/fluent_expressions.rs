//! Macro-only expression-aware support for terminal fluent entry points.

use core::ops::{Deref, DerefMut};

use crate::{AssertThat, mode::Capture};

/// Constrains an input to the original callback's argument type before attaching an expression.
///
/// The macro calls the callback directly so its `Fn`, `FnMut`, or `FnOnce` capabilities remain
/// available on the generated closure.
pub fn callback_input<A, B, F>(_: &F, input: A) -> A
where
    F: FnOnce(A) -> B,
{
    input
}

/// Wrapper used to attach an expression only to capture-mode assertion callback inputs.
///
/// The specialized inherent `attach` method wins for `AssertThat<_, Capture>`. Other callback
/// inputs reach [`AttachExpressionFallback::attach`] through `DerefMut` and remain unchanged.
pub struct AttachExpression<T> {
    value: AttachExpressionFallback<T>,
    expression: &'static str,
}

impl<T> AttachExpression<T> {
    /// Wraps a callback input and the expression to attach when the input is an assertion chain.
    #[must_use]
    pub fn new(value: T, expression: &'static str) -> Self {
        Self {
            value: AttachExpressionFallback(Some(value)),
            expression,
        }
    }
}

impl<'t, T, R> AttachExpression<AssertThat<'t, T, Capture, R>> {
    /// Attaches the expression to a capture-mode assertion callback input.
    #[must_use]
    pub fn attach(mut self) -> AssertThat<'t, T, Capture, R> {
        self.value
            .0
            .take()
            .expect("the fluent-expression callback input is present")
            .with_expression(self.expression)
    }
}

impl<T> Deref for AttachExpression<T> {
    type Target = AttachExpressionFallback<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for AttachExpression<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// Autoref-specialization fallback for callback inputs unrelated to assertr.
pub struct AttachExpressionFallback<T>(Option<T>);

impl<T> AttachExpressionFallback<T> {
    /// Returns an unrelated callback input unchanged.
    #[must_use]
    pub fn attach(&mut self) -> T {
        self.0
            .take()
            .expect("the fluent-expression callback input is present")
    }
}
