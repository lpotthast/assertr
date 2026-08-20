use alloc::vec::Vec;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode, mode::Capture, prelude::SliceAssertions,
};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait VecAssertions<'t, T, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E> + Clone;

    /// Test that at least one element matches the given predicate.
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + Clone;

    /// Test that at least one element satisfies the given assertions.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + Clone;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E> + Clone;

    /// Test that no element matches the given predicate.
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    /// Test that no element satisfies the given assertions.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>
            + Clone;

    /// Tests that each element matches the predicate at the same position. Order is important.
    /// The number of predicates must equal the number of elements.
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    /// Tests that each element satisfies the assertions at the same position. Order is important.
    /// The number of assertion closures must equal the number of elements.
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + Clone;

    /// Tests multiset equality, including identical duplicate counts.
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    /// Tests a one-to-one, order-independent match between predicates and actual elements.
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    /// Tests a one-to-one, order-independent match between assertion closures and actual
    /// elements.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone;

    /// Deprecated name of [`VecAssertions::contains_exactly_in_any_order_matching`].
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
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
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
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }
}

impl<'t, T, M: Mode, R> VecAssertions<'t, T, R> for AssertThat<'t, Vec<T>, M, R> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E> + Clone,
    {
        self.derive(Vec::as_slice).contains(expected);
        self
    }

    #[track_caller]
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + Clone,
    {
        self.derive(Vec::as_slice).contains_matching(predicate);
        self
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + Clone,
    {
        self.derive(Vec::as_slice).contains_satisfying(assertions);
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E> + Clone,
    {
        self.derive(Vec::as_slice).does_not_contain(not_expected);
        self
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .does_not_contain_matching(predicate);
        self
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .does_not_contain_satisfying(assertions);
        self
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>
            + Clone,
    {
        self.derive(Vec::as_slice).contains_exactly(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .contains_exactly_matching(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + Clone,
    {
        self.derive(Vec::as_slice)
            .contains_exactly_satisfying(assertions);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .contains_exactly_in_any_order(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool, // predicate
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .contains_exactly_in_any_order_matching(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
    {
        self.derive(Vec::as_slice)
            .contains_exactly_in_any_order_satisfying(assertions);
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!(vec![1, 2, 3]).contains(1);
            assert_that!(vec![1, 2, 3]).contains(2);
            assert_that!(vec![1, 2, 3]).contains(3);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo"]).contains("foo".to_owned());
            assert_that!(vec!["foo".to_owned()]).contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3]).with_location(false).contains(4);
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
        use crate::prelude::*;

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!(vec![1, 2, 3]).contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .contains_matching(|it: &i32| *it > 7);
            })
            .has_type::<String>()
            .contains("does not contain an element matching the given predicate.");
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!(vec![1, 2, 3]).contains_satisfying(|it| {
                it.is_equal_to(2);
            });
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!(vec![1, 2, 3]).does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo"]).does_not_contain("bar".to_owned());
            assert_that!(vec!["foo".to_owned()]).does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!(vec![1, 2, 3]).does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .does_not_contain_matching(|it: &i32| *it % 2 == 0);
            })
            .has_type::<String>()
            .contains("contains elements matching the given predicate");
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!(vec![1, 2, 3]).does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            });
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_exact_match() {
            assert_that!(vec![1, 2, 3]).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_different_type_combinations() {
            assert_that!(vec!["foo".to_owned()]).contains_exactly(["foo"]);
            assert_that!(vec!["foo"]).contains_exactly(["foo".to_owned()]);
            assert_that!(vec!["foo"]).contains_exactly(["foo"]);
            assert_that!(vec!["foo"]).contains_exactly(vec!["foo".to_owned()]);
            assert_that!(vec!["foo"]).contains_exactly(vec!["foo".to_owned()].into_iter());
            assert_that!(vec!["foo"]).contains_exactly(vec!["foo".to_owned()].as_slice());
        }

        #[test]
        fn panics_when_not_exact_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;

        #[test]
        fn succeeds_when_each_element_matches_its_predicate() {
            assert_that!(vec![1, 2, 3]).contains_exactly_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 2,
                |it: &i32| *it == 3,
            ]);
        }

        #[test]
        fn panics_when_an_element_does_not_match_its_predicate() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;

        #[test]
        fn succeeds_when_each_element_satisfies_its_assertions() {
            assert_that!(vec![1, 2]).contains_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy_its_assertions() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2])
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
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_values_match() {
            assert_that!(vec![1, 2, 3]).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_predicates_match() {
            assert_that!(vec![1, 2, 3]).contains_exactly_in_any_order_matching(
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
            assert_that!(vec![1, 2])
                .contains_exactly_matching_in_any_order([|it: &i32| *it == 2, |it: &i32| *it == 1]);
        }

        #[test]
        fn succeeds_when_predicates_match_in_different_order() {
            assert_that!(vec![1, 2, 3]).contains_exactly_in_any_order_matching(
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
                assert_that!(vec![1, 2, 3])
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
        use crate::prelude::*;

        #[test]
        fn succeeds_when_assertions_are_satisfied_in_different_order() {
            assert_that!(vec![1, 2]).contains_exactly_in_any_order_satisfying([
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
                assert_that!(vec![1, 2])
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
