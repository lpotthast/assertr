use crate::{
    AssertThat, DebugRenderer, Expectation, Mode, ValueRenderer,
    assertions::iterator::{
        Contains, ContainsAll, ContainsExactlyInAnyOrder, ContainsMatching, DoesNotContain,
        DoesNotContainMatching, ElementsAreInAnyOrder, IsEmpty, IsNotEmpty, LengthScan,
        PositionReporting::Unavailable, run,
    },
    borrow_for::{BorrowFor, borrow_for},
    expectation::{MatcherList, lists::SatisfyingList, satisfying},
    mode::Capture,
};

/// Chainable assertions over a fresh borrowed iteration of a collection-like value.
///
/// Each method calls `IntoIterator::into_iter(&subject)` exactly once and returns the original
/// assertion. Chaining therefore performs one fresh borrowed traversal per assertion. Streaming,
/// bounded-preview and potential-nontermination behavior matches [`super::IteratorAssertions`].
/// Method names are prefixed to avoid collisions with more specific collection assertion traits.
/// The temporary iterator stays alive until rejection diagnostics own their rendered values, then
/// drops before failure handling or continuation. No iterator observation is repeated for
/// diagnostics.
///
/// Bulk value lists use [repeatable expected data](crate#expected-lists).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait IntoIteratorAssertions<T, R = DebugRenderer> {
    /// Asserts that a borrowed traversal contains an element equal to `expected`.
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that every expected element is present during one borrowed traversal.
    ///
    /// Extra subject elements are allowed and duplicates are not counted, matching
    /// [`CollectionAssertions::contains_all`](crate::assertions::collection::CollectionAssertions::contains_all).
    /// The traversal stops when all expected elements have been found. It cannot complete on a
    /// non-terminating source if an expected element never occurs.
    fn into_iter_contains_all<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that a borrowed traversal contains an element matching `expected`.
    fn into_iter_contains_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize>;
    /// Asserts that a borrowed traversal contains an element satisfying `assertions`.
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that no element in a borrowed traversal equals `not_expected`.
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that no element in a borrowed traversal matches `expected`.
    fn into_iter_does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize> + ValueRenderer<T>;
    /// Asserts that no element in a borrowed traversal satisfies `assertions`.
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>;

    /// Asserts multiset equality with `expected`, ignoring order but preserving duplicate counts.
    fn into_iter_contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts one-to-one matching between elements and the expected matcher list, independent of
    /// order.
    fn into_iter_contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;
    /// Asserts one-to-one matching between elements and `assertions`, independent of order.
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that a borrowed traversal yields no elements.
    fn into_iter_is_empty(self) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>;

    /// Asserts that a borrowed traversal yields at least one element.
    fn into_iter_is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>;
    /// Asserts that a borrowed traversal yields exactly `expected` elements.
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>;
}

