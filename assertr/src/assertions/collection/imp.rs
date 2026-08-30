//! Algorithms and diagnostics shared by every collection assertion.
//!
//! The public [`CollectionAssertions`](super::CollectionAssertions) methods are thin wrappers
//! around these functions, so every collection type produces identical failure messages.

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Display, Write};
use indoc::writedoc;

use super::{Collection, CollectionStyle};
use crate::{
    AssertThat, AssertrPartialEq, EqContext, Mode, ValueRenderer, mode::Capture,
    util::failure::join_failures, util::matching::match_bipartite,
};

/// Renders a [`Collection::TYPE_NAME`] as the prefix of a failure's "Actual" value, e.g. the
/// `HashSet ` in `Actual: HashSet {1, 2}`. Renders nothing for a collection that does not name
/// itself. Built-in sequences use no prefix because `[1, 2]` already reads as a list.
pub(crate) struct TypePrefix(pub(crate) Option<&'static str>);

impl Display for TypePrefix {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(name) => f.write_fmt(format_args!("{name} ")),
            None => Ok(()),
        }
    }
}

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
                let mut probe_ctx = EqContext::with_renderer(ctx.renderer);
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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();
    if !actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, expected, Some(&mut ctx))
    }) {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        let expected = this.render_value(expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
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
        let actual_values = actual.elements().collect::<Vec<_>>();
        let rendered_actual = this.render_values(&actual_values, C::STYLE);
        let rendered_expected =
            this.render_borrowed_values::<E, _>(expected, CollectionStyle::List);
        let not_found = this.render_values(not_found.as_slice(), CollectionStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{rendered_actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();
    if actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, not_expected, Some(&mut ctx))
    }) {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        let not_expected = this.render_value(not_expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();
    if !actual.elements().any(predicate) {
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();

    let mut unsatisfied = Vec::new();
    for (index, element) in actual.elements().enumerate() {
        let failures = this.collect_element_failures(element, assertions);
        if failures.is_empty() {
            return;
        }
        unsatisfied.push((index, failures));
    }

    let mut details = Vec::new();
    for (index, failures) in &unsatisfied {
        details.push(format!(
            "Element at index {index} does not satisfy the assertions:\n{}",
            join_failures(failures)
        ));
    }

    let actual_values = actual.elements().collect::<Vec<_>>();
    let actual = this.render_values(&actual_values, C::STYLE);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w, r"
            Actual: {prefix}{actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();
    let matching = actual
        .elements()
        .filter(|it| predicate(it))
        .collect::<Vec<_>>();
    if !matching.is_empty() {
        let matching = this.render_values(matching.as_slice(), CollectionStyle::List);
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?}

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
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();
    let satisfying = actual
        .elements()
        .filter(|element| {
            this.collect_element_failures(*element, assertions)
                .is_empty()
        })
        .collect::<Vec<_>>();
    if !satisfying.is_empty() {
        let satisfying = this.render_values(satisfying.as_slice(), CollectionStyle::List);
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?}

                unexpectedly contains elements satisfying the given assertions: {satisfying:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: Collection<Item = T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    this.track_assertion();
    let prefix = TypePrefix(C::TYPE_NAME);
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
                this.render_values(result.not_in_expected.as_slice(), CollectionStyle::List)
            ));
        }
        if !result.not_in_actual.is_empty() {
            details.push(format!(
                "Elements not found: {:#?}",
                this.render_values(result.not_in_actual.as_slice(), CollectionStyle::List)
            ));
        }
        if only_differing_in_order {
            details.push("The order of elements does not match!".to_owned());
        }

        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        let expected = this.render_borrowed_values::<E, _>(expected, CollectionStyle::List);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?},

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
    C: Collection<Item = T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: ValueRenderer<T>,
{
    this.track_assertion();
    let prefix = TypePrefix(C::TYPE_NAME);
    let actual = this.actual();

    let same_length = actual.length() == predicates.len();
    let mut not_matched = Vec::new();
    for (index, (element, predicate)) in actual.elements().zip(predicates).enumerate() {
        if !predicate(element) {
            not_matched.push((index, element));
        }
    }

    if !same_length || !not_matched.is_empty() {
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
                this.render_value(element)
            ));
        }

        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?},

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
    C: Collection<Item = T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: ValueRenderer<T> + Clone,
{
    this.track_assertion();
    let prefix = TypePrefix(C::TYPE_NAME);
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
                join_failures(&failures)
            ));
        }

        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?},

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
    let prefix = TypePrefix(C::TYPE_NAME);
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
        let actual_values = actual.elements().collect::<Vec<_>>();
        let rendered_actual = this.render_values(&actual_values, C::STYLE);
        let expected = this.render_borrowed_values::<E, _>(expected, CollectionStyle::List);
        let elements_not_found =
            this.render_values(elements_not_found.as_slice(), CollectionStyle::List);
        let elements_not_expected =
            this.render_values(elements_not_expected.as_slice(), CollectionStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{rendered_actual:#?},

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
    let prefix = TypePrefix(C::TYPE_NAME);
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
                this.render_values(not_matched.as_slice(), CollectionStyle::List)
            ));
        }
        if !result.unmatched_expected.is_empty() {
            details.push(format!(
                "Predicates not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?},

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
    let prefix = TypePrefix(C::TYPE_NAME);
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
                this.render_values(not_matched.as_slice(), CollectionStyle::List)
            ));
        }
        if !result.unmatched_expected.is_empty() {
            details.push(format!(
                "Assertions not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual_values = actual.elements().collect::<Vec<_>>();
        let actual = this.render_values(&actual_values, C::STYLE);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {prefix}{actual:#?},

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

            assert!(!result.only_differing_in_order());
            assert!(result.strictly_equal);
            assert!(result.same_length);
            assert!(result.not_in_actual.is_empty());
            assert!(result.not_in_expected.is_empty());
        }

        #[test]
        fn returns_equal_on_equal_input() {
            let result = compare_slices(&[1, 2, 3], &[1, 2, 3]);

            assert!(!result.only_differing_in_order());
            assert!(result.strictly_equal);
            assert!(result.same_length);
            assert!(result.not_in_actual.is_empty());
            assert!(result.not_in_expected.is_empty());
        }

        #[test]
        fn returns_not_equal_on_equal_but_rearranged_input() {
            let result = compare_slices(&[1, 2, 3], &[3, 2, 1]);

            assert!(result.only_differing_in_order());
            assert!(!result.strictly_equal);
            assert!(result.same_length);
            assert!(result.not_in_actual.is_empty());
            assert!(result.not_in_expected.is_empty());
        }

        #[test]
        fn returns_not_equal_and_lists_differences_on_differing_input() {
            let result = compare_slices(&[1, 5, 7], &[5, 3, 4, 42]);

            assert!(!result.only_differing_in_order());
            assert!(!result.strictly_equal);
            assert!(!result.same_length);
            assert_eq!(result.not_in_actual, [&3, &4, &42]);
            assert_eq!(result.not_in_expected, [&1, &7]);
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
