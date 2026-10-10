use jiff::{Zoned, tz::TimeZone};

use crate::{
    AssertThat,
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Mode,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Explains a time-zone comparison, naming the subject's time zone on rejection.
fn explain_time_zone<X: ?Sized, R>(
    expected: &X,
    rejected: Option<(&Zoned, (&TimeZone, &X))>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder
where
    R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<X>,
{
    let render = context.render();
    let failure = failure
        .relations(
            rejected.map(|(actual, _)| render.value(actual)),
            "is in time zone",
            "is not in time zone",
        )
        .expected(render.value(expected));
    match rejected {
        None => failure,
        Some((_, (zone, _))) => {
            failure.fact(Fact::labelled("Actual time zone", render.value(zone)))
        }
    }
}

/// Compares the observed time-zone rules with an expected [`TimeZone`].
///
/// The expected operand selects a `TimeZone` view through [`BorrowFor`].
#[derive(Debug, Clone)]
pub struct IsInTimeZone<E>(E);

impl<E> IsInTimeZone<E> {
    /// Expects the time-zone rules of this time zone.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<E, R> Expectation<Zoned, R> for IsInTimeZone<E>
where
    E: BorrowFor<TimeZone, View = TimeZone>,
    R: ValueRenderer<Zoned> + ValueRenderer<TimeZone>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Zoned: 'a;
    type Rejection<'a>
        = (&'a TimeZone, &'a TimeZone)
    where
        Self: 'a,
        Zoned: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Zoned,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<TimeZone, _>(&self.0);
        let actual = actual.time_zone();
        if actual == expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Zoned, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let expected = rejected.map_or_else(
            || borrow_for::<TimeZone, _>(&self.0),
            |(_, (_, expected))| expected,
        );
        explain_time_zone(expected, rejected, failure, context)
    }
}

/// Compares the observed time zone's IANA name with an expected name.
///
/// A time zone without an IANA name, such as a fixed offset, never matches.
#[derive(Debug, Clone)]
pub struct IsInTimeZoneNamed<E>(E);

impl<E> IsInTimeZoneNamed<E> {
    /// Expects a time zone with this IANA name.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<E, R> Expectation<Zoned, R> for IsInTimeZoneNamed<E>
where
    E: AsRef<str>,
    R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<str>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Zoned: 'a;
    type Rejection<'a>
        = (&'a TimeZone, &'a str)
    where
        Self: 'a,
        Zoned: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Zoned,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.0.as_ref();
        let actual = actual.time_zone();
        if actual.iana_name() == Some(expected) {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Zoned, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let expected = rejected.map_or_else(|| self.0.as_ref(), |(_, (_, expected))| expected);
        explain_time_zone(expected, rejected, failure, context)
    }
}

/// Assertions for [`Zoned`] date-times.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ZonedAssertions<R = DebugRenderer> {
    /// Asserts that the subject's time zone equals `expected`.
    ///
    /// This uses jiff's `TimeZone` equality, which compares how a zone is represented rather than
    /// its rules. Zones with identical rules from different sources can differ, for example
    /// `TimeZone::get("Etc/UTC")` and `TimeZone::UTC`. Use
    /// [`is_in_time_zone_named`](Self::is_in_time_zone_named) to compare IANA names instead.
    fn is_in_time_zone<E: BorrowFor<TimeZone, View = TimeZone>>(self, expected: E) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone>;

    /// Asserts that the subject has an IANA time-zone name equal to `expected`.
    ///
    /// A subject using an unnamed fixed-offset or POSIX time zone fails this assertion.
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<str>;
}

impl<M: Mode, R> ZonedAssertions<R> for AssertThat<'_, Zoned, M, R> {
    #[track_caller]
    fn is_in_time_zone<E: BorrowFor<TimeZone, View = TimeZone>>(self, expected: E) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone>,
    {
        self.matches(IsInTimeZone::new(expected))
    }

    #[track_caller]
    fn is_in_time_zone_named(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Zoned> + ValueRenderer<TimeZone> + ValueRenderer<str>,
    {
        self.matches(IsInTimeZoneNamed::new(expected))
    }
}

