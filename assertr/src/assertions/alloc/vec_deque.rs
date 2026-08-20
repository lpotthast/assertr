use alloc::collections::VecDeque;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode, assertions::collection, mode::Capture,
};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait VecDequeAssertions<'t, T, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>;

    /// Test that at least one element matches the given predicate.
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>>;

    /// Test that at least one element satisfies the given assertions.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + Clone;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>;

    /// Test that no element matches the given predicate.
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>;

    /// Test that no element satisfies the given assertions.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T> + Clone;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>;

    /// Tests that each element matches the predicate at the same position. Order is important.
    /// The number of predicates must equal the number of elements.
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>;

    /// Tests that each element satisfies the assertions at the same position. Order is important.
    /// The number of assertion closures must equal the number of elements.
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + Clone;

    /// Tests multiset equality, including identical duplicate counts.
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<[T]> + AssertionRenderer<T>;

    /// Tests a one-to-one, order-independent match between predicates and actual elements.
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>;

    /// Tests a one-to-one, order-independent match between assertion closures and actual
    /// elements.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T> + Clone;

    /// Deprecated name of [`VecDequeAssertions::contains_exactly_in_any_order_matching`].
    #[deprecated(
        since = "0.7.0",
        note = "renamed to `contains_exactly_in_any_order_matching`"
    )]
    #[cfg_attr(feature = "fluent", no_fluent_alias)]
    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self: Sized,
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }

    /// Deprecated name of the `contain_exactly_in_any_order_matching` fluent alias.
    #[cfg(feature = "fluent")]
    #[deprecated(
        since = "0.7.0",
        note = "renamed to `contain_exactly_in_any_order_matching`"
    )]
    #[track_caller]
    fn contain_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self: Sized,
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }
}

