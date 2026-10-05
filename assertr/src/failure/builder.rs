//! The builder every leaf assertion raises its failure through.
//!
//! The builder collects the rendered values, the relation, the facts, and the children of one
//! failure and turns them into an [`AssertionFailure`]. Its text is derived from those fields by
//! [`ToHumanReadableText`](super::adapter::ToHumanReadableText), so assertion code never formats a
//! failure body by hand and the grammar of every failure comes from one place.

use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use core::panic::Location;

use super::{
    AssertionFailure, Fact, FailureKind, PathSegment, panic_presentation::PanicPresentation,
};
use crate::{
    AssertThat, ChainRecords, Expression,
    mode::Mode,
    renderer::{IntoRendered, Rendered},
};

/// The target of a builder started by [`AssertThat::failure`]: the failure is raised on that chain
/// by [`FailureBuilder::raise`].
pub struct Attached<'c> {
    /// The records of the chain the failure is raised on. Failures are stored on their root.
    records: &'c ChainRecords<'c>,
    /// Whether failures are collected for later inspection (`true`) or raise an immediate panic.
    captures: bool,
    /// Whether the failure records the caller location.
    include_location: bool,
    /// The user-provided name of the subject.
    subject_name: Option<&'c str>,
    /// The source expression of the subject, possibly still pending fluent attachment.
    expression: Expression,
    /// A context-specific adapter that produces panic text.
    panic_presentation: Option<&'c PanicPresentation>,
    /// Where the failing assertion was invoked.
    location: &'static Location<'static>,
}

impl<'c> Attached<'c> {
    fn new<T, M: Mode, R>(
        assertion: &'c AssertThat<'_, T, M, R>,
        location: &'static Location<'static>,
    ) -> Self {
        let state = &assertion.state;
        Self {
            records: &state.records,
            captures: M::CAPTURES,
            include_location: state.include_location,
            subject_name: state.subject_name.as_deref(),
            expression: state.expression,
            panic_presentation: state.panic_presentation.as_deref(),
            location,
        }
    }
}

/// The target of a builder started by [`FailureBuilder::detached`]: the failure is returned by
/// [`FailureBuilder::build`], to become a child of another failure.
pub struct Detached;

/// Builds one [`AssertionFailure`].
///
/// Leaf assertions receive a builder in
/// [`ExpectationDiagnostics::explain`](crate::ExpectationDiagnostics::explain), fill in the
/// rendered values, the relation, facts, and children, and return it. The chain executor raises
/// it. Execution adapters, which own an operation the expectation protocol cannot express, obtain
/// a builder through [`AssertThat::failure`] instead and finish with [`raise`](Self::raise).
///
/// Every value shown by a failure is passed as an adapter obtained from
/// [`AssertionContext::render`](crate::AssertionContext::render) or [`AssertThat::render`], so the
/// chain's [`ValueRenderer`](crate::ValueRenderer) and [`RenderingBudget`](crate::RenderingBudget)
/// apply. See [custom assertions](crate#custom-assertions) for a complete example.
///
/// [`FailureBuilder::detached`] starts a failure that is not raised but returned by
/// [`build`](Self::build), for the nested failures a parent attaches through [`child`](Self::child)
/// and [`children`](Self::children).
#[must_use = "a failure is only recorded by `raise` or `build`"]
pub struct FailureBuilder<T> {
    target: T,
    failure: AssertionFailure,
}

impl<'c> FailureBuilder<Attached<'c>> {
    pub(crate) fn attached<T, M: Mode, R>(
        assertion: &'c AssertThat<'_, T, M, R>,
        location: &'static Location<'static>,
        kind: FailureKind,
    ) -> Self {
        Self::new(
            Attached::new(assertion, location),
            core::any::type_name::<T>(),
            kind,
        )
    }

    /// Records the failure in capture mode and panics with its rendered form otherwise.
    ///
    /// # Panics
    ///
    /// Panics with the formatted failure message when not in capture mode.
    #[track_caller]
    pub fn raise(self) {
        let Attached {
            records,
            captures,
            include_location,
            subject_name,
            expression,
            panic_presentation,
            location,
        } = self.target;
        let location = include_location.then_some(location);
        let mut messages = Vec::new();
        records.collect_messages(&mut messages);

        let failure = self.into_failure(
            location,
            subject_name.map(String::from),
            expression.get(),
            messages,
        );

        if captures {
            records.store_failure(
                failure,
                #[cfg(feature = "fluent")]
                expression.pending_fluent(),
            );
        } else {
            let text = super::panic_presentation::render(&failure, panic_presentation);
            panic!("{text}");
        }
    }
}

impl FailureBuilder<Detached> {
    /// Starts a failure over a subject of type `T` that is not raised on a chain but returned by
    /// [`build`](Self::build), to be attached to another failure as a child.
    ///
    /// Locate such a child within its parent's subject with [`Self::path`].
    pub fn detached<T: ?Sized>(kind: FailureKind) -> Self {
        Self::new(Detached, core::any::type_name::<T>(), kind)
    }

    /// Finishes the failure.
    #[must_use]
    pub fn build(self) -> AssertionFailure {
        self.into_failure(None, None, None, Vec::new())
    }
}

