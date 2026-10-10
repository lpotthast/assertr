//! Structured assertion failures and the builder that creates them.
//!
//! An [`AssertionFailure`] is what capture mode returns and what panic mode turns into a panic
//! message. It stores every part of the failure as data: the rendered
//! [`actual`](AssertionFailure::actual) and [`expected`](AssertionFailure::expected) values, the
//! [`relation`](AssertionFailure::relation) between them, additional
//! [`facts`](AssertionFailure::facts), and nested [`children`](AssertionFailure::children). Code
//! that processes failures reads these fields and never has to parse the report text.
//!
//! ```
//! use assertr::prelude::*;
//!
//! let failures = assert_that!(42).capture(|it| it.is_less_than(0).is_equal_to(43));
//! let reports: Vec<String> = failures.iter().map(ToString::to_string).collect();
//! assert_that!(&reports[0]).contains("is not less than");
//! assert_that!(&reports[1]).contains("Expected: 43");
//! ```
//!
//! ## From assertion to report
//!
//! 1. **Building:** A failing [`Expectation`](crate::expectation::Expectation) describes itself in
//!    [`Expectation::explain`](crate::expectation::Expectation::explain) by filling in a
//!    [`FailureBuilder`]. Assertions that cannot use an expectation start a builder with
//!    [`AssertThat::failure`] instead. Values are rendered into [`Rendered`] trees through the
//!    chain's [renderer and budget](crate::renderer).
//! 2. **Handling:** In capture mode, [`AssertThat::capture`] collects the failures and returns
//!    them. In panic mode, the first failure panics with the text produced by the [panic
//!    presentation](AssertThat::with_panic_presentation).
//! 3. **Presentation:** The `Display` implementation of [`AssertionFailure`] produces the default
//!    report, which is also the default panic text. Your own code can build any other format from
//!    the fields.
//!
//! A presentation receives rendered values, not the original Rust values. It can inspect their
//! structure, types, and omission counts without rendering anything again. To create failures in
//! your own assertions, see [custom assertions](crate#custom-assertions).

mod builder;
mod failures;
pub use failures::AssertionFailures;
pub(crate) mod panic_presentation;
pub(crate) mod report;

use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};

pub use builder::FailureBuilder;

use crate::{AssertThat, mode::Mode, renderer::Rendered};

/// Delimiter opening and closing every rendered failure message.
pub(crate) const BANNER: &str = "-------- assertr --------\n";

/// The family an assertion belongs to, recorded on every [`AssertionFailure`].
///
/// One tag per family, never one per method: the kind exists so consumers can filter or group
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

/// One piece of evidence attached to an [`AssertionFailure`], optionally labelled.
///
/// Facts carry what is neither the expected nor the actual value: lengths, missing keys, unexpected
/// elements, recorded differences, or a panic payload. A fact without a label is a plain note.
/// Labels receive no special treatment in the built-in report. Locations belong in
/// [`AssertionFailure::path`].
///
/// ```
/// use assertr::{failure::Fact, prelude::*};
///
/// let fact = Fact::labelled("Actual length", "3");
/// assert_that!(fact.label.as_deref()).is_equal_to(Some("Actual length"));
///
/// let note = Fact::note("The collection was empty.");
/// assert_that!(note.label).is_none();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Fact {
    /// What the value describes. `None` for a plain note.
    pub label: Option<Cow<'static, str>>,

    /// The rendered evidence tree.
    pub value: Rendered,
}

impl Fact {
    /// Creates a labelled fact from a rendered evidence tree.
    ///
    /// Render diagnostic values through
    /// [`AssertionContext::render`](crate::expectation::AssertionContext::render) in expectations
    /// or [`AssertThat::render`] in execution adapters, so the active renderer and budget
    /// apply. Structural metadata and caller-authored prose may be passed as verbatim text.
    pub fn labelled(label: impl Into<Cow<'static, str>>, value: impl Into<Rendered>) -> Self {
        Self {
            label: Some(label.into()),
            value: value.into(),
        }
    }

    /// Creates an unlabelled note from a rendered evidence tree.
    ///
    /// Pass diagnostic values through
    /// [`AssertionContext::render`](crate::expectation::AssertionContext::render)
    /// or [`AssertThat::render`]. Caller-authored prose may be supplied as verbatim text.
    pub fn note(value: impl Into<Rendered>) -> Self {
        Self {
            label: None,
            value: value.into(),
        }
    }
}

