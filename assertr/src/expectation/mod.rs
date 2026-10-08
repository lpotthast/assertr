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
//! assert_that!(3).apply_assertion(&expected);
//! assert_that!([3, 3]).matches(each(&expected));
//! ```
//!
//! The [custom assertions guide](crate#custom-assertions) walks through a complete implementation.

use crate::{
    AssertionFailure, DebugRenderer,
    failure::{FailureBuilder, FailureKind},
};
use alloc::vec::Vec;

/// Declares the associated items of a transparent composition: no success observation, owned
/// child [`Evidence`] as the rejection, and flattening into the receiving context.
macro_rules! composite_items {
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
        const FLATTEN: bool = true;
    };
}
pub(crate) use composite_items;

mod all_of;
mod any_of;
mod anything;
mod dereferenced;
pub(crate) mod field;
pub(crate) mod lists;
mod predicate;
mod satisfying;
#[cfg(test)]
pub(crate) mod test_support;

pub use all_of::{AllOf, all_of};
pub use any_of::{AnyOf, any_of};
pub use anything::{Anything, anything};
pub use dereferenced::{Dereferenced, dereferenced};
pub use field::{Field, field};
pub use lists::{MatcherList, predicate_list};
pub use predicate::{Predicate, predicate};
pub use satisfying::{Satisfying, satisfying};

pub(crate) mod context;
pub use context::AssertionContext;

/// Owned, bounded child failures from one evaluation, with their omission count.
///
/// Child paths are relative to the scope that produced them. The group retains no borrowed
/// subjects, expectation definitions, or guards. Composition transfers it into the common failure
/// builder.
#[derive(Debug, Default)]
pub struct Evidence {
    pub(crate) children: Vec<AssertionFailure>,
    pub(crate) omitted: usize,
}

impl Evidence {
    /// Attaches the child failures and their omission count to the enclosing expectation's failure.
    pub fn explain(self, failure: FailureBuilder) -> FailureBuilder {
        failure
            .children(self.children)
            .omitted_children(self.omitted)
    }
}

/// A reusable expectation that evaluates a borrowed subject once and explains its rejection.
///
/// Evaluation does not track or raise an assertion. The executor supplies the context and decides
/// whether to continue with a success or explain a rejection. Compositions explain child
/// rejections immediately and retain owned [`Evidence`], releasing each child's observation
/// before evaluating its siblings.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an expectation for `{T}`",
    label = "cannot check `{T}` with `{Self}`",
    note = "to compare with a plain value, wrap it in `eq(value)`. Other matchers are in `assertr::matchers`"
)]
pub trait Expectation<T: ?Sized, R = DebugRenderer> {
    /// The original successful observation. Use `()` when no witness is needed.
    type Success<'a>
    where
        Self: 'a,
        T: 'a;

    /// The original rejection, which may borrow the subject or definition.
    type Rejection<'a>
    where
        Self: 'a,
        T: 'a;

    /// The failure family used for both ordinary and nested diagnostics. Custom checks of a domain
    /// property can keep the default.
    const KIND: FailureKind = FailureKind::Predicate;

    /// Whether composition contributes this definition's children directly, applying the current
    /// context's path to their relative paths.
    ///
    /// In composition, a flattening definition contributes only its children and omission count.
    /// Its own relation, actual, expected, unexpected, and facts are not shown. Use it only for
    /// transparent groups whose children carry the complete explanation. Library probes never call
    /// [`explain`](Self::explain). Ordinary chain execution still retains the definition's
    /// enclosing failure.
    const FLATTEN: bool = false;

    /// Evaluates once with the executor's context. User panics propagate.
    ///
    /// # Errors
    /// Returns the original rejection when the expectation is not satisfied. Diagnostic budgets
    /// and probes must never change whether evaluation succeeds.
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>>;

    /// Explains a rejected observation or describes a missing expected subject.
    ///
    /// `Some((actual, rejection))` explains the original rejected observation. `None` describes an
    /// unmet expectation for which there is no subject to evaluate. It never represents a
    /// successful evaluation. Both cases use the same operand roles, structured fields, renderer,
    /// and budget. Each definition writes its own relations. No generic negation occurs.
    ///
    /// Populate and return the supplied structured builder. Do not track or raise an assertion.
    /// Render shared operands through [`AssertionContext::render`] and reuse retained views from
    /// `rejected` when present. Bulk expected lists may be accessed and their operands borrowed
    /// repeatedly if they describe the same logical list and comparison values. Access counts
    /// and interleaving with comparisons are unspecified. Constructors store these inputs without
    /// accessing their views. Library-controlled access occurs after assertion tracking.
    /// Prepare stateful inputs before the assertion or retain their observation in a custom
    /// expectation. Scalar borrowing, matcher, callback, guard, and identity contracts are
    /// unchanged. Do not repeat comparisons, searches, lookups, callbacks, consumption, or
    /// other observations, or retain guards after returning. The chain executor raises the
    /// completed failure. Child contexts instead build and retain it as evidence for the
    /// enclosing assertion.
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
