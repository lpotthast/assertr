//! Assertions on a value that changes over time: [`eventually`](EventualAssertions::eventually)
//! meets an expectation, or [`consistently`](EventualAssertions::consistently) keeps meeting it.
//!
//! The subject is an observation: a closure returning a future of the current value, such as
//! `|| log.text()`. The assertion observes it repeatedly, with the [`Patience`] configured globally
//! or for the chain, and applies any [matcher](crate::matchers) or assertion callback to every
//! observed value.
//!
//! ```rust
//! use assertr::prelude::*;
//! use std::sync::atomic::{AtomicU32, Ordering};
//!
//! # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
//! let counter = AtomicU32::new(0);
//! let observe = || async { counter.fetch_add(1, Ordering::SeqCst) + 1 };
//!
//! // Waits until the counter has reached 3, then continues with the observed value.
//! assert_that!(observe)
//!     .eventually()
//!     .satisfies(|count| {
//!         count.is_greater_or_equal_to(3);
//!     })
//!     .await
//!     .is_less_than(100);
//! # });
//! ```

mod patience;
mod sleep;

use core::{marker::PhantomData, panic::Location, time::Duration};
use std::{collections::VecDeque, time::Instant};

use crate::{
    AssertThat,
    actual::Actual,
    assert_that::DetachedChain,
    expectation::Expectation,
    failure::{Fact, FailureBuilder, FailureKind},
    matchers::satisfying,
    mode::{Capture, Panic},
    renderer::{Rendered, ValueRenderer},
};
use patience::Overrides;
pub use patience::Patience;

/// Marks an eventual assertion whose observation's output is the subject.
#[derive(Debug, Clone, Copy)]
pub struct Plain;

/// Marks an eventual assertion whose observation returns a `Result`: the `Ok` value is the
/// subject, and an `Err` is an observation that failed.
#[derive(Debug, Clone, Copy)]
pub struct Fallible;

/// Eventual assertions on an observation: a closure returning a future of the current value.
///
/// These methods start a builder that configures the [`Patience`] for this chain and ends with the
/// assertion itself, `matches` or `satisfies`. They are available in panic mode, and the subject
/// may be borrowed or owned.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait EventualAssertions: Sized {
    /// Observes the subject until the assertion holds, failing after the [`Patience`]'s timeout
    /// with the last observed value and the values seen before it.
    fn eventually(self) -> Eventually<Self, Plain>;

    /// Like [`eventually`](Self::eventually), for an observation returning a `Result`: an `Err`
    /// is an observation that failed and is retried. If observing still fails at the timeout,
    /// the failure shows the error.
    fn eventually_ok(self) -> Eventually<Self, Fallible>;

    /// Observes the subject for the [`Patience`]'s consistency duration, failing as soon as one
    /// observation does not satisfy the assertion.
    fn consistently(self) -> Consistently<Self, Plain>;

    /// Like [`consistently`](Self::consistently), for an observation returning a `Result`. An
    /// `Err` fails the assertion right away.
    fn consistently_ok(self) -> Consistently<Self, Fallible>;
}

impl<F, Fut, R> EventualAssertions for AssertThat<'_, F, Panic, R>
where
    F: Fn() -> Fut,
    Fut: Future,
{
    fn eventually(self) -> Eventually<Self, Plain> {
        Eventually::new(self)
    }

    fn eventually_ok(self) -> Eventually<Self, Fallible> {
        Eventually::new(self)
    }

    fn consistently(self) -> Consistently<Self, Plain> {
        Consistently::new(self)
    }

    fn consistently_ok(self) -> Consistently<Self, Fallible> {
        Consistently::new(self)
    }
}

/// An assertion that the observed value meets an expectation within the timeout. Started by
/// [`eventually`](EventualAssertions::eventually) and
/// [`eventually_ok`](EventualAssertions::eventually_ok).
#[must_use = "an eventual assertion does nothing until `matches` or `satisfies` is awaited"]
pub struct Eventually<C, K> {
    chain: C,
    overrides: Overrides,
    kind: PhantomData<K>,
}

/// An assertion that the observed value keeps meeting an expectation for the consistency
/// duration. Started by [`consistently`](EventualAssertions::consistently) and
/// [`consistently_ok`](EventualAssertions::consistently_ok).
#[must_use = "an eventual assertion does nothing until `matches` or `satisfies` is awaited"]
pub struct Consistently<C, K> {
    chain: C,
    overrides: Overrides,
    kind: PhantomData<K>,
}

