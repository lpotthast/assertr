//! Ordinary value comparisons for finite collections.
//!
//! Membership and positional equality share `PartialEq` evaluation with `equal_to`. Expected-side
//! matching and captured assertions use the policies in `crate::matchers`.

use alloc::vec::Vec;

use super::{Collection, StableOrder};
use crate::failure::{Fact, FailureBuilder, FailureKind};
use crate::renderer::{GroupStyle, RenderingOrder};
use crate::{AssertThat, Mode, ValueRenderer, util::matching::match_bipartite};

pub(crate) struct ExactCompareResult<'t, T, E> {
    pub(crate) strictly_equal: bool,
    pub(crate) same_length: bool,
    /// Actual elements that have no equal in `expected`.
    pub(crate) not_in_expected: Vec<&'t T>,
    /// Expected elements that have no equal in the actual collection.
    pub(crate) not_in_actual: Vec<&'t E>,
}

impl<T, E> ExactCompareResult<'_, T, E> {
    pub(crate) fn only_differing_in_order(&self) -> bool {
        !self.strictly_equal
            && self.same_length
            && self.not_in_expected.is_empty()
            && self.not_in_actual.is_empty()
    }
}

/// `PartialEq` like, order-respecting comparison of a collection against expected elements,
/// collecting the elements missing on either side when the inputs are not strictly equal.
pub(crate) fn compare<'t, C, T, E>(actual: &'t C, expected: &'t [E]) -> ExactCompareResult<'t, T, E>
where
    C: Collection<Item = T> + ?Sized,
    T: PartialEq<E>,
{
    let same_length = actual.length() == expected.len();
    let strictly_equal = same_length
        && actual
            .elements()
            .zip(expected)
            .all(|(actual, expected)| crate::matchers::equals(actual, expected));

    if strictly_equal {
        return ExactCompareResult {
            strictly_equal: true,
            same_length: true,
            not_in_expected: Vec::new(),
            not_in_actual: Vec::new(),
        };
    }

    let elements = actual.elements().collect::<Vec<_>>();
    let matched = match_bipartite(
        elements.len(),
        expected.len(),
        |actual_index, expected_index| {
            crate::matchers::equals(elements[actual_index], &expected[expected_index])
        },
    );
    let not_in_expected = matched
        .unmatched_actual
        .iter()
        .map(|index| elements[*index])
        .collect();
    let not_in_actual = matched
        .unmatched_expected
        .iter()
        .map(|index| &expected[*index])
        .collect();

    ExactCompareResult {
        strictly_equal: false,
        same_length,
        not_in_expected,
        not_in_actual,
    }
}

/// Whether diagnostics over `C`'s elements are sorted by their rendered text because the collection
/// has no deterministic iteration order.
fn sorts_for_rendering<C: Collection + ?Sized>() -> bool {
    C::PRESENTATION.order() == RenderingOrder::SortByRenderedText
}

#[track_caller]
pub(crate) fn assert_contains<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &E)
where
    C: Collection<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if !actual
        .elements()
        .any(|it| crate::matchers::equals(it, expected))
    {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("does not contain")
            .expected(this.render().value(expected))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_all<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: Collection<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let not_found = expected
        .iter()
        .filter(|expected| {
            !actual
                .elements()
                .any(|it| crate::matchers::equals(it, expected))
        })
        .collect::<Vec<_>>();

    if !not_found.is_empty() {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("does not contain all of")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            )
            .fact(Fact::labelled(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(not_found.as_slice(), GroupStyle::List),
            ))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    not_expected: &E,
) where
    C: Collection<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if actual
        .elements()
        .any(|it| crate::matchers::equals(it, not_expected))
    {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("contains")
            .unexpected(this.render().value(not_expected))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_starts_with<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    this.track_assertion();
    let actual = this.actual();
    let mismatch = actual
        .elements()
        .zip(expected)
        .enumerate()
        .find(|(_, (actual, expected))| !crate::matchers::equals(*actual, *expected));

    if actual.length() < expected.len() || mismatch.is_some() {
        let mut failure = this
            .failure(FailureKind::Membership)
            .actual(this.render().stable_collection(actual))
            .relation("does not start with")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        if actual.length() < expected.len() {
            failure = failure.fact(Fact::labelled(
                "Actual length",
                this.render().value(&actual.length()),
            ));
        }
        if let Some((index, (element, expected))) = mismatch {
            failure = failure.child(
                FailureBuilder::detached::<T>(FailureKind::Equality)
                    .actual(this.render().value(element))
                    .expected(this.render().value(expected))
                    .build()
                    .located_at(Fact::index(index)),
            );
        }
        failure.raise();
    }
}

