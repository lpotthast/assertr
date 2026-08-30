use alloc::vec::Vec;

use crate::{
    AssertThat, AssertionFailure,
    actual::Actual,
    mode::{Capture, Panic},
};

/// Fluent entry points into an assertion context, available on every value with the `fluent`
/// feature.
///
/// `must()` and `verify()` borrow the value, `must_owned()` and `verify_owned()` take ownership.
/// The borrowing and ownership-taking methods live on separate traits so reference receivers have
/// one unambiguous meaning. The shorter names borrow and keep the value usable. Ownership is
/// required only by consuming assertions such as `panics()`.
///
/// `IntoAssertContext` is implemented for `&T` and `&mut T`. Method-call autoref makes it available
/// as `value.must()` for an owned binding, while a receiver that is already a shared or mutable
/// reference borrows its pointee directly. All three forms therefore produce `AssertThat<T>` for a
/// sized `T`. In particular, `reference.must()` works for `reference: &mut T` without writing
/// `(*reference).must()`, exposes every assertion implemented for `T`, and reborrows rather than
/// consumes the mutable reference.
///
/// Unsized slices and strings cannot be the `T` in `AssertThat<T>`. Mutable references to them are
/// reborrowed as shared references instead: `&mut [T]` produces `AssertThat<&[T]>`, and `&mut str`
/// produces `AssertThat<&str>`. Their slice, collection, string, and length assertions remain
/// available.
///
/// Ownership-taking calls use the separate [`IntoOwnedAssertContext`] trait and preserve the exact
/// receiver expression. `(&value).must_owned()` owns the `&T` and produces `AssertThat<&T>`. It
/// cannot take ownership of `value`.
///
/// ```
/// use assertr::prelude::*;
///
/// fn accepts_string(_: AssertThat<'_, String, Panic>) {}
///
/// let mut value = String::from("value");
/// accepts_string(assert_that!(&mut value));
/// let reference = &mut value;
/// accepts_string(reference.must());
/// reference.must().start_with("val");
/// reference.push('!');
///
/// let owned_reference: &mut String = reference.must_owned().unwrap_inner();
/// owned_reference.push('!');
/// ```
///
/// # Alias names
///
/// The fluent names are derived mechanically from the assertion names. `is_x` becomes `be_x`,
/// `has_x` becomes `have_x`, and other verbs turn imperative. For example, `contains` -> `contain`,
/// `starts_with` -> `start_with`, `exists` -> `exist`, `panics` -> `panic`,
/// `needs_drop` -> `need_drop`. Negations put `not` first, as in "must not be equal to".
/// `is_not_x` -> `not_be_x`, `has_not_x` -> `not_have_x`, `does_not_x` -> `not_x`. The possessive
/// `has_no_x` keeps its order as `have_no_x`. Namespace prefixes stay in front of the alias.
/// `into_iter_contains` -> `into_iter_contain`. Explicit aliases cover names outside these rules,
/// such as `is(condition)` -> `be(condition)`.
///
/// This trait is re-exported by [`crate::prelude`]. Import the prelude and use method syntax rather than
/// implementing this trait downstream. See the fluent entry-point guide above for reference
/// normalization, ownership, and alias naming.
#[cfg(feature = "fluent")]
pub trait IntoAssertContext<'t> {
    /// The subject type assertion methods are resolved against.
    type Subject: 't;

    /// Borrows the pointee and starts a panic-mode assertion: failures panic immediately.
    #[must_use]
    fn must(self) -> AssertThat<'t, Self::Subject, Panic>;

    /// Borrows the pointee and runs the given assertions in capture mode: failures do not panic
    /// but are collected and returned as structured [`AssertionFailure`] values.
    ///
    /// See [`AssertThat::capture`] for the capture-closure contract.
    #[must_use = "The captured failures must be inspected. Use `must()` to panic on failure instead."]
    fn verify<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, Self::Subject, Capture>) -> AssertThat<'t, U, Capture, R2>;
}

