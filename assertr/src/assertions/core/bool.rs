use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};

/// Assertions for boolean values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait BoolAssertions<R = crate::DebugRenderer> {
    /// Asserts that the subject is `true`.
    fn is_true(self) -> Self
    where
        R: ValueRenderer<bool>;

    /// Asserts that the subject is `false`.
    fn is_false(self) -> Self
    where
        R: ValueRenderer<bool>;
}

impl<M: Mode, R> BoolAssertions<R> for AssertThat<'_, bool, M, R> {
    #[track_caller]
    fn is_true(self) -> Self
    where
        R: ValueRenderer<bool>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !*actual {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual))
                .relation("is not true")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_false(self) -> Self
    where
        R: ValueRenderer<bool>,
    {
        self.track_assertion();
        let actual = self.actual();
        if *actual {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual))
                .relation("is not false")
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

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, bool, Panic, NoRenderer> => BoolAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!(false)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(BoolAssertions::is_true);

            assert_that!(failures[0].description()).contains(SENTINEL);
        }
    }

    mod is_true {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            true.must().be_true();
        }

        #[test]
        fn succeeds_when_true() {
            assert_that!(true).is_true();
        }

        #[test]
        fn panics_when_false() {
            assert_that_panic_by(|| assert_that!(false).with_location(false).is_true())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `false`

                    Actual: false

                    is not true
                    -------- assertr --------
                "});
        }
    }

    mod is_false {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            false.must().be_false();
        }

        #[test]
        fn succeeds_when_false() {
            assert_that!(false).is_false();
        }

        #[test]
        fn panics_when_true() {
            assert_that_panic_by(|| assert_that!(true).with_location(false).is_false())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `true`

                    Actual: true

                    is not false
                    -------- assertr --------
                "});
        }
    }
}
