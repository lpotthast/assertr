use alloc::string::String;
use core::fmt::Write;
use indoc::writedoc;

use crate::{AssertThat, Mode, ValueRenderer};

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
        let expected = &true;
        if actual != expected {
            let actual = self.render_value(actual);
            let expected = self.render_value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
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
        let expected = &false;
        if actual != expected {
            let actual = self.render_value(actual);
            let expected = self.render_value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {

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
                    Expected: true

                      Actual: false
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
                    Expected: false

                      Actual: true
                    -------- assertr --------
                "});
        }
    }
}
