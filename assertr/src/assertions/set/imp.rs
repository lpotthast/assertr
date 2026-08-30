//! Algorithms and diagnostics shared by every set assertion.
//!
//! The public [`SetAssertions`](super::SetAssertions) methods are thin wrappers around these
//! functions, so every set type produces identical failure messages.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use super::Set;
use crate::assertions::collection::{CollectionStyle, imp::TypePrefix};
use crate::{AssertThat, Mode, ValueRenderer};

fn type_difference_detail<S, O>() -> Option<String>
where
    S: Set,
    O: Set,
{
    match (S::TYPE_NAME, O::TYPE_NAME) {
        (Some(actual_type), Some(other_type)) if actual_type != other_type => Some(String::from(
            "The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.",
        )),
        _ => None,
    }
}

#[track_caller]
pub(crate) fn assert_is_subset_of<S, O, M, R>(this: &AssertThat<'_, S, M, R>, expected_superset: &O)
where
    S: Set,
    O: Set<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual_prefix = TypePrefix(S::TYPE_NAME);
    let expected_prefix = TypePrefix(O::TYPE_NAME);
    let actual = this.actual();

    let elements_not_in_expected = actual
        .elements()
        .filter(|it| !expected_superset.contains_element(it))
        .collect::<Vec<_>>();

    if !elements_not_in_expected.is_empty() {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let rendered_actual = this.render_values(&actual_values, S::STYLE);
        let expected_values = expected_superset.elements().collect::<Vec<_>>();
        let expected_superset_rendered = this.render_values(&expected_values, O::STYLE);
        let elements_rendered =
            this.render_values(elements_not_in_expected.as_slice(), CollectionStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual_prefix}{rendered_actual:#?}

                is not a subset of expected

                Expected superset: {expected_prefix}{expected_superset_rendered:#?}

                Elements not in expected: {elements_rendered:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_is_superset_of<S, O, M, R>(this: &AssertThat<'_, S, M, R>, expected_subset: &O)
where
    S: Set,
    O: Set<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual_prefix = TypePrefix(S::TYPE_NAME);
    let expected_prefix = TypePrefix(O::TYPE_NAME);
    let actual = this.actual();

    let elements_not_in_actual = expected_subset
        .elements()
        .filter(|it| !actual.contains_element(it))
        .collect::<Vec<_>>();

    if !elements_not_in_actual.is_empty() {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let rendered_actual = this.render_values(&actual_values, S::STYLE);
        let expected_values = expected_subset.elements().collect::<Vec<_>>();
        let expected_subset_rendered = this.render_values(&expected_values, O::STYLE);
        let elements_rendered =
            this.render_values(elements_not_in_actual.as_slice(), CollectionStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual_prefix}{rendered_actual:#?}

                is not a superset of expected

                Expected subset: {expected_prefix}{expected_subset_rendered:#?}

                Elements not in actual: {elements_rendered:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_is_disjoint_from<S, O, M, R>(this: &AssertThat<'_, S, M, R>, other: &O)
where
    S: Set,
    O: Set<Item = S::Item>,
    M: Mode,
    R: ValueRenderer<S::Item>,
{
    this.track_assertion();
    let actual_prefix = TypePrefix(S::TYPE_NAME);
    let other_prefix = TypePrefix(O::TYPE_NAME);
    let actual = this.actual();

    let overlapping_elements = actual
        .elements()
        .filter(|it| other.contains_element(it))
        .collect::<Vec<_>>();

    if !overlapping_elements.is_empty() {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let rendered_actual = this.render_values(&actual_values, S::STYLE);
        let other_values = other.elements().collect::<Vec<_>>();
        let other_rendered = this.render_values(&other_values, O::STYLE);
        let elements_rendered =
            this.render_values(overlapping_elements.as_slice(), CollectionStyle::List);
        this.fail_with_details(type_difference_detail::<S, O>(), |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual_prefix}{rendered_actual:#?}

                is not disjoint from expected

                Expected disjoint set: {other_prefix}{other_rendered:#?}

                Overlapping elements: {elements_rendered:#?}
            "}
        });
    }
}
