#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
// Allow functions named `is_*`, taking self by value instead of taking self by mutable reference or reference.
#![allow(clippy::wrong_self_convention)]

//! Fluent assertions for Rust, with `no_std` support.
//!
//! ```
//! use assertr::prelude::*;
//!
//! assert_that!([1, 2, 3]).contains(2).has_length(3);
//! ```
//!
//! # Entry points
//!
//! - [`assert_that!`] handles owned values and references transparently: `assert_that!(&value)` borrows,
//!   `assert_that!(value)` takes ownership.
//! - With the `fluent` feature, [`IntoAssertContext`] provides `value.must()` and `value.verify()`, which borrow the
//!   value and assert in panic mode (fail immediately) or capture mode (collect failures), plus `value.must_owned()`
//!   and `value.verify_owned()`, which take ownership instead.
//!
//! The borrowing and owning entry points are separate functions because Rust has no specialization to unify them
//! under one name. The borrowing variants deliberately carry the shorter, more prominent names: borrowing is the
//! more useful default, since the value remains usable for further assertions and surrounding code. Real ownership
//! is only rarely needed, e.g. for consuming assertions like `panics()`.
//!
//! # The mental model
//!
//! Four ideas explain this crate's design:
//!
//! 1. **[`AssertThat<T>`](AssertThat) is "an assertion about a `T`". Ownership is hidden inside.** The struct holds
//!    an [`Actual<T>`](actual::Actual) that is either owned or borrowed, and assertion methods are looked up by `T`
//!    alone. `assert_that!(&my_string)` therefore yields an `AssertThat<String>` holding a borrow, not an
//!    `AssertThat<&String>`. Reference-typed subjects such as `AssertThat<&str>` or `AssertThat<&[T]>` appear only
//!    where the target type itself is unsized.
//!
//! 2. **Derived assertions answer one question: "how do I assert on a part of my subject?"** A child `AssertThat`
//!    over the part is created, linked to its parent so that failures propagate to the root assertion.
//!    [`AssertThat::derive`] builds the child and hands it to you. The [`AssertThat::satisfies`] family builds it,
//!    runs your closure against it, and returns the original subject for further chaining.
//!
//! 3. **Three `satisfies` spellings exist only because of how the part is produced.** Use
//!    [`AssertThat::satisfies_borrowed`] when the part is borrowed from the subject (the common case),
//!    [`AssertThat::satisfies`] when it is computed or cloned, and [`AssertThat::satisfies_ref`] when it is unsized
//!    (`str`, `[T]`, ...). Conceptually this is one method. Rust cannot currently express a mapper that may either
//!    return an owned value or borrow from its input, forcing the split.
//!
//! 4. **Capture mode turns "did these assertions pass?" into a value.** In panic mode failures panic immediately.
//!    In capture mode (`verify()`, [`AssertThat::with_capture`]) they are collected and read out with
//!    [`AssertThat::capture_failures`]. The collection `_satisfying` assertions build on this: your assertion
//!    closure runs against each element in capture mode, and "no failures" is the matching criterion.

extern crate alloc;
extern crate core;
extern crate self as assertr;
#[cfg(all(test, not(feature = "std")))]
extern crate std;

use actual::Actual;
use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec::Vec};
use core::{
    any::{Any, type_name},
    cell::RefCell,
    fmt,
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    mem::needs_drop,
    panic::{RefUnwindSafe, UnwindSafe},
};
use details::WithDetail;
use failure::Fallible;
use mode::{Capture, Mode, Panic};
use tracking::{AssertionTracking, NumberOfAssertions};

#[doc(hidden)]
pub mod __private;
pub mod actual;
#[doc(hidden)]
pub mod assert_that_macro;
pub mod assertions;
pub mod cmp;
pub mod condition;
mod conversion;
pub mod details;
pub mod failure;
pub mod mode;
pub mod renderer;
pub mod tracking;
pub mod util;

pub use renderer::{
    AssertionRenderer, CustomRenderer, DebugRenderer, Renderable, RenderableValues,
};

