use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, PanicValue,
    actual::Actual,
    failure::{FailureBuilder, FailureKind},
    mode::{Mode, Panic},
};
use alloc::boxed::Box;
use core::any::Any;

use super::boxed::{IsOfType, downcast, explain_type_mismatch};

impl<E: 'static, R> Expectation<PanicValue, R> for IsOfType<E> {
    type Success<'a> = &'a E;
    type Rejection<'a> = &'a dyn Any;
    fn evaluate<'a>(
        &'a self,
        actual: &'a PanicValue,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a E, &'a dyn Any> {
        actual.0.downcast_ref::<E>().ok_or(&*actual.0)
    }
}

impl<E: 'static, R> ExpectationDiagnostics<PanicValue, R> for IsOfType<E> {
    const KIND: FailureKind = FailureKind::Panic;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a PanicValue, &'a dyn Any)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("is of the expected type")
                .expected(core::any::type_name::<E>()),
            Some((_, any)) => explain_type_mismatch::<E, _>(any, failure, ERASED_TYPE_NOTE),
        }
    }
}

/// Explains the erased type name reported for a panic payload that is neither a `&str` nor a
/// `String`.
const ERASED_TYPE_NOTE: &str = "The panic value can only be captured as Box<dyn Any>, meaning that the concrete type was erased. It will be shown as `dyn Any`. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.";

/// Type checks for `PanicValue` subjects in panic and capture mode.
/// Use [`PanicValueExtractAssertions::has_type`] to continue with the downcast payload.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PanicValueAssertions<'t, R = crate::DebugRenderer> {
    /// Asserts that the payload has type `E`, preserving the original subject.
    fn is_of_type<E: 'static>(self) -> Self;
}

impl<'t, M: Mode, R> PanicValueAssertions<'t, R> for AssertThat<'t, PanicValue, M, R> {
    #[track_caller]
    fn is_of_type<E: 'static>(self) -> Self {
        self.apply_assertion(IsOfType::<E>::new())
    }
}

/// Downcasting assertions for [`PanicValue`] subjects.
///
/// These methods are available only in panic mode because a failed downcast cannot produce the
/// requested subject type.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PanicValueExtractAssertions<'t, R = crate::DebugRenderer> {
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

impl<'t, R> PanicValueExtractAssertions<'t, R> for AssertThat<'t, PanicValue, Panic, R> {
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
        let boxed = self
            .apply_assertion_with_failure(IsOfType::<E>::new(), |_, failure| {
                // Extraction has historically reported the boxed payload as its subject type.
                failure.subject_type::<Box<dyn Any>>()
            })
            .map::<Box<dyn Any>>(|actual| match actual {
                Actual::Borrowed(value) => Actual::Borrowed(&value.0),
                Actual::Owned(value) => Actual::Owned(value.0),
            });
        downcast(boxed)
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

#[cfg(test)]
mod tests {
    mod is_of_type {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let value: crate::PanicValue = crate::PanicValue(Box::new("foo"));
            value.must().be_of_type::<&str>();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let value = crate::PanicValue(alloc::boxed::Box::new(1_i32));
            assert_caller_location!(assert_that!(value), is_of_type::<u8>());
        }

        #[test]
        fn checks_the_type_without_extracting_in_both_modes() {
            let value: crate::PanicValue = crate::PanicValue(Box::new("foo"));
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
                        .is_equal_to(crate::FailureKind::Panic);
                },
            ]);
        }

        #[test]
        fn captures_the_exact_type_mismatch_report() {
            let value: crate::PanicValue = crate::PanicValue(Box::new("foo"));
            let failures = assert_that!(value)
                .with_location(false)
                .capture(PanicValueAssertions::is_of_type::<u32>);
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

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(AssertThat<'static, crate::PanicValue, Capture, NoRenderer> => PanicValueAssertions<'static, NoRenderer>);
            assert_trait_impl!(AssertThat<'static, crate::PanicValue, Panic, NoRenderer> => PanicValueAssertions<'static, NoRenderer>);

            assert_trait_impl!(
                AssertThat<'static, crate::PanicValue, Panic, NoRenderer>
                    => PanicValueExtractAssertions<'static, NoRenderer>
            );

            assert_trait_impl!(crate::assertions::alloc::boxed::IsOfType<i32> => crate::ExpectationDiagnostics<crate::PanicValue, NoRenderer>);
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
        fn caller_location_is_as_expected() {
            let value = crate::PanicValue(alloc::boxed::Box::new(1_i32));
            assert_caller_location!(assert_that!(value), has_type::<u8>());
        }

        #[test]
        fn preserves_the_boxed_subject_metadata_for_panic_presentation() {
            use crate::failure::adapter::{Adapter, HumanReadableText};
            use core::{
                any::{Any, type_name},
                convert::Infallible,
            };

            struct SubjectType;
            impl Adapter<AssertionFailure> for SubjectType {
                type Output = HumanReadableText;
                type Error = Infallible;
                fn adapt(
                    &self,
                    failure: &AssertionFailure,
                ) -> Result<HumanReadableText, Infallible> {
                    assert_that!(failure.kind).is_equal_to(crate::FailureKind::Panic);
                    Ok(HumanReadableText::new(failure.subject_type_name))
                }
            }
            let actual = PanicValue(Box::new("text"));
            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_panic_presentation(SubjectType)
                    .has_type::<u32>();
            })
            .has_type::<String>()
            .is_equal_to(type_name::<Box<dyn Any>>());
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
        fn caller_location_is_as_expected() {
            let value = crate::PanicValue(alloc::boxed::Box::new(1_i32));
            assert_caller_location!(assert_that!(value), has_type_ref::<u8>());
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
