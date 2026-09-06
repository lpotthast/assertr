use super::AssertrMatcher;
use crate::{
    AssertThat, AssertionFailure, DebugRenderer, Mode, RenderingBudget,
    failure::{FailureBuilder, FailureKind, PathSegment, adapter::ToHumanReadableText},
    renderer::{IntoRendered, Rendered, RenderingContext, RenderingOrder},
};
use alloc::{borrow::Cow, vec::Vec};
use core::ops::{Deref, DerefMut};

/// Owned constraint data, independent of whether a subject satisfies it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    /// Lowercase relation without embedded values.
    pub relation: Cow<'static, str>,
    /// Rendered expected operand, if any.
    pub expected: Option<Rendered>,
    /// Composed constraints, in declaration order.
    pub children: Vec<Description>,
    /// Number of constraint branches omitted by the rendering budget.
    pub omitted_children: usize,
}

impl Description {
    /// Describes a relation.
    pub fn new(relation: impl Into<Cow<'static, str>>) -> Self {
        Self {
            relation: relation.into(),
            expected: None,
            children: Vec::new(),
            omitted_children: 0,
        }
    }

    /// Attaches an operand rendered through the match context.
    #[must_use]
    pub fn expected(mut self, value: impl IntoRendered) -> Self {
        self.expected = Some(value.into_rendered());
        self
    }

    /// Attaches composed descriptions.
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children.extend(children);
        self
    }

    /// Records branches that were not described because of the rendering budget.
    #[must_use]
    pub fn omitted_children(mut self, count: usize) -> Self {
        self.omitted_children = count;
        self
    }
}

/// The truth result, independent of retained diagnostics or rendering limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchResult {
    /// Whether the positive constraint matched.
    pub matched: bool,
}

impl MatchResult {
    /// Constructs a truth result. Record evidence separately on the context.
    #[must_use]
    pub const fn new(matched: bool) -> Self {
        Self { matched }
    }
}

/// One detached mismatch, using the ordinary assertion failure model.
#[derive(Debug)]
pub struct Mismatch(AssertionFailure);

impl From<AssertionFailure> for Mismatch {
    fn from(value: AssertionFailure) -> Self {
        Self(value)
    }
}

impl From<Mismatch> for AssertionFailure {
    fn from(value: Mismatch) -> Self {
        value.0
    }
}

/// Rendering, requested polarity, relative paths, and isolated matcher evidence.
///
/// Evaluation never raises a root assertion. A probe suppresses built-in diagnostic rendering, but
/// cannot undo side effects or rendering performed by downstream code or assertion closures.
pub struct MatchContext<'r, R = DebugRenderer> {
    rendering: RenderingContext<'r, R>,
    positive: bool,
    pub(crate) include_location: bool,
    diagnostic: bool,
    limit: usize,
    evidence_order: RenderingOrder,
    path: Vec<PathSegment>,
    pub(crate) evidence: Vec<AssertionFailure>,
    pub(crate) omitted: usize,
}

impl<'r, R> MatchContext<'r, R> {
    /// Creates a positive diagnostic evaluation with explicit renderer and budget.
    pub fn new(renderer: &'r R, budget: RenderingBudget) -> Self {
        Self::from_rendering(RenderingContext::new(renderer, budget))
    }

    /// Uses an assertion's active rendering context.
    #[must_use]
    pub fn from_rendering(rendering: RenderingContext<'r, R>) -> Self {
        Self {
            limit: rendering.max_items(),
            evidence_order: RenderingOrder::PreserveIteration,
            rendering,
            positive: true,
            include_location: true,
            diagnostic: true,
            path: Vec::new(),
            evidence: Vec::new(),
            omitted: 0,
        }
    }

    pub(crate) fn for_assertion<T, M>(assertion: &'r AssertThat<'_, T, M, R>) -> Self
    where
        M: Mode,
    {
        let mut context = Self::from_rendering(assertion.render());
        context.include_location = assertion.state.include_location;
        context
    }

