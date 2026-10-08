//! The two assertion modes: fail immediately, or collect failures.
//!
//! A chain's mode decides what happens when one of its assertions fails. In [`Panic`] mode, the
//! default, the first failure panics with its report. In [`Capture`] mode, failures are collected
//! and returned by [`AssertThat::capture`](crate::AssertThat::capture). Child chains share the mode
//! of their root.
//!
//! Assertions that extract a value, such as `get_some` or `get_ok`, exist only in panic mode. After
//! a failure there is no value to continue with. In capture mode, use the `*_satisfying` variants,
//! such as `is_some_satisfying`, instead.
//!
//! Write custom assertions generic over `M: Mode` to support both modes. [`Mode`] is sealed, so
//! [`Panic`] and [`Capture`] are its only implementations.

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Panic {}
    impl Sealed for super::Capture {}
}

/// The mode of an assertion, deciding what happens when an assertion fails.
///
/// This trait is sealed. [`Panic`] and [`Capture`] are its only implementations and are type-state
/// markers, not extension points. Every assertion derived from a root assertion retains the root's
/// mode.
pub trait Mode:
    sealed::Sealed + core::panic::UnwindSafe + core::panic::RefUnwindSafe + 'static
{
    /// Whether failures are collected for later inspection (`true`) or raise an immediate panic
    /// (`false`).
    const CAPTURES: bool;
}

/// Panic mode, in which the first failure panics immediately.
///
/// This is the default mode. Projections that cannot produce a continuation after failure, such as
/// `get_ok`, are available only in this mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Panic;

/// Capture mode, in which failures are collected instead of panicking.
///
/// [`crate::AssertThat::capture`] and the fluent `verify` entry points return the collected
/// failures when their closure completes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capture;

impl Mode for Panic {
    const CAPTURES: bool = false;
}

impl Mode for Capture {
    const CAPTURES: bool = true;
}
