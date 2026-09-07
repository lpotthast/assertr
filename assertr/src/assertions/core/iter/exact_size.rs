use crate::{AssertThat, DebugRenderer, Fact, Mode, ValueRenderer, failure::FailureKind};

/// Non-consuming assertions for the exact number of elements remaining in an iterator.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ExactSizeIteratorAssertions<R = DebugRenderer> {
    /// Asserts that [`ExactSizeIterator::len`] equals `expected` without advancing the iterator.
    fn has_remaining_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>;
    /// Asserts that no elements remain without advancing the iterator.
    fn has_no_remaining_elements(self) -> Self
    where
        R: ValueRenderer<usize>;
    /// Asserts that at least one element remains without advancing the iterator.
    fn has_remaining_elements(self) -> Self;
}

impl<I: ExactSizeIterator, M: Mode, R> ExactSizeIteratorAssertions<R> for AssertThat<'_, I, M, R> {
    #[track_caller]
    fn has_remaining_count(self, expected: usize) -> Self
    where
        R: ValueRenderer<usize>,
    {
        self.track_assertion();
        let actual = self.actual().len();
        if actual != expected {
            self.failure(FailureKind::Length)
                .relation("does not have the expected remaining count")
                .expected(self.render().value(&expected))
                .fact(Fact::labelled(
                    "Actual remaining count",
                    self.render().value(&actual),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_no_remaining_elements(self) -> Self
    where
        R: ValueRenderer<usize>,
    {
        self.track_assertion();
        let actual = self.actual().len();
        if actual != 0 {
            self.failure(FailureKind::Length)
                .relation("unexpectedly has remaining elements")
                .fact(Fact::labelled(
                    "Remaining count",
                    self.render().value(&actual),
                ))
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_remaining_elements(self) -> Self {
        self.track_assertion();
        if self.actual().len() == 0 {
            self.failure(FailureKind::Length)
                .relation("has no remaining elements")
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, core::array::IntoIter<i32, 1>, Panic, NoRenderer>
                    => ExactSizeIteratorAssertions<NoRenderer>
            );
        }
    }

    mod has_remaining_count {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].into_iter().must().have_remaining_count(2);
        }

        #[test]
        fn remaining_counts_are_rendered_usize_values() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, assert_custom_value};
            let failures = assert_that!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.has_remaining_count(3));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                does not have the expected remaining count

                Expected: custom(3)

                Details:
                  - Actual remaining count: custom(2)
                -------- assertr --------
            "});

            assert_custom_value(&failures[0].facts[0].value, &2_usize);
            assert_custom_value(failures[0].expected.as_ref().unwrap(), &3_usize);
        }

        #[test]
        fn succeeds_when_count_matches_without_advancing() {
            let mut iterator = [1, 2, 3].into_iter();
            assert_that!(iterator.next()).is_equal_to(Some(1));
            assert_that!(iterator)
                .has_remaining_count(2)
                .has_remaining_count(2);
        }

        #[test]
        fn panics_when_count_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].into_iter())
                    .with_location(false)
                    .has_remaining_count(3);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2].into_iter()`

                    does not have the expected remaining count

                    Expected: 3

                    Details:
                      - Actual remaining count: 2
                    -------- assertr --------
                "});
        }
    }

    mod has_no_remaining_elements {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1].into_iter().skip(1).must().have_no_remaining_elements();
        }

        #[test]
        fn remaining_counts_are_rendered_usize_values() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, assert_custom_value};
            let failures = assert_that!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ExactSizeIteratorAssertions::has_no_remaining_elements);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                unexpectedly has remaining elements

                Details:
                  - Remaining count: custom(2)
                -------- assertr --------
            "});

            assert_custom_value(&failures[0].facts[0].value, &2_usize);
        }

        #[test]
        fn succeeds_when_no_elements_remain() {
            assert_that!([1].into_iter().skip(1)).has_no_remaining_elements();
        }

        #[test]
        fn panics_when_elements_remain() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].into_iter())
                    .with_location(false)
                    .has_no_remaining_elements();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2].into_iter()`

                    unexpectedly has remaining elements

                    Details:
                      - Remaining count: 2
                    -------- assertr --------
                "});
        }
    }

    mod has_remaining_elements {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].into_iter().must().have_remaining_elements();
        }

        #[test]
        fn succeeds_when_elements_remain_without_advancing() {
            assert_that!([1, 2].into_iter())
                .has_remaining_elements()
                .has_remaining_count(2);
        }

        #[test]
        fn panics_when_no_elements_remain() {
            assert_that_panic_by(|| {
                assert_that!([1_i32; 0].into_iter())
                    .with_location(false)
                    .has_remaining_elements();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1_i32; 0].into_iter()`

                    has no remaining elements
                    -------- assertr --------
                "});
        }

        #[test]
        fn requires_no_numeric_renderer() {
            use crate::test_support::NoRenderer;
            assert_that!([1].into_iter())
                .with_renderer(NoRenderer)
                .with_location(false)
                .has_remaining_elements();
        }
    }
}
