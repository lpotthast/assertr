//! Algorithms and diagnostics shared by every set assertion.
//!
//! The public [`SetAssertions`](super::SetAssertions) methods are thin wrappers around these
//! functions, so every set type produces identical failure messages.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use super::SetLookup;
use crate::renderer::GroupStyle;
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
        let rendered_actual = this.render().collection(actual);
        let expected_superset_rendered = this.render().collection(expected_superset);
        let elements_rendered = this
            .render()
            .borrowed_values::<S::Item, _>(elements_not_in_expected.as_slice(), GroupStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                is not a subset of expected

                Expected superset: {expected_superset_rendered:#?}

                Elements not in expected: {elements_rendered:#?}
            "}
        });
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
        let rendered_actual = this.render().collection(actual);
        let expected_subset_rendered = this.render().collection(expected_subset);
        let elements_rendered = this
            .render()
            .borrowed_values::<S::Item, _>(elements_not_in_actual.as_slice(), GroupStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                is not a superset of expected

                Expected subset: {expected_subset_rendered:#?}

                Elements not in actual: {elements_rendered:#?}
            "}
        });
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
        let rendered_actual = this.render().collection(actual);
        let other_rendered = this.render().collection(other);
        let elements_rendered = this
            .render()
            .borrowed_values::<S::Item, _>(overlapping_elements.as_slice(), GroupStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                is not disjoint from expected

                Expected disjoint set: {other_rendered:#?}

                Overlapping elements: {elements_rendered:#?}
            "}
        });
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
