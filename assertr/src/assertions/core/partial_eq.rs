use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::{
    AssertThat, Mode,
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    renderer::{DebugRenderer, RenderingOrder, ValueRenderer},
};

/// Implements [`Expectation`](crate::expectation::Expectation) for a check of the subject against
/// one stored operand.
///
/// The generics must name the renderer `R`. Evaluation borrows the operand stored in the `borrow`
/// field once, selecting its view for the `for` type, and retains that view on rejection. `holds`
/// states whether the relation between the subject and the view holds. An `expected` operand must
/// satisfy the relation and an `unexpected` operand must not. A requirement description states the
/// first relation and borrows the operand anew. A rejection renders the subject with `actual`,
/// states the second relation when one is given, and presents the retained view in its role.
macro_rules! operand_expectation {
    (
        impl [$($generics:tt)*] for $name:ty, subject $subject:ty, where [$($bounds:tt)*];
        borrow $field:tt for $target:ty, view $view:ty;
        kind $kind:ident;
        holds |$actual:ident, $expected:ident| $holds:expr;
        actual |$render:ident, $rendered:ident| $render_actual:expr;
        $role:ident $relation:literal $(, $rejection:literal)?;
    ) => {
        impl<$($generics)*> $crate::expectation::Expectation<$subject, R> for $name
        where
            $($bounds)*
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                $subject: 'a;
            type Rejection<'a>
                = &'a $view
            where
                Self: 'a,
                $subject: 'a;

            fn evaluate<'a>(
                &'a self,
                $actual: &'a $subject,
                _: &$crate::expectation::AssertionContext<'_, R>,
            ) -> Result<(), &'a $view> {
                let $expected = $crate::borrow_for::borrow_for::<$target, _>(&self.$field);
                operand_expectation!(@outcome $role, $holds, $expected)
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a $subject, &'a $view)>,
                failure: $crate::failure::FailureBuilder,
                context: &$crate::expectation::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                let $render = context.render();
                let (failure, operand) = match rejected {
                    None => (
                        failure.relation($relation),
                        $crate::borrow_for::borrow_for::<$target, _>(&self.$field),
                    ),
                    Some(($rendered, operand)) => (
                        failure.actual($render_actual)$(.relation($rejection))?,
                        operand,
                    ),
                };
                failure.$role($render.value(operand))
            }
        }
    };
    (@outcome expected, $holds:expr, $view:ident) => {
        if $holds { Ok(()) } else { Err($view) }
    };
    (@outcome unexpected, $holds:expr, $view:ident) => {
        if $holds { Err($view) } else { Ok(()) }
    };
}

pub(crate) use operand_expectation;

/// Reusable equality with an owned or borrowed expected value, selected through [`BorrowFor`].
///
/// [`PartialEqAssertions::is_equal_to`] executes this same definition.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::matchers::EqualTo;
///
/// let expected = EqualTo::new("hello");
/// assert_that!("hello").matches(&expected);
/// ```
#[derive(Debug, Clone)]
pub struct EqualTo<E>(E);

/// Matches values equal to `expected` through the actual value's `PartialEq` implementation.
///
/// This is a convenience constructor for [`EqualTo::new`].
///
/// ```
/// use assertr::{matchers::eq, prelude::*};
///
/// assert_that!([String::from("hello")]).matches(elements_are![eq("hello")]);
/// ```
#[must_use]
pub const fn eq<E>(expected: E) -> EqualTo<E> {
    EqualTo::new(expected)
}

