#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Allow functions named `is_*`, taking self by value instead of taking self by mutable reference or
// reference.
#![allow(clippy::wrong_self_convention)]
#![doc = include_str!(concat!(
env!("CARGO_MANIFEST_DIR"),
"/",
env!("CARGO_PKG_README")
))]
//!
//! ## Core model
//!
//! An [`AssertThat<T>`](AssertThat) holds an owned or borrowed [`Actual<T>`](Actual). Methods are
//! selected by `T`, independent of ownership. Borrowing entry points normalize sized references to
//! their pointee. Owned references and unsized targets remain reference-typed subjects.
//!
//! [`AssertThat::derive`] creates a child assertion for part of a subject. Its failures propagate
//! to the root. The [`AssertThat::satisfies`] family asserts on a child and returns the original
//! chain. Its variants cover borrowed, owned, and unsized projections.
//!
//! Panic mode stops at the first failure. Capture mode collects structured [`AssertionFailure`]
//! values within [`AssertThat::capture`] or the fluent `verify` entry points. A failure carries its
//! structured rendered [`actual`](AssertionFailure::actual) and
//! [`expected`](AssertionFailure::expected) values, the [`relation`](AssertionFailure::relation)
//! between them, further [`facts`](AssertionFailure::facts), nested
//! [`children`](AssertionFailure::children), and a [`kind`](AssertionFailure::kind) as data. An
//! [`Adapter`](failure::adapter::Adapter) transforms that data, and adapters compose into typed
//! chains. Capture mode stores failures without invoking presentation. Panic mode uses the
//! context's [presentation adapter](AssertThat::with_panic_presentation) to produce the panic text,
//! defaulting to [`ToHumanReadableText`](failure::adapter::ToHumanReadableText).
//!
//! ## Custom assertions
//!
//! Add a method such as `.is_adult()` when a domain check appears throughout your tests. For a
//! single check on a field, start with [`AssertThat::satisfies`]. To describe selected fields and
//! nested values together, use [`partial!`](mod@matchers#structural-syntax). Its field expectations
//! can use existing assertion methods through [`matchers::satisfying`], including your custom ones.
//!
//! Existing assertion families also work with custom types that implement their capabilities.
//! For example, [`HasLength`](assertions::HasLength) provides length assertions and
//! [`Collection`](assertions::collection::Collection) provides order-free element assertions. See
//! the [assertion families](assertions) before introducing a separate trait.
//!
//! ### Define a chainable method
//!
//! Define your own assertion trait and implement it for `AssertThat<'_, YourType, M, R>`. Use
//! `M: Mode` so the same implementation works in panic and capture mode. Keep `R` unconstrained on
//! the impl, and put renderer and `Clone` bounds on each method that needs them, in both the trait
//! and impl. This keeps one method's rendering needs from hiding the entire trait. A default of
//! `R = DebugRenderer` lets callers name the trait without specifying a renderer.
//!
//! For a chainable check, take and return `Self`. Mark the method `#[track_caller]` so failures
//! report its caller's location. The example below shows two implementation styles:
//!
//! - **Composition:** Delegate to existing assertions through [`AssertThat::satisfies`] and
//!   friends. Delegated assertions handle tracking, diagnostics, and capture mode. Do not call
//!   [`AssertThat::track_assertion`] again in a method that only delegates.
//! - **Leaf assertion:** Call [`AssertThat::track_assertion`] first. When the condition fails,
//!   build structured evidence and raise it as described below.
//!
//! ### Build a leaf failure
//!
//! Start with [`AssertThat::failure`] and the [`FailureKind`] of the assertion's family. Supply
//! the [`actual`](failure::FailureBuilder::actual) value, a lowercase
//! [`relation`](failure::FailureBuilder::relation) sentence without embedded values or a trailing
//! period, and any [`expected`](failure::FailureBuilder::expected) or
//! [`unexpected`](failure::FailureBuilder::unexpected) value. Add labeled
//! [`fact`](failure::FailureBuilder::fact)s, [`note`](failure::FailureBuilder::note)s, or nested
//! [`children`](failure::FailureBuilder::children) for further evidence. Call
//! [`raise`](failure::FailureBuilder::raise) to record the failure or panic according to the mode.
//!
//! Render diagnostic values through [`AssertThat::render`]. Its
//! [`value`](renderer::RenderingContext::value), [`values`](renderer::RenderingContext::values),
//! and [`borrowed_values`](renderer::RenderingContext::borrowed_values) adapters apply the active
//! renderer and rendering budget. Pass these adapters directly to the builder. This preserves
//! structured values and type metadata for [failure adapters](failure::adapter) and lets Assertr
//! produce a consistent report. See the [rendering guide](renderer) for customization.
//!
//! ### Example
//!
//! ```
//! use assertr::prelude::*;
//! use assertr::failure::FailureKind;
//!
//! #[derive(Debug)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! trait PersonAssertions<R = DebugRenderer> {
//!     #[track_caller]
//!     fn is_adult(self) -> Self
//!     where
//!         R: Clone + ValueRenderer<u32>;
//!
//!     #[track_caller]
//!     fn has_name(self) -> Self
//!     where
//!         R: ValueRenderer<Person> + ValueRenderer<String>;
//! }
//!
//! impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
//!     // Composed: the delegated assertion tracks itself and formats the failure.
//!     #[track_caller]
//!     fn is_adult(self) -> Self
//!     where
//!         R: Clone + ValueRenderer<u32>,
//!     {
//!         self.satisfies(|person| &person.age, |age| {
//!             age.is_greater_or_equal_to(18);
//!         })
//!     }
//!
//!     // Leaf: track first, then raise a failure built from rendered values.
//!     #[track_caller]
//!     fn has_name(self) -> Self
//!     where
//!         R: ValueRenderer<Person> + ValueRenderer<String>,
//!     {
//!         self.track_assertion();
//!         if self.actual().name.is_empty() {
//!             self.failure(FailureKind::Predicate)
//!                 .actual(self.render().value(self.actual()))
//!                 .relation("has no name")
//!                 .fact("Name", self.render().value(&self.actual().name))
//!                 .raise();
//!         }
//!         self
//!     }
//! }
//!
//! assert_that!(Person { name: "Ada".into(), age: 36 }).is_adult().has_name();
//!
//! let failures = assert_that!(Person { name: "".into(), age: 16 })
//!     .capture(|person| person.is_adult().has_name());
//! assert_eq!(failures.len(), 2);
//! ```
//!
//! Assertr's own `*Assertions` traits are public for method discovery only. Implementing them for
//! other types is not supported. See [API stability](#api-stability).

