//! Reusable borrowed-target identity expectations for finite collections.

use alloc::vec::Vec;
use core::{borrow::Borrow, marker::PhantomData, ptr};

use super::{Collection, StableOrder};
use crate::{
    assertions::support::length_facts,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind, PathSegment},
    renderer::{Rendered, RenderingOrder},
    util::matching::assign_exactly,
};

const METADATA_NOTE: &str = "Some pointers have equal data addresses but different metadata.";

/// Borrowed targets that evaluation already visited, in iteration order.
///
/// Probes retain nothing, because they are never explained. Explanation completes the targets
/// without borrowing a visited element again.
struct ObservedTargets<'a, U: ?Sized> {
    targets: Vec<&'a U>,
    retain: bool,
}

impl<'a, U: ?Sized> ObservedTargets<'a, U> {
    fn new<R>(context: &AssertionContext<'_, R>) -> Self {
        Self {
            targets: Vec::new(),
            retain: !context.is_probe(),
        }
    }

    fn observe(&mut self, target: &'a U) -> &'a U {
        if self.retain {
            self.targets.push(target);
        }
        target
    }

    /// Renders the subject from the visited targets, borrowing only the elements evaluation did
    /// not reach.
    fn render<C: Collection + ?Sized, R>(
        mut self,
        actual: &'a C,
        order: RenderingOrder,
        context: &AssertionContext<'_, R>,
    ) -> Rendered
    where
        C::Item: Borrow<U>,
    {
        let rendering = context.render().identities();
        let remaining = if order == RenderingOrder::SortByRenderedText {
            usize::MAX
        } else {
            rendering.max_items().saturating_sub(self.targets.len())
        };
        let visited = self.targets.len();
        self.targets.extend(
            actual
                .elements()
                .skip(visited)
                .take(remaining)
                .map(Borrow::borrow),
        );
        rendering.observed_collection(actual, &self.targets, order)
    }
}

/// Retained borrowed-target evidence from an identity membership rejection.
pub struct IdentityMembershipRejection<'a, U: ?Sized> {
    observed: ObservedTargets<'a, U>,
    same_address: bool,
}

struct IdentityMismatch<'a, U: ?Sized> {
    index: usize,
    actual: &'a U,
    expected: &'a U,
    same_address: bool,
}

/// Retained length, first mismatch, and bounded evidence from an ordered identity rejection.
pub struct ExactIdentityRejection<'a, U: ?Sized> {
    expected: &'a [&'a U],
    length: usize,
    mismatch: Option<IdentityMismatch<'a, U>>,
    observed: ObservedTargets<'a, U>,
}

/// Retained target assignment from an unordered identity rejection.
/// Diagnostic assignment is omitted during probes.
pub struct UnorderedIdentityRejection<'a, U: ?Sized> {
    expected: &'a [&'a U],
    observed: Vec<&'a U>,
    missing: Vec<&'a U>,
    unexpected: Vec<&'a U>,
    same_address: bool,
}

/// Debug-formats target references as their addresses.
struct Addresses<'a, U: ?Sized>(&'a [&'a U]);

impl<U: ?Sized> core::fmt::Debug for Addresses<'_, U> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_list()
            .entries(self.0.iter().map(|target| ptr::from_ref(*target)))
            .finish()
    }
}

/// Implements `Clone`, `Copy`, `Debug`, and `new` for an expectation borrowing one target. A
/// derive would wrongly require `U: Clone`, and `Debug` shows only the target's address because
/// identity never inspects its contents.
macro_rules! single_target_traits {
    ($name:ident) => {
        impl<U: ?Sized> Clone for $name<'_, U> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<U: ?Sized> Copy for $name<'_, U> {}

        impl<U: ?Sized> core::fmt::Debug for $name<'_, U> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&ptr::from_ref(self.0))
                    .finish()
            }
        }

        impl<'e, U: ?Sized> $name<'e, U> {
            /// Borrows the expected target, including pointer metadata for unsized targets.
            #[must_use]
            pub const fn new(expected: &'e U) -> Self {
                Self(expected)
            }
        }
    };
}

