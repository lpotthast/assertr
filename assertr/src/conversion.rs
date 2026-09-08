#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
use crate::{AssertThat, actual::Actual, mode::Mode};

#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
impl<'t, T: serde::Serialize, M: Mode, R> AssertThat<'t, T, M, R> {
    /// Serializes the borrowed view of the subject once, returning a JSON `Result` subject.
    ///
    /// Available in both assertion modes, without renderer or `Clone` requirements. Serialization
    /// errors are returned unchanged. This transformation preserves the renderer, budget, and
    /// chain state and does not count as an assertion. Use Result assertions to check or extract
    /// it.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// let value = [1, 2];
    /// assert_that!(value).as_json().get_ok().is_equal_to("[1,2]");
    /// let failures = assert_that!(value).capture(|it| {
    ///     it.as_json().is_ok_satisfying(|json| {
    ///         json.is_equal_to("[1,2]");
    ///     })
    /// });
    /// assert!(failures.is_empty());
    /// ```
    #[cfg(feature = "serde-json")]
    #[must_use]
    pub fn as_json(self) -> AssertThat<'t, Result<String, serde_json::Error>, M, R> {
        self.map(|it| Actual::Owned(serde_json::to_string(it.borrowed())))
    }

    /// Serializes the borrowed view of the subject once, returning a TOML `Result` subject.
    ///
    /// Available in both assertion modes, without renderer or `Clone` requirements. Serialization
    /// errors are returned unchanged. This transformation preserves the renderer, budget, and
    /// chain state and does not count as an assertion. Use Result assertions to check or extract
    /// it.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// #[derive(serde::Serialize)]
    /// struct Config { value: u32 }
    /// assert_that!(Config { value: 42 })
    ///     .as_toml().get_ok().is_equal_to("value = 42\n");
    /// let failures = assert_that!(Config { value: 42 }).capture(|it| {
    ///     it.as_toml().is_ok_satisfying(|toml| {
    ///         toml.is_equal_to("value = 42\n");
    ///     })
    /// });
    /// assert!(failures.is_empty());
    /// ```
    #[cfg(feature = "serde-toml")]
    #[must_use]
    pub fn as_toml(self) -> AssertThat<'t, Result<String, toml::ser::Error>, M, R> {
        self.map(|it| Actual::Owned(toml::to_string(it.borrowed())))
    }
}

#[cfg(all(test, any(feature = "serde-json", feature = "serde-toml")))]
mod tests {
    use crate::{
        Actual,
        prelude::*,
        renderer::RenderedBody,
        test_support::{NoRenderer, RedactingRenderer, assert_redacted},
    };
    use core::cell::Cell;

    struct Serialized<'a> {
        calls: &'a Cell<usize>,
        fail: bool,
    }
    impl serde::Serialize for Serialized<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            use serde::ser::{Error, SerializeStruct};
            self.calls.set(self.calls.get() + 1);
            if self.fail {
                return Err(S::Error::custom("private-serialization-value"));
            }
            let mut record = serializer.serialize_struct("Serialized", 1)?;
            record.serialize_field("value", &42)?;
            record.end()
        }
    }

    macro_rules! conversion_tests {
        ($module:ident, $method:ident) => {
            mod $module {
                use super::*;
                #[test]
                fn serializes_once_without_renderer_in_both_modes_and_ownership_forms() {
                    for fail in [false, true] {
                        let calls = Cell::new(0);
                        let subject = Serialized {
                            calls: &calls,
                            fail,
                        };
                        let converted = assert_that!(subject)
                            .with_renderer(NoRenderer)
                            .with_location(false)
                            .$method();
                        assert_eq!(converted.actual().is_err(), fail);
                        assert_eq!(converted.state.records.assertion_count(), 0);
                        assert!(matches!(converted.actual, Actual::Owned(_)));
                        let converted = assert_that_owned!(Serialized {
                            calls: &calls,
                            fail
                        })
                        .with_renderer(NoRenderer)
                        .with_location(false)
                        .$method();
                        assert_eq!(converted.actual().is_err(), fail);
                        assert_eq!(converted.state.records.assertion_count(), 0);
                        let converted = AssertThat::new_capturing(Actual::Borrowed(&subject))
                            .with_renderer(NoRenderer)
                            .with_location(false)
                            .$method();
                        assert_eq!(converted.actual().is_err(), fail);
                        assert_eq!(converted.state.records.assertion_count(), 0);
                        let converted = AssertThat::new_capturing(Actual::Owned(Serialized {
                            calls: &calls,
                            fail,
                        }))
                        .with_renderer(NoRenderer)
                        .with_location(false)
                        .$method();
                        assert_eq!(converted.actual().is_err(), fail);
                        assert_eq!(converted.state.records.assertion_count(), 0);
                        assert_eq!(calls.get(), 4);
                    }
                }
                #[test]
                fn result_assertions_keep_renderer_budget_and_chain_state() {
                    use indoc::formatdoc;

                    let calls = Cell::new(0);
                    let subject = Serialized {
                        calls: &calls,
                        fail: false,
                    };
                    let budget = RenderingBudget::builder().max_leaf_characters(3).build();
                    let converted = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .with_subject_name("serialized subject")
                        .with_rendering_budget(budget)
                        .$method()
                        .get_ok();
                    let failures = converted.capture(|it| it.is_equal_to("wrong"));
                    assert_that!(failures).contains_exactly_satisfying([
                        |element: AssertThat<AssertionFailure, Capture>| {
                            element.derive(|value| value).has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Subject: serialized subject
                        Expression: `subject`

                        Expected: <re... 7 more characters ...

                          Actual: <re... 7 more characters ...
                        -------- assertr --------
                    "});

                            element
                                .derive_owned(|value| value.subject_name.as_deref())
                                .is_equal_to(Some("serialized subject"));
                            element
                                .derive(|value| &value.actual.as_ref().unwrap().body)
                                .is_equal_to(RenderedBody::Text {
                                    text: "<re".into(),
                                    omitted_characters: 7,
                                });
                        },
                    ]);
                    let subject = Serialized {
                        calls: &calls,
                        fail: true,
                    };
                    let converted = assert_that!(subject).$method();
                    assert!(
                        converted
                            .actual()
                            .as_ref()
                            .unwrap_err()
                            .to_string()
                            .contains("private-serialization-value")
                    );
                    let failures = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .capture(|it| it.$method().is_ok_satisfying(|_| {}));
                    assert_that!(failures).contains_exactly_satisfying([
                        |element: AssertThat<AssertionFailure, Capture>| {
                            element.derive(|value| value).has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `subject`

                        Actual: Err(
                            <redacted>,
                        )

                        is not the expected variant

                        Expected: Result::Ok
                        -------- assertr --------
                    "});

                            assert_redacted(element.actual(), &["private-serialization-value"]);
                        },
                    ]);
                }
            }
        };
    }
    #[cfg(feature = "serde-json")]
    conversion_tests!(as_json, as_json);
    #[cfg(feature = "serde-toml")]
    conversion_tests!(as_toml, as_toml);
}
