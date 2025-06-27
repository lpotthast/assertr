use crate::AssertThat;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use core::borrow::Borrow;
use indoc::writedoc;
use jiff::Zoned;
use jiff::tz::TimeZone;
use std::fmt::Write;

pub trait ZonedAssertions {
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self;

    fn be_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self
    where
        Self: Sized,
    {
        self.is_in_time_zone(expected)
    }

    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self;

    fn be_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        self.is_in_time_zone_named(expected)
    }
}

impl<M: Mode> ZonedAssertions for AssertThat<'_, Zoned, M> {
    #[track_caller]
    fn is_in_time_zone(self, expected: impl Borrow<TimeZone>) -> Self {
        self.track_assertion();

        let zdt = self.actual();

        let expected = expected.borrow();
        if self.actual().time_zone() != expected.borrow() {
            let actual = self
                .actual()
                .time_zone()
                .iana_name()
                .map(|it| it.to_owned())
                .unwrap_or_else(|| format!("{expected:?}"));

            let expected = expected
                .iana_name()
                .map(|it| it.to_owned())
                .unwrap_or_else(|| format!("{expected:?}"));

            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected}

                      Actual: {actual}

                      Object: {zdt:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();

        let expected = expected.as_ref();
        let actual = self.actual().time_zone().iana_name();

        match actual {
            None => self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: '{expected}'

                      Actual: Zoned without a named time zone.

                      Object: {:#?}
                "#, self.actual()}
            }),
            Some(actual) if actual != expected => self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected}

                      Actual: {actual}

                      Object: {:#?}
                "#, self.actual()}
            }),
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
        use jiff::tz::TimeZone;

        #[test]
        fn succeeds_when_matches() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("America/New_York").expect("valid");
            zdt.must().be_in_time_zone(tz);
        }

        #[test]
        fn panics_when_in_different_time_zone() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            let tz = TimeZone::get("Europe/Berlin").expect("valid");

            assert_that_panic_by(|| {
                zdt.assert().with_location(false).is_in_time_zone(tz);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: Europe/Berlin

                      Actual: America/New_York

                      Object: 2024-06-19T15:22:00-04:00[America/New_York]
                    -------- assertr --------
                "#});
        }
    }

    mod is_in_time_zone_named {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::Zoned;

        #[test]
        fn succeeds_when_matches() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            zdt.must().be_in_time_zone_named("America/New_York");
        }

        #[test]
        fn panics_when_in_different_time_zone() {
            let zdt: Zoned = "2024-06-19 15:22[America/New_York]".parse().expect("valid");
            assert_that_panic_by(|| {
                zdt.assert()
                    .with_location(false)
                    .is_in_time_zone_named("Europe/Berlin");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: Europe/Berlin

                      Actual: America/New_York

                      Object: 2024-06-19T15:22:00-04:00[America/New_York]
                    -------- assertr --------
                "#});
        }
    }
}