pub mod prelude {
    // A `no_std` crate does not receive the standard prelude in its unit-test modules even though
    // the hosted test harness links `std`. Re-export the alloc prelude pieces those tests use,
    // without changing the production prelude or feature surface.
    #[cfg(all(test, not(feature = "std")))]
    pub(crate) use alloc::{
        borrow::ToOwned,
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    #[cfg(feature = "derive")]
    pub use assertr_derive::AssertrEq;

    #[cfg(feature = "fluent")]
    pub use crate::IntoAssertContext;
    pub use crate::any;
    #[allow(deprecated)]
    pub use crate::assert_that;
    #[allow(deprecated)]
    pub use crate::assert_that_owned;
    #[cfg(feature = "std")]
    pub use crate::assert_that_panic_by;
    #[cfg(feature = "std")]
    pub use crate::assert_that_panic_by_async;
    pub use crate::assert_that_type;
    pub use crate::assertions::HasLength;
    pub use crate::assertions::alloc::prelude::*;
    pub use crate::assertions::condition::ConditionAssertions;
    pub use crate::assertions::condition::IterableConditionAssertions;
    pub use crate::assertions::core::prelude::*;
    #[cfg(feature = "http")]
    pub use crate::assertions::http::prelude::*;
    #[cfg(feature = "jiff")]
    pub use crate::assertions::jiff::prelude::*;
    #[cfg(feature = "num")]
    pub use crate::assertions::num::NumAssertions;
    #[cfg(feature = "program")]
    pub use crate::assertions::program::Program;
    #[cfg(feature = "program")]
    pub use crate::assertions::program::ProgramAssertions;
    #[cfg(feature = "program")]
    pub use crate::assertions::program::ProgramAssertionsRequiringPanicMode;
    #[cfg(feature = "reqwest")]
    pub use crate::assertions::reqwest::prelude::*;
    #[cfg(feature = "rootcause")]
    pub use crate::assertions::rootcause::prelude::*;
    #[cfg(feature = "std")]
    pub use crate::assertions::std::prelude::*;
    #[cfg(feature = "tokio")]
    pub use crate::assertions::tokio::prelude::*;
    pub use crate::condition::Condition;
    #[cfg(feature = "serde")]
    pub use crate::conversion::json;
    #[cfg(feature = "serde")]
    pub use crate::conversion::toml;
    pub use crate::eq;
    pub use crate::mode::{Capture, Mode, Panic};
    #[cfg(all(test, not(feature = "std")))]
    pub(crate) use crate::no_std_test_support::assert_that_panic_by;
    pub use crate::pattern;
    pub use crate::{AssertThat, AssertionRenderer, DebugRenderer};
}

pub(crate) fn enforce_drop_contracts() -> bool {
    #[cfg(feature = "std")]
    {
        !std::thread::panicking()
    }

    #[cfg(not(feature = "std"))]
    {
        false
    }
}

pub struct PanicValue(Box<dyn Any>);

/// The main entrypoint into an assertion context for borrowed values.
///
/// Borrows the value, allowing it to be used after the assertion.
///
/// #### Example Usage
/// ```rust,no_run
/// use assertr::prelude::*;
///
/// // This will panic with a descriptive message and a pointer to the actual line of the assertion.
/// assert_that(&3).is_equal_to(4);
///
/// // This instead captures the assertion failure for later inspection.
/// let failures = assert_that(&3)
///     .with_capture()
///     .is_equal_to(4) // This will collect a failure instead of panicking.
///     .capture_failures();
///
/// assert_that(&failures)
///     .has_length(1)
///     .contains("");
/// ```
#[deprecated(
    since = "0.4.4",
    note = "Use the `assert_that!()` macro or the fluent `.must()` / `.verify()` entry points instead."
)]
#[track_caller]
#[must_use]
pub fn assert_that<T>(actual: &T) -> AssertThat<'_, T, Panic> {
    AssertThat::new_panicking(Actual::Borrowed(actual))
}

/// Entrypoint into an assertion context that takes ownership of the value.
///
/// Use this when the assertion requires ownership (e.g. `FnOnce` assertions).
/// For most cases, prefer [`assert_that()`] which borrows instead.
#[deprecated(
    since = "0.4.4",
    note = "Use the `assert_that!()` macro or the fluent `.must_owned()` / `.verify_owned()` entry points instead."
)]
#[track_caller]
#[must_use]
pub fn assert_that_owned<'t, T>(actual: T) -> AssertThat<'t, T, Panic> {
    AssertThat::new_panicking(Actual::Owned(actual))
}

/// Ergonomic macro entrypoint that handles both owned values and references.
///
/// Uses autoref specialization to transparently handle both
/// `assert_that!(owned_value)` and `assert_that!(&borrowed_value)` without
/// requiring the user to think about ownership.
#[macro_export]
macro_rules! assert_that {
    ($e:expr) => {
        $crate::assert_that_macro::Wrap {
            inner: $crate::assert_that_macro::Fallback(core::cell::Cell::new(Some($e))),
        }
        .into_assert_that()
    };
}

/// Fluent entry points into an assertion context, available on every value with the `fluent` feature.
///
/// `must()` and `verify()` borrow the value, `must_owned()` and `verify_owned()` take ownership. These are separate
/// methods because Rust has no specialization to unify them under one name. The borrowing variants deliberately
/// carry the shorter, more prominent names: borrowing is the more useful default, keeping the value usable for
/// further assertions and surrounding code. Ownership is only rarely required, e.g. for consuming assertions such
/// as `panics()`.
#[cfg(feature = "fluent")]
pub trait IntoAssertContext<'t, T> {
    /// Borrows the value and starts a panic-mode assertion: failures panic immediately.
    #[must_use]
    fn must(&'t self) -> AssertThat<'t, T, Panic>;

    /// Takes ownership of the value and starts a panic-mode assertion. Only needed when a consuming
    /// assertion requires ownership; prefer [`IntoAssertContext::must`] otherwise.
    #[must_use]
    fn must_owned(self) -> AssertThat<'t, T, Panic>;

    /// Borrows the value and starts a capture-mode assertion: failures are collected and must be
    /// read with [`AssertThat::capture_failures`].
    #[must_use]
    fn verify(&'t self) -> AssertThat<'t, T, Capture>;

    /// Takes ownership of the value and starts a capture-mode assertion. Only needed when a
    /// consuming assertion requires ownership; prefer [`IntoAssertContext::verify`] otherwise.
    #[must_use]
    fn verify_owned(self) -> AssertThat<'t, T, Capture>;
}

#[cfg(feature = "fluent")]
impl<'t, T> IntoAssertContext<'t, T> for T {
    fn must(&'t self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }
    fn must_owned(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Owned(self))
    }

    fn verify(&'t self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Borrowed(self))
    }
    fn verify_owned(self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Owned(self))
    }
}

#[cfg(feature = "fluent")]
impl<'t, T> IntoAssertContext<'t, T> for &'t T {
    fn must(&'t self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }
    fn must_owned(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }

    fn verify(&'t self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Borrowed(self))
    }
    fn verify_owned(self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Borrowed(self))
    }
}

#[cfg(feature = "fluent")]
impl<'t, T> IntoAssertContext<'t, T> for &'t mut T {
    fn must(&'t self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }
    fn must_owned(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }

    fn verify(&'t self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Borrowed(self))
    }
    fn verify_owned(self) -> AssertThat<'t, T, Capture> {
        AssertThat::new_capturing(Actual::Borrowed(self))
    }
}

#[track_caller]
#[must_use]
#[cfg(feature = "std")]
#[allow(deprecated)]
pub fn assert_that_panic_by<'t, R>(
    fun: impl FnOnce() -> R + 't,
) -> AssertThat<'t, PanicValue, Panic> {
    use crate::prelude::FnOnceAssertions;

    assert_that_owned(fun).panics()
}

