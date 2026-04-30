use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use crate::{AssertThat, AssertionRenderer, AssertrPartialEq, Mode, tracking::AssertionTracking};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SliceAssertions<'t, T, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>;

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

    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T>;

    /// `P` - Predicate
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool;
}

impl<'t, T, M: Mode, R> SliceAssertions<'t, T, R> for AssertThat<'t, &[T], M, R> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual();
        let expected = expected;
        if !actual.iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(*actual);
            let expected = self.render_value(&expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    does not contain expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<[T]> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual();

        if actual.iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &not_expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(*actual);
            let not_expected = self.render_value(&not_expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    contains unexpected: {not_expected:#?}
                "}
            });
        }
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
        self.track_assertion();
        let actual = *self.actual();
        let expected = expected.as_ref();

        let mut ctx = self.eq_context();
        let result = crate::util::slice::compare_with_context(actual, expected, Some(&mut ctx));

        if !result.strictly_equal {
            if !result.not_in_b.is_empty() {
                self.add_detail_message(format!(
                    "Elements not expected: {:#?}",
                    self.render_values(result.not_in_b.as_slice())
                ));
            }
            if !result.not_in_a.is_empty() {
                self.add_detail_message(format!(
                    "Elements not found: {:#?}",
                    self.render_values(result.not_in_a.as_slice())
                ));
            }
            if result.only_differing_in_order() {
                self.add_detail_message("The order of elements does not match!".to_owned());
            }

            let actual = self.render_value(*self.actual());
            let expected = self.render_value(expected);

            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?},

                    did not exactly match

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
    {
        self.track_assertion();
        let actual: &[T] = self.actual();
        let expected: &[T] = expected.as_ref();

        let mut elements_found = Vec::new();
        let mut elements_not_found: Vec<&T> = Vec::new();
        let mut elements_not_expected = Vec::new();

        for e in expected {
            let found = actual.as_ref().iter().find(|it| *it == e);

            match found {
                Some(_e) => elements_found.push(e),
                None => elements_not_found.push(e),
            }
        }

        for e in actual {
            match elements_found.iter().find(|it| **it == e) {
                Some(_) => {}
                None => elements_not_expected.push(e),
            }
        }

        if !elements_not_found.is_empty() || !elements_not_expected.is_empty() {
            let actual = self.render_value(*self.actual());
            let expected = self.render_value(expected);
            let elements_not_found = self.render_values(elements_not_found.as_slice());
            let elements_not_expected = self.render_values(elements_not_expected.as_slice());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?},

                    Elements expected: {expected:#?}

                    Elements not found: {elements_not_found:#?}

                    Elements not expected: {elements_not_expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: AssertionRenderer<[T]> + AssertionRenderer<T>,
        P: Fn(&T) -> bool,
    {
        self.track_assertion();
        let actual = *self.actual();
        let expected = expected.as_ref();

        let result = crate::util::slice::test_matching_any(actual, expected);

        if !result.not_matched.is_empty() {
            self.add_detail_message(format!(
                "Elements not matched: {:#?}",
                self.render_values(result.not_matched.as_slice())
            ));
            let actual = self.render_value(*self.actual());

            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?},

                    did not exactly match predicates in any order.
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
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

    mod contains_exactly_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
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

    mod contains_exactly_matching_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_slices_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_matching_in_any_order(
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
            assert_that!([1, 2, 3].as_slice()).contains_exactly_matching_in_any_order(
                [
                    move |it: &i32| *it == 3,
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                ]
                .as_slice(),
            );
        }

        #[test]
        fn panics_when_slice_contains_non_matching_data() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
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
