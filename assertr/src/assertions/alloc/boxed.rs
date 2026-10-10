use alloc::{boxed::Box, string::String};
use core::any::{Any, type_name, type_name_of_val};

use crate::{
    AssertThat, PanicValue,
    actual::Actual,
    assertions::support::project_checked,
    entry::panic_message,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::{Mode, Panic},
    renderer::DebugRenderer,
};

mod payload {
    use super::{Any, Box, PanicValue};

    /// A type-erased payload: a boxed `Any` value or a captured panic payload.
    ///
    /// This trait is sealed. Its implementations are `Box<dyn Any>`, `Box<dyn Any + Send>`,
    /// `Box<dyn Any + Send + Sync>`, and [`PanicValue`].
    pub trait Payload: 'static {
        /// Explains the erased type name reported for a payload that is neither a `&str` nor a
        /// `String`.
        const ERASED_TYPE_NOTE: &'static str;

        /// Borrows the erased payload.
        fn payload(&self) -> &dyn Any;

        /// Takes the erased payload.
        fn into_any(self) -> Box<dyn Any>;
    }

    const BOX_NOTE: &str = "The concrete type of a boxed `dyn Any` is erased. Only `&str` and `String` values can be named. Check which type was boxed.";

    macro_rules! boxed_any {
        ($($object:ty),+) => {$(
            impl Payload for Box<$object> {
                const ERASED_TYPE_NOTE: &'static str = BOX_NOTE;

                fn payload(&self) -> &dyn Any {
                    &**self
                }

                fn into_any(self) -> Box<dyn Any> {
                    self
                }
            }
        )+};
    }

    boxed_any!(dyn Any, dyn Any + Send, dyn Any + Send + Sync);

    impl Payload for PanicValue {
        const ERASED_TYPE_NOTE: &'static str = "The panic value can only be captured as Box<dyn Any + Send>, meaning that the concrete type was erased. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.";

        fn payload(&self) -> &dyn Any {
            &*self.0
        }

        fn into_any(self) -> Box<dyn Any> {
            self.0
        }
    }
}

use payload::Payload;

/// Checks a boxed `Any` value's or captured panic payload's concrete type, returning a borrowed
/// value. Both ordinary checks and downcasting assertions execute this definition without a
/// renderer.
///
/// Supported subjects are `Box<dyn Any>`, `Box<dyn Any + Send>`, `Box<dyn Any + Send + Sync>`,
/// and [`PanicValue`].
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

type_selection_traits!(IsOfType);

impl<E: 'static, P: Payload, R> Expectation<P, R> for IsOfType<E> {
    type Success<'a>
        = &'a E
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = &'a dyn Any
    where
        Self: 'a,
        P: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a E, &'a dyn Any> {
        let payload = actual.payload();
        payload.downcast_ref::<E>().ok_or(payload)
    }

    const KIND: FailureKind = FailureKind::Variant;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a P, &'a dyn Any)>,
        failure: FailureBuilder,
        _: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let failure = failure.expected(type_name::<E>());
        let Some((_, payload)) = rejected else {
            return failure.relation("is of the expected type");
        };
        let failure = failure.relation("is not of the expected type");
        if payload.is::<&str>() {
            failure.actual("&str")
        } else if payload.is::<String>() {
            failure.actual("String")
        } else {
            failure
                .actual(type_name_of_val(payload))
                .fact(Fact::note(P::ERASED_TYPE_NOTE))
        }
    }
}

/// Checks that a boxed `Any` value or captured panic payload is a `&str` or `String` message,
/// returning the borrowed message on success.
///
/// `panic!("literal")` raises a `&str` payload, while `panic!("{value}")` raises a `String`.
/// This definition accepts both, so a message check does not depend on how the panic was
/// formatted. [`BoxExtractAssertions::has_message`] executes this same definition and continues
/// with the message. Supported subjects are the same as for [`IsOfType`].
///
/// ```
/// use assertr::{matchers::HasPanicMessage, prelude::*};
///
/// let id = 7;
/// let payload = std::panic::catch_unwind(|| panic!("no user {id}")).unwrap_err();
/// assert_that!(payload).matches(HasPanicMessage);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct HasPanicMessage;