impl<T, I, M: Mode, R> IntoIteratorAssertions<T, R> for AssertThat<'_, I, M, R>
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    #[track_caller]
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        run(&self, || {
            let expected = borrow_for::<T, _>(&expected);
            (self.actual().into_iter(), Contains::<T, _>::new(expected))
        });
        self
    }
    #[track_caller]
    fn into_iter_contains_all<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        run(&self, || {
            let scan = ContainsAll::<T, _>::new(expected.as_ref());
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_contains_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize>,
    {
        run(&self, || {
            let scan = ContainsMatching::<T, _>::new(expected, Unavailable);
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        self.into_iter_contains_matching(satisfying(assertions))
    }
    #[track_caller]
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        run(&self, || {
            let unexpected = borrow_for::<T, _>(&not_expected);
            let scan = DoesNotContain::<T, _>::new(unexpected, Unavailable);
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_matching<P>(self, expected: P) -> Self
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize> + ValueRenderer<T>,
    {
        run(&self, || {
            let scan = DoesNotContainMatching::<T, _>::new(expected, Unavailable);
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
    {
        self.into_iter_does_not_contain_matching(satisfying(assertions))
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        run(&self, || {
            let scan = ContainsExactlyInAnyOrder::<T, _>::new(expected.as_ref());
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_matching<P>(self, expected: P) -> Self
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        run(&self, || {
            let scan = ElementsAreInAnyOrder::<T, _>::new(expected);
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        run(&self, || {
            let expected = SatisfyingList::new(assertions.as_ref());
            let scan = ElementsAreInAnyOrder::<T, _>::new(expected);
            (self.actual().into_iter(), scan)
        });
        self
    }
    #[track_caller]
    fn into_iter_is_empty(self) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        run(&self, || (self.actual().into_iter(), IsEmpty::<T>::new()));
        self
    }
    #[track_caller]
    fn into_iter_is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        run(&self, || {
            (self.actual().into_iter(), IsNotEmpty::<T>::new())
        });
        self
    }
    #[track_caller]
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        run(&self, || {
            (self.actual().into_iter(), LengthScan::<T>::new(expected))
        });
        self
    }
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
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
            let values = vec![1, 2, 3];
            values.must().into_iter_contain(2);
            values.must().into_iter_contain_all([1, 3]);
            values.must().into_iter_contain_matching(eq(2));
            values.must().into_iter_contain_satisfying(is(2));
            values.must().into_iter_not_contain(4);
            values.must().into_iter_not_contain_matching(eq(4));
            values.must().into_iter_not_contain_satisfying(is(4));
            values
                .must()
                .into_iter_contain_exactly_in_any_order([3, 1, 2]);
            values
                .must()
                .into_iter_contain_exactly_in_any_order_matching([eq(3), eq(1), eq(2)]);
            values
                .must()
                .into_iter_contain_exactly_in_any_order_satisfying([is(3), is(1), is(2)]);
            Vec::<i32>::new().must().into_iter_be_empty();
            values.must().into_iter_not_be_empty();
            values.must().into_iter_have_length(3);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{
                ComparisonRenderer, NoRenderer, RendererActual, RendererExpected, assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => IntoIteratorAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn matchers_render_elements_without_debug() {
            use crate::{
                expectation::{anything, predicate},
                test_support::{NumericRenderer, SentinelRenderer},
            };

            struct Opaque;

            assert_that!([Opaque])
                .with_renderer(SentinelRenderer)
                .into_iter_contains_matching(anything())
                .into_iter_does_not_contain_matching(predicate(|_: &Opaque| false));
            assert_that!([Opaque, Opaque])
                .with_renderer(NumericRenderer)
                .into_iter_contains_exactly_in_any_order_matching(matchers![
                    anything(),
                    anything(),
                ]);
        }

        #[test]
        fn membership_uses_the_active_renderer_type() {
            assert_that!(vec![RendererActual(1), RendererActual(2)])
                .with_renderer(ComparisonRenderer)
                .into_iter_contains(RendererExpected::new(2))
                .into_iter_contains_all([RendererExpected::new(1)]);
        }
    }

    mod into_iter_contains {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2, 3]), into_iter_contains(4));
        }

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_contains(2);
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that!(vec!["foo".to_owned()]).into_iter_contains("foo");
        }
    }

    mod into_iter_contains_all {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2, 3]), into_iter_contains_all([2, 4]));
        }

        #[test]
        fn succeeds_when_every_expected_element_is_present() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_all([3, 1]);
        }

        #[test]
        fn succeeds_with_vec_input() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_all(vec![3, 1]);
        }

        #[test]
        fn ignores_duplicate_expectations_and_extra_actual_elements() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_all([1, 1, 1]);
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()]).into_iter_contains_all(["b", "a"]);
            assert_that!(vec!["a", "b"]).into_iter_contains_all(["b", "a"]);
        }

        #[test]
        fn panics_when_any_expected_value_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_all([2, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain all of

                    Expected: [
                        2,
                        4,
                    ]

                    Details:
                      - Elements not found: [
                            4,
                        ]
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2, 3]),
                into_iter_contains_matching(crate::expectation::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_matching(crate::expectation::predicate(|it: &i32| *it > 7));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `vec![1, 2, 3]`

                does not contain a matching element

                Details:
                  - Consumed elements: 3
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

    mod into_iter_contains_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2]),
                into_iter_contains_satisfying(is_seven)
            );
        }

        #[test]
        fn borrows_non_clone_items_and_continues_on_the_original_subject() {
            struct Item(i32);
            let values = [Item(1), Item(2), Item(3)];

            assert_that!(values)
                .into_iter_contains_satisfying(|item| {
                    item.derive(|item| &item.0).is_equal_to(2);
                })
                .derive(|items| &items[2].0)
                .is_equal_to(3);
            assert_that!(values[0].0).is_equal_to(1);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }
    }

    mod into_iter_does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2, 3]), into_iter_does_not_contain(2));
        }

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain(4);
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that!(vec!["foo".to_owned()]).into_iter_does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                    ]

                    contains

                    Unexpected: 2

                    Details:
                      - Consumed elements: 2
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2, 3]),
                into_iter_does_not_contain_matching(crate::expectation::predicate(|it: &i32| {
                    *it % 2 == 0
                }))
            );
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain_matching(crate::expectation::predicate(
                        |it: &i32| *it % 2 == 0,
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `vec![1, 2, 3]`

                contains an unexpected matching element

                Details:
                  - Consumed elements: 2
                Nested failures:
                  - Actual: 2

                    matches the unwanted constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod into_iter_does_not_contain_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2, 3]),
                into_iter_does_not_contain_satisfying(is_two)
            );
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }
    }

    mod into_iter_contains_exactly_in_any_order {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2, 3]),
                into_iter_contains_exactly_in_any_order([1, 2, 9])
            );
        }

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that!(vec![2, 1, 1]).into_iter_contains_exactly_in_any_order([1, 2, 1]);
        }
    }

    mod into_iter_contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, 2, 3]),
                into_iter_contains_exactly_in_any_order_matching(
                    crate::expectation::predicate_list([is_one, is_two, is_nine,])
                )
            );
        }

        #[test]
        fn matches_items_without_equality_through_predicates() {
            /// An item without `PartialEq`, matched by predicates over its field.
            #[derive(Debug)]
            struct Opaque(u8);

            assert_that!(vec![Opaque(1), Opaque(2)])
                .into_iter_contains_exactly_in_any_order_matching(
                    crate::expectation::predicate_list([
                        |it: &Opaque| it.0 == 2,
                        |it: &Opaque| it.0 == 1,
                    ]),
                );
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order_matching(
                        crate::expectation::predicate_list([is_one, is_two, is_nine]),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `vec![1, 2, 3]`

                does not match exactly in any order

                Details:
                  - Consumed elements: 3
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
                      - Actual: 3

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

    mod into_iter_contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(vec![1, -1, 2]),
                into_iter_contains_exactly_in_any_order_satisfying([positive, positive, positive,])
            );
        }

        fn positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }
    }

    mod into_iter_is_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1]), into_iter_is_empty());
        }

        #[test]
        fn succeeds_when_empty() {
            assert_that!(Vec::<i32>::new()).into_iter_is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!(vec![1])
                    .with_location(false)
                    .into_iter_is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1]`

                    Actual: [
                        1,
                    ]

                    is not empty

                    Details:
                      - Consumed elements: 1
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_is_not_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Vec::<i32>::new()), into_iter_is_not_empty());
        }

        #[test]
        fn succeeds_when_not_empty() {
            assert_that!(vec![1]).into_iter_is_not_empty();
        }

        #[test]
        fn panics_when_empty() {
            assert_that_panic_by(|| {
                assert_that!(Vec::<i32>::new())
                    .with_location(false)
                    .into_iter_is_not_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `Vec::<i32>::new()`

                    Actual: []

                    is unexpectedly empty
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_has_length {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2, 3]), into_iter_has_length(2));
        }

        #[test]
        fn succeeds_when_length_matches() {
            assert_that!(vec![1, 2]).into_iter_has_length(2);
        }

        #[test]
        fn panics_when_length_differs() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    does not have the expected length

                    Expected: 2

                    Details:
                      - Reported length: 3
                    -------- assertr --------
                "});
        }
    }

    mod chaining {
        use crate::prelude::*;

        #[test]
        fn assertions_chain_on_the_original_subject() {
            assert_that!(vec![1, 2])
                .into_iter_contains(1)
                .into_iter_does_not_contain(3)
                .into_iter_has_length(2)
                .into_iter_is_not_empty();
        }
    }
}
