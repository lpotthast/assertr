//! Reusable value expectations for finite collections.

use alloc::vec::Vec;
use core::marker::PhantomData;

use super::{Collection, Placement, StableOrder};
use crate::{
    assertions::support::indexed_equality_mismatch,
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Expectation, passed},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{RenderingContext, RenderingOrder, ValueRenderer},
    util::matching::{assign_exactly, match_bipartite},
};

/// The rejection of [`ContainsAll`] and [`map::ContainsKeys`](crate::matchers::map::ContainsKeys):
/// the expected operands that the subject did not contain.
///
/// It borrows each missing element or key view from the expectation, in expected order, so
/// explanation lists exactly what evaluation found missing without searching the subject again.
/// The contents are private. Pass the rejection back to the `explain` method of the expectation
/// that produced it, which reports the missing operands under "Elements not found" or
/// "Keys not found".
///
/// Name this type when a concrete matcher delegates to a built-in one and declares its own
/// [`Expectation::Rejection`]:
///
/// ```
/// use assertr::{
///     expectation::{AssertionContext, Expectation},
///     failure::FailureBuilder,
///     matchers::collection::{ContainsAll, MissingElementsRejection},
///     prelude::*,
/// };
///
/// /// Requires the ports every service of ours listens on.
/// struct OpensRequiredPorts(ContainsAll<u16>);
///
/// impl Expectation<Vec<u16>> for OpensRequiredPorts {
///     type Success<'a> = ();
///     type Rejection<'a> = MissingElementsRejection<'a, u16>;
///
///     fn evaluate<'a>(
///         &'a self,
///         actual: &'a Vec<u16>,
///         context: &AssertionContext<'_>,
///     ) -> Result<(), Self::Rejection<'a>> {
///         self.0.evaluate(actual, context)
///     }
///
///     fn explain<'a>(
///         &'a self,
///         rejected: Option<(&'a Vec<u16>, Self::Rejection<'a>)>,
///         failure: FailureBuilder,
///         context: &AssertionContext<'_>,
///     ) -> FailureBuilder {
///         self.0.explain(rejected, failure, context)
///     }
/// }
///
/// let required = OpensRequiredPorts(ContainsAll::new(vec![80, 443]));
/// assert_that!(vec![22, 80, 443]).matches(&required);
/// ```
#[derive(Debug)]
pub struct MissingElementsRejection<'a, E: ?Sized> {
    pub(crate) missing: Vec<&'a E>,
}

/// The rejection of [`StartsWith`] and [`EndsWith`]: the subject's length and the first
/// mismatching position.
///
/// `A` is the collection item type and `E` the borrowed view of an expected element. The
/// rejection records the subject's length and, when a compared position differs, borrows the
/// actual element and the expected view at the first such index. A subject shorter than the
/// expected prefix or suffix is rejected even when no compared position differs. Explanation
/// names the index and both values without traversing the subject again.
/// The contents are private. Pass the rejection back to the `explain` method of the expectation
/// that produced it.
#[derive(Debug)]
pub struct PositionalRejection<'a, A: ?Sized, E: ?Sized> {
    length: usize,
    mismatch: Option<ElementMismatch<'a, A, E>>,
}

#[derive(Debug)]
struct ElementMismatch<'a, A: ?Sized, E: ?Sized> {
    index: usize,
    actual: &'a A,
    expected: &'a E,
}

/// The rejection of [`ContainsExactly`] and [`ContainsExactlyInAnyOrder`]: the occurrences left
/// unmatched by an exact comparison.
///
/// `A` is the collection item type and `E` the borrowed view of an expected element. The
/// rejection borrows the actual elements no expected element claimed and the expected elements
/// no actual element claimed, and records whether only their order differs. An ordered
/// comparison also retains its first mismatching position, reported as a child failure. Explanation
/// reports them under "Elements not found" and "Elements not expected" without comparing the
/// elements again. Probes, which are never explained, skip computing this assignment. The contents
/// are private. Pass the rejection back to the `explain` method of the expectation that produced
/// it.
#[derive(Debug)]
pub struct ExactElementsRejection<'a, A: ?Sized, E: ?Sized> {
    unexpected: Vec<&'a A>,
    missing: Vec<&'a E>,
    only_order_differs: bool,
    mismatch: Option<ElementMismatch<'a, A, E>>,
}

/// Labels for the values an exact element comparison left unmatched.
const ELEMENTS: [&str; 2] = ["Elements not found", "Elements not expected"];

