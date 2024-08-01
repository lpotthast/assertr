use std::fmt::Debug;

use crate::{tracking::AssertionTracking, AssertThat, AssertrPartialEq, EqContext, Mode};

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

impl<'t, T, M: Mode> PartialEqAssertions<T> for AssertThat<'t, T, M> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        let mut ctx = EqContext {
            differences: Vec::new(),
        };

        if !AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            self.fail_with_arguments(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
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

        let mut ctx = EqContext {
            differences: Vec::new(),
        };

        if AssertrPartialEq::eq(actual, expected, Some(&mut ctx)) {
            self.fail_with_arguments(format_args!(
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
