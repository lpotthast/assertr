use crate::borrow_for::{BorrowFor, borrow_for};

use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};

/// Reusable equality with an owned or borrowed expected value, selected through [`BorrowFor`].
///
/// [`PartialEqAssertions::is_equal_to`] executes this same definition.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_eq::EqualTo;
///
/// let expected = EqualTo::new("hello");
/// assert_that!("hello").matches(&expected);
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

impl<T: ?Sized, E, R> Expectation<T, R> for EqualTo<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = borrow_for::<T, _>(&self.0);
        if actual.eq(expected) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}
impl<T: ?Sized, E, R> ExpectationDiagnostics<T, R> for EqualTo<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (failure.relation("is equal to"), borrow_for::<T, _>(&self.0)),
            Some((actual, expected)) => (failure.actual(render.value(actual)), expected),
        };
        failure.expected(render.value(expected))
    }
}
impl<T: ?Sized, E, R> Expectation<T, R> for NotEqualTo<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = borrow_for::<T, _>(&self.0);
        if actual.eq(expected) {
            Err(expected)
        } else {
            Ok(())
        }
    }
}
impl<T: ?Sized, E, R> ExpectationDiagnostics<T, R> for NotEqualTo<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("is not equal to"),
                borrow_for::<T, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure.actual(render.value(actual)).relation("is equal to"),
                expected,
            ),
        };
        failure.unexpected(render.value(expected))
    }
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
pub trait PartialEqAssertions<T, R> {
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
}

impl<T, M: Mode, R> PartialEqAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        self.apply_assertion(EqualTo::new(expected))
    }

    #[track_caller]
    fn is_not_equal_to<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
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
            borrow_for::BorrowFor,
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };
        use core::borrow::Borrow;

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
        fn string_vectors_compare_with_string_literal_arrays() {
            assert_that!(vec![String::from("hello")]).is_equal_to(["hello"]);
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
        fn custom_heterogeneous_comparisons_use_predicates() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    true
                }
            }

            assert_that!(Foo {}).matches(crate::expectation::predicate(|actual: &Foo| {
                actual.eq(&Bar {})
            }));
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
        fn custom_heterogeneous_comparisons_use_predicates() {
            #[derive(Debug)]
            struct Foo {}

            #[derive(Debug)]
            struct Bar {}

            impl PartialEq<Bar> for Foo {
                fn eq(&self, _other: &Bar) -> bool {
                    false
                }
            }

            assert_that!(Foo {}).matches(crate::expectation::predicate(|actual: &Foo| {
                !actual.eq(&Bar {})
            }));
        }
    }

    mod borrowed_operands {
        use super::super::{EqualTo, NotEqualTo};
        use crate::{
            prelude::*,
            test_support::{BorrowSpy, NoRenderer, assert_trait_impl},
        };
        use core::cell::Cell;

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
            assert_that!(point()).matches(&matcher);
            assert_that!(&actual).matches(&matcher);
            assert_that!(expected).is_equal_to(point());
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
        fn direct_unsized_targets_and_declared_references() {
            let context = AssertionContext::default();
            assert_that!(EqualTo::new("hello").evaluate("hello", &context).is_ok()).is_true();
            assert_that!(NotEqualTo::new("world").evaluate("hello", &context).is_ok()).is_true();
            assert_that!(
                EqualTo::new(vec![1, 2])
                    .evaluate([1, 2].as_slice(), &context)
                    .is_ok()
            )
            .is_true();
            let value = point();
            assert_that_owned!(&value).matches(EqualTo::new(&value));
            assert_that_owned!(&value).matches(matchers::dereferenced(EqualTo::new(point())));
            assert_trait_impl!(EqualTo<String> => Expectation<str, NoRenderer>);
            assert_trait_impl!(EqualTo<Vec<i32>> => Expectation<[i32], NoRenderer>);
        }

        #[test]
        fn inequality_negates_eq_even_when_ne_is_overridden() {
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
            assert_that!(Unusual)
                .is_not_equal_to(&Unusual)
                .matches(NotEqualTo::new(&Unusual));
        }

        #[test]
        fn operand_wrappers_need_no_renderer() {
            #[derive(PartialEq)]
            struct Value(i32);
            struct Renderer;
            impl ValueRenderer<Value> for Renderer {
                fn fmt(
                    &self,
                    value: &Value,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "value({})", value.0)
                }
            }
            let calls = Cell::new(0);
            let failures = assert_that!(Value(1))
                .with_renderer(Renderer)
                .capture(|it| {
                    it.is_equal_to(BorrowSpy {
                        value: Value(2),
                        observe: || calls.set(calls.get() + 1),
                    })
                });
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(ToHumanReadableText.render(&failures[0])).contains("value(2)");
        }
    }
}

#[cfg(test)]
mod sequence_views {
    use super::{EqualTo, NotEqualTo};
    use crate::{
        prelude::*,
        test_support::{FailureReportAssertions, NoRenderer, assert_trait_impl},
    };
    use indoc::formatdoc;

    #[derive(Debug, PartialEq)]
    struct Element(u8);