// #[track_caller] // This is implied in the default async desugaring.
#[must_use]
#[cfg(feature = "std")]
#[allow(deprecated)]
pub async fn assert_that_panic_by_async<'t, F, Fut, R>(fun: F) -> AssertThat<'t, PanicValue, Panic>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = R> + UnwindSafe,
{
    use crate::prelude::AsyncFnOnceAssertions;

    assert_that_owned(fun).panics_async().await
}

#[cfg(all(test, not(feature = "std")))]
mod no_std_test_support {
    use super::{Actual, AssertThat, Panic, PanicValue};
    use core::panic::AssertUnwindSafe;

    /// Captures a panic for unit tests while the library itself is built without its `std` feature.
    ///
    /// The test harness is hosted and can therefore use `std`; this helper is crate-private and is
    /// never present in a production build.
    #[track_caller]
    pub(crate) fn assert_that_panic_by<'t, R>(
        fun: impl FnOnce() -> R + 't,
    ) -> AssertThat<'t, PanicValue, Panic> {
        let result = std::panic::catch_unwind(AssertUnwindSafe(fun));
        let result = std::panic::catch_unwind(AssertUnwindSafe(move || result.map(drop)));
        let panic = result
            .flatten()
            .expect_err("expected the tested function to panic");

        AssertThat::new_panicking(Actual::Owned(PanicValue(panic)))
    }
}

pub struct Type<T> {
    phantom: PhantomData<T>,
}

impl<T> Type<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn get_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    #[must_use]
    pub fn needs_drop(&self) -> bool {
        needs_drop::<T>()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        size_of::<T>()
    }
}

impl<T> Default for Type<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn assert_that_type<T>() -> AssertThat<'static, Type<T>, Panic> {
    AssertThat::new_panicking(Actual::Owned(Type::<T>::new()))
}

/// `AssertThat` is the core structure used for assertions. It allows developers to perform
/// assertions on actual values in a fluent and expressive manner, supporting detailed messages
/// as well as different modes of operation, such as panic or capture modes.
///
/// An `AssertThat<T>` is "an assertion about a `T`": whether the subject is owned or borrowed is
/// hidden inside the contained [`Actual<T>`](actual::Actual), and assertion methods are looked up
/// by `T` alone. `assert_that!(&value)` therefore yields an `AssertThat<Value>` holding a borrow,
/// not an `AssertThat<&Value>`. Reference-typed subjects such as `AssertThat<&str>` or
/// `AssertThat<&[T]>` appear only where the target type itself is unsized.
///
/// ### Type Parameters
/// - `'t`: The lifetime of the actual value being asserted.
/// - `T`: The type of the actual value being asserted.
/// - `M`: The assertion mode, implementing the [`Mode`] trait. Examples include `Panic` and `Capture` modes.
///
/// ### Fields
/// - `parent`: A reference to the parent assertion, if this is a derived assertion. Failures will propagate to the root assertion.
/// - `actual`: The value being asserted against.
/// - `subject_name`: An optional subject name for the assertion, allowing for more descriptive error messages.
/// - `detail_messages`: A collection of additional messages that provide context for the assertion.
/// - `print_location`: A boolean indicating whether the source code location of the assertion should be printed on failure.
/// - `number_of_assertions`: Tracks the number of assertions made.
/// - `failures`: A collection of failure messages for assertions in `Capture` mode.
/// - `mode`: The mode used for this assertion, determining behavior on failure.
///
/// ### Key Features
/// - **Fluent API**: Chainable and composable methods for making expressive assertions.
/// - **Detail Messages**: Add custom messages to provide context for failures.
/// - **Modes**:
///     - **Panic Mode**: The default mode where failures result in immediate panics.
///     - **Capture Mode**: Collect failures instead of panicking, useful for batch processing scenarios.
/// - **Derived Assertions**: Assertions derived from parent assertions, facilitating nested or mapped assertions.
///
/// ### Notes
/// - When using `Capture` mode, failures must be captured explicitly.
/// - Panic-on-drop checks for unused or uncaptured chains are disabled without the `std` feature,
///   because active unwinding cannot be detected in `core`.
/// - This struct is designed to handle both simple and complex assertion chaining scenarios.
pub struct AssertThat<'t, T, M: Mode, R = DebugRenderer> {
    // Derived assertions can be created. Calling `.fail*` on them should propagate to the root assertion!
    parent: Option<&'t dyn DynAssertThat>,

    actual: Actual<'t, T>,

    subject_name: Option<String>,
    detail_messages: RefCell<Vec<String>>,
    print_location: bool,

    number_of_assertions: RefCell<NumberOfAssertions>,
    failures: RefCell<Vec<String>>,

    mode: RefCell<M>,
    renderer: R,
}

pub(crate) trait DynAssertThat:
    Fallible + WithDetail + AssertionTracking + UnwindSafe + RefUnwindSafe
{
}
impl<T, M: Mode, R> DynAssertThat for AssertThat<'_, T, M, R> {}

impl<T, M: Mode, R> UnwindSafe for AssertThat<'_, T, M, R> {}
impl<T, M: Mode, R> RefUnwindSafe for AssertThat<'_, T, M, R> {}

impl<'t, T> AssertThat<'t, T, Panic> {
    #[track_caller]
    pub(crate) const fn new_panicking(actual: Actual<'t, T>) -> Self {
        AssertThat {
            parent: None,
            actual,
            subject_name: None,
            detail_messages: RefCell::new(Vec::new()),
            print_location: true,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(Panic::DEFAULT),
            renderer: DebugRenderer,
        }
    }
}

