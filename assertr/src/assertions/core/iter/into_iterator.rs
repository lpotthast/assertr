use alloc::vec::Vec;

use crate::{
    AssertThat, AssertrPartialEq, Mode, ValueRenderer, assertions::iterator, mode::Capture,
};

/// Chainable assertions over a fresh borrowed iteration of a collection-like value.
///
/// Each method calls `IntoIterator::into_iter(&subject)` exactly once and returns the original
/// assertion. Chaining therefore performs one fresh borrowed traversal per assertion. Streaming,
/// bounded-preview and potential-nontermination behavior matches [`super::IteratorAssertions`].
/// Method names are prefixed to avoid collisions with more specific collection assertion traits.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait IntoIteratorAssertions<T, R> {
    /// Asserts that a borrowed traversal contains an element equal to `expected`.
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that every expected element is present during one borrowed traversal.
    ///
    /// Extra subject elements are allowed and duplicates are not counted, matching
    /// [`CollectionAssertions::contains_all`](crate::assertions::collection::CollectionAssertions::contains_all).
    /// The traversal stops when all expected elements have been found. It cannot complete on a
    /// non-terminating source if an expected element never occurs.
    fn into_iter_contains_all<E, EI>(self, expected: EI) -> Self
    where
        T: AssertrPartialEq<E, R>,
        EI: IntoIterator<Item = E>,
        R: ValueRenderer<T> + ValueRenderer<E>;
    /// Asserts that a borrowed traversal contains an element matching `predicate`.
    fn into_iter_contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;
    /// Asserts that a borrowed traversal contains an element satisfying `assertions`.
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;
    /// Asserts that no element in a borrowed traversal equals `not_expected`.
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;
    /// Asserts that no element in a borrowed traversal matches `predicate`.
    fn into_iter_does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;
    /// Asserts that no element in a borrowed traversal satisfies `assertions`.
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;
    /// Asserts multiset equality with `expected`, ignoring order but preserving duplicate counts.
    fn into_iter_contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;
    /// Asserts one-to-one matching between elements and `predicates`, independent of order.
    fn into_iter_contains_exactly_in_any_order_matching<P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;
    /// Asserts one-to-one matching between elements and `assertions`, independent of order.
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;
    /// Asserts that a borrowed traversal yields no elements.
    fn into_iter_is_empty(self) -> Self
    where
        R: ValueRenderer<T>;
    /// Asserts that a borrowed traversal yields at least one element.
    fn into_iter_is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>;
    /// Asserts that a borrowed traversal yields exactly `expected` elements.
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T>;

    #[deprecated(since = "0.6.2", note = "renamed to `into_iter_is_empty`")]
    #[cfg_attr(feature = "fluent", no_fluent_alias)]
    /// Deprecated forwarding name for [`IntoIteratorAssertions::into_iter_is_empty`].
    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: ValueRenderer<T>;
}

