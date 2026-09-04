use crate::failure::FailureKind;
use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use core::borrow::Borrow;
use jiff::Zoned;
use jiff::tz::TimeZone;

/// Assertions for [`Zoned`] date-times.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ZonedAssertions<R = crate::DebugRenderer> {
    /// Asserts that the subject uses the same time-zone rules as `expected`.
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        R: ValueRenderer<Zoned>;

    /// Asserts that the subject has an IANA time-zone name equal to `expected`.
    ///
    /// A subject using an unnamed fixed-offset or POSIX time zone fails this assertion.
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned>;
}

/// The label of the fact naming the subject's time zone.
const ACTUAL_TIME_ZONE: &str = "Actual time zone";

impl<M: Mode, R> ZonedAssertions<R> for AssertThat<'_, Zoned, M, R> {
    #[track_caller]
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        R: ValueRenderer<Zoned>,
    {
        self.track_assertion();

        let expected = expected.borrow();
        let actual = self.actual().time_zone();
        if actual != expected {
            let expected = time_zone_name(expected);
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("is not in time zone")
                .expected(format_args!("{expected}"))
                .fact(ACTUAL_TIME_ZONE, time_zone_name(actual))
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned>,
    {
        self.track_assertion();

        let expected = expected.as_ref();
        let actual = self.actual().time_zone();
        if actual.iana_name() != Some(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .relation("is not in time zone")
                .expected(format_args!("{expected}"))
                .fact(ACTUAL_TIME_ZONE, time_zone_name(actual))
                .raise();
        }
        self
    }
}

/// The IANA name of a time zone, or its `Debug` form for a fixed-offset or POSIX time zone.
fn time_zone_name(time_zone: &TimeZone) -> String {
    time_zone
        .iana_name()
        .map_or_else(|| format!("{time_zone:?}"), ToOwned::to_owned)
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

            assert_that!(TextReporter.report(&failures[0])).contains(SENTINEL);
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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00-04:00[America/New_York]

                    is not in time zone

                    Expected: Europe/Berlin

                    Details:
                      - Actual time zone: America/New_York
                    -------- assertr --------
                "});
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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00+05:00[+05:00]

                    is not in time zone

                    Expected: Europe/Berlin

                    Details:
                      - Actual time zone: TimeZone(05:00:00)
                    -------- assertr --------
                "});
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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00-04:00[America/New_York]

                    is not in time zone

                    Expected: Europe/Berlin

                    Details:
                      - Actual time zone: America/New_York
                    -------- assertr --------
                "});
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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `zdt`

                    Actual: 2024-06-19T15:22:00+05:00[+05:00]

                    is not in time zone

                    Expected: Europe/Berlin

                    Details:
                      - Actual time zone: TimeZone(05:00:00)
                    -------- assertr --------
                "});
        }
    }
}
