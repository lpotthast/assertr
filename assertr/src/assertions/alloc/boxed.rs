use crate::{tracking::AssertionTracking, AssertThat, Mode};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use core::any::{type_name, Any};
use indoc::writedoc;
use std::fmt::Write;

/// Assertions for boxed values.
pub trait BoxAssertions<'t, M: Mode> {
    /// If this fails in capturing mode, a panic is raised!
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M>;

    /// If this fails in capturing mode, a panic is raised!
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M>;
}

impl<'t, M: Mode> BoxAssertions<'t, M> for AssertThat<'t, Box<dyn Any>, M> {
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M> {
        self.track_assertion();

        enum CastResult<'c, C> {
            Owned(Box<C>),
            Ref(&'c C),
            Err {
                err: String,
                actual_type_name: Cow<'static, str>,
                actual_type_name_will_be_any: bool,
            },
        }

        let cast = match self.actual {
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
                    Cow::Borrowed(std::any::type_name_of_val(borrowed_boxed_any))
                };

                borrowed_boxed_any
                    .downcast_ref::<E>()
                    .map(|it| CastResult::Ref(it))
                    .unwrap_or_else(|| CastResult::Err {
                        err: String::from("asd"),
                        actual_type_name,
                        actual_type_name_will_be_any,
                    })
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
                    Cow::Borrowed(std::any::type_name_of_val(&*owned_box_any))
                };

                owned_box_any
                    .downcast::<E>()
                    .map(|it| CastResult::Owned(it))
                    .unwrap_or_else(|err| CastResult::Err {
                        err: format!("{err:#?}"),
                        actual_type_name,
                        actual_type_name_will_be_any,
                    })
            }
        };

        match cast {
            CastResult::Owned(casted) => {
                AssertThat {
                    parent: self.parent,
                    actual: (*casted).into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    number_of_assertions: self.number_of_assertions,
                    failures: self.failures,
                    mode: self.mode,
                }
            }
            CastResult::Ref(casted) => {
                AssertThat {
                    parent: self.parent,
                    actual: casted.into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    number_of_assertions: self.number_of_assertions,
                    failures: self.failures,
                    mode: self.mode,
                }
            }
            CastResult::Err {
                err,
                actual_type_name,
                actual_type_name_will_be_any,
            } => {
                let err = AssertThat {
                    parent: self.parent,
                    actual: err.into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    number_of_assertions: self.number_of_assertions,
                    failures: self.failures,
                    mode: self.mode,
                };

                let expected_type_name = type_name::<E>();

                if actual_type_name_will_be_any {
                    err.add_detail_message("A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.");
                };

                err.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Expected value type: {expected_type_name}

                          Actual value type: {actual_type_name}
                    "#}
                });
                panic!("Cannot continue in capturing mode!"); // TODO: Consider typestates!
            }
        }
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M> {
        self.track_assertion();

        let any = &self.actual();
        match any.downcast_ref::<E>() {
            Some(casted) => self.derive(|_actual| casted),
            None => {
                let expected_type_name = type_name::<E>();

                let is_str = any.downcast_ref::<&str>().is_some();
                let is_string = any.downcast_ref::<String>().is_some();

                let actual_type_name = if is_str {
                    Cow::Borrowed("&str")
                } else if is_string {
                    Cow::Borrowed("String")
                } else {
                    // Note: This call to `type_name_of_val` will just return "dyn core::any::Any"...
                    self.add_detail_message("A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.");
                    Cow::Borrowed(std::any::type_name_of_val(&**self.actual()))
                };

                self.fail(|w: &mut String| {
                    writedoc! {w, r#"
                        Expected value type: {expected_type_name}

                          Actual value type: {actual_type_name}
                    "#}
                });
                panic!("Cannot continue in capturing mode!"); // Consider typestates!
            }
        }
    }
}

/*
TODO: implement for &Box?
impl<'t, M: Mode> BoxAssertions<'t, M> for AssertThat<'t, &Box<dyn Any>, M> {
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M> {}

    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M> {}
}
*/

#[cfg(test)]
mod tests {
    mod has_type {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::any::Any;

        #[test]
        fn succeeds_when_type_of_contained_value_matches_expected_type() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            assert_that(boxed_any).has_type::<&str>().is_equal_to("foo");
        }

        #[test]
        fn panics_when_type_of_contained_value_does_not_match_expected_type() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            assert_that_panic_by(|| {
                assert_that(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value type: u32

                      Actual value type: &str
                    -------- assertr --------
                "#});
        }
    }

    mod has_type_ref {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::any::Any;

        #[test]
        fn succeeds_when_type_matches() {
            let actual: Box<dyn Any> = Box::new(String::from("foo"));

            assert_that(actual)
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_string() {
            let actual: Box<dyn Any> = Box::new(String::from("foo"));

            assert_that_panic_by(|| {
                assert_that(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected value type: u32

                  Actual value type: String
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_when_str() {
            let actual: Box<dyn Any> = Box::new("foo");

            assert_that_panic_by(|| {
                assert_that(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected value type: u32

                  Actual value type: &str
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_type_does_not_match_showing_actual_type_as_any_when_not_deducible() {
            struct Foo {}
            let actual: Box<dyn Any> = Box::new(Foo {});

            assert_that_panic_by(|| {
                assert_that(actual)
                    .with_location(false)
                    .has_type_ref::<u32>();
            })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected value type: u32

                  Actual value type: dyn core::any::Any

                Details: [
                    A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.,
                ]
                -------- assertr --------
            "#});
        }
    }
}
