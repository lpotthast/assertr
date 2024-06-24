use std::{
    any::Any,
    cell::RefCell,
    future::Future,
    panic::{RefUnwindSafe, UnwindSafe},
};

use actual::Actual;
use details::WithDetail;
use failure::Fallible;
use mode::{Capture, Mode, Panic};
use tracking::{AssertionTracking, NumAssertions};

pub mod actual;
pub mod assertions;
pub mod condition;
pub mod details;
pub mod failure;
pub mod mode;
pub mod tracking;
pub mod util;

pub mod prelude {
    pub use crate::assert_that;
    pub use crate::assert_that_panic_by;
    pub use crate::assertions::std::array;
    pub use crate::assertions::std::array::ArrayAssertions;
    pub use crate::assertions::std::bool;
    pub use crate::assertions::std::bool::BoolAssertions;
    pub use crate::assertions::std::boxed;
    pub use crate::assertions::std::boxed::BoxAssertions;
    pub use crate::assertions::std::debug;
    pub use crate::assertions::std::display;
    pub use crate::assertions::std::hashmap;
    pub use crate::assertions::std::iter;
    pub use crate::assertions::std::iter::IntoIteratorAssertions;
    pub use crate::assertions::std::mutex;
    pub use crate::assertions::std::option;
    pub use crate::assertions::std::panic_value;
    pub use crate::assertions::std::panic_value::PanicValueAssertions;
    pub use crate::assertions::std::partial_eq;
    pub use crate::assertions::std::partial_ord;
    pub use crate::assertions::std::path;
    pub use crate::assertions::std::range;
    pub use crate::assertions::std::ref_cell;
    pub use crate::assertions::std::result;
    pub use crate::assertions::std::slice;
    pub use crate::assertions::std::str_slice;
    pub use crate::assertions::std::string;
    pub use crate::condition::Condition;
    pub use crate::condition::ConditionAssertions;
    pub use crate::mode::Mode;
    pub use crate::AssertThat;
    pub use crate::Asserting;
}

pub struct PanicValue(Box<dyn Any>);

#[track_caller]
pub fn assert_that<'t, T>(actual: impl Into<Actual<'t, T>>) -> AssertThat<'t, T, Panic> {
    AssertThat::new(actual.into())
}

#[track_caller]
pub fn assert_that_panic_by<'t, F: FnOnce() -> R + UnwindSafe, R>(
    fun: F,
) -> AssertThat<'t, PanicValue, Panic> {
    assert_that(std::panic::catch_unwind(move || {
        fun();
    }))
    .with_detail_message("Function did not panic as expected!")
    .is_err()
    .map(|it| PanicValue(it.unwrap_owned()).into())
}

pub trait Asserting {
    fn assert_that<'t, U>(self, map: impl Fn(Self) -> U) -> AssertThat<'t, U, Panic>
    where
        Self: Sized;

    fn assert_that_it<'t>(self) -> AssertThat<'t, Self, Panic>
    where
        Self: Sized;
}

impl<T> Asserting for T {
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

pub struct AssertThat<'t, T, M: Mode> {
    // Derived assertions can be created. Calling `.fail*` on them should propagate to the root assertion!
    parent: Option<&'t dyn DynAssertThat>,

    actual: Actual<'t, T>,

    subject_name: Option<String>,
    detail_messages: RefCell<Vec<String>>,
    print_location: bool,

    num_assertions: RefCell<NumAssertions>,
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
impl<'t, T, M: Mode> DynAssertThat for AssertThat<'t, T, M> {}

impl<'t, T, M: Mode> UnwindSafe for AssertThat<'t, T, M> {}
impl<'t, T, M: Mode> RefUnwindSafe for AssertThat<'t, T, M> {}

impl<'t, T> AssertThat<'t, T, Panic> {
    #[track_caller]
    pub(crate) fn new(actual: Actual<'t, T>) -> Self {
        AssertThat {
            parent: None,
            actual,
            subject_name: None,
            detail_messages: RefCell::new(Vec::new()),
            print_location: true,
            num_assertions: RefCell::new(NumAssertions::new()),
            failures: RefCell::new(Vec::new()),
            mode: RefCell::new(Panic { derived: false }),
        }
    }
}

impl<'t, T> AssertThat<'t, T, Capture> {
    #[must_use]
    pub fn capture_failures(self) -> Vec<String> {
        let mut mode = self.mode.borrow_mut();
        assert!(!mode.captured);
        mode.captured = true;
        self.failures.take()
    }
}

impl<'t, T, M: Mode> AssertThat<'t, T, M> {
    pub fn actual(&self) -> &T {
        self.actual.borrowed()
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
            num_assertions: self.num_assertions,
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
            num_assertions: self.num_assertions,
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
            num_assertions: self.num_assertions,
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
            detail_messages: RefCell::new(Vec::new()), // TODO: keep messages?
            print_location: self.print_location,
            num_assertions: RefCell::new(NumAssertions::new()),
            failures: RefCell::new(Vec::new()), // TODO: keep failures?
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
            detail_messages: RefCell::new(Vec::new()), // TODO: keep messages?
            print_location: self.print_location,
            num_assertions: RefCell::new(NumAssertions::new()),
            failures: RefCell::new(Vec::new()), // TODO: keep failures?
            mode: RefCell::new(mode),
        }
    }

    // It would be nice to optimize this, so that:
    // - we do not need satisfies and satisfies_ref
    // - we use a for<'a: 'b, 'b> (see https://users.rust-lang.org/t/why-cant-i-use-lifetime-bounds-in-hrtbs/97277/2) bound for F and A,
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

    /// Control wether the location is shown on assertion failure.
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
            num_assertions: self.num_assertions,
            failures: self.failures,
            mode: RefCell::new(Capture {
                derived: false,
                captured: false,
            }),
        }
    }

    /// Control wether the location is shown on assertion failure.
    ///
    /// It can be helpful to call `.with_location(false)` when you want to test the panic message for exact equality
    /// and do not want to be bothered by constantly differing line and column numbers fo the assert-location.
    #[allow(dead_code)]
    pub fn with_location(mut self, value: bool) -> Self {
        self.print_location = value;
        self
    }
}

#[cfg(test)]
mod tests {
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
            .has_type::<String>()
            .is_equal_to(format!("You dropped an `assert_that(..)` value, on which `.with_capture()` was called, without actually capturing the assertion failures using `.capture_failures()`!"));
    }

    #[test]
    fn asserting_that_this_allows_entering_assertion_context() {
        42.assert_that_it().is_equal_to(42);
    }
}