/// Implements `Clone`, `Debug`, and `new` for an expectation storing expected target references
/// as `expected: B`. Only `B` needs `Clone`, and `Debug` shows the targets' addresses because
/// identity never inspects their contents.
macro_rules! target_list_traits {
    ($name:ident) => {
        impl<'e, U: ?Sized + 'e, B: Clone> Clone for $name<'e, U, B> {
            fn clone(&self) -> Self {
                Self {
                    expected: self.expected.clone(),
                    target: PhantomData,
                }
            }
        }

        impl<'e, U: ?Sized + 'e, B: AsRef<[&'e U]>> core::fmt::Debug for $name<'e, U, B> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("expected", &Addresses(self.expected.as_ref()))
                    .finish()
            }
        }

        impl<'e, U: ?Sized + 'e, B: AsRef<[&'e U]>> $name<'e, U, B> {
            /// Stores expected target references without converting their storage yet.
            #[must_use]
            pub const fn new(expected: B) -> Self {
                Self {
                    expected,
                    target: PhantomData,
                }
            }
        }
    };
}

/// Checks that some collection element borrows the same instance as the expected target, without
/// equality or target rendering capabilities.
pub struct ContainsSameInstanceAs<'e, U: ?Sized>(&'e U);

single_target_traits!(ContainsSameInstanceAs);

impl<C: Collection + ?Sized, U: ?Sized, R> Expectation<C, R> for ContainsSameInstanceAs<'_, U>
where
    C::Item: Borrow<U>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = IdentityMembershipRejection<'a, U>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let mut same_address = false;
        let mut observed = ObservedTargets::new(context);
        for element in actual.elements() {
            let target = observed.observe(element.borrow());
            same_address |= ptr::addr_eq(target, self.0);
            if ptr::eq(target, self.0) {
                return Ok(());
            }
        }
        Err(IdentityMembershipRejection {
            observed,
            same_address,
        })
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let rendering = context.render().identities();
        let failure = match rejected {
            None => failure.relation("contains the same instance as"),
            Some((
                actual,
                IdentityMembershipRejection {
                    observed,
                    same_address,
                },
            )) => {
                let mut failure = failure
                    .actual(observed.render(actual, C::PRESENTATION.order(), context))
                    .relation("does not contain the same instance as");
                if same_address {
                    failure = failure.fact(Fact::note(METADATA_NOTE));
                }
                failure
            }
        };
        failure.expected(rendering.value(self.0))
    }
}

/// Checks that no collection element borrows the same instance as the expected target, without
/// equality or target rendering capabilities.
pub struct DoesNotContainSameInstanceAs<'e, U: ?Sized>(&'e U);

single_target_traits!(DoesNotContainSameInstanceAs);

impl<C: Collection + ?Sized, U: ?Sized, R> Expectation<C, R> for DoesNotContainSameInstanceAs<'_, U>
where
    C::Item: Borrow<U>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = IdentityMembershipRejection<'a, U>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let mut observed = ObservedTargets::new(context);
        for element in actual.elements() {
            if ptr::eq(observed.observe(element.borrow()), self.0) {
                return Err(IdentityMembershipRejection {
                    observed,
                    same_address: false,
                });
            }
        }
        Ok(())
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let actual = rejected.map(|(actual, IdentityMembershipRejection { observed, .. })| {
            observed.render(actual, C::PRESENTATION.order(), context)
        });
        failure
            .relations(
                actual,
                "does not contain the same instance as",
                "contains the same instance as",
            )
            .unexpected(context.render().identities().value(self.0))
    }
}

/// Requires exact borrowed-target identity, including multiplicity and pointer metadata.
pub struct ContainsExactlySameInstances<'e, U: ?Sized + 'e, B = Vec<&'e U>> {
    expected: B,
    target: PhantomData<&'e U>,
}

target_list_traits!(ContainsExactlySameInstances);

impl<'e, C: StableOrder + ?Sized, U: ?Sized + 'e, B, R> Expectation<C, R>
    for ContainsExactlySameInstances<'e, U, B>
