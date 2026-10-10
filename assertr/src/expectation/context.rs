use alloc::string::String;
use core::fmt;

#[cfg(test)]
use crate::renderer::RenderingBudget;
use crate::{
    AssertThat,
    actual::Actual,
    expectation::{Evidence, Expectation},
    failure::{AssertionFailure, FailureBuilder, FailureKind, PathSegment, report},
    mode::Capture,
    renderer::{DebugRenderer, RenderingContext, RenderingOrder},
    util::selection::{Keyed, Smallest},
};

/// The settings and evidence collector an [`Expectation`] runs with.
///
/// assertr passes a context to [`Expectation::evaluate`] and [`Expectation::explain`]. There is no
/// public constructor. Run an expectation through an assertion chain, for example with
/// [`AssertThat::matches`] or `.matches(..)`.
///
/// # Checking a single value
///
/// Most expectations need only two methods:
///
/// - [`render`](Self::render) renders values for the failure report with the chain's renderer and
///   rendering budget.
/// - [`is_diagnostic`](Self::is_diagnostic) tells whether evidence is wanted right now. Skip
///   optional work when it returns `false`.
///
/// # Combining expectations
///
/// An expectation that runs other expectations, like [`all_of`](crate::matchers::all_of), collects
/// their failures as evidence:
///
/// 1. Start a fresh collector with [`isolated`](Self::isolated).
/// 2. Run each child with [`evaluate`](Self::evaluate). It returns whether the child passed and
///    records the child's failure if not. Wrap it in [`scoped`](Self::scoped) to report the failure
///    at a field, index, or key.
/// 3. On rejection, return the collected failures from [`into_evidence`](Self::into_evidence) and
///    attach them in `explain` with [`FailureBuilder::evidence`].
///
/// To only find out whether a child passes, for example while searching for a matching element,
/// use [`probe`](Self::probe). It records nothing and renders nothing.
///
/// This expectation applies one matcher to both bounds of a range and reports each failing bound
/// at its field:
///
/// ```
/// use assertr::expectation::Evidence;
/// use assertr::failure::{FailureBuilder, PathSegment};
/// use assertr::matchers::ge;
/// use assertr::prelude::*;
/// use assertr::{expectation::AssertionContext, expectation::Expectation, failure::FailureKind};
///
/// struct Bounds {
///     start: u32,
///     end: u32,
/// }
///
/// struct BothBounds<M>(M);
///
/// impl<M: Expectation<u32, R>, R> Expectation<Bounds, R> for BothBounds<M> {
///     type Success<'a> = () where Self: 'a;
///     type Rejection<'a> = Evidence where Self: 'a;
///
///     const KIND: FailureKind = FailureKind::Matching;
///
///     fn evaluate(&self, bounds: &Bounds, context: &AssertionContext<'_, R>) -> Result<(), Evidence> {
///         let mut children = context.isolated();
///         let start = children.scoped(PathSegment::Field("start"), |children| {
///             children.evaluate(&bounds.start, &self.0)
///         });
///         let end = children.scoped(PathSegment::Field("end"), |children| {
///             children.evaluate(&bounds.end, &self.0)
///         });
///         if start && end { Ok(()) } else { Err(children.into_evidence()) }
///     }
///
///     fn explain(
///         &self,
///         rejected: Option<(&Bounds, Evidence)>,
///         failure: FailureBuilder,
///         _: &AssertionContext<'_, R>,
///     ) -> FailureBuilder {
///         match rejected {
///             Some((_, evidence)) => failure.relation("has a failing bound").evidence(evidence),
///             None => failure.relation("has matching bounds"),
///         }
///     }
/// }
///
/// assert_that!(Bounds { start: 1, end: 9 }).matches(BothBounds(ge(1)));
///
/// let failures = assert_that!(Bounds { start: 0, end: 9 })
///     .with_location(false)
///     .capture(|it| it.matches(BothBounds(ge(1))));
/// assert_that!(failures[0].children).has_length(1);
/// assert_that!(failures[0].to_string()).contains("At .start:");
/// ```
///
/// Children may run user code with side effects. A probe or a full budget suppresses evidence,
/// but it cannot undo those effects.
///
/// ```compile_fail
/// use assertr::{expectation::AssertionContext, renderer::DebugRenderer, renderer::RenderingBudget};
/// let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
/// ```
pub struct AssertionContext<'r, R = DebugRenderer> {
    rendering: RenderingContext<'r, R>,
    include_location: bool,
    diagnostic: bool,
    order: EvidenceOrder,
    // Paths are relative to this scope. `scoped` prepends its segment when committing evidence.
    children: Smallest<Keyed<(Option<String>, usize), AssertionFailure>>,
    omitted: usize,
}

