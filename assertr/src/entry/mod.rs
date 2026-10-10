//! Assertion entry points and the subjects created by them.

#[cfg(feature = "fluent")]
mod fluent;
mod panic;
mod type_subject;

#[cfg(feature = "fluent")]
pub use fluent::{FluentEntry, OwnedFluentEntry};
pub use panic::PanicValue;
pub(crate) use panic::panic_message;
pub use type_subject::{Type, assert_that_type};

/// The main macro entry point into an assertion chain. Borrows its input.
///
/// `assert_that!(value)` borrows `value`, so a named value remains usable after the assertion.
/// Temporaries and literals live until the end of the enclosing statement. For a sized pointee,
/// `assert_that!(&value)` and `assert_that!(value)` are equivalent: a reference expression is
/// unwrapped one level, so both yield an `AssertThat<Value>`. References to unsized targets such as
/// `str` and `[T]` remain reference-typed subjects.
///
/// A closure literal or `async` block is a temporary that nothing else can use, so `assert_that!`
/// takes ownership of it. Assertions that run a closure, such as `panics()`, therefore work
/// directly. For other assertions that consume their subject, such as iterator assertions, or for a
/// named closure, use [`crate::assert_that_owned!`].
///
/// ```
/// use assertr::prelude::*;
///
/// let value = String::from("hello");
/// assert_that!(value).starts_with("hel");
/// assert_that!(value.len()).is_equal_to(5); // `value` is still usable.
///
/// # #[cfg(feature = "std")]
/// assert_that!(|| value.parse::<u32>().unwrap()).panics();
/// ```
#[macro_export]
macro_rules! assert_that {
    // Closure literals and `async` blocks are owned. These arms only match code that starts with
    // closure syntax or `async`, so every other expression still borrows.
    (|| $($closure:tt)*) => {
        $crate::assert_that_owned!(|| $($closure)*)
    };
    (| $($closure:tt)*) => {
        $crate::assert_that_owned!(| $($closure)*)
    };
    (move $($closure:tt)*) => {
        $crate::assert_that_owned!(move $($closure)*)
    };
    (async $($closure:tt)*) => {
        $crate::assert_that_owned!(async $($closure)*)
    };
    ($e:expr) => {
        $crate::__private::with_expression(
            $crate::__private::assert_that_macro::Wrap {
                inner: $crate::__private::assert_that_macro::Fallback(&$e),
            }
            .into_assert_that(),
            ::core::stringify!($e),
        )
    };
}

/// Macro entry point into an assertion chain that takes ownership of its input.
///
/// Use this for assertions that consume their subject, such as iterator assertions or running a
/// named closure. Prefer [`assert_that!`] otherwise, because it keeps the value usable. Closure
/// literals need no `assert_that_owned!`, because `assert_that!` already owns them.
///
/// ```
/// use assertr::prelude::*;
///
/// assert_that_owned!([1, 2, 3].into_iter()).contains(2);
/// ```
#[macro_export]
macro_rules! assert_that_owned {
    ($e:expr) => {
        $crate::__private::with_expression(
            $crate::__private::assert_that_macro::owned($e),
            ::core::stringify!($e),
        )
    };
}

#[cfg(test)]
mod tests {
    mod assert_that {
        use crate::prelude::*;

        #[test]
        fn owns_closure_literals_and_records_their_source() {
            let failures = assert_that!(|| 1)
                .with_location(false)
                .capture(|it| it.matches(matchers::anything()));
            assert_that!(failures).is_empty();

            let subject = assert_that!(|| 1);
            assert_that!(subject.state.settings.expression.explicit()).is_equal_to(Some("|| 1"));
            assert_that!(subject.unwrap_inner()()).is_equal_to(1);
        }

        #[test]
        fn owns_closures_with_parameters_and_move_closures() {
            let offset = 1;
            assert_that!(assert_that!(|value: i32| value + offset).unwrap_inner()(1))
                .is_equal_to(2);
            assert_that!(assert_that!(move || offset).unwrap_inner()()).is_equal_to(1);
        }

        #[test]
        fn owns_async_closures_and_blocks() {
            let closure = assert_that!(async || 1).unwrap_inner();
            assert_that!(crate::test_support::block_on(closure())).is_equal_to(1);
            let block = assert_that!(async { 2 }).unwrap_inner();
            assert_that!(crate::test_support::block_on(block)).is_equal_to(2);
        }

        #[test]
        fn borrows_other_expressions() {
            let value = String::from("hello");
            assert_that!(value).has_length(5);
            assert_that!(value).starts_with("he"); // `value` was not moved.
        }
    }
}
