//! Structured assertion failures and the builder that raises them.
//!
//! [`AssertionFailure`] is what capture mode hands back and what panic mode renders. Its fields
//! carry every part of a failure as data: the rendered [`actual`](AssertionFailure::actual) and
//! [`expected`](AssertionFailure::expected) values, the [`relation`](AssertionFailure::relation)
//! between them, additional [`facts`](AssertionFailure::facts), and nested
//! [`children`](AssertionFailure::children). Adapters consume these fields directly, so no
//! machine-readable use needs to parse the human-readable text report.
//!
//! ## From assertion to report
//!
//! 1. **Construction:** A failed leaf assertion builds an [`AssertionFailure`] through
//!    [`AssertThat::failure`] and [`FailureBuilder`]. Diagnostic values are rendered through the
//!    chain's [renderer and budget](crate::renderer) into owned [`Rendered`] trees.
//! 2. **Handling:** [`AssertThat::capture`] stores failures and returns them to the caller. Panic
//!    mode stops at the first failure and asks the selected [presentation
//!    adapter](AssertThat::with_panic_presentation) for panic text.
//! 3. **Presentation:** An [adapter] reads the structured failure and produces another
//!    representation. [`ToHumanReadableText`](adapter::ToHumanReadableText) produces the default
//!    report. Capture mode leaves this step to the caller.
//!
//! Adapters receive rendered evidence, not the original Rust values. They can inspect structure,
//! type metadata, and omission counts without parsing a report or rendering leaves again. For
//! examples, start with [capturing failures](AssertThat::capture) or [processing them](adapter).
//! To create failures in your own methods, see [custom assertions](crate#custom-assertions).

pub mod adapter;
mod builder;
mod failures;
pub use failures::AssertionFailures;
pub(crate) mod panic_presentation;

use crate::{
    AssertThat,
    prelude::Mode,
    renderer::{Compact, IntoRendered, Rendered},
};
use alloc::{borrow::Cow, string::String, vec::Vec};

pub use builder::{Attached, Detached, FailureBuilder, FailureTarget};

/// Delimiter opening and closing every rendered failure message.
pub(crate) const BANNER: &str = "-------- assertr --------\n";

/// The family an assertion belongs to, recorded on every [`AssertionFailure`].
///
/// One tag per family, never one per method: the kind exists so adapters can filter or group
/// failures, not to describe them. The description lives in the failure's other fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FailureKind {
    /// The subject was compared for equality with a value, such as `is_equal_to` or
    /// `contains_exactly`.
    Equality,
    /// An expected-side matcher rejected the subject.
    Matching,
    /// The subject was compared by order or range, such as `is_greater_than` or `is_in_range`.
    Ordering,
    /// An element, key, entry, prefix, suffix, or subset was looked up, such as `contains` or
    /// `is_subset_of`.
    Membership,
    /// The subject's length or emptiness was checked.
    Length,
    /// The subject was checked for an enum variant, such as `is_some` or `is_ok`.
    Variant,
    /// The subject or its elements were checked against a predicate, condition, or nested
    /// assertions, such as `matches`, `is(condition)`, or `contains_satisfying`.
    Predicate,
    /// A closure was expected to panic or not to panic.
    Panic,
    /// A failure of any other family.
    Other,
}

/// A relative location within a matched subject. Paths compose from parent to child.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathSegment {
    /// A named field.
    Field(&'static str),
    /// A tuple field.
    TupleIndex(usize),
    /// A required enum variant.
    Variant(&'static str),
    /// A position in a stable-order collection.
    Index(usize),
    /// A map key rendered with the active renderer.
    Key(Rendered),
}

/// One labeled piece of evidence attached to an [`AssertionFailure`].
///
/// Facts carry what is neither the expected nor the actual value: lengths, missing keys, unexpected
/// elements, recorded differences, or a panic payload. A fact with an empty label is a plain note.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Fact {
    /// What the value describes. Empty for a plain note.
    pub label: Cow<'static, str>,

    /// The rendered evidence tree.
    pub value: Rendered,
}

impl Fact {
    /// The label of the fact locating a nested failure at a zero-based element index.
    ///
    /// Positional collection and iterator assertions attach it to each
    /// [child](AssertionFailure::children) they raise for one element. Order-free collections do
    /// not expose an iteration offset and attach no such fact.
    pub const INDEX: &'static str = "index";

    /// The label of the fact locating a nested failure at a rendered map key.
    pub const KEY: &'static str = "key";

    /// Creates a labeled fact, rendering its value once into an owned evidence tree.
    ///
    /// Pass diagnostic values through [`AssertThat::render`] so the active renderer and budget
    /// apply. Structural metadata and caller-authored prose may be passed as verbatim text.
    pub fn labelled(label: impl Into<Cow<'static, str>>, value: impl IntoRendered) -> Self {
        Self {
            label: label.into(),
            value: value.into_rendered(),
        }
    }