impl<T> FailureBuilder<T> {
    fn new(target: T, subject_type_name: &'static str, kind: FailureKind) -> Self {
        Self {
            target,
            failure: AssertionFailure {
                constraint: None,
                path: Vec::new(),
                omitted_children: 0,
                location: None,
                subject_name: None,
                expression: None,
                subject_type_name,
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

    /// Preserves an execution adapter's subject type when it validates before mapping.
    pub(crate) fn subject_type<U: ?Sized>(mut self) -> Self {
        self.failure.subject_type_name = core::any::type_name::<U>();
        self
    }

    pub(crate) fn kind(mut self, kind: FailureKind) -> Self {
        self.failure.kind = kind;
        self
    }

    /// Sets the rendered subject. Pass an adapter obtained from [`AssertThat::render`]. It is
    /// consumed into an owned value tree here, with every leaf rendered exactly once.
    pub fn actual(mut self, actual: impl IntoRendered) -> Self {
        self.failure.actual = Some(actual.into_rendered());
        self
    }

    /// Supplies a default subject only when an execution adapter has not already rendered it.
    pub(crate) fn actual_or_else(mut self, actual: impl FnOnce() -> Rendered) -> Self {
        if self.failure.actual.is_none() {
            self.failure.actual = Some(actual());
        }
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

    /// Sets the rendered value the subject was compared with.
    pub fn expected(mut self, expected: impl IntoRendered) -> Self {
        self.failure.expected = Some(expected.into_rendered());
        self
    }

    /// Sets the rendered value a negated assertion found although it was not expected.
    pub fn unexpected(mut self, unexpected: impl IntoRendered) -> Self {
        self.failure.unexpected = Some(unexpected.into_rendered());
        self
    }

    /// Attaches one fact, preserving its rendered evidence and type metadata.
    ///
    /// Construct it with [`Fact::labelled`] or [`Fact::note`], passing diagnostic values through
    /// [`AssertThat::render`]. Facts are already rendered and are not rendered again here.
    pub fn fact(mut self, fact: Fact) -> Self {
        self.failure.facts.push(fact);
        self
    }

    /// Appends facts in iteration order, after any facts already attached.
    ///
    /// Labeled facts and notes may be mixed. Their rendered evidence is preserved unchanged.
    pub fn facts(mut self, facts: impl IntoIterator<Item = Fact>) -> Self {
        self.failure.facts.extend(facts);
        self
    }

    /// Attaches a note stating how many `noun`s the rendering budget left out of the facts or
    /// children, when `omitted` is nonzero. `noun` is singular and pluralized as needed.
    pub fn omitted(self, omitted: usize, noun: &str) -> Self {
        if omitted == 0 {
            self
        } else {
            self.fact(Fact::note(crate::renderer::omission(omitted, noun)))
        }
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

    fn into_failure(
        mut self,
        location: Option<&'static Location<'static>>,
        subject_name: Option<String>,
        expression: Option<&'static str>,
        messages: Vec<String>,
    ) -> AssertionFailure {
        self.failure.location = location;
        self.failure.subject_name = subject_name;
        self.failure.expression = expression;
        self.failure.messages = messages;
        self.failure
    }
}

#[cfg(test)]
mod tests {
    use core::{cell::Cell, fmt};
    use indoc::formatdoc;

    use super::FailureBuilder;
    use crate::{
        Fact, FailureKind,
        prelude::*,
        renderer::{RenderedBody, RenderingContext},
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

        #[test]
        fn preserves_labelled_evidence_without_rendering_again() {
            let renders = Cell::new(0);
            let renderer = EvidenceRenderer(&renders);
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(3),
            );
            let fact = Fact::labelled("Reason", rendering.value(&Evidence));
            assert_that!(renders.get()).is_equal_to(1);

            let failure = FailureBuilder::detached::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(fact)
                .build();

            assert_that!(failure).has_text_report(formatdoc! {"
                -------- assertr --------
                does not hold

                Details:
                  - Reason: evi... 5 more characters ...
                -------- assertr --------
            "});
            assert_that!(failure.facts[0].value.type_name)
                .is_equal_to(Some(core::any::type_name::<Evidence>()));
            assert_that!(failure.facts[0].value.body).is_equal_to(RenderedBody::Text {
                text: "evi".into(),
                omitted_characters: 5,
            });
            assert_that!(renders.get()).is_equal_to(1);
        }

        #[test]
        fn preserves_note_evidence_without_rendering_again() {
            let renders = Cell::new(0);
            let renderer = EvidenceRenderer(&renders);
            let rendering = RenderingContext::new(
                &renderer,
                RenderingBudget::default().with_max_leaf_characters(3),
            );
            let fact = Fact::note(rendering.value(&Evidence));
            assert_that!(renders.get()).is_equal_to(1);

            let failure = FailureBuilder::detached::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(fact)
                .build();

            assert_that!(failure).has_text_report(formatdoc! {"
                -------- assertr --------
                does not hold

                Details:
                  - evi... 5 more characters ...
                -------- assertr --------
            "});
            assert_that!(failure.facts[0].label.as_ref()).is_empty();
            assert_that!(failure.facts[0].value.type_name)
                .is_equal_to(Some(core::any::type_name::<Evidence>()));
            assert_that!(failure.facts[0].value.body).is_equal_to(RenderedBody::Text {
                text: "evi".into(),
                omitted_characters: 5,
            });
            assert_that!(renders.get()).is_equal_to(1);
        }
    }

    mod omitted_children {
        use super::*;

        #[test]
        fn accumulates_across_calls() {
            let failure = FailureBuilder::detached::<()>(FailureKind::Other)
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
            let failure = FailureBuilder::detached::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(Fact::note("First."))
                .facts([
                    Fact::labelled("Reason", rendering.value(&Evidence)),
                    Fact::note(rendering.value(&Evidence)),
                ])
                .facts(["Next.", "Last."].into_iter().map(Fact::note))
                .build();

            assert_that!(failure).has_text_report(formatdoc! {"
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
            let failure = FailureBuilder::detached::<()>(FailureKind::Other)
                .relation("does not hold")
                .fact(Fact::note("First."))
                .facts(core::iter::empty())
                .fact(Fact::labelled("Last", "unchanged"))
                .build();

            assert_that!(failure).has_text_report(formatdoc! {"
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