#[cfg(feature = "fluent")]
impl<'t, T> AssertThat<'t, T, Capture> {
    #[track_caller]
    pub(crate) const fn new_capturing(actual: Actual<'t, T>) -> Self {
        AssertThat {
            parent: None,
            actual,
            subject_name: None,
            detail_messages: RefCell::new(Vec::new()),
            print_location: true,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(Capture::DEFAULT),
            renderer: DebugRenderer,
        }
    }
}

impl<T, R> AssertThat<'_, T, Capture, R> {
    /// Extracts all assertion failures captured until now.
    ///
    /// Allows this `AssertThat` to be dropped again without raising a panic.
    ///
    /// ```rust
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!(42)
    ///     .with_capture()
    ///     .is_less_than(0)
    ///     .is_equal_to(43)
    ///     .capture_failures();
    ///
    /// assert_that!(failures).has_length(2);
    /// ```
    #[must_use]
    pub fn capture_failures(mut self) -> Vec<String> {
        self.take_failures()
    }

    /// Extracts all assertion failures captured until now.
    ///
    /// Allows this `AssertThat` to be dropped again without raising a panic.
    ///
    /// **Prefer `capture_failures` if you don't require ownership after extraction.**
    ///
    /// # Panics
    ///
    /// Panics if failures have already been captured.
    #[must_use]
    pub fn take_failures(&mut self) -> Vec<String> {
        let mut mode = self.mode.borrow_mut();
        assert!(
            !mode.captured,
            "You can only capture the assertion failures once!"
        );
        mode.captured = true;
        self.failures.take()
    }
}

impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R> {
    pub fn actual(&self) -> &T {
        self.actual.borrowed()
    }

    pub fn render_value<'a, U: ?Sized>(&'a self, value: &'a U) -> Renderable<'a, U, R> {
        Renderable {
            value,
            renderer: &self.renderer,
        }
    }

    pub fn render_values<'a, U>(&'a self, values: &'a [&'a U]) -> RenderableValues<'a, U, R> {
        RenderableValues {
            values,
            renderer: &self.renderer,
        }
    }

    pub(crate) fn eq_context(&self) -> EqContext<'_, R> {
        EqContext::with_renderer(&self.renderer)
    }

    pub(crate) fn replace_actual_with<'u, U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        new_actual: Actual<'u, U>,
    ) -> (Actual<'t, T>, AssertThat<'u, U, M, R>)
    where
        't: 'u,
    {
        let previous_actual: Actual<'t, T> = self.actual;
        let mapped = AssertThat {
            parent: self.parent,
            actual: new_actual,
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
            renderer: self.renderer,
        };
        (previous_actual, mapped)
    }

    pub fn map<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Actual<U>,
    ) -> AssertThat<'t, U, M, R> {
        AssertThat {
            parent: self.parent,
            actual: mapper(self.actual),
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
            renderer: self.renderer,
        }
    }

    pub fn map_owned<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(<T as ToOwned>::Owned) -> U,
    ) -> AssertThat<'t, U, M, R>
    where
        T: ToOwned,
    {
        AssertThat {
            parent: self.parent,
            actual: Actual::Owned(mapper(self.actual.borrowed().to_owned())),
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
            renderer: self.renderer,
        }
    }

    pub async fn map_async<U: 't, Fut>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Fut,
    ) -> AssertThat<'t, U, M, R>
    where
        Fut: Future<Output = U>,
    {
        AssertThat {
            parent: self.parent,
            actual: mapper(self.actual).await.into(),
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
            renderer: self.renderer,
        }
    }

    /// Derives a new assertion from this one by mapping the actual value to an owned projection.
    ///
    /// The derived assertion stays linked to its parent: failures raised on it propagate to the
    /// root assertion and are handled according to the root's mode, and its assertions count
    /// towards the parent's tracking, so the parent is not considered unused. Dropping a derived
    /// assertion is always allowed, even when failures were raised on it.
    ///
    /// The mapper receives the actual value by reference and must return an owned value. A
    /// projection borrowing from the actual value cannot be expressed with this method.
    ///
    /// Use `derive` when you need the derived `AssertThat` itself, e.g. to store it or to keep
    /// chaining on the projection. When you only want to run a group of assertions against a
    /// projection and then continue with the original subject, use the `satisfies_*` family
    /// instead. See [`AssertThat::satisfies`] for a comparison of its variants.
    pub fn derive<'u, U: 'u>(&'t self, mapper: impl FnOnce(&'t T) -> U) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        // The parent's mode must stay untouched: swapping it out would strip a derived parent's
        // own `derived` exemption and trip its drop contract when derivations are nested.
        let mut mode = self.mode.borrow().clone();
        mode.set_derived();

        AssertThat {
            parent: Some(self),
            actual: Actual::Owned(mapper(self.actual())),
            subject_name: None, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: RefCell::new(Vec::new()),
            print_location: self.print_location,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(mode),
            renderer: self.renderer.clone(),
        }
    }

    /// Like [`AssertThat::derive`], but stores a borrow of the mapped value instead of owning it.
    ///
    /// The derived assertion is over `U` itself, not `&U`, matching what `assert_that!(&value)`
    /// produces, so all assertion implementations for `U` are applicable.
    ///
    /// Intentionally private: [`AssertThat::satisfies_borrowed`] is the one public way to run
    /// assertions against a borrowed projection. `derive` cannot subsume this method because its
    /// `U` cannot borrow from the mapper's input.
    fn derive_borrowed<'u, U>(
        &'t self,
        mapper: impl FnOnce(&'t T) -> &'u U,
    ) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        // See `derive`: the parent's mode must stay untouched.
        let mut mode = self.mode.borrow().clone();
        mode.set_derived();

        AssertThat {
            parent: Some(self),
            actual: Actual::Borrowed(mapper(self.actual())),
            subject_name: None, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: RefCell::new(Vec::new()),
            print_location: self.print_location,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(mode),
            renderer: self.renderer.clone(),
        }
    }

    /// The async variant of [`AssertThat::derive`]: the mapper returns a future producing the
    /// owned projection, which is awaited before the derived assertion is created.
    pub async fn derive_async<'u, U: 'u, Fut: Future<Output = U>>(
        &'t self,
        mapper: impl FnOnce(&'t T) -> Fut,
    ) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        // See `derive`: the parent's mode must stay untouched.
        let mut mode = self.mode.borrow().clone();
        mode.set_derived();

        AssertThat {
            parent: Some(self),
            actual: Actual::Owned(mapper(self.actual()).await),
            subject_name: None, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: RefCell::new(Vec::new()),
            print_location: self.print_location,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(mode),
            renderer: self.renderer.clone(),
        }
    }

    // It would be nice to optimize this, so that:
    // - we do not need separate satisfies, satisfies_borrowed and satisfies_ref methods
    // - we use a `for<'a: 'b, 'b>` (see https://users.rust-lang.org/t/why-cant-i-use-lifetime-bounds-in-hrtbs/97277/2) bound for F and A,
    //   telling the compiler that the returned values live shorter than the input.
    // - we can replace () with some type R (return), letting the user write more succinct closures.

    /// Runs the given assertions against an owned projection of the actual value.
    ///
    /// The `satisfies_*` family runs a group of nested assertions against a projection of the
    /// actual value and then continues the current chain. Every variant derives a child
    /// assertion (see [`AssertThat::derive`]) and hands it to the `assertions` closure: failures
    /// raised inside the closure propagate to the root assertion, and the closure may freely
    /// drop the child, even after failures. The closure must return `()`, so end each assertion
    /// statement with a semicolon.
    ///
    /// The variants differ only in how the projection is obtained and typed:
    ///
    /// | Method | Mapper returns | Closure receives | Use when |
    /// |---|---|---|---|
    /// | [`satisfies`](AssertThat::satisfies) | owned `U` | `AssertThat<U>` | The projection is computed (or cloned), not borrowed from the subject. |
    /// | [`satisfies_borrowed`](AssertThat::satisfies_borrowed) | `&U` | `AssertThat<U>` | The projection borrows from the subject and `U` is sized. |
    /// | [`satisfies_ref`](AssertThat::satisfies_ref) | `&U` | `AssertThat<&U>` | `U` is unsized (`str`, `[T]`, ...). |
    ///
    /// `satisfies_borrowed` hands out a value-typed `AssertThat<U>` internally holding the
    /// borrow, exactly as `assert_that!(&value)` does, so every assertion implemented for `U` is
    /// applicable. Only fall back to `satisfies_ref` for unsized `U`, where a value-typed
    /// assertion cannot exist and the assertion traits target the reference type itself (e.g.
    /// `&str`, `&[T]`).
    ///
    /// Conceptually these variants are one method: the split exists only because Rust cannot
    /// currently express a mapper that may either return an owned value or borrow from its input
    /// (a higher-ranked trait bound limitation).
    ///
    /// # Example
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that!(("foo".to_owned(), 42))
    ///     .satisfies(|it| it.0.len(), |len| {
    ///         len.is_equal_to(3);
    ///     })
    ///     .satisfies_borrowed(|it| &it.0, |name| {
    ///         name.contains("oo");
    ///     })
    ///     .satisfies_ref(|it| it.0.as_str(), |name| {
    ///         name.starts_with("f");
    ///     });
    /// ```
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(self.derive(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        self.satisfies(mapper, assertions)
    }

    /// Runs the given assertions against a borrowed projection of the actual value.
    ///
    /// The closure receives a value-typed `AssertThat<U>` internally holding the borrow,
    /// matching what `assert_that!(&value)` produces, so every assertion implemented for `U` is
    /// applicable. Prefer this over [`AssertThat::satisfies`] whenever the projection borrows
    /// from the subject and would otherwise need cloning, and over
    /// [`AssertThat::satisfies_ref`] whenever `U` is sized.
    ///
    /// See [`AssertThat::satisfies`] for a comparison of the whole family and an example.
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies_borrowed<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(self.derive_borrowed(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_borrowed`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy_borrowed<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        self.satisfies_borrowed(mapper, assertions)
    }

    /// Runs the given assertions against a borrowed, unsized projection of the actual value.
    ///
    /// The closure receives a reference-typed `AssertThat<&U>`. This is only appropriate when
    /// `U` is unsized (`str`, `[T]`, ...): no value-typed `AssertThat<U>` can exist for such
    /// types, and the assertion traits target the reference type itself (e.g. `&str`, `&[T]`).
    /// For sized projections use [`AssertThat::satisfies_borrowed`], whose value-typed child
    /// makes all assertions implemented for `U` applicable.
    ///
    /// See [`AssertThat::satisfies`] for a comparison of the whole family and an example.
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        assertions(self.derive(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_ref`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        self.satisfies_ref(mapper, assertions)
    }

    /// Runs `assertions` against `element` on a capture-mode assertion, returning every failure
    /// raised. An empty result means that the element satisfies the assertions.
    ///
    /// Used by the collection assertions treating per-element assertions as a matching criterion.
    pub(crate) fn collect_element_failures<'e, U, A>(
        &self,
        element: &'e U,
        assertions: &A,
    ) -> Vec<String>
    where
        A: for<'a> Fn(AssertThat<'a, U, Capture, R>),
        R: Clone,
    {
        // The closure consumes the assertion handed to it, so failures are collected in a
        // capture-mode sink the closure never owns: `satisfies_borrowed` derives the handed-out
        // assertion from the sink, letting its failures propagate there.
        let mut sink = AssertThat::new_panicking(Actual::Borrowed(element))
            .with_renderer(self.renderer.clone())
            .with_location(self.print_location)
            .with_capture()
            .satisfies_borrowed(|it| it, assertions);
        sink.take_failures()
    }

    /// Gives the `actual` value contained in this assertion a descriptive name.
    /// This name will be part of panic messages when set.
    #[allow(dead_code)]
    #[must_use]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.subject_name = Some(subject_name.into());
        self
    }

    /// Control whether the location (file, line and column) is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test a panic message
    /// for exact equality and do not want to be bothered by constantly differing line and column
    /// numbers for the assert-location.
    #[allow(dead_code)]
    #[must_use]
    pub fn with_location(mut self, value: bool) -> Self {
        self.print_location = value;
        self
    }

    #[must_use]
    pub fn with_debug_format<F>(self, renderer: F) -> AssertThat<'t, T, M, CustomRenderer<F>>
    where
        F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self.with_renderer(CustomRenderer(renderer))
    }

    #[must_use]
    pub fn with_renderer<R2>(self, renderer: R2) -> AssertThat<'t, T, M, R2> {
        AssertThat {
            parent: self.parent,
            actual: self.actual,
            subject_name: self.subject_name,
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
            renderer,
        }
    }
}

