//! The builder every leaf assertion raises its failure through.
//!
//! The builder collects the rendered values, the relation, the facts, and the children of one
//! failure and turns them into an [`AssertionFailure`]. Its text is derived from those fields by
//! [`ToHumanReadableText`](super::adapter::ToHumanReadableText), so assertion code never formats a
//! failure body by hand and the grammar of every failure comes from one place.

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::panic::Location;

use super::{AssertionFailure, Fact, FailureKind, Fallible, PathSegment};
use crate::{
    AssertThat,
    details::WithDetail,
    mode::Mode,
    renderer::{IntoRendered, Rendered},
};

/// The chain a failure is raised on, seen through the pieces the builder needs from it.
pub(crate) trait FailureSink: Fallible + WithDetail {
    /// Whether failures are collected for later inspection (`true`) or raise an immediate panic
    /// (`false`).
    fn captures(&self) -> bool;

    /// Whether the source location of the assertion should be included in assertion failures. This
    /// pinpoints the location in user's code that failed. Typically turned off for internal assertr
    /// unit tests, to avoid frequent failure message churn.
    fn include_location(&self) -> bool;

    /// User provided descriptive name of the thing assertions are made on.
    fn subject_name(&self) -> Option<String>;

    /// Rust expression written inside an `assert_that!(...)` or fluent `.must(...)` call.
    /// Typically, captured automatically by macro code.
    fn expression(&self) -> Option<&'static str>;

    /// A context-specific adapter that produces panic text.
    fn panic_presentation(&self) -> Option<&super::panic_presentation::PanicPresentation>;
}

impl<T, M: Mode, R> FailureSink for AssertThat<'_, T, M, R> {
    fn captures(&self) -> bool {
        M::CAPTURES
    }

    fn include_location(&self) -> bool {
        self.state.include_location
    }

    fn subject_name(&self) -> Option<String> {
        self.state.subject_name.clone()
    }

    fn expression(&self) -> Option<&'static str> {
        self.state.expression
    }

    fn panic_presentation(&self) -> Option<&super::panic_presentation::PanicPresentation> {
        self.state.panic_presentation.as_deref()
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Where a [`FailureBuilder`] delivers its failure: [`Attached`] to an assertion chain or
/// [`Detached`] as a value. This trait is sealed.
pub trait FailureTarget: sealed::Sealed {}

/// The target of a builder started by [`AssertThat::failure`]: the failure is raised on that chain
/// by [`FailureBuilder::raise`].
pub struct Attached<'c> {
    sink: &'c dyn FailureSink,
    location: &'static Location<'static>,
}

impl sealed::Sealed for Attached<'_> {}
impl FailureTarget for Attached<'_> {}

/// The target of a builder started by [`FailureBuilder::detached`]: the failure is returned by
/// [`FailureBuilder::build`], to become a child of another failure.
pub struct Detached;

impl sealed::Sealed for Detached {}
impl FailureTarget for Detached {}

/// Builds one [`AssertionFailure`].
///
/// Obtain a builder through [`AssertThat::failure`] inside a leaf assertion, fill in the rendered
/// values, the relation, facts, and children, and finish with [`raise`](Self::raise). Every value
/// shown by a failure is passed as an adapter obtained from [`AssertThat::render`], so the chain's
/// [`ValueRenderer`](crate::ValueRenderer) and [`RenderingBudget`](crate::RenderingBudget) apply.
/// See [custom assertions](crate#custom-assertions) for a complete example.
///
/// [`FailureBuilder::detached`] starts a failure that is not raised but returned by
/// [`build`](Self::build), for the nested failures a parent attaches through [`child`](Self::child)
/// and [`children`](Self::children).
#[must_use = "a failure is only recorded by `raise` or `build`"]
pub struct FailureBuilder<T: FailureTarget> {
    path: Vec<PathSegment>,
    omitted_children: usize,
    constraint: Option<crate::matchers::ConstraintDescription>,
    target: T,
    subject_type_name: &'static str,
    kind: FailureKind,
    actual: Option<Rendered>,
    relation: Option<Cow<'static, str>>,
    expected: Option<Rendered>,
    unexpected: Option<Rendered>,
    facts: Vec<Fact>,
    children: Vec<AssertionFailure>,
}

impl<'c> FailureBuilder<Attached<'c>> {
    pub(crate) fn attached(
        sink: &'c dyn FailureSink,
        subject_type_name: &'static str,
        location: &'static Location<'static>,
        kind: FailureKind,
    ) -> Self {
        Self::new(Attached { sink, location }, subject_type_name, kind)
    }