extern crate alloc;
extern crate core;
extern crate self as assertr;
#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[doc(hidden)]
pub mod __private;
pub mod actual;
mod assert_that;
pub mod assertions;
pub mod condition;
mod conversion;
mod details;
mod entry;
pub mod failure;
pub mod matchers;
pub mod mode;
/// One glob import brings every assertion into scope.
///
/// ```
/// use assertr::prelude::*;
/// ```
pub mod prelude;
pub mod renderer;
#[cfg(test)]
mod test_support;
mod tracking;
mod util;

use actual::Actual;
use alloc::{string::String, vec::Vec};
use core::{
    cell::RefCell,
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};
use details::WithDetail;
use failure::Fallible;
use mode::Mode;
use tracking::NumberOfAssertions;

#[cfg(feature = "fluent")]
pub use assertr_macros::fluent_expressions;
#[cfg(feature = "matchers")]
pub use assertr_macros::partial;
#[cfg(feature = "fluent")]
pub use entry::{IntoAssertContext, IntoOwnedAssertContext};
pub use entry::{PanicValue, Type, assert_that_type};
#[cfg(feature = "std")]
pub use entry::{assert_that_panic_by, assert_that_panic_by_async};
pub use failure::{AssertionFailure, Fact, FailureKind};
pub use renderer::{CustomRenderer, DebugRenderer, RenderingBudget, ValueRenderer};