impl<E> EqualTo<E> {
    /// Owns an expected operand. Pass a reference to reuse an expected value.
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
/// use assertr::matchers::NotEqualTo;
///
/// assert_that!(3).matches(NotEqualTo::new(4));
/// ```
#[derive(Debug, Clone)]
pub struct NotEqualTo<E>(E);

impl<E> NotEqualTo<E> {
    /// Owns the operand that must not equal the subject.
    #[must_use]
    pub const fn new(unexpected: E) -> Self {
        Self(unexpected)
    }
}

/// Matches a subject equal to one of the listed values: an array, slice, or vector whose
/// elements are compared like [`eq`]'s operand, so `&str` candidates match a `String` subject.
///
/// ```
/// use assertr::{matchers::one_of, prelude::*};
///
/// let remaining = vec!["Toast 1", "Toast 3"];
/// assert_that!(String::from("Toast 3")).matches(one_of(&remaining));
/// ```
pub struct IsOneOf<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(IsOneOf<E, B>, operand);

impl<E, B: AsRef<[E]>> IsOneOf<E, B> {
    /// Stores an array, slice, or vector of candidates without accessing their views.
    #[must_use]
    pub const fn new(candidates: B) -> Self {
        Self {
            expected: candidates,
            operand: PhantomData,
        }
    }
}

/// Matches a subject equal to one of `candidates`. This is a convenience constructor for
/// [`IsOneOf::new`].
#[must_use]
pub const fn one_of<E, B: AsRef<[E]>>(candidates: B) -> IsOneOf<E, B> {
    IsOneOf::new(candidates)
}

impl<T: ?Sized, E, B, R> Expectation<T, R> for IsOneOf<E, B>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    B: AsRef<[E]>,
    R: ValueRenderer<T> + ValueRenderer<E::View>,
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

    const KIND: FailureKind = FailureKind::Equality;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        let mut candidates = self.expected.as_ref().iter().map(borrow_for::<T, _>);
        if candidates.any(|candidate| actual.eq(candidate)) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let candidates = self
            .expected
            .as_ref()
            .iter()
            .map(borrow_for::<T, _>)
            .collect::<Vec<_>>();
        let candidates = context
            .render()
            .borrowed_values::<E::View, _>(&candidates, RenderingOrder::PreserveIteration);
        match rejected {
            None => failure.relation("is one of").expected(candidates),
            Some((actual, ())) => failure
                .actual(context.render().value(actual))
                .relation("is not one of")
                .expected(candidates),
        }
    }
}

operand_expectation! {
    impl [T: ?Sized, E, R] for EqualTo<E>, subject T, where [
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    ];
    borrow 0 for T, view E::View;
    kind Equality;
    holds |actual, expected| actual.eq(expected);
    actual |render, actual| render.value(actual);
    expected "is equal to";
}

operand_expectation! {
    impl [T: ?Sized, E, R] for NotEqualTo<E>, subject T, where [
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    ];
    borrow 0 for T, view E::View;
    kind Equality;
    holds |actual, expected| actual.eq(expected);
    actual |render, actual| render.value(actual);
    unexpected "is not equal to", "is equal to";
}

