//! Algorithms and diagnostics shared by every set assertion.
//!
//! The public [`SetAssertions`](super::SetAssertions) methods are thin wrappers around these
//! functions, so every set type produces identical failure messages.

use alloc::string::String;
use alloc::vec::Vec;

use super::SetLookup;
use crate::failure::FailureKind;
use crate::renderer::{GroupStyle, RenderingOrder};
use crate::{AssertThat, Mode, ValueRenderer};

fn type_difference_detail<S, O>() -> Option<String>
where
    S: SetLookup,
    O: SetLookup,
{
    if set_type_name::<S>() == set_type_name::<O>() {
        None
    } else {
        Some(String::from(
            "The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.",
        ))
    }
}

fn set_type_name<S: ?Sized>() -> &'static str {
    let mut name = core::any::type_name::<S>();
    while let Some(unreferenced) = name.strip_prefix('&') {
        name = unreferenced.strip_prefix("mut ").unwrap_or(unreferenced);
    }
    name
}

/// Whether diagnostics over `S`'s elements are sorted by their rendered text because the set has
/// no deterministic iteration order.
fn sorts_for_rendering<S: SetLookup + ?Sized>() -> bool {
    S::PRESENTATION.order() == RenderingOrder::SortByRenderedText
}

#[track_caller]
pub(crate) fn assert_is_subset_of<S, O, M, R>(this: &AssertThat<'_, S, M, R>, expected_superset: &O)
where
    S: SetLookup,
    O: SetLookup<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual = this.actual();

    let elements_not_in_expected = actual
        .elements()
        .filter(|it| !expected_superset.contains_element(it))
        .collect::<Vec<_>>();

    if !elements_not_in_expected.is_empty() {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("is not a subset of")
            .expected(this.render().collection(expected_superset))
            .fact(
                "Elements not in expected",
                this.render()
                    .borrowed_values::<S::Item, _>(
                        elements_not_in_expected.as_slice(),
                        GroupStyle::List,
                    )
                    .sort_for_rendering(sorts_for_rendering::<S>()),
            )
            .notes(type_difference_detail::<S, O>())
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_is_superset_of<S, O, M, R>(this: &AssertThat<'_, S, M, R>, expected_subset: &O)
where
    S: SetLookup,
    O: SetLookup<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual = this.actual();

    let elements_not_in_actual = expected_subset
        .elements()
        .filter(|it| !actual.contains_element(it))
        .collect::<Vec<_>>();

    if !elements_not_in_actual.is_empty() {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("is not a superset of")
            .expected(this.render().collection(expected_subset))
            .fact(
                "Elements not in actual",
                this.render()
                    .borrowed_values::<S::Item, _>(
                        elements_not_in_actual.as_slice(),
                        GroupStyle::List,
                    )
                    .sort_for_rendering(sorts_for_rendering::<O>()),
            )
            .notes(type_difference_detail::<S, O>())
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_is_disjoint_from<S, O, M, R>(this: &AssertThat<'_, S, M, R>, other: &O)
where
    S: SetLookup,
    O: SetLookup<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual = this.actual();

    let overlapping_elements = actual
        .elements()
        .filter(|it| other.contains_element(it))
        .collect::<Vec<_>>();

    if !overlapping_elements.is_empty() {
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("is not disjoint from")
            .expected(this.render().collection(other))
            .fact(
                "Overlapping elements",
                this.render()
                    .borrowed_values::<S::Item, _>(
                        overlapping_elements.as_slice(),
                        GroupStyle::List,
                    )
                    .sort_for_rendering(sorts_for_rendering::<S>()),
            )
            .notes(type_difference_detail::<S, O>())
            .raise();
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use crate::{prelude::*, test_support::UnorderedSet};

    use super::type_difference_detail;

    #[test]
    fn type_difference_compares_rust_types() {
        assert_that!(type_difference_detail::<BTreeSet<i32>, BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, &BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, &&BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, UnorderedSet>()).is_some();
    }
}
