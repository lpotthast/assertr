use crate::{
    AssertThat,
    assertions::support::{explain_variant, project_checked},
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    mode::{Mode, Panic},
    renderer::{DebugRenderer, ValueRenderer},
};

/// Checks for `Ok` and returns a borrowed value on success.
/// Checks, extraction, and ordinary callbacks execute this same definition.
#[derive(Debug, Clone, Copy)]
pub struct IsOk;

impl<T, E, R: ValueRenderer<E>> Expectation<Result<T, E>, R> for IsOk {
    type Success<'a>
        = &'a T
    where
        Self: 'a,
        Result<T, E>: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        Result<T, E>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Result<T, E>,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a T, &'a E> {
        actual.as_ref()
    }

    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Result<T, E>, &'a E)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        explain_variant(
            failure,
            rejected.map(|(actual, value)| render.variant(actual, "Err", value)),
            "Result::Ok",
        )
    }
}

/// Checks for `Err` and returns a borrowed error on success.
/// Checks, extraction, and ordinary callbacks execute this same definition.
#[derive(Debug, Clone, Copy)]
pub struct IsErr;

impl<T, E, R: ValueRenderer<T>> Expectation<Result<T, E>, R> for IsErr {
    type Success<'a>
        = &'a E
    where
        Self: 'a,
        Result<T, E>: 'a;
    type Rejection<'a>
        = &'a T
    where
        Self: 'a,
        Result<T, E>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Result<T, E>,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a E, &'a T> {
        match actual {
            Err(value) => Ok(value),
            Ok(value) => Err(value),
        }
    }

    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Result<T, E>, &'a T)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        explain_variant(
            failure,
            rejected.map(|(actual, value)| render.variant(actual, "Ok", value)),
            "Result::Err",
        )
    }
}

/// Panic-mode extraction from `Result` subjects.
///
/// A failed variant assertion cannot produce the requested subject type. Use
/// [`ResultAssertions::is_ok_satisfying`] or [`ResultAssertions::is_err_satisfying`] in capture
/// mode. Use the non-extracting [`ResultAssertions::is_ok`] or [`ResultAssertions::is_err`] when
/// the contained value is irrelevant.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ResultExtractAssertions<'t, T, E, R = DebugRenderer> {
    /// Asserts that the subject is `Ok`, then returns an assertion over its value.
    ///
    /// A borrowed subject yields a borrowed value. An owned subject yields an owned value.
    fn get_ok(self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<E>;

    /// Asserts that the subject is `Err`, then returns an assertion over its error.
    ///
    /// A borrowed subject yields a borrowed error. An owned subject yields an owned error.
    fn get_err(self) -> AssertThat<'t, E, Panic, R>
    where
        R: ValueRenderer<T>;
}

impl<'t, T, E, R> ResultExtractAssertions<'t, T, E, R> for AssertThat<'t, Result<T, E>, Panic, R> {
    #[track_caller]
    fn get_ok(self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<E>,
    {
        self.matches(IsOk)
            .map(|actual| project_checked(actual, Result::ok, |it| it.as_ref().ok()))
    }

    #[track_caller]
    fn get_err(self) -> AssertThat<'t, E, Panic, R>
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsErr)
            .map(|actual| project_checked(actual, Result::err, |it| it.as_ref().err()))
    }
}

/// Non-extracting assertions for `Result` subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ResultAssertions<T, E, M: Mode, R = DebugRenderer> {
    /// Asserts that the subject is `Ok`.
    ///
    /// Non-extracting: the subject stays the full `Result`, so further assertions can be chained in
    /// any mode. Use [`ResultExtractAssertions::get_ok`] to extract the contained value in panic
    /// mode, or [`ResultAssertions::is_ok_satisfying`] to assert on it in any mode.
    fn is_ok(self) -> Self
    where
        R: ValueRenderer<E>;

    /// Asserts that the subject is `Err`.
    ///
    /// Non-extracting: the subject stays the full `Result`, so further assertions can be chained in
    /// any mode. Use [`ResultExtractAssertions::get_err`] to extract the contained error in panic
    /// mode, or [`ResultAssertions::is_err_satisfying`] to assert on it in any mode.
    fn is_err(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject is `Ok`, then runs `assertions` on its value.
    ///
    /// The closure receives an `AssertThat<T>` borrowing the contained value.
    fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<E> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>);

    /// Asserts that the subject is `Err`, then runs `assertions` on its error.
    ///
    /// The closure receives an `AssertThat<E>` borrowing the contained error.
    fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>);
}