/// Equality and inequality assertions using [`PartialEq`].
///
/// Expected values can be owned or borrowed. [`BorrowFor`] selects their borrowed type,
/// allowing string literals to compare with `String` without allocation.
///
/// ```
/// use assertr::prelude::*;
/// let actual = String::from("hello");
/// let expected = String::from("hello");
/// assert_that!(actual).is_equal_to(&expected);
/// assert_that!(actual).is_equal_to("hello");
/// ```
///
/// Custom cross-type comparisons require a [`BorrowFor`] implementation or an explicit view,
/// predicate, or custom expectation. `PartialEq` alone is insufficient:
///
/// ```compile_fail
/// use assertr::prelude::*;
/// #[derive(Debug)]
/// struct Length(usize);
/// impl PartialEq<usize> for Length {
///     fn eq(&self, other: &usize) -> bool { self.0 == *other }
/// }
/// assert_that!(Length(5)).is_equal_to(5_usize);
/// ```
///
/// ```
/// use assertr::{prelude::*, matchers::predicate};
/// #[derive(Debug)]
/// struct Length(usize);
/// assert_that!(Length(5)).matches(predicate(|value: &Length| value.0 == 5));
/// ```
///
/// Reference-valued subjects keep their declared type. Use [`crate::matchers::dereferenced`]
/// to compare their pointees:
///
/// ```
/// use assertr::{prelude::*, matchers::{dereferenced, eq}};
/// let value = String::from("hello");
/// assert_that_owned!(&value).matches(dereferenced(eq("hello")));
/// ```
///
/// ```compile_fail
/// use assertr::{prelude::*, matchers::eq};
/// let value = String::from("hello");
/// assert_that_owned!(&value).matches(eq(String::from("hello")));
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PartialEqAssertions<T, R = DebugRenderer> {
    /// Asserts that the subject equals the value borrowed from `expected`.
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that the subject does not equal the value borrowed from `expected`.
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that the subject equals one of `candidates`, an array, slice, or vector whose
    /// elements are borrowed like [`is_equal_to`](Self::is_equal_to)'s operand.
    fn is_one_of<E>(self, candidates: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;
}

impl<T, M: Mode, R> PartialEqAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        self.matches(EqualTo::new(expected))
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        self.matches(NotEqualTo::new(expected))
    }

    #[track_caller]
    fn is_one_of<E>(self, candidates: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        self.matches(IsOneOf::new(candidates))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            "foo"
                .must()
                .be_equal_to("foo")
                .not_be_equal_to("bar")
                .be_one_of(["bar", "foo"]);
        }
    }

    mod is_one_of {
        use indoc::formatdoc;

        use crate::{matchers::one_of, prelude::*};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1), is_one_of([2, 3]));
        }

        #[test]
        fn succeeds_for_any_listed_candidate_borrowing_each() {
            assert_that!(String::from("b")).is_one_of(["a", "b"]);
            assert_that!(String::from("b")).matches(one_of(vec!["b"]));
            let candidates = [String::from("a"), String::from("b")];
            assert_that!("a").is_one_of(&candidates[..]);
        }

        #[test]
        fn panics_with_the_candidates() {
            assert_that!(|| {
                assert_that!(String::from("c"))
                    .with_location(false)
                    .is_one_of(["a", "b"]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `String::from("c")`

                Actual: "c"

                is not one of

                Expected: [
                    "a",
                    "b",
                ]
                -------- assertr --------
            "#});
        }
    }

    mod diagnostics {
        use core::cell::RefCell;

        use super::super::{EqualTo, NotEqualTo};
        use crate::{
            failure::{FailureBuilder, FailureKind},
            prelude::*,
        };

        struct RecordingRenderer<'a>(&'a RefCell<Vec<i32>>);
        impl ValueRenderer<i32> for RecordingRenderer<'_> {
            fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.borrow_mut().push(*value);
                write!(f, "{value}")
            }
        }

        #[test]
        fn shared_operands_render_once_and_follow_the_actual_value() {
            let calls = RefCell::new(Vec::new());
            let failures = assert_that!(1)
                .with_renderer(RecordingRenderer(&calls))
                .capture(|it| it.matches(EqualTo::new(2)));
            assert_that!(failures).has_length(1);
            assert_that!(*calls.borrow()).contains_exactly([1, 2]);
            calls.borrow_mut().clear();
            let renderer = RecordingRenderer(&calls);
            let context = AssertionContext::new(&renderer, RenderingBudget::default());
            let description = <NotEqualTo<i32> as Expectation<i32, _>>::explain(
                &NotEqualTo::new(2),
                None,
                FailureBuilder::new::<i32>(FailureKind::Equality),
                &context,
            )
            .build();
            assert_that!(*calls.borrow()).contains_exactly([2]);
            assert_that!(description.expected).is_none();
            assert_that!(description.unexpected)
                .is_equal_to(Some(AssertionContext::default().render().value(&2)));
        }

        #[test]
        fn missing_elements_keep_the_negative_operand_role() {
            let failures = assert_that!([] as [i32; 0])
                .with_location(false)
                .capture(|it| {
                    it.matches(crate::assertions::collection::elements_are(matchers![
                        NotEqualTo::new(3)
                    ]))
                });
            let description = failures[0].children[0].constraint.as_ref().unwrap();
            assert_that!(description.expected).is_none();
            assert_that!(description.unexpected)
                .is_equal_to(Some(AssertionContext::default().render().value(&3)));
            assert_that!(failures[0].to_string()).contains("Unexpected: 3");
        }
    }

    mod renderer_contract {
        use core::borrow::Borrow;

        use crate::{
            borrow_for::BorrowFor,
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        #[derive(PartialEq)]
        struct Actual(u32);
        struct Expected(Actual);

        impl BorrowFor<Actual> for Expected {
            type View = Actual;
        }

        impl Borrow<Actual> for Expected {
            fn borrow(&self) -> &Actual {
                &self.0
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
        fn borrowed_operands_render_the_target_with_the_active_renderer() {
            let failures = assert_that!(Actual(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_equal_to(Expected(Actual(2))));

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod is_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn byte_slices_compare_with_byte_string_literals() {
            let actual: &[u8] = b"hello";
            assert_that!(actual).is_equal_to(b"hello");
        }

        #[test]
        fn cow_strings_compare_with_string_literals() {
            use alloc::borrow::Cow;

            assert_that!(Cow::<str>::Borrowed("hello")).is_equal_to("hello");
            assert_that!(Cow::<str>::Owned(String::from("hello"))).is_equal_to("hello");
        }

        #[test]
        fn panics_when_not_equal() {
            assert_that!(|| assert_that!("foo").with_location(false).is_equal_to("bar"))
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo"`

                    Expected: "bar"

                      Actual: "foo"
                    -------- assertr --------
                "#});
        }
    }

    mod is_not_equal_to {
        use indoc::formatdoc;

        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!("foo")
                    .with_location(false)
                    .is_not_equal_to("foo")
            })
            .panics()
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
        fn negates_eq_even_when_ne_is_overridden() {
            #[derive(Debug)]
            struct Unusual;
            #[allow(clippy::partialeq_ne_impl)]
            impl PartialEq for Unusual {
                fn eq(&self, _: &Self) -> bool {
                    false
                }
                fn ne(&self, _: &Self) -> bool {
                    false
                }
            }
            assert_that!(Unusual).is_not_equal_to(&Unusual);
        }
    }

    // `operand_expectation!` is shared by the equality, ordering, range, and map value operands.
    // These tests cover its borrowing contract once.
    mod borrowed_operands {
        use core::cell::Cell;

        use super::super::{EqualTo, NotEqualTo};
        use crate::{prelude::*, test_support::BorrowSpy};

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }
        fn point() -> Point {
            Point { x: 1, y: 2 }
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed temporaries are the contract under test.
        fn accepts_all_value_reference_forms_and_reusable_matchers() {
            let actual = point();
            let expected = point();
            assert_that!(point()).is_equal_to(point());
            assert_that!(&actual).is_equal_to(point());
            assert_that!(point()).is_equal_to(&expected);
            assert_that!(&actual).is_equal_to(&expected);
            assert_that!(&actual).is_equal_to(&point());
            let matcher = EqualTo::new(&expected);
            assert_that!(point()).matches(&matcher).matches(&matcher);
            assert_that!(&actual).matches(&matcher);
        }

        #[test]
        fn borrows_once_after_tracking_and_retains_failure_evidence() {
            for negative in [false, true] {
                for actual in [1, 2] {
                    let calls = Cell::new(0);
                    let failures = assert_that!(()).capture(|root| {
                        let expected = BorrowSpy {
                            value: 2,
                            observe: || {
                                assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                                calls.set(calls.get() + 1);
                            },
                        };
                        let it = root.derive_owned(|()| actual);
                        if negative {
                            it.is_not_equal_to(expected);
                        } else {
                            it.is_equal_to(expected);
                        }
                        root
                    });
                    assert_that!(calls.get()).is_equal_to(1);
                    assert_that!(failures).has_length(usize::from(negative == (actual == 2)));
                }
            }
        }

        #[test]
        fn selects_sequence_and_string_views_for_the_declared_subject() {
            let context = AssertionContext::default();
            assert_that!(vec![String::from("hello")]).is_equal_to(["hello"]);
            assert_that!([1, 2].as_slice()).is_equal_to(vec![1, 2]);
            assert_that!(String::from("hello"))
                .is_equal_to("hello")
                .is_not_equal_to("world");
            assert_that!("hello").is_equal_to(String::from("hello"));
            assert_that!(EqualTo::new("hello").evaluate("hello", &context).is_ok()).is_true();
            assert_that!(NotEqualTo::new("world").evaluate("hello", &context).is_ok()).is_true();
            let value = point();
            assert_that_owned!(&value).matches(EqualTo::new(&value));
            assert_that_owned!(&value).matches(matchers::dereferenced(EqualTo::new(point())));
        }
    }
}
