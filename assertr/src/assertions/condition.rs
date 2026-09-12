use crate::{
    AssertThat, AssertionContext, DebugRenderer, Expectation, ExpectationDiagnostics, Fact, Mode,
    ValueRenderer,
    condition::AssertrCondition,
    failure::{FailureBuilder, FailureKind},
};
use core::any::type_name;

/// A reusable assertion adapting an [`AssertrCondition`] and retaining its original typed error.
///
/// Use [`new`](Self::new) for structural expectations or [`Expectation::evaluate`] for evaluation
/// alone. [`ConditionAssertions`] and [`IterableConditionAssertions`] execute this same assertion.
/// Diagnostics render only the condition's error, without requiring a subject renderer.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::condition::Condition;
///
/// struct IsEven;
/// impl AssertrCondition<i32> for IsEven {
///     type Error = &'static str;
///     fn test(&self, value: &i32) -> Result<(), Self::Error> {
///         if value % 2 == 0 { Ok(()) } else { Err("value is odd") }
///     }
/// }
///
/// let even = Condition::new(IsEven);
/// assert_that!(2).matches(&even);
/// ```
pub struct Condition<C>(C);

/// Adapts a domain condition without requiring a subject renderer.
///
/// This is a convenience constructor for [`Condition::new`]. Diagnostic evaluation renders the
/// original error through `ValueRenderer<C::Error>`. The default renderer requires `Debug`.
/// Probes never render condition errors.
pub fn condition<C>(condition: C) -> Condition<C> {
    Condition::new(condition)
}

impl<C> Condition<C> {
    /// Owns a condition. Pass a reference to keep the condition usable elsewhere.
    #[must_use]
    pub const fn new(condition: C) -> Self {
        Self(condition)
    }
}

impl<T, C: AssertrCondition<T>, R> Expectation<T, R> for Condition<C> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = C::Error
    where
        Self: 'a,
        T: 'a;

    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), C::Error> {
        self.0.test(actual)
    }
}

impl<T, C: AssertrCondition<T>, R: ValueRenderer<C::Error>> ExpectationDiagnostics<T, R>
    for Condition<C>
{
    const KIND: FailureKind = FailureKind::Predicate;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, C::Error)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("satisfies the condition")
                .expected(type_name::<C>()),
            Some((_, error)) => failure
                .relation("does not match the condition")
                .fact(Fact::note(render.value(&error))),
        }
    }
}

/// Assertions that apply a reusable [`AssertrCondition`] to the subject.
///
/// `has` is a readability alias of `is`. For example, `is(alive)` and `has(name("Bob"))`.
///
/// With the `fluent` feature enabled, `be` is the fluent alias of `is`. For example,
/// `person.must().be(alive)`. `has` has no fluent alias because `have` would be ambiguous with
/// [`IterableConditionAssertions::have`] on iterable subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ConditionAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject matches the given condition.
    ///
    /// On failure, the original error is rendered as an unlabeled note in
    /// [`AssertionFailure::facts`](crate::AssertionFailure::facts). The default renderer requires
    /// `C::Error: Debug`. A custom renderer can support errors with no formatting traits.
    /// No subject renderer is required.
    ///
    /// Pass `&condition` to keep the condition usable for further assertions.
    #[cfg_attr(feature = "fluent", fluent_alias("be"))]
    fn is<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>;

    /// Readability synonym of [`is`](ConditionAssertions::is).
    fn has<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>;
}

impl<T, M: Mode, R> ConditionAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>,
    {
        self.apply_assertion(Condition::new(condition))
    }

    #[track_caller]
    fn has<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>,
    {
        self.is(condition)
    }
}

/// Assertions that apply a reusable condition to every element of an iterable subject.
///
/// Each non-matching element raises its own failure, so capture mode reports every offending
/// element. This order-free assertion never describes traversal offsets as collection indexes; the
/// condition's error should identify the offending value when that context matters.
///
/// `have` is a readability alias of `are`. It also serves as the fluent spelling because
/// `people.must().have(condition)` already reads imperatively.
#[allow(clippy::return_self_not_must_use)]
pub trait IterableConditionAssertions<T, I, R = DebugRenderer>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    /// Asserts that every element of the subject matches the given condition.
    ///
    /// Each offending element's original error is rendered as an unlabeled note in its own
    /// [`AssertionFailure::facts`](crate::AssertionFailure::facts). Only error rendering is needed,
    /// and the active budget applies. The default renderer requires `C::Error: Debug`.
    fn are<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>;

    /// Readability synonym of [`are`](IterableConditionAssertions::are).
    fn have<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>;
}

impl<I, T, M: Mode, R> IterableConditionAssertions<T, I, R> for AssertThat<'_, I, M, R>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn are<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>,
    {
        self.track_assertion();
        let condition = Condition::new(condition);
        let location = core::panic::Location::caller();
        for actual in self.actual() {
            self.test_observation_after_tracking(actual, &condition, location);
        }
        self
    }

    #[track_caller]
    fn have<C: AssertrCondition<T>>(self, condition: C) -> Self
    where
        R: ValueRenderer<C::Error>,
    {
        self.are(condition)
    }
}

#[cfg(test)]
mod renderer_contract {
    use crate::{
        prelude::*,
        test_support::{NoRenderer, assert_trait_impl},
    };

