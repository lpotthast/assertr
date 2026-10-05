//! Assertions for consuming iterators, borrowed iteration, and exact remaining counts.

mod exact_size;
mod into_iterator;
mod iterator;

pub use exact_size::{
    ExactSizeIteratorAssertions, HasNoRemainingElements, HasRemainingCount, HasRemainingElements,
};
pub use into_iterator::IntoIteratorAssertions;
pub use iterator::IteratorAssertions;
