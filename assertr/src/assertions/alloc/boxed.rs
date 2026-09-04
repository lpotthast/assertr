use crate::{AssertThat, actual::Actual, failure::FailureKind, mode::Panic};
use alloc::boxed::Box;
use alloc::string::String;
use core::any::{Any, type_name, type_name_of_val};

/// Explains the erased type name reported for a box whose payload is neither a `&str` nor a
/// `String`.
const ERASED_TYPE_NOTE: &str = "A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.";

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
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
        downcast(self, FailureKind::Variant, ERASED_TYPE_NOTE)
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone,
    {
        self.track_assertion();

        match self.actual().downcast_ref::<E>() {
            Some(casted) => self.derive_owned(|_actual| casted),
            None => raise_type_mismatch::<_, _, E>(
                self,
                FailureKind::Variant,
                &**self.actual(),
                ERASED_TYPE_NOTE,
            ),
        }
    }
}

/// Downcasts a boxed `Any` subject to `E`.
///
/// This is the body of every `has_type` over a `Box<dyn Any>`, shared with the panic-payload
/// assertions. A box holding another type raises a failure of `kind`, with `erased_note`
/// attached when that type cannot be named.
#[track_caller]
pub(super) fn downcast<'t, E: 'static, R>(
    this: AssertThat<'t, Box<dyn Any>, Panic, R>,
    kind: FailureKind,
    erased_note: &'static str,
) -> AssertThat<'t, E, Panic, R> {
    this.track_assertion();
    let AssertThat { actual, state } = this;

    let actual = match actual {
        Actual::Borrowed(boxed) => match boxed.downcast_ref::<E>() {
            Some(casted) => {
                return AssertThat {
                    actual: Actual::Borrowed(casted),
                    state,
                };
            }
            None => Actual::Borrowed(boxed),
        },
        Actual::Owned(boxed) => match boxed.downcast::<E>() {
            Ok(casted) => {
                return AssertThat {
                    actual: Actual::Owned(*casted),
                    state,
                };
            }
            Err(boxed) => Actual::Owned(boxed),
        },
    };

    let this = AssertThat { actual, state };
    raise_type_mismatch::<_, _, E>(&this, kind, &**this.actual(), erased_note)
}

/// Raises the failure of a downcast of `any` to `E` on a panic-mode chain and therefore never
/// returns.
///
/// The payload types `panic!` produces, `&str` and `String`, are named. Any other type is
/// reported as the erased `dyn Any`, explained by `erased_note`.
#[track_caller]
pub(super) fn raise_type_mismatch<T, R, E: 'static>(
    this: &AssertThat<'_, T, Panic, R>,
    kind: FailureKind,
    any: &dyn Any,
    erased_note: &'static str,
) -> ! {
    let (actual_type_name, erased) = if any.is::<&str>() {
        ("&str", false)
    } else if any.is::<String>() {
        ("String", false)
    } else {
        // `type_name_of_val` cannot see through the trait object and yields "dyn core::any::Any".
        (type_name_of_val(any), true)
    };

    let mut failure = this
        .failure(kind)
        .actual(format_args!("{actual_type_name}"))
        .relation("is not of the expected type")
        .expected(format_args!("{}", type_name::<E>()));
    if erased {
        failure = failure.note(erased_note);
    }
    failure.raise();
    unreachable!("Panic mode always panics on fail")
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

                    Actual: &str

                    is not of the expected type

                    Expected: u32
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

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

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

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

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

                Actual: String

                is not of the expected type

                Expected: u32
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

                Actual: &str

                is not of the expected type

                Expected: u32
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

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

                Details:
                  - A Box<dyn Any> means that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }
}
