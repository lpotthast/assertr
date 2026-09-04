#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Allow functions named `is_*`, taking self by value instead of taking self by mutable reference
// or reference.
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
//! selected by `T`, independent of ownership. Borrowing entry points normalize sized references
//! to their pointee. Owned references and unsized targets remain reference-typed subjects.
//!
//! [`AssertThat::derive`] creates a child assertion for part of a subject. Its failures propagate
//! to the root. The [`AssertThat::satisfies`] family asserts on a child and returns the original
//! chain. Its variants cover borrowed, owned, and unsized projections.
//!
//! Panic mode stops at the first failure. Capture mode collects structured [`AssertionFailure`]
//! values within [`AssertThat::capture`] or the fluent `verify` entry points. A failure carries
//! its structured rendered [`actual`](AssertionFailure::actual) and
//! [`expected`](AssertionFailure::expected) values, the
//! [`relation`](AssertionFailure::relation) between them, further
//! [`facts`](AssertionFailure::facts), nested [`children`](AssertionFailure::children), and a
//! [`kind`](AssertionFailure::kind) as data. An
//! [`Adapter`](failure::adapter::Adapter) transforms that data, and adapters compose into typed
//! pipelines. [`ToHumanReadableText`](failure::adapter::ToHumanReadableText) produces the default
//! panic message.
//!
//! ## Custom assertions
//!
//! Define an assertion trait for your type and implement it for `AssertThat<'_, YourType, M, R>`
//! with `M: Mode` and an unconstrained `R`, so the trait is implemented no matter which renderer
//! is active. Put renderer bounds such as `R: ValueRenderer<u32>` on the individual methods that
//! need them, never on the impl: one method's needs must not hide the entire trait. Give the
//! trait an `R = DebugRenderer` default so callers can name it without the parameter. Every
//! method takes and returns `Self` so it chains like a built-in one, and carries `#[track_caller]`
//! so failures report the caller's location.
//!
//! There are two ways to decide the outcome:
//!
//! - **Composition**: delegate to existing assertions through [`AssertThat::satisfies`] and
//!   friends. Tracking, failure formatting, and capture-mode behavior come from the delegated
//!   assertions.
//! - **Leaf assertions**: call [`AssertThat::track_assertion`] first, then, when the condition
//!   does not hold, start a failure with [`AssertThat::failure`] and the [`FailureKind`] of the
//!   assertion's family. The returned [`FailureBuilder`](failure::FailureBuilder) takes the
//!   rendered [`actual`](failure::FailureBuilder::actual) value, the
//!   [`relation`](failure::FailureBuilder::relation) sentence, the
//!   [`expected`](failure::FailureBuilder::expected) or
//!   [`unexpected`](failure::FailureBuilder::unexpected) value, labeled
//!   [`fact`](failure::FailureBuilder::fact)s, and nested
//!   [`child`](failure::FailureBuilder::child) failures, and
//!   [`raise`](failure::FailureBuilder::raise) records or panics. Every diagnostic value passed to
//!   the builder implements [`IntoRendered`](renderer::IntoRendered), so adapters remain
//!   structured instead of being flattened into text.
//!
//! Render values shown by a leaf assertion through [`AssertThat::render`]. Its
//! [`value`](renderer::RenderingContext::value), [`values`](renderer::RenderingContext::values), and
//! [`borrowed_values`](renderer::RenderingContext::borrowed_values) adapters apply the active
//! renderer and rendering budget. Formatting the values with `Debug` directly bypasses both. A
//! rendered leaf always retains type metadata. Customize its hint and
//! text visibility through [`Typed::with_type_hint`](renderer::Typed::with_type_hint) and
//! [`Typed::show_type_hint`](renderer::Typed::show_type_hint).
//! Calling [`IntoRendered::into_rendered`](renderer::IntoRendered::into_rendered) consumes an
//! adapter into the same [`Rendered`](renderer::Rendered) tree stored by a failure.
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
//!     fn is_adult(self) -> Self
//!     where
//!         R: Clone + ValueRenderer<u32>;
//!
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
//! ```
//!
//! Assertr's own `*Assertions` traits are public for method discovery only. Implementing them for
//! other types is not supported; see [API stability](#api-stability).
//!
//! ## API stability
//!
//! Publicly exported items are public API and follow the usual Semantic Versioning rules unless
//! their documentation explicitly says otherwise.
//!
//! The `*Assertions` traits are public for method discovery and are not supported as downstream
//! implementation interfaces. Adding a method to one of these traits is therefore considered
//! compatible. Removing or incompatibly changing an existing method is not. Define a separate
//! assertion trait for custom types instead.
//!
//! [`__private`] is the explicitly unsupported exception. It must be publicly reachable because
//! macros expand through it, but downstream code must not name it directly and it may change in
//! any release.