impl<T, E, M: Mode, R> ResultAssertions<T, E, M, R> for AssertThat<'_, Result<T, E>, M, R> {
    #[track_caller]
    fn is_ok(self) -> Self
    where
        R: ValueRenderer<E>,
    {
        self.matches(IsOk)
    }

    #[track_caller]
    fn is_err(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsErr)
    }

    #[track_caller]
    fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<E> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
    {
        self.satisfy_success(self.test_assertion(&IsOk), assertions);
        self
    }

    #[track_caller]
    fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
    {
        self.satisfy_success(self.test_assertion(&IsErr), assertions);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{failure::FailureKind, prelude::*, test_support::rejected_kind};

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            Result::<i32, ()>::Ok(42).must().be_ok();
            Result::<(), i32>::Err(42).must().be_err();
            Result::<i32, ()>::Ok(42).must().be_ok_satisfying(|ok| {
                ok.is_equal_to(42);
            });
            Result::<(), i32>::Err(42).must().be_err_satisfying(|err| {
                err.is_equal_to(42);
            });
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        struct Secret;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Result<i32, i32>, Panic, NoRenderer>
                    => ResultAssertions<i32, i32, Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Result<i32, i32>, Panic, NoRenderer>
                    => ResultExtractAssertions<'static, i32, i32, NoRenderer>
            );
        }

        #[test]
        fn rejected_variants_are_rendered_from_their_leaf_values() {
            let failures = assert_that!(Result::<(), Secret>::Err(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(ResultAssertions::is_ok);
            assert_that!(failures[0].to_string())
                .contains("Err(")
                .contains(SENTINEL);
            let failures = assert_that!(Result::<Secret, ()>::Ok(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(ResultAssertions::is_err);
            assert_that!(failures[0].to_string())
                .contains("Ok(")
                .contains(SENTINEL);
        }
    }

    mod is_ok {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<i32, String>::Err("someError".to_owned())),
                is_ok()
            );
        }

        #[test]
        fn succeeds_when_ok_and_retains_the_subject() {
            assert_that!(Result::<i32, ()>::Ok(42))
                .is_ok()
                .is_equal_to(Ok(42));
        }

        #[test]
        fn panics_when_error() {
            assert_that!(|| {
                assert_that!(Result::<i32, String>::Err("someError".to_owned()))
                    .with_location(false)
                    .is_ok();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `Result::<i32, String>::Err("someError".to_owned())`

                Actual: Err(
                    "someError",
                )

                is not the expected variant

                Expected: Result::Ok
                -------- assertr --------
            "#});
        }

        #[test]
        fn continues_in_capture_mode() {
            let failures = assert_that!(Result::<i32, String>::Err("someError".to_owned()))
                .capture(|it| it.is_ok().is_err());
            assert_that!(failures).has_length(1);
        }
    }

    mod is_err {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Result::<i32, String>::Ok(42)), is_err());
        }

        #[test]
        fn succeeds_when_error_and_retains_the_subject() {
            assert_that!(Result::<(), i32>::Err(42))
                .is_err()
                .is_equal_to(Err(42));
        }

        #[test]
        fn panics_when_ok() {
            assert_that!(|| {
                assert_that!(Result::<i32, String>::Ok(42))
                    .with_location(false)
                    .is_err();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Result::<i32, String>::Ok(42)`

                Actual: Ok(
                    42,
                )

                is not the expected variant

                Expected: Result::Err
                -------- assertr --------
            "});
        }
    }

    mod get_ok {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<i32, String>::Err("someError".to_owned())),
                get_ok()
            );
        }

        #[test]
        fn extracts_borrowed_and_owned_values() {
            let result: Result<String, ()> = Ok(String::from("value"));
            assert_that!(result).get_ok().is_equal_to("value");
            assert_that_owned!(result).get_ok().is_equal_to("value");
        }

        #[test]
        fn rejects_errors_like_is_ok() {
            rejected_kind(FailureKind::Variant, || {
                let _ = assert_that!(Result::<i32, i32>::Err(1))
                    .with_panic_presentation(|failure| format!("{:?}", failure.kind))
                    .get_ok();
            });
        }
    }

    mod get_err {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Result::<i32, String>::Ok(42)), get_err());
        }

        #[test]
        fn extracts_borrowed_and_owned_errors() {
            let result: Result<(), String> = Err(String::from("someError"));
            assert_that!(result).get_err().is_equal_to("someError");
            assert_that_owned!(result)
                .get_err()
                .is_equal_to("someError");
        }

        #[test]
        fn rejects_values_like_is_err() {
            rejected_kind(FailureKind::Variant, || {
                let _ = assert_that!(Result::<i32, i32>::Ok(1))
                    .with_panic_presentation(|failure| format!("{:?}", failure.kind))
                    .get_err();
            });
        }
    }

    mod is_ok_satisfying {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<i32, i32>::Err(1)),
                is_ok_satisfying(|_| {})
            );
        }

        #[test]
        fn retains_fn_once_callbacks_and_tracks_the_variant_once() {
            let owned = String::from("consumed by callback");
            let assertion =
                assert_that!(Ok::<_, ()>(3)).is_ok_satisfying(|it: AssertThat<'_, i32, Panic>| {
                    drop(owned);
                    it.is_equal_to(3);
                });
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(2);
        }

        #[test]
        fn does_not_run_the_callback_for_errors() {
            let failures = assert_that!(Result::<i32, i32>::Err(1))
                .capture(|it| it.is_ok_satisfying(|_| panic!("assertions should not run")));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].kind).is_equal_to(FailureKind::Variant);
        }
    }

    mod is_err_satisfying {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Result::<i32, i32>::Ok(1)),
                is_err_satisfying(|_| {})
            );
        }

        #[test]
        fn hands_out_an_error_typed_assertion() {
            let failures =
                assert_that!(Result::<(), String>::Err(String::from("boom"))).capture(|it| {
                    it.is_err_satisfying(|err| {
                        err.contains("oo").contains("xyz");
                    })
                });
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains("does not contain");
        }

        #[test]
        fn does_not_run_the_callback_for_values() {
            let failures = assert_that!(Result::<i32, i32>::Ok(1))
                .capture(|it| it.is_err_satisfying(|_| panic!("assertions should not run")));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].kind).is_equal_to(FailureKind::Variant);
        }
    }
}
