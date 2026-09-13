//! Reusable value expectations for finite collections.

use super::{Collection, StableOrder};
use crate::borrow_for::{BorrowFor, borrow_for};
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::GroupStyle,
    util::matching::match_bipartite,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Retained missing values from a membership rejection.
pub struct MissingElementsRejection<'a, E: ?Sized> {
    missing: Vec<&'a E>,
}

/// Retained length and first mismatch from a prefix or suffix rejection.
pub struct PositionalRejection<'a, A: ?Sized, E: ?Sized> {
    length: usize,
    mismatch: Option<ElementMismatch<'a, A, E>>,
}

struct ElementMismatch<'a, A: ?Sized, E: ?Sized> {
    index: usize,
    actual: &'a A,
    expected: &'a E,
}

/// Retained unmatched occurrences from an exact collection rejection.
/// Diagnostic assignment is omitted during probes.
pub struct ExactElementsRejection<'a, A: ?Sized, E: ?Sized> {
    unexpected: Vec<&'a A>,
    missing: Vec<&'a E>,
    only_order_differs: bool,
}

/// Checks collection membership with the actual element’s `PartialEq` implementation and a borrowed
/// item operand.
pub struct Contains<E>(E);

impl<E> Contains<E> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<C: Collection + ?Sized, E, R> Expectation<C, R> for Contains<E>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<C::Item, _>(&self.0);
        if actual.elements().any(|it| it.eq(expected)) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<C: Collection + ?Sized, E, R> ExpectationDiagnostics<C, R> for Contains<E>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("contains"),
                borrow_for::<C::Item, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.collection(actual))
                    .relation("does not contain"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// Checks collection membership with the actual element’s `PartialEq` implementation and a borrowed
/// item operand.
pub struct DoesNotContain<E>(E);

impl<E> DoesNotContain<E> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<C: Collection + ?Sized, E, R> Expectation<C, R> for DoesNotContain<E>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<C::Item, _>(&self.0);
        if actual.elements().any(|it| it.eq(expected)) {
            Err(expected)
        } else {
            Ok(())
        }
    }
}

impl<C: Collection + ?Sized, E, R> ExpectationDiagnostics<C, R> for DoesNotContain<E>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("does not contain"),
                borrow_for::<C::Item, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.collection(actual))
                    .relation("contains"),
                expected,
            ),
        };
        failure.unexpected(render.value(expected))
    }
}

/// Requires a match for each expected value. Duplicate expectations may share a matching element.
pub struct ContainsAll<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsAll<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: Collection + ?Sized, E, B, R> Expectation<C, R> for ContainsAll<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = MissingElementsRejection<'a, E::View>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let missing = expected
            .iter()
            .map(borrow_for::<C::Item, _>)
            .filter(|expected| !actual.elements().any(|it| it.eq(*expected)))
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MissingElementsRejection { missing })
        }
    }
}