extern crate alloc;
extern crate core;
extern crate self as assertr;
#[cfg(all(test, not(feature = "std")))]
extern crate std;

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

#[doc(hidden)]
pub mod __private;
pub mod actual;
mod assert_that;
pub mod assertions;
pub mod cmp;
pub mod condition;
mod conversion;
mod details;
pub mod failure;
pub mod mode;
pub mod renderer;
#[cfg(test)]
mod test_support;
mod tracking;
mod util;

#[cfg(feature = "fluent")]
pub use assertr_derive::fluent_expressions;
pub use cmp::{AssertrPartialEq, Differences, Eq, EqContext, any, eq};
pub use failure::{AssertionFailure, Fact, FailureKind};
pub use renderer::{CustomRenderer, DebugRenderer, RenderingBudget, ValueRenderer};

/// One glob import brings every assertion into scope.
///
/// ```
/// use assertr::prelude::*;
/// ```
pub mod prelude;

mod entry;
#[cfg(feature = "fluent")]
pub use entry::{IntoAssertContext, IntoOwnedAssertContext};
pub use entry::{PanicValue, Type, assert_that_type};
#[cfg(feature = "std")]
pub use entry::{assert_that_panic_by, assert_that_panic_by_async};

/// An assertion chain over a subject of type `T`.
///
/// Ownership is stored in [`Actual<T>`](Actual), while assertion methods are selected by `T`.
/// Borrowing a sized reference therefore produces `AssertThat<Value>`. Taking ownership of the
/// same reference produces `AssertThat<&Value>`. Unsized targets such as `str` and `[T]` use
/// shared-reference subjects.
///
/// `'t` is the lifetime of a borrowed subject. `M` is [`mode::Panic`] or [`mode::Capture`]. `R` is the active
/// renderer. Renderer capabilities are required by individual methods rather than by the chain,
/// so projections can preserve `R` even when it cannot render every intermediate subject.
///
/// Derived assertions share their root's mode, failure storage, detail messages, and assertion
/// count. A failure on a child therefore behaves as a failure on the root.
pub struct AssertThat<'t, T, M: Mode, R = DebugRenderer> {
    actual: Actual<'t, T>,
    state: ChainState<'t, M, R>,
}

struct ChainState<'t, M: Mode, R> {
    // Derived assertions can be created. Calling `.fail*` on them should propagate to the root assertion!
    parent: Option<&'t dyn DynAssertThat>,

    subject_name: Option<String>,
    expression: Option<&'static str>,
    detail_messages: RefCell<Vec<String>>,
    print_location: bool,
    rendering_budget: RenderingBudget,

    number_of_assertions: RefCell<NumberOfAssertions>,
    failures: RefCell<Vec<AssertionFailure>>,

    mode: PhantomData<M>,

    // `R` is intentionally not constrained by `ValueRenderer<T>` here. A chain must be able
    // to install a renderer after construction (including for a non-`Debug` `T`), and projections
    // preserve `R` while changing `T` even when the next assertion does not render the new subject.
    // Each assertion method must instead declare the exact `ValueRenderer<U>` capabilities
    // used by its failure path. Keep blanket assertion-trait impls renderer-unconstrained so one
    // unavailable rendering capability does not hide the entire trait.
    renderer: R,
}

pub(crate) trait DynAssertThat: Fallible + WithDetail + UnwindSafe + RefUnwindSafe {
    /// Object-safe entry point for [`AssertThat::track_assertion`]'s propagation to the parent.
    fn track_assertion_on_chain(&self);
}

// Asserting unwind safety is valid for this representation: the interior mutability of an
// `AssertThat` (detail messages, assertion counter, collected failures) is only mutated in
// short, non-panicking sections, and since completion contracts are no longer enforced on drop,
// a chain observed after a caught panic cannot act on logically inconsistent state. The private
// parent trait carries the same guarantees so derived assertions retain them through its trait
// object.
impl<T, M: Mode, R> DynAssertThat for AssertThat<'_, T, M, R> {
    fn track_assertion_on_chain(&self) {
        self.track_assertion();
    }
}

impl<T, M: Mode, R> UnwindSafe for AssertThat<'_, T, M, R> {}
impl<T, M: Mode, R> RefUnwindSafe for AssertThat<'_, T, M, R> {}
