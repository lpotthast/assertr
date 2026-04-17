use crate::{AssertThat, PanicValue, actual::Actual, mode::Panic, tracking::AssertionTracking};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use core::any::{Any, type_name, type_name_of_val};
use core::fmt::Write;
use indoc::writedoc;

use super::boxed::BoxAssertions;

/// Data-extracting assertions for `PanicValue`'s, the output of a panic occurred within an `assert_that_panic_by`.
/// Only available in Panic mode, as the extracted type cannot be produced when the downcast fails.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PanicValueAssertions<'t> {
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic>;

    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic>;
}

impl<'t> PanicValueAssertions<'t> for AssertThat<'t, PanicValue, Panic> {
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic> {
        self.map::<Box<dyn Any>>(|it| match it {
            Actual::Borrowed(b) => Actual::Borrowed(&b.0),
            Actual::Owned(o) => Actual::Owned(o.0),
        })
        .has_type::<E>()
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic> {
        self.track_assertion();

        let any = &self.actual().0;
        if let Some(casted) = any.downcast_ref::<E>() {
            self.derive(|_actual| casted)
        } else {
            let expected_type_name = type_name::<E>();

            let is_str = any.downcast_ref::<&str>().is_some();
            let is_string = any.downcast_ref::<String>().is_some();

            let actual_type_name = if is_str {
                Cow::Borrowed("&str")
            } else if is_string {
                Cow::Borrowed("String")
            } else {
                // Note: This call to `type_name_of_val` will just return "dyn core::any::Any"...
                self.add_detail_message("The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.");
                Cow::Borrowed(type_name_of_val(&*self.actual().0))
            };

            self.fail(|w: &mut String| {
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
    mod has_type {
        use crate::{PanicValue, prelude::*};
        use indoc::formatdoc;

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
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value type: u32

                      Actual value type: String
                    -------- assertr --------
                "#});
        }
    }

    mod has_type_ref {
        use crate::{PanicValue, prelude::*};
        use indoc::formatdoc;

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
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected panic value type: u32

                  Actual panic value type: String
                -------- assertr --------
            "#});
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
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected panic value type: u32

                  Actual panic value type: &str
                -------- assertr --------
            "#});
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
                .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected panic value type: u32

                  Actual panic value type: dyn core::any::Any

                Details: [
                    The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.,
                ]
                -------- assertr --------
            "#});
        }
    }
}
