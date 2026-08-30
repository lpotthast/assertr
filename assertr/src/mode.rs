//! The two assertion modes: fail immediately, or collect failures.
//!
//! [`Mode`] is sealed. [`Panic`] and [`Capture`] are its only implementations.

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Panic {}
    impl Sealed for super::Capture {}
}

/// The mode of an assertion, deciding what happens when an assertion fails.
///
/// This trait is sealed. [`Panic`] and [`Capture`] are its only implementations and are type-state
/// markers, not extension points. Every assertion derived from a root assertion retains the
/// root's mode.
pub trait Mode: sealed::Sealed + 'static {
    /// Whether failures are collected for later inspection (`true`) or raise an immediate panic
    /// (`false`).
    const CAPTURES: bool;
}

/// Panic mode, in which the first failure panics immediately.
///
/// This is the default mode. Projections that cannot produce a continuation after failure, such
/// as `get_ok`, are available only in this mode.
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