#[cfg(test)]
mod tests {
    use jiff::{
        Zoned,
        tz::{self, TimeZone},
    };

    fn new_york() -> Zoned {
        "2024-06-19 15:22[America/New_York]".parse().expect("valid")
    }

    fn fixed_offset() -> Zoned {
        jiff::civil::date(2024, 6, 19)
            .at(15, 22, 0, 0)
            .to_zoned(TimeZone::fixed(tz::offset(5)))
            .expect("valid")
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            let tz = TimeZone::get("America/New_York").expect("valid");
            new_york()
                .must()
                .be_in_time_zone(tz)
                .be_in_time_zone_named("America/New_York");
        }
    }

    mod renderer_contract {
        use jiff::{Zoned, tz::TimeZone};

        use super::{fixed_offset, new_york};
        use crate::{
            prelude::*,
            test_support::{
                CustomValueRenderer, NoRenderer, RedactingRenderer, assert_custom_value,
                assert_redacted, assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Zoned, Panic, NoRenderer> => ZonedAssertions<NoRenderer>
            );
        }

        #[test]
        fn zone_evidence_renders_and_redacts_through_the_active_renderer() {
            let berlin = TimeZone::get("Europe/Berlin").unwrap();
            for subject in [new_york(), fixed_offset()] {
                macro_rules! case {
                    ($check:expr, $expected:expr) => {{
                        let failures = assert_that!(subject)
                            .with_renderer(CustomValueRenderer)
                            .capture($check);
                        assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject);
                        assert_custom_value(failures[0].expected.as_ref().unwrap(), $expected);
                        assert_custom_value(&failures[0].facts[0].value, subject.time_zone());

                        let failures = assert_that!(subject)
                            .with_renderer(RedactingRenderer)
                            .with_location(false)
                            .capture($check);
                        assert_redacted(
                            &failures[0],
                            &["Europe/Berlin", "America/New_York", "05:00"],
                        );
                    }};
                }
                case!(|it| it.is_in_time_zone(&berlin), &berlin);
                case!(
                    |it| it.is_in_time_zone_named("Europe/Berlin"),
                    "Europe/Berlin"
                );
            }
        }
    }

    mod is_in_time_zone {
        use indoc::formatdoc;
        use jiff::tz::TimeZone;

        use super::{fixed_offset, new_york};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let tz = TimeZone::get("Europe/Berlin").expect("valid");
            assert_caller_location!(assert_that!(new_york()), is_in_time_zone(tz));
        }

        #[test]
        fn succeeds_when_matches() {
            let tz = TimeZone::get("America/New_York").expect("valid");
            assert_that!(new_york()).is_in_time_zone(tz);
        }

        #[test]
        fn panics_with_the_actual_zone_when_it_is_unnamed() {
            let zdt = fixed_offset();
            let tz = TimeZone::get("Europe/Berlin").expect("valid");

            assert_that!(|| {
                assert_that!(zdt).with_location(false).is_in_time_zone(tz);
            })
            .panics()
            .has_message()
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
        use indoc::formatdoc;

        use super::{fixed_offset, new_york};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(new_york()),
                is_in_time_zone_named("Europe/Berlin")
            );
        }

        #[test]
        fn succeeds_when_matches() {
            assert_that!(new_york()).is_in_time_zone_named("America/New_York");
        }

        #[test]
        fn never_matches_an_unnamed_zone() {
            let failures =
                assert_that!(fixed_offset()).capture(|it| it.is_in_time_zone_named("+05:00"));
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn panics_when_in_different_time_zone() {
            let zdt = new_york();
            assert_that!(|| {
                assert_that!(zdt)
                    .with_location(false)
                    .is_in_time_zone_named("Europe/Berlin");
            })
            .panics()
            .has_message()
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
    }
}
