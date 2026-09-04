//! Algorithms and diagnostics shared by every collection assertion.
//!
//! The public [`CollectionAssertions`](super::CollectionAssertions) methods are thin wrappers
//! around these functions, so every collection type produces identical failure messages.

use alloc::vec::Vec;

use super::{Collection, StableOrder};
use crate::failure::{Fact, FailureBuilder, FailureKind};
use crate::renderer::{GroupStyle, RenderingOrder};
use crate::report::TextReporter;
use crate::{
    AssertThat, AssertionFailure, AssertrPartialEq, EqContext, Mode, ValueRenderer, mode::Capture,
    util::matching::match_bipartite,
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

/// Whether diagnostics over `C`'s elements are sorted by their rendered text because the
/// collection has no deterministic iteration order.
fn sorts_for_rendering<C: Collection + ?Sized>() -> bool {
    C::PRESENTATION.order() == RenderingOrder::SortByRenderedText
}

/// Flattens the failures of unsatisfied elements into the children of one failure.
///
/// Elements are kept in the collection's rendering order, so a collection without a
/// deterministic iteration order lists them sorted by their rendered text. At most `maximum`
/// elements are kept. Returns the children and the number of omitted elements.
fn element_children<C: Collection + ?Sized>(
    mut unsatisfied: Vec<Vec<AssertionFailure>>,
    maximum: usize,
) -> (Vec<AssertionFailure>, usize) {
    if sorts_for_rendering::<C>() {
        unsatisfied.sort_by_cached_key(|failures| {
            failures
                .iter()
                .map(|failure| TextReporter.report(failure))
                .collect::<alloc::string::String>()
        });
    }
    let omitted = unsatisfied.len().saturating_sub(maximum);
    unsatisfied.truncate(maximum);
    (unsatisfied.into_iter().flatten().collect(), omitted)
}

/// Flattens the failures of unsatisfied positional elements into children, each located at its
/// element index. At most `maximum` elements are kept. Returns the children and the number of
/// omitted elements.
fn indexed_children(
    mut unsatisfied: Vec<(usize, Vec<AssertionFailure>)>,
    maximum: usize,
) -> (Vec<AssertionFailure>, usize) {
    let omitted = unsatisfied.len().saturating_sub(maximum);
    unsatisfied.truncate(maximum);
    let children = unsatisfied
        .into_iter()
        .flat_map(|(index, failures)| {
            failures
                .into_iter()
                .map(move |failure| failure.located_at(Fact::index(index)))
        })
        .collect();
    (children, omitted)
}

/// A child failure for an element that did not match its predicate.
fn unmatched_element<T, M, R>(
    this: &AssertThat<'_, impl Collection<Item = T>, M, R>,
    index: usize,
    element: &T,
) -> AssertionFailure
where
    M: Mode,
    R: ValueRenderer<T>,
{
    FailureBuilder::detached::<T>(FailureKind::Predicate)
        .actual(this.render().value(element))
        .relation("does not match its predicate")
        .build()
        .located_at(Fact::index(index))
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
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("does not contain all of")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            )
            .fact(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(not_found.as_slice(), GroupStyle::List),
            )
            .raise();
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
        this.failure(FailureKind::Membership)
            .actual(this.render().collection(actual))
            .relation("contains")
            .unexpected(this.render().value(not_expected))
            .raise();
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
        this.failure(FailureKind::Predicate)
            .actual(this.render().collection(actual))
            .relation("does not contain an element matching the predicate")
            .raise();
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

    let mut unsatisfied = Vec::new();
    for element in actual.elements() {
        let failures = this.collect_element_failures(element, assertions);
        if failures.is_empty() {
            return;
        }
        unsatisfied.push(failures);
    }

    let (children, omitted) = element_children::<C>(unsatisfied, this.render().max_items());
    this.failure(FailureKind::Predicate)
        .actual(this.render().collection(actual))
        .relation("does not contain an element satisfying the assertions")
        .omitted(omitted, "unsatisfied element")
        .children(children)
        .raise();
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
        this.failure(FailureKind::Predicate)
            .actual(this.render().collection(actual))
            .relation("contains elements matching the predicate")
            .unexpected(
                this.render()
                    .borrowed_values::<T, _>(matching.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            )
            .raise();
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
        this.failure(FailureKind::Predicate)
            .actual(this.render().collection(actual))
            .relation("contains elements satisfying the assertions")
            .unexpected(
                this.render()
                    .borrowed_values::<T, _>(satisfying.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            )
            .raise();
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
        .enumerate()
        .find(|(_, (actual, expected))| {
            !AssertrPartialEq::eq(*actual, *expected, Some(&mut this.eq_context()))
        });

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
            failure = failure.fact("Actual length", actual.length());
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
        .enumerate()
        .find(|(_, (actual, predicate))| !predicate(actual));

    if actual.length() < predicates.len() || mismatch.is_some() {
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not start with elements matching the predicates");
        if actual.length() < predicates.len() {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", predicates.len());
        }
        if let Some((index, (element, _))) = mismatch {
            failure = failure.child(unmatched_element(this, index, element));
        }
        failure.raise();
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
    let mut unsatisfied = Vec::new();
    for (index, (element, assertions)) in actual.elements().zip(assertions).enumerate() {
        let failures = this.collect_element_failures(element, assertions);
        if !failures.is_empty() {
            unsatisfied.push((index, failures));
        }
    }

    if actual.length() < assertions.len() || !unsatisfied.is_empty() {
        let (children, omitted) = indexed_children(unsatisfied, this.render().max_items());
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not start with elements satisfying the assertions");
        if actual.length() < assertions.len() {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", assertions.len());
        }
        failure
            .omitted(omitted, "unsatisfied element")
            .children(children)
            .raise();
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
        .enumerate()
        .find(|(_, (actual, expected))| {
            !AssertrPartialEq::eq(*actual, *expected, Some(&mut this.eq_context()))
        });

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
            failure = failure.fact("Actual length", actual.length());
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
        .enumerate()
        .find(|(_, (actual, predicate))| !predicate(actual));

    if actual.length() < predicates.len() || mismatch.is_some() {
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not end with elements matching the predicates");
        if actual.length() < predicates.len() {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", predicates.len());
        }
        if let Some((index, (element, _))) = mismatch {
            failure = failure.child(unmatched_element(this, offset + index, element));
        }
        failure.raise();
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
    let mut unsatisfied = Vec::new();
    for (index, (element, assertions)) in actual.elements().skip(offset).zip(assertions).enumerate()
    {
        let failures = this.collect_element_failures(element, assertions);
        if !failures.is_empty() {
            unsatisfied.push((offset + index, failures));
        }
    }

    if actual.length() < assertions.len() || !unsatisfied.is_empty() {
        let (children, omitted) = indexed_children(unsatisfied, this.render().max_items());
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not end with elements satisfying the assertions");
        if actual.length() < assertions.len() {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", assertions.len());
        }
        failure
            .omitted(omitted, "unsatisfied element")
            .children(children)
            .raise();
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
        this.failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not contain a contiguous subsequence matching the predicates")
            .raise();
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
    // The per-element failures of the last candidate window, each located at its element index.
    let mut final_failures = Vec::new();
    let found = assertions.is_empty()
        || elements
            .windows(assertions.len())
            .enumerate()
            .any(|(start, window)| {
                final_failures = window
                    .iter()
                    .zip(assertions)
                    .enumerate()
                    .filter_map(|(offset, (element, assertions))| {
                        let failures = this.collect_element_failures(*element, assertions);
                        (!failures.is_empty()).then_some((start + offset, failures))
                    })
                    .collect::<Vec<_>>();
                final_failures.is_empty()
            });

    if !found {
        let (children, omitted) = indexed_children(final_failures, this.render().max_items());
        this.failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not contain a contiguous subsequence satisfying the assertions")
            .omitted(omitted, "unsatisfied element")
            .children(children)
            .raise();
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
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(this.render().stable_collection(actual))
            .relation("does not contain exactly")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        if !only_differing_in_order && !ctx.differences.differences.is_empty() {
            failure = failure.fact("Differences", format_args!("{:#?}", ctx.differences));
        }
        if !result.not_in_expected.is_empty() {
            failure = failure.fact(
                "Elements not expected",
                this.render()
                    .borrowed_values::<T, _>(result.not_in_expected.as_slice(), GroupStyle::List),
            );
        }
        if !result.not_in_actual.is_empty() {
            failure = failure.fact(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(result.not_in_actual.as_slice(), GroupStyle::List),
            );
        }
        if only_differing_in_order {
            failure = failure.note("Only the order of the elements differs.");
        }
        failure.raise();
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
    let mut unmatched = Vec::new();
    let mut number_unmatched = 0_usize;
    for (index, (element, predicate)) in actual.elements().zip(predicates).enumerate() {
        if !predicate(element) {
            number_unmatched += 1;
            if unmatched.len() < maximum {
                unmatched.push(unmatched_element(this, index, element));
            }
        }
    }

    if !same_length || number_unmatched != 0 {
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not exactly match the predicates");
        if !same_length {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", predicates.len());
        }
        failure
            .omitted(
                number_unmatched.saturating_sub(maximum),
                "unmatched element",
            )
            .children(unmatched)
            .raise();
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
    let mut unsatisfied = Vec::new();
    for (index, (element, element_assertions)) in actual.elements().zip(assertions).enumerate() {
        let failures = this.collect_element_failures(element, element_assertions);
        if !failures.is_empty() {
            unsatisfied.push((index, failures));
        }
    }

    if !same_length || !unsatisfied.is_empty() {
        let (children, omitted) = indexed_children(unsatisfied, this.render().max_items());
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().stable_collection(actual))
            .relation("does not exactly satisfy the assertions");
        if !same_length {
            failure = failure
                .fact("Actual length", actual.length())
                .fact("Expected length", assertions.len());
        }
        failure
            .omitted(omitted, "unsatisfied element")
            .children(children)
            .raise();
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
        let mut failure = this
            .failure(FailureKind::Equality)
            .actual(this.render().collection(actual))
            .relation("does not contain exactly in any order")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        if !elements_not_found.is_empty() {
            failure = failure.fact(
                "Elements not found",
                this.render()
                    .borrowed_values::<E, _>(elements_not_found.as_slice(), GroupStyle::List),
            );
        }
        if !elements_not_expected.is_empty() {
            failure = failure.fact(
                "Elements not expected",
                this.render()
                    .borrowed_values::<T, _>(elements_not_expected.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            );
        }
        failure.raise();
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
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().collection(actual))
            .relation("does not exactly match the predicates in any order");
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| elements[*index])
                .collect::<Vec<_>>();
            failure = failure.fact(
                "Elements not matched",
                this.render()
                    .borrowed_values::<T, _>(not_matched.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            );
        }
        if !result.unmatched_expected.is_empty() {
            failure = failure.fact("Predicates not matched", result.unmatched_expected.len());
        }
        failure.raise();
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
        let mut failure = this
            .failure(FailureKind::Predicate)
            .actual(this.render().collection(actual))
            .relation("does not exactly satisfy the assertions in any order");
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| elements[*index])
                .collect::<Vec<_>>();
            failure = failure.fact(
                "Elements not matched",
                this.render()
                    .borrowed_values::<T, _>(not_matched.as_slice(), GroupStyle::List)
                    .sort_for_rendering(sorts_for_rendering::<C>()),
            );
        }
        if !result.unmatched_expected.is_empty() {
            failure = failure.fact("Assertions not matched", result.unmatched_expected.len());
        }
        failure.raise();
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