#[cfg(feature = "fluent")]
impl<'t, T: 't> IntoAssertContext<'t> for &'t T {
    type Subject = T;

    fn must(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }

    #[track_caller]
    fn verify<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, T, Capture>) -> AssertThat<'t, U, Capture, R2>,
    {
        AssertThat::new_capturing(Actual::Borrowed(self)).run_and_collect(assertions)
    }
}

#[cfg(feature = "fluent")]
impl<'t, T: 't> IntoAssertContext<'t> for &'t mut T {
    type Subject = T;

    fn must(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Borrowed(self))
    }

    #[track_caller]
    fn verify<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, T, Capture>) -> AssertThat<'t, U, Capture, R2>,
    {
        AssertThat::new_capturing(Actual::Borrowed(self)).run_and_collect(assertions)
    }
}

#[cfg(feature = "fluent")]
impl<'t, T: 't> IntoAssertContext<'t> for &'t mut [T] {
    type Subject = &'t [T];

    fn must(self) -> AssertThat<'t, &'t [T], Panic> {
        let shared: &'t [T] = self;
        AssertThat::new_panicking(Actual::Owned(shared))
    }

    #[track_caller]
    fn verify<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, &'t [T], Capture>) -> AssertThat<'t, U, Capture, R2>,
    {
        let shared: &'t [T] = self;
        AssertThat::new_capturing(Actual::Owned(shared)).run_and_collect(assertions)
    }
}

#[cfg(feature = "fluent")]
impl<'t> IntoAssertContext<'t> for &'t mut str {
    type Subject = &'t str;

    fn must(self) -> AssertThat<'t, &'t str, Panic> {
        let shared: &'t str = self;
        AssertThat::new_panicking(Actual::Owned(shared))
    }

    #[track_caller]
    fn verify<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, &'t str, Capture>) -> AssertThat<'t, U, Capture, R2>,
    {
        let shared: &'t str = self;
        AssertThat::new_capturing(Actual::Owned(shared)).run_and_collect(assertions)
    }
}

/// Fluent entry points that preserve ownership of their receiver.
///
/// This trait is re-exported by [`crate::prelude`]. It is separate from [`IntoAssertContext`] so `must()`
/// and `verify()` can consistently borrow reference pointees, while `must_owned()` and
/// `verify_owned()` keep the exact type passed by the caller. Import the prelude and use method
/// syntax rather than implementing this trait downstream.
#[cfg(feature = "fluent")]
pub trait IntoOwnedAssertContext<'t>: Sized {
    /// Takes ownership of the value and starts a panic-mode assertion.
    ///
    /// Use this when an assertion consumes its subject. Prefer [`IntoAssertContext::must`] when
    /// ownership is not required.
    #[must_use]
    fn must_owned(self) -> AssertThat<'t, Self, Panic>;

    /// Takes ownership of the value and runs the given assertions in capture mode.
    ///
    /// Use this when an assertion consumes its subject. Prefer [`IntoAssertContext::verify`] when
    /// ownership is not required.
    #[must_use = "The captured failures must be inspected. Use `must_owned()` to panic on failure instead."]
    fn verify_owned<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        Self: 't,
        F: FnOnce(AssertThat<'t, Self, Capture>) -> AssertThat<'t, U, Capture, R2>;
}

