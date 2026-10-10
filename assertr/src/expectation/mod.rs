//! The [`Expectation`] trait and the building blocks for combining expectations.
//!
//! To use built-in checks, start with the [`matchers`](mod@crate::matchers) catalog, which
//! re-exports every public expectation. This module is for writing your own and for the machinery
//! that combines them. A matcher is just an expectation used in composition.
//!
//! An [`Expectation`] inspects a subject once in [`Expectation::evaluate`] and keeps what it
//! observed, either a successful value or the reason for rejection. On failure,
//! [`Expectation::explain`] turns that rejection into a report through a [`FailureBuilder`]. It is
//! also called without a subject when the subject is missing entirely, such as an expected element
//! that a collection lacks. The chain counts assertions and raises failures. The expectation does
//! neither.
//!
//! Expectations are plain values that can be built anywhere. Running one requires an
//! [`AssertionContext`], which assertr supplies. Implementations use it to run child expectations
//! and to render values with the chain's settings.
//!
//! ```
//! use assertr::{matchers::{EqualTo, each}, prelude::*};
//!
//! let expected = EqualTo::new(3);
//! assert_that!(3).matches(&expected);
//! assert_that!([3, 3]).matches(each(&expected));
//! ```
//!
//! The [custom assertions guide](crate#custom-assertions) walks through a complete implementation.
//!
//! An expectation that runs a list of child expectations, like
//! [`all_of`](crate::matchers::all_of), accepts any [`MatcherList`]: a
//! [`matchers!`](crate::matchers!) list of mixed matcher types, or an array, slice, or vector of
//! one matcher type. [`EntryMatcherList`] is the keyed form behind
//! [`entries_are!`](crate::entries_are). Both traits are sealed. Name them in bounds or as
//! `impl MatcherList<T>` return types:
//!
//! ```
//! use assertr::{expectation::MatcherList, matchers::{all_of, eq, gt}, prelude::*};
//!
//! fn small_positive() -> impl MatcherList<i32> {
//!     matchers![gt(0), eq(2)]
//! }
//!
//! assert_that!(2).matches(all_of(small_positive()));
//! ```

use alloc::vec::Vec;

use crate::{
    failure::{AssertionFailure, FailureBuilder, FailureKind},
    renderer::DebugRenderer,
};

/// Declares the associated items of a composition that keeps no success observation and rejects
/// with owned child [`Evidence`].
macro_rules! evidence_items {
    ($subject:ty) => {
        type Success<'a>
            = ()
        where
            Self: 'a,
            $subject: 'a;
        type Rejection<'a>
            = $crate::expectation::Evidence
        where
            Self: 'a,
            $subject: 'a;
    };
}
pub(crate) use evidence_items;

/// Declares the associated items of a transparent composition: the [`evidence_items!`] plus
/// flattening into the receiving context.
macro_rules! composite_items {
    ($subject:ty) => {
        $crate::expectation::evidence_items!($subject);
        const FLATTEN: bool = true;
    };
}
pub(crate) use composite_items;

/// Converts the outcome of a check that retains no observation into an evaluation result.
pub(crate) const fn passed(condition: bool) -> Result<(), ()> {
    if condition { Ok(()) } else { Err(()) }
}

pub(crate) mod all_of;
pub(crate) mod any_of;
pub(crate) mod anything;
pub(crate) mod dereferenced;
pub(crate) mod field;
pub(crate) mod lists;
pub(crate) mod predicate;
pub(crate) mod satisfying;

pub(crate) mod context;
pub use context::AssertionContext;
pub use lists::MatcherList;

pub use crate::assertions::map::EntryMatcherList;

/// Failures collected while running child expectations, plus a count of those the rendering
/// budget left out.
///
/// Get it from [`AssertionContext::into_evidence`] and attach it to a failure with
/// [`FailureBuilder::evidence`]. It owns its failures and borrows nothing, so it works as the
/// [`Rejection`](Expectation::Rejection) of a combining expectation.
#[derive(Debug)]
pub struct Evidence {
    pub(crate) children: Vec<AssertionFailure>,
    pub(crate) omitted: usize,
}

/// A reusable check with its own failure report.
///
/// [`evaluate`](Self::evaluate) checks a borrowed subject once and keeps what it observed. If the
/// check fails, [`explain`](Self::explain) turns that observation into a report. The chain counts
/// the assertion and raises the failure, so neither method does either.
///
/// The [custom assertions guide](crate#implement-an-expectation) walks through an implementation.
/// [`AssertionContext`] shows how to combine expectations.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an expectation for `{T}`",
    label = "cannot check `{T}` with `{Self}`",
    note = "to compare with a plain value, wrap it in `eq(value)`. Other matchers are in `assertr::matchers`"
)]
pub trait Expectation<T: ?Sized, R = DebugRenderer> {
    /// What a passing check observed, such as a parsed value or a lock guard. Use `()` when there
    /// is nothing to keep. [`AssertThat::test_assertion`](crate::AssertThat::test_assertion)
    /// returns it.
    type Success<'a>
    where
        Self: 'a,
        T: 'a;

    /// What a failing check observed, passed on to [`explain`](Self::explain). It may borrow the
    /// subject or the expectation.
    type Rejection<'a>
    where
        Self: 'a,
        T: 'a;

    /// The [`FailureKind`] of every failure this expectation reports. Domain checks can keep the
    /// default.
    const KIND: FailureKind = FailureKind::Predicate;

    /// Whether a nested failure of this expectation is replaced by its children.
    ///
    /// Set it for pure grouping expectations, like [`all_of`](crate::matchers::all_of), whose
    /// children already explain everything. Inside another expectation, only the children and
    /// their omission count are reported, not this expectation's relation, values, or facts. Run
    /// directly on a chain, the expectation still reports its own failure. A rejection explained
    /// without children or omissions has nothing to flatten and is reported whole, so it is never
    /// lost. Attach the evidence of the children with
    /// [`FailureBuilder::evidence`](crate::failure::FailureBuilder::evidence).
    const FLATTEN: bool = false;

    /// Checks `actual` and returns what was observed.
    ///
    /// Return `Ok` with the [`Success`](Self::Success) value, or `Err` with what
    /// [`explain`](Self::explain) needs to describe the failure. Use `context` to run child
    /// expectations. The result must not depend on the rendering budget or on whether this is a
    /// [`probe`](AssertionContext::probe). Panics in user code are not caught.
    ///
    /// # Errors
    ///
    /// Returns the rejection when the check fails.
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>>;

    /// Fills in the failure report.
    ///
    /// `rejected` is `Some((actual, rejection))` after a failed check, with the rejection returned
    /// by [`evaluate`](Self::evaluate). It is `None` when there is no subject at all, for example
    /// an expected element missing from a collection. Then describe what was expected. `None`
    /// never means the check passed.
    ///
    /// Render every value through [`AssertionContext::render`]. Report what the rejection kept
    /// instead of checking again: never repeat comparisons, lookups, callbacks, or iterator
    /// consumption. Expected lists may be read again. Release any guard held by the rejection
    /// before returning. Return the populated builder. The chain raises it.
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder;
}

impl<T: ?Sized, R, D: Expectation<T, R> + ?Sized> Expectation<T, R> for &D {
    type Success<'a>
        = D::Success<'a>
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = D::Rejection<'a>
    where
        Self: 'a,
        T: 'a;

    const KIND: FailureKind = D::KIND;
    const FLATTEN: bool = D::FLATTEN;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        (**self).evaluate(actual, context)
    }

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        (**self).explain(rejected, failure, context)
    }
}
