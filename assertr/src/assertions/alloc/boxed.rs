use crate::{
    AssertThat, AssertionContext, DebugRenderer, Expectation, ExpectationDiagnostics, Fact,
    assertions::support::project_checked,
    failure::{FailureBuilder, FailureKind},
    mode::{Mode, Panic},
};
use alloc::{boxed::Box, string::String};
use core::any::{Any, type_name, type_name_of_val};

/// Checks a boxed `Any` value's or captured panic payload's concrete type, returning a borrowed
/// value. Both ordinary checks and downcasting assertions execute this definition without a
/// renderer.
///
/// Supported subjects are `Box<dyn Any>`, `Box<dyn Any + Send>`, `Box<dyn Any + Send + Sync>`,
/// and [`PanicValue`](crate::PanicValue).
pub struct IsOfType<E>(core::marker::PhantomData<fn() -> E>);

impl<E> IsOfType<E> {
    /// Selects the expected concrete type.
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<E> Default for IsOfType<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Explains the erased type name reported for a box whose payload is neither a `&str` nor a
/// `String`.
const ERASED_TYPE_NOTE: &str = "The concrete type of a boxed `dyn Any` is erased and shown as `dyn Any`. Only `&str` and `String` values can be named. Check which type was boxed.";

/// Type checks for boxed `Any` values in panic and capture mode.
///
/// Implemented for `Box<dyn Any>`, `Box<dyn Any + Send>`, and `Box<dyn Any + Send + Sync>`
/// subjects, including the payloads returned by `std::panic::catch_unwind` and
/// `std::thread::JoinHandle::join`.
/// Use [`BoxExtractAssertions::has_type`] to continue with the downcast value.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait BoxAssertions<R = DebugRenderer> {
    /// Asserts that the payload has type `E`, preserving the original subject.
    fn is_of_type<E: 'static>(self) -> Self;
}

/// Downcasting assertions for boxed `Any` subjects.
///
/// Implemented for the same subjects as [`BoxAssertions`]. These methods are available only in
/// panic mode because a failed downcast cannot produce the requested subject type.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait BoxExtractAssertions<'t, R = DebugRenderer> {
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

/// Implements the type expectation and both assertion traits for one boxed `Any` trait object.
macro_rules! boxed_any {
    ($($object:ty),+ $(,)?) => {$(
        impl<E: 'static, R> Expectation<Box<$object>, R> for IsOfType<E> {
            type Success<'a> = &'a E;
            type Rejection<'a> = &'a dyn Any;
            fn evaluate<'a>(
                &'a self,
                actual: &'a Box<$object>,
                _: &AssertionContext<'_, R>,
            ) -> Result<&'a E, &'a dyn Any> {
                actual.downcast_ref::<E>().ok_or(&**actual)
            }
        }

        impl<E: 'static, R> ExpectationDiagnostics<Box<$object>, R> for IsOfType<E> {
            const KIND: FailureKind = FailureKind::Variant;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a Box<$object>, &'a dyn Any)>,
                failure: FailureBuilder<Target>,
                _context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                explain_type::<E, _>(rejected.map(|(_, any)| any), failure, ERASED_TYPE_NOTE)
            }
        }

        impl<M: Mode, R> BoxAssertions<R> for AssertThat<'_, Box<$object>, M, R> {
            #[track_caller]
            fn is_of_type<E: 'static>(self) -> Self {
                self.apply_assertion(IsOfType::<E>::new())
            }
        }

        impl<'t, R> BoxExtractAssertions<'t, R> for AssertThat<'t, Box<$object>, Panic, R> {
            #[track_caller]
            fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
                self.apply_assertion(IsOfType::<E>::new()).map(|actual| {
                    project_checked(
                        actual,
                        |boxed| boxed.downcast::<E>().ok().map(|value| *value),
                        |boxed| boxed.downcast_ref::<E>(),
                    )
                })
            }

            #[track_caller]
            fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
            where
                R: Clone,
            {
                let value = self
                    .test_assertion(&const { IsOfType::<E>::new() })
                    .expect("Panic mode raises rejected type checks");
                self.derive_owned(|_| value)
            }
        }
    )+};
}

boxed_any!(dyn Any, dyn Any + Send, dyn Any + Send + Sync);

/// Describes the expected type or names common string payloads and explains other erased types.
pub(super) fn explain_type<E: 'static, Target>(
    rejected: Option<&dyn Any>,
    failure: FailureBuilder<Target>,
    erased_note: &'static str,
) -> FailureBuilder<Target> {
    let Some(any) = rejected else {
        return failure
            .relation("is of the expected type")
            .expected(type_name::<E>());
    };
    let (actual_type_name, erased) = if any.is::<&str>() {
        ("&str", false)
    } else if any.is::<String>() {
        ("String", false)
    } else {
        (type_name_of_val(any), true)
    };
    let failure = failure
        .actual(actual_type_name)
        .relation("is not of the expected type")
        .expected(type_name::<E>());
    if erased {
        failure.fact(Fact::note(erased_note))
    } else {
        failure
    }
}

