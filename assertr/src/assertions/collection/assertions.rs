use alloc::vec::Vec;
use core::borrow::Borrow;

use super::{Collection, identity, value};
use crate::{AssertThat, Mode, ValueRenderer, mode::Capture};

/// Assertions over the elements of a collection: slices, arrays, `Vec`, `VecDeque`, and every type
/// implementing [`Collection`].
///
/// The collection structure is rendered by Assertr, so value-based methods require rendering
/// support for the element type rather than the collection type. Identity methods display addresses
/// and require no rendering support.
///
/// For a type that supports borrowed traversal but does not implement [`Collection`], use
/// [`IntoIteratorAssertions`](crate::assertions::core::iter::IntoIteratorAssertions). Its methods
/// carry the `into_iter_` prefix.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CollectionAssertions<T, R> {
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
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that at least one element matches `expected`.
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::AssertrMatcher<T, R>;

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
        T: PartialEq<E>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element equals `not_expected`.
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that no element matches `expected`.
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::AssertrMatcher<T, R>;

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
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts one-to-one matching between subject elements and predicates, independent of order.
    ///
    /// A maximum matching makes overlapping predicates order-independent.
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts one-to-one matching between subject elements and assertion closures, independent of
    /// order. A maximum matching makes overlapping assertions order-independent.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
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
        identity::assert_contains_same_instance_as(&self, expected);
        self
    }

    #[track_caller]
    fn does_not_contain_same_instance_as<U: ?Sized>(self, expected: &U) -> Self
    where
        C::Item: Borrow<U>,
    {
        identity::assert_does_not_contain_same_instance_as(&self, expected);
        self
    }

    #[track_caller]
    fn contains_exactly_same_instances_in_any_order<'e, U: ?Sized + 'e>(
        self,
        expected: impl AsRef<[&'e U]>,
    ) -> Self
    where
        C::Item: Borrow<U>,
    {
        identity::assert_contains_exactly_same_instances_in_any_order(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        value::assert_contains(&self, &expected);
        self
    }

    #[track_caller]
    fn contains_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::AssertrMatcher<C::Item, R>,
    {
        self.track_assertion();
        self.assert_matcher(&crate::matchers::contains_matching(expected), true);
        self
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.track_assertion();
        self.assert_matcher(
            &crate::matchers::contains_matching(crate::matchers::satisfying(assertions)),
            true,
        );
        self
    }

    #[track_caller]
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        C::Item: PartialEq<E>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        let expected = expected.into_iter().collect::<Vec<_>>();
        value::assert_contains_all(&self, expected.as_slice());
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        value::assert_does_not_contain(&self, &not_expected);
        self
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::AssertrMatcher<C::Item, R>,
    {
        self.track_assertion();
        self.assert_matcher(&crate::matchers::contains_matching(expected), false);
        self
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.track_assertion();
        self.assert_matcher(
            &crate::matchers::contains_matching(crate::matchers::satisfying(assertions)),
            false,
        );
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: PartialEq<E>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        value::assert_contains_exactly_in_any_order(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: crate::matchers::MatcherList<C::Item, R>,
        R: ValueRenderer<usize>,
    {
        self.track_assertion();
        self.assert_matcher(&crate::matchers::elements_are_in_any_order(expected), true);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<C::Item> + Clone + ValueRenderer<usize>,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        self.track_assertion();
        self.assert_matcher(
            &crate::matchers::elements_are_in_any_order(
                assertions
                    .as_ref()
                    .iter()
                    .map(crate::matchers::satisfying)
                    .collect::<Vec<_>>(),
            ),
            true,
        );
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
            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2, 3].as_slice()), contains(4));
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
                .contain_matching(crate::matchers::predicate(|it: &i32| *it % 2 == 0));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                contains_matching(crate::matchers::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!([1, 2, 3].as_slice())
                .contains_matching(crate::matchers::predicate(|it: &i32| *it % 2 == 0));
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_matching(crate::matchers::predicate(|it: &i32| *it > 7));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2].as_slice()),
                contains_satisfying(|it| {
                    it.is_equal_to(7);
                })
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].as_slice()`

                does not match

                Nested failures:
                  - Expected: 7

                      Actual: 1
                  - Expected: 7

                      Actual: 2
                -------- assertr --------
            "});
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
            assert_that!(failures[0].children[0].actual.as_ref().map(rendered_text))
                .is_equal_to(Some("123... 3 more characters ...".to_owned()));
            assert_that!(failures[0].omitted_children).is_equal_to(2);
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
                .not_contain_matching(crate::matchers::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_matching(crate::matchers::predicate(|it: &i32| *it % 2 == 1))
            );
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!([1, 2, 3].as_slice())
                .does_not_contain_matching(crate::matchers::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn panics_when_elements_match_and_lists_them() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .does_not_contain_matching(crate::matchers::predicate(|it: &i32| *it % 2 == 1));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                matches unexpectedly

                Nested failures:
                  - satisfies the constraint unexpectedly

                    Constraint:
                        satisfies the predicate
                  - satisfies the constraint unexpectedly

                    Constraint:
                        satisfies the predicate
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2, 3].as_slice()),
                does_not_contain_satisfying(|it| {
                    it.is_greater_than(1);
                })
            );
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
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                matches unexpectedly

                Nested failures:
                  - satisfies the constraint unexpectedly

                    Constraint:
                        satisfies the assertions
                  - satisfies the constraint unexpectedly

                    Constraint:
                        satisfies the assertions
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

        #[cfg(feature = "matchers")]
        #[derive(Debug)]
        struct DerivedActual {
            pub value: u8,
        }

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        impl<R> AssertrMatcher<Actual, R> for WildcardExpected {
            fn evaluate(
                &self,
                actual: &Actual,
                c: &mut crate::matchers::MatchContext<'_, R>,
            ) -> crate::matchers::MatchResult {
                c.outcome(
                    match self {
                        Self::Any => true,
                        Self::Value(value) => actual.0 == *value,
                    },
                    |_| {
                        crate::matchers::ConstraintDescription::new(
                            "matches the wildcard constraint",
                        )
                    },
                )
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
        fn supports_heterogeneous_partial_eq() {
            assert_that!([Actual(1), Actual(2)].as_slice())
                .contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn supports_non_equivalence_matchers() {
            assert_that!([Actual(2), Actual(1)].as_slice()).contains_exactly_in_any_order_matching(
                [WildcardExpected::Any, WildcardExpected::Value(2)],
            );
        }

        #[test]
        #[cfg(feature = "matchers")]
        fn supports_structural_wildcards() {
            let actual = [DerivedActual { value: 2 }, DerivedActual { value: 1 }];
            assert_that!(actual.as_slice()).contains_exactly_in_any_order_matching(matchers![
                partial!(DerivedActual {
                    value: crate::matchers::anything()
                }),
                partial!(DerivedActual { value: 2 }),
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
                .contain_exactly_in_any_order_matching(crate::matchers::predicate_list(
                    [
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 2,
                        move |it: &i32| *it == 3,
                    ]
                    .as_slice(),
                ));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_matching([crate::matchers::equal_to(2)])
            );
        }

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order_matching(
                crate::matchers::predicate_list(
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
                crate::matchers::predicate_list(
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

            assert_that!([1, 2].as_slice()).contains_exactly_in_any_order_matching(
                crate::matchers::predicate_list(predicates),
            );
        }

        #[test]
        fn rejects_unmatched_predicates() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(crate::matchers::predicate_list(
                        predicates,
                    ));
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1].as_slice()`

                does not match

                Nested failures:
                  - has no distinct matching element

                    Constraint:
                        satisfies the predicate

                    Details:
                      - expected slot: 1
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_slice_contains_non_matching_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(crate::matchers::predicate_list(
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
                  - has no distinct matching element

                    Constraint:
                        satisfies the predicate

                    Details:
                      - expected slot: 2
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn unordered_evaluates_each_visited_pair_once() {
            let count = core::cell::Cell::new(0);
            let first = crate::matchers::predicate(|a: &i32| {
                count.set(count.get() + 1);
                *a <= 2
            });
            let second = crate::matchers::predicate(|a: &i32| {
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1]),
                contains_exactly_in_any_order_satisfying([|it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                }])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].as_slice()`

                does not match

                Nested failures:
                  - has no distinct matching element

                    Constraint:
                        satisfies the assertions

                    Details:
                      - expected slot: 2
                    Nested failures:
                      - Expected: 4

                          Actual: 1
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - Expected: 2

                          Actual: 1
                      - Expected: 3

                          Actual: 1
                -------- assertr --------
            "});
        }
    }
}
