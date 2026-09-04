//! The builder every leaf assertion raises its failure through.
//!
//! The builder collects the rendered values, the relation, the facts, and the children of one
//! failure and turns them into an [`AssertionFailure`]. Its text is derived from those fields by
//! [`TextReporter`](crate::report::TextReporter), so assertion code never formats a failure body
//! by hand and the grammar of every failure comes from one place.

use alloc::{borrow::Cow, format, string::String, vec::Vec};
use core::{fmt::Display, panic::Location};

use super::{AssertionFailure, Fact, FailureKind, Fallible};
use crate::{
    AssertThat,
    details::WithDetail,
    mode::Mode,
    renderer::{IntoRendered, Rendered},
};

/// The chain a failure is raised on, seen through the pieces the builder needs from it.
pub(crate) trait FailureSink: Fallible + WithDetail {
    fn captures(&self) -> bool;
    fn print_location(&self) -> bool;
    fn subject_name(&self) -> Option<String>;
    fn expression(&self) -> Option<&'static str>;
}

impl<T, M: Mode, R> FailureSink for AssertThat<'_, T, M, R> {
    fn captures(&self) -> bool {
        M::CAPTURES
    }

    fn print_location(&self) -> bool {
        self.state.print_location
    }

    fn subject_name(&self) -> Option<String> {
        self.state.subject_name.clone()
    }

    fn expression(&self) -> Option<&'static str> {
        self.state.expression
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Where a [`FailureBuilder`] delivers its failure: [`Attached`] to an assertion chain or
/// [`Detached`] as a value. This trait is sealed.
pub trait FailureTarget: sealed::Sealed {}

/// The target of a builder started by [`AssertThat::failure`]: the failure is raised on that
/// chain by [`FailureBuilder::raise`].
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
/// [`build`](Self::build), for the nested failures a parent attaches through
/// [`child`](Self::child) and [`children`](Self::children).
#[must_use = "a failure is only recorded by `raise` or `build`"]
pub struct FailureBuilder<T: FailureTarget> {
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
    pub fn raise(self) {
        let Attached { sink, location } = self.target;
        let location = if sink.print_location() {
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
            panic!("{}", crate::report::report_for_panic(&failure));
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

    /// Sets the rendered subject. Pass an adapter obtained from [`AssertThat::render`]. It is
    /// consumed into an owned value tree here, with every leaf rendered exactly once.
    pub fn actual(mut self, actual: impl IntoRendered) -> Self {
        self.actual = Some(actual.into_rendered());
        self
    }

    /// Sets the sentence between the actual and the expected value, such as `does not contain`.
    ///
    /// A failure without a relation is a direct comparison and renders as an aligned
    /// `Expected:` / `Actual:` pair. A relation never embeds a value: values belong to
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

    /// Attaches a labeled rendered fact. Rendering adapters should be passed directly so their
    /// structure and type metadata are retained.
    pub fn fact(mut self, label: impl Into<Cow<'static, str>>, value: impl IntoRendered) -> Self {
        self.facts.push(Fact::new(label, value));
        self
    }

    /// Attaches an unlabeled note, a complete sentence.
    pub fn note(mut self, note: impl Display) -> Self {
        self.facts.push(Fact::note(format!("{note}")));
        self
    }

    /// Attaches unlabeled notes.
    pub fn notes(mut self, notes: impl IntoIterator<Item = String>) -> Self {
        self.facts.extend(notes.into_iter().map(Fact::note));
        self
    }

    /// Attaches a note stating how many `noun`s the rendering budget left out of the facts or
    /// children, when `omitted` is nonzero. `noun` is singular and pluralized as needed.
    pub fn omitted(self, omitted: usize, noun: &str) -> Self {
        if omitted == 0 {
            self
        } else {
            self.note(crate::renderer::omission(omitted, noun))
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
