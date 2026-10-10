//! Reusable set relations and their rejection evidence.

use alloc::vec::Vec;

use super::SetLookup;
use crate::{
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::ValueRenderer,
};

/// One set relation: which elements to inspect, which membership rejects them, and how to
/// describe the relation and its offending elements.
struct Relation {
    /// Inspect the expected set's elements instead of the subject's.
    expected_elements: bool,
    /// Reject an inspected element when the other set contains it, instead of when it does not.
    reject_members: bool,
    holds: &'static str,
    fails: &'static str,
    offending: &'static str,
}

const SUBSET: Relation = Relation {
    expected_elements: false,
    reject_members: false,
    holds: "is a subset of",
    fails: "is not a subset of",
    offending: "Elements not in expected",
};

const SUPERSET: Relation = Relation {
    expected_elements: true,
    reject_members: false,
    holds: "is a superset of",
    fails: "is not a superset of",
    offending: "Elements not in actual",
};

const DISJOINT: Relation = Relation {
    expected_elements: false,
    reject_members: true,
    holds: "is disjoint from",
    fails: "is not disjoint from",
    offending: "Overlapping elements",
};

impl Relation {
    /// Collects the inspected elements that violate the relation.
    fn evaluate<'a, S, O>(&self, actual: &'a S, expected: &'a O) -> Result<(), Vec<&'a S::Item>>
    where
        S: SetLookup + ?Sized,
        O: SetLookup<Item = S::Item>,
    {
        let offending: Vec<_> = if self.expected_elements {
            expected
                .elements()
                .filter(|it| actual.contains_element(it) == self.reject_members)
                .collect()
        } else {
            actual
                .elements()
                .filter(|it| expected.contains_element(it) == self.reject_members)
                .collect()
        };
        if offending.is_empty() {
            Ok(())
        } else {
            Err(offending)
        }
    }

    fn explain<S, O, R>(
        &self,
        expected: &O,
        rejected: Option<(&S, Vec<&S::Item>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder
    where
        S: SetLookup + ?Sized,
        O: SetLookup<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation(self.holds),
            Some((actual, offending)) => {
                // Offending elements keep the presentation order of the set they come from.
                let order = if self.expected_elements {
                    O::PRESENTATION.order()
                } else {
                    S::PRESENTATION.order()
                };
                failure
                    .actual(render.collection(actual))
                    .relation(self.fails)
                    .fact(Fact::labelled(
                        self.offending,
                        render.borrowed_values::<S::Item, _>(&offending, order),
                    ))
            }
        };
        failure.expected(render.collection(expected))
    }
}

/// Implements a public set relation expectation by delegating to its [`Relation`].
macro_rules! set_relation {
    ($(#[$attr:meta])* $name:ident, $relation:ident) => {
        $(#[$attr])*
        #[derive(Debug, Clone)]
        pub struct $name<O>(O);

        impl<O> $name<O> {
            /// Owns the expected operand.
            #[must_use]
            pub const fn new(expected: O) -> Self {
                Self(expected)
            }
        }

        impl<S: SetLookup + ?Sized, O, R> Expectation<S, R> for $name<O>
        where
            O: SetLookup<Item = S::Item>,
            R: ValueRenderer<S::Item>,
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
                $relation.evaluate(actual, &self.0)
            }

            const KIND: FailureKind = FailureKind::Membership;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a S, Self::Rejection<'a>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                $relation.explain(&self.0, rejected, failure, context)
            }
        }
    };
}

set_relation!(
    /// Checks that a set is a subset of another set using native lookup.
    IsSubsetOf,
    SUBSET
);
set_relation!(
    /// Checks that a set is a superset of another set using native lookup.
    IsSupersetOf,
    SUPERSET
);
set_relation!(
    /// Checks that a set is disjoint from another set using native lookup.
    IsDisjointFrom,
    DISJOINT
);