impl<C, K> Eventually<C, K> {
    fn new(chain: C) -> Self {
        Self {
            chain,
            overrides: Overrides::default(),
            kind: PhantomData,
        }
    }

    /// Waits up to `timeout` instead of the [`Patience`]'s timeout.
    pub fn within(mut self, timeout: Duration) -> Self {
        self.overrides = self.overrides.within(timeout);
        self
    }

    /// Pauses `interval` between two observations instead of the [`Patience`]'s interval.
    pub fn polling_every(mut self, interval: Duration) -> Self {
        self.overrides = self.overrides.polling_every(interval);
        self
    }

    /// Uses `patience` instead of the global one.
    pub fn with_patience(mut self, patience: Patience) -> Self {
        self.overrides = Overrides::all(patience);
        self
    }
}

impl<C, K> Consistently<C, K> {
    fn new(chain: C) -> Self {
        Self {
            chain,
            overrides: Overrides::default(),
            kind: PhantomData,
        }
    }

    /// Requires the assertion to hold for `duration` instead of the [`Patience`]'s consistency
    /// duration, e.g. past a timer the check must outlast.
    pub fn for_at_least(mut self, duration: Duration) -> Self {
        self.overrides = self.overrides.consistently_for(duration);
        self
    }

    /// Pauses `interval` between two observations instead of the [`Patience`]'s interval.
    pub fn polling_every(mut self, interval: Duration) -> Self {
        self.overrides = self.overrides.polling_every(interval);
        self
    }

    /// Uses `patience` instead of the global one.
    pub fn with_patience(mut self, patience: Patience) -> Self {
        self.overrides = Overrides::all(patience);
        self
    }
}

impl<'t, F, Fut, T: 't, R> Eventually<AssertThat<'t, F, Panic, R>, Plain>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    /// Asserts that an observed value eventually meets `expected`, then continues with that
    /// value. Pass `&expected` to reuse a matcher.
    #[track_caller]
    pub fn matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        R: ValueRenderer<T>,
    {
        let observation = Observation::start(self.chain, Location::caller(), self.overrides);
        observation.until(expected, |output, _| Ok(output))
    }

    /// Asserts that an observed value eventually passes `assertions`, then continues with that
    /// value. The callback runs in capture mode on every observation. End its last assertion
    /// with a semicolon.
    #[track_caller]
    pub fn satisfies<A>(self, assertions: A) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<T>,
    {
        self.matches(satisfying(assertions))
    }
}

impl<'t, F, Fut, T: 't, E, R> Eventually<AssertThat<'t, F, Panic, R>, Fallible>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    /// Asserts that an observation eventually succeeds with a value meeting `expected`, then
    /// continues with that value.
    #[track_caller]
    pub fn matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        let observation = Observation::start(self.chain, Location::caller(), self.overrides);
        observation.until(expected, |output, chain| {
            output.map_err(|error| chain.render().value(&error))
        })
    }

    /// Asserts that an observation eventually succeeds with a value passing `assertions`, then
    /// continues with that value. End the callback's last assertion with a semicolon.
    #[track_caller]
    pub fn satisfies<A>(self, assertions: A) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<T> + ValueRenderer<E>,
    {
        self.matches(satisfying(assertions))
    }
}

impl<'t, F, Fut, T: 't, R> Consistently<AssertThat<'t, F, Panic, R>, Plain>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    /// Asserts that every observed value meets `expected` for the consistency duration, then
    /// continues with the last one. Pass `&expected` to reuse a matcher.
    #[track_caller]
    pub fn matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        R: ValueRenderer<T>,
    {
        let observation = Observation::start(self.chain, Location::caller(), self.overrides);
        observation.throughout(expected, |output, _| Ok(output))
    }

    /// Asserts that every observed value passes `assertions` for the consistency duration, then
    /// continues with the last one. End the callback's last assertion with a semicolon.
    #[track_caller]
    pub fn satisfies<A>(self, assertions: A) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<T>,
    {
        self.matches(satisfying(assertions))
    }
}

