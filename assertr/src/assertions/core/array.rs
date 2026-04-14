use core::fmt::Debug;

use crate::{AssertThat, AssertrPartialEq, Mode, prelude::SliceAssertions};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ArrayAssertions<'t, T: Debug> {
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug;

    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq;

    /// `P` - Predicate
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool;
}

/// Assertions for generic arrays.
impl<'t, T: Debug, const N: usize, M: Mode> ArrayAssertions<'t, T> for AssertThat<'t, [T; N], M> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(<[T; N]>::as_slice).contains(expected);
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(<[T; N]>::as_slice)
            .does_not_contain(not_expected);
        self
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(<[T; N]>::as_slice).contains_exactly(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
    {
        self.derive(<[T; N]>::as_slice)
            .contains_exactly_in_any_order(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
    {
        self.derive(<[T; N]>::as_slice)
            .contains_exactly_matching_in_any_order(expected);
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_value_is_present() {
            assert_that!([1, 2, 3]).contains(2);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3]).with_location(false).contains(4);
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

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_value_is_absent() {
            assert_that!([1, 2, 3]).does_not_contain(4);
        }

        #[test]
        fn panics_when_value_is_present() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
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
            assert_that!([1, 2, 3]).contains_exactly([1, 2, 3]);
            assert_that!([1, 2, 3]).contains_exactly(&[1, 2, 3]);
            assert_that!(["foo".to_owned()]).contains_exactly(["foo"]);
        }

        #[test]
        fn panics_when_not_exact_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .contains_exactly([3, 4, 1])
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match

                    Expected: [
                        3,
                        4,
                        1,
                    ]

                    Details: [
                        Elements not expected: [
                            2,
                        ],
                        Elements not found: [
                            4,
                        ],
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_values_match() {
            assert_that!([1, 2, 3]).contains_exactly_in_any_order([2, 3, 1]);
        }

        #[test]
        fn panics_when_value_is_missing() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 3, 4])
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

    mod contains_exactly_matching_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_predicates_match() {
            assert_that!([1, 2, 3]).contains_exactly_matching_in_any_order(
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
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .contains_exactly_matching_in_any_order(
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
                    ]
                    -------- assertr --------
                "});
        }
    }
}
