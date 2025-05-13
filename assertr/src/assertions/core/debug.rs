use crate::assertions::core::strip_quotation_marks;
use crate::{AssertThat, Mode, tracking::AssertionTracking};
use alloc::format;
use core::fmt::Debug;

/// Assertions for values implementing [Debug].
pub trait DebugAssertions {
    /// Test that actual has the `expected` `Debug` representation.
    fn has_debug_string(self, expected: impl AsRef<str>) -> Self;

    /// Test that actual and expected have the same `Debug` representation.
    fn has_debug_value(self, expected: impl Debug) -> Self;
}

impl<T: Debug, M: Mode> DebugAssertions for AssertThat<'_, T, M> {
    #[track_caller]
    fn has_debug_string(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();

        let actual_string = format!("{:?}", self.actual());

        // Prevent debug formatting the expected value, as it is already in usable string form!
        // Debug formatting it would lead to double-escaping of already escaped characters.
        // But if the user has given a string, we must not mess with that input, as it should
        // already represent the exact debug output of actual.
        let expected_string = expected.as_ref();

        let actual_str = strip_quotation_marks(actual_string.as_str());
        let expected_str = strip_quotation_marks(expected_string);

        if actual_str != expected_str {
            self.fail(format_args!(
                "Expected: {expected_str:?}\n\n  Actual: {actual_str:?}\n"
            ));
        }
        self
    }

    #[track_caller]
    fn has_debug_value(self, expected: impl Debug) -> Self {
        self.track_assertion();

        let actual_string = format!("{:?}", self.actual());
        let expected_string = format!("{expected:?}");

        let actual_str = strip_quotation_marks(actual_string.as_str());
        let expected_str = strip_quotation_marks(expected_string.as_ref());

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
    mod has_debug_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_equal() {
            assert_that(42).has_debug_string("42");
            assert_that(42).has_debug_string(&"42");
            assert_that(42).has_debug_string("42".to_string());
            assert_that(42).has_debug_string(&"42".to_string());
        }

        #[test]
        fn succeeds_when_equal_on_static_string_containing_escaped_characters() {
            assert_that("\n").has_debug_string(r#"\n"#);
        }

        #[test]
        fn succeeds_when_equal_on_struct_debug_string_containing_escaped_characters() {
            #[derive(Debug)]
            struct Data(#[expect(unused)] &'static str);

            assert_that(Data("\n")).has_debug_string(r#"Data("\n")"#);
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| assert_that(42).with_location(false).has_debug_string("foo"))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: "foo"

                  Actual: "42"
                -------- assertr --------
            "#});
        }
    }

    mod has_debug_value {
        mod with_number {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_equal_using_same_value() {
                assert_that(42).has_debug_value(42);
                assert_that(42).has_debug_value(&42);
            }

            // Although `has_debug_string` should be used instead!
            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(42).has_debug_value("42");
                assert_that(42).has_debug_value(&"42");
                assert_that(42).has_debug_value("42".to_string());
                assert_that(42).has_debug_value(&"42".to_string());
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| assert_that(42).with_location(false).has_debug_value(43))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "43"

                      Actual: "42"
                    -------- assertr --------
                "#});
            }
        }

        mod with_string {
            use crate::prelude::*;
            use indoc::formatdoc;

            // That's why we also have `has_debug_string`.
            #[test]
            fn panics_when_trying_to_compare_with_string_containing_escaped_characters_although_user_would_expect_this_to_be_successful()
             {
                assert_that_panic_by(|| {
                    assert_that("\n")
                        .with_location(false)
                        .has_debug_value(r#"\n"#)
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: "\\\\n"

                      Actual: "\\n"
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
            fn succeeds_when_equal_using_value() {
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
            fn succeeds_when_equal_using_borrowed_value() {
                let expected = Person {
                    age: 42,
                    alive: true,
                };
                assert_that(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_value(&expected);
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_string("Person { age: 42, alive: true }");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that(Person {
                        age: 42,
                        alive: true,
                    })
                    .with_location(false)
                    .has_debug_string("foo")
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