impl<'t, F, Fut, T: 't, E, R> Consistently<AssertThat<'t, F, Panic, R>, Fallible>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    /// Asserts that every observation succeeds with a value meeting `expected` for the
    /// consistency duration, then continues with the last one.
    #[track_caller]
    pub fn matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        let observation = Observation::start(self.chain, Location::caller(), self.overrides);
        observation.throughout(expected, |output, chain| {
            output.map_err(|error| chain.render().value(&error))
        })
    }

    /// Asserts that every observation succeeds with a value passing `assertions` for the
    /// consistency duration, then continues with the last one. End the callback's last assertion
    /// with a semicolon.
    #[track_caller]
    pub fn satisfies<A>(self, assertions: A) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<T> + ValueRenderer<E>,
    {
        self.matches(satisfying(assertions))
    }
}

/// The result of one observation: its value, or the rendered error of a failed observation.
type Observed<T> = Result<T, Rendered>;

/// What one observation led to.
enum Step<T> {
    /// The expectation holds for this value.
    Met(T),
    /// The expectation does not hold, explained when the assertion fails with this observation.
    NotMet(Option<Box<FailureBuilder>>),
}

/// A started eventual assertion: the observation, separated from its [detached](DetachedChain)
/// chain. It holds no chain records, so its future is `Send` whenever the observation, the
/// expectation and the renderer are.
struct Observation<'t, F, R> {
    observe: Actual<'t, F>,
    chain: DetachedChain<R>,
    location: &'static Location<'static>,
    patience: Patience,
}

impl<'t, F, Fut, R> Observation<'t, F, R>
where
    F: Fn() -> Fut,
    Fut: Future,
{
    /// Counts the assertion on `chain` and fixes its patience, at the assertion's call.
    fn start(
        chain: AssertThat<'t, F, Panic, R>,
        location: &'static Location<'static>,
        overrides: Overrides,
    ) -> Self {
        chain.track_assertion();
        let (observe, chain) = chain.detach();
        Self {
            observe,
            chain,
            location,
            patience: overrides.resolve(),
        }
    }

    /// Observes until `expected` holds or the timeout passes, then fails with the last
    /// observation.
    async fn until<T: 't, D>(
        self,
        expected: D,
        extract: impl Fn(Fut::Output, &DetachedChain<R>) -> Observed<T>,
    ) -> AssertThat<'t, T, Panic, R>
    where
        D: Expectation<T, R>,
        R: ValueRenderer<T>,
    {
        let mut history = History::start();
        let deadline = history.started + self.patience.timeout();
        loop {
            let output = (self.observe.borrowed())().await;
            let last_attempt = Instant::now() >= deadline;
            let observed = extract(output, &self.chain);
            match observe_once(&self.chain, &mut history, observed, &expected, last_attempt) {
                Step::Met(value) => return self.chain.attach(Actual::Owned(value)),
                Step::NotMet(Some(failure)) => {
                    let waited = Fact::labelled(
                        "Waited",
                        format!(
                            "{} ({} observations)",
                            format_duration(history.started.elapsed()),
                            history.observations
                        ),
                    );
                    self.chain.raise_at(
                        (*failure).fact(waited).facts(history.changes_fact()),
                        self.location,
                    );
                }
                Step::NotMet(None) => sleep::sleep(self.patience.interval()).await,
            }
        }
    }

    /// Observes for the consistency duration, failing with the first observation that does not
    /// meet `expected`.
    async fn throughout<T: 't, D>(
        self,
        expected: D,
        extract: impl Fn(Fut::Output, &DetachedChain<R>) -> Observed<T>,
    ) -> AssertThat<'t, T, Panic, R>
    where
        D: Expectation<T, R>,
        R: ValueRenderer<T>,
    {
        let mut history = History::start();
        let deadline = history.started + self.patience.consistency();
        loop {
            let output = (self.observe.borrowed())().await;
            let observed = extract(output, &self.chain);
            match observe_once(&self.chain, &mut history, observed, &expected, true) {
                Step::Met(value) if Instant::now() >= deadline => {
                    return self.chain.attach(Actual::Owned(value));
                }
                Step::Met(_) => sleep::sleep(self.patience.interval()).await,
                Step::NotMet(failure) => {
                    let held = Fact::labelled(
                        "Held",
                        format!(
                            "for {} ({} observations), then not",
                            format_duration(history.started.elapsed()),
                            history.observations
                        ),
                    );
                    let failure =
                        failure.expect("every rejection of the last attempt is explained");
                    self.chain.raise_at(
                        (*failure).fact(held).facts(history.changes_fact()),
                        self.location,
                    );
                }
            }
        }
    }
}

