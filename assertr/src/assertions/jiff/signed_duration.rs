use crate::assertions::distance::NumericDistance;
use crate::borrow_for::BorrowFor;
use crate::mode::Mode;
use crate::{
    AssertThat, DebugRenderer, ValueRenderer,
    renderer::{Rendered, RenderingContext},
};
use jiff::SignedDuration;

// SignedDuration's alternate Debug form shows raw nanoseconds. Keep its compact form in reports.
fn compact<R: ValueRenderer<SignedDuration>>(
    render: RenderingContext<'_, R>,
    value: &SignedDuration,
) -> Rendered {
    render.compact().value(value)
}

sign_expectations!(SignedDuration, zero: SignedDuration::ZERO, present: compact);

pub use crate::assertions::distance::IsCloseTo;

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
    fn is_close_to<E, D>(self, expected: E, allowed_deviation: D) -> Self
    where
        E: BorrowFor<SignedDuration, View = SignedDuration>,
        D: BorrowFor<SignedDuration, View = SignedDuration>,
        R: ValueRenderer<SignedDuration>,
    {
        self.apply_assertion(IsCloseTo::new(expected, allowed_deviation))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;
        use jiff::SignedDuration;

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

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

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
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

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
        use crate::test_support::{SENTINEL, SentinelRenderer};
        use indoc::formatdoc;
        use jiff::SignedDuration;

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
            use super::super::IsCloseTo;
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
        fn resolves_operands_once_in_order_and_reuses_rejected_views() {
            use super::super::IsCloseTo;
            use crate::borrow_for::BorrowFor;
            use core::{
                borrow::Borrow,
                cell::{Cell, RefCell},
            };
            struct Operand<'a> {
                value: SignedDuration,
                calls: Cell<usize>,
                name: &'static str,
                events: &'a RefCell<Vec<&'static str>>,
            }
            impl Borrow<SignedDuration> for Operand<'_> {
                fn borrow(&self) -> &SignedDuration {
                    self.events.borrow_mut().push(self.name);
                    let previous = self.calls.replace(self.calls.get() + 1);
                    if previous == 0 {
                        &self.value
                    } else {
                        &SignedDuration::MAX
                    }
                }
            }
            impl BorrowFor<SignedDuration> for Operand<'_> {
                type View = SignedDuration;
            }
            struct DurationRenderer;
            impl ValueRenderer<SignedDuration> for DurationRenderer {
                fn fmt(
                    &self,
                    value: &SignedDuration,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "{} seconds", value.as_secs())
                }
            }
            for deviation in [-1, 0, 1] {
                let events = RefCell::new(Vec::new());
                let operand = |value, name| Operand {
                    value: SignedDuration::from_secs(value),
                    calls: Cell::new(0),
                    name,
                    events: &events,
                };
                let matcher =
                    IsCloseTo::new(operand(3, "expected"), operand(deviation, "deviation"));
                assert_that!(&*events.borrow()).is_empty();
                let failures = assert_that!(SignedDuration::from_secs(4))
                    .with_renderer(DurationRenderer)
                    .with_location(false)
                    .capture(|it| it.matches(&matcher));
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);
                let plain = assert_that!(SignedDuration::from_secs(4))
                    .with_renderer(DurationRenderer)
                    .with_location(false)
                    .capture(|it| {
                        it.is_close_to(
                            SignedDuration::from_secs(3),
                            SignedDuration::from_secs(deviation),
                        )
                    });
                assert_that!(failures).is_equal_to(plain);
                events.borrow_mut().clear();
                let root = assert_that!(SignedDuration::ZERO).with_renderer(DurationRenderer);
                let description = root
                    .assertion_context()
                    .describe::<SignedDuration, _>(&matcher);
                assert_that!(description.relation.as_deref()).is_equal_to(Some("is close to"));
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);
                // Re-evaluation resolves the new views.
                events.borrow_mut().clear();
                let root = assert_that!(SignedDuration::MAX);
                assert_that!(
                    matcher
                        .evaluate(&SignedDuration::MAX, &root.assertion_context())
                        .is_ok()
                )
                .is_true();
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);

                events.borrow_mut().clear();
                assert_that!(SignedDuration::from_secs(4))
                    .with_renderer(DurationRenderer)
                    .is_close_to(operand(3, "expected"), SignedDuration::from_secs(1));
                assert_that!(&*events.borrow()).contains_exactly(["expected"]);
                #[cfg(feature = "fluent")]
                {
                    events.borrow_mut().clear();
                    let failures = assert_that!(SignedDuration::from_secs(4))
                        .with_renderer(DurationRenderer)
                        .capture(|it| {
                            it.be_close_to(operand(3, "expected"), operand(deviation, "deviation"))
                        });
                    assert_that!(failures).has_length(usize::from(deviation < 1));
                    assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);
                }
            }
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
                  - The allowed deviation must be zero or positive.
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
            assert_that!(rendered(&failures[0].facts[0].value)).is_equal_to(SENTINEL);
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
