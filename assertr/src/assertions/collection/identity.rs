//! Reusable borrowed-target identity expectations for finite collections.

use super::{Collection, StableOrder};
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{GroupStyle, IntoRendered, Rendered, RenderingOrder},
    util::matching::match_bipartite,
};
use alloc::{string::String, vec::Vec};
use core::{borrow::Borrow, marker::PhantomData, ptr};

const METADATA_NOTE: &str = "Some pointers have equal data addresses but different metadata.";

// Retain only the diagnostic prefix, or the smallest rendered identities for sorted
// presentation. Inspected counts are independent of retention, so explanation never repeats
// Borrow on a target that evaluation already visited.
struct ObservedTargets<'a, U: ?Sized> {
    targets: Vec<&'a U>,
    sort_keys: Vec<String>,
    inspected: usize,
    limit: usize,
    order: RenderingOrder,
}

impl<'a, U: ?Sized> ObservedTargets<'a, U> {
    fn new<R>(context: &AssertionContext<'_, R>, order: RenderingOrder) -> Self {
        Self {
            targets: Vec::new(),
            sort_keys: Vec::new(),
            inspected: 0,
            limit: if context.is_diagnostic() {
                context.render().max_items()
            } else {
                0
            },
            order,
        }
    }

    fn observe<R>(&mut self, target: &'a U, context: &AssertionContext<'_, R>) {
        self.inspected += 1;
        if self.limit == 0 {
            return;
        }
        if self.order == RenderingOrder::PreserveIteration {
            if self.targets.len() < self.limit {
                self.targets.push(target);
            }
            return;
        }
        // Compare the budgeted text, including truncation markers, just like collection rendering.
        let key = context
            .render()
            .identities()
            .value(target)
            .into_rendered()
            .text(true);
        let index = self.sort_keys.partition_point(|existing| existing <= &key);
        if index < self.limit {
            if self.targets.len() == self.limit {
                self.targets.pop();
                self.sort_keys.pop();
            }
            self.targets.insert(index, target);
            self.sort_keys.insert(index, key);
        }
    }

    fn complete<C: Collection + ?Sized, R>(
        &mut self,
        actual: &'a C,
        context: &AssertionContext<'_, R>,
    ) where
        C::Item: Borrow<U>,
    {
        let remaining = if self.limit > 0 && self.order == RenderingOrder::SortByRenderedText {
            usize::MAX
        } else {
            self.limit.saturating_sub(self.targets.len())
        };
        for element in actual.elements().skip(self.inspected).take(remaining) {
            self.observe(element.borrow(), context);
        }
    }

    fn render<C: Collection + ?Sized, R>(
        &self,
        actual: &C,
        context: &AssertionContext<'_, R>,
    ) -> Rendered {
        context.render().identities().observed_collection(
            actual,
            &self.targets,
            actual.length(),
            self.order,
        )
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
pub struct UnorderedIdentityRejection<'a, U: ?Sized> {
    expected: &'a [&'a U],
    observed: Vec<&'a U>,
    missing: Vec<&'a U>,
    unexpected: Vec<&'a U>,
    same_address: bool,
}

/// Checks borrowed-target collection identity without equality or target rendering capabilities.
pub struct ContainsSameInstanceAs<'e, U: ?Sized>(&'e U);
impl<'e, U: ?Sized> ContainsSameInstanceAs<'e, U> {
    /// Borrows the expected target, including pointer metadata for unsized targets.
    #[must_use]
    pub const fn new(expected: &'e U) -> Self {
        Self(expected)
    }
}

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
        let mut observed = ObservedTargets::new(context, C::PRESENTATION.order());
        for element in actual.elements() {
            let target = <C::Item as Borrow<U>>::borrow(element);
            observed.observe(target, context);
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
}

impl<C: Collection + ?Sized, U: ?Sized, R> ExpectationDiagnostics<C, R>
    for ContainsSameInstanceAs<'_, U>
where
    C::Item: Borrow<U>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
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
                    .actual(observed.render(actual, context))
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

