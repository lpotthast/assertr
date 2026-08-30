//! Non-blocking assertions for Tokio synchronization primitives.

/// Assertions for Tokio mutex state.
pub mod mutex;
/// Assertions for Tokio read-write lock state.
pub mod rw_lock;
/// Assertions and extraction for Tokio watch receivers.
pub mod watch;

/// Tokio assertion traits.
pub mod prelude {
    pub use super::mutex::TokioMutexAssertions;
    pub use super::rw_lock::TokioRwLockAssertions;
    pub use super::watch::TokioWatchReceiverAssertions;
    pub use super::watch::TokioWatchReceiverExtractAssertions;
}
