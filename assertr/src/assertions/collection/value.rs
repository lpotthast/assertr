//! Reusable value expectations for finite collections.

use super::{Collection, StableOrder};
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::GroupStyle,
    util::matching::match_bipartite,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Retained expected elements and missing values from a membership rejection.
pub struct MissingElementsRejection<'a, E> {
    expected: &'a [E],
    missing: Vec<&'a E>,
}

/// Retained length and first mismatch from a prefix or suffix rejection.
pub struct PositionalRejection<'a, A: ?Sized, E> {
    expected: &'a [E],
    length: usize,
    mismatch: Option<ElementMismatch<'a, A, E>>,
}

struct ElementMismatch<'a, A: ?Sized, E: ?Sized> {
    index: usize,
    actual: &'a A,
    expected: &'a E,
}

/// Retained operands and unmatched occurrences from an exact collection rejection.
/// Diagnostic assignment is omitted during probes.
pub struct ExactElementsRejection<'a, A: ?Sized, E> {
    expected: &'a [E],
    unexpected: Vec<&'a A>,
    missing: Vec<&'a E>,
    only_order_differs: bool,
}

/// Checks collection membership with the actual element’s heterogeneous `PartialEq` implementation.
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
    C::Item: PartialEq<E>,
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
        if actual.elements().any(|it| it.eq(&self.0)) {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<C: Collection + ?Sized, E, R> ExpectationDiagnostics<C, R> for Contains<E>
where
    C::Item: PartialEq<E>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("contains"),
            Some((actual, ())) => failure
                .actual(render.collection(actual))
                .relation("does not contain"),
        };
        failure.expected(render.value(&self.0))
    }
}

/// Checks collection membership with the actual element’s heterogeneous `PartialEq` implementation.
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
    C::Item: PartialEq<E>,
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
        if actual.elements().any(|it| it.eq(&self.0)) {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<C: Collection + ?Sized, E, R> ExpectationDiagnostics<C, R> for DoesNotContain<E>
where
    C::Item: PartialEq<E>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("does not contain"),
            Some((actual, ())) => failure
                .actual(render.collection(actual))
                .relation("contains"),
        };
        failure.unexpected(render.value(&self.0))
    }
}

/// Requires a match for each expected value. Duplicate expectations may share a matching element.
pub struct ContainsAll<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsAll<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = MissingElementsRejection<'a, E>
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
            .filter(|expected| !actual.elements().any(|it| it.eq(expected)))
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MissingElementsRejection { expected, missing })
        }
    }
}

