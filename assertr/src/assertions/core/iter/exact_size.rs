use crate::DebugRenderer;
use crate::{
    AssertThat, AssertionContext, Expectation, Mode, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind},
};

/// Requires [`ExactSizeIterator::len`] to equal an expected count, without advancing the iterator.
#[derive(Debug, Clone, Copy)]
pub struct HasRemainingCount(usize);
impl HasRemainingCount {
    /// Requires exactly this many remaining items.
    #[must_use]
    pub const fn new(expected: usize) -> Self {
        Self(expected)
    }
}

impl<I: ExactSizeIterator, R> Expectation<I, R> for HasRemainingCount
where
    R: ValueRenderer<usize>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        I: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        I: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a I,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let length = actual.len();
        if length == self.0 {
            Ok(())
        } else {
            Err(length)
        }
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a I, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("has remaining count"),
            Some((_, rejection)) => failure
                .relation("does not have the expected remaining count")
                .fact(Fact::labelled(
                    "Actual remaining count",
                    render.value(&rejection),
                )),
        };
        failure.expected(render.value(&self.0))
    }
}

/// Requires [`ExactSizeIterator::len`] to be zero, without advancing the iterator.
#[derive(Debug, Clone, Copy)]
pub struct HasNoRemainingElements;

impl<I: ExactSizeIterator, R> Expectation<I, R> for HasNoRemainingElements
where
    R: ValueRenderer<usize>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        I: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        I: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a I,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let length = actual.len();
        if length == 0 { Ok(()) } else { Err(length) }
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a I, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure.relation("has no remaining elements"),
            Some((_, rejection)) => failure
                .relation("unexpectedly has remaining elements")
                .fact(Fact::labelled("Remaining count", render.value(&rejection))),
        }
    }
}

/// Requires [`ExactSizeIterator::len`] to be nonzero, without advancing the iterator.
#[derive(Debug, Clone, Copy)]
pub struct HasRemainingElements;

impl<I: ExactSizeIterator, R> Expectation<I, R> for HasRemainingElements {
    type Success<'a>
        = ()
    where
        Self: 'a,
        I: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        I: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a I,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let length = actual.len();
        if length != 0 { Ok(()) } else { Err(()) }
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a I, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("has remaining elements"),
            Some((_, ())) => failure.relation("has no remaining elements"),
        }
    }
}

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
        self.apply_assertion(HasRemainingCount::new(expected))
    }

    #[track_caller]
    fn has_no_remaining_elements(self) -> Self
    where
        R: ValueRenderer<usize>,
    {
        self.apply_assertion(HasNoRemainingElements)
    }

    #[track_caller]
    fn has_remaining_elements(self) -> Self {
        self.apply_assertion(HasRemainingElements)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            [1, 2].into_iter().must().have_remaining_count(2);
            [1].into_iter().skip(1).must().have_no_remaining_elements();
            [1, 2].into_iter().must().have_remaining_elements();
        }
    }

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

        #[test]
        fn counts_use_the_active_renderer() {
            use crate::test_support::{
                CustomValueRenderer, assert_custom_fact, assert_custom_value,
            };
            let failures = assert_that!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.has_remaining_count(3).has_no_remaining_elements());
            assert_custom_value(failures[0].expected.as_ref().unwrap(), &3_usize);
            assert_custom_fact(&failures[0], "Actual remaining count", 2);
            assert_custom_fact(&failures[1], "Remaining count", 2);
        }
    }

    mod has_remaining_count {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!([1, 2].into_iter()), has_remaining_count(3));
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1, 2].into_iter()),
                has_no_remaining_elements()
            );
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!([1_i32; 0].into_iter()),
                has_remaining_elements()
            );
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