impl<C: Collection + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsAll<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains all of"),
            Some((actual, rejection)) => {
                let MissingElementsRejection { missing, .. } = rejection;
                failure
                    .actual(render.collection(actual))
                    .relation("does not contain all of")
                    .fact(Fact::labelled(
                        "Elements not found",
                        render.borrowed_values::<E::View, _>(&missing, GroupStyle::List),
                    ))
            }
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal collection prefix, retaining the first mismatch and observed length.
pub struct StartsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> StartsWith<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for StartsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = PositionalRejection<'a, C::Item, E::View>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let length = actual.length();
        let offset = 0;
        let mismatch = actual
            .elements()
            .skip(offset)
            .zip(expected.iter().map(borrow_for::<C::Item, _>))
            .enumerate()
            .find(|(_, (actual, expected))| !(*actual).eq(*expected))
            .map(|(index, (actual, expected))| ElementMismatch {
                index: offset + index,
                actual,
                expected,
            });
        if length < expected.len() || mismatch.is_some() {
            Err(PositionalRejection { length, mismatch })
        } else {
            Ok(())
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for StartsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("starts with"),
            Some((actual, rejection)) => {
                let PositionalRejection { length, mismatch } = rejection;
                let mut failure = failure
                    .actual(render.stable_collection(actual))
                    .relation("does not start with");
                if length < expected.len() {
                    failure = failure.fact(Fact::labelled("Actual length", render.value(&length)));
                }
                if let Some(ElementMismatch {
                    index,
                    actual: element,
                    expected,
                }) = mismatch
                {
                    failure = failure.child(
                        FailureBuilder::detached::<C::Item>(FailureKind::Equality)
                            .actual(render.value(element))
                            .expected(render.value(expected))
                            .build()
                            .located_at(Fact::index(index)),
                    );
                }
                failure
            }
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal collection suffix, retaining the first mismatch and observed length.
pub struct EndsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> EndsWith<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for EndsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = PositionalRejection<'a, C::Item, E::View>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let length = actual.length();
        let offset = length.saturating_sub(expected.len());
        let mismatch = actual
            .elements()
            .skip(offset)
            .zip(expected.iter().map(borrow_for::<C::Item, _>))
            .enumerate()
            .find(|(_, (actual, expected))| !(*actual).eq(*expected))
            .map(|(index, (actual, expected))| ElementMismatch {
                index: offset + index,
                actual,
                expected,
            });
        if length < expected.len() || mismatch.is_some() {
            Err(PositionalRejection { length, mismatch })
        } else {
            Ok(())
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for EndsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("ends with"),
            Some((actual, rejection)) => {
                let PositionalRejection { length, mismatch } = rejection;
                let mut failure = failure
                    .actual(render.stable_collection(actual))
                    .relation("does not end with");
                if length < expected.len() {
                    failure = failure.fact(Fact::labelled("Actual length", render.value(&length)));
                }
                if let Some(ElementMismatch {
                    index,
                    actual: element,
                    expected,
                }) = mismatch
                {
                    failure = failure.child(
                        FailureBuilder::detached::<C::Item>(FailureKind::Equality)
                            .actual(render.value(element))
                            .expected(render.value(expected))
                            .build()
                            .located_at(Fact::index(index)),
                    );
                }
                failure
            }
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal contiguous subsequence in a collection with stable order.
pub struct ContainsContiguous<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsContiguous<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for ContainsContiguous<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let elements = actual.elements().collect::<Vec<_>>();
        let found = expected.is_empty()
            || elements.windows(expected.len()).any(|window| {
                window
                    .iter()
                    .zip(expected.iter().map(borrow_for::<C::Item, _>))
                    .all(|(actual, expected)| (*actual).eq(expected))
            });
        if found { Ok(()) } else { Err(()) }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsContiguous<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains the contiguous subsequence"),
            Some((actual, ())) => failure
                .actual(render.stable_collection(actual))
                .relation("does not contain the contiguous subsequence"),
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Requires exact positional equality, retaining unmatched occurrences from maximum assignment.
pub struct ContainsExactly<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsExactly<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for ContainsExactly<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ExactElementsRejection<'a, C::Item, E::View>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let same_length = actual.length() == expected.len();
        if same_length
            && actual
                .elements()
                .zip(expected.iter().map(borrow_for::<C::Item, _>))
                .all(|(actual, expected)| actual.eq(expected))
        {
            return Ok(());
        }

        if context.is_probe() {
            return Err(ExactElementsRejection {
                unexpected: Vec::new(),
                missing: Vec::new(),
                only_order_differs: false,
            });
        }

        let elements = actual.elements().collect::<Vec<_>>();
        let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
            elements[a].eq(borrow_for::<C::Item, _>(&expected[e]))
        });

        let unexpected = matched
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect();
        let missing = matched
            .unmatched_expected
            .iter()
            .map(|index| borrow_for::<C::Item, _>(&expected[*index]))
            .collect();
        let only_order_differs = same_length && matched.is_exact();
        Err(ExactElementsRejection {
            unexpected,
            missing,
            only_order_differs,
        })
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsExactly<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains exactly"),
            Some((actual, rejection)) => {
                let ExactElementsRejection {
                    unexpected,
                    missing,
                    only_order_differs,
                    ..
                } = rejection;
                let mut failure = failure
                    .actual(render.stable_collection(actual))
                    .relation("does not contain exactly");
                if !unexpected.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Elements not expected",
                        render.borrowed_values::<C::Item, _>(&unexpected, GroupStyle::List),
                    ));
                }
                if !missing.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Elements not found",
                        render.borrowed_values::<E::View, _>(&missing, GroupStyle::List),
                    ));
                }
                if only_order_differs {
                    failure = failure.fact(Fact::note("Only the order of the elements differs."));
                }
                failure
            }
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Requires exact unordered multiplicity, retaining unmatched occurrences from maximum assignment.
pub struct ContainsExactlyInAnyOrder<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsExactlyInAnyOrder<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<C: Collection + ?Sized, E, B, R> Expectation<C, R> for ContainsExactlyInAnyOrder<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ExactElementsRejection<'a, C::Item, E::View>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();

        let elements = actual.elements().collect::<Vec<_>>();
        let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
            elements[a].eq(borrow_for::<C::Item, _>(&expected[e]))
        });
        if matched.is_exact() {
            return Ok(());
        }

        let unexpected = matched
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect();
        let missing = matched
            .unmatched_expected
            .iter()
            .map(|index| borrow_for::<C::Item, _>(&expected[*index]))
            .collect();
        Err(ExactElementsRejection {
            unexpected,
            missing,
            only_order_differs: false,
        })
    }
}

