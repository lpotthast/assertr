use core::fmt::Debug;

use crate::{AssertThat, Mode, prelude::SliceAssertions};

pub trait ArrayAssertions<T: Debug> {
    fn contains_exactly<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq;

    fn contain_exactly<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        Self: Sized,
    {
        self.contains_exactly(expected)
    }

    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq;

    fn contain_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
        Self: Sized,
    {
        self.contains_exactly_in_any_order(expected)
    }
}

/// Assertions for generic arrays.
impl<T: Debug, const N: usize, M: Mode> ArrayAssertions<T> for AssertThat<'_, [T; N], M> {
    #[track_caller]
    fn contains_exactly<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
    {
        self.derive(|actual| actual.as_slice())
            .contains_exactly(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq,
    {
        self.derive(|actual| actual.as_slice())
            .contains_exactly_in_any_order(expected);
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn contains_exactly_succeeds_when_exact_match() {
        assert_that!([1, 2, 3]).contains_exactly([1, 2, 3]);
    }

    #[test]
    fn contains_exactly_succeeds_when_exact_match_provided_as_slice() {
        assert_that!([1, 2, 3]).contains_exactly(&[1, 2, 3]);
    }

    #[test]
    fn contains_exactly_panics_when_not_exact_match() {
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

    #[test]
    fn contains_exactly_panics_with_detail_message_when_only_differing_in_order() {
        assert_that_panic_by(|| {
            assert_that!([1, 2, 3])
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
        assert_that!([1, 2, 3]).contains_exactly_in_any_order([2, 3, 1]);
    }

    #[test]
    fn contains_exactly_in_any_order_panics_when_slice_contains_unknown_data() {
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
