use alloc::vec::Vec;

use super::{Collection, imp};
use crate::{AssertThat, AssertrPartialEq, Mode, ValueRenderer, mode::Capture};

/// Assertions over the elements of a collection: slices, arrays, `Vec`, `VecDeque`, and every
/// type implementing [`Collection`].
///
/// The collection structure is rendered by Assertr, so methods require rendering support for the
/// element type rather than the collection type.
///
/// For a type that supports borrowed traversal but does not implement [`Collection`], use
/// [`IntoIteratorAssertions`](crate::assertions::core::iter::IntoIteratorAssertions). Its methods
/// carry the `into_iter_` prefix.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait CollectionAssertions<T, R> {
    /// Asserts that at least one element equals `expected`.
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that at least one element matches `predicate`.
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool;

    /// Asserts that at least one element satisfies `assertions`.
    ///
    /// The assertions run in capture mode against each element. An element matches when no
    /// assertion failure is raised. On failure, every element's captured failures are reported.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Asserts that every expected element has an equal element in the subject.
    ///
    /// Extra subject elements are allowed. Expectations are independent, so duplicates do not
    /// require distinct matches. Use `contains_exactly_in_any_order` for multiset equality.
    ///
    /// `E` is the element type of the expected values, which only has to be comparable to `T`, not
    /// identical to it. Any iterable of expected values is accepted, including another collection.
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: AssertrPartialEq<E, R>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element equals `not_expected`.
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element matches `predicate`.
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool;

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
    /// duplicate counts must match. [`AssertrPartialEq`] permits different element types.
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts one-to-one matching between subject elements and predicates, independent of order.
    ///
    /// A maximum matching makes overlapping predicates order-independent.
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool;

    /// Asserts one-to-one matching between subject elements and assertion closures, independent of
    /// order. A maximum matching makes overlapping assertions order-independent.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Deprecated name of [`CollectionAssertions::contains_exactly_in_any_order_matching`].
    #[deprecated(
        since = "0.6.2",
        note = "renamed to `contains_exactly_in_any_order_matching`"
    )]
    #[cfg_attr(feature = "fluent", no_fluent_alias)]
    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self: Sized,
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }

    /// Deprecated name of the `contain_exactly_in_any_order_matching` fluent alias.
    #[cfg(feature = "fluent")]
    #[deprecated(
        since = "0.6.2",
        note = "renamed to `contain_exactly_in_any_order_matching`"
    )]
    #[track_caller]
    fn contain_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self: Sized,
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }
}