/* Fluent connect */

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Filler that allows you to add an "and" inside your assertion chain.
    ///
    /// This is completely optional (noop).
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that!(42).is_greater_than(0).and().is_less_than(100);
    /// assert_that!(42).is_greater_than(0).is_less_than(100);
    /// ```
    #[inline]
    #[allow(clippy::return_self_not_must_use)]
    pub fn and(self) -> Self {
        self
    }
}

/* Mode changes */

impl<'t, T, R> AssertThat<'t, T, Panic, R> {
    /// Control whether the location is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_capture(self) -> AssertThat<'t, T, Capture, R> {
        AssertThat {
            parent: self.parent,
            actual: self.actual,
            subject_name: self.subject_name,
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: RefCell::new(Capture {
                derived: false,
                captured: false,
            }),
            renderer: self.renderer,
        }
    }
}

impl<'t, T, R> AssertThat<'t, T, Capture, R> {
    /// Switch to non-capturing mode.
    ///
    /// Panics if assertion failures were already captured!
    // TODO: Add an easy way in which users can check if assertion failures were recorded.
    //  Or that none were recorded!
    #[allow(deprecated)]
    pub fn without_capture(mut self) -> AssertThat<'t, T, Panic, R> {
        // Take out all assertion failures, marking the `Capture` as "captured".
        // Assert that no failures exist.
        use crate::assertions::core::length::LengthAssertions;
        assert_that_owned(self.take_failures())
            .with_location(self.print_location)
            .with_subject_name("Assertion failures")
            .with_detail_message(
                "You cannot unwrap the inner value if assertion failures were already recorded!",
            )
            .is_empty();

        AssertThat {
            parent: self.parent,
            actual: self.actual,
            subject_name: self.subject_name,
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: RefCell::new(Panic {
                derived: self.mode.borrow().derived,
            }),
            renderer: self.renderer,
        }
    }
}

/* Unwrapping */

impl<T, R> AssertThat<'_, T, Panic, R> {
    /// **Panics** Panics if the actual value was not owned.
    // TODO: We could relax this by having `AssertThat` be generic over the type of actual value.
    #[track_caller]
    pub fn unwrap_inner(self) -> T {
        self.actual.unwrap_owned()
    }
}

impl<T, R> AssertThat<'_, T, Capture, R> {
    /// **Panics**
    /// - If assertion errors are present.
    /// - If the actual value is not owned.
    // TODO: We could relax this by having `AssertThat` be generic over the type of actual value.
    #[track_caller]
    pub fn unwrap_inner(self) -> T {
        // Switch to panicking behaviour, asserting that no failures were recorded.
        let panicking = self.without_capture();

        panicking.actual.unwrap_owned()
    }
}

pub struct Differences {
    differences: Vec<String>,
}

impl Default for Differences {
    fn default() -> Self {
        Self::new()
    }
}

impl Differences {
    #[must_use]
    pub fn new() -> Self {
        Self {
            differences: Vec::new(),
        }
    }
}

impl Debug for Differences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.differences.iter().map(|it| details::DisplayString(it)))
            .finish()
    }
}

pub struct EqContext<'r, R = DebugRenderer> {
    differences: Differences,
    renderer: &'r R,
}

impl Default for EqContext<'static, DebugRenderer> {
    fn default() -> Self {
        Self::new()
    }
}

impl EqContext<'static, DebugRenderer> {
    #[must_use]
    pub fn new() -> Self {
        Self::with_renderer(&DebugRenderer)
    }
}

impl<'r, R> EqContext<'r, R> {
    #[must_use]
    pub fn with_renderer(renderer: &'r R) -> Self {
        Self {
            differences: Differences::default(),
            renderer,
        }
    }

    pub fn add_difference(&mut self, difference: String) {
        self.differences.differences.push(difference);
    }

    pub fn add_field_difference_without_values(&mut self, field_name: &str) {
        self.differences
            .differences
            .push(format!("\"{field_name}\": values are not equal"));
    }

    pub fn add_field_difference_rendered<A: ?Sized, E: ?Sized>(
        &mut self,
        field_name: &str,
        expected: &E,
        actual: &A,
    ) where
        R: AssertionRenderer<A> + AssertionRenderer<E>,
    {
        let expected = self.render_value(expected);
        let actual = self.render_value(actual);
        self.differences.differences.push(format!(
            "\"{field_name}\": expected {expected:#?}, but was {actual:#?}"
        ));
    }

