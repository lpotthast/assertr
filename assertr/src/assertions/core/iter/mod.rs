//! Assertions for consuming iterators and exact remaining counts.

mod exact_size;
mod iterator;

pub use exact_size::{
    ExactSizeIteratorAssertions, HasNoRemainingElements, HasRemainingCount, HasRemainingElements,
};
pub use iterator::IteratorAssertions;