impl<P: Payload, R> Expectation<P, R> for HasPanicMessage {
    type Success<'a>
        = &'a str
    where
        Self: 'a,
        P: 'a;
    type Rejection<'a>
        = &'a dyn Any
    where
        Self: 'a,
        P: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a P,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a str, &'a dyn Any> {
        let payload = actual.payload();
        panic_message(payload).ok_or(payload)
    }

    const KIND: FailureKind = FailureKind::Variant;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a P, &'a dyn Any)>,
        failure: FailureBuilder,
        _: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let failure = failure.expected("&str or String");
        let Some((_, payload)) = rejected else {
            return failure.relation("is a string message");
        };
        failure
            .actual(type_name_of_val(payload))
            .relation("is not a string message")
            .fact(Fact::note(P::ERASED_TYPE_NOTE))
    }
}

/// Takes the message out of a payload that [`HasPanicMessage`] accepted.
fn into_message(payload: Box<dyn Any>) -> Option<String> {
    match payload.downcast::<String>() {
        Ok(message) => Some(*message),
        Err(payload) => panic_message(&*payload).map(String::from),
    }
}

/// Type checks for boxed `Any` values and captured panic payloads in panic and capture mode.
///
/// Implemented for `Box<dyn Any>`, `Box<dyn Any + Send>`, and `Box<dyn Any + Send + Sync>`
/// subjects, including the payloads returned by `std::panic::catch_unwind` and
/// `std::thread::JoinHandle::join`, and for [`PanicValue`] subjects.
/// Use [`BoxExtractAssertions::has_type`] to continue with the downcast value.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait BoxAssertions {
    /// Asserts that the payload has type `E`, preserving the original subject.
    fn is_of_type<E: 'static>(self) -> Self;
}

/// Downcasting assertions for boxed `Any` values and captured panic payloads.
///
/// Implemented for the same subjects as [`BoxAssertions`]. These methods are available only in
/// panic mode because a failed downcast cannot produce the requested subject type.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait BoxExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that the payload has type `E` and returns an assertion over that value.
    ///
    /// An owned subject produces an `AssertThat<E>` owning `E`. A borrowed subject produces an
    /// `AssertThat<E>` borrowing it.
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R>;

    /// Asserts that the payload has type `E` and returns an assertion over `&E`.
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone;

    /// Asserts that the payload is a `&str` or `String` message and returns an assertion over
    /// that message.
    ///
    /// `panic!("literal")` raises a `&str` payload, while `panic!("{value}")` raises a `String`.
    /// This method accepts both, so prefer it over [`has_type`](Self::has_type) when only the
    /// message matters. The message is copied into a `String` unless an owned subject already
    /// holds one.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let id = 7;
    /// # #[cfg(feature = "std")]
    /// assert_that!(move || panic!("no user {id}"))
    ///     .panics()
    ///     .has_message()
    ///     .is_equal_to("no user 7");
    /// ```
    fn has_message(self) -> AssertThat<'t, String, Panic, R>;
}

impl<P: Payload, M: Mode, R> BoxAssertions for AssertThat<'_, P, M, R> {
    #[track_caller]
    fn is_of_type<E: 'static>(self) -> Self {
        self.matches(IsOfType::<E>::new())
    }
}