/// Records one observation and evaluates `expected` on it. A rejection is explained only when
/// `explain` is set, so an assertion that keeps waiting renders nothing but the history.
fn observe_once<T, R, D>(
    chain: &DetachedChain<R>,
    history: &mut History,
    observed: Observed<T>,
    expected: &D,
    explain: bool,
) -> Step<T>
where
    D: Expectation<T, R>,
    R: ValueRenderer<T>,
{
    let value = match observed {
        Ok(value) => value,
        Err(error) => {
            history.record(error.clone());
            return Step::NotMet(explain.then(|| {
                Box::new(
                    FailureBuilder::new::<T>(FailureKind::Other)
                        .relation("could not be observed")
                        .fact(Fact::labelled("Error", error)),
                )
            }));
        }
    };
    history.record(chain.render().value(&value));
    let context = chain.assertion_context();
    let failure = match expected.evaluate(&value, &context) {
        Ok(_) => None,
        Err(rejection) => Some(explain.then(|| {
            Box::new(expected.explain(
                Some((&value, rejection)),
                FailureBuilder::new::<T>(D::KIND),
                &context,
            ))
        })),
    };
    match failure {
        None => Step::Met(value),
        Some(failure) => Step::NotMet(failure),
    }
}

/// How many distinct values the history keeps.
const HISTORY_LENGTH: usize = 8;

/// The observations of one assertion: how many, and the distinct values in the order they were
/// first seen, with when.
struct History {
    started: Instant,
    observations: usize,
    changes: VecDeque<(Duration, Rendered)>,
    earlier_changes: usize,
}

impl History {
    fn start() -> Self {
        Self {
            started: Instant::now(),
            observations: 0,
            changes: VecDeque::new(),
            earlier_changes: 0,
        }
    }

    fn record(&mut self, observed: Rendered) {
        self.observations += 1;
        if self
            .changes
            .back()
            .is_some_and(|(_, last)| *last == observed)
        {
            return;
        }
        if self.changes.len() == HISTORY_LENGTH {
            self.changes.pop_front();
            self.earlier_changes += 1;
        }
        self.changes.push_back((self.started.elapsed(), observed));
    }

    /// The values observed, when they changed at least once.
    fn changes_fact(&self) -> Option<Fact> {
        if self.changes.len() < 2 {
            return None;
        }
        let mut lines = Vec::new();
        if self.earlier_changes > 0 {
            lines.push(format!("({} earlier values)", self.earlier_changes));
        }
        for (at, value) in &self.changes {
            lines.push(format!("+{}: {value}", format_duration(*at)));
        }
        Some(Fact::labelled("Observed values", lines.join("\n")))
    }
}