#[cfg(test)]
mod tests {
    mod is_of_type {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let value: Box<dyn core::any::Any> = Box::new("foo");
            value.must().be_of_type::<&str>();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let value: Box<dyn core::any::Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), is_of_type::<u8>());
        }

        #[test]
        fn checks_the_type_without_extracting_in_both_modes() {
            let value: Box<dyn core::any::Any> = Box::new("foo");
            assert_that!(value)
                .is_of_type::<&str>()
                .has_type::<&str>()
                .is_equal_to("foo");
            let failures =
                assert_that!(value).capture(|it| it.is_of_type::<String>().is_of_type::<&str>());
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(crate::FailureKind::Variant);
                },
            ]);
        }

        #[test]
        fn captures_the_exact_type_mismatch_report() {
            let value: Box<dyn core::any::Any> = Box::new("foo");
            let failures = assert_that!(value)
                .with_location(false)
                .capture(BoxAssertions::is_of_type::<u32>);
            assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {"
                -------- assertr --------
                Expression: `value`

                Actual: &str

                is not of the expected type

                Expected: u32
                -------- assertr --------
            "});
        }
    }

    mod thread_safe_payloads {
        use crate::prelude::*;
        use core::any::Any;

        #[test]
        fn supports_send_boxes_from_caught_panics() {
            let payload: Box<dyn Any + Send> =
                std::panic::catch_unwind(|| panic!("boom")).unwrap_err();
            assert_that!(payload)
                .is_of_type::<&str>()
                .has_type::<&str>()
                .is_equal_to("boom");

            let payload: Box<dyn Any + Send> =
                std::thread::spawn(|| panic!("joined")).join().unwrap_err();
            assert_that_owned!(payload)
                .has_type::<&str>()
                .is_equal_to("joined");
        }

        #[test]
        fn supports_send_sync_boxes() {
            let value: Box<dyn Any + Send + Sync> = Box::new(String::from("foo"));
            assert_that!(value)
                .is_of_type::<String>()
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
            assert_that_owned!(value)
                .has_type::<String>()
                .is_equal_to(String::from("foo"));
        }

        #[test]
        fn reports_the_same_mismatch_for_every_box() {
            struct Foo;
            let report = |failures: AssertionFailures| failures[0].to_string();
            let expected = {
                let value: Box<dyn Any> = Box::new(Foo);
                report(
                    assert_that!(value)
                        .with_location(false)
                        .capture(BoxAssertions::is_of_type::<u32>),
                )
            };
            let send = {
                let value: Box<dyn Any + Send> = Box::new(Foo);
                report(
                    assert_that!(value)
                        .with_location(false)
                        .capture(BoxAssertions::is_of_type::<u32>),
                )
            };
            let send_sync = {
                let value: Box<dyn Any + Send + Sync> = Box::new(Foo);
                report(
                    assert_that!(value)
                        .with_location(false)
                        .capture(BoxAssertions::is_of_type::<u32>),
                )
            };
            assert_that!(send).is_equal_to(&expected);
            assert_that!(send_sync).is_equal_to(&expected);
        }
    }

    mod renderer_contract {
        use alloc::boxed::Box;

        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(AssertThat<'static, Box<dyn core::any::Any>, Capture, NoRenderer> => BoxAssertions<NoRenderer>);
            assert_trait_impl!(AssertThat<'static, Box<dyn core::any::Any>, Panic, NoRenderer> => BoxAssertions<NoRenderer>);

            assert_trait_impl!(
                AssertThat<'static, Box<dyn core::any::Any>, Panic, NoRenderer>
                    => BoxExtractAssertions<'static, NoRenderer>
            );

            assert_trait_impl!(super::super::IsOfType<i32> => ExpectationDiagnostics<Box<dyn core::any::Any>, NoRenderer>);
            assert_trait_impl!(super::super::IsOfType<i32> => ExpectationDiagnostics<Box<dyn core::any::Any + Send>, NoRenderer>);
            assert_trait_impl!(super::super::IsOfType<i32> => ExpectationDiagnostics<Box<dyn core::any::Any + Send + Sync>, NoRenderer>);
            assert_trait_impl!(AssertThat<'static, Box<dyn core::any::Any + Send>, Capture, NoRenderer> => BoxAssertions<NoRenderer>);
            assert_trait_impl!(
                AssertThat<'static, Box<dyn core::any::Any + Send + Sync>, Panic, NoRenderer>
                    => BoxExtractAssertions<'static, NoRenderer>
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
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), has_type::<u8>());
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
                  - The concrete type of a boxed `dyn Any` is erased and shown as `dyn Any`. Only `&str` and `String` values can be named. Check which type was boxed.
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
                  - The concrete type of a boxed `dyn Any` is erased and shown as `dyn Any`. Only `&str` and `String` values can be named. Check which type was boxed.
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
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), has_type_ref::<u8>());
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
                  - The concrete type of a boxed `dyn Any` is erased and shown as `dyn Any`. Only `&str` and `String` values can be named. Check which type was boxed.
                -------- assertr --------
            "});
        }
    }
}