impl<R> fmt::Debug for AssertionContext<'_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssertionContext")
            .field("budget", &self.rendering.budget())
            .field("include_location", &self.include_location)
            .field("diagnostic", &self.diagnostic)
            .field("retained", &self.children.len())
            .field("omitted", &self.omitted)
            .finish_non_exhaustive()
    }
}

/// How a scope ranks the evidence it retains.
///
/// Sorted scopes rank complete child reports, including paths. Retained paths are relative to
/// their scope, but ranking must not depend on that: a scope below a path segment ranks each
/// report as if rendered beneath that (shared) prefix, so retention agrees with every enclosing
/// scope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct EvidenceOrder {
    sorted: bool,
    nested: bool,
}

impl EvidenceOrder {
    /// The sort key text for a retained failure, or `None` when iteration order is preserved.
    pub(crate) fn text(self, failure: &mut AssertionFailure) -> Option<String> {
        if !self.sorted {
            return None;
        }
        if !self.nested {
            return Some(report::child_text(failure));
        }
        // Any common prefix ranks the remaining text identically, so a placeholder suffices.
        failure.path.insert(0, PathSegment::Field(""));
        let text = report::child_text(failure);
        failure.path.remove(0);
        Some(text)
    }
}

/// Wraps the description of a constraint that has no subject to show, such as an expected element
/// that is missing, as a "does not satisfy the constraint" failure.
pub(crate) fn unsatisfied(constraint: AssertionFailure) -> AssertionFailure {
    FailureBuilder::new::<()>(FailureKind::Matching)
        .relation("does not satisfy the constraint")
        .constraint(constraint)
        .build()
}

impl<'r, R> AssertionContext<'r, R> {
    /// Creates a diagnostic evaluation with explicit renderer and budget.
    #[cfg(test)]
    pub(crate) fn new(renderer: &'r R, budget: RenderingBudget) -> Self {
        Self::from_rendering(RenderingContext::new(renderer, budget), true)
    }

    /// Creates an evaluation context from rendering settings and location policy.
    #[must_use]
    pub(crate) fn from_rendering(
        rendering: RenderingContext<'r, R>,
        include_location: bool,
    ) -> Self {
        Self {
            order: EvidenceOrder::default(),
            children: Smallest::new(rendering.max_items()),
            rendering,
            include_location,
            diagnostic: true,
            omitted: 0,
        }
    }

    /// Whether captured assertion callbacks retain caller locations.
    #[cfg(test)]
    pub(crate) fn include_location(&self) -> bool {
        self.include_location
    }

    pub(crate) fn with_diagnostics(mut self, enabled: bool) -> Self {
        self.diagnostic = enabled;
        self
    }

    // A probe never reaches root explanation. Exhausted child evidence or a zero-item budget
    // can still require root diagnostics, so leaves must not conflate those cases.
    pub(crate) fn is_probe(&self) -> bool {
        !self.diagnostic
    }