impl<C: Collection + ?Sized, E, B, R> ExpectationDiagnostics<C, R>
    for ContainsExactlyInAnyOrder<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains exactly in any order"),
            Some((actual, rejection)) => {
                let ExactElementsRejection {
                    unexpected,
                    missing,
                    ..
                } = rejection;
                let mut failure = failure
                    .actual(render.collection(actual))
                    .relation("does not contain exactly in any order");
                if !missing.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Elements not found",
                        render.borrowed_values::<E::View, _>(&missing, GroupStyle::List),
                    ));
                }
                if !unexpected.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Elements not expected",
                        render
                            .borrowed_values::<C::Item, _>(&unexpected, GroupStyle::List)
                            .with_order(C::PRESENTATION.order()),
                    ));
                }
                failure
            }
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

#[cfg(test)]
mod tests {
    mod contains_exactly {
        use super::super::ContainsExactly;
        use crate::{AssertionContext, prelude::*, test_support::NoRenderer};
        use core::cell::Cell;

        struct Counted<'a> {
            value: i32,
            comparisons: &'a Cell<usize>,
        }
        impl PartialEq for Counted<'_> {
            fn eq(&self, other: &Self) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value == other.value
            }
        }

        #[test]
        fn probes_skip_diagnostic_assignment_after_a_positional_rejection() {
            let comparisons = Cell::new(0);
            let actual = [1, 2, 3].map(|value| Counted {
                value,
                comparisons: &comparisons,
            });
            for limit in [0, 1, usize::MAX] {
                let context = AssertionContext::new(
                    &NoRenderer,
                    RenderingBudget::default().with_max_items(limit),
                );
                comparisons.set(0);
                assert_that!(context.probe(
                    &actual,
                    &ContainsExactly::new([&actual[2], &actual[1], &actual[0]])
                ))
                .is_false();
                assert_that!(comparisons.get()).is_equal_to(1);
                comparisons.set(0);
                assert_that!(
                    context.probe(&actual, &ContainsExactly::new([&actual[0], &actual[1]]))
                )
                .is_false();
                assert_that!(comparisons.get()).is_equal_to(0);
                comparisons.set(0);
                assert_that!(context.probe(&actual, &ContainsExactly::new(&actual))).is_true();
                assert_that!(comparisons.get()).is_equal_to(3);
            }
        }
    }

    mod borrowed_operands {
        use super::super::*;
        use crate::{prelude::*, test_support::BorrowSpy};
        use core::cell::Cell;

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed temporaries are the contract under test.
        fn borrowed_non_copy_elements_and_lists_work_in_methods_and_definitions() {
            let a = String::from("a");
            let b = String::from("b");
            let values = [String::from("a"), String::from("b")];
            assert_that!(values)
                .contains(&a)
                .does_not_contain(&String::from("c"))
                .contains_all([&a, &b])
                .starts_with([&a])
                .ends_with([&b])
                .contains_contiguous([&a, &b])
                .contains_exactly([&a, &b])
                .contains_exactly_in_any_order([&b, &a]);
            assert_that!(values)
                .matches(Contains::new(&a))
                .matches(DoesNotContain::new(&String::from("c")))
                .matches(ContainsAll::new([&a, &b]))
                .matches(StartsWith::new([&a]))
                .matches(EndsWith::new([&b]))
                .matches(ContainsContiguous::new([&a, &b]))
                .matches(ContainsExactly::new([&a, &b]))
                .matches(ContainsExactlyInAnyOrder::new([&b, &a]));
            assert_that!([&a]).contains(&a).contains_exactly([&a]);
        }

        #[test]
        fn operands_are_accessed_after_tracking_including_explanation() {
            for method in 0..8 {
                let calls = Cell::new(0);
                let failures = assert_that!(()).capture(|root| {
                    let expected = BorrowSpy {
                        value: if method == 1 { 2 } else { 9 },
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    let it = root.derive_owned(|()| [1, 2, 3]);
                    match method {
                        0 => it.contains(expected),
                        1 => it.does_not_contain(expected),
                        2 => it.contains_all([expected]),
                        3 => it.starts_with([expected]),
                        4 => it.ends_with([expected]),
                        5 => it.contains_contiguous([expected]),
                        6 => it.contains_exactly([expected]),
                        _ => it.contains_exactly_in_any_order([expected]),
                    };
                    root
                });
                if method < 2 {
                    assert_that!(calls.get()).is_equal_to(1);
                } else {
                    assert_that!(calls.get()).is_greater_than(0);
                }
                assert_that!(failures).has_length(1);
            }
        }
    }
}

