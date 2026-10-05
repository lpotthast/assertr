#[cfg(test)]
use crate::RenderingBudget;
use crate::{
    AssertionFailure, DebugRenderer, ExpectationDiagnostics,
    expectation::Evidence,
    failure::{
        FailureBuilder, FailureKind, PathSegment,
        adapter::{HumanReadableText, ToHumanReadableText},
    },
    renderer::{RenderingContext, RenderingOrder},
    util::selection::{Keyed, Smallest},
};
use alloc::vec::Vec;

/// Executor-provided rendering, paths, and isolated expectation evidence.
///
/// Evaluation never raises a root assertion. A probe suppresses built-in diagnostic rendering, but
/// cannot undo side effects or rendering performed by downstream code or assertion closures.
///
/// There is no public constructor. Use an assertion chain to execute a definition. Downstream
/// compositions receive this context and can create an [`isolated`](Self::isolated) child scope.
///
/// ```compile_fail
/// use assertr::{AssertionContext, DebugRenderer, RenderingBudget};
/// let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
/// ```
pub struct AssertionContext<'r, R = DebugRenderer> {
    rendering: RenderingContext<'r, R>,
    include_location: bool,
    diagnostic: bool,
    limit: usize,
    evidence_order: RenderingOrder,
    path: Vec<PathSegment>,
    pub(crate) children: Smallest<Keyed<(Option<HumanReadableText>, usize), AssertionFailure>>,
    pub(crate) omitted: usize,
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
            limit: rendering.max_items(),
            evidence_order: RenderingOrder::PreserveIteration,
            children: Smallest::new(rendering.max_items()),
            rendering,
            include_location,
            diagnostic: true,
            path: Vec::new(),
            omitted: 0,
        }
    }

    /// Whether captured assertion callbacks retain caller locations.
    #[must_use]
    pub fn include_location(&self) -> bool {
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

    /// Consumes a completed evaluation's bounded evidence, retaining its already scoped paths.
    #[must_use]
    pub fn into_evidence(self) -> Evidence {
        Evidence {
            children: self.children.into_values(),
            omitted: self.omitted,
            path_prefix_len: self.path.len(),
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
        if !matched && !self.has_evidence() {
            self.outcome(false, fallback);
        }
        matched
    }

    /// Whether this scope has retained or omitted failure evidence.
    pub(crate) fn has_evidence(&self) -> bool {
        self.children.len() > 0 || self.omitted > 0
    }

    /// Executes and immediately explains one definition. Observations cannot escape to siblings.
    pub fn evaluate<A: ?Sized, D: ExpectationDiagnostics<A, R> + ?Sized>(
        &mut self,
        actual: &A,
        definition: &D,
    ) -> bool {
        let result = definition.evaluate(actual, self);
        let matched = result.is_ok();
        if let Err(rejection) = result {
            // Transparent groups have already applied their budgets and rendered their children.
            // Even a zero-capacity group must transfer its omitted count. Probes never explain.
            if self.diagnostic && (D::FLATTEN || self.is_diagnostic()) {
                let failure = FailureBuilder::detached::<A>(D::KIND);
                let failure = definition
                    .explain(Some((actual, rejection)), failure, self)
                    .build();
                if D::FLATTEN {
                    self.omitted += failure.omitted_children;
                    for child in failure.children {
                        self.record(child);
                    }
                } else {
                    self.record(failure);
                }
            } else {
                drop(rejection);
                self.omitted += usize::from(self.diagnostic);
            }
        }
        matched
    }

    /// Describes an unmet expectation for which there is no subject to evaluate.
    pub(crate) fn describe<A: ?Sized, D: ExpectationDiagnostics<A, R> + ?Sized>(
        &self,
        definition: &D,
    ) -> AssertionFailure {
        definition
            .explain(None, FailureBuilder::detached::<A>(D::KIND), self)
            .build()
    }

    pub(crate) fn describe_list<A: ?Sized, L, Target>(
        &self,
        list: &L,
        failure: FailureBuilder<Target>,
    ) -> FailureBuilder<Target>
    where
        L: crate::expectation::MatcherList<A, R>,
    {
        let retained = list.len().min(self.rendering.max_items());
        failure
            .omitted_children(list.len() - retained)
            .children((0..retained).map(|index| list.describe_at(index, self)))
    }

    /// Returns the active rendering context.
    #[must_use]
    pub fn render(&self) -> RenderingContext<'r, R> {
        self.rendering
    }

    /// Whether evidence is requested. Built-in leaves do not render during probes.
    #[must_use]
    pub fn is_diagnostic(&self) -> bool {
        self.diagnostic && self.child_limit() > 0
    }

    /// Records detached evidence, preserving structured values and inner assertion metadata.
    pub fn record(&mut self, failure: AssertionFailure) {
        self.record_with(|_| failure);
    }

    /// Builds detached evidence only when this scope can retain it.
    pub(crate) fn record_with(&mut self, description: impl FnOnce(&Self) -> AssertionFailure) {
        if !self.diagnostic {
            return;
        }
        if !self.is_diagnostic() {
            self.omitted += 1;
            return;
        }
        let mut failure = description(self);
        let mut path = self.path.clone();
        path.append(&mut failure.path);
        failure.path = path;
        self.retain(failure);
    }

    /// Records a leaf constraint when it rejects the subject.
    ///
    /// `description` must describe the unmet constraint as if no subject existed, for example
    /// through [`ExpectationDiagnostics::explain`] with `None`. The recorded failure wraps it as
    /// the constraint of a "does not satisfy the constraint" failure. The closure runs only when
    /// this scope can retain the failure.
    pub fn outcome(
        &mut self,
        matched: bool,
        description: impl FnOnce(&Self) -> AssertionFailure,
    ) -> bool {
        if !matched {
            if self.is_diagnostic() {
                self.record(
                    FailureBuilder::detached::<()>(FailureKind::Matching)
                        .relation("does not satisfy the constraint")
                        .constraint(description(self))
                        .build(),
                );
            } else if self.diagnostic {
                self.omitted += 1;
            }
        }
        matched
    }

    /// Evaluates truth in isolation without committing evidence.
    pub fn probe<A: ?Sized, M>(&self, actual: &A, matcher: &M) -> bool
    where
        M: crate::Expectation<A, R>,
    {
        let context =
            Self::from_rendering(self.rendering, self.include_location).with_diagnostics(false);
        matcher.evaluate(actual, &context).is_ok()
    }

    /// Evaluates within one relative path segment and commits that scope's evidence once.
    pub fn scoped<T>(&mut self, path: PathSegment, f: impl FnOnce(&mut Self) -> T) -> T {
        let mut child = self.isolated();
        child.path.push(path);
        let result = f(&mut child);
        self.append(child.into_evidence());
        result
    }

    /// Starts an independent evaluation scope with the remaining evidence budget and current path.
    /// Dropping it discards its evidence. Consume it with [`Self::into_evidence`] to retain
    /// children.
    #[must_use]
    pub fn isolated(&self) -> Self {
        let limit = self.child_limit();
        Self {
            limit,
            evidence_order: self.evidence_order,
            children: Smallest::new(limit),
            rendering: self.rendering,
            include_location: self.include_location,
            diagnostic: self.diagnostic,
            path: self.path.clone(),
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

    /// Effective allowance and inherited order for independently retained candidate evidence.
    pub(crate) fn evidence_policy(&self) -> (usize, RenderingOrder) {
        let limit = if self.diagnostic {
            self.child_limit()
        } else {
            0
        };
        (limit, self.evidence_order)
    }

    // Sorted children keep competing after the group fills. Iteration-preserving children share
    // the remaining slots. Probes inherit the same allowance but disable diagnostics separately.
    fn child_limit(&self) -> usize {
        if self.evidence_order == RenderingOrder::SortByRenderedText {
            self.limit
        } else {
            self.limit.saturating_sub(self.children.len())
        }
    }

    fn retain(&mut self, failure: AssertionFailure) {
        let rank = self.children.len() + self.omitted;
        self.omitted += usize::from(self.children.len() == self.limit);
        let text = (self.evidence_order == RenderingOrder::SortByRenderedText && self.limit > 0)
            .then(|| ToHumanReadableText::render_child(&failure));
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
    use crate::{AssertionContext, expectation::predicate, prelude::*};
    use core::cell::Cell;

    mod record_with {
        use super::*;
        use crate::{
            DebugRenderer, RenderingBudget,
            failure::{FailureBuilder, FailureKind, PathSegment},
            renderer::RenderingOrder,
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
                            context.record_with(|_| {
                                calls.set(calls.get() + 1);
                                FailureBuilder::detached::<i32>(FailureKind::Equality)
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
                context.record_with(|_| {
                    calls.set(calls.get() + 1);
                    FailureBuilder::detached::<i32>(FailureKind::Matching)
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
        use super::*;
        use crate::failure::adapter::HumanReadableText;
        use crate::{
            assertions::{collection::contains_matching, core::partial_eq::equal_to},
            expectation::all_of,
            failure::PathSegment,
            renderer::{IntoRendered, RenderingOrder},
        };
        use core::fmt;

        struct Group<D>(D);

        impl<D: ExpectationDiagnostics<i32>> Expectation<i32> for Group<D> {
            type Success<'a>
                = ()
            where
                Self: 'a;
            type Rejection<'a>
                = crate::expectation::Evidence
            where
                Self: 'a;

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
        }

        impl<D: ExpectationDiagnostics<i32>> ExpectationDiagnostics<i32> for Group<D> {
            const KIND: crate::FailureKind = crate::FailureKind::Matching;

            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a i32, Self::Rejection<'a>)>,
                failure: crate::failure::FailureBuilder<Target>,
                _: &AssertionContext<'_>,
            ) -> crate::failure::FailureBuilder<Target> {
                match rejected {
                    Some((_, evidence)) => {
                        evidence.explain(failure.relation("does not satisfy the group"))
                    }
                    None => failure.relation("satisfies the group"),
                }
            }
        }

        #[test]
        fn grouped_evidence_has_relative_paths_without_losing_repeated_field_names() {
            use crate::__private::field::field;
            let mut context = AssertionContext::default();
            context.scoped(PathSegment::Field("value"), |context| {
                context.evaluate(&1, &Group(equal_to(2)));
                context.evaluate(
                    &1,
                    &Group(field(
                        |value: &i32| Some(value),
                        equal_to(2),
                        PathSegment::Field("value"),
                    )),
                );
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
            assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(
                HumanReadableText::new(indoc::indoc! {r"
                -------- assertr --------
                does not satisfy the group

                Nested failures:
                  - Expected: 2

                      Actual: 1
                -------- assertr --------
            "}),
            );
        }

        #[test]
        fn recording_relative_children_preserves_a_repeated_field_name() {
            let mut context = AssertionContext::default();
            context.scoped(PathSegment::Field("value"), |context| {
                context.record(
                    crate::failure::FailureBuilder::detached::<i32>(crate::FailureKind::Matching)
                        .child(
                            crate::failure::FailureBuilder::detached::<i32>(
                                crate::FailureKind::Equality,
                            )
                            .path([PathSegment::Field("value")])
                            .build(),
                        )
                        .build(),
                );
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
            let matcher = all_of((
                contains_matching(equal_to(9)),
                contains_matching(equal_to(8)),
            ));

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
                        .is_equal_to(Some(
                            &AssertionContext::default()
                                .render()
                                .value(&1)
                                .into_rendered(),
                        ));
                    failure
                        .derive_owned(|failure| failure.expected.as_ref())
                        .is_equal_to(Some(
                            &AssertionContext::default()
                                .render()
                                .value(&8)
                                .into_rendered(),
                        ));
                },
            ]);
        }

        #[test]
        fn scoped_paths_participate_in_sorting_before_truncation() {
            use alloc::collections::BTreeSet;
            let actual = BTreeSet::from([[1]]);
            let matcher =
                contains_matching(all_of((equal_to([9]), crate::elements_are![equal_to(9)])));
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
                    context.evaluate(&0, &equal_to(9));
                    let matcher = contains_matching(equal_to(9));
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
            expectation::{Evidence, all_of},
            failure::{FailureBuilder, FailureKind},
        };

        struct Flat<'a>(&'a Cell<usize>);

        impl Expectation<i32> for Flat<'_> {
            type Success<'a>
                = ()
            where
                Self: 'a;
            type Rejection<'a>
                = Evidence
            where
                Self: 'a;

            fn evaluate(&self, _: &i32, context: &AssertionContext<'_>) -> Result<(), Evidence> {
                Err(context.isolated().into_evidence())
            }
        }

        impl ExpectationDiagnostics<i32> for Flat<'_> {
            const KIND: FailureKind = FailureKind::Matching;
            const FLATTEN: bool = true;

            fn explain<'a, Target>(
                &'a self,
                _: Option<(&'a i32, Evidence)>,
                failure: FailureBuilder<Target>,
                _: &AssertionContext<'_>,
            ) -> FailureBuilder<Target> {
                self.0.set(self.0.get() + 1);
                failure
            }
        }

        #[test]
        fn does_not_explain_flattening_definitions() {
            let explanations = Cell::new(0);
            let context = AssertionContext::default();

            assert_that!(context.probe(&1, &all_of((Flat(&explanations),)))).is_false();
            assert_that!(explanations.get()).is_equal_to(0);
            assert_that!(context.probe(&[1], &matchers::each(all_of((Flat(&explanations),)))))
                .is_false();
            assert_that!(explanations.get()).is_equal_to(0);

            let mut context = AssertionContext::default();
            assert_that!(context.evaluate(&1, &all_of((Flat(&explanations),)))).is_false();
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
                fork.evaluate(&1, &matcher);
                context.append(fork.into_evidence());
            }

            assert_that!(context.into_evidence().children).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
        }
    }
}
