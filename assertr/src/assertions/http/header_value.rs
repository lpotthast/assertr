use crate::AssertThat;
use crate::mode::{Mode, Panic};
use crate::prelude::{BoolAssertions, PartialEqAssertions, PartialOrdAssertions};

#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HttpHeaderValueAssertions<'t, M: Mode> {
    fn is_empty(self) -> Self;

    #[cfg_attr(feature = "fluent", fluent_alias("not_be_empty"))]
    fn is_not_empty(self) -> Self;

    fn is_sensitive(self) -> Self;

    fn is_insensitive(self) -> Self;

    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M>);
}

impl<'t, M: Mode> HttpHeaderValueAssertions<'t, M>
    for AssertThat<'t, http::header::HeaderValue, M>
{
    fn is_empty(self) -> Self {
        self.derive(|it| it.len())
            .with_detail_message("Expected an empty header value.")
            .is_equal_to(0);
        self
    }

    fn is_not_empty(self) -> Self {
        self.derive(|it| it.len())
            .with_detail_message("Expected a non-empty header value.")
            .is_greater_than(0);
        self
    }

    fn is_sensitive(self) -> Self {
        self.derive(|it| it.is_sensitive())
            .with_detail_message("Expected a sensitive header value. You might have forgotten to call `set_sensitive(true)` on the header value.")
            .is_true();
        self
    }

    fn is_insensitive(self) -> Self {
        self.derive(|it| it.is_sensitive())
            .with_detail_message("Expected an insensitive header value. You might have forgotten to call `set_sensitive(false)` on the header value.")
            .is_false();
        self
    }

    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M>),
    {
        use crate::tracking::AssertionTracking;

        self.track_assertion();

        if self.actual().to_str().is_ok() {
            self.satisfies_ref(|hv| hv.to_str().expect("already checked"), assertions)
        } else {
            self.fail(format_args!(
                "Actual: {actual:?}\n\nis not valid ASCII\n",
                actual = self.actual()
            ));
            self
        }
    }
}

/// Data-extracting assertion for `HeaderValue`.
/// Only available in Panic mode, as the extracted `String` cannot be produced when the value is not ASCII.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HttpHeaderValueExtractAssertions<'t> {
    fn is_ascii(self) -> AssertThat<'t, String, Panic>;
}

impl<'t> HttpHeaderValueExtractAssertions<'t> for AssertThat<'t, http::header::HeaderValue, Panic> {
    fn is_ascii(self) -> AssertThat<'t, String, Panic> {
        use crate::prelude::ResultExtractAssertions;

        self.map(|it| it.borrowed().to_str().map(|it| it.to_owned()).into())
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    mod has_debug_value {
        use crate::prelude::*;
        use http::header::HeaderValue;

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
        fn succeeds_when_empty() {
            let actual = HeaderValue::from_str("").expect("valid header value");

            assert_that!(actual).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 0

                      Actual: 8

                    Details: [
                        Expected an empty header value.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_not_empty {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

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
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: 0

                    is not greater than

                    Expected: 0

                    Details: [
                        Expected a non-empty header value.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_sensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

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
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: true

                      Actual: false

                    Details: [
                        Expected a sensitive header value. You might have forgotten to call `set_sensitive(true)` on the header value.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_insensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

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
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: false

                      Actual: true

                    Details: [
                        Expected an insensitive header value. You might have forgotten to call `set_sensitive(false)` on the header value.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[tokio::test]
        async fn succeeds_when_constructed_from_visible_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_ascii().is_equal_to("http/1.1");
        }

        #[tokio::test]
        async fn succeeds_when_constructed_from_visible_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 34]).expect("valid header value");

            assert_that!(actual).is_ascii().is_equal_to(" !\"");
        }

        #[tokio::test]
        async fn panics_when_constructed_from_non_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("Ä").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: Err(
                        ToStrError {{
                            _priv: (),
                        }},
                    )

                    is not of expected variant: Result:Ok
                    -------- assertr --------
                "#});
        }

        #[tokio::test]
        async fn panics_when_constructed_from_non_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: Err(
                        ToStrError {{
                            _priv: (),
                        }},
                    )

                    is not of expected variant: Result:Ok
                    -------- assertr --------
                "#});
        }
    }

    mod is_ascii_satisfying {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

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

            let failures = assert_that!(actual)
                .with_capture()
                .with_location(false)
                .is_ascii_satisfying(|s| {
                    s.starts_with("ftp");
                })
                .capture_failures();
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn collects_failure_in_capture_mode_when_not_ascii() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            let failures = assert_that!(actual)
                .with_capture()
                .with_location(false)
                .is_ascii_satisfying(|s| {
                    s.starts_with("http");
                })
                .capture_failures();
            assert_that!(&failures).has_length(1);
            assert_that!(failures.first())
                .is_some()
                .map(|it| it.borrowed().as_str().into())
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