#[track_caller]
pub(crate) fn assert_ends_with<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    this.track_assertion();
    let actual = this.actual();
    let offset = actual.length().saturating_sub(expected.len());
    let mismatch = actual
        .elements()
        .skip(offset)
        .zip(expected)
        .enumerate()
        .find(|(_, (actual, expected))| !crate::matchers::equals(*actual, *expected));

    if actual.length() < expected.len() || mismatch.is_some() {
        let mut failure = this
            .failure(FailureKind::Membership)
            .actual(this.render().stable_collection(actual))
            .relation("does not end with")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        if actual.length() < expected.len() {
            failure = failure.fact(Fact::labelled(
                "Actual length",
                this.render().value(&actual.length()),
            ));
        }
        if let Some((index, (element, expected))) = mismatch {
            failure = failure.child(
                FailureBuilder::detached::<T>(FailureKind::Equality)
                    .actual(this.render().value(element))
                    .expected(this.render().value(expected))
                    .build()
                    .located_at(Fact::index(offset + index)),
            );
        }
        failure.raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[E],
) where
    C: StableOrder<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();
    let found = expected.is_empty()
        || elements.windows(expected.len()).any(|window| {
            window
                .iter()
                .zip(expected)
                .all(|(actual, expected)| crate::matchers::equals(*actual, expected))
        });

    if !found {
        this.failure(FailureKind::Membership)
            .actual(this.render().stable_collection(actual))
            .relation("does not contain the contiguous subsequence")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            )
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let result = compare(actual, expected);

    if !result.strictly_equal {
        let only_differing_in_order = result.only_differing_in_order();
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(this.render().stable_collection(actual))
            .relation("does not contain exactly")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );

        if !result.not_in_expected.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Elements not expected",
                this.render()
                    .borrowed_values::<T, _>(result.not_in_expected.as_slice(), GroupStyle::List),
            ));
        }
        if !result.not_in_actual.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(result.not_in_actual.as_slice(), GroupStyle::List),
            ));
        }
        if only_differing_in_order {
            failure = failure.fact(Fact::note("Only the order of the elements differs."));
        }
        failure.raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[E],
) where
    C: Collection<Item = T>,
    T: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();

    let result = match_bipartite(
        elements.len(),
        expected.len(),
        |actual_index, expected_index| {
            crate::matchers::equals(elements[actual_index], &expected[expected_index])
        },
    );

    if !result.is_exact() {
        let elements_not_found = result
            .unmatched_expected
            .iter()
            .map(|index| &expected[*index])
            .collect::<Vec<_>>();
        let elements_not_expected = result
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect::<Vec<_>>();
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(this.render().collection(actual))
            .relation("does not contain exactly in any order")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        if !elements_not_found.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(elements_not_found.as_slice(), GroupStyle::List),
            ));
        }
        if !elements_not_expected.is_empty() {
            failure = failure.fact(Fact::labelled(
                "Elements not expected",
                this.render()
                    .borrowed_values::<T, _>(elements_not_expected.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            ));
        }
        failure.raise();
    }
}

#[cfg(test)]
mod tests {
    mod compare {
        use crate::assertions::collection::value::{ExactCompareResult, compare};
        use crate::prelude::*;

        fn compare_slices<'t, A, B>(aa: &'t [A], bb: &'t [B]) -> ExactCompareResult<'t, A, B>
        where
            A: PartialEq<B>,
        {
            compare::<_, A, B>(aa, bb)
        }

        #[test]
        fn returns_equal_on_equal_input_using_refs() {
            let result = compare_slices(&[&1, &2, &3], &[&1, &2, &3]);

            assert_that!(result.only_differing_in_order()).is_false();
            assert_that!(result.strictly_equal).is_true();
            assert_that!(result.same_length).is_true();
            assert_that!(result.not_in_actual).is_empty();
            assert_that!(result.not_in_expected).is_empty();
        }

        #[test]
        fn returns_equal_on_equal_input() {
            let result = compare_slices(&[1, 2, 3], &[1, 2, 3]);

            assert_that!(result.only_differing_in_order()).is_false();
            assert_that!(result.strictly_equal).is_true();
            assert_that!(result.same_length).is_true();
            assert_that!(result.not_in_actual).is_empty();
            assert_that!(result.not_in_expected).is_empty();
        }

        #[test]
        fn returns_not_equal_on_equal_but_rearranged_input() {
            let result = compare_slices(&[1, 2, 3], &[3, 2, 1]);

            assert_that!(result.only_differing_in_order()).is_true();
            assert_that!(result.strictly_equal).is_false();
            assert_that!(result.same_length).is_true();
            assert_that!(result.not_in_actual).is_empty();
            assert_that!(result.not_in_expected).is_empty();
        }

        #[test]
        fn returns_not_equal_and_lists_differences_on_differing_input() {
            let result = compare_slices(&[1, 5, 7], &[5, 3, 4, 42]);

            assert_that!(result.only_differing_in_order()).is_false();
            assert_that!(result.strictly_equal).is_false();
            assert_that!(result.same_length).is_false();
            assert_that!(result.not_in_actual.as_slice()).is_equal_to([&3, &4, &42].as_slice());
            assert_that!(result.not_in_expected.as_slice()).is_equal_to([&1, &7].as_slice());
        }

        #[test]
        fn returns_not_equal_and_lists_differences_when_multiplicities_differ() {
            let result = compare_slices(&[1, 1, 2], &[1, 2, 2]);

            assert_that!(result.only_differing_in_order()).is_false();
            assert_that!(result.strictly_equal).is_false();
            assert_that!(result.same_length).is_true();
            assert_that!(result.not_in_actual).contains_exactly([&2]);
            assert_that!(result.not_in_expected).contains_exactly([&1]);
        }
    }
}
