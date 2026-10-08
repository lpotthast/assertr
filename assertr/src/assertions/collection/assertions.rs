use crate::borrow_for::BorrowFor;
use core::borrow::Borrow;

use super::{
    Collection, ContainsMatching, DoesNotContainMatching, elements_are_in_any_order, identity,
    value,
};
use crate::{
    AssertThat, Mode, expectation::Expectation, mode::Capture, renderer::DebugRenderer,
    renderer::ValueRenderer,
};
use crate::{expectation::lists::SatisfyingList, matchers::satisfying};

/// Assertions over the elements of a collection: slices, arrays, `Vec`, `VecDeque`, and every type
/// implementing [`Collection`].
///
/// Expected values can be owned or borrowed through [`BorrowFor<Collection::Item>`](BorrowFor).
/// Reference-valued items keep their declared type. Use [`crate::matchers::dereferenced`] to match
/// their pointees. Empty expected lists may need an explicit element type:
///
/// ```
/// use assertr::prelude::*;
/// assert_that!([] as [String; 0]).contains_exactly_in_any_order([] as [String; 0]);
/// ```
///
/// The collection structure is rendered by Assertr, so value-based methods require rendering
/// support for the element type rather than the collection type. Identity methods display addresses
/// and require no rendering support.
///
/// For a type that supports borrowed traversal but does not implement [`Collection`], use
/// [`IntoIteratorAssertions`](crate::assertions::IntoIteratorAssertions). Its methods
/// carry the `into_iter_` prefix.
///
/// Bulk value lists use [repeatable expected data](crate#expected-lists).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CollectionAssertions<T, R = DebugRenderer> {
    /// Asserts that at least one element borrows the same instance as `expected`.
    ///
    /// Compares `Borrow<U>` targets with [`core::ptr::eq`], without equality or rendering bounds.
    /// Stored values, references, and smart pointers are supported. The expected reference's type
    /// selects the borrowed view. For unsized targets, both address and metadata must match.
    /// Trait-object equality inherits `ptr::eq`'s vtable caveats, and distinct zero-sized values
    /// can have equal addresses. Identity is not a unique allocation or logical object ID.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// struct Key { _opaque: u8 }
    /// let keys = [Key { _opaque: 1 }, Key { _opaque: 1 }];
    /// assert_that!([&keys[0]])
    ///     .contains_same_instance_as(&keys[0])
    ///     .does_not_contain_same_instance_as(&keys[1]);
    /// ```
    fn contains_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        T: Borrow<U>;

    /// Asserts that no element borrows the same instance as `expected`.
    ///
    /// Uses the borrowed-target identity semantics of
    /// [`contains_same_instance_as`](Self::contains_same_instance_as), without rendering bounds.
    fn does_not_contain_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        T: Borrow<U>;

    /// Asserts that the collection borrows exactly the expected instances, ignoring order.
    ///
    /// Every expected occurrence must match a distinct actual occurrence, so duplicate counts must
    /// match. Uses the borrowed-target identity semantics of
    /// [`contains_same_instance_as`](Self::contains_same_instance_as), without rendering bounds.
    /// Arrays, slices, and vectors of expected references are accepted. An empty expectation may
    /// need a type annotation, such as `[] as [&Key; 0]`, to select the borrowed target.
    fn contains_exactly_same_instances_in_any_order<'e, U: ?Sized + 'e>(
        self,
        expected: impl AsRef<[&'e U]>,
    ) -> Self
    where
        T: Borrow<U>;

    /// Asserts that at least one element equals `expected`.
    fn contains<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that at least one element matches `expected`.
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>;

    /// Asserts that at least one element satisfies `assertions`.
    ///
    /// The assertions run in capture mode against each element. An element matches when no
    /// assertion failure is raised. On failure, every element's captured failures are reported.
    /// Only the callback's assertions need rendering support. The element itself need not be
    /// renderable.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Asserts that every expected element has an equal element in the subject.
    ///
    /// Extra subject elements are allowed. Expectations are independent, so duplicates do not
    /// require distinct matches. Use `contains_exactly_in_any_order` for multiset equality.
    ///
    /// Each expected element `E` selects a borrowed view through [`BorrowFor`] for the declared
    /// collection item type `T`. Arrays, slices, and vectors reuse their storage. Prepare
    /// generators explicitly with `.contains_all(generator.collect::<Vec<_>>())`.
    fn contains_all<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that no element equals `not_expected`.
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that no element matches `expected`.
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>,
        R: ValueRenderer<T>;

    /// Asserts that no element satisfies `assertions`.
    ///
    /// The assertions run in capture mode against each element. An element matches when no
    /// assertion failure is raised. On failure, the matching elements are reported.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Asserts multiset equality with `expected`.
    ///
    /// Order is ignored, but each expected element must match a distinct subject element, so
    /// duplicate counts must match. [`PartialEq`] permits different element types.
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts one-to-one matching between subject elements and matchers, independent of order.
    ///
    /// A maximum matching makes overlapping matchers order-independent.
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts one-to-one matching between subject elements and assertion closures, independent of
    /// order. A maximum matching makes overlapping assertions order-independent.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: Clone + ValueRenderer<usize>,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);
}

