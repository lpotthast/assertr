use alloc::{boxed::Box, string::String};
use core::any::Any;

/// A captured panic payload used as the subject of panic-value assertions.
///
/// Only the `panics` and `panics_async` function assertions create this subject, and they require
/// the `std` feature. Without `std`, the type exists but no value of it can be produced. Its
/// payload is type-erased, like the `Box<dyn Any + Send>` returned by `std::panic::catch_unwind`.
/// Use [`has_message`](crate::assertions::BoxExtractAssertions::has_message) to check the message
/// of a `&str` or `String` payload, whichever `panic!` produced. Use the other
/// [`BoxExtractAssertions`](crate::assertions::BoxExtractAssertions) and
/// [`BoxAssertions`](crate::assertions::BoxAssertions) methods to inspect payloads of other types.
///
/// ```
/// use assertr::prelude::*;
///
/// # #[cfg(feature = "std")]
/// assert_that!(|| panic!("boom")).panics().has_message().is_equal_to("boom");
/// ```
pub struct PanicValue(pub(crate) Box<dyn Any + Send>);

/// The message of a panic payload raised through `panic!` or `panic_any` with a `&str` or a
/// `String`. A payload of any other type carries no message that could be shown.
pub(crate) fn panic_message(payload: &dyn Any) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

#[cfg(test)]
mod tests {
    use crate::test_support::assert_trait_impl;

    #[test]
    fn panic_value_keeps_the_payload_send() {
        assert_trait_impl!(crate::PanicValue => Send);
    }
}
