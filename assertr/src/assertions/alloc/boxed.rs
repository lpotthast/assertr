use crate::{AssertThat, actual::Actual, mode::Panic};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::{Any, type_name, type_name_of_val};
use core::fmt::Write;
use indoc::writedoc;

/// Downcasting assertions for `Box<dyn Any>` subjects.
///
/// These methods are available only in panic mode because a failed downcast cannot produce the
/// requested subject type.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait BoxAssertions<'t, R> {
    /// Asserts that the boxed value has type `E` and returns an assertion over that value.
    ///
    /// An owned box produces an `AssertThat<E>` owning `E`. A borrowed box produces an
    /// `AssertThat<E>` borrowing it.
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R>;

    /// Asserts that the boxed value has type `E` and returns an assertion over `&E`.
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone;
}

impl<'t, R> BoxAssertions<'t, R> for AssertThat<'t, Box<dyn Any>, Panic, R> {
    #[track_caller]
    #[allow(clippy::too_many_lines)]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
        enum CastResult<'c, C> {
            Owned(Box<C>),
            Ref(&'c C),
            Err {
                actual: Actual<'c, Box<dyn Any>>,
                actual_type_name: Cow<'static, str>,
                actual_type_name_will_be_any: bool,
            },
        }

        self.track_assertion();
        let AssertThat { actual, state } = self;

        let cast = match actual {
            crate::actual::Actual::Borrowed(borrowed_boxed_any) => {
                let is_str = borrowed_boxed_any.downcast_ref::<&str>().is_some();
                let is_string = borrowed_boxed_any.downcast_ref::<String>().is_some();

                let mut actual_type_name_will_be_any = false;
                let actual_type_name = if is_str {
                    Cow::Borrowed("&str")
                } else if is_string {
                    Cow::Borrowed("String")
                } else {
                    // Note: This call to `type_name_of_val` will just return "dyn core::any::Any"...
                    actual_type_name_will_be_any = true;
                    Cow::Borrowed(type_name_of_val(&**borrowed_boxed_any))
                };

                borrowed_boxed_any.downcast_ref::<E>().map_or_else(
                    || CastResult::Err {
                        actual: Actual::Borrowed(borrowed_boxed_any),
                        actual_type_name,
                        actual_type_name_will_be_any,
                    },
                    |it| CastResult::Ref(it),
                )
            }
            crate::actual::Actual::Owned(owned_box_any) => {
                let is_str = owned_box_any.downcast_ref::<&str>().is_some();
                let is_string = owned_box_any.downcast_ref::<String>().is_some();

                let mut actual_type_name_will_be_any = false;
                let actual_type_name = if is_str {
                    Cow::Borrowed("&str")
                } else if is_string {
                    Cow::Borrowed("String")
                } else {
                    // Note: This call to `type_name_of_val` will just return "dyn core::any::Any"...
                    actual_type_name_will_be_any = true;
                    Cow::Borrowed(type_name_of_val(&*owned_box_any))
                };

                owned_box_any.downcast::<E>().map_or_else(
                    |actual| CastResult::Err {
                        actual: Actual::Owned(actual),
                        actual_type_name,
                        actual_type_name_will_be_any,
                    },
                    |it| CastResult::Owned(it),
                )
            }
        };

        match cast {
            CastResult::Owned(casted) => AssertThat {
                actual: (*casted).into(),
                state,
            },
            CastResult::Ref(casted) => AssertThat {
                actual: casted.into(),
                state,
            },
            CastResult::Err {
                actual,
                actual_type_name,
                actual_type_name_will_be_any,
            } => {
                let assertion = AssertThat { actual, state };

                let expected_type_name = type_name::<E>();

                let mut details = Vec::new();
                if actual_type_name_will_be_any {
                    details.push(String::from("A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code."));
                }

                assertion.fail_with_details(details, |w: &mut String| {
                    writedoc! {w, r"
                        Expected value type: {expected_type_name}

                          Actual value type: {actual_type_name}
                    "}
                });
                unreachable!("Panic mode always panics on fail")
            }
        }
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone,
    {
        self.track_assertion();

        let any = &self.actual();
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
                details.push(String::from("A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code."));
                Cow::Borrowed(type_name_of_val(&**self.actual()))
            };

            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Expected value type: {expected_type_name}

                      Actual value type: {actual_type_name}
                "}
            });
            unreachable!("Panic mode always panics on fail")
        }
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use alloc::boxed::Box;

        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Box<dyn core::any::Any>, Panic, NoRenderer>
                    => BoxAssertions<'static, NoRenderer>
            );
        }
    }

    mod has_type {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::any::Any;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            boxed_any.must().have_type::<&str>();
        }

        #[test]
        fn succeeds_when_type_of_contained_value_matches_expected_type() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            assert_that!(boxed_any)
                .has_type::<&str>()
                .is_equal_to("foo");
        }

        #[test]
        fn accepts_an_explicit_reference_to_the_box() {
            let boxed_any: Box<dyn Any> = Box::new("foo");
            let boxed_any = &boxed_any;

            assert_that!(boxed_any)
                .has_type::<&str>()
                .is_equal_to("foo");
        }

        #[test]
        fn panics_when_type_of_contained_value_does_not_match_expected_type() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            assert_that_panic_by(|| {
                assert_that!(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `boxed_any`

                    Expected value type: u32

                      Actual value type: &str
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_with_the_erased_type_name_for_a_borrowed_box() {
            struct Foo;
            let boxed_any: Box<dyn Any> = Box::new(Foo);

            assert_that_panic_by(|| {
                assert_that!(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `boxed_any`

                Expected value type: u32

                  Actual value type: dyn core::any::Any

                Details:
                  - A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_with_the_erased_type_name_for_an_owned_box() {
            struct Foo;
            let boxed_any: Box<dyn Any> = Box::new(Foo);

            assert_that_panic_by(|| {
                assert_that_owned!(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `boxed_any`

                Expected value type: u32

                  Actual value type: dyn core::any::Any

                Details:
                  - A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }

    mod has_type_ref {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::any::Any;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual: Box<dyn Any> = Box::new(String::from("foo"));

            actual.must().have_type_ref::<String>();
        }

        #[test]
        fn succeeds_when_type_matches() {
            let actual: Box<dyn Any> = Box::new(String::from("foo"));

            assert_that!(actual)
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_string() {
            let actual: Box<dyn Any> = Box::new(String::from("foo"));

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
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

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_str() {
            let actual: Box<dyn Any> = Box::new("foo");

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Expected value type: u32

                  Actual value type: &str
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_as_any_when_not_deducible() {
            struct Foo {}
            let actual: Box<dyn Any> = Box::new(Foo {});

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Expected value type: u32

                  Actual value type: dyn core::any::Any

                Details:
                  - A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }
}
