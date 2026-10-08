//! Assertions for heap-backed values and captured panic payloads.

/// Assertions that downcast boxed `Any` values and captured panic payloads.
pub mod boxed;

/// Assertion traits for heap-backed values and panic payloads.
pub mod prelude {
    pub use super::boxed::{BoxAssertions, BoxExtractAssertions};
}
