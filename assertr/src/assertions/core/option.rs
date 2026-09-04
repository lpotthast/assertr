use crate::{AssertThat, Mode, ValueRenderer, actual::Actual, failure::FailureKind, mode::Panic};
use core::option::Option;

/// Panic-mode extraction from `Option` subjects.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionExtractAssertions<'t, T, R> {
    /// Asserts that the subject is `Some`, then returns an assertion over its value.
    ///
    /// A borrowed subject yields a borrowed value. An owned subject yields an owned value.
    ///
    /// This is available only in `Panic` mode because `None` cannot produce a `T`. Use
    /// [`OptionAssertions::is_some_satisfying`] for capture mode, or the
    /// non-extracting [`OptionAssertions::is_some`] when the contained value is irrelevant.
    fn get_some(self) -> AssertThat<'t, T, Panic, R>;
}

impl<'t, T, R> OptionExtractAssertions<'t, T, R> for AssertThat<'t, Option<T>, Panic, R> {
    #[track_caller]
    fn get_some(self) -> AssertThat<'t, T, Panic, R> {
        self.track_assertion();

        if !self.actual().is_some() {
            self.failure(FailureKind::Variant)
                .actual(format_args!("None"))
                .relation("is not the expected variant")
                .expected(format_args!("Option::Some"))
                .raise();
        }

        self.map(|actual| match actual {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }
}

/// Non-extracting assertions for `Option` subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionAssertions<'t, T, M: Mode, R> {
    /// Asserts that the subject is `Some`.
    ///
    /// Non-extracting: the subject stays the full `Option`, so further assertions can be chained
    /// in any mode. Use [`OptionExtractAssertions::get_some`] to extract the contained value in
    /// panic mode, or [`OptionAssertions::is_some_satisfying`] to assert on it in any mode.
    fn is_some(self) -> Self;

    /// Asserts that the subject is `None`.
    ///
    /// Non-extracting: the subject stays the full `Option`, so further assertions can be chained
    /// in any mode.
    fn is_none(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject is `Some`, then runs `assertions` on its value.
    ///
    /// The closure receives an `AssertThat<T>` borrowing the contained value.
    fn is_some_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>);
}

