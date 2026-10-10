use core::fmt::Display;

use super::debug::formatted_value_expectation;
use crate::{
    AssertThat, Mode,
    renderer::{DebugRenderer, ValueRenderer},
};

formatted_value_expectation! {
    /// Compares the complete `Display` representation with the expected value's representation.
    ///
    /// A string operand is compared verbatim, because a string displays as itself.
    pub struct HasDisplayValue;
    format Display, "{}";
    relation "has the expected Display representation";
}

/// Assertions over a subject's [`Display`] representation.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait DisplayAssertions<R = DebugRenderer> {
    /// Asserts that the subject and `expected` have the same `Display` representation.
    ///
    /// Compares the complete representation exactly, including quotes and escape sequences. A
    /// string `expected` displays as itself, so it is compared verbatim.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use core::net::Ipv4Addr;
    ///
    /// assert_that!(42).has_display_value(42);
    /// assert_that!(Ipv4Addr::LOCALHOST).has_display_value("127.0.0.1");
    /// ```
    fn has_display_value(self, expected: impl Display) -> Self
    where
        R: ValueRenderer<str>;
}

impl<T: Display, M: Mode, R> DisplayAssertions<R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn has_display_value(self, expected: impl Display) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(HasDisplayValue::new(expected))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            42.must().have_display_value(42);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer> => DisplayAssertions<NoRenderer>
            );
        }
    }

    mod has_display_value {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(42), has_display_value("foo"));
        }

        // `formatted_value_expectation!` is shared with `HasDebugValue`, whose tests pin how often
        // each operand is formatted.

        #[test]
        fn quotes_and_escape_characters_are_significant() {
            for value in ["\"foo\"", "\"foo", "foo\"", "\n", "\\n", "é🦀"] {
                assert_that!(value).has_display_value(value);
                let failures = assert_that!(value).capture(|it| it.has_display_value("foo"));
                assert_that!(failures).has_length(1);
            }
        }

        mod with_number {
            use indoc::formatdoc;

            use crate::prelude::*;

            #[test]
            fn succeeds_when_equal_using_same_value() {
                assert_that!(42).has_display_value(42);
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that!(42).has_display_value("42");
            }

            #[test]
            fn panics_when_not_equal() {
                assert_that!(|| {
                    assert_that!(42)
                        .with_location(false)
                        .has_display_value("foo")
                })
                .panics()
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

        mod with_string {
            use crate::prelude::*;

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that!("foo:bar").has_display_value("foo:bar");
            }
        }

        mod with_custom_struct {
            use core::fmt;

            use crate::prelude::*;

            struct Person {
                age: u32,
                alive: bool,
            }

            impl fmt::Display for Person {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_fmt(format_args!(
                        "PERSON<AGE={},ALIVE={}>",
                        self.age, self.alive
                    ))
                }
            }

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that!(Person {
                    age: 42,
                    alive: true,
                })
                .has_display_value("PERSON<AGE=42,ALIVE=true>");
            }
        }
    }
}