/// `950ms`, `1.25s`.
fn format_duration(duration: Duration) -> String {
    if duration < Duration::from_secs(1) {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{failure::AssertionFailure, prelude::*};
    use alloc::sync::Arc;
    use core::cell::Cell;
    use std::sync::Mutex;

    /// A patience short enough for tests. Tests pass it explicitly, so a test changing the global
    /// patience never affects them.
    const QUICK: Patience = Patience::DEFAULT
        .within(Duration::from_millis(200))
        .polling_every(Duration::from_millis(5))
        .consistently_for(Duration::from_millis(40));

    /// An observation counting its calls, starting at 1.
    fn counter(calls: &Cell<u32>) -> impl Fn() -> core::future::Ready<u32> + '_ {
        move || {
            calls.set(calls.get() + 1);
            core::future::ready(calls.get())
        }
    }

    /// The structured failure that the eventual assertion built by `assertion` raises.
    async fn raised<Fut: Future>(
        assertion: impl FnOnce(crate::test_support::LocationRecorder) -> Fut,
    ) -> AssertionFailure {
        let raised_failure = Arc::new(Mutex::new(None));
        let sink = Arc::clone(&raised_failure);
        let pending = assertion(Box::new(move |failure: &AssertionFailure| {
            *sink.lock().unwrap() = Some(failure.clone());
            failure.to_string()
        }));
        assert_that_owned!(move || pending).panics_async().await;
        raised_failure
            .lock()
            .unwrap()
            .take()
            .expect("the assertion raised a failure")
    }

    fn fact(failure: &AssertionFailure, label: &str) -> Option<String> {
        failure
            .facts
            .iter()
            .find(|fact| fact.label == label)
            .map(|fact| fact.value.to_string())
    }

    mod eventually_matches {
        use super::*;
        use crate::matchers::{eq, ge};

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { 1 };
            assert_caller_location!(
                async assert_that!(observe),
                eventually().matches(eq(2))
            );
        }

        #[tokio::test]
        async fn observes_until_the_expectation_holds_and_continues_with_that_value() {
            let calls = Cell::new(0);
            let observe = counter(&calls);
            assert_that!(observe)
                .eventually()
                .with_patience(QUICK)
                .matches(ge(3))
                .await
                .is_equal_to(3);
            assert_that!(calls.get()).is_equal_to(3);
        }

        #[tokio::test]
        async fn counts_one_assertion_on_the_chain_and_its_parents() {
            let observe = || async { 1 };
            let parent = assert_that!(observe);
            parent
                .derive(|observe| observe)
                .eventually()
                .with_patience(QUICK)
                .matches(eq(1))
                .await;
            assert_that!(parent.state.records.assertion_count()).is_equal_to(1);
        }

        #[tokio::test]
        async fn fails_after_the_timeout_with_the_last_value_and_the_observed_values() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = counter(&calls);
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .with_subject_name("the counter")
                        .eventually()
                        .with_patience(QUICK.polling_every(Duration::from_millis(20)))
                        .matches(eq(0))
                        .await;
                }
            })
            .await;

            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
            assert_that!(failure.subject_name.as_deref()).is_equal_to(Some("the counter"));
            assert_that!(failure.subject_type_name).is_equal_to("u32");
            assert_that!(failure.expected.as_ref().map(ToString::to_string))
                .is_equal_to(Some("0".to_owned()));
            let last = calls.get();
            assert_that!(failure.actual.as_ref().map(ToString::to_string))
                .is_equal_to(Some(last.to_string()));
            assert_that!(fact(&failure, "Waited"))
                .get_some()
                .ends_with(format!("({last} observations)"));
            assert_that!(fact(&failure, "Observed values"))
                .get_some()
                .starts_with("(")
                .contains(format!(": {last}"));
        }

        #[tokio::test]
        async fn uses_the_global_patience_with_local_overrides() {
            let previous = Patience::global();
            QUICK.within(Duration::from_millis(30)).set_global();
            let resolved = Overrides::default()
                .polling_every(Duration::from_millis(1))
                .resolve();
            previous.set_global();

            assert_that!(resolved.timeout()).is_equal_to(Duration::from_millis(30));
            assert_that!(resolved.interval()).is_equal_to(Duration::from_millis(1));
            assert_that!(resolved.consistency()).is_equal_to(QUICK.consistency());
        }
    }

    mod futures {
        use super::*;
        use crate::matchers::eq;

        fn requires_send<F: Future + Send>(future: F) -> F {
            future
        }

        #[tokio::test]
        async fn are_send_while_awaiting_so_tests_may_move_between_threads() {
            let shared = Arc::new(Mutex::new(1));
            let observe = || {
                let shared = Arc::clone(&shared);
                async move { *shared.lock().unwrap() }
            };
            let parent = assert_that!(&observe).with_detail_message("the parent's message");
            requires_send(parent.eventually().with_patience(QUICK).matches(eq(1))).await;
            requires_send(
                assert_that!(observe)
                    .consistently()
                    .with_patience(QUICK)
                    .satisfies(|value| {
                        value.is_equal_to(1);
                    }),
            )
            .await;
        }

        #[tokio::test]
        async fn keep_the_messages_of_the_chain_and_its_parents() {
            let failure = raised(|presentation| async move {
                let observe = || async { 1 };
                let parent = assert_that_owned!(observe)
                    .with_panic_presentation(presentation)
                    .with_detail_message("the parent's message");
                parent
                    .derive(|observe| observe)
                    .with_detail_message("the child's message")
                    .eventually()
                    .with_patience(QUICK.within(Duration::ZERO))
                    .matches(eq(2))
                    .await;
            })
            .await;

            assert_that!(failure.messages)
                .contains_exactly(["the child's message", "the parent's message"]);
        }
    }

    mod eventually_satisfies {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { 1 };
            assert_caller_location!(
                async assert_that!(observe),
                eventually().satisfies(|it| {
                    it.is_equal_to(2);
                })
            );
        }

        #[tokio::test]
        async fn applies_the_assertions_to_every_observation() {
            let calls = Cell::new(0);
            let observe = counter(&calls);
            assert_that!(observe)
                .eventually()
                .with_patience(QUICK)
                .satisfies(|count| {
                    count.is_greater_than(1).is_less_than(10);
                })
                .await
                .is_equal_to(2);
        }

        #[tokio::test]
        async fn fails_with_the_failures_of_the_last_observation() {
            let failure = raised(|presentation| async move {
                let observe = || async { 5 };
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .eventually()
                    .with_patience(QUICK.within(Duration::from_millis(20)))
                    .satisfies(|count| {
                        count.is_less_than(3);
                    })
                    .await;
            })
            .await;

            assert_that!(failure.kind).is_equal_to(FailureKind::Matching);
            assert_that!(failure.children).has_length(1);
            assert_that!(failure.children[0].kind).is_equal_to(FailureKind::Ordering);
            // One value, so there is no history of changes.
            assert_that!(fact(&failure, "Observed values")).is_none();
        }
    }

    mod eventually_ok_matches {
        use super::*;
        use crate::matchers::eq;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { Ok::<_, &str>(1) };
            assert_caller_location!(
                async assert_that!(observe),
                eventually_ok().matches(eq(2))
            );
        }

        #[tokio::test]
        async fn retries_failed_observations() {
            let calls = Cell::new(0);
            let observe = || {
                calls.set(calls.get() + 1);
                core::future::ready(if calls.get() < 3 {
                    Err("not yet")
                } else {
                    Ok(calls.get())
                })
            };
            assert_that!(observe)
                .eventually_ok()
                .with_patience(QUICK)
                .matches(eq(3))
                .await
                .is_equal_to(3);
        }

        #[tokio::test]
        async fn fails_with_the_error_when_observing_keeps_failing() {
            let failure = raised(|presentation| async move {
                let observe = || async { Err::<u32, _>("stale element") };
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .eventually_ok()
                    .with_patience(QUICK.within(Duration::from_millis(20)))
                    .matches(eq(1))
                    .await;
            })
            .await;

            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"stale element\"".to_owned()));
        }
    }

    mod consistently_matches {
        use super::*;
        use crate::matchers::{eq, lt};

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { 1 };
            assert_caller_location!(
                async assert_that!(observe),
                consistently().matches(eq(2))
            );
        }

        #[tokio::test]
        async fn observes_for_the_duration_and_continues_with_the_last_value() {
            let calls = Cell::new(0);
            let observe = counter(&calls);
            let started = Instant::now();
            assert_that!(observe)
                .consistently()
                .with_patience(QUICK)
                .matches(lt(1000))
                .await
                .is_equal_to(calls.get());
            assert_that!(started.elapsed()).is_greater_or_equal_to(QUICK.consistency());
            assert_that!(calls.get()).is_greater_than(1);
        }

        #[tokio::test]
        async fn fails_with_the_first_value_breaking_the_expectation() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = counter(&calls);
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .consistently()
                        .with_patience(QUICK.consistently_for(Duration::from_secs(10)))
                        .matches(lt(3))
                        .await;
                }
            })
            .await;

            assert_that!(calls.get()).is_equal_to(3);
            assert_that!(failure.actual.as_ref().map(ToString::to_string))
                .is_equal_to(Some("3".to_owned()));
            assert_that!(fact(&failure, "Held"))
                .get_some()
                .ends_with("(3 observations), then not");
            assert_that!(fact(&failure, "Observed values"))
                .get_some()
                .contains(": 1\n")
                .contains(": 3");
        }

        #[tokio::test]
        async fn for_at_least_overrides_the_duration() {
            let observe = || async { 1 };
            let started = Instant::now();
            assert_that!(observe)
                .consistently()
                .with_patience(QUICK)
                .for_at_least(Duration::from_millis(80))
                .matches(eq(1))
                .await;
            assert_that!(started.elapsed()).is_greater_or_equal_to(Duration::from_millis(80));
        }
    }

    mod consistently_satisfies {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { 1 };
            assert_caller_location!(
                async assert_that!(observe),
                consistently().satisfies(|it| {
                    it.is_equal_to(2);
                })
            );
        }
    }

    mod consistently_ok_matches {
        use super::*;
        use crate::matchers::eq;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { Ok::<_, &str>(1) };
            assert_caller_location!(
                async assert_that!(observe),
                consistently_ok().matches(eq(2))
            );
        }

        #[tokio::test]
        async fn fails_on_the_first_failed_observation() {
            let failure = raised(|presentation| async move {
                let observe = || async { Err::<u32, _>("gone") };
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .consistently_ok()
                    .with_patience(QUICK)
                    .matches(eq(1))
                    .await;
            })
            .await;

            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(fact(&failure, "Held"))
                .get_some()
                .ends_with("(1 observations), then not");
        }
    }
}
