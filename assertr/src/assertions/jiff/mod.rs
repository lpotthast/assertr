//! Assertions for `jiff` durations, spans, and zoned date-times.

/// Generates the `IsZero`, `IsNegative`, and `IsPositive` expectations for a signed `jiff` type.
///
/// `present` optionally wraps every rendered value, for example to keep a compact form.
macro_rules! sign_expectations {
    ($subject:ident, zero: $zero:expr $(, present: $present:path)? $(,)?) => {
        property_expectation! {
            #[doc = concat!("Checks whether a `", stringify!($subject), "` is zero.")]
            pub struct IsZero for $subject;
            kind Equality;
            check |actual| actual.is_zero();
            expected $zero, "is zero";
            $(present $present;)?
        }

        property_expectation! {
            #[doc = concat!("Checks whether a `", stringify!($subject), "` is negative.")]
            pub struct IsNegative for $subject;
            kind Ordering;
            check |actual| actual.is_negative();
            relations "is negative", "is not negative";
            $(present $present;)?
        }

        property_expectation! {
            #[doc = concat!("Checks whether a `", stringify!($subject), "` is positive.")]
            pub struct IsPositive for $subject;
            kind Ordering;
            check |actual| actual.is_positive();
            relations "is positive", "is not positive";
            $(present $present;)?
        }
    };
}

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
