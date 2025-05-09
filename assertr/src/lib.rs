#![cfg_attr(not(feature = "std"), no_std)]
// Allow functions named `is_*`, taking self by value instead of taking self by mutable reference or reference.
#![allow(clippy::wrong_self_convention)]

extern crate alloc;
extern crate core;

use actual::Actual;
use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec::Vec};
use core::{
    any::Any,
    cell::RefCell,
    fmt::{Debug, Formatter},
    future::Future,
    panic::{RefUnwindSafe, UnwindSafe},
};
use details::WithDetail;
use failure::Fallible;
use mode::{Capture, Mode, Panic};
use std::marker::PhantomData;
use tracking::{AssertionTracking, NumberOfAssertions};

pub mod actual;
pub mod assertions;
pub mod cmp;
pub mod condition;
mod conversion;
pub mod details;
pub mod failure;
pub mod mode;
pub mod tracking;
pub mod util;

pub mod prelude {
    #[cfg(feature = "derive")]
    pub use assertr_derive::AssertrEq;

    pub use crate::AssertThat;
    pub use crate::AssertingThat;
    pub use crate::AssertingThatRef;
    pub use crate::any;
    pub use crate::assert_that;
    #[cfg(feature = "std")]
    pub use crate::assert_that_panic_by;
    pub use crate::assert_that_ref;
    pub use crate::assert_that_type;
    pub use crate::assertions::HasLength;
    pub use crate::assertions::alloc::prelude::*;
    pub use crate::assertions::condition::ConditionAssertions;
    pub use crate::assertions::condition::IterableConditionAssertions;
    pub use crate::assertions::core::prelude::*;
    #[cfg(feature = "jiff")]
    pub use crate::assertions::jiff::prelude::*;
    #[cfg(feature = "num")]
    pub use crate::assertions::num::NumAssertions;
    #[cfg(feature = "reqwest")]
    pub use crate::assertions::reqwest::prelude::*;
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
    pub use crate::mode::Mode;
}

pub struct PanicValue(Box<dyn Any>);

/// The main entrypoint in an assertion context.
///
/// #### Example Usage
/// ```rust,no_run
/// use assertr::prelude::*;
///
/// // This will panic with a descriptive message and a pointer to the actual line of the assertion.
/// assert_that(3).is_equal_to(4);
///
/// // This instead captures the assertion failure for later inspection.
/// let failures = assert_that(3)
///     .with_capture()
///     .is_equal_to(4) // This will collect a failure instead of panicking.
///     .capture_failures();
///
/// assert_that(failures)
///     .has_length(1)
///     .contains("");
/// ```
#[track_caller]
#[must_use]
pub fn assert_that<'t, T>(actual: T) -> AssertThat<'t, T, Panic> {
    AssertThat::new(Actual::Owned(actual))
}

#[track_caller]
#[must_use]
pub fn assert_that_ref<T>(actual: &T) -> AssertThat<T, Panic> {
    AssertThat::new(Actual::Borrowed(actual))
}

#[track_caller]
#[must_use]
#[cfg(feature = "std")]
pub fn assert_that_panic_by<'t, R>(
    fun: impl FnOnce() -> R + 't,
) -> AssertThat<'t, PanicValue, Panic> {
    use crate::prelude::FnOnceAssertions;

    assert_that(fun).panics()
}

pub struct Type<T> {
    phantom: PhantomData<T>,
}

impl<T> Type<T> {
    pub fn get_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    pub fn needs_drop(&self) -> bool {
        std::mem::needs_drop::<T>()
    }

    pub fn size(&self) -> usize {
        size_of::<T>()
    }
}

pub fn assert_that_type<T>() -> AssertThat<'static, Type<T>, Panic> {
    AssertThat::new(Actual::Owned(Type {
        phantom: Default::default(),
    }))
}

pub trait AssertingThat {
    fn assert_that<'t, U>(self, map: impl Fn(Self) -> U) -> AssertThat<'t, U, Panic>
    where
        Self: Sized;

    fn assert_that_it<'t>(self) -> AssertThat<'t, Self, Panic>
    where
        Self: Sized;
}

impl<T> AssertingThat for T {
    fn assert_that<'t, U>(self, map: impl Fn(T) -> U) -> AssertThat<'t, U, Panic>
    where
        Self: Sized,
    {
        assert_that(map(self))
    }

    fn assert_that_it<'t>(self) -> AssertThat<'t, Self, Panic> {
        assert_that(self)
    }
}

pub trait AssertingThatRef {
    type Owned;

    fn assert_that<U>(&self, map: impl Fn(&Self) -> &U) -> AssertThat<U, Panic>
    where
        Self: Sized;