    // The `FnOnce` closures are stored in `RefCell<Option<F>>` and consumed by the first
    // `Debug::fmt` call. The single `format!` below renders each value exactly once, so the
    // single-format invariant holds. If a future caller wraps the resulting `RenderedWith`
    // in a context that double-formats, the second call yields `fmt::Error`; switch to `Fn`
    // bounds first.
    #[doc(hidden)]
    pub fn add_field_difference_rendered_with<A: ?Sized, E: ?Sized, FA, FE>(
        &mut self,
        field_name: &str,
        expected: &E,
        actual: &A,
        render_expected: FE,
        render_actual: FA,
    ) where
        FE: FnOnce(&R, &E, &mut fmt::Formatter<'_>) -> fmt::Result,
        FA: FnOnce(&R, &A, &mut fmt::Formatter<'_>) -> fmt::Result,
    {
        struct RenderedWith<'a, T: ?Sized, R, F> {
            renderer: &'a R,
            value: &'a T,
            render: RefCell<Option<F>>,
        }

        impl<T: ?Sized, R, F> Debug for RenderedWith<'_, T, R, F>
        where
            F: FnOnce(&R, &T, &mut fmt::Formatter<'_>) -> fmt::Result,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let render = self.render.borrow_mut().take().ok_or(fmt::Error)?;
                render(self.renderer, self.value, f)
            }
        }

        let expected = RenderedWith {
            renderer: self.renderer,
            value: expected,
            render: RefCell::new(Some(render_expected)),
        };
        let actual = RenderedWith {
            renderer: self.renderer,
            value: actual,
            render: RefCell::new(Some(render_actual)),
        };
        self.differences.differences.push(format!(
            "\"{field_name}\": expected {expected:#?}, but was {actual:#?}"
        ));
    }

    pub fn add_field_difference(
        &mut self,
        field_name: &str,
        expected: impl Debug,
        actual: impl Debug,
    ) {
        self.differences.differences.push(format!(
            "\"{field_name}\": expected {expected:#?}, but was {actual:#?}"
        ));
    }

    pub fn render_value<'a, T: ?Sized>(&'a self, value: &'a T) -> Renderable<'a, T, R> {
        Renderable {
            value,
            renderer: self.renderer,
        }
    }

    pub fn render_values<'a, T>(&'a self, values: &'a [&'a T]) -> RenderableValues<'a, T, R> {
        RenderableValues {
            values,
            renderer: self.renderer,
        }
    }
}