    /// Returns the failures this collector recorded, to be attached in
    /// [`Expectation::explain`] with [`FailureBuilder::evidence`].
    ///
    /// Paths are relative to this collector. A [`scoped`](Self::scoped) segment added by an
    /// enclosing collector is applied when it takes over the evidence.
    #[must_use]
    pub fn into_evidence(self) -> Evidence {
        Evidence {
            children: self.children.into_values(),
            omitted: self.omitted,
        }
    }

    /// Completes a composition using its truth result, independently of retained evidence.
    /// A rejection with no retained or omitted failures gets the caller's fallback constraint.
    /// The fallback is lazy and obeys the same budget and probe rules as other outcomes.
    pub(crate) fn finish(
        mut self,
        matched: bool,
        fallback: impl FnOnce(&Self) -> AssertionFailure,
    ) -> Result<(), Evidence> {
        if self.complete(matched, fallback) {
            Ok(())
        } else {
            Err(self.into_evidence())
        }
    }

    /// Applies [`Self::finish`]'s fallback rule within a scope that stays open, returning
    /// `matched`.
    pub(crate) fn complete(
        &mut self,
        matched: bool,
        fallback: impl FnOnce(&Self) -> AssertionFailure,
    ) -> bool {
        if !matched && self.children.len() == 0 && self.omitted == 0 {
            self.record(|context| unsatisfied(fallback(context)));
        }
        matched
    }

    /// Runs a child expectation and returns whether it passed.
    ///
    /// A failing child is explained right away and its failure recorded here, unless this is a
    /// probe or the rendering budget is used up. Either way, the child's observation is dropped
    /// before this returns, so a guard it held is released before the next child runs.
    #[must_use = "a failing child must make the combined check fail"]
    pub fn evaluate<A: ?Sized, D: Expectation<A, R> + ?Sized>(
        &mut self,
        actual: &A,
        definition: &D,
    ) -> bool {
        let Err(rejection) = definition.evaluate(actual, self) else {
            return true;
        };
        // Transparent groups have already applied their budgets and rendered their children.
        // Even a zero-capacity group must transfer its omitted count. Probes never explain.
        if self.diagnostic && (D::FLATTEN || self.is_diagnostic()) {
            let failure = FailureBuilder::new::<A>(D::KIND);
            let failure = definition
                .explain(Some((actual, rejection)), failure, self)
                .build();
            if D::FLATTEN {
                self.append(Evidence {
                    children: failure.children,
                    omitted: failure.omitted_children,
                });
            } else {
                self.retain(failure);
            }
        } else {
            drop(rejection);
            self.omitted += usize::from(self.diagnostic);
        }
        false
    }

    /// Describes `definition` for a subject that does not exist, such as an expected element that
    /// a collection lacks.
    ///
    /// The description is built by [`Expectation::explain`] without a rejection and never
    /// evaluates `definition`, so predicates and callbacks do not run. Use it when a combining
    /// expectation reports a child that had nothing to check, for example by attaching the result
    /// with [`FailureBuilder::constraint`] or [`FailureBuilder::child`].
    ///
    /// ```
    /// use assertr::{
    ///     expectation::{AssertionContext, Expectation},
    ///     failure::{FailureBuilder, FailureKind},
    ///     matchers::eq,
    ///     prelude::*,
    /// };
    ///
    /// /// Expects `Some` value matching `M`.
    /// struct HoldsMatching<M>(M);
    ///
    /// impl<M: Expectation<i32, R>, R> Expectation<Option<i32>, R> for HoldsMatching<M> {
    ///     type Success<'a> = () where Self: 'a;
    ///     type Rejection<'a> = () where Self: 'a;
    ///
    ///     fn evaluate(&self, actual: &Option<i32>, context: &AssertionContext<'_, R>) -> Result<(), ()> {
    ///         match actual {
    ///             Some(value) if context.probe(value, &self.0) => Ok(()),
    ///             _ => Err(()),
    ///         }
    ///     }
    ///
    ///     fn explain(
    ///         &self,
    ///         _: Option<(&Option<i32>, ())>,
    ///         failure: FailureBuilder,
    ///         context: &AssertionContext<'_, R>,
    ///     ) -> FailureBuilder {
    ///         failure
    ///             .relation("does not hold a matching value")
    ///             .constraint(context.describe::<i32, _>(&self.0))
    ///     }
    /// }
    ///
    /// let failures = assert_that!(None::<i32>).capture(|it| it.matches(HoldsMatching(eq(2))));
    /// let constraint = failures[0].constraint.as_ref().unwrap();
    /// assert_that!(constraint.expected.as_ref().map(ToString::to_string)).is_equal_to(Some("2".to_owned()));
    /// ```
    #[must_use]
    pub fn describe<A: ?Sized, D: Expectation<A, R> + ?Sized>(
        &self,
        definition: &D,
    ) -> AssertionFailure {
        definition
            .explain(None, FailureBuilder::new::<A>(D::KIND), self)
            .build()
    }