#[cfg(test)]
mod string_views {
    use super::*;
    use crate::{
        prelude::*,
        test_support::{StrOperand, StringRenderer},
    };
    use core::cell::Cell;

    #[test]
    fn literal_lists_work_in_all_value_methods_and_definitions() {
        let values = [String::from("a"), String::from("b"), String::from("a")];
        let list = ["a", "b", "a"];
        assert_that!(values)
            .contains("b")
            .does_not_contain("c")
            .contains_all(["a", "b"])
            .starts_with(["a", "b"])
            .ends_with(["b", "a"])
            .contains_contiguous(["b", "a"])
            .contains_exactly(&list)
            .contains_exactly_in_any_order(["b", "a", "a"]);
        assert_that!(values)
            .matches(Contains::new("a"))
            .matches(DoesNotContain::new("c"))
            .matches(ContainsAll::new(["b", "a"]))
            .matches(StartsWith::new(["a"]))
            .matches(EndsWith::new(["a"]))
            .matches(ContainsContiguous::new(["b", "a"]))
            .matches(ContainsExactly::new(&list))
            .matches(ContainsExactlyInAnyOrder::new(["a", "a", "b"]));
        let failures =
            assert_that!(values).capture(|it| it.contains_exactly_in_any_order(["b", "b", "a"]));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn unsized_operands_are_accessed_after_tracking_through_explanation() {
        for method in 0..8 {
            let calls = Cell::new(0);
            let failures = assert_that!([String::from("a")])
                .with_renderer(StringRenderer)
                .capture(|root| {
                    let expected = StrOperand {
                        value: if method == 1 { "a" } else { "b" },
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    let it = root.derive(|it| it);
                    match method {
                        0 => it.contains(expected),
                        1 => it.does_not_contain(expected),
                        2 => it.contains_all([expected]),
                        3 => it.starts_with([expected]),
                        4 => it.ends_with([expected]),
                        5 => it.contains_contiguous([expected]),
                        6 => it.contains_exactly([expected]),
                        _ => it.contains_exactly_in_any_order([expected]),
                    };
                    root
                });
            if method < 2 {
                assert_that!(calls.get()).is_equal_to(1);
            } else {
                assert_that!(calls.get()).is_greater_than(0);
            }
            assert_that!(failures).has_length(1);
        }
    }
}

#[cfg(test)]
mod repeatable_expected_data {
    use super::*;
    use crate::{
        prelude::*,
        test_support::{BorrowSpy, NoRenderer},
    };
    use core::{borrow::Borrow, cell::Cell};

    #[test]
    fn borrowed_wrapper_lists_reuse_the_stored_operand_selection_across_subjects() {
        let list = [1, 2]
            .map(|value| BorrowSpy {
                value,
                observe: || {},
            })
            .into_iter()
            .collect::<Vec<_>>();
        let membership = ContainsAll::new(&list);
        let prefix = StartsWith::new(list.as_slice());
        let suffix = EndsWith::new(&list);
        let exact = ContainsExactly::new(&list);
        let contiguous = ContainsContiguous::new(&list);
        let unordered = ContainsExactlyInAnyOrder::new(&list);
        assert_that!([1, 2])
            .contains_all(&list)
            .matches(&membership)
            .matches(&prefix)
            .matches(&suffix)
            .matches(&exact)
            .matches(&contiguous)
            .matches(&unordered);
        assert_that!(alloc::vec![1, 2])
            .contains_all(list.as_slice())
            .matches(&membership)
            .matches(&prefix)
            .matches(&suffix)
            .matches(&exact)
            .matches(&contiguous)
            .matches(&unordered);
        assert_that!([1, 2]).contains_all((1..=2).collect::<Vec<_>>());
    }

    #[test]
    fn short_circuit_comparisons_do_not_borrow_unreached_operands() {
        for method in 0..4 {
            let inputs = [0, 1].map(|index| BorrowSpy {
                value: 9,
                observe: move || {
                    assert_that!(index).is_equal_to(0);
                },
            });
            let context = AssertionContext::new(&NoRenderer, RenderingBudget::default());
            let actual = [1, 2];
            let result = match method {
                0 => context.probe(&actual, &StartsWith::new(&inputs)),
                1 => context.probe(&actual, &EndsWith::new(&inputs)),
                2 => context.probe(&actual, &ContainsExactly::new(&inputs)),
                _ => context.probe(&actual, &ContainsContiguous::new(&inputs)),
            };
            assert_that!(result).is_false();
        }
    }

    #[derive(Debug)]
    struct Actual<'a>(&'a Cell<usize>);
    impl PartialEq<i32> for Actual<'_> {
        fn eq(&self, _: &i32) -> bool {
            self.0.set(self.0.get() + 1);
            false
        }
    }
    struct Operand<'a>(&'a Cell<usize>);
    impl Borrow<i32> for Operand<'_> {
        fn borrow(&self) -> &i32 {
            self.0.set(self.0.get() + 1);
            &9
        }
    }
    impl BorrowFor<Actual<'_>> for Operand<'_> {
        type View = i32;
    }

    fn check_explanation<'a, D>(actual: &[Actual<'a>; 2], definition: &D, comparisons: &Cell<usize>)
    where
        D: ExpectationDiagnostics<[Actual<'a>; 2]>,
    {
        let context =
            AssertionContext::new(&DebugRenderer, RenderingBudget::default().with_max_items(1));
        let Err(rejection) = definition.evaluate(actual, &context) else {
            panic!("expected a rejection");
        };
        let observed = comparisons.get();
        assert_that!(observed).is_greater_than(0);
        let _ = definition
            .explain(
                Some((actual, rejection)),
                FailureBuilder::detached::<[Actual; 2]>(D::KIND),
                &context,
            )
            .build();
        assert_that!(comparisons.get()).is_equal_to(observed);
        context.describe::<[Actual; 2], _>(definition);
        assert_that!(comparisons.get()).is_equal_to(observed);
    }

    #[test]
    fn explanation_and_missing_subject_descriptions_never_repeat_comparisons() {
        let comparisons = Cell::new(0);
        let borrows = Cell::new(0);
        let actual = [Actual(&comparisons), Actual(&comparisons)];
        let operands = [Operand(&borrows), Operand(&borrows)];
        check_explanation(&actual, &ContainsAll::new(&operands), &comparisons);
        check_explanation(&actual, &StartsWith::new(&operands), &comparisons);
        check_explanation(&actual, &EndsWith::new(&operands), &comparisons);
        check_explanation(&actual, &ContainsContiguous::new(&operands), &comparisons);
        check_explanation(&actual, &ContainsExactly::new(&operands), &comparisons);
        check_explanation(
            &actual,
            &ContainsExactlyInAnyOrder::new(&operands),
            &comparisons,
        );
        assert_that!(borrows.get()).is_greater_than(0);
    }

    #[test]
    fn constructors_are_lazy_and_missing_subject_borrowing_respects_the_budget() {
        struct Inputs<'a> {
            operands: [Operand<'a>; 2],
            views: &'a Cell<usize>,
        }
        impl<'a> AsRef<[Operand<'a>]> for Inputs<'a> {
            fn as_ref(&self) -> &[Operand<'a>] {
                self.views.set(self.views.get() + 1);
                &self.operands
            }
        }
        let views = Cell::new(0);
        let first = Cell::new(0);
        let omitted = Cell::new(0);
        let inputs = Inputs {
            operands: [Operand(&first), Operand(&omitted)],
            views: &views,
        };
        let prefix = StartsWith::new(&inputs);
        let suffix = EndsWith::new(&inputs);
        let exact = ContainsExactly::new(&inputs);
        let membership = ContainsAll::new(&inputs);
        let contiguous = ContainsContiguous::new(&inputs);
        let unordered = ContainsExactlyInAnyOrder::new(&inputs);
        assert_that!((views.get(), first.get(), omitted.get())).is_equal_to((0, 0, 0));
        let context =
            AssertionContext::new(&DebugRenderer, RenderingBudget::default().with_max_items(1));
        context.describe::<[Actual; 2], _>(&prefix);
        context.describe::<[Actual; 2], _>(&suffix);
        context.describe::<[Actual; 2], _>(&exact);
        context.describe::<[Actual; 2], _>(&membership);
        context.describe::<[Actual; 2], _>(&contiguous);
        context.describe::<[Actual; 2], _>(&unordered);
        assert_that!(views.get()).is_greater_than(0);
        assert_that!(first.get()).is_greater_than(0);
        assert_that!(omitted.get()).is_equal_to(0);
    }
}