/// An assertion chain over a subject of type `T`.
///
/// Ownership is stored in [`Actual<T>`](Actual), while assertion methods are selected by `T`.
/// Borrowing a sized reference therefore produces `AssertThat<Value>`. Taking ownership of the same
/// reference produces `AssertThat<&Value>`. Unsized targets such as `str` and `[T]` use
/// shared-reference subjects.
///
/// `'t` is the lifetime of a borrowed subject. `M` is [`mode::Panic`] or [`mode::Capture`]. `R` is
/// the active renderer. Renderer capabilities are required by individual methods rather than by the
/// chain, so projections can preserve `R` even when it cannot render every intermediate subject.
///
/// Derived assertions share their root's mode, failure storage, detail messages, and assertion
/// count. A failure on a child therefore behaves as a failure on the root.
pub struct AssertThat<'t, T, M: Mode, R = DebugRenderer> {
    actual: Actual<'t, T>,
    state: ChainState<'t, M, R>,
}

struct ChainState<'t, M: Mode, R> {
    /// The parent receives assertion counts and captured failures from derived assertions.
    parent: Option<&'t dyn DynAssertThat>,

    /// User provided descriptive name of the thing assertions are made on.
    subject_name: Option<String>,

    /// Rust expression written inside an `assert_that!(...)` or fluent `.must(...)` call.
    /// Typically, captured automatically by macro code.
    expression: Option<&'static str>,

    detail_messages: RefCell<Vec<String>>,

    /// Whether the source location of the assertion should be included in assertion failures. This
    /// pinpoints the location in user's code that failed. Typically turned off for internal
    /// assertr unit tests, to avoid frequent failure message churn.
    include_location: bool,

    rendering_budget: RenderingBudget,

    /// An inherited context override for panic text. `None` uses `ToHumanReadableText`. Capture
    /// mode never invokes presentation. Local adapters need not be thread-safe. `Rc` shares the
    /// adapter with derived contexts without requiring the adapter to be `Clone`.
    panic_presentation: Option<alloc::rc::Rc<failure::panic_presentation::PanicPresentation>>,

    number_of_assertions: RefCell<NumberOfAssertions>,
    failures: RefCell<Vec<AssertionFailure>>,

    mode: PhantomData<M>,

    /// `R` is intentionally not constrained by `ValueRenderer<T>` here. A chain must be able to
    /// install a renderer after construction (including for a non-`Debug` `T`), and projections
    /// preserve `R` while changing `T` even when the next assertion does not render the new
    /// subject. Each assertion method must instead declare the exact `ValueRenderer<U>`
    /// capabilities used by its failure path. Keep blanket assertion-trait impls
    /// renderer-unconstrained so one unavailable rendering capability does not hide the entire
    /// trait.
    renderer: R,
}

pub(crate) trait DynAssertThat: Fallible + WithDetail + UnwindSafe + RefUnwindSafe {
    /// Object-safe entry point for [`AssertThat::track_assertion`]'s propagation to the parent.
    fn track_assertion_on_chain(&self);
}

// Asserting unwind safety is valid for this representation: the interior mutability of an
// `AssertThat` (detail messages, assertion counter, collected failures) is only mutated in short,
// non-panicking sections, and since completion contracts are no longer enforced on drop, a chain
// observed after a caught panic cannot act on logically inconsistent state. The private parent
// trait carries the same guarantees so derived assertions retain them through its trait object.
impl<T, M: Mode, R> DynAssertThat for AssertThat<'_, T, M, R> {
    fn track_assertion_on_chain(&self) {
        self.track_assertion();
    }
}

impl<T, M: Mode, R> UnwindSafe for AssertThat<'_, T, M, R> {}
impl<T, M: Mode, R> RefUnwindSafe for AssertThat<'_, T, M, R> {}
