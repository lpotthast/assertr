use super::debug::{compare_text, explain_text};
use crate::{
    AssertThat, Mode,
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    renderer::DebugRenderer,
    renderer::ValueRenderer,
};
use alloc::{format, string::String};
use core::fmt::Display;

/// Compares the complete `Display` representation with the expected value's representation.
/// Formatting determines truth even when diagnostic rendering is disabled or budgeted.
/// Rejections retain the formatted operands so explanation never formats them again.
#[derive(Debug, Clone)]
pub struct HasDisplayValue<E>(E);

impl<E> HasDisplayValue<E> {
    /// Owns the expected operand, which may itself be borrowed.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<T: Display + ?Sized, E: Display, R: ValueRenderer<str>> Expectation<T, R>
    for HasDisplayValue<E>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = (String, String)
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        compare_text(format!("{actual}"), format!("{}", self.0))
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_text(
            rejected.map(|(_, texts)| texts),
            || format!("{}", self.0),
            "has the expected Display representation",
            failure,
            context,
        )
    }
}

/// Assertions over a subject's [`Display`] representation.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait DisplayAssertions<R = DebugRenderer> {
    /// Asserts that the subject and `expected` have the same `Display` representation.
    ///
    /// Compares the complete representation exactly, including quotes and escape sequences.
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

        #[test]
        fn formats_each_operand_once_in_ordinary_matching_and_probe_execution() {
            use super::super::HasDisplayValue;
            use crate::{expectation::AssertionContext, expectation::Expectation};
            use core::{cell::Cell, fmt};

            struct Value<'a>(&'a Cell<usize>, &'a str);
            impl fmt::Display for Value<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.set(self.0.get() + 1);
                    f.write_str(self.1)
                }
            }
            for expected_text in ["actual", "expected"] {
                let actual_calls = Cell::new(0);
                let expected_calls = Cell::new(0);
                let actual = Value(&actual_calls, "actual");
                let expected = Value(&expected_calls, expected_text);
                let failures = assert_that!(actual).capture(|it| it.has_display_value(&expected));
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((1, 1));
                assert_that!(failures.len()).is_equal_to(usize::from(expected_text != "actual"));
                let failures =
                    assert_that!(actual).capture(|it| it.matches(HasDisplayValue::new(&expected)));
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((2, 2));
                assert_that!(failures.len()).is_equal_to(usize::from(expected_text != "actual"));
                let context = AssertionContext::new(
                    &DebugRenderer,
                    RenderingBudget::default().with_max_leaf_characters(0),
                )
                .with_diagnostics(false);
                let observation = HasDisplayValue::new(&expected)
                    .evaluate(&actual, &context)
                    .is_ok();
                assert_that!(observation).is_equal_to(expected_text == "actual");
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((3, 3));
            }
        }

        #[test]
        fn quotes_and_escape_characters_are_significant() {
            use crate::prelude::*;
            for value in ["\"foo\"", "\"foo", "foo\"", "\n", "\\n", "é🦀"] {
                assert_that!(value).has_display_value(value);
                let failures = assert_that!(value).capture(|it| it.has_display_value("foo"));
                assert_that!(failures).has_length(1);
            }
            assert_that!(42).has_display_value("42");
        }

        mod with_number {
            use crate::prelude::*;
            use indoc::formatdoc;

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
                assert_that_panic_by(|| {
                    assert_that!(42)
                        .with_location(false)
                        .has_display_value("foo")
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

        mod with_string {
            use crate::prelude::*;

            #[test]
            fn succeeds_when_equal_using_string_representation() {
                assert_that!("foo:bar").has_display_value("foo:bar");
            }
        }

        mod with_custom_struct {
            use std::fmt::Display;

            use crate::prelude::*;

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
                assert_that!(Person {
                    age: 42,
                    alive: true,
                })
                .has_display_value("PERSON<AGE=42,ALIVE=true>");
            }
        }
    }
}
