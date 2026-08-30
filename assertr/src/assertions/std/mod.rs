//! Assertions for standard-library types requiring the `std` feature.

/// Assertions for process commands.
pub mod command;
/// Assertions about a type's memory properties.
pub mod mem;
/// Assertions for mutex state.
pub mod mutex;
/// Assertions for paths.
pub mod path;

/// Standard-library assertion traits.
pub mod prelude {
    pub use super::command::CommandAssertions;
    pub use super::mem::MemAssertions;
    pub use super::mutex::MutexAssertions;
    pub use super::path::PathAssertions;
}
