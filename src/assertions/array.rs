use crate::{
    failure::GenericFailure,
    AssertThat,
};
use std::fmt::Debug;

/// Assertions for generic arrays.
impl<'t, T, const N: usize> AssertThat<'t, [T; N]> {
    #[track_caller]
    pub fn is_empty(self) -> Self
    where
        T: Debug,
    {
        if !self.actual.borrowed().as_ref().is_empty() {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\nwas expected to be empty, but it is not!",
                    actual = self.actual.borrowed(),
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn contains_exactly<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq + Debug,
    {
        self.derive(|actual| actual.as_slice())
            .contains_exactly(expected);
        self
    }

    #[track_caller]
    pub fn contains_exactly_in_any_order<E: AsRef<[T]>>(self, expected: E) -> Self
    where
        T: PartialEq + Debug,
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
    fn is_empty_succeeds_when_empty() {
        let arr: [i32; 0] = [];
        assert_that(arr).is_empty();
    }

    #[test]
    fn is_empty_panics_when_not_empty() {
        assert_that_panic_by(|| assert_that([1, 2, 3]).with_location(false).is_empty())
            .has_box_type::<String>()
            .has_debug_value(formatdoc! {r#"
                -------- assertr --------
                Actual: [1, 2, 3]

                was expected to be empty, but it is not!
                -------- assertr --------
            "#});
    }

    #[test]
    fn contains_exactly_succeeds_when_exact_match() {
        assert_that([1, 2, 3]).contains_exactly([1, 2, 3]);
    }

    #[test]
    fn contains_exactly_succeeds_when_exact_match_provided_as_slice() {
        assert_that([1, 2, 3]).contains_exactly(&[1, 2, 3]);
    }

    #[test]
    fn contains_exactly_panics_when_not_exact_match() {
        assert_that_panic_by(|| {
            assert_that([1, 2, 3])
                .with_location(false)
                .contains_exactly([3, 4, 1])
        })
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
                -------- assertr --------
                actual: [
                    1,
                    2,
                    3,
                ],

                did not exactly match

                expected: [
                    3,
                    4,
                    1,
                ]
                -------- assertr --------
            "#});
    }

    #[test]
    fn contains_exactly_in_any_order_succeeds_when_slices_match() {
        assert_that([1, 2, 3]).contains_exactly_in_any_order([2, 3, 1]);
    }

    #[test]
    fn contains_exactly_in_any_order_panics_when_slice_contains_unknown_data() {
        assert_that_panic_by(|| {
            assert_that([1, 2, 3])
                .with_location(false)
                .contains_exactly_in_any_order([2, 3, 4])
        })
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {"
                -------- assertr --------
                actual: [
                    1,
                    2,
                    3,
                ],

                elements expected: [
                    2,
                    3,
                    4,
                ]

                elements not found: [
                    4,
                ]

                elements not expected: [
                    1,
                ]
                -------- assertr --------
            "});
    }
}