/// Attaches the missing and unexpected occurrences of an exact comparison, when there are any.
pub(super) fn unmatched_facts<A: ?Sized, E: ?Sized, R>(
    failure: FailureBuilder,
    render: RenderingContext<'_, R>,
    [not_found, not_expected]: [&'static str; 2],
    missing: &[&E],
    unexpected: &[&A],
    unexpected_order: RenderingOrder,
) -> FailureBuilder
where
    R: ValueRenderer<A> + ValueRenderer<E>,
{
    let failure = if missing.is_empty() {
        failure
    } else {
        failure.fact(Fact::labelled(
            not_found,
            render.borrowed_values::<E, _>(missing, RenderingOrder::PreserveIteration),
        ))
    };
    if unexpected.is_empty() {
        failure
    } else {
        failure.fact(Fact::labelled(
            not_expected,
            render.borrowed_values::<A, _>(unexpected, unexpected_order),
        ))
    }
}

/// Checks collection membership with the actual element's `PartialEq` implementation and a borrowed
/// item operand.
///
/// [`CollectionAssertions::contains`](crate::assertions::CollectionAssertions::contains)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::Contains, prelude::*};
///
/// assert_that!(["a", "b"]).matches(Contains::new("b"));
/// ```
#[derive(Debug, Clone)]
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
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
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

/// Checks that no collection element equals a borrowed item operand, using the actual element's
/// `PartialEq` implementation.
///
/// [`CollectionAssertions::does_not_contain`](crate::assertions::CollectionAssertions::does_not_contain)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::DoesNotContain, prelude::*};
///
/// assert_that!(["a", "b"]).matches(DoesNotContain::new("c"));
/// ```
#[derive(Debug, Clone)]
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
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
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
///
/// [`CollectionAssertions::contains_all`](crate::assertions::CollectionAssertions::contains_all)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::ContainsAll, prelude::*};
///
/// assert_that!([1, 2, 3]).matches(ContainsAll::new([3, 1, 1]));
/// ```
pub struct ContainsAll<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(ContainsAll<E, B>, operand, new);

impl<C: Collection + ?Sized, E, B, R> Expectation<C, R> for ContainsAll<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains all of"),
            Some((actual, rejection)) => {
                let MissingElementsRejection { missing } = rejection;
                failure
                    .actual(render.collection(actual))
                    .relation("does not contain all of")
                    .fact(Fact::labelled(
                        "Elements not found",
                        render.borrowed_values::<E::View, _>(
                            &missing,
                            RenderingOrder::PreserveIteration,
                        ),
                    ))
            }
        };
        failure.expected(
            render.borrowed_values::<E::View, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

/// A rejected affix subject with its retained evidence.
type RejectedAffix<'a, C, V> = (&'a C, PositionalRejection<'a, <C as Collection>::Item, V>);

/// Compares the positions aligned with a [`Placement::Prefix`] or [`Placement::Suffix`],
/// stopping at the first mismatch.
///
/// A suffix aligns both lists at their ends, so a subject shorter than the expected suffix is
/// compared with the expected list's tail. Success allocates nothing, and expected operands past
/// the first mismatch are never borrowed.
fn evaluate_affix<'a, C, E>(
    placement: Placement,
    actual: &'a C,
    expected: &'a [E],
) -> Result<(), PositionalRejection<'a, C::Item, E::View>>
where
    C: StableOrder + ?Sized,
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
{
    let length = actual.length();
    let (offset, compared) = if placement == Placement::Suffix {
        (
            length.saturating_sub(expected.len()),
            &expected[expected.len().saturating_sub(length)..],
        )
    } else {
        (0, expected)
    };
    let mismatch = actual
        .elements()
        .skip(offset)
        .zip(compared.iter().map(borrow_for::<C::Item, _>))
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

/// Explains a rejection from [`evaluate_affix`], or describes the unmet expectation.
fn explain_affix<C, E, R>(
    placement: Placement,
    expected: &[E],
    rejected: Option<RejectedAffix<'_, C, E::View>>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder
where
    C: StableOrder + ?Sized,
    E: BorrowFor<C::Item>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    let (holds, fails) = if placement == Placement::Suffix {
        ("ends with", "does not end with")
    } else {
        ("starts with", "does not start with")
    };
    let render = context.render();
    let failure = match rejected {
        None => failure.relation(holds),
        Some((actual, PositionalRejection { length, mismatch })) => {
            let mut failure = failure
                .actual(render.stable_collection(actual))
                .relation(fails);
            if length < expected.len() {
                failure = failure.fact(Fact::labelled("Actual length", render.value(&length)));
            }
            if let Some(ElementMismatch {
                index,
                actual: element,
                expected,
            }) = mismatch
            {
                let mut child = context.isolated();
                child
                    .record(|context| indexed_equality_mismatch(context, index, element, expected));
                failure = failure.evidence(child.into_evidence());
            }
            failure
        }
    };
    failure
        .expected(render.borrowed_values::<E::View, _>(expected, RenderingOrder::PreserveIteration))
}

/// Requires an equal collection prefix, retaining the first mismatch and observed length.
///
/// [`StableOrderAssertions::starts_with`](crate::assertions::StableOrderAssertions::starts_with)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::StartsWith, prelude::*};
///
/// assert_that!([1, 2, 3]).matches(StartsWith::new([1, 2]));
/// ```
pub struct StartsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(StartsWith<E, B>, operand, new);

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for StartsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View> + ValueRenderer<usize>,
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
        evaluate_affix(Placement::Prefix, actual, self.expected.as_ref())
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_affix(
            Placement::Prefix,
            self.expected.as_ref(),
            rejected,
            failure,
            context,
        )
    }
}

