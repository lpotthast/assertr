use crate::{failure::GenericFailure, AssertThat};
use std::fmt::Debug;

// Assertions for generic slices.
impl<'t, T> AssertThat<'t, &[T]> {
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
        let actual: &[T] = *self.actual.borrowed();
        let expected: &[T] = expected.as_ref();

        if actual != expected {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "actual: {actual:#?},\n\ndid not exactly match\n\nexpected: {expected:#?}",
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
        let actual: &[T] = *self.actual.borrowed();
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
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "actual: {actual:#?},\n\nelements expected: {expected:#?}\n\nelements not found: {elements_not_found:#?}\n\nelements not expected: {elements_not_expected:#?}",
                    actual = actual,
                    expected = expected.as_ref()
                )
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::{assert_that, assert_that_panic_by};

    #[test]
    fn is_empty_slice_succeeds_when_empty() {
        let slice: &[i32] = [].as_slice();
        assert_that(slice).is_empty();
    }

    #[test]
    fn is_empty_slice_panics_when_not_empty() {
        assert_that_panic_by(|| {
            assert_that([42].as_slice())
                .with_location(false)
                .is_empty();
        })
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
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
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {"
                -------- assertr --------
                actual: [
                    1,
                    2,
                    3,
                ],

                did not exactly match

                expected: [
                    2,
                    3,
                    4,
                ]
                -------- assertr --------
            "});
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
