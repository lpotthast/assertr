//! Assertions and panic-mode projections for [`reqwest::Response`].

/// Assertions and projections for HTTP responses.
pub mod response;

/// Reqwest assertion traits.
pub mod prelude {
    pub use super::response::ReqwestResponseAssertions;
    pub use super::response::ReqwestResponseExtractAssertions;
}