/// A single structured assertion failure.
///
/// Capture-mode assertions (see [`AssertThat::capture`]) collect these instead of panicking. Every
/// part of a failure is exposed as its own field, so consumers can inspect failures
/// programmatically or compose their own rendering without parsing formatted text.
///
/// Fields can be asserted on through [`AssertThat::derive`]:
///
/// ```
/// use assertr::prelude::*;
///
/// let failures = assert_that!([1]).capture(|it| it.has_length(2));
/// assert_that!(failures[0])
///     .derive(|failure| &failure.facts)
///     .contains_satisfying(|fact| {
///         fact.derive_owned(|fact| fact.label.as_deref())
///             .is_equal_to(Some("Actual length"));
///         fact.derive(|fact| &fact.value.type_name)
///             .is_equal_to(Some("usize"));
///     });
/// ```
///
/// `Display` and `Debug` produce the complete human-readable report, the default panic text.
/// This type also implements [`core::error::Error`] for ordinary Result propagation, without
/// treating nested assertion evidence as an error cause chain. Panic mode uses the selected
/// [panic presentation](crate::AssertThat::with_panic_presentation) to produce the panic text.
/// Capture mode retains the fields without invoking presentation.
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
    /// `with_location(false)`, and for nested failures, which their [`path`](Self::path) locates.
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

    /// The subject, rendered through the chain's
    /// [`ValueRenderer`](crate::renderer::ValueRenderer), if the assertion shows it.
    pub actual: Option<Rendered>,

    /// The sentence between the actual and the expected value, such as `does not contain` or `is
    /// not greater than`. A failure without a relation is a direct comparison of
    /// [`expected`](Self::expected) and [`actual`](Self::actual), which the human-readable report
    /// renders as an aligned `Expected:` / `Actual:` pair.
    pub relation: Option<Cow<'static, str>>,

    /// The value the subject was compared with, rendered through the chain's
    /// [`ValueRenderer`](crate::renderer::ValueRenderer), if the assertion has one.
    pub expected: Option<Rendered>,

    /// The value a negated assertion found, although it was not expected, rendered through the
    /// chain's [`ValueRenderer`](crate::renderer::ValueRenderer). Set by assertions such as
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
    /// Starts a failure of the given kind over this chain's subject type.
    ///
    /// This is the failure path of an execution adapter: an assertion that owns an invocation,
    /// consumption, or polling step the borrowed expectation protocol cannot express. Such an
    /// adapter calls [`AssertThat::track_assertion`] before its operation. When the condition
    /// does not hold, fill in the values rendered through [`AssertThat::render`], the relation,
    /// facts, and children, then pass the builder to [`AssertThat::raise`].
    /// [`AssertThat::render`] shows an example.
    ///
    /// Reusable leaf checks implement [`Expectation`](crate::expectation::Expectation) instead. The
    /// executor then supplies the builder and raises the failure. See [custom
    /// assertions](crate#custom-assertions).
    pub fn failure(&self, kind: FailureKind) -> FailureBuilder {
        FailureBuilder::new::<T>(kind)
    }

    /// Records the failure in capture mode and panics with its rendered form otherwise. The
    /// failure is located at the caller.
    ///
    /// # Panics
    ///
    /// Panics with the formatted failure message when not in capture mode.
    #[track_caller]
    pub fn raise(&self, failure: FailureBuilder) {
        self.raise_at(failure, core::panic::Location::caller());
    }

    /// Raises a failure at an explicit location, adding this chain's subject metadata.
    #[track_caller]
    pub(crate) fn raise_at(
        &self,
        failure: FailureBuilder,
        location: &'static core::panic::Location<'static>,
    ) {
        let state = &self.state;
        let failure = self.complete_at(failure, location);

        if crate::mode::captures::<M>() {
            state.records.store_failure(
                failure,
                #[cfg(feature = "fluent")]
                state.settings.expression.pending_fluent(),
            );
        } else {
            present_and_panic(&failure, state.settings.panic_presentation.as_deref());
        }
    }

    /// Completes failure metadata without presenting, panicking, or recording it.
    pub(crate) fn complete_at(
        &self,
        failure: FailureBuilder,
        location: &'static core::panic::Location<'static>,
    ) -> AssertionFailure {
        let mut failure = self.state.settings.complete(failure, location);
        self.state.records.collect_messages(&mut failure.messages);
        failure
    }
}

/// Panics with the text the panic presentation produces for `failure`.
#[track_caller]
pub(crate) fn present_and_panic(
    failure: &AssertionFailure,
    presentation: Option<&panic_presentation::PanicPresentation>,
) -> ! {
    let text = panic_presentation::render(failure, presentation);
    panic!("{text}");
}

impl core::fmt::Display for AssertionFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        report::write_failure(self, f)
    }
}

impl core::fmt::Debug for AssertionFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl core::error::Error for AssertionFailure {}
