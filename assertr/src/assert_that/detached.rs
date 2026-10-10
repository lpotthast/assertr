use alloc::{string::String, vec::Vec};
use core::{marker::PhantomData, panic::Location};

use crate::{
    AssertThat, ChainRecords, ChainState, DiagnosticSettings,
    actual::Actual,
    expectation::AssertionContext,
    failure::{AssertionFailure, FailureBuilder, present_and_panic},
    mode::Panic,
    renderer::RenderingContext,
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
    settings: DiagnosticSettings,
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
            settings: state.settings,
            renderer: state.renderer,
        };
        (self.actual, detached)
    }
}

impl<R> DetachedChain<R> {
    /// Renders values as the chain did.
    pub(crate) const fn render(&self) -> RenderingContext<'_, R> {
        self.settings.render(&self.renderer)
    }

    /// Evaluates expectations as the chain did.
    pub(crate) fn assertion_context(&self) -> AssertionContext<'_, R> {
        self.settings.assertion_context(&self.renderer)
    }

    /// Raises `failure` as the chain would have.
    #[track_caller]
    pub(crate) fn raise_at(
        self,
        failure: FailureBuilder,
        location: &'static Location<'static>,
    ) -> ! {
        let presentation = self.settings.panic_presentation.clone();
        let failure = self.complete_at(failure, location);
        present_and_panic(&failure, presentation.as_deref())
    }

    /// Completes a returned failure with exactly the metadata used by panic mode.
    pub(crate) fn complete_at(
        self,
        failure: FailureBuilder,
        location: &'static Location<'static>,
    ) -> AssertionFailure {
        let mut failure = self.settings.complete(failure, location);
        failure.messages.extend(self.messages);
        failure
    }

    /// Continues as a chain on `actual`, keeping the collected messages.
    pub(crate) fn attach<T>(self, actual: Actual<'_, T>) -> AssertThat<'_, T, Panic, R> {
        let mut records = ChainRecords::new(None);
        records.inherited_messages = self.messages;
        AssertThat {
            actual,
            state: ChainState {
                records,
                settings: self.settings,
                mode: PhantomData,
                renderer: self.renderer,
            },
        }
    }
}
