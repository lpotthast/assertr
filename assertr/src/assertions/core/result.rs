use crate::{
    AssertThat, ValueRenderer,
    actual::Actual,
    failure::FailureKind,
    mode::{Mode, Panic},
};

/// Panic-mode extraction from `Result` subjects.
///
/// A failed variant assertion cannot produce the requested subject type.
/// Use [`ResultAssertions::is_ok_satisfying`] or [`ResultAssertions::is_err_satisfying`] in
/// capture mode. Use the non-extracting [`ResultAssertions::is_ok`] or
/// [`ResultAssertions::is_err`] when the contained value is irrelevant.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ResultExtractAssertions<'t, T, E, R> {
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
        self.track_assertion();

        if self.actual().is_err() {
            let actual = match self.actual() {
                Err(error) => self.render().variant(self.actual(), "Err", error),
                Ok(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Ok"))
                .raise();
        }

        // Calling `unwrap` is safe here, as we would have seen a panic when the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(match o {
                Ok(ok) => ok,
                Err(_) => unreachable!("already checked"),
            }),
            Actual::Borrowed(b) => Actual::Borrowed(match b.as_ref() {
                Ok(ok) => ok,
                Err(_) => unreachable!("already checked"),
            }),
        })
    }

    #[track_caller]
    fn get_err(self) -> AssertThat<'t, E, Panic, R>
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();

        if self.actual().is_ok() {
            let actual = match self.actual() {
                Ok(value) => self.render().variant(self.actual(), "Ok", value),
                Err(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Err"))
                .raise();
        }

        // Calling `unwrap_err` is safe here, as we would have seen a panic when the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(match o {
                Ok(_) => unreachable!("already checked"),
                Err(err) => err,
            }),
            Actual::Borrowed(b) => Actual::Borrowed(match b.as_ref() {
                Ok(_) => unreachable!("already checked"),
                Err(err) => err,
            }),
        })
    }
}

/// Non-extracting assertions for `Result` subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ResultAssertions<'t, M: Mode, T, E, R> {
    /// Asserts that the subject is `Ok`.
    ///
    /// Non-extracting: the subject stays the full `Result`, so further assertions can be chained
    /// in any mode. Use [`ResultExtractAssertions::get_ok`] to extract the contained value in
    /// panic mode, or [`ResultAssertions::is_ok_satisfying`] to assert on it in any mode.
    fn is_ok(self) -> Self
    where
        R: ValueRenderer<E>;

    /// Asserts that the subject is `Err`.
    ///
    /// Non-extracting: the subject stays the full `Result`, so further assertions can be chained
    /// in any mode. Use [`ResultExtractAssertions::get_err`] to extract the contained error in
    /// panic mode, or [`ResultAssertions::is_err_satisfying`] to assert on it in any mode.
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

