use crate::{AssertThat, AssertrPartialEq, Mode, ValueRenderer, failure::FailureKind};

/// Equality and inequality assertions using [`AssertrPartialEq`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PartialEqAssertions<T, R> {
    /// Asserts that the subject equals `expected`.
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that the subject does not equal `expected`.
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;
}

impl<T, M: Mode, R> PartialEqAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = self.eq_context();

        if !AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            let mut failure = self
                .failure(FailureKind::Equality)
                .actual(self.render().value(actual))
                .expected(self.render().value(expected));
            if !ctx.differences.differences.is_empty() {
                failure = failure.fact("Differences", format_args!("{:#?}", ctx.differences));
            }
            failure.raise();
        }
        self
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = self.eq_context();

        if AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual))
                .relation("is equal to")
                .unexpected(self.render().value(expected))
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};

        struct Actual(u32);
        struct Expected(u32);

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PartialEqAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn heterogeneous_failures_render_both_types_with_the_active_renderer() {
            let failures = assert_that!(Actual(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_equal_to(Expected(2)));

            assert_that!(failures[0].description()).contains(SENTINEL);
        }
    }

    mod is_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo".must().be_equal_to("foo");
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!("foo").is_equal_to("foo");
            assert_that!("foo".to_string()).is_equal_to("foo".to_string());
            assert_that!("foo".to_string()).is_equal_to("foo");
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| assert_that!("foo").with_location(false).is_equal_to("bar"))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo"`

                    Expected: "bar"
                    
                      Actual: "foo"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn accepts_expected_being_of_different_type() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    true
                }
            }

            assert_that!(Foo {}).is_equal_to(Bar {});
        }
    }

    mod is_not_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo".must().not_be_equal_to("bar");
        }

        #[test]
        fn succeeds_when_not_equal() {
            assert_that!("foo").is_not_equal_to("bar");
        }

        #[test]
        fn panics_when_equal() {
            assert_that_panic_by(|| {
                assert_that!("foo")
                    .with_location(false)
                    .is_not_equal_to("foo")
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo"`

                    Actual: "foo"

                    is equal to

                    Unexpected: "foo"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn accepts_expected_being_of_different_type() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    false
                }
            }

            assert_that!(Foo {}).is_not_equal_to(Bar {});
        }
    }
}