where
    C::Item: Borrow<U>,
    B: AsRef<[&'e U]>,
    R: crate::renderer::ValueRenderer<usize>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = ExactIdentityRejection<'a, U>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let length = actual.length();
        let mut observed = ObservedTargets::new(context);
        let mismatch = actual
            .elements()
            .map(|element| observed.observe(element.borrow()))
            .zip(expected.iter().copied())
            .enumerate()
            .find(|(_, (element, expected))| !ptr::eq(*element, *expected))
            .map(|(index, (element, expected))| IdentityMismatch {
                index,
                actual: element,
                expected,
                same_address: ptr::addr_eq(element, expected),
            });
        if length != expected.len() || mismatch.is_some() {
            Err(ExactIdentityRejection {
                expected,
                length,
                mismatch,
                observed,
            })
        } else {
            Ok(())
        }
    }

    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let rendering = render.identities();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("contains exactly the same instances in order"),
            Some((actual, rejection)) => {
                let ExactIdentityRejection {
                    expected: _,
                    length,
                    mismatch,
                    observed,
                } = rejection;
                let mut failure = failure
                    .actual(observed.render(actual, RenderingOrder::PreserveIteration, context))
                    .relation("does not contain exactly the same instances in order");
                if length != expected.len() {
                    failure = failure.facts(length_facts(render, length, expected.len()));
                }
                if let Some(IdentityMismatch {
                    index,
                    actual: element,
                    expected,
                    same_address,
                }) = mismatch
                {
                    if same_address {
                        failure = failure.fact(Fact::note(METADATA_NOTE));
                    }
                    let mut child = context.isolated();
                    child.record(|context| {
                        let rendering = context.render().identities();
                        FailureBuilder::new::<U>(FailureKind::Equality)
                            .actual(rendering.value(element))
                            .relation("is not the same instance as")
                            .expected(rendering.value(expected))
                            .path([PathSegment::Index(index)])
                            .build()
                    });
                    failure = failure.evidence(child.into_evidence());
                }
                failure
            }
        };
        failure.expected(
            rendering.borrowed_values::<U, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

/// Requires exact borrowed-target identity, including multiplicity and pointer metadata.
pub struct ContainsExactlySameInstancesInAnyOrder<'e, U: ?Sized + 'e, B = Vec<&'e U>> {
    expected: B,
    target: PhantomData<&'e U>,
}

target_list_traits!(ContainsExactlySameInstancesInAnyOrder);

impl<'e, C: Collection + ?Sized, U: ?Sized + 'e, B, R> Expectation<C, R>
    for ContainsExactlySameInstancesInAnyOrder<'e, U, B>
