//! General-purpose assertions and assertions for core-language value families.
//!
//! - Equality, ordering, formatting, and patterns: [`PartialEqAssertions`](partial_eq::PartialEqAssertions),
//!   [`PartialOrdAssertions`](partial_ord::PartialOrdAssertions),
//!   [`DebugAssertions`](debug::DebugAssertions), [`DisplayAssertions`](display::DisplayAssertions),
//!   and [`PatternAssertions`](pattern::PatternAssertions)
//! - Primitive and structural values: [`BoolAssertions`](bool::BoolAssertions),
//!   [`CharAssertions`](char::CharAssertions), [`LengthAssertions`](length::LengthAssertions),
//!   [`RangeAssertions`](range::RangeAssertions), and
//!   [`RangeBoundAssertions`](range::RangeBoundAssertions)
//! - State and extraction: the assertion and extraction traits in [`option`], [`result`], and
//!   [`poll`], plus [`RefCellAssertions`](ref_cell::RefCellAssertions)
//! - Iteration: [`IteratorAssertions`](iter::IteratorAssertions),
//!   [`IntoIteratorAssertions`](iter::IntoIteratorAssertions), and
//!   [`ExactSizeIteratorAssertions`](iter::ExactSizeIteratorAssertions)
//! - String-like values: [`StrAssertions`](string::StrAssertions)
//!
//! Function and async-function assertions appear in the `fn` module when the `std` feature is
//! enabled.

/// Boolean assertions.
pub mod bool;
/// Character assertions.
pub mod char;
/// Assertions over a value's `Debug` representation.
pub mod debug;
/// Assertions over a value's `Display` representation.
pub mod display;
#[cfg(feature = "std")]
/// Assertions that invoke synchronous or asynchronous functions.
pub mod r#fn;
/// Iterator and borrowed-iteration assertions.
pub mod iter;
/// Assertions for subjects implementing [`crate::assertions::HasLength`].
pub mod length;
/// `Option` state and extraction assertions.
pub mod option;
/// Equality and inequality assertions.
pub mod partial_eq;
/// Partial-order assertions.
pub mod partial_ord;
/// Pattern-matching assertions.
pub mod pattern;
/// `Poll` state and extraction assertions.
pub mod poll;
/// Range membership assertions.
pub mod range;
/// `RefCell` borrow-state assertions.
pub mod ref_cell;
/// `Result` state and extraction assertions.
pub mod result;
/// Assertions for string-like subjects.
pub mod string;

/// General-purpose assertion traits.
pub mod prelude {
    pub use super::bool::BoolAssertions;
    pub use super::char::CharAssertions;
    pub use super::debug::DebugAssertions;
    pub use super::display::DisplayAssertions;
    // All inner fn's are already std-gated, so we remove this otherwise noise-generating export.
    #[cfg(feature = "std")]
    pub use super::r#fn::AsyncFnOnceAssertions;
    // All inner fn's are already std-gated, so we remove this otherwise noise-generating export.
    #[cfg(feature = "std")]
    pub use super::r#fn::FnOnceAssertions;
    pub use super::iter::{
        ExactSizeIteratorAssertions, IntoIteratorAssertions, IteratorAssertions,
    };
    pub use super::length::LengthAssertions;
    pub use super::option::OptionAssertions;
    pub use super::option::OptionExtractAssertions;
    pub use super::partial_eq::PartialEqAssertions;
    pub use super::partial_ord::PartialOrdAssertions;
    pub use super::pattern::PatternAssertions;
    pub use super::poll::PollAssertions;
    pub use super::poll::PollExtractAssertions;
    pub use super::range::RangeAssertions;
    pub use super::range::RangeBoundAssertions;
    pub use super::ref_cell::RefCellAssertions;
    pub use super::result::ResultAssertions;
    pub use super::result::ResultExtractAssertions;
    pub use super::string::StrAssertions;
}

pub(crate) fn strip_quotation_marks(mut str: &str) -> &str {
    if str.starts_with('"') {
        str = str.strip_prefix('"').unwrap();
    }
    if str.ends_with('"') {
        str = str.strip_suffix('"').unwrap();
    }
    str
}
