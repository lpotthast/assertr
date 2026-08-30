use crate::mode::{Mode, Panic};
use crate::prelude::{BoolAssertions, PartialEqAssertions, PartialOrdAssertions};
use crate::{AssertThat, ValueRenderer};
use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt::Write;
use indoc::writedoc;

/// Non-extracting assertions for [`http::HeaderValue`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HttpHeaderValueAssertions<'t, M: Mode, R = crate::DebugRenderer> {
    /// Asserts that the header value contains no bytes.
    fn is_empty(self) -> Self
    where
        R: ValueRenderer<usize> + Clone;

    /// Asserts that the header value contains at least one byte.
    fn is_not_empty(self) -> Self
    where
        R: ValueRenderer<usize> + Clone;

    /// Asserts that the header value is marked sensitive.
    fn is_sensitive(self) -> Self
    where
        R: ValueRenderer<bool> + Clone;

    /// Asserts that the header value is not marked sensitive.
    fn is_insensitive(self) -> Self
    where
        R: ValueRenderer<bool> + Clone;

    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value.
    ///
    /// This permits printable ASCII and horizontal tabs, but rejects opaque bytes. The subject
    /// stays the full `HeaderValue`, so further assertions can be chained in any mode. Use
    /// [`HttpHeaderValueExtractAssertions::get_ascii`] to extract a `String` in panic mode, or
    /// [`HttpHeaderValueAssertions::is_ascii_satisfying`] to assert on it in any mode.
    fn is_ascii(self) -> Self
    where
        R: ValueRenderer<http::header::HeaderValue>;

    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value, then runs
    /// additional assertions on the resulting string.
    ///
    /// The closure receives `AssertThat<&str>`. The projection target `str` is unsized, so the
    /// string assertion traits operate on the reference itself.
    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M, R>),
        R: ValueRenderer<http::header::HeaderValue> + Clone;
}

impl<'t, M: Mode, R> HttpHeaderValueAssertions<'t, M, R>
    for AssertThat<'t, http::header::HeaderValue, M, R>
{
    #[track_caller]
    fn is_empty(self) -> Self
    where
        R: ValueRenderer<usize> + Clone,
    {
        self.derive_owned(http::HeaderValue::len)
            .with_detail_message("Expected an empty header value.")
            .is_equal_to(0);
        self
    }

    #[track_caller]
    fn is_not_empty(self) -> Self
    where
        R: ValueRenderer<usize> + Clone,
    {
        self.derive_owned(http::HeaderValue::len)
            .with_detail_message("Expected a non-empty header value.")
            .is_greater_than(0);
        self
    }

    #[track_caller]
    fn is_sensitive(self) -> Self
    where
        R: ValueRenderer<bool> + Clone,
    {
        self.derive_owned(http::HeaderValue::is_sensitive)
            .with_detail_message("Expected a sensitive header value. You might have forgotten to call `set_sensitive(true)` on the header value.")
            .is_true();
        self
    }

    #[track_caller]
    fn is_insensitive(self) -> Self
    where
        R: ValueRenderer<bool> + Clone,
    {
        self.derive_owned(http::HeaderValue::is_sensitive)
            .with_detail_message("Expected an insensitive header value. You might have forgotten to call `set_sensitive(false)` on the header value.")
            .is_false();
        self
    }

    #[track_caller]
    fn is_ascii(self) -> Self
    where
        R: ValueRenderer<http::header::HeaderValue>,
    {
        self.track_assertion();

        if self.actual().to_str().is_err() {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:?}

                    is not valid ASCII
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M, R>),
        R: ValueRenderer<http::header::HeaderValue> + Clone,
    {
        self.track_assertion();

        if self.actual().to_str().is_ok() {
            self.satisfies_ref(|hv| hv.to_str().expect("already checked"), assertions)
        } else {
            let actual = self.render_value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:?}

                    is not valid ASCII
                "}
            });
            self
        }
    }
}

/// Panic-mode string extraction from [`HeaderValue`](http::HeaderValue) subjects.
///
/// A rejected value cannot produce the requested `String`.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HttpHeaderValueExtractAssertions<'t, R = crate::DebugRenderer> {
    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value, then
    /// extracts it as an owned `String`.
    ///
    /// Use [`HttpHeaderValueAssertions::is_ascii_satisfying`] for capture mode, or
    /// [`HttpHeaderValueAssertions::is_ascii`] when the text is irrelevant.
    fn get_ascii(self) -> AssertThat<'t, String, Panic, R>
    where
        R: ValueRenderer<http::header::HeaderValue>;
}