    fn assert_that_it(&self) -> AssertThat<Self::Owned, Panic>
    where
        Self: Sized;
}

impl<T> AssertingThatRef for &T {
    type Owned = T;

    fn assert_that<U>(&self, map: impl Fn(&Self) -> &U) -> AssertThat<U, Panic>
    where
        Self: Sized,
    {
        assert_that_ref(map(self))
    }

    fn assert_that_it(&self) -> AssertThat<Self::Owned, Panic>
    where
        Self: Sized,
    {
        assert_that_ref(self)
    }
}

/// `AssertThat` is the core structure used for assertions. It allows developers to perform
/// assertions on actual values in a fluent and expressive manner, supporting detailed messages
/// as well as different modes of operation, such as panic or capture modes.
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
/// - This struct is designed to handle both simple and complex assertion chaining scenarios.
pub struct AssertThat<'t, T, M: Mode> {
    // Derived assertions can be created. Calling `.fail*` on them should propagate to the root assertion!
    parent: Option<&'t dyn DynAssertThat>,

    actual: Actual<'t, T>,

    subject_name: Option<String>,
    detail_messages: RefCell<Vec<String>>,
    print_location: bool,

    number_of_assertions: RefCell<NumberOfAssertions>,
    failures: RefCell<Vec<String>>,

    mode: RefCell<M>,
}

/*
// TODO: Consider this
pub struct DerivedAssertThat<'t, T> {
    // Derived assertions can be created. Calling `.fail*` on them should propagate to the root assertion!
    parent: Option<&'t dyn DynAssertThat>,

    actual: Actual<'t, T>,

    subject_name: Option<String>,
    detail_messages: RefCell<Vec<String>>,

    num_assertions: RefCell<NumAssertions>,
}
*/

pub(crate) trait DynAssertThat: Fallible + WithDetail + AssertionTracking {}
impl<T, M: Mode> DynAssertThat for AssertThat<'_, T, M> {}

impl<T, M: Mode> UnwindSafe for AssertThat<'_, T, M> {}
impl<T, M: Mode> RefUnwindSafe for AssertThat<'_, T, M> {}

impl<'t, T> AssertThat<'t, T, Panic> {
    #[track_caller]
    pub(crate) const fn new(actual: Actual<'t, T>) -> Self {
        AssertThat {
            parent: None,
            actual,
            subject_name: None,
            detail_messages: RefCell::new(Vec::new()),
            print_location: true,
            number_of_assertions: RefCell::new(NumberOfAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(Panic { derived: false }),
        }
    }
}

impl<T> AssertThat<'_, T, Capture> {
    #[must_use]
    pub fn capture_failures(self) -> Vec<String> {
        let mut mode = self.mode.borrow_mut();
        assert!(
            !mode.captured,
            "You can only capture the assertion failures once!"
        );
        mode.captured = true;
        self.failures.take()
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    pub fn actual(&self) -> &T {
        self.actual.borrowed()
    }

    pub(crate) fn replace_actual_with<'u, U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        new_actual: Actual<'u, U>,
    ) -> (Actual<'t, T>, AssertThat<'u, U, M>)
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
        };
        (previous_actual, mapped)
    }

    pub fn map<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Actual<U>,
    ) -> AssertThat<'t, U, M> {
        AssertThat {
            parent: self.parent,
            actual: mapper(self.actual),
            subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
            detail_messages: self.detail_messages,
            print_location: self.print_location,
            number_of_assertions: self.number_of_assertions,
            failures: self.failures,
            mode: self.mode,
        }
    }

    pub fn map_owned<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(<T as ToOwned>::Owned) -> U,
    ) -> AssertThat<'t, U, M>
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
        }
    }

    pub async fn map_async<U: 't, Fut>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Fut,
    ) -> AssertThat<'t, U, M>
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
        }
    }

    pub fn derive<'u, U: 'u>(&'t self, mapper: impl FnOnce(&'t T) -> U) -> AssertThat<'u, U, M>
    where
        't: 'u,
    {
        let mut mode = self.mode.replace(M::default());
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
        }
    }

    pub async fn derive_async<'u, U: 'u, Fut: Future<Output = U>>(
        &'t self,
        mapper: impl FnOnce(&'t T) -> Fut,
    ) -> AssertThat<'u, U, M>
    where
        't: 'u,
    {
        let mut mode = self.mode.replace(M::default());
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
        }
    }

    // It would be nice to optimize this, so that:
    // - we do not need satisfies and satisfies_ref
    // - we use a `for<'a: 'b, 'b>` (see https://users.rust-lang.org/t/why-cant-i-use-lifetime-bounds-in-hrtbs/97277/2) bound for F and A,
    //   telling the compiler that the returned values live shorter than the input.
    // - we can replace () with some type R (return), letting the user write more succinct closures.

    pub fn satisfies<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M>),
    {
        assertions(self.derive(mapper));
        self
    }

    pub fn satisfies_ref<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M>),
    {
        assertions(self.derive(mapper));
        self
    }

    /// Gives the `actual` value contain in this assertion a descriptive name.
    /// This name will be part of panic messages when set.
    #[allow(dead_code)]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.subject_name = Some(subject_name.into());
        self
    }

    /// Control whether the location is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_capture(self) -> AssertThat<'t, T, Capture> {
        *self.mode.borrow_mut() = M::default();

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
        }
    }

    /// Control whether the location (file, line and column) is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test a panic message
    /// for exact equality and do not want to be bothered by constantly differing line and column
    /// numbers for the assert-location.
    #[allow(dead_code)]
    pub fn with_location(mut self, value: bool) -> Self {
        self.print_location = value;
        self
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
    pub fn new() -> Self {
        Self {
            differences: Vec::new(),
        }
    }
}