#[cfg(feature = "fluent")]
impl<'t, T: 't> IntoOwnedAssertContext<'t> for T {
    fn must_owned(self) -> AssertThat<'t, T, Panic> {
        AssertThat::new_panicking(Actual::Owned(self))
    }

    #[track_caller]
    fn verify_owned<F, U: 't, R2>(self, assertions: F) -> Vec<AssertionFailure>
    where
        F: FnOnce(AssertThat<'t, T, Capture>) -> AssertThat<'t, U, Capture, R2>,
    {
        AssertThat::new_capturing(Actual::Owned(self)).run_and_collect(assertions)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[cfg(feature = "fluent")]
    mod fluent_entry_points {
        use super::*;

        fn accepts_string_subject(_: AssertThat<'_, String, Panic>) {}

        fn accepts_subject<T>(_: AssertThat<'_, T, Panic>) {}

        fn accepts_shared_slice_subject<T>(_: AssertThat<'_, &[T], Panic>) {}

        fn accepts_shared_str_subject(_: AssertThat<'_, &str, Panic>) {}

        fn assert_mutable_reference_borrows_pointee<T>(value: &mut T) {
            accepts_subject(value.must());
        }

        #[test]
        fn work_for_values() {
            42.must().be_equal_to(42);
            42.must_owned().be_equal_to(42);

            42.verify(|it| it.be_equal_to(42)).must().be_empty();
            42.verify_owned(|it| it.be_equal_to(42)).must().be_empty();
        }

        #[test]
        fn are_unambiguous_for_shared_references() {
            let value = String::from("foo");
            let reference = &value;

            accepts_string_subject(reference.must());
            reference.must().have_debug_value("foo");
            let owned_reference: &String = reference.must_owned().unwrap_inner();
            assert_that!(owned_reference).is_equal_to("foo");

            let failures = reference.verify(|it| it.have_debug_value("foo"));
            assert_that!(failures).is_empty();
            let failures = reference.verify_owned(|it| it.have_debug_value("foo"));
            assert_that!(failures).is_empty();
        }

        #[test]
        fn are_unambiguous_for_mutable_references() {
            let mut value = String::from("foo");

            {
                let reference = &mut value;
                accepts_string_subject(reference.must());
                assert_mutable_reference_borrows_pointee(reference);
                reference.must().have_debug_value("foo");
                let failures = reference.verify(|it| it.have_debug_value("foo"));
                assert_that!(failures).is_empty();
                reference.push('!');
            }

            {
                let reference = &mut value;
                let owned_reference: &mut String = reference.must_owned().unwrap_inner();
                owned_reference.push('!');
            }
            let reference = &mut value;
            let failures = reference.verify_owned(|it| it.have_debug_value("foo!!"));
            assert_that!(failures).is_empty();
            assert_that!(value).is_equal_to("foo!!");
        }

        #[test]
        fn borrowed_entry_points_normalize_reference_expressions() {
            let shared_value = String::from("foo");
            accepts_string_subject(assert_that!(&shared_value));
            accepts_string_subject((&shared_value).must());

            let mut value = String::from("foo");

            accepts_string_subject(assert_that!(&mut value));
            accepts_string_subject((&mut value).must());

            let failures = (&mut value)
                .verify(|it: AssertThat<'_, String, Capture>| it.has_debug_value("foo"));
            assert_that!(failures).is_empty();
        }

        #[test]
        fn mutable_references_gain_the_pointee_assertions() {
            let mut truth = true;
            (&mut truth).must().be_true();

            let mut option = Some(42);
            (&mut option).must().be_some();

            #[cfg(feature = "num")]
            {
                let mut number = 42;
                (&mut number).must().be_positive();
            }

            let mut values = vec![1, 2, 3];
            (&mut values).must().contain(2).have_length(3);
        }

        #[test]
        fn mutable_unsized_references_use_shared_reference_subjects() {
            let mut values = [1, 2, 3];
            let slice: &mut [i32] = &mut values;
            accepts_shared_slice_subject(slice.must());
            slice.must().contain(2).have_length(3);
            let failures =
                slice.verify(|it: AssertThat<'_, &[i32], Capture>| it.contain(2).have_length(3));
            assert_that!(failures).is_empty();

            let mut value = String::from("value");
            let string: &mut str = value.as_mut_str();
            accepts_shared_str_subject(string.must());
            string.must().start_with("val").have_length(5);
            let failures = string
                .verify(|it: AssertThat<'_, &str, Capture>| it.start_with("val").have_length(5));
            assert_that!(failures).is_empty();
        }
    }
}
