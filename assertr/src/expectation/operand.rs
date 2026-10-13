use core::borrow::Borrow;

use crate::borrow_for::BorrowFor;

/// An expected value that can be compared with values of type `Context`, through the borrowed
/// view its [`BorrowFor<Context>`](crate::borrow_for::BorrowFor) implementation selects.
///
/// Comparing assertions such as `is_equal_to`, `contains`, `contains_key`, and the `eq` matcher
/// accept any expected value implementing this trait. Every
/// [`BorrowFor<Context>`](crate::borrow_for::BorrowFor) operand implements it, so implement
/// [`BorrowFor`](crate::borrow_for::BorrowFor) for a custom operand rather than this trait. The
/// trait exists to explain a rejected operand: `PartialEq` between the two types is not enough,
/// because assertr borrows the expected value as the type it compares.
///
/// ```compile_fail
/// use assertr::prelude::*;
///
/// struct Meters(u32);
///
/// impl PartialEq<u32> for Meters {
///     fn eq(&self, other: &u32) -> bool {
///         self.0 == *other
///     }
/// }
///
/// // `u32` cannot be used as an expected value for a `Meters` subject.
/// assert_that!(Meters(3)).is_equal_to(3_u32);
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as an expected value for `{Context}`",
    label = "no comparison view of `{Self}` for `{Context}`",
    note = "assertr borrows the expected value as the type it is compared with, so `PartialEq` between the two types is not enough",
    note = "pass a value of type `{Context}`, a reference to one, or a borrowed form such as `&str` for `String`",
    note = "or continue with a comparable subject first, such as with `.some()`, `.ok()`, or `.derive(..)`, or compare with `predicate(|actual| ..)`"
)]
pub trait ExpectedOperand<Context: ?Sized>: Borrow<Self::View> {
    /// The borrowed type compared with `Context`, chosen by
    /// [`BorrowFor<Context>`](crate::borrow_for::BorrowFor).
    type View: ?Sized;
}

#[diagnostic::do_not_recommend]
impl<C: ?Sized, E: BorrowFor<C> + ?Sized> ExpectedOperand<C> for E {
    type View = <E as BorrowFor<C>>::View;
}

/// Borrows the view that `operand` selects for `Context`.
pub(crate) fn operand_view<Context, E>(operand: &E) -> &E::View
where
    Context: ?Sized,
    E: ExpectedOperand<Context> + ?Sized,
{
    Borrow::<E::View>::borrow(operand)
}