impl Debug for Differences {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.differences.iter().map(|it| details::DisplayString(it)))
            .finish()
    }
}

pub struct EqContext {
    differences: Differences,
}

impl Default for EqContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EqContext {
    pub fn new() -> Self {
        Self {
            differences: Differences::default(),
        }
    }

    pub fn add_difference(&mut self, difference: String) {
        self.differences.differences.push(difference);
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
}

pub trait AssertrPartialEq<Rhs: ?Sized = Self> {
    /// This method tests for `self` and `other` values to be equal.
    #[must_use]
    fn eq(&self, other: &Rhs, ctx: Option<&mut EqContext>) -> bool;

    /// This method tests for `!=`. The default implementation is almost always
    /// sufficient, and should not be overridden without very good reason.
    #[must_use]
    fn ne(&self, other: &Rhs, ctx: Option<&mut EqContext>) -> bool {
        !self.eq(other, ctx)
    }
}

// AssertrPartialEq must be implemented for each type already being PartialEq,
// so that we can solely rely on, and call, this ctx-enabled version in our assertions.
impl<Rhs: ?Sized, T: PartialEq<Rhs>> AssertrPartialEq<Rhs> for T {
    fn eq(&self, other: &Rhs, _ctx: Option<&mut EqContext>) -> bool {
        PartialEq::eq(self, other)
    }
    fn ne(&self, other: &Rhs, _ctx: Option<&mut EqContext>) -> bool {
        PartialEq::ne(self, other)
    }
}

impl<T1: AssertrPartialEq<T2>, T2> AssertrPartialEq<[T2]> for [T1] {
    fn eq(&self, other: &[T2], mut ctx: Option<&mut EqContext>) -> bool {
        self.len() == other.len()
            && self.iter().enumerate().all(|(i, t1)| {
                other
                    .get(i)
                    .is_some_and(|t2| AssertrPartialEq::eq(t1, t2, ctx.as_deref_mut()))
            })
    }

    fn ne(&self, other: &[T2], ctx: Option<&mut EqContext>) -> bool {
        !Self::eq(self, other, ctx)
    }
}

// Note: T does not necessarily need to be `PartialEq`.
// T might itself be a type we want to compare using AssertrEq instead of PartialEq!
pub enum Eq<T> {
    Any,
    Eq(T),
}

pub fn eq<T>(v: T) -> Eq<T> {
    Eq::Eq(v)
}

pub fn any<T>() -> Eq<T> {
    Eq::Any
}

impl<T: Debug> Debug for Eq<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Eq::Any => f.write_str("Eq::Any"),
            Eq::Eq(v) => f.write_fmt(format_args!("Eq::Eq({v:?})")),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn with_capture_yields_failures_and_does_not_panic() {
        let failures = assert_that(42)
            .with_location(false)
            .with_capture()
            .is_greater_than(100)
            .is_equal_to(1)
            .capture_failures();

        assert_that(failures.as_slice())
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
    fn dropping_a_capturing_assert_panics_when_failures_occurred_which_were_not_captured() {
        let assert = assert_that(42)
            .with_location(false)
            .with_capture()
            .is_equal_to(43);
        assert_that_panic_by(move || drop(assert))
            .has_type::<&str>()
            .is_equal_to("You dropped an `assert_that(..)` value, on which `.with_capture()` was called, without actually capturing the assertion failures using `.capture_failures()`!");
    }

    #[test]
    fn asserting_that_this_allows_entering_assertion_context() {
        42.assert_that_it().is_equal_to(42);
    }
}
