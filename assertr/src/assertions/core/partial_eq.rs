use alloc::format;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;

use crate::{AssertThat, AssertrPartialEq, EqContext, Mode, tracking::AssertionTracking};

pub trait PartialEqAssertions<T> {
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug;
}

impl<T, M: Mode> PartialEqAssertions<T> for AssertThat<'_, T, M> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = EqContext::default();

        if !AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            if !ctx.differences.differences.is_empty() {
                self.add_detail_message(format!("Differences: {:#?}", ctx.differences));
            }
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}
                    
                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = EqContext::default();

        if AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            if !ctx.differences.differences.is_empty() {
                self.add_detail_message(format!("Differences: {:#?}", ctx.differences));
            }
            self.fail(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
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
            assert_that("foo").is_equal_to("foo");
            assert_that("foo".to_owned()).is_equal_to("foo".to_owned());
            assert_that::<&String>(&"foo".to_owned()).is_equal_to(&"foo".to_owned());
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| assert_that("foo").with_location(false).is_equal_to("bar"))
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

            assert_that(Foo {}).is_equal_to(Bar {});
        }
    }
}
