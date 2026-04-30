use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, EqContext, Mode, tracking::AssertionTracking,
};

struct CompareResult<'t, A, B> {
    strictly_equal: bool,
    same_length: bool,
    not_in_expected: Vec<&'t A>,
    not_in_actual: Vec<&'t B>,
}

impl<A, B> CompareResult<'_, A, B> {
    fn only_differing_in_order(&self) -> bool {
        !self.strictly_equal
            && self.same_length
            && self.not_in_actual.is_empty()
            && self.not_in_expected.is_empty()
    }
}

fn compare<'t, A, B, R>(
    actual: &'t VecDeque<A>,
    expected: &'t [B],
    mut ctx: Option<&mut EqContext<'_, R>>,
) -> CompareResult<'t, A, B>
where
    A: AssertrPartialEq<B, R>,
{
    let strictly_equal = actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| AssertrPartialEq::eq(actual, expected, ctx.as_deref_mut()));

    if strictly_equal {
        return CompareResult {
            strictly_equal: true,
            same_length: true,
            not_in_expected: Vec::new(),
            not_in_actual: Vec::new(),
        };
    }

    let mut not_in_expected = Vec::new();
    let mut not_in_actual = Vec::new();

    for actual in actual {
        if !expected
            .iter()
            .any(|expected| AssertrPartialEq::eq(actual, expected, ctx.as_deref_mut()))
        {
            not_in_expected.push(actual);
        }
    }

    for expected in expected {
        if !actual
            .iter()
            .any(|actual| AssertrPartialEq::eq(actual, expected, ctx.as_deref_mut()))
        {
            not_in_actual.push(expected);
        }
    }

    CompareResult {
        strictly_equal: false,
        same_length: actual.len() == expected.len(),
        not_in_expected,
        not_in_actual,
    }
}

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait VecDequeAssertions<'t, T, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>;

    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<[T]> + AssertionRenderer<T>;

    /// `P` - Predicate
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>;
}

impl<'t, T, M: Mode, R> VecDequeAssertions<'t, T, R> for AssertThat<'t, VecDeque<T>, M, R> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(actual);
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
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual();
        if actual.iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &not_expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(actual);
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
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: 't,
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<VecDeque<T>>
            + AssertionRenderer<[E]>
            + AssertionRenderer<T>
            + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual();
        let expected = expected.as_ref();

        let mut ctx = self.eq_context();
        let result = compare(actual, expected, Some(&mut ctx));

        if !result.strictly_equal {
            if !result.not_in_expected.is_empty() {
                self.add_detail_message(format!(
                    "Elements not expected: {:#?}",
                    self.render_values(result.not_in_expected.as_slice())
                ));
            }
            if !result.not_in_actual.is_empty() {
                self.add_detail_message(format!(
                    "Elements not found: {:#?}",
                    self.render_values(result.not_in_actual.as_slice())
                ));
            }
            if result.only_differing_in_order() {
                self.add_detail_message("The order of elements does not match!".to_string());
            }
            let actual = self.render_value(actual);
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
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<[T]> + AssertionRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        let expected: &[T] = expected.as_ref();

        let mut elements_found = Vec::new();
        let mut elements_not_found: Vec<&T> = Vec::new();
        let mut elements_not_expected = Vec::new();

        for e in expected {
            let found = actual.iter().find(|it| *it == e);

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
            let actual = self.render_value(actual);
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
        P: Fn(&T) -> bool,
        R: AssertionRenderer<VecDeque<T>> + AssertionRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        let expected = expected.as_ref();

        let not_matched = actual
            .iter()
            .filter(|actual| !expected.iter().any(|predicate| predicate(actual)))
            .collect::<Vec<_>>();

        if !not_matched.is_empty() {
            self.add_detail_message(format!(
                "Elements not matched: {:#?}",
                self.render_values(not_matched.as_slice())
            ));
            let actual = self.render_value(actual);

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

    mod contains_exactly_matching_in_any_order {
        use super::vec_deque;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_predicates_match() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_matching_in_any_order(
                [
                    move |it: &i32| *it == 1,
                    move |it: &i32| *it == 2,
                    move |it: &i32| *it == 3,
                ]
                .as_slice(),
            );
        }

        #[test]
        fn succeeds_when_predicates_match_in_different_order() {
            assert_that!(vec_deque([1, 2, 3])).contains_exactly_matching_in_any_order(
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
