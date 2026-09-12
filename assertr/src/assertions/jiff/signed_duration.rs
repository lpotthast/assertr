use crate::failure::{Fact, FailureKind};
use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer, renderer::Compact};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use jiff::SignedDuration;

/// Checks whether a `SignedDuration` is zero.
pub struct IsZero;
impl<R> Expectation<SignedDuration, R> for IsZero {
    type Success<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a SignedDuration,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_zero() { Ok(()) } else { Err(()) }
    }
}
impl<R> ExpectationDiagnostics<SignedDuration, R> for IsZero
where
    R: ValueRenderer<SignedDuration>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a SignedDuration, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("is zero")
                .expected(Compact(render.value(&SignedDuration::ZERO))),
            Some((actual, ())) => failure
                .actual(Compact(render.value(actual)))
                .expected(Compact(render.value(&SignedDuration::ZERO))),
        }
    }
}
/// Checks whether a `SignedDuration` is negative.
pub struct IsNegative;
impl<R> Expectation<SignedDuration, R> for IsNegative {
    type Success<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a SignedDuration,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_negative() {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<R> ExpectationDiagnostics<SignedDuration, R> for IsNegative
where
    R: ValueRenderer<SignedDuration>,
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a SignedDuration, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is negative"),
            Some((actual, ())) => failure
                .actual(Compact(render.value(actual)))
                .relation("is not negative"),
        }
    }
}
/// Checks whether a `SignedDuration` is positive.
pub struct IsPositive;
impl<R> Expectation<SignedDuration, R> for IsPositive {
    type Success<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a SignedDuration,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_positive() {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<R> ExpectationDiagnostics<SignedDuration, R> for IsPositive
where
    R: ValueRenderer<SignedDuration>,
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a SignedDuration, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is positive"),
            Some((actual, ())) => failure
                .actual(Compact(render.value(actual)))
                .relation("is not positive"),
        }
    }
}
/// Compares exact nanosecond distance within a non-negative inclusive deviation.
pub struct IsCloseTo {
    expected: SignedDuration,
    allowed_deviation: SignedDuration,
}
impl<R> Expectation<SignedDuration, R> for IsCloseTo {
    type Success<'a>
        = ()
    where
        Self: 'a,
        SignedDuration: 'a;
    type Rejection<'a>
        = bool
    where
        Self: 'a,
        SignedDuration: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a SignedDuration,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if self.allowed_deviation.is_negative() {
            return Err(true);
        }
        // The full MIN-to-MAX distance fits in i128 nanoseconds.
        let distance = (actual.as_nanos() - self.expected.as_nanos()).abs();
        if distance <= self.allowed_deviation.as_nanos() {
            Ok(())
        } else {
            Err(false)
        }
    }
}
impl<R> ExpectationDiagnostics<SignedDuration, R> for IsCloseTo
where
    R: ValueRenderer<SignedDuration>,
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a SignedDuration, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            Some((_, true)) => failure
                .relation("was given an invalid allowed deviation")
                .fact(Fact::labelled(
                    "Allowed deviation",
                    Compact(render.value(&self.allowed_deviation)),
                ))
                .fact(Fact::note(
                    "The allowed deviation must be a non-negative duration.",
                )),
            observation => {
                let failure = match observation {
                    None => failure.relation("is close to"),
                    Some((actual, _)) => failure
                        .actual(Compact(render.value(actual)))
                        .relation("is not close to"),
                };
                failure
                    .expected(Compact(render.value(&self.expected)))
                    .fact(Fact::labelled(
                        "Allowed deviation",
                        Compact(render.value(&self.allowed_deviation)),
                    ))
            }
        }
    }
}
impl IsCloseTo {
    /// Expects a duration within this inclusive deviation of `expected`.
    #[must_use]
    pub const fn new(expected: SignedDuration, allowed_deviation: SignedDuration) -> Self {
        Self {
            expected,
            allowed_deviation,
        }
    }
}

/// Assertions for [`SignedDuration`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait SignedDurationAssertions<R = crate::DebugRenderer> {
    /// Asserts that the duration is zero.
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is strictly negative.
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is strictly positive.
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is within `allowed_deviation` of `expected`.
    ///
    /// Compares the exact absolute distance in nanoseconds to `allowed_deviation`, inclusively,
    /// without overflowing even at [`SignedDuration::MIN`] or [`SignedDuration::MAX`].
    ///
    /// A negative `allowed_deviation` is invalid and fails the assertion.
    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self
    where
        R: ValueRenderer<SignedDuration>;
}