    #[test]
    fn traits_are_implemented_without_renderer_support() {
        assert_trait_impl!(
            AssertThat<'static, i32, Panic, NoRenderer> => ConditionAssertions<i32, NoRenderer>
        );
        assert_trait_impl!(
            AssertThat<'static, [i32; 1], Panic, NoRenderer>
                => IterableConditionAssertions<i32, [i32; 1], NoRenderer>
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        renderer::RenderedBody,
        test_support::{RedactingRenderer, assert_redacted},
    };
    use core::fmt;

    struct Error(u32);
    struct Reject;
    impl AssertrCondition<u32> for Reject {
        type Error = Error;
        fn test(&self, value: &u32) -> Result<(), Error> {
            Err(Error(*value))
        }
    }
    #[derive(Clone, Copy)]
    struct ErrorRenderer;
    impl ValueRenderer<Error> for ErrorRenderer {
        fn fmt(&self, value: &Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "error({})", value.0)
        }
    }
    fn assert_condition_error(failure: &AssertionFailure) {
        assert_that!(failure.facts[0].label).is_equal_to("");
        assert_that!(failure.facts[0].value.type_name)
            .is_equal_to(Some(core::any::type_name::<Error>()));
        assert_that!(failure.facts[0].value.body).is_equal_to(RenderedBody::Text {
            text: "err".into(),
            omitted_characters: 6,
        });
    }
    mod is {
        use super::*;
        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            struct Accept;
            impl AssertrCondition<u32> for Accept {
                type Error = Error;
                fn test(&self, _: &u32) -> Result<(), Error> {
                    Ok(())
                }
            }
            42_u32
                .must()
                .with_renderer(ErrorRenderer)
                .with_location(false)
                .be(Accept);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(42_u32).with_renderer(ErrorRenderer),
                is(Reject)
            );
        }
        #[test]
        fn typed_errors_need_no_formatting_traits_or_subject_renderer() {
            use crate::{
                Expectation,
                assertions::condition::Condition,
                test_support::{NoRenderer, assert_trait_impl},
            };
            use indoc::formatdoc;

            assert_trait_impl!(Condition<Reject> => Expectation<u32, NoRenderer>);

            let subject = 42_u32;
            let failures = assert_that!(subject)
                .with_renderer(ErrorRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| it.is(Reject));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                does not match the condition

                Details:
                  - err... 6 more characters ...
                -------- assertr --------
            "});

                    assert_condition_error(element.actual());
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.is(Reject));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                does not match the condition

                Details:
                  - <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &["42"]);
                },
            ]);
        }
    }
    mod has {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(42_u32).with_renderer(ErrorRenderer),
                has(Reject)
            );
        }
    }

    mod are {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([42_u32]).with_renderer(ErrorRenderer),
                are(Reject)
            );
        }

        #[test]
        fn preserves_each_typed_error_and_budget() {
            use indoc::formatdoc;

            let failures = assert_that!([42_u32, 43])
                .with_renderer(ErrorRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| {
                    let it = it.are(Reject);
                    assert_that!(it.state.records.assertion_count()).is_equal_to(1);
                    it
                });
            assert_that!(failures).contains_exactly_satisfying(
                [|element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[42_u32, 43]`

                does not match the condition

                Details:
                  - err... 6 more characters ...
                -------- assertr --------
            "});

                    assert_condition_error(element.actual());
                }; 2],
            );
        }

        #[test]
        fn panic_mode_stops_at_the_first_rejection() {
            use core::sync::atomic::{AtomicUsize, Ordering};

            struct Counted<'a>(&'a AtomicUsize);
            impl AssertrCondition<u32> for Counted<'_> {
                type Error = Error;
                fn test(&self, value: &u32) -> Result<(), Error> {
                    self.0.fetch_add(1, Ordering::Relaxed);
                    Err(Error(*value))
                }
            }

            let calls = AtomicUsize::new(0);
            assert_that_panic_by(|| {
                assert_that!([42_u32, 43])
                    .with_renderer(ErrorRenderer)
                    .are(Counted(&calls));
            })
            .has_type::<String>();
            assert_that!(calls.load(Ordering::Relaxed)).is_equal_to(1);
        }
    }
    mod have {
        use super::*;
        #[test]
        fn fluent_alias_is_as_expected() {
            struct Accept;
            impl AssertrCondition<u32> for Accept {
                type Error = Error;
                fn test(&self, _: &u32) -> Result<(), Error> {
                    Ok(())
                }
            }
            assert_that!([42_u32])
                .with_renderer(ErrorRenderer)
                .with_location(false)
                .have(Accept);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([42_u32]).with_renderer(ErrorRenderer),
                have(Reject)
            );
        }
    }
    mod matches {
        use super::*;
        use crate::{assertions::condition::condition, expectation::all_of};
        use indoc::formatdoc;

        struct NeverRender;
        impl ValueRenderer<Error> for NeverRender {
            fn fmt(&self, _: &Error, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                panic!("probe rendered")
            }
        }

        #[test]
        fn preserves_typed_condition_errors_in_composed_matchers() {
            let matcher = all_of((condition(Reject),));
            let failures = assert_that!(42_u32)
                .with_renderer(ErrorRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| it.matches(matcher));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `42_u32`

                does not match

                Nested failures:
                  - does not match the condition

                    Details:
                      - err... 6 more characters ...
                -------- assertr --------
            "});

                    assert_condition_error(&element.actual().children[0]);
                },
            ]);
        }

        #[test]
        fn probes_do_not_render_condition_errors() {
            let context = AssertionContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&42_u32, &condition(Reject))).is_false();
            assert_that!(context.into_evidence().children).is_empty();
        }

        #[test]
        fn a_zero_item_budget_preserves_failure_without_rendering_errors() {
            let failures = assert_that!(42_u32)
                .with_renderer(NeverRender)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.matches(all_of((condition(Reject),))));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
            -------- assertr --------
            Expression: `42_u32`

            does not match

            Details:
              - ... 1 more nested failure ...
            -------- assertr --------
        "});
                    element
                        .derive(|value| &value.omitted_children)
                        .is_equal_to(1);
                },
            ]);
        }
    }
}
