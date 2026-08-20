use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use crate::{AssertThat, AssertionRenderer, AssertrPartialEq, Mode, tracking::AssertionTracking};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PartialEqAssertions<T, R> {
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<T> + AssertionRenderer<E>;

    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<T> + AssertionRenderer<E>;
}

impl<T, M: Mode, R> PartialEqAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<T> + AssertionRenderer<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = self.eq_context();

        if !AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            let mut details = Vec::new();
            if !ctx.differences.differences.is_empty() {
                details.push(format!("Differences: {:#?}", ctx.differences));
            }
            let actual = self.render_value(actual);
            let expected = self.render_value(expected);
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<T> + AssertionRenderer<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = self.eq_context();

        if AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            let mut details = Vec::new();
            if !ctx.differences.differences.is_empty() {
                details.push(format!("Differences: {:#?}", ctx.differences));
            }
            let actual = self.render_value(actual);
            let expected = self.render_value(expected);
            self.fail_with_details(details, |w: &mut String| {
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
    mod is_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

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
}
