use alloc::string::String;
use core::fmt::Write;

use indoc::writedoc;

use crate::{AssertThat, Mode, ValueRenderer};

/// A Rust pattern together with the predicate and source text needed to assert that it matches.
///
/// Create patterns with [`pattern!`](crate::pattern) rather than constructing this type directly.
pub struct Pattern<P> {
    description: &'static str,
    predicate: P,
}

impl<P> Pattern<P> {
    /// Creates the runtime representation emitted by [`pattern!`](crate::pattern).
    pub(crate) fn new(description: &'static str, predicate: P) -> Self {
        Self {
            description,
            predicate,
        }
    }
}

/// Creates a pattern for use with [`PatternAssertions`].
///
/// The subject is matched by reference, so ordinary patterns benefit from Rust's match
/// ergonomics and do not consume the assertion subject. Pattern guards are supported.
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug)]
/// enum Error {
///     Invalid(&'static str),
/// }
///
/// let result: Result<(), Error> = Err(Error::Invalid("invalid UTF-8"));
/// assert_that!(result)
///     .is_matching(pattern!(Err(Error::Invalid(reason)) if reason.contains("UTF-8")));
/// ```
#[macro_export]
macro_rules! pattern {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        $crate::__private::new_pattern(
            ::core::stringify!($pattern $(if $guard)?),
            |actual: &_| ::core::matches!(actual, $pattern $(if $guard)?),
        )
    };
}

/// Assertions based on arbitrary Rust patterns.
///
/// Failure diagnostics include the pattern's source text and the subject rendered through the
/// active [`ValueRenderer`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PatternAssertions<T, R> {
    /// Asserts that the subject matches `pattern`.
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that the subject does not match `pattern`.
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>;
}

impl<T, M: Mode, R> PatternAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.track_assertion();

        let Pattern {
            description,
            predicate,
        } = pattern;
        if !predicate(self.actual()) {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected pattern: {description}

                              Actual: {actual:#?}
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_not_matching<P>(self, pattern: Pattern<P>) -> Self
    where
        P: FnOnce(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.track_assertion();

        let Pattern {
            description,
            predicate,
        } = pattern;
        if predicate(self.actual()) {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Unexpected pattern: {description}

                              Actual: {actual:#?}
                "}
            });
        }

        self
    }
}

#[cfg(test)]
mod tests {
    mod is_matching {
        use core::fmt;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Some(42).must().be_matching(pattern!(Some(42)));
        }

        #[derive(Debug)]
        enum TestError {
            MissingQueryParams,
            MissingTokenQueryParam,
            InvalidToken { reason: String },
        }

        #[test]
        fn succeeds_when_pattern_matches() {
            assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams))
                .is_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn supports_patterns_with_guards() {
            assert_that!(Result::<(), TestError>::Err(TestError::InvalidToken {
                reason: "invalid UTF-8".to_owned(),
            }))
            .is_matching(pattern!(
                Err(TestError::InvalidToken { reason }) if reason.contains("UTF-8")
            ));
        }

        #[test]
        fn supports_or_patterns() {
            assert_that!(Result::<(), TestError>::Err(
                TestError::MissingTokenQueryParam
            ))
            .is_matching(pattern!(Err(
                TestError::MissingQueryParams | TestError::MissingTokenQueryParam
            )));
        }

        #[test]
        fn borrows_the_actual_value_while_matching() {
            let actual = Some(String::from("value"));

            assert_that!(&actual).is_matching(pattern!(Some(value) if value == "value"));

            assert_that!(actual).get_some().is_equal_to("value");
        }

        #[test]
        fn evaluates_the_actual_expression_once() {
            let mut evaluations = 0;

            assert_that!({
                evaluations += 1;
                Some(42)
            })
            .is_matching(pattern!(Some(42)));

            assert_that!(evaluations).is_equal_to(1);
        }

        #[test]
        fn panics_with_the_expected_pattern_and_rendered_actual_value() {
            assert_that_panic_by(|| {
                assert_that!(Result::<(), TestError>::Err(
                    TestError::MissingTokenQueryParam
                ))
                .with_location(false)
                .is_matching(pattern!(Err(TestError::MissingQueryParams)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Result::<(), TestError>::Err(TestError::MissingTokenQueryParam)`

                Expected pattern: Err(TestError::MissingQueryParams)

                          Actual: Err(
                    MissingTokenQueryParam,
                )
                -------- assertr --------
            "});
        }

        #[test]
        fn works_in_capture_mode() {
            let failures = assert_that!(Some(42))
                .with_location(false)
                .capture(|it| it.is_matching(pattern!(None)));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Some(42)`

                        Expected pattern: None

                                  Actual: Some(
                            42,
                        )
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn uses_the_active_renderer() {
            struct Opaque(u8);

            assert_that_panic_by(|| {
                assert_that!(Opaque(1))
                    .with_debug_format(|actual: &Opaque, f: &mut fmt::Formatter<'_>| {
                        write!(f, "Opaque({})", actual.0)
                    })
                    .with_location(false)
                    .is_matching(pattern!(Opaque(2)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Opaque(1)`

                Expected pattern: Opaque(2)

                          Actual: Opaque(1)
                -------- assertr --------
            "});
        }
    }

    mod is_not_matching {
        use core::fmt;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Some(42).must().not_be_matching(pattern!(Some(43)));
        }

        #[derive(Debug)]
        enum TestError {
            MissingQueryParams,
            MissingTokenQueryParam,
            InvalidToken { reason: String },
        }

        #[test]
        fn succeeds_when_pattern_does_not_match() {
            assert_that!(Result::<(), TestError>::Err(
                TestError::MissingTokenQueryParam
            ))
            .is_not_matching(pattern!(Err(TestError::MissingQueryParams)));
        }

        #[test]
        fn supports_patterns_with_guards() {
            assert_that!(Result::<(), TestError>::Err(TestError::InvalidToken {
                reason: "expired token".to_owned(),
            }))
            .is_not_matching(pattern!(
                Err(TestError::InvalidToken { reason }) if reason.contains("UTF-8")
            ));
        }

        #[test]
        fn panics_with_the_unexpected_pattern_and_rendered_actual_value() {
            assert_that_panic_by(|| {
                assert_that!(Result::<(), TestError>::Err(TestError::MissingQueryParams))
                    .with_location(false)
                    .is_not_matching(pattern!(Err(TestError::MissingQueryParams)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Result::<(), TestError>::Err(TestError::MissingQueryParams)`

                Unexpected pattern: Err(TestError::MissingQueryParams)

                          Actual: Err(
                    MissingQueryParams,
                )
                -------- assertr --------
            "});
        }

        #[test]
        fn works_in_capture_mode() {
            let failures = assert_that!(Some(42))
                .with_location(false)
                .capture(|it| it.is_not_matching(pattern!(Some(42))));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Some(42)`

                        Unexpected pattern: Some(42)

                                  Actual: Some(
                            42,
                        )
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn uses_the_active_renderer() {
            struct Opaque(u8);

            assert_that_panic_by(|| {
                assert_that!(Opaque(1))
                    .with_debug_format(|actual: &Opaque, f: &mut fmt::Formatter<'_>| {
                        write!(f, "Opaque({})", actual.0)
                    })
                    .with_location(false)
                    .is_not_matching(pattern!(Opaque(1)));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Opaque(1)`

                Unexpected pattern: Opaque(1)

                          Actual: Opaque(1)
                -------- assertr --------
            "});
        }
    }
}
