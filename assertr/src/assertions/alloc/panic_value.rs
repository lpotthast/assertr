use crate::{AssertThat, PanicValue, actual::Actual, mode::Panic};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::{Any, type_name, type_name_of_val};
use core::fmt::Write;
use indoc::writedoc;

use super::boxed::BoxAssertions;

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
        self.map::<Box<dyn Any>>(|it| match it {
            Actual::Borrowed(b) => Actual::Borrowed(&b.0),
            Actual::Owned(o) => Actual::Owned(o.0),
        })
        .has_type::<E>()
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone,
    {
        self.track_assertion();

        let any = &self.actual().0;
        if let Some(casted) = any.downcast_ref::<E>() {
            self.derive_owned(|_actual| casted)
        } else {
            let expected_type_name = type_name::<E>();

            let is_str = any.downcast_ref::<&str>().is_some();
            let is_string = any.downcast_ref::<String>().is_some();

            let mut details = Vec::new();
            let actual_type_name = if is_str {
                Cow::Borrowed("&str")
            } else if is_string {
                Cow::Borrowed("String")
            } else {
                // Note: This call to `type_name_of_val` will just return "dyn core::any::Any"...
                details.push(String::from("The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code."));
                Cow::Borrowed(type_name_of_val(&*self.actual().0))
            };

            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Expected panic value type: {expected_type_name}

                      Actual panic value type: {actual_type_name}
                "}
            });
            unreachable!("Panic mode always panics on fail")
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

                    Expected value type: u32

                      Actual value type: String
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

                Expected panic value type: u32

                  Actual panic value type: String
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

                Expected panic value type: u32

                  Actual panic value type: &str
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

                Expected panic value type: u32

                  Actual panic value type: dyn core::any::Any

                Details:
                  - The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }
}