where
    C::Item: Borrow<U>,
    B: AsRef<[&'e U]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = UnorderedIdentityRejection<'a, U>
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        // A probe of a different length cannot match, so it rejects without borrowing targets.
        if context.is_probe() && actual.length() != expected.len() {
            return Err(UnorderedIdentityRejection {
                expected,
                observed: Vec::new(),
                missing: Vec::new(),
                unexpected: Vec::new(),
                same_address: false,
            });
        }
        let elements = actual.elements().map(Borrow::borrow).collect::<Vec<&U>>();
        let unmatched = assign_exactly(
            context.is_probe(),
            elements.len(),
            expected.len(),
            |a, e| ptr::eq(elements[a], expected[e]),
        );
        let (unexpected, missing) = match unmatched {
            Ok(()) => return Ok(()),
            Err(None) => (Vec::new(), Vec::new()),
            Err(Some(matched)) => matched.unmatched(&elements, |index| expected[index]),
        };
        let same_address = unexpected.iter().any(|actual| {
            missing
                .iter()
                .any(|expected| ptr::addr_eq(*actual, *expected))
        });
        Err(UnorderedIdentityRejection {
            expected,
            observed: elements,
            missing,
            unexpected,
            same_address,
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
        let rendering = render.identities();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("contains exactly the same instances in any order"),
            Some((actual, rejection)) => {
                let UnorderedIdentityRejection {
                    observed,
                    missing,
                    unexpected,
                    same_address,
                    ..
                } = rejection;

                let failure = failure
                    .actual(rendering.observed_collection(
                        actual,
                        &observed,
                        C::PRESENTATION.order(),
                    ))
                    .relation("does not contain exactly the same instances in any order");
                let mut failure = super::value::unmatched_facts(
                    failure,
                    rendering,
                    ["Instances not found", "Instances not expected"],
                    &missing,
                    &unexpected,
                    C::PRESENTATION.order(),
                );
                if same_address {
                    failure = failure.fact(Fact::note(METADATA_NOTE));
                }
                failure
            }
        };
        failure.expected(
            rendering.borrowed_values::<U, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use super::*;
    use crate::{
        prelude::*,
        renderer::{CollectionPresentation, Rendered, RenderedBody, RenderingContext},
        test_support::{NoRenderer, NumericRenderer, PreservedBag, UnorderedSet},
    };

    struct Opaque {
        _byte: u8,
    }

    fn keys() -> [Opaque; 3] {
        [
            Opaque { _byte: 1 },
            Opaque { _byte: 1 },
            Opaque { _byte: 1 },
        ]
    }

    fn items(value: &Rendered) -> &[Rendered] {
        let RenderedBody::Group { items, .. } = &value.body else {
            panic!("expected an identity group");
        };
        items
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        fn are_as_expected() {
            let keys = keys();
            let actual = [&keys[0], &keys[1]];
            actual.must().contain_same_instance_as(&keys[0]);
            actual.must().not_contain_same_instance_as(&keys[2]);
            actual
                .must()
                .contain_exactly_same_instances([&keys[0], &keys[1]]);
            actual
                .must()
                .contain_exactly_same_instances_in_any_order([&keys[1], &keys[0]]);
        }
    }

    mod contains_same_instance_as {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_same_instance_as(&keys[1])
            );
        }

        #[test]
        fn finds_the_borrowed_pointee_instead_of_the_reference_slot() {
            let keys = keys();
            let actual = [&keys[0], &keys[1]];
            let assertion = assert_that!(actual)
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&keys[1]);
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn rejects_distinct_equal_instances_with_an_exact_report() {
            let keys = keys();
            let actual = [&keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| it.contains_same_instance_as(&keys[1]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(FailureKind::Membership);
                    element.has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                ]

                does not contain the same instance as

                Expected: {e:p}
                -------- assertr --------
            ", a = &keys[0], e = &keys[1]});
                },
            ]);
        }

        #[test]
        fn rejects_an_empty_collection() {
            let keys = keys();
            let failures = assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(&keys[0]));
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn explains_a_slice_metadata_mismatch() {
            let data = [1, 2];
            let short = &data[..1];
            let long = &data[..];
            let failures = assert_that!([short])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(long));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.facts)
                        .is_equal_to([Fact::note(METADATA_NOTE)]);
                    element
                        .derive_owned(|value| {
                            format!("{:#}", items(value.actual.as_ref().unwrap())[0])
                        })
                        .is_equal_to(format!("{:#}", element.actual().expected.as_ref().unwrap()));
                },
            ]);
        }

        #[test]
        fn panic_mode_raises_the_identity_failure() {
            let keys = keys();
            assert_that!(|| {
                assert_that!([&keys[0]])
                    .with_renderer(NoRenderer)
                    .contains_same_instance_as(&keys[1]);
            })
            .panics()
            .has_type::<String>()
            .contains("does not contain the same instance as");
        }
    }

    mod does_not_contain_same_instance_as {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                does_not_contain_same_instance_as(&keys[0])
            );
        }

        #[test]
        fn accepts_missing_instances_and_empty_collections() {
            let keys = keys();
            assert_that!([&keys[0]])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&keys[1]);
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&keys[0]);
        }

        #[test]
        fn reports_the_unexpected_instance_without_an_index() {
            let keys = keys();
            let actual = [&keys[0], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| it.does_not_contain_same_instance_as(&keys[0]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| &value.expected).is_none();
                    element.derive(|value| &value.facts).is_empty();
                    element.derive(|value| &value.children).is_empty();
                    element.has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                    {a:p},
                ]

                contains the same instance as

                Unexpected: {a:p}
                -------- assertr --------
            ", a = &keys[0]});
                },
            ]);
        }

        #[test]
        fn same_address_with_different_metadata_is_not_a_match() {
            let data = [1, 2];
            assert_that!([&data[..1]])
                .with_renderer(NoRenderer)
                .does_not_contain_same_instance_as(&data[..]);
        }
    }

    mod contains_exactly_same_instances {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_exactly_same_instances([&keys[1]])
            );
        }

        #[test]
        fn accepts_the_requested_candidate_order_and_duplicate_references() {
            let keys = keys();
            let candidates = [&keys[1], &keys[2], &keys[0]];
            assert_that!(candidates)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([&keys[1], &keys[2], &keys[0]]);
            let repeated = [&keys[0], &keys[0]];
            let assertion = assert_that!(repeated)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([&keys[0], &keys[0]]);
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn reports_the_first_reordered_pair_as_an_indexed_child() {
            let keys = keys();
            let actual = [&keys[1], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly_same_instances([&keys[0], &keys[1]]));
            assert_that!(failures).contains_exactly_satisfying([
                |item: AssertThat<AssertionFailure, Capture>| {
                    item.derive(|subject| &subject.children)
                        .contains_exactly_satisfying([
                            |element: AssertThat<AssertionFailure, Capture>| {
                                element
                                    .derive(|value| &value.path)
                                    .is_equal_to([PathSegment::Index(0)]);
                                element.derive(|value| &value.facts).is_empty();
                            },
                        ]);
                    item.has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {b:p},
                    {a:p},
                ]

                does not contain exactly the same instances in order

                Expected: [
                    {a:p},
                    {b:p},
                ]

                Nested failures:
                  - At [0]:
                    Actual: {b:p}

                    is not the same instance as

                    Expected: {a:p}
                -------- assertr --------
            ", a = &keys[0], b = &keys[1]});
                },
            ]);
        }

        #[test]
        fn checks_lengths_even_when_the_common_prefix_matches() {
            let keys = keys();
            for (actual, expected) in [
                (vec![&keys[0]], vec![]),
                (vec![], vec![&keys[0]]),
                (vec![&keys[0], &keys[0]], vec![&keys[0]]),
                (vec![&keys[0]], vec![&keys[0], &keys[0]]),
            ] {
                let failures = assert_that!(actual)
                    .with_renderer(NumericRenderer)
                    .capture(|it| it.contains_exactly_same_instances(&expected));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| &item.facts).is_equal_to([
                            Fact::labelled(
                                "Actual length",
                                RenderingContext::new(&DebugRenderer, RenderingBudget::default())
                                    .value(&actual.len()),
                            ),
                            Fact::labelled(
                                "Expected length",
                                RenderingContext::new(&DebugRenderer, RenderingBudget::default())
                                    .value(&expected.len()),
                            ),
                        ]);
                        element
                            .derive_owned(|item| item.children.is_empty())
                            .is_true();
                    },
                ]);
            }
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([] as [&Opaque; 0]);
        }

        #[test]
        fn checks_metadata_and_keeps_the_borrowed_child_type() {
            let data = [1, 2];
            let failures = assert_that!([&data[..1]])
                .with_renderer(NumericRenderer)
                .capture(|it| it.contains_exactly_same_instances([&data[..]]));
            assert_that!(failures[0].facts).contains_exactly([Fact::note(METADATA_NOTE)]);
            assert_that!(failures[0].children[0].subject_type_name).is_equal_to("[i32]");
        }

        #[test]
        fn length_facts_need_only_a_numeric_renderer() {
            use crate::test_support::assert_custom_value;
            struct Numbers;
            impl ValueRenderer<usize> for Numbers {
                fn fmt(
                    &self,
                    value: &usize,
                    f: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result {
                    write!(f, "custom({value})")
                }
            }
            let keys = keys();
            let subject = [&keys[0]];
            let failures = assert_that!(subject)
                .with_renderer(Numbers)
                .with_location(false)
                .capture(|it| it.contains_exactly_same_instances([] as [&Opaque; 0]));

            assert_custom_value(&failures[0].facts[0].value, &1_usize);
            assert_custom_value(&failures[0].facts[1].value, &0_usize);
            let pointer = format!("{:p}", subject[0]);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: [
                    {pointer},
                ]

                does not contain exactly the same instances in order

                Expected: []

                Details:
                  - Actual length: custom(1)
                  - Expected length: custom(0)
                -------- assertr --------
            "});
        }
    }

    mod contains_exactly_same_instances_in_any_order {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let keys = keys();
            assert_caller_location!(
                assert_that!([&keys[0]]),
                contains_exactly_same_instances_in_any_order([&keys[1]])
            );
        }

        #[test]
        fn accepts_reordering_and_matching_duplicate_counts() {
            let keys = keys();
            let repeated = [&keys[0], &keys[1], &keys[0]];
            let assertion = assert_that!(repeated)
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([&keys[0], &keys[0], &keys[1]]);
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
            assert_that!([] as [&Opaque; 0])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([] as [&Opaque; 0]);
        }

        #[test]
        fn reports_missing_and_unexpected_duplicate_occurrences() {
            let keys = keys();
            let actual = [&keys[0], &keys[0], &keys[0]];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| {
                    it.contains_exactly_same_instances_in_any_order([&keys[0], &keys[1], &keys[1]])
                });
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| &value.children).is_empty();
                    element.has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: [
                    {a:p},
                    {a:p},
                    {a:p},
                ]

                does not contain exactly the same instances in any order

                Expected: [
                    {a:p},
                    {b:p},
                    {b:p},
                ]

                Details:
                  - Instances not found: [
                        {b:p},
                        {b:p},
                    ]
                  - Instances not expected: [
                        {a:p},
                        {a:p},
                    ]
                -------- assertr --------
            ", a = &keys[0], b = &keys[1]});
                },
            ]);
        }

        #[test]
        fn rejects_missing_and_extra_occurrences_including_empty_sides() {
            let keys = keys();
            for (actual, expected, label) in [
                (vec![&keys[0]], vec![], "Instances not expected"),
                (vec![], vec![&keys[0]], "Instances not found"),
                (
                    vec![&keys[0], &keys[0]],
                    vec![&keys[0]],
                    "Instances not expected",
                ),
                (
                    vec![&keys[0]],
                    vec![&keys[0], &keys[0]],
                    "Instances not found",
                ),
            ] {
                let failures = assert_that!(actual)
                    .with_renderer(NoRenderer)
                    .capture(|it| it.contains_exactly_same_instances_in_any_order(&expected));
                assert_that!(failures).contains_exactly_satisfying([
                    |item: AssertThat<AssertionFailure, Capture>| {
                        item.derive(|subject| &subject.facts)
                            .contains_exactly_satisfying([|element: AssertThat<Fact, Capture>| {
                                element
                                    .derive_owned(|value| value.label.as_deref())
                                    .is_equal_to(Some(label));
                            }]);
                    },
                ]);
            }
        }

        #[test]
        fn distinguishes_slice_metadata_when_matching_instances() {
            let data = [1, 2];
            let short = &data[..1];
            let long = &data[..];
            assert_that!([short, long])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([long, short]);
            let failures = assert_that!([short, short])
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_exactly_same_instances_in_any_order([short, long]));
            assert_that!(failures[0].facts.last().unwrap()).is_equal_to(Fact::note(METADATA_NOTE));
        }
    }

    mod adapters {
        use alloc::{
            collections::{BTreeSet, BinaryHeap, LinkedList, VecDeque},
            rc::Rc,
        };

        use super::*;

        fn check_ordered<C: StableOrder + ?Sized>(actual: &C, expected: [&Opaque; 3])
        where
            C::Item: Borrow<Opaque>,
        {
            assert_that!(actual)
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(expected[1])
                .contains_exactly_same_instances(expected)
                .contains_exactly_same_instances_in_any_order([
                    expected[2],
                    expected[0],
                    expected[1],
                ]);
        }

        #[test]
        fn ordered_adapters_borrow_owned_values_and_references() {
            let keys = keys();
            let expected = [&keys[0], &keys[1], &keys[2]];
            check_ordered(&keys, expected);
            check_ordered(keys.as_slice(), expected);
            check_ordered(&expected, expected);
            check_ordered(&expected.to_vec(), expected);
            check_ordered(&VecDeque::from(expected), expected);
            check_ordered(&LinkedList::from(expected), expected);
        }

        #[test]
        fn smart_pointers_borrow_their_opaque_targets() {
            let boxed = Box::new(Opaque { _byte: 1 });
            assert_that!([&boxed])
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&boxed);
            let actual = [boxed];
            assert_that!(actual)
                .with_renderer(NumericRenderer)
                .contains_exactly_same_instances([actual[0].as_ref()]);
            let shared = Rc::new(Opaque { _byte: 1 });
            assert_that!([Rc::clone(&shared), Rc::clone(&shared)])
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order([shared.as_ref(), shared.as_ref()]);
            #[cfg(target_has_atomic = "ptr")]
            {
                let shared = alloc::sync::Arc::new(Opaque { _byte: 1 });
                assert_that!([alloc::sync::Arc::clone(&shared)])
                    .with_renderer(NoRenderer)
                    .contains_same_instance_as(shared.as_ref());
            }
        }

        #[test]
        fn order_free_adapters_expose_instance_membership() {
            let values = [1, 2];
            let expected = [&values[1], &values[0]];
            assert_that!(BTreeSet::from(expected))
                .with_renderer(NoRenderer)
                .contains_same_instance_as(&values[0])
                .contains_exactly_same_instances_in_any_order(expected);
            assert_that!(BinaryHeap::from(expected))
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order(expected);
            #[cfg(feature = "std")]
            assert_that!(std::collections::HashSet::from(expected))
                .with_renderer(NoRenderer)
                .contains_exactly_same_instances_in_any_order(expected);
        }

        #[test]
        fn supports_str_and_trait_object_targets() {
            trait Marker {}
            impl Marker for Opaque {}

            let text = String::from("text");
            assert_that!([text.as_str()])
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(text.as_str())
                .contains_exactly_same_instances([text.as_str()]);
            let key = Opaque { _byte: 1 };
            let object: &dyn Marker = &key;
            assert_that!([object])
                .with_renderer(NumericRenderer)
                .contains_same_instance_as(object)
                .contains_exactly_same_instances([object])
                .contains_exactly_same_instances_in_any_order([object]);
        }
    }

    mod rendering {
        use super::*;

        #[test]
        fn sorts_order_free_evidence_using_presentation_metadata() {
            let actual = UnorderedSet(vec![1, 2]);
            let expected = [9, 10];
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .capture(|it| {
                    it.contains_exactly_same_instances_in_any_order([&expected[1], &expected[0]])
                });
            let mut addresses = actual
                .0
                .iter()
                .map(|value| format!("{value:p}"))
                .collect::<Vec<_>>();
            addresses.sort();
            for rendered in [
                failures[0].actual.as_ref().unwrap(),
                &failures[0].facts[1].value,
            ] {
                assert_that!(rendered.body)
                    .is_matching(pattern!(RenderedBody::Group { sorted: true, .. }));
                assert_that!(
                    items(rendered)
                        .iter()
                        .map(|value| format!("{value:#}"))
                        .collect::<Vec<_>>()
                )
                .contains_exactly(&addresses);
            }
            let actual = PreservedBag(vec![2, 1]);
            let failures = assert_that!(actual)
                .with_renderer(NoRenderer)
                .capture(|it| it.contains_same_instance_as(&expected[0]));
            assert_that!(failures[0].actual.as_ref().unwrap().body)
                .is_matching(pattern!(RenderedBody::Group { sorted: false, .. }));
        }

        #[test]
        fn positional_evidence_ignores_presentation_sorting() {
            use crate::assertions::HasLength;

            struct SortedPresentation<'a>([&'a Opaque; 2]);
            impl HasLength for SortedPresentation<'_> {
                fn length(&self) -> usize {
                    2
                }
            }
            impl<'a> Collection for SortedPresentation<'a> {
                type Item = &'a Opaque;
                const PRESENTATION: CollectionPresentation =
                    CollectionPresentation::list().with_order(RenderingOrder::SortByRenderedText);
                fn elements(&self) -> impl Iterator<Item = &Self::Item> {
                    self.0.iter()
                }
            }
            impl StableOrder for SortedPresentation<'_> {}
            let keys = keys();
            let mut references = [&keys[0], &keys[1]];
            references
                .sort_by_key(|value| core::cmp::Reverse(format!("{value:p}", value = *value)));
            let actual = SortedPresentation(references);
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .capture(|it| it.contains_exactly_same_instances([references[1], references[0]]));
            let rendered = failures[0].actual.as_ref().unwrap();
            assert_that!(rendered.body)
                .is_matching(pattern!(RenderedBody::Group { sorted: false, .. }));
            assert_that!(items(rendered)[0].type_name)
                .is_equal_to(Some(core::any::type_name::<Opaque>()));
            assert_that!(format!("{:#}", items(rendered)[0]))
                .is_equal_to(format!("{:p}", references[0]));
            assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::Index(0)]);
            assert_that!(failures[0].children[0].facts).is_empty();
        }

        #[test]
        fn budgets_limit_evidence_without_limiting_comparison() {
            let keys = keys();
            let actual = [&keys[0], &keys[1]];
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(1)
                        .with_max_leaf_characters(2),
                )
                .capture(|it| it.contains_exactly_same_instances([&keys[0], &keys[2]]));
            let rendered = failures[0].actual.as_ref().unwrap();
            assert_that!(rendered.body)
                .is_matching(pattern!(RenderedBody::Group { omitted: 1, .. }));
            assert_that!(&items(rendered)[0].body).is_matching(pattern!(
                RenderedBody::Text { text, omitted_characters }
                    if text == "0x" && *omitted_characters > 0
            ));
            assert_that!(failures[0].children[0].path).contains_exactly([PathSegment::Index(1)]);
            assert_that!(failures[0].children[0].facts).is_empty();

            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(0)
                        .with_max_leaf_characters(0),
                )
                .capture(|it| {
                    it.contains_same_instance_as(&keys[1])
                        .contains_exactly_same_instances([&keys[0], &keys[2]])
                        .contains_exactly_same_instances_in_any_order([&keys[0], &keys[2]])
                });
            assert_that!(failures).contains_exactly_satisfying([false, true].map(|unordered| {
                move |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive(|failure| &failure.actual)
                        .is_some_satisfying(|actual| {
                            actual.derive_owned(items).is_empty();
                        });
                    failure.derive(|failure| &failure.children).is_empty();
                    failure
                        .derive(|failure| &failure.omitted_children)
                        .is_equal_to(usize::from(!unordered));
                    let facts = failure.derive(|failure| &failure.facts);
                    if unordered {
                        facts.contains_exactly_satisfying(
                            [|fact: AssertThat<Fact, Capture>| {
                                fact.derive_owned(|fact| items(&fact.value)).is_empty();
                            }; 2],
                        );
                    } else {
                        facts.is_empty();
                    }
                }
            }));
        }
    }

    mod evaluation {
        use core::cell::Cell;

        use super::*;

        #[test]
        fn membership_rejections_render_within_the_budget() {
            let targets = [0_u32; 4096];
            let missing = 1_u32;
            for limit in [0, 1, 3] {
                let failures = assert_that!(targets)
                    .with_renderer(NumericRenderer)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| {
                        it.contains_same_instance_as(&missing)
                            .does_not_contain_same_instance_as(&targets[4095])
                    });
                for failure in &failures {
                    let RenderedBody::Group { items, omitted, .. } =
                        &failure.actual.as_ref().unwrap().body
                    else {
                        panic!("expected identity evidence")
                    };
                    assert_that!(items).has_length(limit);
                    assert_that!(*omitted).is_equal_to(targets.len() - limit);
                }
            }
        }

        #[test]
        fn probes_retain_no_identity_targets() {
            let targets = [0_u32; 4096];
            let missing = 1_u32;
            let context = AssertionContext::new(&DebugRenderer, RenderingBudget::unlimited())
                .with_diagnostics(false);
            let matcher = ContainsSameInstanceAs::new(&missing);
            let rejection = matcher.evaluate(&targets, &context).err().unwrap();
            assert_that!(rejection.observed.targets).is_empty();
            let expected = targets.iter().rev().collect::<Vec<_>>();
            let matcher = ContainsExactlySameInstances::new(expected);
            let rejection = matcher.evaluate(&targets, &context).err().unwrap();
            assert_that!(rejection.observed.targets).is_empty();
        }

        #[test]
        fn sorted_evidence_matches_full_rendering_before_truncation() {
            let missing = 99_i32;
            let actual = UnorderedSet((0..32).rev().collect());
            for limit in [0, 1, 3, 32, 33, usize::MAX] {
                for leaf_limit in [0, 4, usize::MAX] {
                    let budget = RenderingBudget::default()
                        .with_max_items(limit)
                        .with_max_leaf_characters(leaf_limit);
                    let failures = assert_that!(&actual)
                        .with_renderer(NoRenderer)
                        .with_rendering_budget(budget)
                        .capture(|it| it.contains_same_instance_as(&missing));
                    let expected = RenderingContext::new(&DebugRenderer, budget)
                        .identities()
                        .collection(&actual);
                    assert_that!(failures[0].actual.as_ref()).is_equal_to(Some(&expected));
                }
            }
        }

        struct Expected<'a> {
            values: [&'a Opaque; 1],
            views: &'a Cell<usize>,
        }
        impl<'a> AsRef<[&'a Opaque]> for Expected<'a> {
            fn as_ref(&self) -> &[&'a Opaque] {
                self.views.set(self.views.get() + 1);
                &self.values
            }
        }

        #[test]
        fn identity_diagnostics_reuse_the_expected_reference_slice() {
            let keys = keys();
            let views = Cell::new(0);
            let expected = || Expected {
                values: [&keys[1]],
                views: &views,
            };
            let failures = assert_that!([&keys[0]])
                .with_renderer(NumericRenderer)
                .capture(|it| {
                    it.contains_exactly_same_instances(expected())
                        .contains_exactly_same_instances_in_any_order(expected())
                });
            assert_that!(views.get()).is_equal_to(2);
            assert_that!(failures).has_length(2);
        }
        #[test]
        fn rejection_diagnostics_do_not_borrow_inspected_targets_again() {
            struct Target<'a> {
                target: &'a Opaque,
                borrows: &'a Cell<usize>,
            }
            impl Borrow<Opaque> for Target<'_> {
                fn borrow(&self) -> &Opaque {
                    self.borrows.set(self.borrows.get() + 1);
                    self.target
                }
            }
            let keys = keys();
            let borrows = [Cell::new(0), Cell::new(0)];
            let actual = [
                Target {
                    target: &keys[0],
                    borrows: &borrows[0],
                },
                Target {
                    target: &keys[1],
                    borrows: &borrows[1],
                },
            ];
            let failures = assert_that!(actual)
                .with_renderer(NumericRenderer)
                .capture(|it| {
                    let it = it.contains_same_instance_as(&keys[2]);
                    assert_that!((borrows[0].get(), borrows[1].get())).is_equal_to((1, 1));
                    let it = it.does_not_contain_same_instance_as(&keys[0]);
                    assert_that!((borrows[0].get(), borrows[1].get())).is_equal_to((2, 2));
                    let it = it.contains_exactly_same_instances([&keys[1], &keys[0]]);
                    assert_that!((borrows[0].get(), borrows[1].get())).is_equal_to((3, 3));
                    let it = it.contains_exactly_same_instances_in_any_order([&keys[2]]);
                    assert_that!((borrows[0].get(), borrows[1].get())).is_equal_to((4, 4));
                    it
                });
            assert_that!(failures).has_length(4);
        }
    }
}