/// Checks borrowed-target collection identity without equality or target rendering capabilities.
pub struct DoesNotContainSameInstanceAs<'e, U: ?Sized>(&'e U);
impl<'e, U: ?Sized> DoesNotContainSameInstanceAs<'e, U> {
    /// Borrows the expected target, including pointer metadata for unsized targets.
    #[must_use]
    pub const fn new(expected: &'e U) -> Self {
        Self(expected)
    }
}

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
        let mut observed = ObservedTargets::new(context, C::PRESENTATION.order());
        for element in actual.elements() {
            let target = <C::Item as Borrow<U>>::borrow(element);
            observed.observe(target, context);
            if ptr::eq(target, self.0) {
                return Err(IdentityMembershipRejection {
                    observed,
                    same_address: false,
                });
            }
        }
        Ok(())
    }
}

impl<C: Collection + ?Sized, U: ?Sized, R> ExpectationDiagnostics<C, R>
    for DoesNotContainSameInstanceAs<'_, U>
where
    C::Item: Borrow<U>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let rendering = context.render().identities();
        let failure = match rejected {
            None => failure.relation("does not contain the same instance as"),
            Some((actual, IdentityMembershipRejection { mut observed, .. })) => {
                observed.complete(actual, context);
                failure
                    .actual(observed.render(actual, context))
                    .relation("contains the same instance as")
            }
        };
        failure.unexpected(rendering.value(self.0))
    }
}

/// Requires exact borrowed-target identity, including multiplicity and pointer metadata.
pub struct ContainsExactlySameInstances<'e, U: ?Sized + 'e, B = Vec<&'e U>> {
    expected: B,
    target: PhantomData<&'e U>,
}
impl<'e, U: ?Sized + 'e, B: AsRef<[&'e U]>> ContainsExactlySameInstances<'e, U, B> {
    /// Stores expected target references without converting their storage yet.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            target: PhantomData,
        }
    }
}

impl<'e, C: StableOrder + ?Sized, U: ?Sized + 'e, B, R> Expectation<C, R>
    for ContainsExactlySameInstances<'e, U, B>
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
        let mut observed = ObservedTargets::new(context, RenderingOrder::PreserveIteration);
        let mismatch = actual
            .elements()
            .map(|element| {
                let target = <C::Item as Borrow<U>>::borrow(element);
                observed.observe(target, context);
                target
            })
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
}

impl<'e, C: StableOrder + ?Sized, U: ?Sized + 'e, B, R> ExpectationDiagnostics<C, R>
    for ContainsExactlySameInstances<'e, U, B>
where
    C::Item: Borrow<U>,
    B: AsRef<[&'e U]>,
    R: crate::ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
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
                    expected,
                    length,
                    mismatch,
                    mut observed,
                } = rejection;
                observed.complete(actual, context);

                let mut failure = failure
                    .actual(observed.render(actual, context))
                    .relation("does not contain exactly the same instances in order");
                if length != expected.len() {
                    failure = failure
                        .fact(Fact::labelled("Actual length", render.value(&length)))
                        .fact(Fact::labelled(
                            "Expected length",
                            render.value(&expected.len()),
                        ));
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
                    if rendering.max_items() == 0 {
                        failure = failure.omitted(1, "unmatched element");
                    } else {
                        failure = failure.child(
                            FailureBuilder::detached::<U>(FailureKind::Equality)
                                .actual(rendering.value(element))
                                .relation("is not the same instance as")
                                .expected(rendering.value(expected))
                                .build()
                                .located_at(Fact::index(index)),
                        );
                    }
                }
                failure
            }
        };
        failure.expected(rendering.borrowed_values::<U, _>(expected, GroupStyle::List))
    }
}

/// Requires exact borrowed-target identity, including multiplicity and pointer metadata.
pub struct ContainsExactlySameInstancesInAnyOrder<'e, U: ?Sized + 'e, B = Vec<&'e U>> {
    expected: B,
    target: PhantomData<&'e U>,
}
impl<'e, U: ?Sized + 'e, B: AsRef<[&'e U]>> ContainsExactlySameInstancesInAnyOrder<'e, U, B> {
    /// Stores expected target references without converting their storage yet.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            target: PhantomData,
        }
    }
}

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
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let elements = actual
            .elements()
            .map(<C::Item as Borrow<U>>::borrow)
            .collect::<Vec<_>>();
        let matched = match_bipartite(elements.len(), expected.len(), |a, e| {
            ptr::eq(elements[a], expected[e])
        });
        if matched.is_exact() {
            return Ok(());
        }
        let missing = matched
            .unmatched_expected
            .iter()
            .map(|index| expected[*index])
            .collect::<Vec<_>>();
        let unexpected = matched
            .unmatched_actual
            .iter()
            .map(|index| elements[*index])
            .collect::<Vec<_>>();
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
}