    #[test]
    #[allow(clippy::needless_borrows_for_generic_args)] // Both operand forms are the contract under test.
    fn vectors_accept_owned_and_borrowed_arrays_without_cloning_elements() {
        let actual = vec![Element(1), Element(2)];
        let expected = [Element(1), Element(2)];
        assert_that!(actual)
            .is_equal_to([Element(1), Element(2)])
            .is_equal_to(&expected)
            .is_not_equal_to([Element(1)])
            .is_not_equal_to([Element(1), Element(3)]);
        let matcher = EqualTo::new(expected);
        assert_that!(actual).matches(&matcher).matches(&matcher);
        assert_trait_impl!(EqualTo<[Element; 2]> => Expectation<Vec<Element>, NoRenderer>);
        assert_trait_impl!(NotEqualTo<[Element; 2]> => Expectation<Vec<Element>, NoRenderer>);
    }

    #[test]
    fn slice_subjects_accept_owned_and_borrowed_vectors_without_cloning_elements() {
        let actual = [Element(1), Element(2)];
        let expected = vec![Element(1), Element(2)];
        let slice = actual.as_slice();
        assert_that!(slice)
            .is_equal_to(vec![Element(1), Element(2)])
            .is_equal_to(&expected)
            .is_not_equal_to(vec![Element(1)])
            .is_not_equal_to(vec![Element(1), Element(3)]);
        let matcher = EqualTo::new(&expected);
        assert_that!(slice).matches(&matcher).matches(&matcher);
        assert_that_owned!(slice).matches(&matcher);
        assert_that!(
            matcher
                .evaluate(slice, &AssertionContext::default())
                .is_ok()
        )
        .is_true();
        assert_trait_impl!(EqualTo<Vec<Element>> => Expectation<&'static [Element], NoRenderer>);
        assert_trait_impl!(EqualTo<&Vec<Element>> => Expectation<&'static [Element], NoRenderer>);
        assert_trait_impl!(EqualTo<&Vec<Element>> => Expectation<[Element], NoRenderer>);
        assert_trait_impl!(NotEqualTo<Vec<Element>> => Expectation<&'static [Element], NoRenderer>);
        assert_trait_impl!(NotEqualTo<&Vec<Element>> => Expectation<&'static [Element], NoRenderer>);
        assert_trait_impl!(NotEqualTo<&Vec<Element>> => Expectation<[Element], NoRenderer>);
    }

    #[test]
    fn sequence_views_preserve_equality_diagnostics() {
        let from_array = assert_that!(vec![1, 2])
            .with_expression("sequence")
            .with_location(false)
            .capture(|it| it.is_equal_to([1, 3]));
        let expected = vec![1, 3];
        let from_vector = assert_that!([1, 2].as_slice())
            .with_expression("sequence")
            .with_location(false)
            .capture(|it| it.is_equal_to(&expected));
        for failures in [from_array, from_vector] {
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `sequence`

                Expected: [
                    1,
                    3,
                ]

                  Actual: [
                    1,
                    2,
                ]
                -------- assertr --------
            "});
        }
    }
}

#[cfg(test)]
mod string_views {
    use super::{EqualTo, NotEqualTo};
    use crate::{
        prelude::*,
        test_support::{NoRenderer, StrOperand, StringRenderer, assert_trait_impl},
    };
    use core::cell::Cell;

    #[test]
    #[allow(clippy::needless_borrows_for_generic_args)]
    fn string_value_reference_matrix_and_reusable_matchers() {
        let actual = String::from("hello");
        let expected = String::from("hello");
        assert_that!(actual).is_equal_to(String::from("hello"));
        assert_that!(&actual).is_equal_to(String::from("hello"));
        assert_that!(actual).is_equal_to(&expected);
        assert_that!(&actual).is_equal_to(&expected);
        assert_that!(actual)
            .is_equal_to("hello")
            .is_equal_to(&"hello");
        assert_that!(&actual).is_equal_to("hello");
        assert_that!("hello")
            .is_equal_to(String::from("hello"))
            .is_equal_to(&expected);
        let borrowed = EqualTo::new(&expected);
        let literal = EqualTo::new("hello");
        assert_that!(actual).matches(&borrowed).matches(&literal);
        assert_that!("hello").matches(&borrowed).matches(&literal);
        let context = AssertionContext::default();
        assert_that!(literal.evaluate("hello", &context).is_ok()).is_true();
        assert_that!(borrowed.evaluate("hello", &context).is_ok()).is_true();
        assert_trait_impl!(EqualTo<&str> => crate::Expectation<String, NoRenderer>);
        assert_trait_impl!(EqualTo<&String> => crate::Expectation<str, NoRenderer>);
    }

    #[test]
    fn unsized_view_is_borrowed_once_after_tracking_and_rendered_on_both_rejections() {
        for negative in [false, true] {
            let calls = Cell::new(0);
            let failures = assert_that!(String::from("hello"))
                .with_renderer(StringRenderer)
                .capture(|root| {
                    let operand = StrOperand {
                        value: if negative { "hello" } else { "world" },
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    if negative {
                        root.derive(|it| it).is_not_equal_to(operand);
                    } else {
                        root.derive(|it| it).is_equal_to(operand);
                    }
                    root
                });
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(failures).has_length(1);
        }
        assert_that!(String::from("hello")).matches(NotEqualTo::new("world"));
    }
}
