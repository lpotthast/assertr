use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};

/// A reusable equality assertion using the subject's heterogeneous `PartialEq` implementation.
///
/// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
/// [`AssertionContext`]. [`PartialEqAssertions::is_equal_to`] executes this same definition on an
/// assertion chain.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_eq::EqualTo;
///
/// let expected = EqualTo::new("hello");
/// assert_that!(String::from("hello")).matches(&expected);
/// ```
pub struct EqualTo<E>(E);

/// Matches through the actual value's ordinary `PartialEq` implementation.
///
/// This is a convenience constructor for [`EqualTo::new`].
pub fn equal_to<E>(expected: E) -> EqualTo<E> {
    EqualTo::new(expected)
}

/// Short alias for [`equal_to`].
///
/// ```
/// use assertr::{matchers::eq, prelude::*};
///
/// assert_that!([String::from("hello")]).matches(elements_are![eq("hello")]);
/// ```
pub use equal_to as eq;

impl<E> EqualTo<E> {
    /// Owns an expected operand. Pass a reference when the subject implements equality with it.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

/// An explicit inequality assertion, with evidence identifying the unexpectedly equal value.
///
/// This definition uses `PartialEq::eq`, like [`PartialEqAssertions::is_not_equal_to`]. It does
/// not depend on an independently overridden `PartialEq::ne` or a generic negation adapter.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_eq::NotEqualTo;
///
/// assert_that!(3).matches(NotEqualTo::new(4));
/// ```
pub struct NotEqualTo<E>(E);

impl<E> NotEqualTo<E> {
    /// Owns the operand that must not equal the subject.
    #[must_use]
    pub const fn new(unexpected: E) -> Self {
        Self(unexpected)
    }
}

// Borrowed expected operands let streaming adapters use the same equality definition without
// cloning values or requiring `T: PartialEq<&E>` for heterogeneous comparisons.
pub(crate) struct EqualToRef<'e, E>(pub(crate) &'e E);

impl<T: ?Sized, E, R> Expectation<T, R> for EqualTo<E>
where
    T: PartialEq<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        T: 'a;

    fn evaluate(&self, actual: &T, context: &AssertionContext<'_, R>) -> Result<(), ()> {
        EqualToRef(&self.0).evaluate(actual, context)
    }
}

impl<T: ?Sized, E, R> Expectation<T, R> for EqualToRef<'_, E>
where
    T: PartialEq<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        T: 'a;

    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.eq(self.0) { Ok(()) } else { Err(()) }
    }
}

impl<T: ?Sized, E, R> Expectation<T, R> for NotEqualTo<E>
where
    T: PartialEq<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        T: 'a;

    fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.eq(&self.0) { Err(()) } else { Ok(()) }
    }
}

impl<T: ?Sized, E, R> ExpectationDiagnostics<T, R> for EqualTo<E>
where
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        EqualToRef(&self.0).explain(rejected, failure, context)
    }
}

impl<T: ?Sized, E, R> ExpectationDiagnostics<T, R> for EqualToRef<'_, E>
where
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is equal to"),
            Some((actual, ())) => failure.actual(render.value(actual)),
        };
        failure.expected(render.value(self.0))
    }
}

impl<T: ?Sized, E, R> ExpectationDiagnostics<T, R> for NotEqualTo<E>
where
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is not equal to"),
            Some((actual, ())) => failure.actual(render.value(actual)).relation("is equal to"),
        };
        failure.unexpected(render.value(&self.0))
    }
}

/// Equality and inequality assertions using [`PartialEq`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PartialEqAssertions<T, R> {
    /// Asserts that the subject equals `expected`.
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that the subject does not equal `expected`.
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;
}

impl<T, M: Mode, R> PartialEqAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.apply_assertion(EqualTo::new(expected))
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.apply_assertion(NotEqualTo::new(expected))
    }
}