impl<'t, M: Mode, T, E, R> ResultAssertions<'t, M, T, E, R> for AssertThat<'t, Result<T, E>, M, R> {
    #[track_caller]
    fn is_ok(self) -> Self
    where
        R: ValueRenderer<E>,
    {
        self.track_assertion();

        if !self.actual().is_ok() {
            let actual = match self.actual() {
                Err(error) => self.render().variant(self.actual(), "Err", error),
                Ok(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Ok"))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_err(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();

        if !self.actual().is_err() {
            let actual = match self.actual() {
                Ok(value) => self.render().variant(self.actual(), "Ok", value),
                Err(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Err"))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<E> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
    {
        self.track_assertion();

        if self.actual().is_ok() {
            self.satisfies(
                |it| match it.as_ref() {
                    Ok(ok) => ok,
                    Err(_) => unreachable!("already checked"),
                },
                assertions,
            )
        } else {
            let actual = match self.actual() {
                Err(error) => self.render().variant(self.actual(), "Err", error),
                Ok(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Ok"))
                .raise();
            self
        }
    }

    #[track_caller]
    fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> FnOnce(AssertThat<'a, E, M, R>),
    {
        self.track_assertion();

        if self.actual().is_err() {
            self.satisfies(
                |it| match it.as_ref() {
                    Ok(_) => unreachable!("already checked"),
                    Err(err) => err,
                },
                assertions,
            )
        } else {
            let actual = match self.actual() {
                Ok(value) => self.render().variant(self.actual(), "Ok", value),
                Err(_) => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Result::Err"))
                .raise();
            self
        }
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};

        struct Secret;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Result<i32, i32>, Panic, NoRenderer>
                    => ResultAssertions<'static, Panic, i32, i32, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Result<i32, i32>, Panic, NoRenderer>
                    => ResultExtractAssertions<'static, i32, i32, NoRenderer>
            );
        }

        #[test]
        fn err_variant_is_rendered_from_its_leaf_value() {
            let failures = assert_that!(Result::<(), Secret>::Err(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(ResultAssertions::is_ok);

            assert_that!(failures[0].description())
                .contains("Err(")
                .contains(SENTINEL);
        }
    }

    mod is_ok {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<i32, ()>::Ok(42).must().be_ok();
        }

        #[test]
        fn succeeds_when_ok_and_retains_the_subject() {
            assert_that!(Result::<i32, ()>::Ok(42))
                .is_ok()
                .is_equal_to(Ok(42));
        }

        #[test]
        fn panics_when_error() {
            assert_that_panic_by(|| {
                assert_that!(Result::<i32, String>::Err("someError".to_owned()))
                    .with_location(false)
                    .is_ok();
            })
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
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(Result::<i32, String>::Err("someError".to_owned()))
                .with_location(false)
                .capture(|it| it.is_ok().is_err());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `Result::<i32, String>::Err("someError".to_owned())`

                        Actual: Err(
                            "someError",
                        )

                        is not the expected variant

                        Expected: Result::Ok
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_err {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<(), i32>::Err(42).must().be_err();
        }

        #[test]
        fn succeeds_when_error_and_retains_the_subject() {
            assert_that!(Result::<(), i32>::Err(42))
                .is_err()
                .is_equal_to(Err(42));
        }

        #[test]
        fn panics_when_ok() {
            assert_that_panic_by(|| {
                assert_that!(Result::<i32, String>::Ok(42))
                    .with_location(false)
                    .is_err();
            })
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
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<i32, ()>::Ok(42).must().get_ok().is_equal_to(42);
        }

        #[test]
        fn extracts_the_borrowed_inner_value() {
            let result: Result<String, ()> = Ok(String::from("value"));

            assert_that!(result).get_ok().is_equal_to("value");

            // The result was only borrowed and remains usable.
            assert_that!(result).is_ok();
        }

        #[test]
        fn extracts_the_owned_inner_value() {
            assert_that_owned!(Result::<String, ()>::Ok(String::from("value")))
                .get_ok()
                .is_equal_to("value");
        }

        #[test]
        fn panics_when_error() {
            assert_that_panic_by(|| {
                assert_that!(Result::<i32, String>::Err("someError".to_owned()))
                    .with_location(false)
                    .get_ok();
            })
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
    }

    mod get_err {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<(), i32>::Err(42).must().get_err().is_equal_to(42);
        }

        #[test]
        fn extracts_the_borrowed_inner_value() {
            let result: Result<(), String> = Err(String::from("someError"));

            assert_that!(result).get_err().is_equal_to("someError");

            // The result was only borrowed and remains usable.
            assert_that!(result).is_err();
        }

        #[test]
        fn extracts_the_owned_inner_value() {
            assert_that_owned!(Result::<(), String>::Err(String::from("someError")))
                .get_err()
                .is_equal_to("someError");
        }

        #[test]
        fn panics_when_ok() {
            assert_that_panic_by(|| {
                assert_that!(Result::<i32, String>::Ok(42))
                    .with_location(false)
                    .get_err();
            })
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

    mod is_ok_satisfying {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<i32, ()>::Ok(42)
                .must()
                .be_ok_satisfying(|ok_value| {
                    ok_value.is_equal_to(42);
                });
        }

        #[test]
        fn succeeds_when_ok_and_assertions_pass() {
            assert_that!(Result::<i32, ()>::Ok(42)).is_ok_satisfying(|ok_value| {
                ok_value.is_equal_to(42);
            });
        }

        #[test]
        fn hands_out_a_value_typed_assertion_supporting_type_specific_assertions() {
            assert_that!(Result::<String, ()>::Ok(String::from("value"))).is_ok_satisfying(
                |ok_value| {
                    ok_value.contains("alu").starts_with("v");
                },
            );
        }

        #[test]
        fn captures_inner_failure_when_ok_and_assertion_fails() {
            let failures = assert_that!(Result::<i32, ()>::Ok(42))
                .with_location(false)
                .capture(|it| {
                    it.is_ok_satisfying(|ok_value| {
                        ok_value.is_greater_than(9000);
                    })
                });
            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {"
                        -------- assertr --------
                        Actual: 42

                        is not greater than

                        Expected: 9000
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn captures_variant_failure_when_err() {
            let failures = assert_that!(Result::<i32, String>::Err(String::from("boom")))
                .with_location(false)
                .capture(|it| it.is_ok_satisfying(|_| panic!("assertions should not run")));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `Result::<i32, String>::Err(String::from("boom"))`

                        Actual: Err(
                            "boom",
                        )

                        is not the expected variant

                        Expected: Result::Ok
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_err_satisfying {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Result::<(), i32>::Err(42).must().be_err_satisfying(|err| {
                err.is_equal_to(42);
            });
        }

        #[test]
        fn succeeds_when_err_and_assertions_pass() {
            assert_that!(Result::<(), String>::Err(String::from("boom"))).is_err_satisfying(
                |err| {
                    err.contains("oo").starts_with("b");
                },
            );
        }

        #[test]
        fn captures_inner_failure_when_err_and_assertion_fails() {
            let failures = assert_that!(Result::<(), String>::Err(String::from("boom")))
                .with_location(false)
                .capture(|it| {
                    it.is_err_satisfying(|err| {
                        err.contains("xyz");
                    })
                });
            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Actual: "boom"

                        does not contain

                        Expected: "xyz"
                        -------- assertr --------
                    "#});
                },
            ]);
        }

        #[test]
        fn captures_variant_failure_when_ok() {
            let failures = assert_that!(Result::<i32, String>::Ok(42))
                .with_location(false)
                .capture(|it| it.is_err_satisfying(|_| panic!("assertions should not run")));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Result::<i32, String>::Ok(42)`

                        Actual: Ok(
                            42,
                        )

                        is not the expected variant

                        Expected: Result::Err
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }
}
