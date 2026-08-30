//! Assertions for heap-backed values and captured panic payloads.

/// Assertions that downcast boxed `Any` values.
pub mod boxed;
/// Assertions that inspect captured panic payloads.
pub mod panic_value;

/// Assertion traits for heap-backed values and panic payloads.
pub mod prelude {
    pub use super::boxed::BoxAssertions;
    pub use super::panic_value::PanicValueAssertions;
}
