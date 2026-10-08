//! General-purpose assertions and assertions for core-language value families.
//!
//! - Equality, ordering, formatting, and patterns:
//!   [`PartialEqAssertions`](partial_eq::PartialEqAssertions),
//!   [`IdentityAssertions`](identity::IdentityAssertions),
//!   [`PartialOrdAssertions`](partial_ord::PartialOrdAssertions),
//!   [`DebugAssertions`](debug::DebugAssertions),
//!   [`DisplayAssertions`](display::DisplayAssertions), and
//!   [`PatternAssertions`](pattern::PatternAssertions)
//! - Primitive and structural values: [`BoolAssertions`](bool::BoolAssertions),
//!   [`CharAssertions`](char::CharAssertions), [`LengthAssertions`](length::LengthAssertions),
//!   [`RangeAssertions`](range::RangeAssertions), and
//!   [`RangeBoundAssertions`](range::RangeBoundAssertions)
//! - State and extraction: the assertion and extraction traits in [`option`], [`result`], and
//!   [`poll`], plus [`RefCellAssertions`](ref_cell::RefCellAssertions)
//! - Iteration: [`IteratorAssertions`](iter::IteratorAssertions),
//!   [`IntoIteratorAssertions`](iter::IntoIteratorAssertions),
//!   [`ExactSizeIteratorAssertions`](iter::ExactSizeIteratorAssertions)
//! - String-like values: [`StrAssertions`](string::StrAssertions)
//! - Type memory properties: [`MemAssertions`](mem::MemAssertions)
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
/// Reference-identity assertions without equality or rendering bounds.
pub mod identity;
/// Iterator and borrowed-iteration assertions.
pub mod iter;
/// Assertions for subjects implementing [`crate::assertions::HasLength`].
pub mod length;
/// Assertions about a type's memory properties.
pub mod mem;
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