impl<T, I, M: Mode, R> IntoIteratorAssertions<T, R> for AssertThat<'_, I, M, R>
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    #[track_caller]
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();
        iterator::assert_contains::<_, T, _, _, _, _>(&self, self.actual().into_iter(), &expected);
        self
    }
    #[track_caller]
    fn into_iter_contains_all<E, EI>(self, expected: EI) -> Self
    where
        T: AssertrPartialEq<E, R>,
        EI: IntoIterator<Item = E>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();
        let expected = expected.into_iter().collect::<Vec<_>>();
        iterator::assert_contains_all::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected.as_slice(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_contains_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &predicate,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &assertions,
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();
        iterator::assert_does_not_contain::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &not_expected,
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_does_not_contain_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &predicate,
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
    {
        self.track_assertion();
        iterator::assert_does_not_contain_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &assertions,
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        iterator::assert_contains_exactly_in_any_order::<_, T, E, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_matching<P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_in_any_order_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_in_any_order_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_is_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_is_empty::<_, T, _, _, _>(
            &self,
            self.actual().into_iter(),
            iterator::PositionReporting::Unavailable,
        );
        self
    }
    #[track_caller]
    fn into_iter_is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_is_not_empty::<_, T, _, _, _>(&self, self.actual().into_iter());
        self
    }
    #[track_caller]
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        iterator::assert_has_length::<_, T, _, _, _>(&self, self.actual().into_iter(), expected);
        self
    }
    #[track_caller]
    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.into_iter_is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => IntoIteratorAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn membership_uses_the_active_renderer_type() {
            assert_that!(vec![RendererActual(1), RendererActual(2)])
                .with_renderer(SentinelRenderer)
                .into_iter_contains(RendererExpected(2))
                .into_iter_contains_all([RendererExpected(1)]);
        }
    }

    mod into_iter_contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3].must().into_iter_contain(2);
        }

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_contains(2);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo".to_owned()]).into_iter_contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains(4);
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

                    does not contain expected: 4

                    Details:
                      - Consumed 3 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_all {
        use core::cell::Cell;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3].must().into_iter_contain_all([1, 3]);
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
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()]).into_iter_contains_all(["b", "a"]);
            assert_that!(vec!["a", "b"]).into_iter_contains_all(["b".to_owned(), "a".to_owned()]);
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

                    does not contain all expected elements

                    Expected: [
                        2,
                        4,
                    ]

                    Elements not found: [
                        4,
                    ]

                    Details:
                      - Consumed 3 element(s).
                    -------- assertr --------
                "});
        }

        struct CountingValues {
            values: Vec<i32>,
            iterations: Cell<usize>,
            yielded: Cell<usize>,
        }

        struct CountingIter<'a> {
            values: core::slice::Iter<'a, i32>,
            yielded: &'a Cell<usize>,
        }

        impl<'a> Iterator for CountingIter<'a> {
            type Item = &'a i32;

            fn next(&mut self) -> Option<Self::Item> {
                let next = self.values.next();
                if next.is_some() {
                    self.yielded.set(self.yielded.get() + 1);
                }
                next
            }
        }

        impl<'a> IntoIterator for &'a CountingValues {
            type Item = &'a i32;
            type IntoIter = CountingIter<'a>;

            fn into_iter(self) -> Self::IntoIter {
                self.iterations.set(self.iterations.get() + 1);
                CountingIter {
                    values: self.values.iter(),
                    yielded: &self.yielded,
                }
            }
        }

        #[test]
        fn creates_one_iterator_and_stops_as_soon_as_all_elements_are_found() {
            let values = CountingValues {
                values: vec![1, 2, 3, 4],
                iterations: Cell::new(0),
                yielded: Cell::new(0),
            };

            assert_that!(values).into_iter_contains_all([1, 3]);

            assert_that!(values.iterations.get()).is_equal_to(1);
            assert_that!(values.yielded.get()).is_equal_to(3);
        }

        #[test]
        fn empty_expectation_still_creates_one_iterator_but_consumes_nothing() {
            let values = CountingValues {
                values: vec![1, 2, 3],
                iterations: Cell::new(0),
                yielded: Cell::new(0),
            };

            let expected: [i32; 0] = [];
            assert_that!(values).into_iter_contains_all(expected);

            assert_that!(values.iterations.get()).is_equal_to(1);
            assert_that!(values.yielded.get()).is_equal_to(0);
        }
    }

    mod into_iter_contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3]
                .must()
                .into_iter_contain_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_matching(|it: &i32| *it > 7);
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

                    does not contain an element matching the predicate.

                    Details:
                      - Consumed 3 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3].must().into_iter_contain_satisfying(is_two);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_satisfying(is_two);
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2])
                    .with_location(false)
                    .into_iter_contains_satisfying(is_seven);
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the assertions.")
            .contains("An element does not satisfy the assertions:\n    Expected: 7");
        }
    }

    mod into_iter_does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3].must().into_iter_not_contain(4);
        }

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
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

                    contains unexpected: 2

                    Details:
                      - Consumed 2 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3]
                .must()
                .into_iter_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain_matching(|it: &i32| *it % 2 == 0);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                    ]

                    unexpectedly contains an element matching the predicate.

                    Details:
                      - Consumed 2 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_does_not_contain_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3]
                .must()
                .into_iter_not_contain_satisfying(is_seven);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain_satisfying(is_seven);
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain_satisfying(is_two);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                    ]

                    unexpectedly contains an element satisfying the assertions.

                    Details:
                      - Consumed 2 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_exactly_in_any_order {
        use crate::prelude::*;
        use crate::{AssertrPartialEq, EqContext};
        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        struct Expected(u8);

        impl<R> AssertrPartialEq<Expected, R> for Actual {
            fn eq(&self, other: &Expected, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
                self.0 == other.0
            }
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![2, 1, 1]
                .must()
                .into_iter_contain_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that!(vec![2, 1, 1]).into_iter_contains_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn supports_assertr_partial_eq_without_partial_eq() {
            assert_that!(vec![Actual(1), Actual(2)])
                .into_iter_contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order([1, 2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    Elements expected: [
                        1,
                        2,
                        9,
                    ]

                    The elements did not match exactly in any order.

                    Details:
                      - Consumed 3 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2]
                .must()
                .into_iter_contain_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        fn is_at_most_two(value: &i32) -> bool {
            *value <= 2
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
        fn succeeds_when_a_maximum_matching_exists_for_overlapping_predicates() {
            assert_that!(vec![1, 2])
                .into_iter_contains_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order_matching([is_one, is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3]`

                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match predicates in any order.

                    Details:
                      - Consumed 3 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![-1, 1]
                .must()
                .into_iter_contain_exactly_in_any_order_satisfying([positive, negative]);
        }

        fn positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        fn negative(it: AssertThat<i32, Capture>) {
            it.is_less_than(0);
        }

        #[test]
        fn succeeds_when_a_maximum_matching_exists() {
            assert_that!(vec![-1, 1])
                .into_iter_contains_exactly_in_any_order_satisfying([positive, negative]);
        }

        #[test]
        fn panics_when_an_element_satisfies_no_assertion() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, -1, 2])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order_satisfying([
                        positive, positive, positive,
                    ]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions in any order.")
            .contains("An element did not satisfy any available assertion:");
        }
    }

    mod into_iter_is_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Vec::<i32>::new().must().into_iter_be_empty();
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

                    is not empty.

                    Details:
                      - Consumed 1 element(s).
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_is_not_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1].must().into_iter_not_be_empty();
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

                    is empty.
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_has_length {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2].must().into_iter_have_length(2);
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

                    Actual: []

                    does not have the correct length

                    Expected: 2
                    Observed: 3

                    Details:
                      - Iterator reported an exact remaining length of 3; no elements were consumed.
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_iterator_is_empty {
        use crate::prelude::*;

        #[test]
        #[allow(deprecated)]
        fn deprecated_name_remains_available() {
            assert_that!(Vec::<i32>::new()).into_iter_iterator_is_empty();
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
