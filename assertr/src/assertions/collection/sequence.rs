//! Order-sensitive element assertions, for collections whose element order is meaningful.
//!
//! Order-free assertions live on [`CollectionAssertions`](super::CollectionAssertions). These
//! methods require [`Sequence`](super::Sequence), giving unordered subjects a focused compiler
//! diagnostic.

use super::{Collection, Sequence, imp};
use crate::{AssertThat, AssertrPartialEq, Mode, ValueRenderer, mode::Capture};

/// Assertions over the elements of a [`Sequence`](super::Sequence), in order.
///
/// Each method requires [`Sequence`](super::Sequence). A call on a set reports the missing bound
/// and recommends `contains_exactly_in_any_order`.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SequenceAssertions<T, R> {
    /// The subject collection whose element order is inspected.
    ///
    /// This associated type keeps the [`Sequence`](super::Sequence) requirement on each method,
    /// where the compiler can apply that marker trait's focused diagnostic.
    type Subject: Collection<Item = T>;

    /// Asserts positional equality with `expected`, including length.
    ///
    /// `E` is the element type of the expected values, which only has to be comparable to `T`,
    /// not identical to it. The expected values are accepted as anything viewable as `&[E]`, so
    /// arrays, slices, and `Vec`s all work.
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        Self::Subject: Sequence,
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that each element matches the predicate at the same position, including length.
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        Self::Subject: Sequence,
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool;

    /// Asserts that each element satisfies the assertions at the same position, including length.
    ///
    /// On failure, each unsatisfied element's captured failures are reported.
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        Self::Subject: Sequence,
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);
}

impl<C, M, R> SequenceAssertions<C::Item, R> for AssertThat<'_, C, M, R>
where
    C: Collection,
    M: Mode,
{
    type Subject = C;

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C: Sequence,
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_contains_exactly(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        C: Sequence,
        R: ValueRenderer<C::Item>,
        P: Fn(&C::Item) -> bool,
    {
        imp::assert_contains_exactly_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        C: Sequence,
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        imp::assert_contains_exactly_satisfying(&self, assertions.as_ref());
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains_exactly {
        use crate::prelude::*;
        use crate::{AssertrPartialEq, EqContext};
        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        enum WildcardExpected {
            Any,
            Value(u8),
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
            [1, 2, 3].as_slice().must().contain_exactly([1, 2, 3]);
        }

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
        #[cfg(feature = "derive")]
        fn reports_assertr_eq_field_differences() {
            #[derive(Debug, AssertrEq)]
            struct Record {
                pub id: u32,
            }

            assert_that_panic_by(|| {
                assert_that!([Record { id: 1 }].as_slice())
                    .with_location(false)
                    .contains_exactly([RecordAssertrEq { id: eq(2) }]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: [
                        Record {{
                            id: 1,
                        }},
                    ],

                    did not exactly match

                    Expected: [
                        RecordAssertrEq {{
                            id: Eq::Eq(2),
                        }},
                    ]

                    Details: [
                        Differences: [
                            "id": expected 2, but was 1,
                        ],
                        Elements not expected: [
                            Record {{
                                id: 1,
                            }},
                        ],
                        Elements not found: [
                            RecordAssertrEq {{
                                id: Eq::Eq(2),
                            }},
                        ],
                    ]
                    -------- assertr --------
                "#});
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

        #[test]
        #[cfg(feature = "derive")]
        fn does_not_report_positional_field_differences_when_only_differing_in_order() {
            #[derive(Debug, AssertrEq)]
            struct Record {
                pub id: u32,
            }

            assert_that_panic_by(|| {
                assert_that!([Record { id: 1 }, Record { id: 2 }].as_slice())
                    .with_location(false)
                    .contains_exactly([
                        RecordAssertrEq { id: eq(2) },
                        RecordAssertrEq { id: eq(1) },
                    ]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: [
                        Record {{
                            id: 1,
                        }},
                        Record {{
                            id: 2,
                        }},
                    ],

                    did not exactly match

                    Expected: [
                        RecordAssertrEq {{
                            id: Eq::Eq(2),
                        }},
                        RecordAssertrEq {{
                            id: Eq::Eq(1),
                        }},
                    ]

                    Details: [
                        The order of elements does not match!,
                    ]
                    -------- assertr --------
                "});
        }

        #[test]
        fn recognizes_non_equivalence_matches_that_only_differ_in_order() {
            assert_that_panic_by(|| {
                assert_that!([Actual(2), Actual(1)].as_slice())
                    .with_location(false)
                    .contains_exactly([WildcardExpected::Any, WildcardExpected::Value(2)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: [
                        Actual(
                            2,
                        ),
                        Actual(
                            1,
                        ),
                    ],

                    did not exactly match

                    Expected: [
                        Any,
                        Value(
                            2,
                        ),
                    ]

                    Details: [
                        The order of elements does not match!,
                    ]
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_with_missing_and_unexpected_elements_when_multiplicities_differ() {
            assert_that_panic_by(|| {
                assert_that!([1, 1, 2].as_slice())
                    .with_location(false)
                    .contains_exactly([1, 2, 2]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: [
                        1,
                        1,
                        2,
                    ],

                    did not exactly match

                    Expected: [
                        1,
                        2,
                        2,
                    ]

                    Details: [
                        Elements not expected: [
                            1,
                        ],
                        Elements not found: [
                            2,
                        ],
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_exactly_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 2,
                |it: &i32| *it == 3,
            ]);
        }

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].as_slice().must().contain_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

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
}
