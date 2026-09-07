use crate::failure::{Fact, FailureKind};
use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use core::borrow::Borrow;
use jiff::Zoned;
use jiff::tz::TimeZone;

/// Assertions for [`Zoned`] date-times.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ZonedAssertions<R = crate::DebugRenderer> {
    /// Asserts that the subject uses the same time-zone rules as `expected`.
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone>;

    /// Asserts that the subject has an IANA time-zone name equal to `expected`.
    ///
    /// A subject using an unnamed fixed-offset or POSIX time zone fails this assertion.
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<str>;
}

/// The label of the fact naming the subject's time zone.
const ACTUAL_TIME_ZONE: &str = "Actual time zone";

impl<M: Mode, R> ZonedAssertions<R> for AssertThat<'_, Zoned, M, R> {
    #[track_caller]
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone>,
    {
        self.track_assertion();

        let expected = expected.borrow();
        let actual = self.actual().time_zone();
        if actual != expected {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("is not in time zone")
                .expected(self.render().value(expected))
                .fact(Fact::labelled(
                    ACTUAL_TIME_ZONE,
                    self.render().value(actual),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<str>,
    {
        self.track_assertion();

        let expected = expected.as_ref();
        let actual = self.actual().time_zone();
        if actual.iana_name() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("is not in time zone")
                .expected(self.render().value(expected))
                .fact(Fact::labelled(
                    ACTUAL_TIME_ZONE,
                    self.render().value(actual),
                ))
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use jiff::Zoned;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Zoned, Panic, NoRenderer> => ZonedAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let zoned: Zoned = "2024-06-19 15:22[America/New_York]"
                .parse()
                .expect("valid zoned datetime");
            let failures = assert_that!(zoned)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_in_time_zone_named("Europe/Berlin"));

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    mod is_in_time_zone {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::Zoned;
        use jiff::tz::{self, TimeZone};

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("America/New_York").expect("valid");
            zdt.must().be_in_time_zone(tz);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("Europe/Berlin").expect("valid");
            assert_caller_location!(assert_that!(zdt), is_in_time_zone(tz));
        }

        #[test]
        fn renders_original_zone_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            for subject in [
                "2024-06-19 15:22[America/New_York]"
                    .parse::<Zoned>()
                    .unwrap(),
                jiff::civil::date(2024, 6, 19)
                    .at(15, 22, 0, 0)
                    .to_zoned(TimeZone::fixed(tz::offset(5)))
                    .unwrap(),
            ] {
                let operand = TimeZone::get("Europe/Berlin").unwrap();
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.is_in_time_zone(&operand));
                let actual_zone = subject.time_zone();
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom({subject:?})

                    is not in time zone

                    Expected: custom(TimeZone(TZif("Europe/Berlin")))

                    Details:
                      - Actual time zone: custom({actual_zone:?})
                    -------- assertr --------
                "#});
                        assert_custom_value(element.actual().expected.as_ref().unwrap(), &operand);
                        assert_custom_value(&element.actual().facts[0].value, subject.time_zone());
                    },
                ]);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.is_in_time_zone(&operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    is not in time zone

                    Expected: <redacted>

                    Details:
                      - Actual time zone: <redacted>
                    -------- assertr --------
                "});

                        assert_redacted(
                            element.actual(),
                            &["Europe/Berlin", "America/New_York", "05:00"],
                        );
                    },
                ]);
            }
        }

        #[test]
        fn succeeds_when_matches() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("America/New_York").expect("valid");
            assert_that!(zdt).is_in_time_zone(tz);
        }

        #[test]
        fn panics_when_in_different_time_zone() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("Europe/Berlin").expect("valid");

            assert_that_panic_by(|| {
                assert_that!(zdt).with_location(false).is_in_time_zone(tz);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00-04:00[America/New_York]

                    is not in time zone

                    Expected: TimeZone(
                        TZif(
                            "Europe/Berlin",
                        ),
                    )

                    Details:
                      - Actual time zone: TimeZone(
                            TZif(
                                "America/New_York",
                            ),
                        )
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_with_actual_zone_when_actual_zone_is_unnamed() {
            let zdt = jiff::civil::date(2024, 6, 19)
                .at(15, 22, 0, 0)
                .to_zoned(TimeZone::fixed(tz::offset(5)))
                .expect("valid");
            let tz = TimeZone::get("Europe/Berlin").expect("valid");

            assert_that_panic_by(|| {
                assert_that!(zdt).with_location(false).is_in_time_zone(tz);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00+05:00[+05:00]

                    is not in time zone

                    Expected: TimeZone(
                        TZif(
                            "Europe/Berlin",
                        ),
                    )

                    Details:
                      - Actual time zone: TimeZone(
                            05:00:00,
                        )
                    -------- assertr --------
                "#});
        }
    }

    mod is_in_time_zone_named {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::Zoned;
        use jiff::tz::{self, TimeZone};

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            zdt.must().be_in_time_zone_named("America/New_York");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            assert_caller_location!(assert_that!(zdt), is_in_time_zone_named("Europe/Berlin"));
        }

        #[test]
        fn renders_original_zone_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            for subject in [
                "2024-06-19 15:22[America/New_York]"
                    .parse::<Zoned>()
                    .unwrap(),
                jiff::civil::date(2024, 6, 19)
                    .at(15, 22, 0, 0)
                    .to_zoned(TimeZone::fixed(tz::offset(5)))
                    .unwrap(),
            ] {
                let operand = "Europe/Berlin";
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| it.is_in_time_zone_named(operand));
                let actual_zone = subject.time_zone();
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: custom({subject:?})

                    is not in time zone

                    Expected: custom("Europe/Berlin")

                    Details:
                      - Actual time zone: custom({actual_zone:?})
                    -------- assertr --------
                "#});
                        assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                        assert_custom_value(&element.actual().facts[0].value, subject.time_zone());
                    },
                ]);
                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| it.is_in_time_zone_named(operand));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subject`

                    Actual: <redacted>

                    is not in time zone

                    Expected: <redacted>

                    Details:
                      - Actual time zone: <redacted>
                    -------- assertr --------
                "});

                        assert_redacted(
                            element.actual(),
                            &["Europe/Berlin", "America/New_York", "05:00"],
                        );
                    },
                ]);
            }
        }

        #[test]
        fn succeeds_when_matches() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            assert_that!(zdt).is_in_time_zone_named("America/New_York");
        }

        #[test]
        fn panics_when_in_different_time_zone() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            assert_that_panic_by(|| {
                assert_that!(zdt)
                    .with_location(false)
                    .is_in_time_zone_named("Europe/Berlin");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00-04:00[America/New_York]

                    is not in time zone

                    Expected: "Europe/Berlin"

                    Details:
                      - Actual time zone: TimeZone(
                            TZif(
                                "America/New_York",
                            ),
                        )
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_with_actual_zone_when_actual_zone_is_unnamed() {
            let zdt = jiff::civil::date(2024, 6, 19)
                .at(15, 22, 0, 0)
                .to_zoned(TimeZone::fixed(tz::offset(5)))
                .expect("valid");

            assert_that_panic_by(|| {
                assert_that!(zdt)
                    .with_location(false)
                    .is_in_time_zone_named("Europe/Berlin");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00+05:00[+05:00]

                    is not in time zone

                    Expected: "Europe/Berlin"

                    Details:
                      - Actual time zone: TimeZone(
                            05:00:00,
                        )
                    -------- assertr --------
                "#});
        }
    }
}