    /// Creates an unlabeled note, rendering its value once into an owned evidence tree.
    ///
    /// Pass diagnostic values through [`AssertThat::render`]. Caller-authored prose may be
    /// supplied as verbatim text.
    pub fn note(value: impl IntoRendered) -> Self {
        Self::labelled("", value)
    }

    /// Creates the [`INDEX`](Self::INDEX) fact locating a nested failure at an element index.
    #[must_use]
    pub fn index(index: usize) -> Self {
        Self::labelled(Self::INDEX, index)
    }

    /// Creates the [`KEY`](Self::KEY) fact locating a nested failure at a map key. Pass the key as
    /// an adapter obtained from [`AssertThat::render`], which is printed compactly here.
    #[must_use]
    pub fn key(rendered_key: impl IntoRendered) -> Self {
        Self {
            label: Cow::Borrowed(Self::KEY),
            value: Compact(rendered_key).into_rendered(),
        }
    }

    /// Returns what this fact describes, or an empty string for a plain note.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Borrows the rendered evidence tree.
    #[must_use]
    pub const fn value(&self) -> &Rendered {
        &self.value
    }

    /// Whether this fact locates a nested failure within its parent's subject.
    pub(crate) fn is_location(&self) -> bool {
        self.label == Self::INDEX || self.label == Self::KEY
    }
}

/// A single structured assertion failure.
///
/// Capture-mode assertions (see [`AssertThat::capture`]) collect these instead of panicking. Every
/// part of a failure is exposed as its own field, so consumers can inspect failures
/// programmatically or compose their own rendering without parsing formatted text.
///
/// Read-only accessors also work as projection functions. Use [`AssertThat::derive`] for
/// borrowed sized values and [`AssertThat::derive_owned`] for slices, strings, optional views,
/// and copied values:
///
/// ```
/// use assertr::{prelude::*, Fact, renderer::Rendered};
///
/// let failures = assert_that!([1]).capture(|it| it.has_length(2));
/// assert_that!(failures[0])
///     .derive_owned(AssertionFailure::facts)
///     .contains_satisfying(|fact| {
///         fact.derive_owned(Fact::label).is_equal_to("Actual length");
///         fact.derive(Fact::value)
///             .derive_owned(Rendered::type_name)
///             .is_equal_to(Some("usize"));
///     });
/// ```
///
/// `Display` and `Debug` use the default plain report. This type also implements
/// [`core::error::Error`] for ordinary Result propagation, without treating nested assertion
/// evidence as an error cause chain. The complete human-readable form is produced by
/// [`ToHumanReadableText`](adapter::ToHumanReadableText). Panic mode uses the selected
/// [presentation adapter](crate::AssertThat::with_panic_presentation) to produce the panic text.
/// Capture mode retains the fields without invoking presentation. Captured failures can be
/// explicitly passed to any [adapter](adapter::Adapter).
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssertionFailure {
    /// A structured matcher constraint, including its rendered operands and branches.
    pub constraint: Option<crate::matchers::ConstraintDescription>,
    /// Relative path from the parent subject.
    pub path: Vec<PathSegment>,

    /// Number of diagnostic children omitted by the rendering budget.
    pub omitted_children: usize,
    /// Where the failing assertion was invoked. `None` when location printing was disabled via
    /// `with_location(false)`.
    pub location: Option<&'static core::panic::Location<'static>>,

    /// The name given to the assertion's subject via `with_subject_name`, if any.
    pub subject_name: Option<String>,

    /// The source expression that produced the assertion subject, if the entry point captured it.
    /// Derived child chains start a new diagnostic subject and do not inherit their parent's
    /// expression.
    pub expression: Option<&'static str>,

    /// The Rust type name of the assertion subject that raised this failure.
    ///
    /// This is produced by [`core::any::type_name`] and is intended for diagnostics. A failure
    /// raised by a derived child records the child's subject type rather than the root's.
    pub subject_type_name: &'static str,

    /// The subject, rendered through the chain's [`ValueRenderer`](crate::ValueRenderer), if the
    /// assertion shows it.
    pub actual: Option<Rendered>,

    /// The sentence between the actual and the expected value, such as `does not contain` or `is
    /// not greater than`. A failure without a relation is a direct comparison of
    /// [`expected`](Self::expected) and [`actual`](Self::actual), which the human-readable adapter
    /// renders as an aligned `Expected:` / `Actual:` pair.
    pub relation: Option<Cow<'static, str>>,

    /// The value the subject was compared with, rendered through the chain's
    /// [`ValueRenderer`](crate::ValueRenderer), if the assertion has one.
    pub expected: Option<Rendered>,

    /// The value a negated assertion found, although it was not expected, rendered through the
    /// chain's [`ValueRenderer`](crate::ValueRenderer). Set by assertions such as
    /// `does_not_contain` and `is_not_equal_to` instead of [`expected`](Self::expected).
    pub unexpected: Option<Rendered>,

    /// Evidence attached by the failing assertion itself and scoped to exactly this failure: the
    /// differences of an equality comparison, the elements a collection assertion could not find,
    /// or the length behind a length assertion. Each fact renders as `label: value`, or as the
    /// bare value for a note.
    pub facts: Vec<Fact>,

    /// User-provided detail messages (`with_detail_message` / `add_detail_message`) collected from
    /// the assertion chain. Contains only the messages provided up to the point this failure was
    /// raised. A message added later appears only in the failures raised after it.
    pub messages: Vec<String>,

    /// Failures raised by nested assertions, such as the per-element assertions of
    /// `contains_satisfying`, or produced for the elements a positional assertion rejected.
    ///
    /// A child raised for one element of a positional subject carries a [`Fact::INDEX`] fact, a
    /// child raised for one map value a [`Fact::KEY`] fact. Children of a sequence are ordered by
    /// element position. Children of a set or map whose iteration order is not deterministic are
    /// ordered by their rendered text.
    pub children: Vec<AssertionFailure>,

    /// The assertion family that raised this failure.
    pub kind: FailureKind,
}