    pub(crate) fn describe_list<A: ?Sized, L>(
        &self,
        list: &L,
        failure: FailureBuilder,
    ) -> FailureBuilder
    where
        L: crate::expectation::MatcherList<A, R>,
    {
        let retained = list.len().min(self.rendering.max_items());
        failure
            .omitted_children(list.len() - retained)
            .children((0..retained).map(|index| list.describe_at(index, self)))
    }

    /// Renders values with the chain's renderer and rendering budget. Use it for every value in a
    /// failure report.
    #[must_use]
    pub fn render(&self) -> RenderingContext<'r, R> {
        self.rendering
    }

    /// Whether recorded failures would be kept.
    ///
    /// Returns `false` during a [`probe`](Self::probe) and once the rendering budget allows no
    /// more items. Use it to skip building optional evidence. Never let it change whether a check
    /// passes.
    #[must_use]
    pub fn is_diagnostic(&self) -> bool {
        self.diagnostic && self.child_limit() > 0
    }

    /// Records the failure built by `failure`. The closure runs only when this collector keeps the
    /// failure, so no work is spent on failures that the budget or a probe would discard.
    pub(crate) fn record(&mut self, failure: impl FnOnce(&Self) -> AssertionFailure) {
        if !self.diagnostic {
            return;
        }
        if self.is_diagnostic() {
            let failure = failure(self);
            self.retain(failure);
        } else {
            self.omitted += 1;
        }
    }

    /// Returns whether `matcher` accepts `actual`, without recording or rendering anything.
    ///
    /// Use it to search, for example for the first matching element, before collecting evidence
    /// for the final result. The matcher still runs, including any user code inside it.
    #[must_use = "probing records nothing, so its result is the only output"]
    pub fn probe<A: ?Sized, M>(&self, actual: &A, matcher: &M) -> bool
    where
        M: Expectation<A, R>,
    {
        matcher.evaluate(actual, &self.probing()).is_ok()
    }

    /// Starts a collector that evaluates children without recording or rendering evidence.
    ///
    /// Evaluating through it is equivalent to [`probe`](Self::probe), but it can run several
    /// children or a [`MatcherList`](crate::expectation::MatcherList) slot.
    pub(crate) fn probing(&self) -> Self {
        self.isolated().with_diagnostics(false)
    }

    /// Runs `f` with a collector whose failures are reported at `path`, such as a field, index,
    /// or key, and returns what `f` returns.
    ///
    /// Recorded failures move into this collector when `f` returns. Nested calls build longer
    /// paths, such as `At .rows[1]:`.
    pub fn scoped<T>(&mut self, path: PathSegment, f: impl FnOnce(&mut Self) -> T) -> T {
        let mut child = self.isolated();
        child.order.nested = true;
        let result = f(&mut child);
        let mut evidence = child.into_evidence();
        // The last child takes `path` itself, so only the others need a clone.
        if let Some((last, others)) = evidence.children.split_last_mut() {
            for failure in others {
                failure.path.insert(0, path.clone());
            }
            last.path.insert(0, path);
        }
        self.append(evidence);
        result
    }

    /// Starts an empty collector with the same settings and the remaining rendering budget.
    ///
    /// Return its failures with [`into_evidence`](Self::into_evidence). Dropping it discards
    /// them, which suits branches that turn out not to matter.
    #[must_use]
    pub fn isolated(&self) -> Self {
        Self {
            order: self.order,
            children: Smallest::new(self.child_limit()),
            rendering: self.rendering,
            include_location: self.include_location,
            diagnostic: self.diagnostic,
            omitted: 0,
        }
    }

    pub(crate) fn isolated_for_order(&self, order: RenderingOrder) -> Self {
        let mut context = self.isolated();
        // Descendants contribute to the same evidence group and must retain by its order.
        context.order.sorted |= order == RenderingOrder::SortByRenderedText;
        context
    }

    /// Runs an assertion callback on a capture chain with this scope's rendering and location
    /// settings, recording its failures here. Returns whether every assertion passed.
    ///
    /// # Panics
    ///
    /// Panics if the callback performed no assertions. User panics propagate.
    pub(crate) fn run_assertions<A>(
        &mut self,
        actual: &A,
        assertions: impl for<'a> FnOnce(AssertThat<'a, A, Capture, R>),
    ) -> bool
    where
        R: Clone,
    {
        let failures = AssertThat::new(Actual::Borrowed(actual))
            .with_renderer(self.rendering.renderer().clone())
            .with_rendering_budget(self.rendering.budget())
            .with_location(self.include_location)
            .collect_failures(|sink| {
                assertions(sink.derive(|value| value));
                sink
            });
        let matched = failures.is_empty();
        for failure in failures {
            self.record(|_| failure);
        }
        matched
    }

    // Sorted children keep competing after the group fills. Iteration-preserving children share
    // the remaining slots. Probes inherit the same allowance but disable diagnostics separately.
    fn child_limit(&self) -> usize {
        let limit = self.children.maximum();
        if self.order.sorted {
            limit
        } else {
            limit.saturating_sub(self.children.len())
        }
    }

    fn retain(&mut self, mut failure: AssertionFailure) {
        let limit = self.children.maximum();
        let rank = self.children.len() + self.omitted;
        self.omitted += usize::from(self.children.len() == limit);
        let text = if limit > 0 {
            self.order.text(&mut failure)
        } else {
            None
        };
        self.children.offer(Keyed {
            key: (text, rank),
            value: failure,
        });
    }

    pub(crate) fn append(&mut self, other: Evidence) {
        self.omitted += other.omitted;
        for failure in other.children {
            self.retain(failure);
        }
    }
}

