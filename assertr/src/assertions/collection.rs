//! Shared implementation core for element-collection assertions.
//!
//! The public per-type assertion traits (slices, `Vec`, arrays, `VecDeque`) stay separate for
//! method discovery, but every assertion body lives here exactly once, so all collection types
//! produce identical failure messages.

use alloc::borrow::ToOwned;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, EqContext, Mode,
    failure::BANNER,
    mode::Capture,
    tracking::AssertionTracking,
    util::slice::{match_bipartite, match_multiset},
};

/// Joins one element's captured assertion failures for embedding into a detail message,
/// stripping the banners every rendered failure carries.
fn join_failures(failures: &[String]) -> String {
    failures
        .iter()
        .map(|failure| {
            let failure = failure.strip_prefix(BANNER).unwrap_or(failure);
            let failure = failure.strip_suffix(BANNER).unwrap_or(failure);
            failure.trim_matches('\n')
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A collection whose elements can be inspected by reference for assertion purposes.
///
/// `Rendered` is the type shown as the "Actual" value in failure messages.
pub(crate) trait CollectionView<T> {
    type Rendered: ?Sized;

    fn rendered(&self) -> &Self::Rendered;
    fn len(&self) -> usize;
    fn element(&self, index: usize) -> &T;
    fn elements<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;
}

impl<T> CollectionView<T> for [T] {
    type Rendered = [T];

    fn rendered(&self) -> &[T] {
        self
    }

    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn element(&self, index: usize) -> &T {
        &self[index]
    }

    fn elements<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.iter()
    }
}

impl<T> CollectionView<T> for VecDeque<T> {
    type Rendered = VecDeque<T>;

    fn rendered(&self) -> &VecDeque<T> {
        self
    }

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn element(&self, index: usize) -> &T {
        &self[index]
    }

    fn elements<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.iter()
    }
}

impl<T, V> CollectionView<T> for &V
where
    V: CollectionView<T> + ?Sized,
{
    type Rendered = V::Rendered;

    fn rendered(&self) -> &V::Rendered {
        V::rendered(self)
    }

    fn len(&self) -> usize {
        V::len(self)
    }

    fn element(&self, index: usize) -> &T {
        V::element(self, index)
    }

    fn elements<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        V::elements(self)
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
    C: CollectionView<T> + ?Sized,
    T: AssertrPartialEq<E, R>,
{
    let same_length = actual.len() == expected.len();
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

    let mut not_in_expected = Vec::new();
    let mut not_in_actual = Vec::new();

    for actual_element in actual.elements() {
        if !expected
            .iter()
            .any(|expected| AssertrPartialEq::eq(actual_element, expected, ctx.as_deref_mut()))
        {
            not_in_expected.push(actual_element);
        }
    }

    for expected_element in expected {
        if !actual
            .elements()
            .any(|actual| AssertrPartialEq::eq(actual, expected_element, ctx.as_deref_mut()))
        {
            not_in_actual.push(expected_element);
        }
    }

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
    C: CollectionView<T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if !actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, expected, Some(&mut ctx))
    }) {
        let actual = this.render_value(actual.rendered());
        let expected = this.render_value(expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain expected: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain<C, T, E, M, R>(
    this: &AssertThat<'_, C, M, R>,
    not_expected: &E,
) where
    C: CollectionView<T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();
    if actual.elements().any(|it| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(it, not_expected, Some(&mut ctx))
    }) {
        let actual = this.render_value(actual.rendered());
        let not_expected = this.render_value(not_expected);
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
    C: CollectionView<T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: AssertionRenderer<C::Rendered>,
{
    this.track_assertion();
    let actual = this.actual();
    if !actual.elements().any(predicate) {
        let actual = this.render_value(actual.rendered());
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
    C: CollectionView<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: AssertionRenderer<C::Rendered> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    let mut unsatisfied = Vec::new();
    for (index, element) in actual.elements().enumerate() {
        let failures = this.collect_element_failures(element, assertions);
        if failures.is_empty() {
            return;
        }
        unsatisfied.push((index, failures));
    }

    for (index, failures) in &unsatisfied {
        this.add_detail_message(format!(
            "Element at index {index} does not satisfy the assertions:\n{}",
            join_failures(failures)
        ));
    }

    let actual = this.render_value(actual.rendered());
    this.fail(|w: &mut String| {
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
    C: CollectionView<T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();
    let matching = actual
        .elements()
        .filter(|it| predicate(it))
        .collect::<Vec<_>>();
    if !matching.is_empty() {
        let matching = this.render_values(matching.as_slice());
        let actual = this.render_value(actual.rendered());
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                contains elements matching the given predicate: {matching:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain_satisfying<C, T, A, M, R>(
    this: &AssertThat<'_, C, M, R>,
    assertions: &A,
) where
    C: CollectionView<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<T> + Clone,
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
        let satisfying = this.render_values(satisfying.as_slice());
        let actual = this.render_value(actual.rendered());
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                contains elements satisfying the given assertions: {satisfying:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<C, T, E, M, R>(this: &AssertThat<'_, C, M, R>, expected: &[E])
where
    C: CollectionView<T>,
    T: AssertrPartialEq<E, R>,
    M: Mode,
    R: AssertionRenderer<C::Rendered>
        + AssertionRenderer<[E]>
        + AssertionRenderer<T>
        + AssertionRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let mut ctx = this.eq_context();
    let result = compare(actual, expected, Some(&mut ctx));

    if !result.strictly_equal {
        if !result.not_in_expected.is_empty() {
            this.add_detail_message(format!(
                "Elements not expected: {:#?}",
                this.render_values(result.not_in_expected.as_slice())
            ));
        }
        if !result.not_in_actual.is_empty() {
            this.add_detail_message(format!(
                "Elements not found: {:#?}",
                this.render_values(result.not_in_actual.as_slice())
            ));
        }
        if result.only_differing_in_order() {
            this.add_detail_message("The order of elements does not match!".to_owned());
        }

        let actual = this.render_value(actual.rendered());
        let expected = this.render_value(expected);

        this.fail(|w: &mut String| {
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
    C: CollectionView<T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();

    let same_length = actual.len() == predicates.len();
    let mut not_matched = Vec::new();
    for (index, (element, predicate)) in actual.elements().zip(predicates).enumerate() {
        if !predicate(element) {
            not_matched.push((index, element));
        }
    }

    if !same_length || !not_matched.is_empty() {
        if !same_length {
            this.add_detail_message(format!(
                "Number of elements ({}) does not match number of predicates ({})!",
                actual.len(),
                predicates.len()
            ));
        }
        for (index, element) in not_matched {
            this.add_detail_message(format!(
                "Element at index {index} does not match its predicate: {:#?}",
                this.render_value(element)
            ));
        }

        let actual = this.render_value(actual.rendered());
        this.fail(|w: &mut String| {
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
    C: CollectionView<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: AssertionRenderer<C::Rendered> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    let same_length = actual.len() == assertions.len();
    let mut unsatisfied = Vec::new();
    for (index, (element, element_assertions)) in actual.elements().zip(assertions).enumerate() {
        let failures = this.collect_element_failures(element, element_assertions);
        if !failures.is_empty() {
            unsatisfied.push((index, failures));
        }
    }

    if !same_length || !unsatisfied.is_empty() {
        if !same_length {
            this.add_detail_message(format!(
                "Number of elements ({}) does not match number of assertions ({})!",
                actual.len(),
                assertions.len()
            ));
        }
        for (index, failures) in unsatisfied {
            this.add_detail_message(format!(
                "Element at index {index} does not satisfy its assertions:\n{}",
                join_failures(&failures)
            ));
        }

        let actual = this.render_value(actual.rendered());
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?},

                did not exactly satisfy the assertions.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order<C, T, M, R>(
    this: &AssertThat<'_, C, M, R>,
    expected: &[T],
) where
    C: CollectionView<T>,
    T: PartialEq,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<[T]> + AssertionRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();

    let result = match_multiset(
        actual.len(),
        expected.len(),
        |actual_index, expected_index| actual.element(actual_index) == &expected[expected_index],
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
            .map(|index| actual.element(*index))
            .collect::<Vec<_>>();
        let rendered_actual = this.render_value(actual.rendered());
        let expected = this.render_value(expected);
        let elements_not_found = this.render_values(elements_not_found.as_slice());
        let elements_not_expected = this.render_values(elements_not_expected.as_slice());
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
    C: CollectionView<T>,
    P: Fn(&T) -> bool,
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<T>,
{
    this.track_assertion();
    let actual = this.actual();

    let result = match_bipartite(
        actual.len(),
        predicates.len(),
        |actual_index, predicate_index| predicates[predicate_index](actual.element(actual_index)),
    );

    if !result.is_exact() {
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| actual.element(*index))
                .collect::<Vec<_>>();
            this.add_detail_message(format!(
                "Elements not matched: {:#?}",
                this.render_values(not_matched.as_slice())
            ));
        }
        if !result.unmatched_expected.is_empty() {
            this.add_detail_message(format!(
                "Predicates not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual = this.render_value(actual.rendered());

        this.fail(|w: &mut String| {
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
    C: CollectionView<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    M: Mode,
    R: AssertionRenderer<C::Rendered> + AssertionRenderer<T> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    // Precomputed, as `match_bipartite` may probe the same pairing multiple times and running
    // assertions is considerably more expensive than calling a plain predicate.
    let satisfied = actual
        .elements()
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
        actual.len(),
        assertions.len(),
        |actual_index, assertion_index| satisfied[actual_index][assertion_index],
    );

    if !result.is_exact() {
        if !result.unmatched_actual.is_empty() {
            let not_matched = result
                .unmatched_actual
                .iter()
                .map(|index| actual.element(*index))
                .collect::<Vec<_>>();
            this.add_detail_message(format!(
                "Elements not matched: {:#?}",
                this.render_values(not_matched.as_slice())
            ));
        }
        if !result.unmatched_expected.is_empty() {
            this.add_detail_message(format!(
                "Assertions not matched: {}",
                result.unmatched_expected.len()
            ));
        }
        let actual = this.render_value(actual.rendered());

        this.fail(|w: &mut String| {
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
        use crate::assertions::collection::{ExactCompareResult, compare};
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
    }
}
