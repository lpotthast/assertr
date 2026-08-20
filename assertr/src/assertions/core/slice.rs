use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode, assertions::collection, mode::Capture,
};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SliceAssertions<'t, T, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>;

    /// Test that at least one element matches the given predicate.
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        R: AssertionRenderer<[T]>,
        P: Fn(&T) -> bool;

    /// Test that at least one element satisfies the given assertions.
    ///
    /// The assertions run in capture mode against each element; an element matches when no
    /// assertion failure is raised. On failure, every element's captured failures are reported.
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: AssertionRenderer<[T]> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>;

    /// Test that no element matches the given predicate.
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool;

    /// Test that no element satisfies the given assertions.
    ///
    /// The assertions run in capture mode against each element; an element matches when no
    /// assertion failure is raised. On failure, the matching elements are reported.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Test that the subject contains exactly the expected elements. Order is important. Lengths must be identical.
    ///
    /// - `T`: Original subject type. The "actual value" is of type `&[T]` (slice T).
    /// - `E`: Type of elements in our "expected value" slice.
    /// - `EE`: The "expected value". Anything that can be seen as `&[E]` (slice E). Having this extra type, instead of directly accepting `&[E]` allows us to be generic over the input in both the element type and collection type.
    fn contains_exactly<E, EE>(self, expected: EE) -> Self
    where
        E: 't,
        EE: AsRef<[E]>,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>;

    /// Tests that each element matches the predicate at the same position. Order is important.
    /// The number of predicates must equal the number of elements.
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool;

    /// Tests that each element satisfies the assertions at the same position. Order is important.
    /// The number of assertion closures must equal the number of elements. On failure, each
    /// unsatisfied element's captured failures are reported.
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: AssertionRenderer<[T]> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Tests multiset equality: order is ignored, but each expected element must match a distinct
    /// actual element, so duplicate counts must be identical.
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T>;

    /// Tests whether every predicate matches a distinct actual element and no actual element is
    /// left unmatched. A maximum matching is used so overlapping predicates are order-independent.
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool;

    /// Tests whether every assertion closure is satisfied by a distinct actual element and no
    /// actual element is left unmatched. A maximum matching is used so overlapping assertions
    /// are order-independent.
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);

    /// Deprecated name of [`SliceAssertions::contains_exactly_in_any_order_matching`].
    #[deprecated(
        since = "0.7.0",
        note = "renamed to `contains_exactly_in_any_order_matching`"
    )]
    #[cfg_attr(feature = "fluent", no_fluent_alias)]
    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self: Sized,
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
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
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
    {
        self.contains_exactly_in_any_order_matching(expected)
    }
}

impl<'t, T, M: Mode, R> SliceAssertions<'t, T, R> for AssertThat<'t, &[T], M, R> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>,
    {
        collection::assert_contains(&self, &expected);
        self
    }

    #[track_caller]
    fn contains_matching<P>(self, predicate: P) -> Self
    where
        R: AssertionRenderer<[T]>,
        P: Fn(&T) -> bool,
    {
        collection::assert_contains_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> Self
    where
        R: AssertionRenderer<[T]> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    {
        collection::assert_contains_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>,
    {
        collection::assert_does_not_contain(&self, &not_expected);
        self
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
    {
        collection::assert_does_not_contain_matching(&self, &predicate);
        self
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    {
        collection::assert_does_not_contain_satisfying(&self, &assertions);
        self
    }

    #[track_caller]
    fn contains_exactly<E, EE>(self, expected: EE) -> Self
    where
        E: 't,
        EE: AsRef<[E]>,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]>
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
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
    {
        collection::assert_contains_exactly_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: AssertionRenderer<[T]> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    {
        collection::assert_contains_exactly_satisfying(&self, assertions.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
    {
        collection::assert_contains_exactly_in_any_order(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
    {
        collection::assert_contains_exactly_in_any_order_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    {
        collection::assert_contains_exactly_in_any_order_satisfying(&self, assertions.as_ref());
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain an element matching the given predicate.
                    -------- assertr --------
                "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

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
            .contains("does not contain an element satisfying the given assertions.")
            .contains("Element at index 0 does not satisfy the assertions:\n    Expected: 7")
            .contains("Element at index 1 does not satisfy the assertions:\n    Expected: 7");
        }
    }

    mod does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    contains elements matching the given predicate: [
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
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    contains elements satisfying the given assertions: [
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

    mod contains_exactly {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_exact_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_different_type_combinations() {
            assert_that!(["foo".to_owned()].as_slice()).contains_exactly(["foo"]);
            assert_that!(["foo"].as_slice()).contains_exactly(["foo"]);
            assert_that!(["foo"].as_slice()).contains_exactly(["foo".to_owned()]);
            assert_that!(["foo"].as_slice()).contains_exactly(vec!["foo".to_owned()]);
            assert_that!(vec!["foo"].as_slice())
                .contains_exactly(vec!["foo".to_owned()].into_iter());
        }

        #[test]
        fn succeeds_when_exact_match_provided_as_slice() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn panics_when_not_exact_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
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
                assert_that!([1, 2, 3].as_slice())
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
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_each_element_matches_its_predicate() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_matching([
                move |it: &i32| *it == 1,
                move |it: &i32| *it < 3,
                move |it: &i32| *it > 2,
            ]);
        }

        #[test]
        fn panics_when_elements_only_match_in_a_different_order() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_matching([
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 3,
                        move |it: &i32| *it == 2,
                    ]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match predicates.

                    Details: [
                        Element at index 1 does not match its predicate: 2,
                        Element at index 2 does not match its predicate: 3,
                    ]
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_lengths_differ() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_matching(predicates);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match predicates.

                    Details: [
                        Number of elements (3) does not match number of predicates (2)!,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_satisfying {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_each_element_satisfies_its_assertions() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                },
            ]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy_its_positional_assertions() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_exactly_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(1);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(3);
                        },
                    ]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions.")
            .contains("Element at index 1 does not satisfy its assertions:\n    Expected: 3");
        }

        #[test]
        fn panics_when_lengths_differ() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_satisfying([|it: AssertThat<i32, Capture>| {
                        it.is_equal_to(1);
                    }]);
            })
            .has_type::<String>()
            .contains("Number of elements (3) does not match number of assertions (1)!");
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
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
            .contains("did not exactly satisfy the assertions in any order.")
            .contains("Elements not matched: [\n        1,\n    ]")
            .contains("Assertions not matched: 1");
        }
    }
}
