use crate::{tracking::AssertionTracking, AssertThat, Mode, AssertrEq, AssertEqTypeOf};
use std::fmt::Debug;

pub trait AssertrPartialEqAssertions<T: AssertrEq<E> + Debug, E: AssertEqTypeOf<T>> {
    fn is_equal_to_assertr(self, expected: E) -> Self;
    fn is_not_equal_to_assertr(self, expected: E) -> Self;
}

impl<'t, T: AssertrEq<E> + Debug, E: AssertEqTypeOf<T>, M: Mode> AssertrPartialEqAssertions<T, E> for AssertThat<'t, T, M> {
    #[track_caller]
    fn is_equal_to_assertr(self, expected: E) -> Self {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        if !actual.eq(&expected) {
            self.fail_with_arguments(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
        }
        self
    }

    #[track_caller]
    fn is_not_equal_to_assertr(self, expected: E) -> Self {
        self.track_assertion();

        let actual = self.actual();
        let expected = &expected;

        if actual.eq(&expected) {
            self.fail_with_arguments(format_args!(
                "Expected: {expected:#?}\n\n  Actual: {actual:#?}",
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod is_equal_to_assertr {
        use indoc::formatdoc;
        use crate::AssertEqTypeOf;
        use crate::prelude::*;

        #[derive(Debug)]
        struct Foo {
            pub id: i32,
        }

        #[derive(Debug)]
        struct FooAssertrEq {
            pub id: crate::Eq<i32>,
        }

        impl AssertEqTypeOf<Foo> for FooAssertrEq {}

        impl crate::AssertrEq<FooAssertrEq> for Foo {
            fn eq(&self, other: &FooAssertrEq) -> bool {
                match other.id {
                    crate::Eq::Any => true,
                    crate::Eq::Eq(v) => self.id == v,
                }
            }
        }

        #[test]
        fn succeeds_when_equal_ignoring_value() {
            assert_that(Foo { id: 1 }).is_equal_to_assertr(FooAssertrEq { id: crate::Eq::Any });
        }

        #[test]
        fn succeeds_when_equal_requiring_value() {
            assert_that(Foo { id: 1 }).is_equal_to_assertr(FooAssertrEq { id: crate::Eq::Eq(1) });
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(
                || assert_that(Foo { id: 1 })
                    .with_location(false)
                    .is_equal_to_assertr(FooAssertrEq { id: crate::Eq::Eq(42) })
            )
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: FooAssertrEq {{
                    id: Eq::Eq(42),
                }}

                  Actual: Foo {{
                    id: 1,
                }}
                -------- assertr --------
            "#});
        }
    }
}