#[cfg(test)]
mod tests {
    mod diagnostics {
        use super::super::{EqualTo, NotEqualTo};
        use crate::{
            failure::{FailureBuilder, FailureKind},
            prelude::*,
            renderer::IntoRendered,
        };
        use core::cell::RefCell;

        struct RecordingRenderer<'a>(&'a RefCell<alloc::vec::Vec<i32>>);
        impl ValueRenderer<i32> for RecordingRenderer<'_> {
            fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.borrow_mut().push(*value);
                write!(f, "{value}")
            }
        }

        #[test]
        fn shared_operands_render_once_and_follow_the_actual_value() {
            let calls = RefCell::new(alloc::vec::Vec::new());
            let failures = assert_that!(1)
                .with_renderer(RecordingRenderer(&calls))
                .capture(|it| it.apply_assertion(EqualTo::new(2)));
            assert_that!(failures).has_length(1);
            assert_that!(*calls.borrow()).contains_exactly([1, 2]);
            calls.borrow_mut().clear();
            let renderer = RecordingRenderer(&calls);
            let context = AssertionContext::new(&renderer, RenderingBudget::default());
            let description = <NotEqualTo<i32> as ExpectationDiagnostics<i32, _>>::explain(
                &NotEqualTo::new(2),
                None,
                FailureBuilder::detached::<i32>(FailureKind::Equality),
                &context,
            )
            .build();
            assert_that!(*calls.borrow()).contains_exactly([2]);
            assert_that!(description.expected).is_none();
            assert_that!(description.unexpected).is_equal_to(Some(
                AssertionContext::default()
                    .render()
                    .value(&2)
                    .into_rendered(),
            ));
        }

        #[test]
        fn missing_elements_keep_the_negative_operand_role() {
            let failures = assert_that!([] as [i32; 0])
                .with_location(false)
                .capture(|it| {
                    it.matches(crate::assertions::collection::elements_are((
                        NotEqualTo::new(3),
                    )))
                });
            let description = failures[0].children[0].constraint.as_ref().unwrap();
            assert_that!(description.expected).is_none();
            assert_that!(description.unexpected).is_equal_to(Some(
                AssertionContext::default()
                    .render()
                    .value(&3)
                    .into_rendered(),
            ));
            assert_that!(ToHumanReadableText.render(&failures[0])).contains("Unexpected: 3");
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        struct Actual(u32);
        struct Expected(u32);

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PartialEqAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn heterogeneous_failures_render_both_types_with_the_active_renderer() {
            let failures = assert_that!(Actual(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_equal_to(Expected(2)));

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

    mod is_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo".must().be_equal_to("foo");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo"), is_equal_to("bar"));
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!("foo").is_equal_to("foo");
            assert_that!("foo".to_string()).is_equal_to("foo".to_string());
            assert_that!("foo".to_string()).is_equal_to("foo");
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that_panic_by(|| assert_that!("foo").with_location(false).is_equal_to("bar"))
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo"`

                    Expected: "bar"

                      Actual: "foo"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn accepts_expected_being_of_different_type() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    true
                }
            }

            assert_that!(Foo {}).is_equal_to(Bar {});
        }
    }

    mod is_not_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo".must().not_be_equal_to("bar");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo"), is_not_equal_to("foo"));
        }

        #[test]
        fn succeeds_when_not_equal() {
            assert_that!("foo").is_not_equal_to("bar");
            assert_that!(f64::NAN).is_not_equal_to(f64::NAN);
        }

        #[test]
        fn panics_when_equal() {
            assert_that_panic_by(|| {
                assert_that!("foo")
                    .with_location(false)
                    .is_not_equal_to("foo")
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo"`

                    Actual: "foo"

                    is equal to

                    Unexpected: "foo"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn accepts_expected_being_of_different_type() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    false
                }
            }

            assert_that!(Foo {}).is_not_equal_to(Bar {});
        }
    }
}
