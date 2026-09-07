use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};
use alloc::format;
use core::fmt::Debug;

/// Assertions for values implementing [`Debug`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait DebugAssertions<R = crate::DebugRenderer> {
    /// Asserts that the subject has the expected `Debug` representation.
    ///
    /// Compares the complete representation exactly, including quotes and escape sequences.
    /// The expected text is used verbatim and is never Debug-formatted again.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// assert_that!(42).has_debug_string("42");
    /// assert_that!("\n").has_debug_string(r#""\n""#);
    /// ```
    fn has_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject and `expected` have the same `Debug` representation.
    ///
    /// Compares the complete representation exactly, including quotes and escape sequences.
    /// Use [`has_debug_string`](Self::has_debug_string) for a preformatted expectation.
    fn has_debug_value(self, expected: impl Debug) -> Self
    where
        R: ValueRenderer<str>;
}

impl<T: Debug, M: Mode, R> DebugAssertions<R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn has_debug_string(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.track_assertion();

        let actual_string = format!("{:?}", self.actual());

        // Prevent debug formatting the expected value, as it is already in usable string form!
        // Debug formatting it would lead to double-escaping of already escaped characters. But if
        // the user has given a string, we must not mess with that input, as it should already
        // represent the exact debug output of actual.
        let expected_string = expected.as_ref();

        let actual_str = actual_string.as_str();
        let expected_str = expected_string;

        if actual_str != expected_str {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual_str))
                .expected(self.render().value(expected_str))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_debug_value(self, expected: impl Debug) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.track_assertion();

        let actual_string = format!("{:?}", self.actual());
        let expected_string = format!("{expected:?}");

        let actual_str = actual_string.as_str();
        let expected_str = expected_string.as_str();

        if actual_str != expected_str {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual_str))
                .expected(self.render().value(expected_str))
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer> => DebugAssertions<NoRenderer>
            );
        }
    }

    mod has_debug_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            42.must().have_debug_string("42");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(42), has_debug_string("foo"));
        }

        #[test]
        fn preserves_quotes_escapes_and_unicode_exactly() {
            for value in ["", "\"", "\\", "\n", "\r\n", "\t", "é🦀"] {
                let expected = format!("{value:?}");
                assert_that!(value).has_debug_string(&expected);
                assert_that!(value).has_debug_value(value);
                let without_quotes = &expected[1..expected.len() - 1];
                assert_that!(assert_that!(value).capture(|it| it.has_debug_string(without_quotes)))
                    .has_length(1);
            }
        }

        #[test]
        fn custom_debug_with_a_lone_quote_is_not_normalized() {
            struct Quoted;
            impl core::fmt::Debug for Quoted {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("\"value")
                }
            }
            assert_that!(Quoted).has_debug_string("\"value");
            assert_that!(assert_that!(Quoted).capture(|it| it.has_debug_string("value")))
                .has_length(1);
        }

        #[test]
        fn diagnostics_use_the_renderer_and_budget_without_changing_truth() {
            struct TextRenderer;
            impl ValueRenderer<str> for TextRenderer {
                fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "text:{value}")
                }
            }
            let failures = assert_that!(123)
                .with_renderer(TextRenderer)
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(5).build())
                .capture(|it| it.has_debug_string("123").has_debug_string("456"));
            assert_that!(failures).has_length(1);
            assert_eq!(
                rendered_text(failures[0].actual.as_ref().unwrap()),
                "text:... 3 more characters ..."
            );
            assert_eq!(
                rendered_text(failures[0].expected.as_ref().unwrap()),
                "text:... 3 more characters ..."
            );
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!(42).has_debug_string("42");
            assert_that!(42).has_debug_string("42");
            assert_that!(42).has_debug_string("42");
            assert_that!(42).has_debug_string("42");
        }

        #[test]
        fn succeeds_when_equal_on_static_string_containing_escaped_characters() {
            assert_that!("\n").has_debug_string(r#""\n""#);
        }

        #[test]
        fn succeeds_when_equal_on_struct_debug_string_containing_escaped_characters() {
            #[derive(Debug)]
            struct Data(#[expect(unused)] &'static str);

            assert_that!(Data("\n")).has_debug_string(r#"Data("\n")"#);
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| {
                assert_that!(42)
                    .with_location(false)
                    .has_debug_string("foo")
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `42`

                Expected: "foo"

                  Actual: "42"
                -------- assertr --------
            "#});
        }
    }

    mod has_debug_value {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            42.must().have_debug_value(42);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(42), has_debug_value(43));
        }

        mod with_number {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_equal_using_same_value() {
                assert_that!(42).has_debug_value(42);
                assert_that!(42).has_debug_value(42);
            }

            #[test]
            fn distinguishes_values_from_formatted_expectations() {
                let failures = assert_that!(42).capture(|it| it.has_debug_value("42"));
                assert_that!(failures).has_length(1);
                assert_that!(42).has_debug_string("42");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| assert_that!(42).with_location(false).has_debug_value(43))
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `42`

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
            fn distinguishes_a_newline_from_a_literal_escape_sequence() {
                assert_that_panic_by(|| {
                    assert_that!("\n")
                        .with_location(false)
                        .has_debug_value(r"\n")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"\n"`

                    Expected: "\"\\\\n\""

                      Actual: "\"\\n\""
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
                assert_that!(Person {
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
                assert_that!(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_value(&Person {
                    age: 42,
                    alive: true,
                });
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that!(Person {
                    age: 42,
                    alive: true,
                })
                .has_debug_string("Person { age: 42, alive: true }");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that_panic_by(|| {
                    assert_that!(Person {
                        age: 42,
                        alive: true,
                    })
                    .with_location(false)
                    .has_debug_string("foo")
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `Person {{ age: 42, alive: true, }}`

                    Expected: "foo"

                      Actual: "Person {{ age: 42, alive: true }}"
                    -------- assertr --------
                "#});
            }
        }
    }
}
