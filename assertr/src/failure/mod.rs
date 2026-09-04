//! Structured assertion failures and the builder that raises them.
//!
//! [`AssertionFailure`] is what capture mode hands back and what panic mode renders. Its fields
//! carry every part of a failure as data: the rendered [`actual`](AssertionFailure::actual) and
//! [`expected`](AssertionFailure::expected) values, the [`relation`](AssertionFailure::relation)
//! between them, additional [`facts`](AssertionFailure::facts), and nested
//! [`children`](AssertionFailure::children). The human-readable text is produced from these fields
//! by the [`Display`] implementation, so a reporter never has to parse prose.
//!
//! A leaf assertion raises a failure through [`AssertThat::failure`], which returns the
//! [`FailureBuilder`] every built-in assertion uses.

mod builder;
mod display;

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::fmt::Display;

pub use builder::{Attached, Detached, FailureBuilder, FailureTarget};

use crate::{AssertThat, prelude::Mode};

/// Delimiter opening and closing every rendered failure message.
pub(crate) const BANNER: &str = "-------- assertr --------\n";

/// The family an assertion belongs to, recorded on every [`AssertionFailure`].
///
/// One tag per family, never one per method: the kind exists so reporters can filter or group
/// failures, not to describe them. The description lives in the failure's other fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FailureKind {
    /// The subject was compared for equality with a value, such as `is_equal_to` or
    /// `contains_exactly`.
    Equality,
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

/// One labeled piece of evidence attached to an [`AssertionFailure`].
///
/// Facts carry what is neither the expected nor the actual value: lengths, missing keys, unexpected
/// elements, recorded differences, or a panic payload. A fact with an empty label is a plain note.
/// [`Display`] renders a fact as `label: value`, or as the bare value when the label is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Fact {
    /// What the value describes. Empty for a plain note.
    pub label: Cow<'static, str>,

    /// The rendered evidence.
    pub value: String,
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

    /// Creates a labeled fact.
    pub fn new(label: impl Into<Cow<'static, str>>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    /// Creates an unlabeled note.
    pub fn note(value: impl Into<String>) -> Self {
        Self::new("", value)
    }

    /// Creates the [`INDEX`](Self::INDEX) fact locating a nested failure at an element index.
    #[must_use]
    pub fn index(index: usize) -> Self {
        Self::new(Self::INDEX, alloc::format!("{index}"))
    }

    /// Creates the [`KEY`](Self::KEY) fact locating a nested failure at a map key. Pass the key
    /// as an adapter obtained from [`AssertThat::render`], which is printed compactly here.
    #[must_use]
    pub fn key(rendered_key: impl core::fmt::Debug) -> Self {
        Self::new(Self::KEY, alloc::format!("{rendered_key:?}"))
    }

    /// Whether this fact locates a nested failure within its parent's subject.
    fn is_location(&self) -> bool {
        self.label == Self::INDEX || self.label == Self::KEY
    }
}

impl Display for Fact {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !self.label.is_empty() {
            f.write_str(&self.label)?;
            f.write_str(": ")?;
        }
        f.write_str(&self.value)
    }
}

/// A single structured assertion failure.
///
/// Capture-mode assertions (see [`AssertThat::capture`]) collect these instead of panicking.
/// Every part of a failure is exposed as its own field, so consumers can inspect failures
/// programmatically or compose their own rendering without parsing formatted text.
///
/// The complete human-readable form is produced by the [`Display`] implementation. In panic mode,
/// it becomes the panic message. Capture mode retains the fields without formatting that complete
/// message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssertionFailure {
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
    pub actual: Option<String>,

    /// The sentence between the actual and the expected value, such as `does not contain` or
    /// `is not greater than`. A failure without a relation is a direct comparison of
    /// [`expected`](Self::expected) and [`actual`](Self::actual), which [`Display`] renders as an
    /// aligned `Expected:` / `Actual:` pair.
    pub relation: Option<Cow<'static, str>>,

    /// The value the subject was compared with, rendered through the chain's
    /// [`ValueRenderer`](crate::ValueRenderer), if the assertion has one.
    pub expected: Option<String>,

    /// The value a negated assertion found, although it was not expected, rendered through the
    /// chain's [`ValueRenderer`](crate::ValueRenderer). Set by assertions such as
    /// `does_not_contain` and `is_not_equal_to` instead of [`expected`](Self::expected).
    pub unexpected: Option<String>,

    /// Evidence attached by the failing assertion itself and scoped to exactly this failure: the
    /// differences of an equality comparison, the elements a collection assertion could not find,
    /// or the length behind a length assertion. Each fact renders as `label: value`, or as the
    /// bare value for a note.
    pub facts: Vec<Fact>,

    /// User-provided detail messages (`with_detail_message` / `add_detail_message`) collected
    /// from the assertion chain. Contains only the messages provided up to the point this
    /// failure was raised. A message added later appears only in the failures raised after it.
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
    /// The assertion-specific text: the rendered subject, the relation, and the expected or
    /// unexpected value, exactly as [`Display`] shows them, without the location, the subject
    /// name, the expression, and the `Messages:`, `Details:`, and `Nested failures:` blocks.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!(42).capture(|it| it.is_equal_to(43));
    ///
    /// assert_that!(failures[0].description()).is_equal_to("Expected: 43\n\n  Actual: 42\n");
    /// ```
    #[must_use]
    pub fn description(&self) -> String {
        display::body(
            self.actual.as_deref(),
            self.relation.as_deref(),
            self.expected.as_deref(),
            self.unexpected.as_deref(),
        )
    }

    /// Prepends a fact locating this failure within its parent's subject, such as
    /// [`Fact::index`] or [`Fact::key`]. [`Display`] uses it as the heading of the nested failure.
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
        match &self.state.parent {
            Some(parent) => parent.store_failure(failure),
            None => self.state.failures.borrow_mut().push(failure),
        }
    }
}

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Starts a failure of the given kind, located at the caller.
    ///
    /// This is the failure path of every leaf assertion, called after
    /// [`AssertThat::track_assertion`] when the condition does not hold. Fill in the rendered
    /// values, the relation, facts, and children, then call [`FailureBuilder::raise`], which
    /// records the failure in capture mode and panics otherwise. See
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
