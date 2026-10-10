use alloc::{string::String, sync::Arc, vec::Vec};
use core::{marker::PhantomData, panic::Location};

use crate::{
    AssertThat, ChainRecords, ChainState, Expression,
    actual::Actual,
    expectation::AssertionContext,
    failure::{AssertionFailure, FailureBuilder, panic_presentation::PanicPresentation},
    mode::Panic,
    renderer::{RenderingBudget, RenderingContext},
};

/// A panic-mode chain without its subject and its link to ancestor records: what an assertion
/// keeps while it awaits. It holds no cells and no borrows of ancestors, so a future holding it is
/// `Send` whenever the renderer is.
///
/// Detaching collects the ancestor messages, exactly as `capture` does. A chain
/// [attached](Self::attach) again is a root, so later assertion counts no longer reach the
/// ancestors. Panic mode needs neither: it raises every failure immediately.
pub(crate) struct DetachedChain<R> {
    messages: Vec<String>,
    subject_name: Option<String>,
    expression: Expression,
    include_location: bool,
    rendering_budget: RenderingBudget,
    panic_presentation: Option<Arc<PanicPresentation>>,
    renderer: R,
}

impl<'t, T, R> AssertThat<'t, T, Panic, R> {
    /// Separates the subject from the chain's diagnostic settings, see [`DetachedChain`].
    pub(crate) fn into_parts(self) -> (Actual<'t, T>, DetachedChain<R>) {
        let state = self.state;
        let mut messages = Vec::new();
        state.records.collect_messages(&mut messages);
        let detached = DetachedChain {
            messages,
            subject_name: state.subject_name,
            expression: state.expression,
            include_location: state.include_location,
            rendering_budget: state.rendering_budget,
            panic_presentation: state.panic_presentation,
            renderer: state.renderer,
        };
        (self.actual, detached)
    }
}

impl<R> DetachedChain<R> {
    /// Renders values as the chain did.
    pub(crate) const fn render(&self) -> RenderingContext<'_, R> {
        RenderingContext::new(&self.renderer, self.rendering_budget)
    }

    /// Evaluates expectations as the chain did.
    pub(crate) fn assertion_context(&self) -> AssertionContext<'_, R> {
        AssertionContext::from_rendering(self.render(), self.include_location)
    }

    /// Raises `failure` as the chain would have.
    #[track_caller]
    pub(crate) fn raise_at(
        self,
        failure: FailureBuilder,
        location: &'static Location<'static>,
    ) -> ! {
        self.attach(Actual::Owned(())).raise_at(failure, location);
        unreachable!("a panic-mode chain panics when it raises a failure")
    }

    /// Completes a returned failure with exactly the metadata used by panic mode.
    pub(crate) fn complete_at(
        self,
        failure: FailureBuilder,
        location: &'static Location<'static>,
    ) -> AssertionFailure {
        self.attach(Actual::Owned(()))
            .complete_failure(failure, location)
    }

    /// Continues as a chain on `actual`, keeping the collected messages.
    pub(crate) fn attach<T>(self, actual: Actual<'_, T>) -> AssertThat<'_, T, Panic, R> {
        let mut records = ChainRecords::new(None);
        records.inherited_messages = self.messages;
        AssertThat {
            actual,
            state: ChainState {
                records,
                subject_name: self.subject_name,
                expression: self.expression,
                include_location: self.include_location,
                rendering_budget: self.rendering_budget,
                panic_presentation: self.panic_presentation,
                mode: PhantomData,
                renderer: self.renderer,
            },
        }
    }
}