impl<M: Mode, R> SignedDurationAssertions<R> for AssertThat<'_, SignedDuration, M, R> {
    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.apply_assertion(IsZero)
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.apply_assertion(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.apply_assertion(IsPositive)
    }

    #[track_caller]
    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.apply_assertion(IsCloseTo::new(expected, allowed_deviation))
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use jiff::SignedDuration;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, SignedDuration, Panic, NoRenderer>
                    => SignedDurationAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!(SignedDuration::from_secs(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(SignedDurationAssertions::is_zero);

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::ZERO.must().be_zero();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let duration: SignedDuration = "2h 30m".parse().unwrap();
            assert_caller_location!(assert_that!(duration), is_zero());
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(SignedDuration::ZERO).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: SignedDuration = "2h 30m".parse().unwrap();

            assert_that_panic_by(|| {
                assert_that!(duration).with_location(false).is_zero();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `duration`

                    Expected: 0s

                      Actual: 2h 30m
                    -------- assertr --------
                "});
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs(-5).must().be_negative();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(SignedDuration::ZERO), is_negative());
        }

        #[test]
        fn succeeds_when_negative() {
            assert_that!(SignedDuration::from_secs(-5)).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::ZERO`

                    Actual: 0s

                    is not negative
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs(5))
                    .with_location(false)
                    .is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs(5)`

                    Actual: 5s

                    is not negative
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs(5).must().be_positive();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(SignedDuration::ZERO), is_positive());
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(SignedDuration::from_secs(5)).is_positive();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::ZERO`

                    Actual: 0s

                    is not positive
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs(-5))
                    .with_location(false)
                    .is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs(-5)`

                    Actual: 5s ago

                    is not positive
                    -------- assertr --------
                "});
        }
    }

    mod is_close_to {
        use crate::failure::FailureKind;
        use crate::prelude::*;
        use crate::test_support::{SENTINEL, SentinelRenderer, rendered_text};
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs_f32(0.333).must().be_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(SignedDuration::ZERO),
                is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1))
            );
        }

        #[test]
        fn succeeds_for_equal_extremes() {
            for value in [SignedDuration::MAX, SignedDuration::MIN] {
                for deviation in [SignedDuration::ZERO, SignedDuration::from_secs(1)] {
                    assert_that!(value).is_close_to(value, deviation);
                }
            }
        }

        #[test]
        fn captures_no_failures_for_equal_extremes() {
            for value in [SignedDuration::MAX, SignedDuration::MIN] {
                for deviation in [SignedDuration::ZERO, SignedDuration::from_secs(1)] {
                    let failures =
                        assert_that!(value).capture(|it| it.is_close_to(value, deviation));
                    assert_that!(failures).is_empty();
                }
            }
        }

        #[test]
        fn compares_exact_distances_near_both_extremes() {
            let second = SignedDuration::from_secs(1);
            let nanosecond = SignedDuration::from_nanos(1);
            for (a, b, failure_count) in [
                (SignedDuration::MIN, SignedDuration::MIN + second, 0),
                (SignedDuration::MAX, SignedDuration::MAX - second, 0),
                (
                    SignedDuration::MIN,
                    SignedDuration::MIN + second + nanosecond,
                    1,
                ),
                (
                    SignedDuration::MAX,
                    SignedDuration::MAX - second - nanosecond,
                    1,
                ),
            ] {
                for (actual, expected) in [(a, b), (b, a)] {
                    let failures =
                        assert_that!(actual).capture(|it| it.is_close_to(expected, second));
                    assert_that!(failures).has_length(failure_count);
                }
            }
        }

        #[test]
        fn compares_exact_distances_across_zero() {
            let negative = SignedDuration::from_nanos(-1);
            let positive = SignedDuration::from_nanos(1);
            for (actual, expected) in [(negative, positive), (positive, negative)] {
                for (deviation, failure_count) in [(2, 0), (1, 1)] {
                    let failures = assert_that!(actual).capture(|it| {
                        it.is_close_to(expected, SignedDuration::from_nanos(deviation))
                    });
                    assert_that!(failures).has_length(failure_count);
                }
            }
        }

        #[test]
        fn handles_distances_at_and_beyond_the_maximum_duration() {
            for (a, b, failure_count) in [
                (SignedDuration::ZERO, SignedDuration::MAX, 0),
                (SignedDuration::MIN, SignedDuration::from_secs(-1), 0),
                (
                    SignedDuration::MIN,
                    SignedDuration::from_nanos(-999_999_999),
                    1,
                ),
                (SignedDuration::ZERO, SignedDuration::MIN, 1),
                (SignedDuration::MIN, SignedDuration::MAX, 1),
            ] {
                for (actual, expected) in [(a, b), (b, a)] {
                    let failures = assert_that!(actual)
                        .capture(|it| it.is_close_to(expected, SignedDuration::MAX));
                    assert_that!(failures).has_length(failure_count);
                }
            }
        }

        #[test]
        fn zero_deviation_rejects_a_one_nanosecond_distance() {
            for (actual, expected) in [
                (SignedDuration::ZERO, SignedDuration::from_nanos(1)),
                (SignedDuration::from_nanos(1), SignedDuration::ZERO),
            ] {
                let failures = assert_that!(actual)
                    .capture(|it| it.is_close_to(expected, SignedDuration::ZERO));
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn rejects_negative_deviations_even_at_extremes() {
            for actual in [
                SignedDuration::MIN,
                SignedDuration::ZERO,
                SignedDuration::MAX,
            ] {
                for expected in [
                    SignedDuration::MIN,
                    SignedDuration::ZERO,
                    SignedDuration::MAX,
                ] {
                    for deviation in [SignedDuration::from_nanos(-1), SignedDuration::MIN] {
                        let failures =
                            assert_that!(actual).capture(|it| it.is_close_to(expected, deviation));
                        assert_that!(failures).contains_exactly_satisfying([
                            |element: AssertThat<AssertionFailure, Capture>| {
                                element
                                    .derive(|value| &value.kind)
                                    .is_equal_to(FailureKind::Ordering);
                                element
                                    .derive_owned(|value| value.relation.as_deref())
                                    .is_equal_to(Some("was given an invalid allowed deviation"));
                            },
                        ]);
                    }
                }
            }
        }

        #[test]
        fn capture_collects_failures_and_allows_further_chaining() {
            let failures = assert_that!(SignedDuration::MAX).capture(|it| {
                it.is_close_to(SignedDuration::MIN, SignedDuration::MAX)
                    .is_close_to(SignedDuration::MAX, SignedDuration::MIN)
                    .is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1))
                    .is_equal_to(SignedDuration::MAX)
            });

            assert_that!(failures).contains_exactly_satisfying(
                ["is not close to", "was given an invalid allowed deviation"].map(|relation| {
                    move |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive(|failure| &failure.kind)
                            .is_equal_to(FailureKind::Ordering);
                        failure
                            .derive_owned(|failure| failure.relation.as_deref())
                            .is_equal_to(Some(relation));
                    }
                }),
            );
        }

        #[test]
        fn reports_extreme_values_without_overflowing() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `SignedDuration::ZERO`

                Actual: 0s

                is not close to

                Expected: 2562047788015215h 30m 7s 999ms 999µs 999ns

                Details:
                  - Allowed deviation: 1s
                -------- assertr --------
            "});
        }

        #[test]
        fn reports_negative_deviation() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_close_to(SignedDuration::ZERO, SignedDuration::from_secs(-1));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `SignedDuration::ZERO`

                was given an invalid allowed deviation

                Details:
                  - Allowed deviation: 1s ago
                  - The allowed deviation must be a non-negative duration.
                -------- assertr --------
            "});
        }

        #[test]
        fn failures_render_all_duration_values_with_the_active_renderer() {
            let failures = assert_that!(SignedDuration::MIN)
                .with_renderer(SentinelRenderer)
                .capture(|it| it.is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1)));

            assert_that!(failures).contains_exactly_satisfying([
                |subject: AssertThat<AssertionFailure, Capture>| {
                    subject
                        .derive_owned(|entry| rendered_text(entry.actual.as_ref().unwrap()))
                        .is_equal_to(SENTINEL);
                    subject
                        .derive_owned(|entry| rendered_text(entry.expected.as_ref().unwrap()))
                        .is_equal_to(SENTINEL);
                    subject
                        .derive(|entry| &entry.facts)
                        .contains_exactly_satisfying([
                            |element: AssertThat<crate::Fact, Capture>| {
                                element
                                    .derive_owned(|item| rendered_text(&item.value))
                                    .is_equal_to(SENTINEL);
                            },
                        ]);
                },
            ]);
        }

        #[test]
        fn invalid_deviation_uses_the_active_renderer() {
            let failures = assert_that!(SignedDuration::MAX)
                .with_renderer(SentinelRenderer)
                .capture(|it| it.is_close_to(SignedDuration::MAX, SignedDuration::MIN));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|item| rendered_text(&item.facts[0].value))
                        .is_equal_to(SENTINEL);
                },
            ]);
        }

        #[test]
        fn panics_when_below_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs_f32(0.3319))
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs_f32(0.3319)`

                    Actual: 331ms 900µs

                    is not close to

                    Expected: 333ms

                    Details:
                      - Allowed deviation: 1ms
                    -------- assertr --------
                "});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that!(SignedDuration::from_secs_f32(0.332)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            assert_that!(SignedDuration::from_secs_f32(0.333)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            assert_that!(SignedDuration::from_secs_f32(0.334)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs_f32(0.3341))
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs_f32(0.3341)`

                    Actual: 334ms 100µs

                    is not close to

                    Expected: 333ms

                    Details:
                      - Allowed deviation: 1ms
                    -------- assertr --------
                "});
        }
    }
}
