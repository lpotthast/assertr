//! Algorithms and diagnostics shared by every collection assertion.
//!
//! The public [`CollectionAssertions`](super::CollectionAssertions) methods are thin wrappers
//! around these functions, so every collection type produces identical failure messages.

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use super::{Collection, StableOrder};
use crate::renderer::GroupStyle;
use crate::{
    AssertThat, AssertrPartialEq, EqContext, Mode, ValueRenderer, mode::Capture,
    renderer::omission, util::failure::join_failures, util::matching::match_bipartite,
};

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
pub(crate) fn compare<'t, C, T, E, R>(
    actual: &'t C,
    expected: &'t [E],
    mut ctx: Option<&mut EqContext<'_, R>>,
) -> ExactCompareResult<'t, T, E>
where
    C: Collection<Item = T> + ?Sized,
    T: AssertrPartialEq<E, R>,
{
    let same_length = actual.length() == expected.len();
    let strictly_equal = same_length
        && actual
            .elements()
            .zip(expected)
            .all(|(actual, expected)| AssertrPartialEq::eq(actual, expected, ctx.as_deref_mut()));

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
            if let Some(ctx) = ctx.as_deref() {
                let mut probe_ctx = ctx.fork();
                AssertrPartialEq::eq(
                    elements[actual_index],
                    &expected[expected_index],
                    Some(&mut probe_ctx),
                )
            } else {
                AssertrPartialEq::eq(elements[actual_index], &expected[expected_index], None)
            }
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

#[track_caller]
pub(crate) fn assert_contains<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &E)
where
    C: Collection<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if !actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, expected, Some(&mut ctx))
    }) {
        let actual = this.render().collection(actual);
        let expected = this.render().value(expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain expected: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_all<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: Collection<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let not_found = expected
        .iter()
        .filter(|expected| {
            !actual.elements().any(|it| {
                let mut ctx = this.eq_context();
                <_ as AssertrPartialEq<_, R>>::eq(it, expected, Some(&mut ctx))
            })
        })
        .collect::<Vec<_>>();

    if !not_found.is_empty() {
        let rendered_actual = this.render().collection(actual);
        let rendered_expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        let not_found = this
            .render()
            .borrowed_values::<E, _>(not_found.as_slice(), GroupStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain all expected elements

                Expected: {rendered_expected:#?}

                Elements not found: {not_found:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    not_expected: &E,
) where
    C: Collection<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, not_expected, Some(&mut ctx))
    }) {
        let actual = this.render().collection(actual);
        let not_expected = this.render().value(not_expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                contains unexpected: {not_expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_matching<C, T, P, M, R>(this: &AssertThat<'_, C, M, R>, predicate: &P)
where
    C: Collection<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    if !actual.elements().any(predicate) {
        let actual = this.render().collection(actual);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain an element matching the given predicate.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &A,
) where
    C: Collection<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    let maximum = this.render().max_items();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_elements = 0_usize;
    for element in actual.elements() {
        let failures = this.collect_element_failures(element, assertions);
        if failures.is_empty() {
            return;
        }
        number_of_unsatisfied_elements += 1;
        if unsatisfied.len() < maximum {
            unsatisfied.push(failures);
        }
    }

    let mut details = Vec::new();
    for failures in &unsatisfied {
        details.push(format!(
            "An element does not satisfy the assertions:\n{}",
            join_failures(failures, this.render().max_items())
        ));
    }
    let omitted = number_of_unsatisfied_elements - unsatisfied.len();
    if omitted != 0 {
        details.push(omission(omitted, "unsatisfied element"));
    }

    let actual = this.render().collection(actual);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w, r"
            Actual: {actual:#?}

            does not contain an element satisfying the given assertions.
        "}
    });
}

#[track_caller]
pub(crate) fn assert_does_not_contain_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicate: &P,
) where
    C: Collection<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let matching = actual
        .elements()
        .filter(|it| predicate(it))
        .collect::<Vec<_>>();
    if !matching.is_empty() {
        let matching = this
            .render()
            .borrowed_values::<T, _>(matching.as_slice(), GroupStyle::List);
        let actual = this.render().collection(actual);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                unexpectedly contains elements matching the given predicate: {matching:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &A,
) where
    C: Collection<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let satisfying = actual
        .elements()
        .filter(|element| {
            this.collect_element_failures(*element, assertions)
                .is_empty()
        })
        .collect::<Vec<_>>();
    if !satisfying.is_empty() {
        let satisfying = this
            .render()
            .borrowed_values::<T, _>(satisfying.as_slice(), GroupStyle::List);
        let actual = this.render().collection(actual);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                unexpectedly contains elements satisfying the given assertions: {satisfying:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_starts_with<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    let mismatch = actual
        .elements()
        .zip(expected)
        .position(|(actual, expected)| {
            !AssertrPartialEq::eq(actual, expected, Some(&mut this.eq_context()))
        });

    if actual.length() < expected.len() || mismatch.is_some() {
        let mut details = Vec::new();
        if actual.length() < expected.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than prefix length {}.",
                actual.length(),
                expected.len()
            ));
        }
        if let Some(index) = mismatch {
            details.push(format!("Prefix differs at zero-based index {index}."));
        }
        let actual = this.render().stable_collection(actual);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not start with expected prefix: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_starts_with_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicates: &[P],
) where
    C: StableOrder<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let mismatch = actual
        .elements()
        .zip(predicates)
        .position(|(actual, predicate)| !predicate(actual));

    if actual.length() < predicates.len() || mismatch.is_some() {
        let mut details = Vec::new();
        if actual.length() < predicates.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than predicate prefix length {}.",
                actual.length(),
                predicates.len()
            ));
        }
        if let Some(index) = mismatch {
            details.push(format!(
                "Element at zero-based index {index} does not match its prefix predicate."
            ));
        }
        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not start with elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_starts_with_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &[A],
) where
    C: StableOrder<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let maximum = this.render().max_items();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_elements = 0_usize;
    for (index, (element, assertions)) in actual.elements().zip(assertions).enumerate() {
        let failures = this.collect_element_failures(element, assertions);
        if !failures.is_empty() {
            number_of_unsatisfied_elements += 1;
            if unsatisfied.len() < maximum {
                unsatisfied.push((index, failures));
            }
        }
    }

    if actual.length() < assertions.len() || number_of_unsatisfied_elements != 0 {
        let mut details = Vec::new();
        if actual.length() < assertions.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than assertion prefix length {}.",
                actual.length(),
                assertions.len()
            ));
        }
        for (index, failures) in unsatisfied {
            details.push(format!(
                "Element at index {index} does not satisfy its prefix assertions:\n{}",
                join_failures(&failures, this.render().max_items())
            ));
        }
        let omitted = number_of_unsatisfied_elements.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "unsatisfied prefix element"));
        }
        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not start with elements satisfying the assertions.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_ends_with<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    let offset = actual.length().saturating_sub(expected.len());
    let mismatch = actual
        .elements()
        .skip(offset)
        .zip(expected)
        .position(|(actual, expected)| {
            !AssertrPartialEq::eq(actual, expected, Some(&mut this.eq_context()))
        });

    if actual.length() < expected.len() || mismatch.is_some() {
        let mut details = Vec::new();
        if actual.length() < expected.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than suffix length {}.",
                actual.length(),
                expected.len()
            ));
        }
        if let Some(index) = mismatch {
            details.push(format!(
                "Suffix differs at zero-based collection index {}.",
                offset + index
            ));
        }
        let actual = this.render().stable_collection(actual);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not end with expected suffix: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_ends_with_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicates: &[P],
) where
    C: StableOrder<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let offset = actual.length().saturating_sub(predicates.len());
    let mismatch = actual
        .elements()
        .skip(offset)
        .zip(predicates)
        .position(|(actual, predicate)| !predicate(actual));

    if actual.length() < predicates.len() || mismatch.is_some() {
        let mut details = Vec::new();
        if actual.length() < predicates.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than predicate suffix length {}.",
                actual.length(),
                predicates.len()
            ));
        }
        if let Some(index) = mismatch {
            details.push(format!(
                "Element at zero-based index {} does not match its suffix predicate.",
                offset + index
            ));
        }
        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not end with elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_ends_with_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &[A],
) where
    C: StableOrder<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let offset = actual.length().saturating_sub(assertions.len());
    let maximum = this.render().max_items();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_elements = 0_usize;
    for (index, (element, assertions)) in actual.elements().skip(offset).zip(assertions).enumerate()
    {
        let failures = this.collect_element_failures(element, assertions);
        if !failures.is_empty() {
            number_of_unsatisfied_elements += 1;
            if unsatisfied.len() < maximum {
                unsatisfied.push((offset + index, failures));
            }
        }
    }

    if actual.length() < assertions.len() || number_of_unsatisfied_elements != 0 {
        let mut details = Vec::new();
        if actual.length() < assertions.len() {
            details.push(format!(
                "Collection has {} element(s), shorter than assertion suffix length {}.",
                actual.length(),
                assertions.len()
            ));
        }
        for (index, failures) in unsatisfied {
            details.push(format!(
                "Suffix element at index {index} does not satisfy its assertions:\n{}",
                join_failures(&failures, this.render().max_items())
            ));
        }
        let omitted = number_of_unsatisfied_elements.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "unsatisfied suffix element"));
        }
        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not end with elements satisfying the assertions.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[E],
) where
    C: StableOrder<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();
    let found = expected.is_empty()
        || elements.windows(expected.len()).any(|window| {
            window.iter().zip(expected).all(|(actual, expected)| {
                AssertrPartialEq::eq(*actual, expected, Some(&mut this.eq_context()))
            })
        });

    if !found {
        let actual = this.render().stable_collection(actual);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain contiguous expected elements: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicates: &[P],
) where
    C: StableOrder<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();
    let found = predicates.is_empty()
        || elements.windows(predicates.len()).any(|window| {
            window
                .iter()
                .zip(predicates)
                .all(|(actual, predicate)| predicate(actual))
        });

    if !found {
        let actual = this.render().stable_collection(actual);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain contiguous elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &[A],
) where
    C: StableOrder<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();
    let mut final_failures = Vec::new();
    let found = assertions.is_empty()
        || elements.windows(assertions.len()).any(|window| {
            let failures = window
                .iter()
                .zip(assertions)
                .flat_map(|(element, assertions)| {
                    this.collect_element_failures(*element, assertions)
                })
                .collect::<Vec<_>>();
            let matched = failures.is_empty();
            final_failures = failures;
            matched
        });

    if !found {
        let mut details = Vec::new();
        if !final_failures.is_empty() {
            details.push(format!(
                "The final contiguous candidate did not satisfy the assertions:\n{}",
                join_failures(&final_failures, this.render().max_items())
            ));
        }
        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain contiguous elements satisfying the assertions.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: StableOrder<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let mut ctx = this.eq_context();
    let result = compare(actual, expected, Some(&mut ctx));

    if !result.strictly_equal {
        let only_differing_in_order = result.only_differing_in_order();
        let mut details = Vec::new();
        if !only_differing_in_order && !ctx.differences.differences.is_empty() {
            details.push(format!("Differences: {:#?}", ctx.differences));
        }
        if !result.not_in_expected.is_empty() {
            details.push(format!(
                "Elements not expected: {:#?}",
                this.render()
                    .borrowed_values::<T, _>(result.not_in_expected.as_slice(), GroupStyle::List)
            ));
        }
        if !result.not_in_actual.is_empty() {
            details.push(format!(
                "Elements not found: {:#?}",
                this.render()
                    .borrowed_values::<E, _>(result.not_in_actual.as_slice(), GroupStyle::List)
            ));
        }
        if only_differing_in_order {
            details.push("The order of elements does not match!".to_owned());
        }

        let actual = this.render().stable_collection(actual);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly match

                Expected: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicates: &[P],
) where
    C: StableOrder<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();

    let same_length = actual.length() == predicates.len();
    let maximum = this.render().max_items();
    let mut not_matched = Vec::new();
    let mut number_not_matched = 0_usize;
    for (index, (element, predicate)) in actual.elements().zip(predicates).enumerate() {
        if !predicate(element) {
            number_not_matched += 1;
            if not_matched.len() < maximum {
                not_matched.push((index, element));
            }
        }
    }

    if !same_length || number_not_matched != 0 {
        let mut details = Vec::new();
        if !same_length {
            details.push(format!(
                "Number of elements ({}) does not match number of predicates ({})!",
                actual.length(),
                predicates.len()
            ));
        }
        for (index, element) in not_matched {
            details.push(format!(
                "Element at index {index} does not match its predicate: {:#?}",
                this.render().value(element)
            ));
        }
        let omitted = number_not_matched.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "element"));
        }

        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly match predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &[A],
) where
    C: StableOrder<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    let same_length = actual.length() == assertions.len();
    let maximum = this.render().max_items();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_elements = 0_usize;
    for (index, (element, element_assertions)) in actual.elements().zip(assertions).enumerate() {
        let failures = this.collect_element_failures(element, element_assertions);
        if !failures.is_empty() {
            number_of_unsatisfied_elements += 1;
            if unsatisfied.len() < maximum {
                unsatisfied.push((index, failures));
            }
        }
    }

    if !same_length || number_of_unsatisfied_elements != 0 {
        let mut details = Vec::new();
        if !same_length {
            details.push(format!(
                "Number of elements ({}) does not match number of assertions ({})!",
                actual.length(),
                assertions.len()
            ));
        }
        for (index, failures) in unsatisfied {
            details.push(format!(
                "Element at index {index} does not satisfy its assertions:\n{}",
                join_failures(&failures, this.render().max_items())
            ));
        }
        let omitted = number_of_unsatisfied_elements.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "unsatisfied element"));
        }

        let actual = this.render().stable_collection(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly satisfy the assertions.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[E],
) where
    C: Collection<Item = T>,
    T: AssertrPartialEq<E, R>,
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
            AssertrPartialEq::eq(
                elements[actual_index],
                &expected[expected_index],
                Some(&mut this.eq_context()),
            )
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
        let rendered_actual = this.render().collection(actual);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        let elements_not_found = this
            .render()
            .borrowed_values::<E, _>(elements_not_found.as_slice(), GroupStyle::List);
        let elements_not_expected = this
            .render()
            .borrowed_values::<T, _>(elements_not_expected.as_slice(), GroupStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?},

                Elements expected: {expected:#?}

                Elements not found: {elements_not_found:#?}

                Elements not expected: {elements_not_expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order_matching<C, T, P, M, R>(
    this: &AssertThat<'_, C, M, R>,
    predicates: &[P],
) where
    C: Collection<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();

    let result = match_bipartite(
        elements.len(),
        predicates.len(),
        |actual_index, predicate_index| predicates[predicate_index](elements[actual_index]),
    );

    if !result.is_exact() {
        let mut details = Vec::new();
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| elements[*index])
                .collect::<Vec<_>>();
            details.push(format!(
                "Elements not matched: {:#?}",
                this.render()
                    .borrowed_values::<T, _>(not_matched.as_slice(), GroupStyle::List)
            ));
        }
        if !result.unmatched_expected.is_empty() {
            details.push(format!(
                "Predicates not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual = this.render().collection(actual);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly match predicates in any order.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &[A],
) where
    C: Collection<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let elements = actual.elements().collect::<Vec<_>>();

    // Precomputed, as `match_bipartite` may probe the same pairing multiple times and running
    // assertions is considerably more expensive than calling a plain predicate.
    let satisfied = elements
        .iter()
        .copied()
        .map(|element| {
            assertions
                .iter()
                .map(|assertions| {
                    this.collect_element_failures(element, assertions)
                        .is_empty()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let result = match_bipartite(
        elements.len(),
        assertions.len(),
        |actual_index, assertion_index| satisfied[actual_index][assertion_index],
    );

    if !result.is_exact() {
        let mut details = Vec::new();
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| elements[*index])
                .collect::<Vec<_>>();
            details.push(format!(
                "Elements not matched: {:#?}",
                this.render()
                    .borrowed_values::<T, _>(not_matched.as_slice(), GroupStyle::List)
            ));
        }
        if !result.unmatched_expected.is_empty() {
            details.push(format!(
                "Assertions not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual = this.render().collection(actual);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly satisfy the assertions in any order.
            "}
        });
    }
}

#[cfg(test)]
mod tests {
    mod compare {
        use crate::assertions::collection::imp::{ExactCompareResult, compare};
        use crate::prelude::*;
        use crate::{AssertrPartialEq, DebugRenderer};

        fn compare_slices<'t, A, B>(aa: &'t [A], bb: &'t [B]) -> ExactCompareResult<'t, A, B>
        where
            A: AssertrPartialEq<B>,
        {
            compare::<_, A, B, DebugRenderer>(aa, bb, None)
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
