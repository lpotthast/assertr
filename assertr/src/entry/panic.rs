use alloc::boxed::Box;
use core::any::Any;

/// A captured panic payload used as the subject of panic-value assertions.
///
/// Only the `panics` and `panics_async` function assertions create this subject, and they require
/// the `std` feature. Without `std`, the type exists but no value of it can be produced. Its
/// payload is type-erased, like the `Box<dyn Any + Send>` returned by `std::panic::catch_unwind`.
/// Use the [`BoxAssertions`](crate::assertions::BoxAssertions) and
/// [`BoxExtractAssertions`](crate::assertions::BoxExtractAssertions) methods to
/// inspect it.
pub struct PanicValue(pub(crate) Box<dyn Any + Send>);

#[cfg(test)]
mod tests {
    #[test]
    fn panic_value_keeps_the_payload_send() {
        fn assert_send<T: Send>() {}
        assert_send::<crate::PanicValue>();
    }
}
