use jiff::SignedDuration;

use crate::{
    AssertThat,
    assertions::distance::NumericDistance,
    borrow_for::BorrowFor,
    mode::Mode,
    renderer::{DebugRenderer, Rendered, RenderingContext, ValueRenderer},
};

// SignedDuration's alternate Debug form shows raw nanoseconds. Keep its compact form in reports.
fn compact<R: ValueRenderer<SignedDuration>>(
    render: RenderingContext<'_, R>,
    value: &SignedDuration,
) -> Rendered {
    render.compact().value(value)
}

sign_expectations!(SignedDuration, zero: SignedDuration::ZERO, present: compact);

use crate::assertions::distance::IsCloseTo;

/// Exact, overflow-checked distance between durations. A distance larger than
/// `SignedDuration::MAX` cannot be represented and therefore never satisfies a deviation.
impl NumericDistance for SignedDuration {
    fn zero_distance() -> Self {
        SignedDuration::ZERO
    }

    fn checked_distance(&self, other: &Self) -> Option<Self> {
        // The full MIN-to-MAX distance fits in i128 nanoseconds, unlike `checked_sub` near MIN.
        const NANOS_PER_SECOND: i128 = 1_000_000_000;
        let distance = (self.as_nanos() - other.as_nanos()).abs();
        let secs = i64::try_from(distance / NANOS_PER_SECOND).ok()?;
        let nanos = i32::try_from(distance % NANOS_PER_SECOND).ok()?;
        Some(SignedDuration::new(secs, nanos))
    }
}

/// Assertions for [`SignedDuration`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait SignedDurationAssertions<R = DebugRenderer> {
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
    /// Both operands can independently be owned, borrowed, or custom [`BorrowFor`] wrappers with
    /// `View = SignedDuration`. The renderer only needs to support that selected duration view.
    ///
    /// Because both parameters are generic, they cannot determine the target type of an untyped
    /// `.into()` or `Default::default()` call. Write `SignedDuration::default()` or otherwise state
    /// the intended type.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use jiff::SignedDuration;
    /// let expected = SignedDuration::from_secs(3);
    /// assert_that!(expected).is_close_to(&expected, SignedDuration::default());
    /// ```
    fn is_close_to<E, D>(self, expected: E, allowed_deviation: D) -> Self
    where
        E: BorrowFor<SignedDuration, View = SignedDuration>,
        D: BorrowFor<SignedDuration, View = SignedDuration>,
        R: ValueRenderer<SignedDuration>;
}

impl<M: Mode, R> SignedDurationAssertions<R> for AssertThat<'_, SignedDuration, M, R> {
    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.matches(IsZero)
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.matches(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.matches(IsPositive)
    }

    #[track_caller]
    fn is_close_to<E, D>(self, expected: E, allowed_deviation: D) -> Self
    where
        E: BorrowFor<SignedDuration, View = SignedDuration>,
        D: BorrowFor<SignedDuration, View = SignedDuration>,
        R: ValueRenderer<SignedDuration>,
    {
        self.matches(IsCloseTo::new(expected, allowed_deviation))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use jiff::SignedDuration;

        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            SignedDuration::ZERO.must().be_zero();
            SignedDuration::from_secs(-5).must().be_negative();
            SignedDuration::from_secs(5)
                .must()
                .be_positive()
                .be_close_to(SignedDuration::from_secs(4), SignedDuration::from_secs(1));
        }
    }

    mod renderer_contract {
        use jiff::SignedDuration;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

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

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod is_zero {
        use indoc::formatdoc;
        use jiff::SignedDuration;

        use crate::prelude::*;

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

            assert_that!(|| {
                assert_that!(duration).with_location(false).is_zero();
            })
            .panics()
            .has_message()
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
        use indoc::formatdoc;
        use jiff::SignedDuration;

        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_negative();
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::ZERO`

                    Actual: 0s

                    is not negative
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use indoc::formatdoc;
        use jiff::SignedDuration;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(SignedDuration::ZERO), is_positive());
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(SignedDuration::from_secs(5)).is_positive();
        }

        #[test]
        fn panics_when_negative() {
            assert_that!(|| {
                assert_that!(SignedDuration::from_secs(-5))
                    .with_location(false)
                    .is_positive();
            })
            .panics()
            .has_message()
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
        use indoc::formatdoc;
        use jiff::SignedDuration;

        use crate::{
            prelude::*,
            test_support::{SENTINEL, SentinelRenderer},
        };

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(SignedDuration::ZERO),
                is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1))
            );
        }

        #[test]
        // Borrowed forms are the API contract being tested, even for Copy durations.
        #[allow(clippy::needless_borrows_for_generic_args)]
        fn accepts_owned_borrowed_and_independently_typed_operands() {
            use crate::assertions::distance::IsCloseTo;
            const TYPED: IsCloseTo<SignedDuration> =
                IsCloseTo::new(SignedDuration::ZERO, SignedDuration::ZERO);
            let expected = SignedDuration::from_secs(3);
            let deviation = SignedDuration::from_secs(1);
            let actual = SignedDuration::from_secs(4);
            assert_that!(SignedDuration::ZERO).matches(TYPED);
            assert_that!(actual)
                .is_close_to(expected, &deviation)
                .is_close_to(&expected, deviation)
                .is_close_to(&expected, &deviation)
                .matches(IsCloseTo::new(&expected, deviation));
            let reusable = IsCloseTo::new(expected, &deviation);
            assert_that!(actual).matches(&reusable).matches(&reusable);
            #[cfg(feature = "fluent")]
            actual.must().be_close_to(&expected, &deviation);
        }

        #[test]
        fn accepts_equal_extremes() {
            for value in [SignedDuration::MAX, SignedDuration::MIN] {
                for deviation in [SignedDuration::ZERO, SignedDuration::from_secs(1)] {
                    assert_that!(value).is_close_to(value, deviation);
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
        fn reports_extreme_values_without_overflowing() {
            assert_that!(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1));
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `SignedDuration::ZERO`

                Actual: 0s

                is not close to

                Expected: 2562047788015215h 30m 7s 999ms 999µs 999ns

                Details:
                  - Distance: 2562047788015215h 30m 7s 999ms 999µs 999ns
                  - Allowed deviation: 1s
                -------- assertr --------
            "});
        }

        #[test]
        fn failures_render_all_duration_values_with_the_active_renderer() {
            let failures = assert_that!(SignedDuration::MIN)
                .with_renderer(SentinelRenderer)
                .capture(|it| {
                    it.is_close_to(SignedDuration::MAX, SignedDuration::from_secs(1))
                        .is_close_to(SignedDuration::MAX, SignedDuration::MIN)
                });

            let rendered = |value: &crate::renderer::Rendered| format!("{value:#}");
            assert_that!(rendered(failures[0].actual.as_ref().unwrap())).is_equal_to(SENTINEL);
            assert_that!(rendered(failures[0].expected.as_ref().unwrap())).is_equal_to(SENTINEL);
            // The distance between the extremes cannot be represented, so the first fact is a note.
            assert_that!(rendered(&failures[0].facts[1].value)).is_equal_to(SENTINEL);
            assert_that!(rendered(&failures[1].facts[0].value)).is_equal_to(SENTINEL);
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
        fn rejects_values_outside_the_allowed_range() {
            for actual in [0.3319, 0.3341] {
                let failures = assert_that!(SignedDuration::from_secs_f32(actual)).capture(|it| {
                    it.is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    )
                });
                assert_that!(failures).has_length(1);
            }
        }
    }
}
