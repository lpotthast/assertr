//! Assertion entry points and the subjects created by them.

#[cfg(feature = "fluent")]
mod fluent;
mod panic;
mod type_subject;

#[cfg(feature = "fluent")]
pub use fluent::{IntoAssertContext, IntoOwnedAssertContext};
pub use panic::PanicValue;
#[cfg(all(test, not(feature = "std")))]
pub(crate) use panic::no_std_test_support::assert_that_panic_by;
#[cfg(feature = "std")]
pub use panic::{assert_that_panic_by, assert_that_panic_by_async};
pub use type_subject::{Type, assert_that_type};

/// The main macro entry point into an assertion context. Borrows its input.
///
/// `assert_that!(value)` borrows `value`, so a named value remains usable after the assertion.
/// Temporaries and literals live until the end of the enclosing statement. For a sized pointee,
/// `assert_that!(&value)` and `assert_that!(value)` are equivalent: a reference expression is
/// unwrapped one level, so both yield an `AssertThat<Value>`. References to unsized targets such
/// as `str` and `[T]` remain reference-typed subjects.
///
/// For assertions that consume their subject, such as `panics()` on a closure or terminal iterator
/// assertions, use [`crate::assert_that_owned!`] instead.
///
/// ```
/// use assertr::prelude::*;
///
/// let value = String::from("hello");
/// assert_that!(value).starts_with("hel");
/// assert_that!(value.len()).is_equal_to(5); // `value` is still usable.
/// ```
#[macro_export]
macro_rules! assert_that {
    ($e:expr) => {
        $crate::__private::assert_that_macro::Wrap {
            inner: $crate::__private::assert_that_macro::Fallback(&$e),
        }
        .into_assert_that()
        .with_expression(::core::stringify!($e))
    };
}

/// Macro entry point into an assertion context that takes ownership of its input.
///
/// Use this for assertions that consume their subject, such as `panics()` on a closure or terminal
/// iterator assertions. Prefer [`assert_that!`] when ownership is not required because it keeps
/// the value usable.
///
/// ```
/// use assertr::prelude::*;
///
/// assert_that_owned!([1, 2, 3].into_iter()).contains(2);
/// ```
#[macro_export]
macro_rules! assert_that_owned {
    ($e:expr) => {
        $crate::__private::assert_that_macro::owned($e).with_expression(::core::stringify!($e))
    };
}