impl<C, M, R> CollectionAssertions<C::Item, R> for AssertThat<'_, C, M, R>
where
    C: Collection,
    M: Mode,
{
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_contains(&self, &expected);
        self
    }

    #[track_caller]
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        R: ValueRenderer<C::Item>,
        P: Fn(&C::Item) -> bool,
    {
        imp::assert_contains_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        imp::assert_contains_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        let expected = expected.into_iter().collect::<Vec<_>>();
        imp::assert_contains_all(&self, expected.as_slice());
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_does_not_contain(&self, &not_expected);
        self
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        R: ValueRenderer<C::Item>,
        P: Fn(&C::Item) -> bool,
    {
        imp::assert_does_not_contain_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        imp::assert_does_not_contain_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_contains_exactly_in_any_order(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: ValueRenderer<C::Item>,
        P: Fn(&C::Item) -> bool,
    {
        imp::assert_contains_exactly_in_any_order_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        imp::assert_contains_exactly_in_any_order_satisfying(&self, assertions.as_ref());
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, RendererActual, RendererExpected, SENTINEL, SentinelRenderer,
            assert_trait_impl,
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
                .with_renderer(SentinelRenderer)
                .contains(RendererExpected(2))
                .contains_all([RendererExpected(1)])
                .contains_exactly_in_any_order([RendererExpected(2), RendererExpected(1)]);

            let failures = assert_that!([RendererActual(1)].as_slice())
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains(RendererExpected(2)));
            assert_that!(failures[0].description()).contains(SENTINEL);
        }
    }

    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain(2);
        }

        #[test]
        fn succeeds_when_value_is_present() {
            assert_that!([1, 2, 3].as_slice()).contains(2);
        }

        #[test]
        fn compiles_for_comparable_but_different_element_types() {
            assert_that!(["foo"].as_slice()).contains("foo".to_owned());
        }

        #[test]
        fn preserves_the_chain_subject_name() {
            let failures = assert_that!(vec![1, 2, 3])
                .with_subject_name("the elements")
                .capture(|it| it.contains(4));

            assert_that!(&failures).has_length(1);
            assert_that!(failures[0].subject_name.as_deref()).is_equal_to(Some("the elements"));
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
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!([1, 2, 3].as_slice()).contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_matching(|it: &i32| *it > 7);
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

                    does not contain an element matching the predicate
                    -------- assertr --------
                "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!([1, 2, 3].as_slice()).contains_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn panics_when_no_element_satisfies_and_lists_every_elements_failures() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_satisfying(|it| {
                        it.is_equal_to(7);
                    });
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the assertions")
            .contains("Nested failures:\n  - Expected: 7\n\n      Actual: 1\n  - Expected: 7\n\n      Actual: 2\n");
        }

        #[test]
        fn rendering_budget_limits_items_and_nested_failure_values() {
            let failures = assert_that!([123_456, 234_567, 345_678].as_slice())
                .with_rendering_budget(
                    RenderingBudget::builder()
                        .max_items(1)
                        .max_leaf_characters(3)
                        .build(),
                )
                .with_location(false)
                .capture(|it| {
                    it.contains_satisfying(|element| {
                        element.is_equal_to(99);
                    })
                });

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].children[0].actual.as_deref())
                .is_equal_to(Some("123... 3 more characters ..."));
            assert_that!(failures[0].facts.as_slice())
                .contains_exactly([crate::Fact::note("... 2 more unsatisfied elements ...")]);
        }
    }

    mod contains_all {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_all([1, 3]);
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
        fn compiles_for_comparable_but_different_type() {
            assert_that!(["foo"].as_slice()).contains_all(["foo".to_owned()]);
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!([1, 2, 3].as_slice()).does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_elements_match_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain_matching(|it: &i32| *it % 2 == 1);
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

                    contains elements matching the predicate

                    Unexpected: [
                        1,
                        3,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!([1, 2, 3].as_slice()).does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn panics_when_elements_satisfy_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain_satisfying(|it| {
                        it.is_greater_than(1);
                    });
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

                    contains elements satisfying the assertions

                    Unexpected: [
                        2,
                        3,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().not_contain(4);
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
        use crate::{AssertrPartialEq, EqContext};
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

        #[cfg(feature = "derive")]
        #[derive(Debug, AssertrEq)]
        struct DerivedActual {
            pub value: u8,
        }

        impl<R> AssertrPartialEq<Expected, R> for Actual {
            fn eq(&self, other: &Expected, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
                self.0 == other.0
            }
        }

        impl<R> AssertrPartialEq<WildcardExpected, R> for Actual {
            fn eq(&self, other: &WildcardExpected, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
                match other {
                    WildcardExpected::Any => true,
                    WildcardExpected::Value(expected) => self.0 == *expected,
                }
            }
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn supports_assertr_partial_eq_without_partial_eq() {
            assert_that!([Actual(1), Actual(2)].as_slice())
                .contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn supports_non_equivalence_assertr_partial_eq() {
            assert_that!([Actual(2), Actual(1)].as_slice())
                .contains_exactly_in_any_order([WildcardExpected::Any, WildcardExpected::Value(2)]);
        }

        #[test]
        #[cfg(feature = "derive")]
        fn supports_derived_assertr_eq_wildcards() {
            let actual = [DerivedActual { value: 2 }, DerivedActual { value: 1 }];
            assert_that!(actual.as_slice()).contains_exactly_in_any_order([
                DerivedActualAssertrEq { value: any() },
                DerivedActualAssertrEq { value: eq(2) },
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .as_slice()
                .must()
                .contain_exactly_in_any_order_matching(
                    [
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                        move |it: &i32| *it == 3,
                    ]
                    .as_slice(),
                );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                [
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                    move |it: &i32| *it == 3,
                ]
                .as_slice(),
            );
        }

        #[test]
        fn succeeds_when_slices_match_in_different_order() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                [
                    move |it: &i32| *it == 3,
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                ]
                .as_slice(),
            );
        }

        #[test]
        fn succeeds_when_overlapping_predicates_have_an_exact_assignment() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it <= 2, |it| *it == 1];

            assert_that!([1, 2].as_slice()).contains_exactly_in_any_order_matching(predicates);
        }

        #[test]
        fn rejects_unmatched_predicates() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(predicates);
            })
            .has_type::<String>()
            .contains("Predicates not matched: 1");
        }

        #[test]
        fn panics_when_slice_contains_non_matching_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(
                        [
                            move |it: &i32| *it == 2,
                            move |it: &i32| *it == 3,
                            move |it: &i32| *it == 4,
                        ]
                        .as_slice(),
                    );
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

                    does not exactly match the predicates in any order

                    Details:
                      - Elements not matched: [
                            1,
                        ]
                      - Predicates not matched: 1
                    -------- assertr --------
                "});
        }

        #[test]
        #[allow(deprecated)]
        fn deprecated_names_remain_available() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 2, |it| *it == 1];

            assert_that!([1, 2].as_slice()).contains_exactly_matching_in_any_order(predicates);

            #[cfg(feature = "fluent")]
            assert_that!([1, 2].as_slice()).contain_exactly_matching_in_any_order(predicates);
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2]
                .as_slice()
                .must()
                .contain_exactly_in_any_order_satisfying([
                    |it: AssertThat<i32, Capture>| {
                        it.is_equal_to(2);
                    },
                    |it: AssertThat<i32, Capture>| {
                        it.is_equal_to(1);
                    },
                ]);
        }

        #[test]
        fn succeeds_when_assertions_are_satisfied_in_different_order() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

        #[test]
        fn succeeds_when_overlapping_assertions_have_an_exact_assignment() {
            assert_that!([1, 2].as_slice()).contains_exactly_in_any_order_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_less_than(3);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
            ]);
        }

        #[test]
        fn panics_when_elements_are_unmatched() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(2);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(3);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(4);
                        },
                    ]);
            })
            .has_type::<String>()
            .contains("does not exactly satisfy the assertions in any order")
            .contains("Elements not matched: [\n        1,\n    ]")
            .contains("Assertions not matched: 1");
        }
    }
}
