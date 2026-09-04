use crate::{AssertThat, PanicValue, actual::Actual, failure::FailureKind, mode::Panic};
use alloc::boxed::Box;
use core::any::Any;

use super::boxed::{downcast, raise_type_mismatch};

/// Explains the erased type name reported for a panic payload that is neither a `&str` nor a
/// `String`.
const ERASED_TYPE_NOTE: &str = "The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.";

/// Downcasting assertions for [`PanicValue`] subjects.
///
/// These methods are available only in panic mode because a failed downcast cannot produce the
/// requested subject type.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PanicValueAssertions<'t, R = crate::DebugRenderer> {
    /// Asserts that the panic payload has type `E` and returns an assertion over that value.
    ///
    /// An owned subject produces an `AssertThat<E>` owning `E`. A borrowed subject produces an
    /// `AssertThat<E>` borrowing it.
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R>;

    /// Asserts that the panic payload has type `E` and returns an assertion over `&E`.
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone;
}

impl<'t, R> PanicValueAssertions<'t, R> for AssertThat<'t, PanicValue, Panic, R> {
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
        let boxed = self.map::<Box<dyn Any>>(|it| match it {
            Actual::Borrowed(b) => Actual::Borrowed(&b.0),
            Actual::Owned(o) => Actual::Owned(o.0),
        });
        downcast(boxed, FailureKind::Panic, ERASED_TYPE_NOTE)
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone,
    {
        self.track_assertion();

        match self.actual().0.downcast_ref::<E>() {
            Some(casted) => self.derive_owned(|_actual| casted),
            None => raise_type_mismatch::<_, _, E>(
                self,
                FailureKind::Panic,
                &*self.actual().0,
                ERASED_TYPE_NOTE,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, crate::PanicValue, Panic, NoRenderer>
                    => PanicValueAssertions<'static, NoRenderer>
            );
        }
    }

    mod has_type {
        use crate::{PanicValue, prelude::*};
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = PanicValue(Box::new(String::from("foo")));

            actual.must().have_type::<String>();
        }

        #[test]
        fn succeeds_when_type_matches() {
            let actual = PanicValue(Box::new(String::from("foo")));

            assert_that!(actual)
                .has_type::<String>()
                .is_equal_to(String::from("foo"));

            let actual = PanicValue(Box::new(String::from("foo")));

            assert_that!(actual)
                .has_type::<String>()
                .is_equal_to(String::from("foo"));
        }

        #[test]
        fn panics_when_type_does_not_match() {
            let actual = PanicValue(Box::new(String::from("foo")));

            assert_that_panic_by(|| {
                assert_that!(actual).with_location(false).has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: String

                    is not of the expected type

                    Expected: u32
                    -------- assertr --------
                "});
        }
    }

    mod has_type_ref {
        use crate::{PanicValue, prelude::*};
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = PanicValue(Box::new(String::from("foo")));

            actual.must().have_type_ref::<String>();
        }

        #[test]
        fn succeeds_when_type_matches() {
            let actual = PanicValue(Box::new(String::from("foo")));

            assert_that!(actual)
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_string() {
            let actual = PanicValue(Box::new(String::from("foo")));

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Actual: String

                is not of the expected type

                Expected: u32
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_str() {
            let actual = PanicValue(Box::new("foo"));

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Actual: &str

                is not of the expected type

                Expected: u32
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_as_any_when_not_deducible() {
            struct Foo {}
            let actual = PanicValue(Box::new(Foo {}));

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

                Details:
                  - The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }
}
