//! Contracts for authoring expectations and executing their composition.
//!
//! To use built-in checks, start with the [`matchers`](mod@crate::matchers) catalog. It re-exports
//! every public expectation and groups subject-specific names into namespaces. This module owns
//! the implementation contracts and composition machinery. A matcher is an expectation used in
//! composition, not a separate trait or implementation.
//!
//! [`Expectation`] evaluates a subject once and retains its successful observation or rejection.
//! [`ExpectationDiagnostics::explain`] uses the common [`FailureBuilder`] for both a rejected
//! observation and an unmet expectation with no subject, such as a missing collection element.
//! The chain tracks assertions, preserves continuation state, and raises the resulting failures.
//!
//! Definitions can be constructed independently. Execution requires an [`AssertionContext`]
//! supplied by the library. Implementations receive that context to evaluate their children and
//! render diagnostic values under the enclosing chain's settings.
//!
//! ```
//! use assertr::{matchers::{EqualTo, each}, prelude::*};
//!
//! let expected = EqualTo::new(3);
//! assert_that!(3).apply_assertion(&expected);
//! assert_that!([3, 3]).matches(each(&expected));
//! ```

use crate::{
    AssertionFailure, DebugRenderer,
    failure::{FailureBuilder, FailureKind},
};
use alloc::vec::Vec;

mod all_of;
mod any_of;
mod anything;
mod dereferenced;
pub(crate) mod lists;
mod predicate;
mod satisfying;
#[cfg(test)]
pub(crate) mod test_support;

pub use all_of::{AllOf, all_of};
pub use any_of::{AnyOf, any_of};
pub use anything::{Anything, anything};
pub use dereferenced::{Dereferenced, dereferenced};
pub use lists::{MatcherList, predicate_list};
pub use predicate::{Predicate, predicate};
pub use satisfying::{Satisfying, satisfying};

mod context;
pub use context::AssertionContext;

/// Owned, bounded child failures from one evaluation, with their omission count.
///
/// Children already carry the complete evaluation path. The group retains no borrowed subjects,
/// expectation definitions, or guards. Composition transfers it into the common failure builder.
#[derive(Debug, Default)]
pub struct Evidence {
    pub(crate) children: Vec<AssertionFailure>,
    pub(crate) omitted: usize,
    pub(crate) path_prefix_len: usize,
}

impl Evidence {
    pub(crate) fn is_empty(&self) -> bool {
        self.children.is_empty() && self.omitted == 0
    }

    /// Attaches child failures with paths relative to the enclosing expectation's subject.
    /// The context's path prefix is removed once, preserving paths within each child failure.
    pub fn explain<T>(mut self, failure: FailureBuilder<T>) -> FailureBuilder<T> {
        for child in &mut self.children {
            child.path.drain(..self.path_prefix_len);
        }
        failure
            .children(self.children)
            .omitted_children(self.omitted)
    }
}

/// A reusable expectation that evaluates a borrowed subject once.
///
/// Evaluation does not track or raise an assertion. The executor supplies the context and decides
/// whether to continue with a success or explain a rejection. Diagnostic renderer bounds belong
/// on [`ExpectationDiagnostics`] unless evaluation itself needs those capabilities.
/// Compositions explain child rejections immediately and retain owned [`Evidence`], releasing
/// each child's observation before evaluating its siblings.
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
}

/// Builds every diagnostic for an [`Expectation`] through the common failure pipeline.
///
/// `Some((actual, rejection))` explains the original rejected observation. `None` describes an
/// unmet expectation for which there is no subject to evaluate. It never represents a successful
/// evaluation. Both cases use the same operand roles, structured fields, renderer, and budget.
/// Each definition writes its own relations. No generic negation or repeated observation occurs.
pub trait ExpectationDiagnostics<T: ?Sized, R = DebugRenderer>: Expectation<T, R> {
    /// The failure family used for both ordinary and nested diagnostics.
    const KIND: FailureKind;

    /// Whether composition contributes this definition's children directly, applying the current
    /// context's path to their relative paths.
    /// Ordinary chain execution still retains the definition's enclosing failure.
    const FLATTEN: bool = false;

    /// Explains a rejected observation or describes a missing expected subject.
    ///
    /// Render shared operands once and reuse retained views from `rejected` when present. Do not
    /// evaluate the subject again, repeat user conversions, or retain guards after returning.
    /// The executor supplies the builder and raises or retains it after this method returns.
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target>;
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

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        (**self).evaluate(actual, context)
    }
}

impl<T: ?Sized, R, D: ExpectationDiagnostics<T, R> + ?Sized> ExpectationDiagnostics<T, R> for &D {
    const KIND: FailureKind = D::KIND;
    const FLATTEN: bool = D::FLATTEN;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        (**self).explain(rejected, failure, context)
    }
}