    /// Records the failure in capture mode and panics with its rendered form otherwise.
    ///
    /// # Panics
    ///
    /// Panics with the formatted failure message when not in capture mode.
    #[track_caller]
    pub fn raise(self) {
        let Attached { sink, location } = self.target;
        let location = if sink.include_location() {
            Some(location)
        } else {
            None
        };
        let mut messages = Vec::new();
        sink.collect_messages(&mut messages);

        let failure = self.into_failure(location, sink.subject_name(), sink.expression(), messages);

        if sink.captures() {
            sink.store_failure(failure);
        } else {
            let text = super::panic_presentation::render(&failure, sink.panic_presentation());
            panic!("{text}");
        }
    }
}

impl FailureBuilder<Detached> {
    /// Starts a failure over a subject of type `T` that is not raised on a chain but returned by
    /// [`build`](Self::build), to be attached to another failure as a child.
    ///
    /// Locate such a child within its parent's subject with [`AssertionFailure::located_at`].
    pub fn detached<T: ?Sized>(kind: FailureKind) -> Self {
        Self::new(Detached, core::any::type_name::<T>(), kind)
    }

    /// Finishes the failure.
    #[must_use]
    pub fn build(self) -> AssertionFailure {
        self.into_failure(None, None, None, Vec::new())
    }
}

impl<T: FailureTarget> FailureBuilder<T> {
    fn new(target: T, subject_type_name: &'static str, kind: FailureKind) -> Self {
        Self {
            path: Vec::new(),
            omitted_children: 0,
            constraint: None,
            target,
            subject_type_name,
            kind,
            actual: None,
            relation: None,
            expected: None,
            unexpected: None,
            facts: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Attaches an independently described matcher constraint.
    pub fn constraint(mut self, description: crate::matchers::ConstraintDescription) -> Self {
        self.constraint = Some(description);
        self
    }

    /// Sets the relative typed path of this failure.
    pub fn path(mut self, path: impl IntoIterator<Item = PathSegment>) -> Self {
        self.path.extend(path);
        self
    }

    /// Records the number of omitted children without losing the truth result.
    pub fn omitted_children(mut self, count: usize) -> Self {
        self.omitted_children = count;
        self
    }

    /// Sets the rendered subject. Pass an adapter obtained from [`AssertThat::render`]. It is
    /// consumed into an owned value tree here, with every leaf rendered exactly once.
    pub fn actual(mut self, actual: impl IntoRendered) -> Self {
        self.actual = Some(actual.into_rendered());
        self
    }

    /// Sets the sentence between the actual and the expected value, such as `does not contain`.
    ///
    /// A failure without a relation is a direct comparison and renders as an aligned `Expected:` /
    /// `Actual:` pair. A relation never embeds a value: values belong to
    /// [`expected`](Self::expected), [`unexpected`](Self::unexpected), or a [`fact`](Self::fact).
    pub fn relation(mut self, relation: impl Into<Cow<'static, str>>) -> Self {
        self.relation = Some(relation.into());
        self
    }

    /// Sets the rendered value the subject was compared with.
    pub fn expected(mut self, expected: impl IntoRendered) -> Self {
        self.expected = Some(expected.into_rendered());
        self
    }

    /// Sets the rendered value a negated assertion found although it was not expected.
    pub fn unexpected(mut self, unexpected: impl IntoRendered) -> Self {
        self.unexpected = Some(unexpected.into_rendered());
        self
    }

    /// Attaches one fact, preserving its rendered evidence and type metadata.
    ///
    /// Construct it with [`Fact::labelled`] or [`Fact::note`], passing diagnostic values through
    /// [`AssertThat::render`]. Facts are already rendered and are not rendered again here.
    pub fn fact(mut self, fact: Fact) -> Self {
        self.facts.push(fact);
        self
    }

    /// Appends facts in iteration order, after any facts already attached.
    ///
    /// Labeled facts and notes may be mixed. Their rendered evidence is preserved unchanged.
    pub fn facts(mut self, facts: impl IntoIterator<Item = Fact>) -> Self {
        self.facts.extend(facts);
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
        self.children.push(child);
        self
    }

    /// Attaches nested failures in the given order.
    pub fn children(mut self, children: impl IntoIterator<Item = AssertionFailure>) -> Self {
        self.children.extend(children);
        self
    }

    fn into_failure(
        self,
        location: Option<&'static Location<'static>>,
        subject_name: Option<String>,
        expression: Option<&'static str>,
        messages: Vec<String>,
    ) -> AssertionFailure {
        AssertionFailure {
            constraint: self.constraint,
            path: self.path,
            omitted_children: self.omitted_children,
            location,
            subject_name,
            expression,
            subject_type_name: self.subject_type_name,
            actual: self.actual,
            relation: self.relation,
            expected: self.expected,
            unexpected: self.unexpected,
            facts: self.facts,
            messages,
            children: self.children,
            kind: self.kind,
        }
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
                RenderingBudget::builder().max_leaf_characters(3).build(),
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
                RenderingBudget::builder().max_leaf_characters(3).build(),
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
