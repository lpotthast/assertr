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
//! 1. **Construction:** A rejected [`Expectation`](crate::Expectation) explains itself through
//!    [`ExpectationDiagnostics::explain`](crate::ExpectationDiagnostics::explain), populating the
//!    [`FailureBuilder`] supplied by the chain executor. Execution adapters that cannot use the
//!    expectation protocol start a builder with [`AssertThat::failure`] instead. Diagnostic values
//!    are rendered through the chain's [renderer and budget](crate::renderer) into owned
//!    [`Rendered`] trees.
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
    renderer::{IntoRendered, Rendered},
};
use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};

pub use builder::{Attached, Detached, FailureBuilder};

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
    /// The subject was checked against a domain property or Rust pattern, such as `is_lowercase`
    /// or `is_matching`.
    Predicate,
    /// A closure was expected to panic or not to panic.
    Panic,
    /// A failure of any other family.
    Other,
}

/// A relative location within an assertion subject. Paths compose from parent to child.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathSegment {
    /// A named field.
    Field(&'static str),
    /// A tuple field.
    TupleIndex(usize),
    /// A required enum variant.
    Variant(&'static str),
    /// A position in a stable-order collection or the yield order of a direct iterator assertion.
    Index(usize),
    /// A map key rendered with the active renderer.
    Key(Rendered),
}

/// One labeled piece of evidence attached to an [`AssertionFailure`].
///
/// Facts carry what is neither the expected nor the actual value: lengths, missing keys, unexpected
/// elements, recorded differences, or a panic payload. A fact with an empty label is a plain note.
/// Labels receive no special treatment in the built-in report. Locations belong in
/// [`AssertionFailure::path`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Fact {
    /// What the value describes. Empty for a plain note.
    pub label: Cow<'static, str>,

    /// The rendered evidence tree.
    pub value: Rendered,
}

impl Fact {
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
    /// An unmet expectation with no subject, using the same diagnostic fields as a rejection.
    pub constraint: Option<Box<AssertionFailure>>,
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
    /// A child raised for one element of a positional subject carries a [`PathSegment::Index`],
    /// and a child raised for one map value a [`PathSegment::Key`], in its relative
    /// [`path`](Self::path). Order-free element evidence has no index path. Children of a
    /// sequence are ordered by element position. Children of a set or map whose iteration
    /// order is not deterministic are ordered by their rendered text.
    pub children: Vec<AssertionFailure>,

    /// The assertion family that raised this failure.
    pub kind: FailureKind,
}

impl AssertionFailure {
    /// Borrows the diagnostic describing an unmet expectation, when present.
    #[must_use]
    pub fn constraint(&self) -> Option<&AssertionFailure> {
        self.constraint.as_deref()
    }

    /// Borrows the relative path from the parent subject.
    #[must_use]
    pub fn path(&self) -> &[PathSegment] {
        &self.path
    }

    /// Returns the number of diagnostic children omitted by the rendering budget.
    #[must_use]
    pub const fn omitted_children(&self) -> usize {
        self.omitted_children
    }

    /// Returns where the failing assertion was invoked, when location recording was enabled.
    #[must_use]
    pub const fn location(&self) -> Option<&'static core::panic::Location<'static>> {
        self.location
    }

    /// Returns the name given to the subject via `with_subject_name`, when present.
    #[must_use]
    pub fn subject_name(&self) -> Option<&str> {
        self.subject_name.as_deref()
    }

    /// Returns the source expression that produced the subject, when the entry point captured it.
    #[must_use]
    pub const fn expression(&self) -> Option<&'static str> {
        self.expression
    }

    /// Returns the Rust type name of the subject that raised this failure.
    #[must_use]
    pub const fn subject_type_name(&self) -> &'static str {
        self.subject_type_name
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
}

impl crate::ChainRecords<'_> {
    /// Stores a captured failure on the root of this chain.
    pub(crate) fn store_failure(
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
    /// This is the failure path of an execution adapter: an assertion that owns an invocation,
    /// consumption, or polling step the borrowed expectation protocol cannot express. Such an
    /// adapter calls [`AssertThat::track_assertion`] before its operation. When the condition
    /// does not hold, fill in the values rendered through [`AssertThat::render`], the relation,
    /// facts, and children, then call [`FailureBuilder::raise`], which records the failure in
    /// capture mode or panics immediately in panic mode. [`AssertThat::render`] shows an example.
    ///
    /// Reusable leaf checks implement [`Expectation`](crate::Expectation) and
    /// [`ExpectationDiagnostics`](crate::ExpectationDiagnostics) instead. The executor then
    /// supplies the builder and raises the failure. See [custom
    /// assertions](crate#custom-assertions).
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
        FailureBuilder::attached(self, location, kind)
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::{FailureBuilder, FailureKind, PathSegment};

    mod accessors {
        use super::*;

        #[test]
        fn expose_the_subject_and_location_fields() {
            let failures = assert_that!(1)
                .with_subject_name("answer")
                .capture(|it| it.is_equal_to(2));
            let failure = &failures[0];

            assert_that!(failure.subject_name()).is_equal_to(Some("answer"));
            assert_that!(failure.expression()).is_equal_to(Some("1"));
            assert_that!(failure.subject_type_name()).is_equal_to("i32");
            assert_that!(failure.location().map(core::panic::Location::file))
                .is_equal_to(Some(file!()));
            assert_that!(failure.path()).is_empty();
            assert_that!(failure.omitted_children()).is_equal_to(0);
        }

        #[test]
        fn expose_the_path_and_omitted_children() {
            let failure = FailureBuilder::detached::<i32>(FailureKind::Other)
                .path([PathSegment::Index(3)])
                .omitted_children(2)
                .build();

            assert_that!(failure.path()).contains_exactly([PathSegment::Index(3)]);
            assert_that!(failure.omitted_children()).is_equal_to(2);
            assert_that!(failure.location()).is_none();
        }
    }
}
