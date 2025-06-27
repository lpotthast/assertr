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

    fn be_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
        Self: Sized,
    {
        self.is_equal_to(expected)
    }

    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn be_not_equal_to<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
        Self: Sized,
    {
        self.is_not_equal_to(expected)
    }
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
            "foo".must().be_equal_to("foo");
            "foo".to_string().must().be_equal_to("foo".to_string());
            "foo".to_string().must().be_equal_to("foo");
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| "foo".assert().with_location(false).is_equal_to("bar"))
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

            Foo {}.must().be_equal_to(Bar {});
        }
    }
}
