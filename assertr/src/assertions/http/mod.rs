//! Assertions and panic-mode extraction for [`http::HeaderValue`].

/// Assertions and extraction for HTTP header values.
pub mod header_value;

/// HTTP assertion traits.
pub mod prelude {
    pub use super::header_value::HttpHeaderValueAssertions;
    pub use super::header_value::HttpHeaderValueExtractAssertions;
}