impl<'t, P: Payload, R> BoxExtractAssertions<'t, R> for AssertThat<'t, P, Panic, R> {
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, Panic, R> {
        self.matches(IsOfType::<E>::new()).map(|actual| {
            project_checked(
                actual,
                |payload| payload.into_any().downcast::<E>().ok().map(|value| *value),
                |payload| payload.payload().downcast_ref::<E>(),
            )
        })
    }

    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, Panic, R>
    where
        R: Clone,
    {
        let value = self.require(&const { IsOfType::<E>::new() });
        self.derive_owned(|_| value)
    }

    #[track_caller]
    fn has_message(self) -> AssertThat<'t, String, Panic, R> {
        self.matches(HasPanicMessage).map(|actual| {
            let message = match actual {
                Actual::Owned(payload) => into_message(payload.into_any()),
                Actual::Borrowed(payload) => panic_message(payload.payload()).map(String::from),
            };
            Actual::Owned(
                message
                    .unwrap_or_else(|| unreachable!("the expectation already checked the subject")),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use core::any::Any;

    use indoc::formatdoc;

    use crate::{PanicValue, prelude::*};

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        fn are_as_expected() {
            let value: Box<dyn Any> = Box::new("foo");
            value.must().be_of_type::<&str>();
            value.must().have_type::<&str>();
            value.must().have_type_ref::<&str>();
            value.must().have_message();
        }
    }

    mod is_of_type {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), is_of_type::<u8>());
        }

        #[test]
        fn checks_the_type_without_extracting_in_both_modes() {
            let value = PanicValue(Box::new("foo"));
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
                        .is_equal_to(crate::failure::FailureKind::Variant);
                },
            ]);
        }

        #[test]
        fn names_string_payloads_and_reports_the_same_mismatch_for_every_payload() {
            fn report<P: super::super::Payload>(value: P) -> String {
                assert_that_owned!(value)
                    .with_location(false)
                    .capture(BoxAssertions::is_of_type::<u32>)[0]
                    .to_string()
            }
            let expected = report::<Box<dyn Any>>(Box::new(String::from("foo")));
            assert_that!(&expected).contains("Actual: String");
            for actual in [
                report::<Box<dyn Any + Send>>(Box::new(String::from("foo"))),
                report::<Box<dyn Any + Send + Sync>>(Box::new(String::from("foo"))),
                report(PanicValue(Box::new(String::from("foo")))),
            ] {
                assert_that!(actual).is_equal_to(&expected);
            }
        }
    }

    mod has_type {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), has_type::<u8>());
        }

        #[test]
        fn extracts_owned_and_borrowed_payloads() {
            let boxed: Box<dyn Any> = Box::new("foo");
            assert_that!(boxed).has_type::<&str>().is_equal_to("foo");
            assert_that!(&boxed).has_type::<&str>().is_equal_to("foo");
            assert_that_owned!(boxed)
                .has_type::<&str>()
                .is_equal_to("foo");
            let value = PanicValue(Box::new(String::from("foo")));
            assert_that!(value).has_type::<String>().is_equal_to("foo");
            assert_that_owned!(value)
                .has_type::<String>()
                .is_equal_to(String::from("foo"));
        }

        #[test]
        fn reports_the_subject_like_is_of_type() {
            use core::any::type_name;

            for extract in [true, false] {
                let actual = PanicValue(Box::new("text"));
                assert_that!(|| {
                    let assertion = assert_that!(actual).with_panic_presentation(|failure| {
                        assert_that!(failure.kind)
                            .is_equal_to(crate::failure::FailureKind::Variant);
                        String::from(failure.subject_type_name)
                    });
                    if extract {
                        let _ = assertion.has_type::<u32>();
                    } else {
                        assertion.is_of_type::<u32>();
                    }
                })
                .panics()
                .has_type::<String>()
                .is_equal_to(type_name::<PanicValue>());
            }
        }

        #[test]
        fn panics_when_type_of_contained_value_does_not_match_expected_type() {
            let boxed_any: Box<dyn Any> = Box::new("foo");

            assert_that!(|| {
                assert_that!(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            })
            .panics()
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
        fn explains_erased_box_types() {
            struct Foo;
            let boxed_any: Box<dyn Any> = Box::new(Foo);

            assert_that!(|| {
                assert_that_owned!(boxed_any)
                    .with_location(false)
                    .has_type::<u32>();
            }).panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `boxed_any`

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

                Details:
                  - The concrete type of a boxed `dyn Any` is erased. Only `&str` and `String` values can be named. Check which type was boxed.
                -------- assertr --------
            "});
        }

        #[test]
        fn explains_erased_panic_payload_types() {
            struct Foo;
            let actual = PanicValue(Box::new(Foo));

            assert_that!(|| {
                assert_that!(actual).with_location(false).has_type::<u32>();
            }).panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Actual: dyn core::any::Any

                is not of the expected type

                Expected: u32

                Details:
                  - The panic value can only be captured as Box<dyn Any + Send>, meaning that the concrete type was erased. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }

    mod has_type_ref {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), has_type_ref::<u8>());
        }

        #[test]
        fn succeeds_when_type_matches() {
            let actual: Box<dyn Any + Send + Sync> = Box::new(String::from("foo"));
            assert_that!(actual)
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
            let actual = PanicValue(Box::new(String::from("foo")));
            assert_that!(actual)
                .has_type_ref::<String>()
                .is_equal_to(&String::from("foo"));
        }
    }

    mod has_message {
        use super::*;
        use crate::matchers::HasPanicMessage;

        #[test]
        fn caller_location_is_as_expected() {
            let value: Box<dyn Any> = Box::new(1_i32);
            assert_caller_location!(assert_that!(value), has_message());
        }

        #[test]
        fn extracts_literal_and_formatted_messages_from_owned_and_borrowed_payloads() {
            let id = 7;
            assert_that!(|| panic!("literal"))
                .panics()
                .has_message()
                .is_equal_to("literal");
            assert_that!(move || panic!("formatted {id}"))
                .panics()
                .has_message()
                .is_equal_to("formatted 7");
            let literal: Box<dyn Any + Send> = Box::new("literal");
            assert_that!(literal).has_message().is_equal_to("literal");
            assert_that_owned!(literal)
                .has_message()
                .is_equal_to("literal");
            let owned = PanicValue(Box::new(String::from("owned")));
            assert_that!(owned).has_message().is_equal_to("owned");
            assert_that_owned!(owned).has_message().is_equal_to("owned");
        }

        #[test]
        fn matcher_checks_messages_in_capture_mode() {
            let failures = assert_that!(PanicValue(Box::new(3_u8)))
                .capture(|it| it.matches(HasPanicMessage).matches(HasPanicMessage));
            assert_that!(failures).has_length(2);
            assert_that!(PanicValue(Box::new("message"))).matches(HasPanicMessage);
        }

        #[test]
        fn explains_payloads_without_a_message() {
            let actual = PanicValue(Box::new(3_u8));

            assert_that!(|| {
                let _ = assert_that!(actual).with_location(false).has_message();
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `actual`

                Actual: dyn core::any::Any

                is not a string message

                Expected: &str or String

                Details:
                  - The panic value can only be captured as Box<dyn Any + Send>, meaning that the concrete type was erased. We already checked for both `&str` and `String`. Try other common types used for panic values or analyze your panicking code.
                -------- assertr --------
            "});
        }
    }

    mod thread_safe_payloads {
        use super::*;

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
    }

    mod renderer_contract {
        use super::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(AssertThat<'static, Box<dyn Any>, Capture, NoRenderer> => BoxAssertions);
            assert_trait_impl!(AssertThat<'static, Box<dyn Any + Send>, Capture, NoRenderer> => BoxAssertions);
            assert_trait_impl!(AssertThat<'static, PanicValue, Capture, NoRenderer> => BoxAssertions);
            assert_trait_impl!(
                AssertThat<'static, Box<dyn Any>, Panic, NoRenderer>
                    => BoxExtractAssertions<'static, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Box<dyn Any + Send + Sync>, Panic, NoRenderer>
                    => BoxExtractAssertions<'static, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, PanicValue, Panic, NoRenderer>
                    => BoxExtractAssertions<'static, NoRenderer>
            );
        }
    }
}