impl<'t, T, M: Mode, R> VecDequeAssertions<'t, T, R> for AssertThat<'t, VecDeque<T>, M, R> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>,
    {
        collection::assert_contains(&self, &expected);
        self
    }

    #[track_caller]
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>>,
    {
        collection::assert_contains_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + Clone,
    {
        collection::assert_contains_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>,
    {
        collection::assert_does_not_contain(&self, &not_expected);
        self
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        collection::assert_does_not_contain_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T> + Clone,
    {
        collection::assert_does_not_contain_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>,
    {
        collection::assert_contains_exactly(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        collection::assert_contains_exactly_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + Clone,
    {
        collection::assert_contains_exactly_satisfying(&self, assertions.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<[T]> + AssertionRenderer<T>,
    {
        collection::assert_contains_exactly_in_any_order(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        collection::assert_contains_exactly_in_any_order_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T> + Clone,
    {
        collection::assert_contains_exactly_in_any_order_satisfying(&self, assertions.as_ref());
        self
    }
}

// These tests cover the delegation into `assertions::collection` and VecDeque-specific concerns
// (e.g. non-contiguous buffers). Edge cases of the shared implementation itself, such as
// multiplicity handling and overlapping predicates, are covered once in the slice tests.
#[cfg(test)]
mod tests {
    use alloc::collections::VecDeque;
    use alloc::vec::Vec;
    use std::fmt::Debug;

    fn vec_deque<T, const N: usize>(values: [T; N]) -> VecDeque<T> {
        values.into_iter().collect()
    }

    /// Forces a `VecDeque` whose `as_slices` call would yield exactly
    /// `(front_values, back_values)`.
    fn non_contiguous_vec_deque<
        T: Debug + PartialEq + Eq + Clone,
        const FRONT: usize,
        const BACK: usize,
    >(
        front_values: &[T; FRONT],
        back_values: [T; BACK],
    ) -> VecDeque<T> {
        let mut deque = VecDeque::with_capacity(FRONT + BACK);
        deque.extend(back_values);
        let mut back_values = Vec::with_capacity(BACK);
        while let Some(value) = deque.pop_front() {
            back_values.push(value);
        }
        deque.extend(front_values.iter().cloned());
        deque.extend(back_values.clone());
        let (front, back) = deque.as_slices();
        assert_eq!(front.len(), FRONT);
        assert_eq!(back.len(), BACK);
        assert_eq!(front, front_values.as_slice());
        assert_eq!(back, back_values.as_slice());
        deque
    }

    mod contains {
        use super::vec_deque;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!(vec_deque([1, 2, 3])).contains(1);
            assert_that!(vec_deque([1, 2, 3])).contains(2);
            assert_that!(vec_deque([1, 2, 3])).contains(3);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec_deque(["foo"])).contains("foo".to_owned());
            assert_that!(vec_deque(["foo".to_owned()])).contains("foo");
        }

        #[test]
        fn works_with_borrowed_vec_deque() {
            let deque = vec_deque([1, 2, 3]);

            assert_that!(&deque).contains(2);
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains(4);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain expected: 4
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use super::vec_deque;
        use crate::prelude::*;

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!(vec_deque([1, 2, 3])).contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_matching(|it: &i32| *it > 7);
            })
            .has_type::<String>()
            .contains("does not contain an element matching the given predicate.");
        }
    }

    mod contains_satisfying {
        use super::vec_deque;
        use crate::prelude::*;

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!(vec_deque([1, 2, 3])).contains_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_satisfying(|it| {
                        it.is_equal_to(7);
                    });
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the given assertions.");
        }
    }

    mod does_not_contain {
        use super::vec_deque;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!(vec_deque([1, 2, 3])).does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec_deque(["foo"])).does_not_contain("bar".to_owned());
            assert_that!(vec_deque(["foo".to_owned()])).does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    contains unexpected: 2
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_matching {
        use super::vec_deque;
        use crate::prelude::*;

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!(vec_deque([1, 2, 3])).does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .does_not_contain_matching(|it: &i32| *it % 2 == 0);
            })
            .has_type::<String>()
            .contains("contains elements matching the given predicate");
        }
    }

    mod does_not_contain_satisfying {
        use super::vec_deque;
        use crate::prelude::*;

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!(vec_deque([1, 2, 3])).does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .does_not_contain_satisfying(|it| {
                        it.is_equal_to(2);
                    });
            })
            .has_type::<String>()
            .contains("contains elements satisfying the given assertions");
        }
    }

    mod contains_exactly {
        use super::{non_contiguous_vec_deque, vec_deque};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_exact_match() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn succeeds_for_non_contiguous_vec_deque_in_logical_order() {
            assert_that!(non_contiguous_vec_deque(&[2, 3], [4, 5, 6]))
                .contains_exactly([2, 3, 4, 5, 6]);
        }

        #[test]
        fn compiles_for_different_type_combinations() {
            assert_that!(vec_deque(["foo".to_owned()])).contains_exactly(["foo"]);
            assert_that!(vec_deque(["foo"])).contains_exactly(["foo".to_owned()]);
            assert_that!(vec_deque(["foo"])).contains_exactly(["foo"]);
            assert_that!(vec_deque(["foo"])).contains_exactly(vec!["foo".to_owned()]);
            assert_that!(vec_deque(["foo"])).contains_exactly(vec!["foo".to_owned()].into_iter());
            assert_that!(vec_deque(["foo"])).contains_exactly(vec!["foo".to_owned()].as_slice());
        }

        #[test]
        fn panics_when_not_exact_match() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_exactly([2, 3, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match

                    Expected: [
                        2,
                        3,
                        4,
                    ]

                    Details: [
                        Elements not expected: [
                            1,
                        ],
                        Elements not found: [
                            4,
                        ],
                    ]
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_with_detail_message_when_only_differing_in_order() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_exactly([3, 2, 1]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match

                    Expected: [
                        3,
                        2,
                        1,
                    ]

                    Details: [
                        The order of elements does not match!,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_matching {
        use super::{non_contiguous_vec_deque, vec_deque};
        use crate::prelude::*;

        #[test]
        fn succeeds_when_each_element_matches_its_predicate() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 2,
                |it: &i32| *it == 3,
            ]);
        }

        #[test]
        fn succeeds_for_non_contiguous_vec_deque_in_logical_order() {
            assert_that!(non_contiguous_vec_deque(&[2, 3], [4, 5, 6])).contains_exactly_matching([
                |it: &i32| *it == 2,
                |it: &i32| *it == 3,
                |it: &i32| *it == 4,
                |it: &i32| *it == 5,
                |it: &i32| *it == 6,
            ]);
        }

        #[test]
        fn panics_when_an_element_does_not_match_its_predicate() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_exactly_matching([
                        |it: &i32| *it == 1,
                        |it: &i32| *it == 3,
                        |it: &i32| *it == 2,
                    ]);
            })
            .has_type::<String>()
            .contains("Element at index 1 does not match its predicate: 2");
        }
    }

    mod contains_exactly_satisfying {
        use super::{non_contiguous_vec_deque, vec_deque};
        use crate::prelude::*;

        #[test]
        fn succeeds_when_each_element_satisfies_its_assertions() {
            assert_that!(vec_deque([1, 2])).contains_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

        #[test]
        fn succeeds_for_non_contiguous_vec_deque_in_logical_order() {
            assert_that!(non_contiguous_vec_deque(&[2, 3], [4])).contains_exactly_satisfying([
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
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy_its_assertions() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2]))
                    .with_location(false)
                    .contains_exactly_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(2);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(1);
                        },
                    ]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions.")
            .contains("Element at index 0 does not satisfy its assertions:");
        }
    }

    mod contains_exactly_in_any_order {
        use super::{non_contiguous_vec_deque, vec_deque};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_values_match() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn succeeds_for_non_contiguous_vec_deque() {
            assert_that!(non_contiguous_vec_deque(&[2, 3], [4, 5, 6]))
                .contains_exactly_in_any_order([6, 5, 4, 3, 2]);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 3, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    Elements expected: [
                        2,
                        3,
                        4,
                    ]

                    Elements not found: [
                        4,
                    ]

                    Elements not expected: [
                        1,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use super::vec_deque;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_predicates_match() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_in_any_order_matching(
                [
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                    move |it: &i32| *it == 3,
                ]
                .as_slice(),
            );
        }

        #[test]
        #[allow(deprecated)]
        fn deprecated_name_remains_available() {
            assert_that!(vec_deque([1, 2]))
                .contains_exactly_matching_in_any_order([|it: &i32| *it == 2, |it: &i32| *it == 1]);
        }

        #[test]
        fn succeeds_when_predicates_match_in_different_order() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_in_any_order_matching(
                [
                    move |it: &i32| *it == 3,
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                ]
                .as_slice(),
            );
        }

        #[test]
        fn panics_when_data_is_unmatched() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2, 3]))
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
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match predicates in any order.

                    Details: [
                        Elements not matched: [
                            1,
                        ],
                        Predicates not matched: 1,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use super::vec_deque;
        use crate::prelude::*;

        #[test]
        fn succeeds_when_assertions_are_satisfied_in_different_order() {
            assert_that!(vec_deque([1, 2])).contains_exactly_in_any_order_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
            ]);
        }

        #[test]
        fn panics_when_elements_are_unmatched() {
            assert_that_panic_by(|| {
                assert_that!(vec_deque([1, 2]))
                    .with_location(false)
                    .contains_exactly_in_any_order_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(2);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(3);
                        },
                    ]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions in any order.");
        }
    }
}
