//! Comparison functions for the `AssertrEq` derive.
//!
//! These are named in `#[assertr_eq(compare_with = "...")]` attributes so a derived
//! partial-equality comparison can descend into collections while collecting human-readable
//! differences.

mod equality;

pub use equality::{AssertrPartialEq, Differences, Eq, EqContext, any, eq};

#[cfg(feature = "std")]
/// Partial comparison for hash maps.
pub mod hashmap;
/// Partial comparison for slices.
pub mod slice;
