use alloc::format;
use core::fmt::Debug;

use crate::{tracking::AssertionTracking, AssertThat, Mode};

/// Assertions for values implementing [Debug].
pub trait DebugAssertions {
    /// Test that actual and expected have the same `Debug` representation.
    fn has_debug_value(self, expected: impl Debug) -> Self;
}

impl<'t, T: Debug, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn has_debug_value(self, expected: impl Debug) -> Self {
        self.track_assertion();

        let actual_string = format!("{:?}", self.actual());
        let expected_string = format!("{:?}", expected);

        let actual_str = strip_quotation_marks(actual_string.as_str());
        let expected_str = strip_quotation_marks(expected_string.as_str());

        if actual_str != expected_str {
            self.fail_with_arguments(format_args!(
                "Expected: {expected_str:?}\n\n  Actual: {actual_str:?}"
            ));
        }
        self
    }
}

fn strip_quotation_marks(mut str: &str) -> &str {
    if str.starts_with('"') {
        str = str.strip_prefix('"').unwrap();
    }
    if str.ends_with('"') {
        str = str.strip_suffix('"').unwrap();
    }
    str
}

#[cfg(test)]
mod tests {

    mod has_debug_value {

        mod with_number {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_equal_using_same_value() {
                assert_that(42).has_debug_value(42);
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(42).has_debug_value("42");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that(42).with_location(false).has_debug_value("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "foo"

                      Actual: "42"
                    -------- assertr --------
                "#});
            }
        }

        mod with_string {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that("foo:bar").has_debug_value("foo:bar");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that("foo:bar")
                        .with_location(false)
                        .has_debug_value("foo:baz")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "foo:baz"

                      Actual: "foo:bar"
                    -------- assertr --------
                "#});
            }
        }

        mod with_custom_struct {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[derive(Debug)]
            #[expect(dead_code)] // Expect fields to never be read.
            struct Person {
                age: u32,
                alive: bool,
            }

            #[test]
            fn succeeds_when_equal_using_other_value() {
                assert_that(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_value(Person {
                    age: 42,
                    alive: true,
                });
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_value("Person { age: 42, alive: true }");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that(Person {
                        age: 42,
                        alive: true,
                    })
                    .with_location(false)
                    .has_debug_value("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "foo"

                      Actual: "Person {{ age: 42, alive: true }}"
                    -------- assertr --------
                "#});
            }
        }
    }
}