impl<'t, R> HttpHeaderValueExtractAssertions<'t, R>
    for AssertThat<'t, http::header::HeaderValue, Panic, R>
{
    #[track_caller]
    fn get_ascii(self) -> AssertThat<'t, String, Panic, R>
    where
        R: ValueRenderer<http::header::HeaderValue>,
    {
        self.is_ascii().map(|it| {
            it.borrowed()
                .to_str()
                .expect("already checked")
                .to_owned()
                .into()
        })
    }
}

#[cfg(test)]
mod tests {
    mod has_debug_value {
        use crate::prelude::*;
        use http::header::HeaderValue;

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().have_debug_value("http/1.1");
        }

        #[tokio::test]
        async fn succeeds_when_matching() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).has_debug_value("http/1.1");
        }
    }

    mod is_empty {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("").expect("valid header value");
            actual.must().be_empty();
        }

        #[test]
        fn succeeds_when_empty() {
            let actual = HeaderValue::from_str("").expect("valid header value");

            assert_that!(actual).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: 0

                      Actual: 8

                    Details: [
                        Expected an empty header value.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_not_empty {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().not_be_empty();
        }

        #[test]
        fn succeeds_when_not_empty() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_not_empty();
        }

        #[test]
        fn panics_when_empty() {
            let actual = HeaderValue::from_str("").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_not_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: 0

                    is not greater than

                    Expected: 0

                    Details: [
                        Expected a non-empty header value.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_sensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);
            actual.must().be_sensitive();
        }

        #[test]
        fn succeeds_when_sensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);

            assert_that!(actual).is_sensitive();
        }

        #[test]
        fn panics_when_insensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(false);

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_sensitive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: true

                      Actual: false

                    Details: [
                        Expected a sensitive header value. You might have forgotten to call `set_sensitive(true)` on the header value.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_insensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_insensitive();
        }

        #[test]
        fn not_sensitive_by_default() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_insensitive();
        }

        #[test]
        fn succeeds_when_insensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(false);

            assert_that!(actual).is_insensitive();
        }

        #[test]
        fn panics_when_sensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_insensitive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: false

                      Actual: true

                    Details: [
                        Expected an insensitive header value. You might have forgotten to call `set_sensitive(false)` on the header value.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_ascii();
        }

        #[test]
        fn succeeds_when_ascii_and_retains_the_subject() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_ascii().is_not_empty();
        }

        #[test]
        fn panics_when_not_ascii() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: " !\xff"

                    is not valid ASCII
                    -------- assertr --------
                "#});
        }

        #[test]
        fn works_in_capture_mode() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            let failures = assert_that!(actual)
                .with_location(false)
                .capture(|it| it.is_ascii().is_not_empty());

            assert_that!(&failures).has_length(1);
            assert_that!(&failures[0].description).contains("is not valid ASCII");
        }
    }

    mod get_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_static("http/1.1");
            actual.must().get_ascii().is_equal_to("http/1.1");
        }

        #[test]
        fn extracts_the_value_when_constructed_from_visible_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).get_ascii().is_equal_to("http/1.1");
        }

        #[test]
        fn extracts_the_value_when_constructed_from_visible_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 34]).expect("valid header value");

            assert_that!(actual).get_ascii().is_equal_to(" !\"");
        }

        #[test]
        fn panics_when_constructed_from_non_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("\u{c4}").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).get_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: "\xc3\x84"

                    is not valid ASCII
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_constructed_from_non_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).get_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: " !\xff"

                    is not valid ASCII
                    -------- assertr --------
                "#});
        }
    }

    mod is_ascii_satisfying {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_ascii_satisfying(|s| {
                s.starts_with("http");
            });
        }

        #[test]
        fn succeeds_when_ascii_and_assertions_pass() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_ascii_satisfying(|s| {
                s.starts_with("http");
            });
        }

        #[test]
        fn collects_failure_in_capture_mode_when_ascii_but_assertion_fails() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            let failures = assert_that!(actual).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("ftp");
                })
            });
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn collects_failure_in_capture_mode_when_not_ascii() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            let failures = assert_that!(actual).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("http");
                })
            });
            assert_that!(&failures).has_length(1);
            assert_that!(failures.first())
                .get_some()
                .map(|it| it.borrowed().description.as_str().into())
                .contains("is not valid ASCII");
        }

        #[test]
        fn panics_when_not_ascii_in_panic_mode() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .is_ascii_satisfying(|s| {
                        s.starts_with("http");
                    })
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: " !\xff"

                is not valid ASCII
                -------- assertr --------
            "#});
        }
    }
}