impl<'e, C: Collection + ?Sized, U: ?Sized + 'e, B, R> ExpectationDiagnostics<C, R>
    for ContainsExactlySameInstancesInAnyOrder<'e, U, B>
where
    C::Item: Borrow<U>,
    B: AsRef<[&'e U]>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
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

                let mut failure = failure
                    .actual(rendering.observed_collection(
                        actual,
                        &observed,
                        actual.length(),
                        C::PRESENTATION.order(),
                    ))
                    .relation("does not contain exactly the same instances in any order");
                if !missing.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Instances not found",
                        rendering.borrowed_values::<U, _>(&missing, GroupStyle::List),
                    ));
                }
                if !unexpected.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Instances not expected",
                        rendering
                            .borrowed_values::<U, _>(&unexpected, GroupStyle::List)
                            .with_order(C::PRESENTATION.order()),
                    ));
                }
                if same_address {
                    failure = failure.fact(Fact::note(METADATA_NOTE));
                }
                failure
            }
        };
        failure.expected(rendering.borrowed_values::<U, _>(expected, GroupStyle::List))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::*,
        renderer::{CollectionPresentation, Rendered, RenderedBody, RenderingContext},
        test_support::{NoRenderer, NumericRenderer, PreservedBag, UnorderedSet, rendered_text},
    };
    use indoc::formatdoc;

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

    mod contains_same_instance_as {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0]].must().contain_same_instance_as(&keys[0]);
        }

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
                    element.derive(|value| value).has_text_report(formatdoc! {"
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
                            rendered_text(&items(value.actual.as_ref().unwrap())[0])
                        })
                        .is_equal_to(rendered_text(element.actual().expected.as_ref().unwrap()));
                },
            ]);
        }

        #[test]
        fn panic_mode_raises_the_identity_failure() {
            let keys = keys();
            assert_that_panic_by(|| {
                assert_that!([&keys[0]])
                    .with_renderer(NoRenderer)
                    .contains_same_instance_as(&keys[1]);
            })
            .has_type::<String>()
            .contains("does not contain the same instance as");
        }
    }

    mod does_not_contain_same_instance_as {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0]].must().not_contain_same_instance_as(&keys[1]);
        }

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
                    element
                        .derive_owned(|value| value.expected.is_none())
                        .is_true();
                    element
                        .derive_owned(|value| value.facts.is_empty())
                        .is_true();
                    element
                        .derive_owned(|value| value.children.is_empty())
                        .is_true();
                    element.derive(|value| value).has_text_report(formatdoc! {"
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[1], &keys[0]]
                .must()
                .contain_exactly_same_instances([&keys[1], &keys[0]]);
        }

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
                                    .derive(|value| &value.facts)
                                    .is_equal_to([Fact::index(0)]);
                            },
                        ]);
                    item.derive(|subject| subject).has_text_report(formatdoc! {"
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
                  - At index 0:
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let keys = keys();
            [&keys[0], &keys[1]]
                .must()
                .contain_exactly_same_instances_in_any_order([&keys[1], &keys[0]]);
        }

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
                    element
                        .derive_owned(|value| value.children.is_empty())
                        .is_true();
                    element.derive(|value| value).has_text_report(formatdoc! {"
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
                            .contains_exactly_satisfying([
                                |element: AssertThat<crate::Fact, Capture>| {
                                    element.derive(|value| &value.label).is_equal_to(label);
                                },
                            ]);
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
        use super::*;
        use alloc::{
            collections::{BTreeSet, BinaryHeap, LinkedList, VecDeque},
            rc::Rc,
        };

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
                        .map(rendered_text)
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
            assert_that!(rendered_text(&items(rendered)[0]))
                .is_equal_to(format!("{:p}", references[0]));
            assert_that!(failures[0].children[0].facts).contains_exactly([Fact::index(0)]);
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
            assert_that!(failures[0].children[0].facts).contains_exactly([Fact::index(1)]);

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
                    let facts = failure.derive(|failure| &failure.facts);
                    if unordered {
                        facts.contains_exactly_satisfying(
                            [|fact: AssertThat<Fact, Capture>| {
                                fact.derive_owned(|fact| items(&fact.value)).is_empty();
                            }; 2],
                        );
                    } else {
                        facts.contains_exactly([Fact::note("... 1 more unmatched element ...")]);
                    }
                }
            }));
        }
    }

    mod evaluation {
        use super::*;
        use core::cell::Cell;

        #[test]
        fn membership_scans_bound_retention_and_preserve_omission_counts() {
            let targets = [0_u32; 4096];
            let missing = 1_u32;
            for limit in [0, 1, 3] {
                let context = AssertionContext::new(
                    &NoRenderer,
                    RenderingBudget::default().with_max_items(limit),
                );
                let matcher = ContainsSameInstanceAs::new(&missing);
                let rejection = matcher.evaluate(&targets, &context).err().unwrap();
                assert_that!(rejection.observed.targets).has_length(limit);
                assert_that!(rejection.observed.inspected).is_equal_to(targets.len());
                let failure = matcher
                    .explain(
                        Some((&targets, rejection)),
                        FailureBuilder::detached::<[u32; 4096]>(FailureKind::Membership),
                        &context,
                    )
                    .build();
                let RenderedBody::Group { items, omitted, .. } =
                    &failure.actual.as_ref().unwrap().body
                else {
                    panic!("expected identity evidence")
                };
                assert_that!(items).has_length(limit);
                assert_that!(*omitted).is_equal_to(targets.len() - limit);

                let matcher = DoesNotContainSameInstanceAs::new(&targets[4095]);
                let rejection = matcher.evaluate(&targets, &context).err().unwrap();
                assert_that!(rejection.observed.targets).has_length(limit);
                assert_that!(rejection.observed.inspected).is_equal_to(targets.len());

                let expected = targets.iter().rev().collect::<Vec<_>>();
                let matcher = ContainsExactlySameInstances::new(expected);
                let rejection = matcher.evaluate(&targets, &context).err().unwrap();
                assert_that!(rejection.observed.targets.len()).is_less_or_equal_to(limit);
            }
        }

        #[test]
        fn probes_retain_no_identity_targets() {
            let targets = [0_u32; 4096];
            let missing = 1_u32;
            let context = AssertionContext::new(&NoRenderer, RenderingBudget::unlimited())
                .with_diagnostics(false);
            let matcher = ContainsSameInstanceAs::new(&missing);
            let rejection = matcher.evaluate(&targets, &context).err().unwrap();
            assert_that!(rejection.observed.targets).is_empty();
            let matcher = DoesNotContainSameInstanceAs::new(&targets[4095]);
            let rejection = matcher.evaluate(&targets, &context).err().unwrap();
            assert_that!(rejection.observed.targets).is_empty();
            let expected = targets.iter().rev().collect::<Vec<_>>();
            let matcher = ContainsExactlySameInstances::new(expected);
            let rejection = matcher.evaluate(&targets, &context).err().unwrap();
            assert_that!(rejection.observed.targets).is_empty();
        }

        #[test]
        fn sorted_retention_matches_full_rendering_before_truncation() {
            let missing = 99_i32;
            let actual = UnorderedSet((0..32).rev().collect());
            for limit in [0, 1, 3] {
                for leaf_limit in [0, 4, usize::MAX] {
                    let context = AssertionContext::new(
                        &NoRenderer,
                        RenderingBudget::default()
                            .with_max_items(limit)
                            .with_max_leaf_characters(leaf_limit),
                    );
                    let matcher = ContainsSameInstanceAs::new(&missing);
                    let rejection = matcher.evaluate(&actual, &context).err().unwrap();
                    assert_that!(rejection.observed.targets).has_length(limit);
                    assert_that!(rejection.observed.sort_keys).has_length(limit);
                    let expected = context
                        .render()
                        .identities()
                        .borrowed_collection::<i32, _>(&actual)
                        .into_rendered();
                    assert_that!(rejection.observed.render(&actual, &context))
                        .is_equal_to(expected);
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
