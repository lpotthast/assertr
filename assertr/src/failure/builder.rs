//! The builder every assertion constructs its failure with.
//!
//! The builder collects the rendered values, the relation, the facts, and the children of one
//! failure and turns them into an [`AssertionFailure`]. Its text is derived from those fields by
//! the failure's `Display` implementation, so assertion code never formats a failure body by hand
//! and the grammar of every failure comes from one place.

use alloc::{borrow::Cow, boxed::Box, vec::Vec};

use super::{AssertionFailure, Fact, FailureKind, PathSegment};
use crate::{expectation::Evidence, renderer::Rendered};

/// Builds one [`AssertionFailure`].
///
/// Leaf assertions receive a builder in
/// [`Expectation::explain`](crate::expectation::Expectation::explain), fill in the rendered values,
/// the relation, facts, and children, and return it. The chain executor raises it. Execution
/// adapters, which own an operation the expectation protocol cannot express, start a builder with
/// [`AssertThat::failure`](crate::AssertThat::failure) and pass it
/// to [`AssertThat::raise`](crate::AssertThat::raise). Nested failures are completed with
/// [`build`](Self::build) and attached to their parent through [`child`](Self::child) or
/// [`children`](Self::children).
///
/// Every value shown by a failure is rendered through the context obtained from
/// [`AssertionContext::render`](crate::expectation::AssertionContext::render) or
/// [`AssertThat::render`](crate::AssertThat::render), so the chain's
/// [`ValueRenderer`](crate::renderer::ValueRenderer) and
/// [`RenderingBudget`](crate::renderer::RenderingBudget) apply. See [custom
/// assertions](crate#custom-assertions) for a complete example.
#[must_use = "a failure is only recorded by `AssertThat::raise` or returned by `build`"]
pub struct FailureBuilder {
    failure: AssertionFailure,
}

impl FailureBuilder {
    /// Starts a failure of the given kind over a subject of type `T`.
    ///
    /// Locate a nested failure within its parent's subject with [`Self::path`].
    pub fn new<T: ?Sized>(kind: FailureKind) -> Self {
        Self {
            failure: AssertionFailure {
                constraint: None,
                path: Vec::new(),
                omitted_children: 0,
                location: None,
                subject_name: None,
                expression: None,
                subject_type_name: core::any::type_name::<T>(),
                kind,
                actual: None,
                relation: None,
                expected: None,
                unexpected: None,
                facts: Vec::new(),
                messages: Vec::new(),
                children: Vec::new(),
            },
        }
    }

    /// Finishes the failure.
    #[must_use]
    pub fn build(self) -> AssertionFailure {
        self.failure
    }

    /// Attaches the diagnostic for an unmet expectation with no subject.
    pub fn constraint(mut self, description: AssertionFailure) -> Self {
        self.failure.constraint = Some(Box::new(description));
        self
    }

    /// Appends segments to the relative typed path of this failure.
    pub fn path(mut self, path: impl IntoIterator<Item = PathSegment>) -> Self {
        self.failure.path.extend(path);
        self
    }

    /// Adds `count` to the number of diagnostic children omitted by the rendering budget.
    ///
    /// Like [`children`](Self::children), this accumulates: each call adds to the count recorded
    /// so far, so several evidence sources can each report the children they left out.
    pub fn omitted_children(mut self, count: usize) -> Self {
        self.failure.omitted_children += count;
        self
    }

    /// Replaces the recorded subject type, for example with a projected field's type.
    pub(crate) fn subject_type<U: ?Sized>(mut self) -> Self {
        self.failure.subject_type_name = core::any::type_name::<U>();
        self
    }

    pub(crate) fn kind(mut self, kind: FailureKind) -> Self {
        self.failure.kind = kind;
        self
    }

    /// Sets the rendered subject, obtained from a rendering context.
    pub fn actual(mut self, actual: impl Into<Rendered>) -> Self {
        self.failure.actual = Some(actual.into());
        self
    }

    /// Sets the sentence between the actual and the expected value, such as `does not contain`.
    ///
    /// A failure without a relation is a direct comparison and renders as an aligned `Expected:` /
    /// `Actual:` pair. A relation never embeds a value: values belong to
    /// [`expected`](Self::expected), [`unexpected`](Self::unexpected), or a [`fact`](Self::fact).
    pub fn relation(mut self, relation: impl Into<Cow<'static, str>>) -> Self {
        self.failure.relation = Some(relation.into());
        self
    }