    /// Returns the active rendering context.
    #[must_use]
    pub fn render(&self) -> RenderingContext<'r, R> {
        self.rendering
    }

    /// Whether evidence is requested. Built-in leaves do not render during probes.
    #[must_use]
    pub fn is_diagnostic(&self) -> bool {
        self.diagnostic
            && (self.evidence.len() < self.limit
                || ((self.evidence_order == RenderingOrder::SortByRenderedText) && self.limit > 0))
    }

    /// Requested truth. Negative matching asks for evidence of an unexpected success.
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.positive
    }

    /// Sets the expected truth for this evaluation.
    pub fn set_positive(&mut self, positive: bool) {
        self.positive = positive;
    }

    /// Records detached evidence, preserving structured values and inner assertion metadata.
    pub fn record(&mut self, mismatch: impl Into<Mismatch>) {
        if !self.diagnostic {
            return;
        }
        if !self.is_diagnostic() {
            self.omitted += 1;
            return;
        }
        let mut failure = mismatch.into().0;
        let mut path = self.path.clone();
        path.append(&mut failure.path);
        failure.path = path;
        self.retain(failure);
    }

    /// Records a leaf constraint when its truth contradicts the requested polarity.
    pub fn outcome(
        &mut self,
        matched: bool,
        description: impl FnOnce(&Self) -> Description,
    ) -> MatchResult {
        if matched != self.positive {
            if self.is_diagnostic() {
                self.record(
                    FailureBuilder::detached::<()>(FailureKind::Matching)
                        .relation(if matched {
                            "satisfies the constraint unexpectedly"
                        } else {
                            "does not satisfy the constraint"
                        })
                        .constraint(description(self))
                        .build(),
                );
            } else if self.diagnostic {
                self.omitted += 1;
            }
        }
        MatchResult::new(matched)
    }

    /// Creates an isolated fork. Dropping it discards its evidence. Only its own parent can receive
    /// it.
    pub fn fork(&mut self) -> MatchFork<'_, 'r, R> {
        let context = self.isolated();
        MatchFork {
            parent: self,
            context,
        }
    }

    /// Evaluates truth in isolation without committing evidence.
    pub fn probe<A: ?Sized, M>(&self, actual: &A, matcher: &M) -> bool
    where
        M: AssertrMatcher<A, R>,
    {
        let mut context = self.isolated();
        context.diagnostic = false;
        matcher.evaluate(actual, &mut context).matched
    }

    /// Evaluates within one relative path segment and commits that scope's evidence once.
    pub fn scoped<T>(&mut self, path: PathSegment, f: impl FnOnce(&mut Self) -> T) -> T {
        let mut fork = self.fork();
        fork.path.push(path);
        let result = f(&mut fork);
        fork.commit();
        result
    }

    /// Consumes retained evidence. The omitted count remains separately inspectable before
    /// consumption.
    #[must_use]
    pub fn into_failures(self) -> Vec<AssertionFailure> {
        self.evidence
    }

    /// Number of omitted diagnostic children.
    #[must_use]
    pub fn omitted_children(&self) -> usize {
        self.omitted
    }

    pub(crate) fn isolated(&self) -> Self {
        Self {
            // Sorted branches compete for the same retained slots, even after the group fills.
            limit: if self.evidence_order == RenderingOrder::SortByRenderedText {
                self.limit
            } else {
                self.limit.saturating_sub(self.evidence.len())
            },
            evidence_order: self.evidence_order,
            rendering: self.rendering,
            positive: self.positive,
            include_location: self.include_location,
            diagnostic: self.diagnostic,
            path: self.path.clone(),
            evidence: Vec::new(),
            omitted: 0,
        }
    }

    pub(crate) fn isolated_for_order(&self, order: RenderingOrder) -> Self {
        let mut context = self.isolated();
        // Descendants contribute to the same evidence group and must retain by its order.
        if order == RenderingOrder::SortByRenderedText {
            context.evidence_order = order;
        }
        context
    }

    fn retain(&mut self, failure: AssertionFailure) {
        if self.evidence_order == RenderingOrder::SortByRenderedText && self.limit > 0 {
            let render = ToHumanReadableText::render_child;
            let text = render(&failure);
            let index = self
                .evidence
                .partition_point(|retained| render(retained) <= text);
            if self.evidence.len() == self.limit {
                self.omitted += 1;
                if index == self.limit {
                    return;
                }
                self.evidence.pop();
            }
            self.evidence.insert(index, failure);
        } else if self.evidence.len() < self.limit {
            self.evidence.push(failure);
        } else {
            self.omitted += 1;
        }
    }

    pub(crate) fn record_group(&mut self, mut failure: AssertionFailure) {
        for child in &mut failure.children {
            if child.path.starts_with(&self.path) {
                child.path.drain(..self.path.len());
            }
        }
        self.record(failure);
    }

    pub(crate) fn append(&mut self, mut other: Self) {
        self.omitted += other.omitted;
        for failure in other.evidence.drain(..) {
            self.retain(failure);
        }
    }
}

impl Default for MatchContext<'static, DebugRenderer> {
    fn default() -> Self {
        Self::new(&DebugRenderer, RenderingBudget::default())
    }
}

/// An evidence transaction tied to its parent by an exclusive borrow. `commit` consumes the
/// transaction, so it cannot be committed twice or to another context.
pub struct MatchFork<'a, 'r, R> {
    parent: &'a mut MatchContext<'r, R>,
    context: MatchContext<'r, R>,
}

impl<R> MatchFork<'_, '_, R> {
    /// Commits this fork to its own parent exactly once.
    pub fn commit(self) {
        self.parent.append(self.context);
    }
}

impl<'r, R> Deref for MatchFork<'_, 'r, R> {
    type Target = MatchContext<'r, R>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<R> DerefMut for MatchFork<'_, '_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

#[cfg(test)]
mod tests {
    use super::MatchContext;
    use crate::{matchers::predicate, prelude::*};
    use core::cell::Cell;

    mod probe {
        use super::*;

        #[test]
        fn evaluates_once_without_retaining_evidence() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let context = MatchContext::default();

            assert_that!(context.probe(&1, &matcher)).is_false();
            assert_that!(context.into_failures()).is_empty();
            assert_that!(calls.get()).is_equal_to(1);
        }
    }

    mod fork {
        use super::*;

        #[test]
        fn dropping_discards_evidence_without_replaying_user_code() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let mut context = MatchContext::default();
            {
                let mut fork = context.fork();
                assert_that!(matcher.evaluate(&1, &mut fork).matched).is_false();
            }

            assert_that!(context.into_failures()).is_empty();
            assert_that!(calls.get()).is_equal_to(1);
        }

        #[test]
        fn committing_retains_evidence_without_replaying_user_code() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let mut context = MatchContext::default();
            {
                let mut fork = context.fork();
                matcher.evaluate(&1, &mut fork);
                fork.commit();
            }

            assert_that!(context.into_failures()).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
        }
    }
}
