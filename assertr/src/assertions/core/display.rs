use alloc::format;
use core::fmt::Display;

use crate::assertions::core::strip_quotation_marks;
use crate::{AssertThat, Mode, tracking::AssertionTracking};

pub trait DisplayAssertions {
    fn has_display_value(self, expected: impl Display) -> Self;
}

impl<T: Display, M: Mode> DisplayAssertions for AssertThat<'_, T, M> {
    #[track_caller]
    fn has_display_value(self, expected: impl Display) -> Self {
        self.track_assertion();

        let actual_string = format!("{}", self.actual());
        let expected_string = format!("{}", expected);

        let actual_str = strip_quotation_marks(actual_string.as_str());
        let expected_str = strip_quotation_marks(expected_string.as_str());

        if actual_str != expected_str {
            self.fail(format_args!(
                "Expected: {expected_str:?}\n\n  Actual: {actual_str:?}\n"
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod has_display_value {

        mod with_number {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_equal_using_same_value() {
                assert_that(42).has_display_value(42);
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(42).has_display_value("42");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that(42)
                        .with_location(false)
                        .has_display_value("foo")
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
                assert_that("foo:bar").has_display_value("foo:bar");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that("foo:bar")
                        .with_location(false)
                        .has_display_value("foo:baz")
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
            use std::fmt::Display;

            use crate::prelude::*;
            use indoc::formatdoc;

            #[allow(dead_code)] // Allow fields to never be read.
            struct Person {
                age: u32,
                alive: bool,
            }

            impl Display for Person {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_fmt(format_args!(
                        "PERSON<AGE={},ALIVE={}>",
                        self.age, self.alive
                    ))
                }
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(Person {
                    age: 42,
                    alive: true,
                })
                .has_display_value("PERSON<AGE=42,ALIVE=true>");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that(Person {
                        age: 42,
                        alive: true,
                    })
                    .with_location(false)
                    .has_display_value("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "foo"

                      Actual: "PERSON<AGE=42,ALIVE=true>"
                    -------- assertr --------
                "#});
            }
        }
    }
}
