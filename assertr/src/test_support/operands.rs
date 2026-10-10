//! Expected operands that observe how often their borrowed views are resolved.

use alloc::string::String;
use core::borrow::Borrow;

use borrow_for::BorrowFor;

/// Calls the observer whenever a user operand is resolved.
pub(crate) struct BorrowSpy<T, F> {
    pub(crate) value: T,
    pub(crate) observe: F,
}

impl<T, F: Fn()> Borrow<T> for BorrowSpy<T, F> {
    fn borrow(&self) -> &T {
        (self.observe)();
        &self.value
    }
}

impl<T, F: Fn()> BorrowFor<T> for BorrowSpy<T, F> {
    type View = T;
}

/// A non-Debug operand whose unsized string view is observed on every borrow.
pub(crate) struct StrOperand<F> {
    pub(crate) value: &'static str,
    pub(crate) observe: F,
}

impl<F: Fn()> Borrow<str> for StrOperand<F> {
    fn borrow(&self) -> &str {
        (self.observe)();
        self.value
    }
}

impl<F: Fn()> BorrowFor<String> for StrOperand<F> {
    type View = str;
}