impl<'t, T, M: Mode, R> OptionAssertions<'t, T, M, R> for AssertThat<'t, Option<T>, M, R> {
    #[track_caller]
    fn is_some(self) -> Self {
        self.track_assertion();

        if !self.actual().is_some() {
            self.failure(FailureKind::Variant)
                .actual(format_args!("None"))
                .relation("is not the expected variant")
                .expected(format_args!("Option::Some"))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_none(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();

        if !self.actual().is_none() {
            let actual = match self.actual() {
                Some(value) => self.render().variant(self.actual(), "Some", value),
                None => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Option::None"))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_some_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
    {
        self.track_assertion();

        if self.actual().is_some() {
            self.satisfies(|it| it.as_ref().unwrap(), assertions)
        } else {
            self.failure(FailureKind::Variant)
                .actual(format_args!("None"))
                .relation("is not the expected variant")
                .expected(format_args!("Option::Some"))
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
                AssertThat<'static, Option<i32>, Panic, NoRenderer>
                    => OptionAssertions<'static, i32, Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Option<i32>, Panic, NoRenderer>
                    => OptionExtractAssertions<'static, i32, NoRenderer>
            );
        }

        #[test]
        fn some_variant_is_rendered_from_its_leaf_value() {
            let failures = assert_that!(Some(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(OptionAssertions::is_none);

            assert_that!(ToHumanReadableText.render(&failures[0]))
                .contains("Some(")
                .contains(SENTINEL);
        }

        #[test]
        fn valueless_variant_needs_no_renderer_capability() {
            let failures = assert_that!(Option::<Secret>::None)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(OptionAssertions::is_some);

            assert_that!(ToHumanReadableText.render(&failures[0])).contains("Actual: None");
        }
    }

    mod is_some {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Option::<i32>::Some(42).must().be_some();
        }

        #[test]
        fn succeeds_when_some_and_retains_the_subject() {
            assert_that!(Option::<i32>::Some(42))
                .is_some()
                .is_equal_to(Some(42));
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::None)
                    .with_location(false)
                    .is_some()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `Option::<i32>::None`

                Actual: None

                is not the expected variant

                Expected: Option::Some
                -------- assertr --------
            "});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(Option::<i32>::None)
                .with_location(false)
                .capture(|it| it.is_some().is_none());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {"
                        -------- assertr --------
                        Expression: `Option::<i32>::None`

                        Actual: None

                        is not the expected variant

                        Expected: Option::Some
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }

    mod get_some {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Some(42).must().get_some().is_equal_to(42);
        }

        #[test]
        fn extracts_the_borrowed_inner_value() {
            let option = Some(String::from("value"));

            assert_that!(option).get_some().is_equal_to("value");

            // The option was only borrowed and remains usable.
            assert_that!(option).is_some();
        }

        #[test]
        fn extracts_the_owned_inner_value() {
            assert_that_owned!(Some(String::from("value")))
                .get_some()
                .is_equal_to("value");
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::None)
                    .with_location(false)
                    .get_some()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `Option::<i32>::None`

                Actual: None

                is not the expected variant

                Expected: Option::Some
                -------- assertr --------
            "});
        }
    }

    mod is_some_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Option::<i32>::Some(42).must().be_some_satisfying(|some| {
                some.is_equal_to(42);
            });
        }

        #[test]
        fn succeeds_when_some_and_assertions_pass() {
            assert_that!(Option::<i32>::Some(42)).is_some_satisfying(|some| {
                some.is_equal_to(42);
            });
        }

        #[test]
        fn hands_out_a_value_typed_assertion_supporting_type_specific_assertions() {
            assert_that!(Some(String::from("value"))).is_some_satisfying(|some| {
                some.contains("alu").starts_with("v");
            });
        }

        #[test]
        fn hands_out_a_value_typed_assertion_in_capture_mode() {
            let failures = assert_that!(Some(String::from("value")))
                .with_location(false)
                .capture(|it| {
                    it.is_some_satisfying(|some| {
                        some.contains("xyz");
                    })
                });

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Actual: "value"

                        does not contain

                        Expected: "xyz"
                        -------- assertr --------
                    "#});
                },
            ]);
        }

        #[test]
        fn captures_inner_failure_when_some_and_assertion_fails() {
            let failures = assert_that!(Option::<i32>::Some(42))
                .with_location(false)
                .capture(|it| {
                    it.is_some_satisfying(|some| {
                        some.is_greater_than(9000);
                    })
                });

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {"
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
        fn captures_variant_failure_when_none() {
            let failures = assert_that!(Option::<i32>::None)
                .with_location(false)
                .capture(|it| it.is_some_satisfying(|_| panic!("assertions should not run")));

            assert_that!(&failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {"
                        -------- assertr --------
                        Expression: `Option::<i32>::None`

                        Actual: None

                        is not the expected variant

                        Expected: Option::Some
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::None)
                    .with_location(false)
                    .is_some_satisfying(|_| panic!("assertions should not run"))
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `Option::<i32>::None`

                Actual: None

                is not the expected variant

                Expected: Option::Some
                -------- assertr --------
            "});
        }
    }

    mod is_none {
        use crate::prelude::*;
        use alloc::string::String;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Option::<i32>::None.must().be_none();
        }

        #[test]
        fn succeeds_when_none_and_retains_the_subject() {
            assert_that!(Option::<i32>::None)
                .is_none()
                .is_equal_to(None);
        }

        #[test]
        fn panics_when_some() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::Some(42))
                    .with_location(false)
                    .is_none()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `Option::<i32>::Some(42)`

                Actual: Some(
                    42,
                )

                is not the expected variant

                Expected: Option::None
                -------- assertr --------
            "});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(Option::<i32>::Some(42))
                .with_location(false)
                .capture(|it| it.is_none().is_some());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {"
                        -------- assertr --------
                        Expression: `Option::<i32>::Some(42)`

                        Actual: Some(
                            42,
                        )

                        is not the expected variant

                        Expected: Option::None
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }
}