#[cfg(test)]
impl Default for AssertionContext<'static, DebugRenderer> {
    fn default() -> Self {
        Self::new(&DebugRenderer, RenderingBudget::default())
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use crate::{expectation::AssertionContext, matchers::predicate, prelude::*};

    mod record {
        use super::*;
        use crate::{
            failure::{FailureBuilder, FailureKind, PathSegment},
            renderer::{DebugRenderer, RenderingBudget, RenderingOrder},
        };

        #[test]
        fn builds_only_retained_failures_and_suppresses_probes() {
            for probe in [false, true] {
                for maximum in [0, 1, 3] {
                    let calls = Cell::new(0);
                    let mut context = AssertionContext::new(
                        &DebugRenderer,
                        RenderingBudget::default().with_max_items(maximum),
                    )
                    .with_diagnostics(!probe);
                    context.scoped(PathSegment::Field("items"), |context| {
                        for index in 0..3 {
                            context.record(|_| {
                                calls.set(calls.get() + 1);
                                FailureBuilder::new::<i32>(FailureKind::Equality)
                                    .path([PathSegment::Index(index)])
                                    .build()
                            });
                        }
                    });
                    let retained = if probe { 0 } else { maximum };
                    assert_that!(calls.get()).is_equal_to(retained);
                    let evidence = context.into_evidence();
                    assert_that!(evidence.children).has_length(retained);
                    assert_that!(evidence.omitted).is_equal_to(if probe {
                        0
                    } else {
                        3 - retained
                    });
                    for (index, failure) in evidence.children.iter().enumerate() {
                        assert_that!(failure.path).contains_exactly([
                            PathSegment::Field("items"),
                            PathSegment::Index(index),
                        ]);
                    }
                }
            }
        }

        #[test]
        fn sorted_evidence_keeps_competing_after_retention_fills() {
            let calls = Cell::new(0);
            let mut context =
                AssertionContext::new(&DebugRenderer, RenderingBudget::default().with_max_items(1))
                    .isolated_for_order(RenderingOrder::SortByRenderedText);
            for relation in ["z", "a", "m"] {
                context.record(|_| {
                    calls.set(calls.get() + 1);
                    FailureBuilder::new::<i32>(FailureKind::Matching)
                        .relation(relation)
                        .build()
                });
            }
            assert_that!(calls.get()).is_equal_to(3);
            let evidence = context.into_evidence();
            assert_that!(evidence.children).has_length(1);
            assert_that!(evidence.children[0].relation.as_deref()).is_equal_to(Some("a"));
            assert_that!(evidence.omitted).is_equal_to(2);
        }
    }

    mod assertion_children {
        use core::fmt;

        use super::*;
        use crate::{
            assertions::{collection::each, core::partial_eq::eq},
            failure::PathSegment,
            matchers::all_of,
            renderer::RenderingOrder,
        };

        struct Group<D>(D);

        impl<D: Expectation<i32>> Expectation<i32> for Group<D> {
            type Success<'a>
                = ()
            where
                Self: 'a,
                i32: 'a;
            type Rejection<'a>
                = crate::expectation::Evidence
            where
                Self: 'a,
                i32: 'a;

            fn evaluate(
                &self,
                actual: &i32,
                context: &AssertionContext<'_>,
            ) -> Result<(), Self::Rejection<'_>> {
                let mut children = context.isolated();
                if children.evaluate(actual, &self.0) {
                    Ok(())
                } else {
                    Err(children.into_evidence())
                }
            }

            const KIND: crate::failure::FailureKind = crate::failure::FailureKind::Matching;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a i32, Self::Rejection<'a>)>,
                failure: crate::failure::FailureBuilder,
                _: &AssertionContext<'_>,
            ) -> crate::failure::FailureBuilder {
                match rejected {
                    Some((_, evidence)) => failure
                        .relation("does not satisfy the group")
                        .evidence(evidence),
                    None => failure.relation("satisfies the group"),
                }
            }
        }

        #[test]
        fn grouped_evidence_has_relative_paths_without_losing_repeated_field_names() {
            use crate::__private::field;
            let mut context = AssertionContext::default();
            context.scoped(PathSegment::Field("value"), |context| {
                assert_that!(context.evaluate(&1, &Group(eq(2)))).is_false();
                assert_that!(context.evaluate(
                    &1,
                    &Group(field(
                        |value: &i32| Some(value),
                        eq(2),
                        PathSegment::Field("value"),
                    )),
                ))
                .is_false();
            });
            let failures = context.into_evidence().children;
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_that!(failure.path).is_equal_to([PathSegment::Field("value")]);
                assert_that!(failure.children).has_length(1);
            }
            assert_that!(failures[0].children[0].path).is_empty();
            assert_that!(failures[1].children[0].path)
                .contains_exactly([PathSegment::Field("value")]);
            assert_that!(failures[0].to_string()).is_equal_to(indoc::indoc! {r"
                -------- assertr --------
                does not satisfy the group

                Nested failures:
                  - Expected: 2

                      Actual: 1
                -------- assertr --------
            "});
        }

        #[test]
        fn recording_relative_children_preserves_a_repeated_field_name() {
            let mut context = AssertionContext::default();
            context.scoped(PathSegment::Field("value"), |context| {
                context.record(|_| {
                    crate::failure::FailureBuilder::new::<i32>(
                        crate::failure::FailureKind::Matching,
                    )
                    .child(
                        crate::failure::FailureBuilder::new::<i32>(
                            crate::failure::FailureKind::Equality,
                        )
                        .path([PathSegment::Field("value")])
                        .build(),
                    )
                    .build()
                });
            });
            let failure = context.into_evidence().children.remove(0);
            assert_that!(failure.path).is_equal_to([PathSegment::Field("value")]);
            assert_that!(failure.children[0].path).is_equal_to([PathSegment::Field("value")]);
        }

        #[test]
        fn inherits_order_and_attaches_scoped_paths_once() {
            let mut context =
                AssertionContext::new(&DebugRenderer, RenderingBudget::default().with_max_items(1))
                    .isolated_for_order(RenderingOrder::SortByRenderedText);
            let matcher = all_of(matchers![each(eq(9)), each(eq(8))]);

            let result = context.scoped(PathSegment::Field("items"), |context| {
                context.evaluate(&[3, 2, 1], &matcher)
            });

            assert_that!(result).is_false();
            assert_that!(context.omitted).is_equal_to(5);
            assert_that!(context.into_evidence().children).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive(|failure| &failure.path)
                        .contains_exactly([PathSegment::Field("items")]);
                    failure
                        .derive_owned(|failure| failure.actual.as_ref())
                        .is_equal_to(Some(&AssertionContext::default().render().value(&1)));
                    failure
                        .derive_owned(|failure| failure.expected.as_ref())
                        .is_equal_to(Some(&AssertionContext::default().render().value(&8)));
                },
            ]);
        }

        #[test]
        fn scoped_paths_participate_in_sorting_before_truncation() {
            use alloc::collections::BTreeSet;
            let actual = BTreeSet::from([[1]]);
            let matcher = each(all_of(matchers![eq([9]), crate::elements_are![eq(9)]]));
            let paths = [
                alloc::vec![PathSegment::Field("items")],
                alloc::vec![PathSegment::Field("items"), PathSegment::Index(0)],
            ];
            for limit in [1, 2] {
                let mut context = AssertionContext::new(
                    &DebugRenderer,
                    RenderingBudget::default().with_max_items(limit),
                );
                let result = context.scoped(PathSegment::Field("items"), |context| {
                    context.evaluate(&actual, &matcher)
                });
                assert_that!(result).is_false();
                assert_that!(context.omitted).is_equal_to(2 - limit);
                let retained = context
                    .into_evidence()
                    .children
                    .into_iter()
                    .map(|failure| failure.path)
                    .collect::<Vec<_>>();
                assert_that!(retained).is_equal_to(&paths[..limit]);
            }
        }

        struct CountingRenderer<'a>(&'a Cell<usize>);

        impl ValueRenderer<i32> for CountingRenderer<'_> {
            fn fmt(&self, value: &i32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(formatter, "{value}")
            }
        }

        #[test]
        fn retains_remaining_capacity_and_suppresses_rendering_in_probes() {
            for order in [
                RenderingOrder::PreserveIteration,
                RenderingOrder::SortByRenderedText,
            ] {
                for limit in [0, 1, 2, 4, 5, usize::MAX] {
                    let renders = Cell::new(0);
                    let renderer = CountingRenderer(&renders);
                    let mut context = AssertionContext::new(
                        &renderer,
                        RenderingBudget::default().with_max_items(limit),
                    )
                    .isolated_for_order(order);
                    assert_that!(context.evaluate(&0, &eq(9))).is_false();
                    let matcher = each(eq(9));
                    assert_that!(context.evaluate(&[1, 2, 3], &matcher)).is_false();

                    let retained = limit.min(4);
                    let expected_renders =
                        if order == RenderingOrder::SortByRenderedText && limit > 0 {
                            8
                        } else {
                            2 * retained
                        };
                    assert_that!(context.omitted).is_equal_to(4 - retained);
                    assert_that!(renders.get()).is_equal_to(expected_renders);
                    assert_that!(context.probe(&[1, 2, 3], &matcher)).is_false();
                    assert_that!(context.probe(&[9], &matcher)).is_true();
                    assert_that!(renders.get()).is_equal_to(expected_renders);
                    let evidence = context
                        .finish(false, |_| panic!("replaced existing evidence"))
                        .err()
                        .unwrap();
                    assert_that!(evidence.children).has_length(retained);
                }
            }
        }
    }

    mod probe {
        use super::*;
        use crate::{
            expectation::Evidence,
            failure::{FailureBuilder, FailureKind},
            matchers::all_of,
        };

        struct Flat<'a>(&'a Cell<usize>);

        impl Expectation<i32> for Flat<'_> {
            type Success<'a>
                = ()
            where
                Self: 'a,
                i32: 'a;
            type Rejection<'a>
                = Evidence
            where
                Self: 'a,
                i32: 'a;

            fn evaluate(&self, _: &i32, context: &AssertionContext<'_>) -> Result<(), Evidence> {
                Err(context.isolated().into_evidence())
            }

            const KIND: FailureKind = FailureKind::Matching;
            const FLATTEN: bool = true;

            fn explain<'a>(
                &'a self,
                _: Option<(&'a i32, Evidence)>,
                failure: FailureBuilder,
                _: &AssertionContext<'_>,
            ) -> FailureBuilder {
                self.0.set(self.0.get() + 1);
                failure
            }
        }

        #[test]
        fn does_not_explain_flattening_definitions() {
            let explanations = Cell::new(0);
            let context = AssertionContext::default();

            assert_that!(context.probe(&1, &all_of(matchers![Flat(&explanations)]))).is_false();
            assert_that!(explanations.get()).is_equal_to(0);
            assert_that!(context.probe(
                &[1],
                &matchers::each(all_of(matchers![Flat(&explanations)]))
            ))
            .is_false();
            assert_that!(explanations.get()).is_equal_to(0);

            let mut context = AssertionContext::default();
            assert_that!(context.evaluate(&1, &all_of(matchers![Flat(&explanations)]))).is_false();
            // The rejection, then the conjunction's fallback description of its empty evidence.
            assert_that!(explanations.get()).is_equal_to(2);
        }

        #[test]
        fn evaluates_once_without_retaining_evidence() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let context = AssertionContext::default();

            assert_that!(context.probe(&1, &matcher)).is_false();
            assert_that!(context.into_evidence().children).is_empty();
            assert_that!(calls.get()).is_equal_to(1);
        }
    }

    mod isolated {
        use super::*;

        #[test]
        fn dropping_discards_evidence_without_replaying_user_code() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let context = AssertionContext::default();
            for finish_success in [false, true] {
                let mut fork = context.isolated();
                assert_that!(fork.evaluate(&1, &matcher)).is_false();
                if finish_success {
                    assert_that!(fork.finish(true, |_| panic!("described a successful group")))
                        .is_ok();
                }
            }

            assert_that!(context.into_evidence().children).is_empty();
            assert_that!(calls.get()).is_equal_to(2);
        }

        #[test]
        fn committing_retains_evidence_without_replaying_user_code() {
            let calls = Cell::new(0);
            let matcher = predicate(|actual: &i32| {
                calls.set(calls.get() + 1);
                *actual == 2
            });
            let mut context = AssertionContext::default();
            {
                let mut fork = context.isolated();
                assert_that!(fork.evaluate(&1, &matcher)).is_false();
                context.append(fork.into_evidence());
            }

            assert_that!(context.into_evidence().children).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
        }
    }
}