impl AssertionFailure {
    /// Borrows the structured matcher constraint, when present.
    #[must_use]
    pub const fn constraint(&self) -> Option<&crate::matchers::ConstraintDescription> {
        self.constraint.as_ref()
    }

    /// Borrows the rendered subject, when present.
    #[must_use]
    pub const fn actual(&self) -> Option<&Rendered> {
        self.actual.as_ref()
    }

    /// Returns the relation sentence, when present.
    #[must_use]
    pub fn relation(&self) -> Option<&str> {
        self.relation.as_deref()
    }

    /// Borrows the rendered expected value, when present.
    #[must_use]
    pub const fn expected(&self) -> Option<&Rendered> {
        self.expected.as_ref()
    }

    /// Borrows the rendered unexpected value, when present.
    #[must_use]
    pub const fn unexpected(&self) -> Option<&Rendered> {
        self.unexpected.as_ref()
    }

    /// Borrows the evidence attached to this failure.
    #[must_use]
    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }

    /// Borrows the user-provided messages collected from the assertion chain.
    #[must_use]
    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    /// Borrows the failures raised by nested assertions.
    #[must_use]
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    /// Returns the assertion family that raised this failure.
    #[must_use]
    pub const fn kind(&self) -> FailureKind {
        self.kind
    }

    /// Prepends a fact locating this failure within its parent's subject, such as [`Fact::index`]
    /// or [`Fact::key`]. The human-readable adapter uses it as the heading of the nested failure.
    #[must_use]
    pub fn located_at(mut self, fact: Fact) -> Self {
        self.facts.insert(0, fact);
        self
    }
}

/// Failures raised on a chain are stored on its root.
pub(crate) trait Fallible {
    fn store_failure(&self, failure: AssertionFailure);
}

impl<T, M: Mode, R> Fallible for AssertThat<'_, T, M, R> {
    fn store_failure(&self, failure: AssertionFailure) {
        #[cfg(feature = "fluent")]
        let expression = match self.state.expression {
            crate::Expression::PendingFluent(location) => Some(location),
            _ => None,
        };
        self.state.records.store_failure(
            failure,
            #[cfg(feature = "fluent")]
            expression,
        );
    }
}

impl crate::ChainRecords<'_> {
    fn store_failure(
        &self,
        failure: AssertionFailure,
        #[cfg(feature = "fluent")] expression: Option<&'static core::panic::Location<'static>>,
    ) {
        match self.parent {
            Some(parent) => parent.store_failure(
                failure,
                #[cfg(feature = "fluent")]
                expression,
            ),
            None => self.failures.borrow_mut().push(
                failure,
                #[cfg(feature = "fluent")]
                expression,
            ),
        }
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Starts a failure of the given kind, located at the caller.
    ///
    /// This is the failure path of every leaf assertion, called after
    /// [`AssertThat::track_assertion`] when the condition does not hold. Fill in the rendered
    /// values, the relation, facts, and children, then call [`FailureBuilder::raise`], which
    /// records the failure in capture mode or panics immediately in panic mode. See
    /// [custom assertions](crate#custom-assertions) for the shape such an assertion takes.
    #[track_caller]
    pub fn failure(&self, kind: FailureKind) -> FailureBuilder<Attached<'_>> {
        self.failure_at(kind, core::panic::Location::caller())
    }

    /// Starts a failure of the given kind at an explicit location.
    pub(crate) fn failure_at(
        &self,
        kind: FailureKind,
        location: &'static core::panic::Location<'static>,
    ) -> FailureBuilder<Attached<'_>> {
        FailureBuilder::attached(self, core::any::type_name::<T>(), location, kind)
    }
}

impl core::fmt::Display for AssertionFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&adapter::ToHumanReadableText.render(self), f)
    }
}

impl core::fmt::Debug for AssertionFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl core::error::Error for AssertionFailure {}