    /// Sets the relation of an expectation and, for a rejection, its rendered subject.
    ///
    /// This covers the common shape of
    /// [`Expectation::explain`](crate::expectation::Expectation::explain). Without a rejected
    /// subject it states `relation`, such as `has a name`. With one, it shows the subject and
    /// states `negated`, such as `has an empty name`.
    ///
    /// ```
    /// # use assertr::{expectation::AssertionContext, renderer::ValueRenderer, failure::FailureBuilder};
    /// # struct Person { name: String }
    /// # fn explain<R: ValueRenderer<str>>(
    /// #     rejected: Option<(&Person, ())>,
    /// #     failure: FailureBuilder,
    /// #     context: &AssertionContext<'_, R>,
    /// # ) -> FailureBuilder {
    /// failure.relations(
    ///     rejected.map(|(person, ())| context.render().value(person.name.as_str())),
    ///     "has a name",
    ///     "has an empty name",
    /// )
    /// # }
    /// ```
    pub fn relations(
        self,
        rejected: Option<Rendered>,
        relation: impl Into<Cow<'static, str>>,
        negated: impl Into<Cow<'static, str>>,
    ) -> Self {
        match rejected {
            None => self.relation(relation),
            Some(actual) => self.actual(actual).relation(negated),
        }
    }

    /// Sets the rendered value the subject was compared with.
    pub fn expected(mut self, expected: impl Into<Rendered>) -> Self {
        self.failure.expected = Some(expected.into());
        self
    }

    /// Sets the rendered value a negated assertion found although it was not expected.
    pub fn unexpected(mut self, unexpected: impl Into<Rendered>) -> Self {
        self.failure.unexpected = Some(unexpected.into());
        self
    }

    /// Attaches one fact, preserving its rendered evidence and type metadata.
    ///
    /// Construct it with [`Fact::labelled`] or [`Fact::note`], passing diagnostic values through
    /// a rendering context. Facts are already rendered and are not rendered again here.
    pub fn fact(mut self, fact: Fact) -> Self {
        self.failure.facts.push(fact);
        self
    }

    /// Appends facts in iteration order, after any facts already attached.
    ///
    /// Labelled facts and notes may be mixed. Their rendered evidence is preserved unchanged.
    pub fn facts(mut self, facts: impl IntoIterator<Item = Fact>) -> Self {
        self.failure.facts.extend(facts);
        self
    }

    /// Attaches one nested failure.
    pub fn child(mut self, child: AssertionFailure) -> Self {
        self.failure.children.push(child);
        self
    }

    /// Attaches nested failures in the given order.
    pub fn children(mut self, children: impl IntoIterator<Item = AssertionFailure>) -> Self {
        self.failure.children.extend(children);
        self
    }

    /// Attaches the failures collected by child expectations, together with the number the
    /// rendering budget left out.
    ///
    /// Collect the evidence with [`AssertionContext`](crate::expectation::AssertionContext) during
    /// [`Expectation::evaluate`](crate::expectation::Expectation::evaluate), return it as the
    /// rejection, and attach it here in
    /// [`Expectation::explain`](crate::expectation::Expectation::explain).
    /// Like [`children`](Self::children) and [`omitted_children`](Self::omitted_children), this
    /// accumulates.
    pub fn evidence(self, evidence: Evidence) -> Self {
        self.children(evidence.children)
            .omitted_children(evidence.omitted)
    }
}

#[cfg(test)]
mod tests {
    use core::{cell::Cell, fmt};

    use indoc::indoc;

    use super::FailureBuilder;
    use crate::{
        failure::{Fact, FailureKind},
        prelude::*,
        renderer::{Rendered, RenderedBody, RenderingContext},
    };

    struct Evidence;

