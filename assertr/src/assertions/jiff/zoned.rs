use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use alloc::borrow::ToOwned;
use core::borrow::Borrow;
use indoc::writedoc;
use jiff::Zoned;
use jiff::tz::TimeZone;
use std::fmt::Write;

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

impl<M: Mode, R> ZonedAssertions<R> for AssertThat<'_, Zoned, M, R> {
    #[track_caller]
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        R: ValueRenderer<Zoned>,
    {
        self.track_assertion();

        let zdt = self.actual();

        let expected = expected.borrow();
        if self.actual().time_zone() != expected.borrow() {
            let actual_time_zone = self.actual().time_zone();
            let actual = actual_time_zone
                .iana_name()
                .map_or_else(|| format!("{actual_time_zone:?}"), ToOwned::to_owned);

            let expected = expected
                .iana_name()
                .map_or_else(|| format!("{expected:?}"), ToOwned::to_owned);

            let zdt = self.render_value(zdt);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected}

                      Actual: {actual}

                      Object: {zdt:#?}
                "}
            });
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
        let actual = self.actual().time_zone().iana_name();

        match actual {
            None => {
                let object = self.render_value(self.actual());
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: '{expected}'

                          Actual: Zoned without a named time zone.

                          Object: {object:#?}
                    "}
                });
            }
            Some(actual) if actual != expected => {
                let object = self.render_value(self.actual());
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Expected: {expected}

                          Actual: {actual}

                          Object: {object:#?}
                    "}
                });
            }
            _ => {}
        }
        self
    }
}

#[cfg(test)]
mod tests {
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
                    Expected: Europe/Berlin

                      Actual: America/New_York

                      Object: 2024-06-19T15:22:00-04:00[America/New_York]
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
                    Expected: Europe/Berlin

                      Actual: TimeZone(05:00:00)

                      Object: 2024-06-19T15:22:00+05:00[+05:00]
                    -------- assertr --------
                "});
        }
    }

    mod is_in_time_zone_named {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::Zoned;

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
                    Expected: Europe/Berlin

                      Actual: America/New_York

                      Object: 2024-06-19T15:22:00-04:00[America/New_York]
                    -------- assertr --------
                "});
        }
    }
}
