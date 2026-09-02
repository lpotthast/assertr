//! Macro-only expression-aware support for terminal fluent entry points.

use core::ops::{Deref, DerefMut};

use crate::{AssertThat, mode::Capture};

/// Preserves the selected entry method's callback input and output constraints across the macro's
/// expression-attachment closure.
pub fn adapt_callback<A, B, F, G>(assertions: F, attach_expression: G) -> impl FnOnce(A) -> B
where
    F: FnOnce(A) -> B,
    G: FnOnce(A) -> A,
{
    move |assertion| assertions(attach_expression(assertion))
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