impl<C: Collection + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsAll<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("contains all of"),
            Some((actual, rejection)) => {
                let MissingElementsRejection { missing, .. } = rejection;
                failure
                    .actual(render.collection(actual))
                    .relation("does not contain all of")
                    .fact(Fact::labelled(
                        "Elements not found",
                        render.borrowed_values::<E, _>(&missing, GroupStyle::List),
                    ))
            }
        };
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal collection prefix, retaining the first mismatch and observed length.
pub struct StartsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> StartsWith<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = PositionalRejection<'a, C::Item, E>
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
            .zip(expected)
            .enumerate()
            .find(|(_, (actual, expected))| !(*actual).eq(*expected))
            .map(|(index, (actual, expected))| ElementMismatch {
                index: offset + index,
                actual,
                expected,
            });
        if length < expected.len() || mismatch.is_some() {
            Err(PositionalRejection {
                expected,
                length,
                mismatch,
            })
        } else {
            Ok(())
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for StartsWith<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E> + ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("starts with"),
            Some((actual, rejection)) => {
                let PositionalRejection {
                    expected,
                    length,
                    mismatch,
                } = rejection;
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
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal collection suffix, retaining the first mismatch and observed length.
pub struct EndsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> EndsWith<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = PositionalRejection<'a, C::Item, E>
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
            .zip(expected)
            .enumerate()
            .find(|(_, (actual, expected))| !(*actual).eq(*expected))
            .map(|(index, (actual, expected))| ElementMismatch {
                index: offset + index,
                actual,
                expected,
            });
        if length < expected.len() || mismatch.is_some() {
            Err(PositionalRejection {
                expected,
                length,
                mismatch,
            })
        } else {
            Ok(())
        }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for EndsWith<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E> + ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("ends with"),
            Some((actual, rejection)) => {
                let PositionalRejection {
                    expected,
                    length,
                    mismatch,
                } = rejection;
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
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Requires an equal contiguous subsequence in a collection with stable order.
pub struct ContainsContiguous<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsContiguous<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = &'a [E]
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
                    .zip(expected)
                    .all(|(actual, expected)| (*actual).eq(expected))
            });
        if found { Ok(()) } else { Err(expected) }
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsContiguous<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected
            .as_ref()
            .map_or_else(|| self.expected.as_ref(), |(_, expected)| *expected);
        let failure = match rejected {
            None => failure.relation("contains the contiguous subsequence"),
            Some((actual, _)) => failure
                .actual(render.stable_collection(actual))
                .relation("does not contain the contiguous subsequence"),
        };
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Requires exact positional equality, retaining unmatched occurrences from maximum assignment.
pub struct ContainsExactly<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsExactly<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ExactElementsRejection<'a, C::Item, E>
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
                .zip(expected)
                .all(|(actual, expected)| actual.eq(expected))
        {
            return Ok(());
        }

        if context.is_probe() {
            return Err(ExactElementsRejection {
                expected,
                unexpected: Vec::new(),
                missing: Vec::new(),
                only_order_differs: false,
            });
        }

        let elements = actual.elements().collect::<Vec<_>>();
        let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
            elements[a].eq(&expected[e])
        });

        let unexpected = matched
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect();
        let missing = matched
            .unmatched_expected
            .iter()
            .map(|index| &expected[*index])
            .collect();
        let only_order_differs = same_length && matched.is_exact();
        Err(ExactElementsRejection {
            expected,
            unexpected,
            missing,
            only_order_differs,
        })
    }
}

impl<C: StableOrder + ?Sized, E, B, R> ExpectationDiagnostics<C, R> for ContainsExactly<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
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
                        render.borrowed_values::<E, _>(&missing, GroupStyle::List),
                    ));
                }
                if only_order_differs {
                    failure = failure.fact(Fact::note("Only the order of the elements differs."));
                }
                failure
            }
        };
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Requires exact unordered multiplicity, retaining unmatched occurrences from maximum assignment.
pub struct ContainsExactlyInAnyOrder<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsExactlyInAnyOrder<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
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
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ExactElementsRejection<'a, C::Item, E>
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
            elements[a].eq(&expected[e])
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
            .map(|index| &expected[*index])
            .collect();
        Err(ExactElementsRejection {
            expected,
            unexpected,
            missing,
            only_order_differs: false,
        })
    }
}

impl<C: Collection + ?Sized, E, B, R> ExpectationDiagnostics<C, R>
    for ContainsExactlyInAnyOrder<E, B>
where
    C::Item: PartialEq<E>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
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
                        render.borrowed_values::<E, _>(&missing, GroupStyle::List),
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
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
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
        impl PartialEq<i32> for Counted<'_> {
            fn eq(&self, other: &i32) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value == *other
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
                assert_that!(context.probe(&actual, &ContainsExactly::new([3, 2, 1]))).is_false();
                assert_that!(comparisons.get()).is_equal_to(1);
                comparisons.set(0);
                assert_that!(context.probe(&actual, &ContainsExactly::new([1, 2]))).is_false();
                assert_that!(comparisons.get()).is_equal_to(0);
                comparisons.set(0);
                assert_that!(context.probe(&actual, &ContainsExactly::new([1, 2, 3]))).is_true();
                assert_that!(comparisons.get()).is_equal_to(3);
            }
        }
    }
}
