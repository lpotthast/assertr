use crate::{
    failure::{ExpectedActualFailure, GenericFailure},
    AssertThat, Mode,
};
use std::fmt::Debug;

// Assertions for generic slices.
impl<'t, T, M: Mode> AssertThat<'t, &[T], M> {
    #[track_caller]
    pub fn is_empty(self) -> Self
    where
        T: Debug,
    {
        if !self.actual().borrowed().as_ref().is_empty() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\nwas expected to be empty, but it is not!",
                    actual = self.actual().borrowed(),
                ),
            });
        }
        self
    }

    // TODO: Test
    #[track_caller]
    pub fn has_length(mut self, expected: usize) -> Self
    where
        T: Debug,
    {
        let actual = self.actual().borrowed().as_ref().len();
        if actual != expected {
            self = self.with_detail_message("Slice was not of expected length!");
            self.fail(ExpectedActualFailure {
                expected: &expected,
                actual: &actual,
            });
        }
        self
    }

    /// Test that the subject contains exactly the expected elements. Order is important. Lengths must be identical.
    ///
    /// T: Original subject type. The "actual value" is of type &[T] (slice T).
    /// E: Type of elements in our "expected value" slice.
    /// EE: The "expected value". Anything that can be seen as &[E] (slice E). Having this extra type, instead of directly accepting `&[E]` allows us to be generic over the input in both internal type and slice representation.
    #[track_caller]
    pub fn contains_exactly<E, EE>(self, expected: EE) -> Self
    where
        E: Debug + 't,
        EE: AsRef<[E]>,
        T: PartialEq<E> + Debug,
    {
        let actual = *self.actual().borrowed();
        let expected = expected.as_ref();

        let result = crate::util::slice::compare(actual, expected);

        if !result.strictly_equal {
            #[allow(dropping_references)]
            drop(actual);

            if !result.not_in_b.is_empty() {
                self.add_detail_message(format!("Elements not expected: {:#?}", result.not_in_b));
            }
            if !result.not_in_a.is_empty() {
                self.add_detail_message(format!("Elements not found: {:#?}", result.not_in_a));
            }
            if result.only_differing_in_order {
                self.add_detail_message("The order of elements does not match!");
            }

            let actual = self.actual().borrowed();

            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?},\n\ndid not exactly match\n\nExpected: {expected:#?}",
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq + Debug,
    {
        let actual: &[T] = self.actual().borrowed();
        let expected: &[T] = expected.as_ref();

        let mut elements_found = Vec::new();
        let mut elements_not_found: Vec<&T> = Vec::new();
        let mut elements_not_expected = Vec::new();

        for e in expected.iter() {
            let found = actual.as_ref().iter().find(|it| *it == e);

            match found {
                Some(_e) => elements_found.push(e),
                None => elements_not_found.push(e),
            }
        }

        for e in actual.iter() {
            match elements_found.iter().find(|it| **it == e) {
                Some(_) => {}
                None => elements_not_expected.push(e),
            }
        }

        if !elements_not_found.is_empty() || !elements_not_expected.is_empty() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?},\n\nElements expected: {expected:#?}\n\nElements not found: {elements_not_found:#?}\n\nElements not expected: {elements_not_expected:#?}",
                    actual = actual,
                    expected = expected
                )
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn is_empty_slice_succeeds_when_empty() {
        let slice: &[i32] = [].as_slice();
        assert_that(slice).is_empty();
    }

    #[test]
    fn is_empty_slice_panics_when_not_empty() {
        assert_that_panic_by(|| {
            assert_that([42].as_slice()).with_location(false).is_empty();
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: [42]

                was expected to be empty, but it is not!
                -------- assertr --------
            "#});
    }

    #[test]
    fn contains_exactly_succeeds_when_exact_match() {
        assert_that([1, 2, 3].as_slice()).contains_exactly([1, 2, 3]);
    }

    #[test]
    fn contains_exactly_compiles_for_different_type_combinations() {
        assert_that(["foo".to_owned()].as_slice()).contains_exactly(["foo"]);
        assert_that(["foo"].as_slice()).contains_exactly(["foo"]);
        assert_that(["foo"].as_slice()).contains_exactly(["foo".to_owned()]);
        assert_that(["foo"].as_slice()).contains_exactly(vec!["foo".to_owned()]);
        assert_that(vec!["foo"].as_slice()).contains_exactly(vec!["foo".to_owned()].into_iter());
    }

    #[test]
    fn contains_exactly_succeeds_when_exact_match_provided_as_slice() {
        assert_that([1, 2, 3].as_slice()).contains_exactly(&[1, 2, 3]);
    }

    #[test]
    fn contains_exactly_panics_when_not_exact_match() {
        assert_that_panic_by(|| {
            assert_that([1, 2, 3].as_slice())
                .with_location(false)
                .contains_exactly([2, 3, 4])
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
            "#});
    }

    #[test]
    fn contains_exactly_panics_with_detail_message_when_only_differing_in_order() {
        assert_that_panic_by(|| {
            assert_that([1, 2, 3].as_slice())
                .with_location(false)
                .contains_exactly([3, 2, 1])
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
                    2,
                    1,
                ]

                Details: [
                    The order of elements does not match!,
                ]
                -------- assertr --------
            "#});
    }

    #[test]
    fn contains_exactly_in_any_order_succeeds_when_slices_match() {
        assert_that([1, 2, 3].as_slice()).contains_exactly_in_any_order([2, 3, 1]);
    }

    #[test]
    fn contains_exactly_in_any_order_panics_when_slice_contains_unknown_data() {
        assert_that_panic_by(|| {
            assert_that([1, 2, 3].as_slice())
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
