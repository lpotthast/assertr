use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};
use alloc::{format, string::String};
use core::fmt::Debug;

/// Compares the complete `Debug` representation with verbatim expected text.
/// Formatting determines truth even when diagnostic rendering is disabled or budgeted.
/// Rejections retain the formatted operands so explanation never formats them again.
pub struct HasDebugString<E>(E);

impl<E> HasDebugString<E> {
    /// Owns the expected operand, which may itself be borrowed.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<T: Debug + ?Sized, E: AsRef<str>, R> Expectation<T, R> for HasDebugString<E> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = (String, &'a str)
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let actual = format!("{actual:?}");
        let expected = self.0.as_ref();
        if actual == expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}

impl<T: Debug + ?Sized, E: AsRef<str>, R: ValueRenderer<str>> ExpectationDiagnostics<T, R>
    for HasDebugString<E>
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => {
                let expected = self.0.as_ref();
                (
                    failure.relation("has the expected Debug representation"),
                    expected,
                )
            }
            Some((_, (actual, expected))) => {
                (failure.actual(render.value(actual.as_str())), expected)
            }
        };
        failure.expected(render.value(expected))
    }
}

/// Compares the complete `Debug` representation with the expected value's representation.
/// Formatting determines truth even when diagnostic rendering is disabled or budgeted.
/// Rejections retain the formatted operands so explanation never formats them again.
pub struct HasDebugValue<E>(E);

impl<E> HasDebugValue<E> {
    /// Owns the expected operand, which may itself be borrowed.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<T: Debug + ?Sized, E: Debug, R> Expectation<T, R> for HasDebugValue<E> {
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
        let actual = format!("{actual:?}");
        let expected = format!("{:?}", self.0);
        if actual == expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}

impl<T: Debug + ?Sized, E: Debug, R: ValueRenderer<str>> ExpectationDiagnostics<T, R>
    for HasDebugValue<E>
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => {
                let expected = format!("{:?}", self.0);
                (
                    failure.relation("has the expected Debug representation"),
                    expected,
                )
            }
            Some((_, (actual, expected))) => {
                (failure.actual(render.value(actual.as_str())), expected)
            }
        };
        failure.expected(render.value(expected.as_str()))
    }
}

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
        self.apply_assertion(HasDebugString::new(expected))
    }

    #[track_caller]
    fn has_debug_value(self, expected: impl Debug) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.apply_assertion(HasDebugValue::new(expected))
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer> => DebugAssertions<NoRenderer>
            );

            assert_trait_impl!(super::super::HasDebugString<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::HasDebugValue<i32> => crate::Expectation<i32, NoRenderer>);
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
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(5))
                .capture(|it| it.has_debug_string("123").has_debug_string("456"));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| rendered_text(value.actual.as_ref().unwrap()))
                        .is_equal_to("text:... 3 more characters ...");
                    element
                        .derive_owned(|value| rendered_text(value.expected.as_ref().unwrap()))
                        .is_equal_to("text:... 3 more characters ...");
                },
            ]);
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

        #[test]
        fn formats_each_operand_once_in_ordinary_matching_and_probe_execution() {
            use super::super::HasDebugValue;
            use crate::{AssertionContext, Expectation, test_support::NoRenderer};
            use core::{cell::Cell, fmt};

            struct Value<'a>(&'a Cell<usize>, &'a str);
            impl fmt::Debug for Value<'_> {
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
                let failures = assert_that!(actual).capture(|it| it.has_debug_value(&expected));
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((1, 1));
                assert_that!(failures.len()).is_equal_to(usize::from(expected_text != "actual"));
                let failures =
                    assert_that!(actual).capture(|it| it.matches(HasDebugValue::new(&expected)));
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((2, 2));
                assert_that!(failures.len()).is_equal_to(usize::from(expected_text != "actual"));
                let context = AssertionContext::new(
                    &NoRenderer,
                    RenderingBudget::default().with_max_leaf_characters(0),
                )
                .with_diagnostics(false);
                let observation = HasDebugValue::new(&expected)
                    .evaluate(&actual, &context)
                    .is_ok();
                assert_that!(observation).is_equal_to(expected_text == "actual");
                assert_that!((actual_calls.get(), expected_calls.get())).is_equal_to((3, 3));
            }
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
