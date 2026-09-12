//! Reusable set relations and their rejection evidence.

use super::SetLookup;
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::GroupStyle,
};
use alloc::{string::String, vec::Vec};

fn type_difference_detail<S, O>() -> Option<String>
where
    S: SetLookup + ?Sized,
    O: SetLookup,
{
    if set_type_name::<S>() == set_type_name::<O>() {
        None
    } else {
        Some(String::from(
            "The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.",
        ))
    }
}

fn set_type_name<S: ?Sized>() -> &'static str {
    let mut name = core::any::type_name::<S>();
    while let Some(unreferenced) = name.strip_prefix('&') {
        name = unreferenced.strip_prefix("mut ").unwrap_or(unreferenced);
    }
    name
}

/// Checks that a set is a subset of another set using native lookup.
pub struct IsSubsetOf<O>(O);

impl<O> IsSubsetOf<O> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: O) -> Self {
        Self(expected)
    }
}

impl<S: SetLookup + ?Sized, O, R> Expectation<S, R> for IsSubsetOf<O>
where
    O: SetLookup<Item = S::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a S::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let elements = actual
            .elements()
            .filter(|it| !self.0.contains_element(it))
            .collect::<Vec<_>>();
        if elements.is_empty() {
            Ok(())
        } else {
            Err(elements)
        }
    }
}

impl<S: SetLookup + ?Sized, O, R> ExpectationDiagnostics<S, R> for IsSubsetOf<O>
where
    O: SetLookup<Item = S::Item>,
    R: ValueRenderer<S::Item>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is a subset of"),
            Some((actual, elements)) => failure
                .actual(render.collection(actual))
                .relation("is not a subset of")
                .fact(Fact::labelled(
                    "Elements not in expected",
                    render
                        .borrowed_values::<S::Item, _>(&elements, GroupStyle::List)
                        .with_order(S::PRESENTATION.order()),
                ))
                .facts(type_difference_detail::<S, O>().map(Fact::note)),
        };
        failure.expected(render.collection(&self.0))
    }
}

/// Checks that a set is a superset of another set using native lookup.
pub struct IsSupersetOf<O>(O);

impl<O> IsSupersetOf<O> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: O) -> Self {
        Self(expected)
    }
}

impl<S: SetLookup + ?Sized, O, R> Expectation<S, R> for IsSupersetOf<O>
where
    O: SetLookup<Item = S::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a S::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let elements = self
            .0
            .elements()
            .filter(|it| !actual.contains_element(it))
            .collect::<Vec<_>>();
        if elements.is_empty() {
            Ok(())
        } else {
            Err(elements)
        }
    }
}

impl<S: SetLookup + ?Sized, O, R> ExpectationDiagnostics<S, R> for IsSupersetOf<O>
where
    O: SetLookup<Item = S::Item>,
    R: ValueRenderer<S::Item>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is a superset of"),
            Some((actual, elements)) => failure
                .actual(render.collection(actual))
                .relation("is not a superset of")
                .fact(Fact::labelled(
                    "Elements not in actual",
                    render
                        .borrowed_values::<S::Item, _>(&elements, GroupStyle::List)
                        .with_order(O::PRESENTATION.order()),
                ))
                .facts(type_difference_detail::<S, O>().map(Fact::note)),
        };
        failure.expected(render.collection(&self.0))
    }
}

/// Checks that a set is disjoint from another set using native lookup.
pub struct IsDisjointFrom<O>(O);

impl<O> IsDisjointFrom<O> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: O) -> Self {
        Self(expected)
    }
}

impl<S: SetLookup + ?Sized, O, R> Expectation<S, R> for IsDisjointFrom<O>
where
    O: SetLookup<Item = S::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a S::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let elements = actual
            .elements()
            .filter(|it| self.0.contains_element(it))
            .collect::<Vec<_>>();
        if elements.is_empty() {
            Ok(())
        } else {
            Err(elements)
        }
    }
}

impl<S: SetLookup + ?Sized, O, R> ExpectationDiagnostics<S, R> for IsDisjointFrom<O>
where
    O: SetLookup<Item = S::Item>,
    R: ValueRenderer<S::Item>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is disjoint from"),
            Some((actual, elements)) => failure
                .actual(render.collection(actual))
                .relation("is not disjoint from")
                .fact(Fact::labelled(
                    "Overlapping elements",
                    render
                        .borrowed_values::<S::Item, _>(&elements, GroupStyle::List)
                        .with_order(S::PRESENTATION.order()),
                ))
                .facts(type_difference_detail::<S, O>().map(Fact::note)),
        };
        failure.expected(render.collection(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use crate::{prelude::*, test_support::UnorderedSet};

    use super::type_difference_detail;

    #[test]
    fn type_difference_compares_rust_types() {
        assert_that!(type_difference_detail::<BTreeSet<i32>, BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, &BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, &&BTreeSet<i32>>()).is_none();
        assert_that!(type_difference_detail::<BTreeSet<i32>, UnorderedSet>()).is_some();
    }
}