/// Requires an equal collection suffix, retaining the first mismatch and observed length.
///
/// Both lists are aligned at their ends. A subject shorter than the suffix is compared with the
/// suffix's final elements, so its rejection reports the length and only genuine mismatches.
///
/// [`StableOrderAssertions::ends_with`](crate::assertions::StableOrderAssertions::ends_with)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::EndsWith, prelude::*};
///
/// assert_that!([1, 2, 3]).matches(EndsWith::new([2, 3]));
/// ```
pub struct EndsWith<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(EndsWith<E, B>, operand, new);

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for EndsWith<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View> + ValueRenderer<usize>,
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
        evaluate_affix(Placement::Suffix, actual, self.expected.as_ref())
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_affix(
            Placement::Suffix,
            self.expected.as_ref(),
            rejected,
            failure,
            context,
        )
    }
}

/// Requires an equal contiguous subsequence in a collection with stable order.
///
/// [`StableOrderAssertions::contains_contiguous`](crate::assertions::StableOrderAssertions::contains_contiguous)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::ContainsContiguous, prelude::*};
///
/// assert_that!([1, 2, 3, 4]).matches(ContainsContiguous::new([2, 3]));
/// ```
pub struct ContainsContiguous<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(ContainsContiguous<E, B>, operand, new);

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for ContainsContiguous<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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
        if expected.is_empty() {
            return Ok(());
        }
        if expected.len() > actual.length() {
            return Err(());
        }
        let elements = actual.elements().collect::<Vec<_>>();
        let found = elements.windows(expected.len()).any(|window| {
            window
                .iter()
                .zip(expected.iter().map(borrow_for::<C::Item, _>))
                .all(|(actual, expected)| (*actual).eq(expected))
        });
        passed(found)
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let expected = self.expected.as_ref();
        let actual = rejected.map(|(actual, ())| render.stable_collection(actual));
        let failure = failure.relations(
            actual,
            "contains the contiguous subsequence",
            "does not contain the contiguous subsequence",
        );
        failure.expected(
            render.borrowed_values::<E::View, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

/// Requires exact positional equality, retaining unmatched occurrences from maximum assignment.
///
/// [`StableOrderAssertions::contains_exactly`](crate::assertions::StableOrderAssertions::contains_exactly)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::ContainsExactly, prelude::*};
///
/// assert_that!(vec![1, 2]).matches(ContainsExactly::new([1, 2]));
/// ```
pub struct ContainsExactly<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(ContainsExactly<E, B>, operand, new);

impl<C: StableOrder + ?Sized, E, B, R> Expectation<C, R> for ContainsExactly<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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
                mismatch: None,
            });
        }

        let elements = actual.elements().collect::<Vec<_>>();
        let mismatch = elements
            .iter()
            .copied()
            .zip(expected.iter().map(borrow_for::<C::Item, _>))
            .enumerate()
            .find(|(_, (actual, expected))| !(*actual).eq(*expected))
            .map(|(index, (actual, expected))| ElementMismatch {
                index,
                actual,
                expected,
            });
        let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
            elements[a].eq(borrow_for::<C::Item, _>(&expected[e]))
        });
        let only_order_differs = same_length && matched.is_exact();
        let (unexpected, missing) = matched.unmatched(&elements, |index| {
            borrow_for::<C::Item, _>(&expected[index])
        });
        Err(ExactElementsRejection {
            unexpected,
            missing,
            only_order_differs,
            mismatch,
        })
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains exactly"),
            Some((actual, rejection)) => {
                let ExactElementsRejection {
                    unexpected,
                    missing,
                    only_order_differs,
                    mismatch,
                } = rejection;
                let mut failure = unmatched_facts(
                    failure
                        .actual(render.stable_collection(actual))
                        .relation("does not contain exactly"),
                    render,
                    ELEMENTS,
                    &missing,
                    &unexpected,
                    RenderingOrder::PreserveIteration,
                );
                if only_order_differs {
                    failure = failure.fact(Fact::note("Only the order of the elements differs."));
                }
                if let Some(ElementMismatch {
                    index,
                    actual: element,
                    expected,
                }) = mismatch
                {
                    let mut child = context.isolated();
                    child.record(|context| {
                        indexed_equality_mismatch(context, index, element, expected)
                    });
                    failure = failure.evidence(child.into_evidence());
                }
                failure
            }
        };
        failure.expected(
            render.borrowed_values::<E::View, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

/// Requires exact unordered multiplicity, retaining unmatched occurrences from maximum assignment.
///
/// [`CollectionAssertions::contains_exactly_in_any_order`](crate::assertions::CollectionAssertions::contains_exactly_in_any_order)
/// executes this same definition.
///
/// ```
/// use assertr::{matchers::collection::ContainsExactlyInAnyOrder, prelude::*};
///
/// assert_that!([1, 2, 2]).matches(ContainsExactlyInAnyOrder::new([2, 1, 2]));
/// ```
pub struct ContainsExactlyInAnyOrder<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
expected_operands_traits!(ContainsExactlyInAnyOrder<E, B>, operand, new);

impl<C: Collection + ?Sized, E, B, R> Expectation<C, R> for ContainsExactlyInAnyOrder<E, B>
where
    C::Item: PartialEq<E::View>,
    E: BorrowFor<C::Item>,
    B: AsRef<[E]>,
    R: ValueRenderer<C::Item> + ValueRenderer<E::View>,
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
        // A probe of a different length cannot match, so it rejects without traversing.
        if context.is_probe() && actual.length() != expected.len() {
            return Err(ExactElementsRejection {
                unexpected: Vec::new(),
                missing: Vec::new(),
                only_order_differs: false,
                mismatch: None,
            });
        }
        let elements = actual.elements().collect::<Vec<_>>();
        let unmatched = assign_exactly(
            context.is_probe(),
            elements.len(),
            expected.len(),
            |a, e| elements[a].eq(borrow_for::<C::Item, _>(&expected[e])),
        );
        let (unexpected, missing) = match unmatched {
            Ok(()) => return Ok(()),
            Err(None) => (Vec::new(), Vec::new()),
            Err(Some(matched)) => matched.unmatched(&elements, |index| {
                borrow_for::<C::Item, _>(&expected[index])
            }),
        };
        Err(ExactElementsRejection {
            unexpected,
            missing,
            only_order_differs: false,
            mismatch: None,
        })
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
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
                unmatched_facts(
                    failure
                        .actual(render.collection(actual))
                        .relation("does not contain exactly in any order"),
                    render,
                    ELEMENTS,
                    &missing,
                    &unexpected,
                    C::PRESENTATION.order(),
                )
            }
        };
        failure.expected(
            render.borrowed_values::<E::View, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

#[cfg(test)]
mod tests {
    mod evidence_budget {
        use core::{cell::Cell, fmt};

        use crate::{
            failure::{FailureKind, PathSegment},
            prelude::*,
        };

        struct Compared<'a> {
            value: i32,
            comparisons: &'a Cell<usize>,
        }

        impl PartialEq for Compared<'_> {
            fn eq(&self, other: &Self) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value == other.value
            }
        }

        struct Renderer<'a>(&'a Cell<usize>);

        impl ValueRenderer<Compared<'_>> for Renderer<'_> {
            fn fmt(&self, value: &Compared<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(f, "{}", value.value)
            }
        }

        impl ValueRenderer<usize> for Renderer<'_> {
            fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{value}")
            }
        }

        #[test]
        fn prefix_and_suffix_bound_mismatch_evidence_without_repeating_comparisons() {
            for suffix in [false, true] {
                for budget in [
                    RenderingBudget::default().with_max_items(0),
                    RenderingBudget::default().with_max_items(1),
                    RenderingBudget::unlimited(),
                ] {
                    let comparisons = Cell::new(0);
                    let renders = Cell::new(0);
                    let item = |value| Compared {
                        value,
                        comparisons: &comparisons,
                    };
                    let actual = [1, 2, 3].map(item);
                    let expected = [if suffix { 2 } else { 1 }, 9].map(item);
                    let failures = assert_that!(actual)
                        .with_renderer(Renderer(&renders))
                        .with_rendering_budget(budget)
                        .capture(|it| {
                            if suffix {
                                it.ends_with(&expected)
                            } else {
                                it.starts_with(&expected)
                            }
                        });
                    assert_that!(comparisons.get()).is_equal_to(2);
                    let retained = usize::from(budget.max_items() > 0);
                    assert_that!(renders.get()).is_equal_to(
                        actual.len().min(budget.max_items())
                            + expected.len().min(budget.max_items())
                            + retained * 2,
                    );
                    assert_that!(failures).has_length(1);
                    let failure = &failures[0];
                    assert_that!(failure.children).has_length(retained);
                    assert_that!(failure.omitted_children).is_equal_to(1 - retained);
                    for child in &failure.children {
                        assert_that!(child.path)
                            .contains_exactly([PathSegment::Index(if suffix { 2 } else { 1 })]);
                        assert_that!(child.kind).is_equal_to(FailureKind::Equality);
                    }
                    // Presenting retained evidence never performs another leaf conversion.
                    let before = renders.get();
                    let report = failure.to_string();
                    assert_that!(report.as_str()).contains(if suffix {
                        "does not end with"
                    } else {
                        "does not start with"
                    });
                    assert_that!(renders.get()).is_equal_to(before);
                }
            }
        }

        #[test]
        fn zero_budget_preserves_success_and_does_not_invent_length_mismatches() {
            let budget = RenderingBudget::default().with_max_items(0);
            let failures = assert_that!([1, 2, 3])
                .with_rendering_budget(budget)
                .capture(|it| {
                    it.starts_with([1, 2])
                        .ends_with([2, 3])
                        .starts_with([] as [i32; 0])
                });
            assert_that!(failures).is_empty();

            let failures = assert_that!([] as [i32; 0])
                .with_rendering_budget(budget)
                .capture(|it| it.starts_with([1]).ends_with([1]));
            assert_that!(failures).has_length(2);
            for failure in failures {
                assert_that!(failure.children).is_empty();
                assert_that!(failure.omitted_children).is_equal_to(0);
            }
        }
    }

    mod child_paths {
        use super::super::{EndsWith, StartsWith};
        use crate::{failure::PathSegment, matchers::all_of, prelude::*};

        #[test]
        fn prefix_and_suffix_mismatches_keep_relative_paths_inside_matchers() {
            let failures = assert_that!([[1, 2, 3]]).capture(|it| {
                it.matches(elements_are![all_of(matchers![
                    StartsWith::new([1, 9]),
                    EndsWith::new([2, 9])
                ])])
            });
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(2);
            for (failure, index) in failures[0].children.iter().zip([1, 2]) {
                assert_that!(failure.path).contains_exactly([PathSegment::Index(0)]);
                assert_that!(failure.children).has_length(1);
                let child = &failure.children[0];
                assert_that!(child.path).contains_exactly([PathSegment::Index(index)]);
                assert_that!(child.facts).is_empty();
            }
        }
    }

    mod ends_with {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn a_shorter_subject_whose_elements_end_the_suffix_reports_only_its_length() {
            let failures = assert_that!(vec![2, 3])
                .with_location(false)
                .capture(|it| it.ends_with([1, 2, 3]));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).is_empty();
            assert_that!(failures[0].omitted_children).is_equal_to(0);
        }

        #[test]
        fn a_shorter_subject_is_compared_with_the_end_of_the_suffix() {
            assert_that!(|| {
                assert_that!(vec![9, 3])
                    .with_location(false)
                    .ends_with([1, 2, 3]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `vec![9, 3]`

                Actual: [
                    9,
                    3,
                ]

                does not end with

                Expected: [
                    1,
                    2,
                    3,
                ]

                Details:
                  - Actual length: 2
                Nested failures:
                  - At [0]:
                    Expected: 2

                      Actual: 9
                -------- assertr --------
            "});
        }
    }

    mod contains_contiguous {
        use super::super::ContainsContiguous;
        use crate::{
            assertions::{HasLength, collection::Collection},
            expectation::AssertionContext,
            prelude::*,
            renderer::CollectionPresentation,
        };

        /// Reports a length but panics when its elements are traversed.
        struct Untraversable(usize);

        impl HasLength for Untraversable {
            fn length(&self) -> usize {
                self.0
            }
        }

        impl Collection for Untraversable {
            type Item = i32;
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

            fn elements(&self) -> impl Iterator<Item = &i32> {
                panic!("elements traversed");
                #[allow(unreachable_code)]
                [].iter()
            }
        }

        impl crate::assertions::collection::StableOrder for Untraversable {}

        #[test]
        fn decides_empty_and_oversized_expectations_without_buffering_elements() {
            let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
            assert_that!(
                context.probe(&Untraversable(2), &ContainsContiguous::new([] as [i32; 0]))
            )
            .is_true();
            assert_that!(context.probe(&Untraversable(2), &ContainsContiguous::new([1, 2, 3])))
                .is_false();
        }
    }

    mod contains_exactly {
        use core::cell::Cell;

        use super::super::ContainsExactly;
        use crate::{expectation::AssertionContext, prelude::*};

        #[derive(Debug)]
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
                    &DebugRenderer,
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

    mod contains_exactly_in_any_order {
        use core::cell::Cell;

        use super::super::ContainsExactlyInAnyOrder;
        use crate::{expectation::AssertionContext, prelude::*};

        #[derive(Debug)]
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
        fn probes_reject_unequal_lengths_without_comparing_and_skip_diagnostic_assignment() {
            let comparisons = Cell::new(0);
            let actual = [1, 2, 3].map(|value| Counted {
                value,
                comparisons: &comparisons,
            });
            let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
            assert_that!(context.probe(
                &actual,
                &ContainsExactlyInAnyOrder::new([&actual[0], &actual[1]])
            ))
            .is_false();
            assert_that!(comparisons.get()).is_equal_to(0);
            assert_that!(context.probe(
                &actual,
                &ContainsExactlyInAnyOrder::new([&actual[2], &actual[0], &actual[1]])
            ))
            .is_true();
            assert_that!(context.probe(
                &actual,
                &ContainsExactlyInAnyOrder::new([&actual[2], &actual[2], &actual[1]])
            ))
            .is_false();
        }
    }

    mod borrowed_operands {
        use core::cell::Cell;

        use super::super::*;
        use crate::{
            prelude::*,
            test_support::{BorrowSpy, StrOperand, StringRenderer},
        };

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

    mod string_views {
        use super::super::*;
        use crate::prelude::*;

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
            let failures = assert_that!(values)
                .capture(|it| it.contains_exactly_in_any_order(["b", "b", "a"]));
            assert_that!(failures).has_length(1);
        }
    }

    mod repeatable_expected_data {
        use core::{borrow::Borrow, cell::Cell};

        use super::super::*;
        use crate::{prelude::*, test_support::BorrowSpy};

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
                let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
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

        fn check_explanation<'a, D>(
            actual: &[Actual<'a>; 2],
            definition: &D,
            comparisons: &Cell<usize>,
        ) where
            D: Expectation<[Actual<'a>; 2]>,
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
                    FailureBuilder::new::<[Actual; 2]>(D::KIND),
                    &context,
                )
                .build();
            assert_that!(comparisons.get()).is_equal_to(observed);
            let _ = context.describe::<[Actual; 2], _>(definition);
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
            let _ = context.describe::<[Actual; 2], _>(&prefix);
            let _ = context.describe::<[Actual; 2], _>(&suffix);
            let _ = context.describe::<[Actual; 2], _>(&exact);
            let _ = context.describe::<[Actual; 2], _>(&membership);
            let _ = context.describe::<[Actual; 2], _>(&contiguous);
            let _ = context.describe::<[Actual; 2], _>(&unordered);
            assert_that!(views.get()).is_greater_than(0);
            assert_that!(first.get()).is_greater_than(0);
            assert_that!(omitted.get()).is_equal_to(0);
        }
    }
}
