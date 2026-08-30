//! Assertions for `jiff` durations, spans, and zoned date-times.

/// Assertions for signed durations.
pub mod signed_duration;
/// Assertions for spans.
pub mod span;
/// Assertions for zoned date-times.
pub mod zoned;

/// Jiff assertion traits.
pub mod prelude {
    pub use super::signed_duration::SignedDurationAssertions;
    pub use super::span::SpanAssertions;
    pub use super::zoned::ZonedAssertions;
}