pub trait AssertrPartialEq<Rhs: ?Sized = Self, R = DebugRenderer> {
    /// This method tests for `self` and `other` values to be equal.
    #[must_use]
    fn eq(&self, other: &Rhs, ctx: Option<&mut EqContext<'_, R>>) -> bool;

    /// This method tests for `!=`. The default implementation is almost always
    /// sufficient, and should not be overridden without very good reason.
    #[must_use]
    fn ne(&self, other: &Rhs, ctx: Option<&mut EqContext<'_, R>>) -> bool {
        !self.eq(other, ctx)
    }
}

// AssertrPartialEq must be implemented for each type already being PartialEq,
// so that we can solely rely on, and call, this ctx-enabled version in our assertions.
impl<Rhs: ?Sized, T: PartialEq<Rhs>, R> AssertrPartialEq<Rhs, R> for T {
    fn eq(&self, other: &Rhs, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
        PartialEq::eq(self, other)
    }
    fn ne(&self, other: &Rhs, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
        PartialEq::ne(self, other)
    }
}

impl<T1, T2, R> AssertrPartialEq<[T2], R> for [T1]
where
    T1: AssertrPartialEq<T2, R>,
{
    fn eq(&self, other: &[T2], mut ctx: Option<&mut EqContext<'_, R>>) -> bool {
        self.len() == other.len()
            && self.iter().enumerate().all(|(i, t1)| {
                other
                    .get(i)
                    .is_some_and(|t2| AssertrPartialEq::eq(t1, t2, ctx.as_deref_mut()))
            })
    }

    fn ne(&self, other: &[T2], ctx: Option<&mut EqContext<'_, R>>) -> bool {
        !Self::eq(self, other, ctx)
    }
}

// Note: T does not necessarily need to be `PartialEq`.
// T might itself be a type we want to compare using AssertrEq instead of PartialEq!
#[derive(Default)]
pub enum Eq<T> {
    #[default]
    Any,
    Eq(T),
}

pub fn eq<T>(v: T) -> Eq<T> {
    Eq::Eq(v)
}

#[must_use]
pub fn any<T>() -> Eq<T> {
    Eq::Any
}

impl<T: Debug> Debug for Eq<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eq::Any => f.write_str("Eq::Any"),
            Eq::Eq(v) => f.write_fmt(format_args!("Eq::Eq({v:?})")),
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use alloc::format;
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn with_capture_yields_failures_and_does_not_panic() {
        let failures = assert_that!(42)
            .with_capture()
            .with_location(false)
            .is_greater_than(100)
            .is_equal_to(1)
            .capture_failures();

        assert_that!(failures.as_slice())
            .has_length(2)
            .contains_exactly([
                formatdoc! {"
                    -------- assertr --------
                    Actual: 42

                    is not greater than

                    Expected: 100
                    -------- assertr --------
                "},
                formatdoc! {"
                    -------- assertr --------
                    Expected: 1

                      Actual: 42
                    -------- assertr --------
                "},
            ]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn dropping_a_capturing_assert_panics_when_failures_occurred_which_were_not_captured() {
        let assert = assert_that!(42)
            .with_capture()
            .with_location(false)
            .is_equal_to(43);
        assert_that_panic_by(move || drop(assert))
            .has_type::<&str>()
            .is_equal_to("You dropped an `assert_that(..)` value, on which `.with_capture()` was called, without actually capturing the assertion failures using `.capture_failures()`!");
    }

    #[test]
    fn nested_derived_assertions_in_capture_mode_keep_the_drop_contract_satisfied() {
        let root = assert_that!(1).with_capture();
        {
            let doubled = root.derive(|it| *it * 2);
            {
                let incremented = doubled.derive(|it| *it + 1);
                incremented.is_equal_to(3);
            }
        }

        let failures = root.capture_failures();
        assert_that!(failures).is_empty();
    }

    #[test]
    fn nested_derived_assertions_propagate_failures_to_the_root() {
        let root = assert_that!(1).with_capture().with_location(false);
        {
            let doubled = root.derive(|it| *it * 2);
            let incremented = doubled.derive(|it| *it + 1);
            incremented.is_equal_to(4);
        }

        let failures = root.capture_failures();
        assert_that!(failures)
            .contains_exactly_matching([|it: &String| it.contains("Expected: 4")])
            .contains_exactly_satisfying([|it: AssertThat<String, Capture>| {
                it.contains("Expected: 4");
            }]);
    }

    #[test]
    fn satisfies_borrowed_hands_out_a_value_typed_assertion_over_the_borrowed_projection() {
        assert_that!(("foo".to_owned(), 42))
            .satisfies_borrowed(
                |it| &it.0,
                |name| {
                    name.contains("oo");
                },
            )
            .satisfies_borrowed(
                |it| &it.1,
                |number| {
                    number.is_equal_to(42);
                },
            );
    }

    #[test]
    fn satisfies_borrowed_propagates_failures_to_the_root() {
        let failures = assert_that!(("foo".to_owned(), 42))
            .with_capture()
            .with_location(false)
            .satisfies_borrowed(
                |it| &it.0,
                |name| {
                    name.contains("xyz");
                },
            )
            .capture_failures();

        assert_that!(failures).contains_exactly_matching([|it: &String| it.contains("xyz")]);
    }

    #[test]
    fn dropping_a_capturing_assert_during_unwinding_preserves_the_original_panic() {
        assert_that_panic_by(|| {
            let _assert = assert_that!(42).with_capture().is_equal_to(43);
            panic!("original panic");
        })
        .has_type::<&str>()
        .is_equal_to("original panic");
    }

    #[test]
    fn without_capture_switches_to_panic_mode() {
        let assert_capturing = assert_that_owned(42)
            .with_location(false)
            .with_capture()
            // Without this assertion, we would see a panic due to zero assertions being made.
            .is_equal_to(42);

        let _assert_panicking = assert_capturing.without_capture();
    }

    #[test]
    fn without_capture_panics_when_assertion_failures_were_already_recorded() {
        let assert_capturing = assert_that_owned(42)
            .with_location(false)
            .with_capture()
            // This records a failure.
            .is_equal_to(43);

        assert_that_panic_by(move || assert_capturing.without_capture())
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Subject: Assertion failures

                Actual: alloc::vec::Vec<alloc::string::String> [
                    "-------- assertr --------\nExpected: 43\n\n  Actual: 42\n-------- assertr --------\n",
                ]

                was expected to be empty, but it is not!

                Details: [
                    You cannot unwrap the inner value if assertion failures were already recorded!,
                ]
                -------- assertr --------
            "#});
    }

    mod unwrap_inner {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn panics_on_borrowed_value_in_panic_mode() {
            let value = String::from("foo");
            let assert = assert_that(&value)
                .with_location(false)
                // Avoid "number-of-assertions not greater 0" panic.
                .is_equal_to("foo");

            assert_that_panic_by(move || assert.unwrap_inner())
                .has_type::<&str>()
                .is_equal_to(formatdoc! {r"Cannot `unwrap_owned` a borrowed value."});
        }

        #[test]
        fn panics_on_borrowed_value_in_capture_mode() {
            let value = String::from("foo");
            let assert = assert_that(&value)
                .with_location(false)
                .with_capture()
                // Avoid "number-of-assertions not greater 0" panic.
                .is_equal_to("foo");

            assert_that_panic_by(move || assert.unwrap_inner())
                .has_type::<&str>()
                .is_equal_to(formatdoc! {r"Cannot `unwrap_owned` a borrowed value."});
        }

        #[test]
        fn succeeds_on_owned_value_in_panic_mode() {
            let assert = assert_that_owned(42)
                .with_location(false)
                // Avoid "number-of-assertions not greater 0" panic.
                .is_equal_to(42);
            let actual = assert.unwrap_inner();
            assert_that!(actual).is_equal_to(42);
        }

        #[test]
        fn succeeds_on_owned_value_in_capture_mode_when_no_failures_were_recorded() {
            let assert = assert_that_owned(42)
                .with_location(false)
                .with_capture()
                .has_display_value("42");
            let actual = assert.unwrap_inner();
            assert_that!(actual).is_equal_to(42);
        }

        #[test]
        fn panics_on_owned_value_in_capture_mode_when_failures_were_recorded() {
            let assert = assert_that_owned(42)
                .with_location(false)
                .with_capture()
                // This records a failure.
                .is_equal_to(43);

            assert_that_panic_by(move || assert.unwrap_inner())
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Subject: Assertion failures

                Actual: alloc::vec::Vec<alloc::string::String> [
                    "-------- assertr --------\nExpected: 43\n\n  Actual: 42\n-------- assertr --------\n",
                ]

                was expected to be empty, but it is not!

                Details: [
                    You cannot unwrap the inner value if assertion failures were already recorded!,
                ]
                -------- assertr --------
            "#});
        }
    }

    #[cfg(feature = "fluent")]
    #[test]
    fn allows_fluent_entry_into_assertion_context() {
        42.must().be_equal_to(42);
        42.must_owned().be_equal_to(42);

        42.verify()
            .be_equal_to(42)
            .capture_failures()
            .must()
            .be_empty();
        42.verify_owned()
            .be_equal_to(42)
            .capture_failures()
            .must()
            .be_empty();

        assert_that(&42).is_equal_to(42);
        assert_that_owned(42).is_equal_to(42);

        let failures = assert_that(&42)
            .with_capture()
            .is_equal_to(42)
            .capture_failures();
        assert_that!(failures).is_empty();
        let failures = assert_that_owned(42)
            .with_capture()
            .is_equal_to(42)
            .capture_failures();
        assert_that!(failures).is_empty();

        assert_that!(&42).is_equal_to(42);
        assert_that!(42).is_equal_to(42);
    }
}