    struct EvidenceRenderer<'a>(&'a Cell<usize>);

    impl ValueRenderer<Evidence> for EvidenceRenderer<'_> {
        fn fmt(&self, _: &Evidence, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            formatter.write_str("evidence")
        }
    }

    mod fact {
        use super::*;

        /// Attaches one fact rendered from `Evidence` and checks that it keeps its evidence without
        /// rendering it again.
        fn assert_preserved(fact: impl FnOnce(Rendered) -> Fact, detail: &str) -> AssertionFailure {
            let renders = Cell::new(0);
            let renderer = EvidenceRenderer(&renders);
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(3),
            );
            let fact = fact(rendering.value(&Evidence));
            assert_that!(renders.get()).is_equal_to(1);

            let failure = FailureBuilder::new::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(fact)
                .build();

            assert_that!(&failure).has_text_report(format!(
                "-------- assertr --------\ndoes not hold\n\nDetails:\n  - {detail}\n-------- assertr --------\n"
            ));
            assert_that!(failure.facts[0].value.type_name)
                .is_equal_to(Some(core::any::type_name::<Evidence>()));
            assert_that!(failure.facts[0].value.body).is_equal_to(RenderedBody::Text {
                text: "evi".into(),
                omitted_characters: 5,
            });
            assert_that!(renders.get()).is_equal_to(1);
            failure
        }

        #[test]
        fn preserves_labelled_evidence_without_rendering_again() {
            let failure = assert_preserved(
                |value| Fact::labelled("Reason", value),
                "Reason: evi... 5 more characters ...",
            );
            assert_that!(failure.facts[0].label.as_deref()).is_equal_to(Some("Reason"));
        }

        #[test]
        fn preserves_note_evidence_without_rendering_again() {
            let failure = assert_preserved(Fact::note, "evi... 5 more characters ...");
            assert_that!(failure.facts[0].label).is_none();
        }
    }

    mod relations {
        use super::*;

        #[test]
        fn states_the_relation_without_a_subject_and_the_negation_with_one() {
            let rendering = RenderingContext::new(&DebugRenderer, RenderingBudget::default());
            let described = FailureBuilder::new::<i32>(FailureKind::Predicate)
                .relations(None, "is even", "is odd")
                .build();
            assert_that!(described.relation.as_deref()).is_equal_to(Some("is even"));
            assert_that!(described.actual).is_none();

            let rejected = FailureBuilder::new::<i32>(FailureKind::Predicate)
                .relations(Some(rendering.value(&3)), "is even", "is odd")
                .build();
            assert_that!(rejected.relation.as_deref()).is_equal_to(Some("is odd"));
            assert_that!(rejected.actual.map(|actual| actual.to_string()))
                .is_equal_to(Some("3".into()));
        }
    }

    mod omitted_children {
        use super::*;

        #[test]
        fn accumulates_across_calls() {
            let failure = FailureBuilder::new::<()>(FailureKind::Other)
                .omitted_children(2)
                .omitted_children(0)
                .omitted_children(3)
                .build();

            assert_that!(failure.omitted_children).is_equal_to(5);
        }
    }

    mod facts {
        use super::*;

        #[test]
        fn appends_mixed_arrays_and_iterators_in_order_without_rendering_again() {
            let renders = Cell::new(0);
            let renderer = EvidenceRenderer(&renders);
            let rendering = RenderingContext::new(&renderer, RenderingBudget::default());
            let failure = FailureBuilder::new::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(Fact::note("First."))
                .facts([
                    Fact::labelled("Reason", rendering.value(&Evidence)),
                    Fact::note(rendering.value(&Evidence)),
                ])
                .facts(["Next.", "Last."].into_iter().map(Fact::note))
                .build();

            assert_that!(failure).has_text_report(indoc! {"
                -------- assertr --------
                does not hold

                Details:
                  - First.
                  - Reason: evidence
                  - evidence
                  - Next.
                  - Last.
                -------- assertr --------
            "});
            for fact in &failure.facts[1..3] {
                assert_that!(fact.value.type_name)
                    .is_equal_to(Some(core::any::type_name::<Evidence>()));
            }
            assert_that!(renders.get()).is_equal_to(2);
        }

        #[test]
        fn an_empty_iterator_preserves_preceding_and_following_facts() {
            let failure = FailureBuilder::new::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(Fact::note("First."))
                .facts(core::iter::empty())
                .fact(Fact::labelled("Last", "unchanged"))
                .build();

            assert_that!(failure).has_text_report(indoc! {"
                -------- assertr --------
                does not hold

                Details:
                  - First.
                  - Last: unchanged
                -------- assertr --------
            "});
        }
    }
}