impl<C, M, R> CollectionAssertions<C::Item, R> for AssertThat<'_, C, M, R>
where
    C: Collection,
    M: Mode,
{
    #[track_caller]
    fn contains_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.matches(identity::ContainsSameInstanceAs::new(expected))
    }

    #[track_caller]
    fn does_not_contain_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.matches(identity::DoesNotContainSameInstanceAs::new(expected))
    }

    #[track_caller]
    fn contains_exactly_same_instances_in_any_order<'e, U: ?Sized + 'e>(
        self,
        expected: impl AsRef<[&'e U]>,
    ) -> Self
    where
        C::Item: Borrow<U>,
    {
        self.matches(identity::ContainsExactlySameInstancesInAnyOrder::new(
            expected,
        ))
    }

    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        C::Item: PartialEq<E::View>,
        E: BorrowFor<C::Item>,
        R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
    {
        self.matches(value::Contains::new(expected))
    }

    #[track_caller]
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<C::Item, R>,
    {
        self.matches(ContainsMatching::new(expected))
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.contains_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_all<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: PartialEq<E::View>,
        E: BorrowFor<C::Item>,
        R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
    {
        self.matches(value::ContainsAll::new(expected))
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        C::Item: PartialEq<E::View>,
        E: BorrowFor<C::Item>,
        R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
    {
        self.matches(value::DoesNotContain::new(not_expected))
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<C::Item, R>,
        R: ValueRenderer<C::Item>,
    {
        self.matches(DoesNotContainMatching::new(expected))
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.does_not_contain_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: PartialEq<E::View>,
        E: BorrowFor<C::Item>,
        R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
    {
        self.matches(value::ContainsExactlyInAnyOrder::new(expected))
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::MatcherList<C::Item, R>,
        R: ValueRenderer<usize>,
    {
        self.matches(elements_are_in_any_order(expected))
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: Clone + ValueRenderer<usize>,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.matches(elements_are_in_any_order(SatisfyingList::new(assertions)))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::{matchers::eq, prelude::*};

        fn is(expected: i32) -> impl Fn(AssertThat<'_, i32, Capture>) {
            move |it| {
                it.is_equal_to(expected);
            }
        }

        #[test]
        fn are_as_expected() {
            let values = [1, 2, 3];
            let values = values.as_slice();
            values.must().contain(2);
            values.must().contain_matching(eq(2));
            values.must().contain_satisfying(is(2));
            values.must().contain_all([1, 3]);
            values.must().not_contain(4);
            values.must().not_contain_matching(eq(4));
            values.must().not_contain_satisfying(is(4));
            values.must().contain_exactly_in_any_order([2, 3, 1]);
            values
                .must()
                .contain_exactly_in_any_order_matching([eq(2), eq(3), eq(1)]);
            values
                .must()
                .contain_exactly_in_any_order_satisfying([is(2), is(3), is(1)]);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{
                ComparisonRenderer, NoRenderer, RendererActual, RendererExpected, SENTINEL,
                assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => CollectionAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn equality_and_failures_use_the_active_renderer_type() {
            assert_that!([RendererActual(1), RendererActual(2)].as_slice())
                .with_renderer(ComparisonRenderer)
                .contains(RendererExpected::new(2))
                .contains_all([RendererExpected::new(1)])
                .contains_exactly_in_any_order([
                    RendererExpected::new(2),
                    RendererExpected::new(1),
                ]);

            let failures = assert_that!([RendererActual(1)].as_slice())
                .with_renderer(ComparisonRenderer)
                .with_location(false)
                .capture(|it| it.contains(RendererExpected::new(2)));
            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2, 3].as_slice()), contains(4));
        }

        #[test]
        fn succeeds_when_value_is_present() {
            assert_that!([1, 2, 3].as_slice()).contains(2);
        }

        #[test]
        fn compiles_for_owned_and_borrowed_elements() {
            let expected = String::from("foo");
            assert_that!([String::from("foo")]).contains(&expected);
            assert_that!(["foo"].as_slice()).contains("foo");
        }

        #[test]
        fn preserves_the_chain_subject_name() {
            let failures = assert_that!(vec![1, 2, 3])
                .with_subject_name("the elements")
                .capture(|it| it.contains(4));

            assert_that!(&failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.subject_name.as_deref())
                        .is_equal_to(Some("the elements"));
                },
            ]);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains(4);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain

                    Expected: 4
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use crate::{prelude::*, test_support::NoRenderer};
        use core::cell::Cell;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                contains_matching(matchers::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            let calls = Cell::new(0);
            assert_that!([1, 2, 3].as_slice()).contains_matching(matchers::predicate(
                |it: &i32| {
                    calls.set(calls.get() + 1);
                    *it % 2 == 0
                },
            ));
            assert_that!(calls.get()).is_equal_to(2);
        }

        #[test]
        fn empty_collection_retains_the_constraint_without_a_renderer() {
            let failures = assert_that!([] as [i32; 0])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_matching(matchers::anything()));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(|failure| {
                            failure
                                .constraint
                                .as_ref()
                                .unwrap()
                                .relation
                                .as_deref()
                                .unwrap()
                        })
                        .is_equal_to("contains a matching element");
                },
            ]);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_matching(matchers::predicate(|it: &i32| *it > 7));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not contain a matching element

                Nested failures:
                  - Actual: 1

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate

                  - Actual: 2

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate

                  - Actual: 3

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2].as_slice()),
                contains_satisfying(|it| {
                    it.is_equal_to(7);
                })
            );
        }
    }

    mod contains_all {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2].as_slice()), contains_all([1, 42]));
        }

        #[test]
        fn succeeds_when_all_expected_values_are_present() {
            assert_that!([1, 2, 3].as_slice()).contains_all([1, 3]);
        }

        #[test]
        fn succeeds_with_vec_input() {
            assert_that!([1, 2, 3].as_slice()).contains_all(vec![1, 3]);
        }

        #[test]
        fn ignores_duplicate_expectations_and_extra_actual_elements() {
            assert_that!([1, 2, 3].as_slice()).contains_all([1, 1, 1]);
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that!(["foo"].as_slice()).contains_all(["foo"]);
            assert_that!(["foo".to_owned()].as_slice()).contains_all(["foo"]);
        }

        #[test]
        fn panics_when_any_expected_value_is_absent() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_all([1, 42]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2].as_slice()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not contain all of

                    Expected: [
                        1,
                        42,
                    ]

                    Details:
                      - Elements not found: [
                            42,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_matching(matchers::predicate(|it: &i32| *it % 2 == 1))
            );
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!([1, 2, 3].as_slice())
                .does_not_contain_matching(matchers::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn panics_when_elements_match_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([-1, 7, 12].as_slice())
                    .with_location(false)
                    .does_not_contain_matching(crate::assertions::core::partial_ord::ge(5));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[-1, 7, 12].as_slice()`

                contains matching elements

                Nested failures:
                  - Actual: 7

                    matches the unwanted constraint

                    Constraint:
                        is greater than or equal to

                        Expected: 5

                  - Actual: 12

                    matches the unwanted constraint

                    Constraint:
                        is greater than or equal to

                        Expected: 5
                -------- assertr --------
            "});
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_satisfying(|it| {
                    it.is_greater_than(1);
                })
            );
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2, 3].as_slice()), does_not_contain(2));
        }

        #[test]
        fn succeeds_when_value_is_absent() {
            assert_that!([1, 2, 3].as_slice()).does_not_contain(4);
        }

        #[test]
        fn panics_when_value_is_present() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    contains

                    Unexpected: 2
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;

        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        struct Expected(u8);

        #[derive(Debug)]
        enum WildcardExpected {
            Any,
            Value(u8),
        }

        #[cfg(feature = "partial")]
        #[derive(Debug)]
        struct DerivedActual {
            pub value: u8,
        }

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        impl<R> Expectation<Actual, R> for WildcardExpected {
            type Success<'a> = ();
            type Rejection<'a> = ();
            fn evaluate(&self, actual: &Actual, _: &AssertionContext<'_, R>) -> Result<(), ()> {
                match self {
                    Self::Any => Ok(()),
                    Self::Value(value) if actual.0 == *value => Ok(()),
                    Self::Value(_) => Err(()),
                }
            }

            const KIND: crate::failure::FailureKind = crate::failure::FailureKind::Matching;
            fn explain(
                &self,
                rejected: Option<(&Actual, ())>,
                failure: crate::failure::FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> crate::failure::FailureBuilder {
                match rejected {
                    None => failure.relation("matches the wildcard constraint"),
                    Some((_, ())) => failure.constraint(context.describe(&self)),
                }
            }
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1].as_slice()),
                contains_exactly_in_any_order([1, 1])
            );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn custom_heterogeneous_comparisons_use_predicates() {
            assert_that!([Actual(1), Actual(2)].as_slice()).contains_exactly_in_any_order_matching(
                matchers::predicate_list([
                    |it: &Actual| it.eq(&Expected(2)),
                    |it: &Actual| it.eq(&Expected(1)),
                ]),
            );
        }

        #[test]
        fn supports_non_equivalence_matchers() {
            assert_that!([Actual(2), Actual(1)].as_slice()).contains_exactly_in_any_order_matching(
                [WildcardExpected::Any, WildcardExpected::Value(2)],
            );
        }

        #[test]
        #[cfg(feature = "partial")]
        fn supports_structural_wildcards() {
            let actual = [DerivedActual { value: 2 }, DerivedActual { value: 1 }];
            assert_that!(actual.as_slice()).contains_exactly_in_any_order_matching(matchers![
                partial!(DerivedActual {
                    value: matchers::anything()
                }),
                partial!(DerivedActual {
                    value: matchers::eq(2)
                }),
            ]);
        }

        #[test]
        fn rejects_different_multiplicities() {
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order([1, 1]);
            })
            .has_type::<String>()
            .contains("Elements not found");
        }

        #[test]
        fn panics_when_slice_contains_unknown_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 3, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly in any order

                    Expected: [
                        2,
                        3,
                        4,
                    ]

                    Details:
                      - Elements not found: [
                            4,
                        ]
                      - Elements not expected: [
                            1,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_matching([crate::assertions::core::partial_eq::eq(
                    2
                )])
            );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                matchers::predicate_list(
                    [
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                        move |it: &i32| *it == 3,
                    ]
                    .as_slice(),
                ),
            );
        }

        #[test]
        fn succeeds_when_slices_match_in_different_order() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                matchers::predicate_list(
                    [
                        move |it: &i32| *it == 3,
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                    ]
                    .as_slice(),
                ),
            );
        }

        #[test]
        fn succeeds_when_overlapping_predicates_have_an_exact_assignment() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it <= 2, |it| *it == 1];

            assert_that!([1, 2].as_slice())
                .contains_exactly_in_any_order_matching(matchers::predicate_list(predicates));
        }

        #[test]
        fn rejects_unmatched_predicates() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(matchers::predicate_list(predicates));
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1].as_slice()`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - At slot: 1
                    Nested failures:
                      - Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_slice_contains_non_matching_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(matchers::predicate_list(
                        [
                            move |it: &i32| *it == 2,
                            move |it: &i32| *it == 3,
                            move |it: &i32| *it == 4,
                        ]
                        .as_slice(),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - At slot: 2
                    Nested failures:
                      - Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 2

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 3

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                  - has unexpected elements

                    Details:
                      - Unexpected count: 1
                    Nested failures:
                      - Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn unordered_evaluates_each_visited_pair_once() {
            let count = core::cell::Cell::new(0);
            let first = matchers::predicate(|a: &i32| {
                count.set(count.get() + 1);
                *a <= 2
            });
            let second = matchers::predicate(|a: &i32| {
                count.set(count.get() + 1);
                *a == 1
            });
            assert_that!([1, 2]).contains_exactly_in_any_order_matching(matchers![first, second]);
            assert_that!(count.get()).is_less_or_equal_to(4);
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_satisfying([|it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                }])
            );
        }
    }
}
