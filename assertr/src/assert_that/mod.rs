//! Core construction and basic operations for assertion chains.

mod capture;
mod projection;
mod rendering;

use crate::{
    AssertThat, ChainState, DynAssertThat,
    actual::Actual,
    mode::{Capture, Mode, Panic},
    renderer::DebugRenderer,
};

impl<'t, M: Mode, R> ChainState<'t, M, R> {
    const fn root(renderer: R) -> Self {
        Self {
            parent: None,
            subject_name: None,
            detail_messages: core::cell::RefCell::new(alloc::vec::Vec::new()),
            print_location: true,
            number_of_assertions: core::cell::RefCell::new(
                crate::tracking::NumberOfAssertions::new(),
            ),
            failures: core::cell::RefCell::new(alloc::vec::Vec::new()),
            mode: core::marker::PhantomData,
            renderer,
        }
    }

    fn child<'u, R2>(&self, parent: &'u dyn DynAssertThat, renderer: R2) -> ChainState<'u, M, R2> {
        ChainState {
            parent: Some(parent),
            subject_name: None,
            detail_messages: core::cell::RefCell::new(alloc::vec::Vec::new()),
            print_location: self.print_location,
            number_of_assertions: core::cell::RefCell::new(
                crate::tracking::NumberOfAssertions::new(),
            ),
            failures: core::cell::RefCell::new(alloc::vec::Vec::new()),
            mode: core::marker::PhantomData,
            renderer,
        }
    }

    fn with_renderer<R2>(self, renderer: R2) -> ChainState<'t, M, R2> {
        ChainState {
            parent: self.parent,
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

impl<'t, T> AssertThat<'t, T, Panic> {
    #[track_caller]
    pub(crate) const fn new_panicking(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            state: ChainState::root(DebugRenderer),
        }
    }
}

impl<'t, T> AssertThat<'t, T, Capture> {
    #[track_caller]
    pub(crate) const fn new_capturing(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            state: ChainState::root(DebugRenderer),
        }
    }
}

/* Fluent connect */

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Returns the chain unchanged, allowing an optional `and()` between assertions.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that!(42).is_greater_than(0).and().is_less_than(100);
    /// assert_that!(42).is_greater_than(0).is_less_than(100);
    /// ```
    ///
    #[inline]
    #[must_use]
    pub fn and(self) -> Self {
        self
    }
}

/* Unwrapping */

impl<T, R> AssertThat<'_, T, Panic, R> {
    /// Unwraps the owned subject from this assertion.
    ///
    /// # Panics
    ///
    /// Panics if the subject is borrowed. Use `assert_that_owned!(...)` or `.must_owned()` to
    /// create an owned assertion.
    #[track_caller]
    #[must_use]
    pub fn unwrap_inner(self) -> T {
        self.actual.unwrap_owned()
    }
}

#[cfg(test)]
mod tests {
    mod unwrap_inner {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn panics_on_borrowed_value_in_panic_mode() {
            let value = String::from("foo");
            let assert = assert_that!(&value).with_location(false).is_equal_to("foo");

            assert_that_panic_by(move || assert.unwrap_inner())
                .has_type::<&str>()
                .is_equal_to(formatdoc! {r"Cannot unwrap a borrowed value. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."});
        }

        #[test]
        fn succeeds_on_owned_value_in_panic_mode() {
            let assert = assert_that_owned!(42).with_location(false).is_equal_to(42);
            let actual = assert.unwrap_inner();
            assert_that!(actual).is_equal_to(42);
        }
    }
}
