//! Eventual assertions: observation, patience, and the private timer behind them.
//!
//! The public builders and retry policies are re-exported from
//! [`assertions::eventually`](crate::assertions::eventually), which documents them.

mod patience;
mod sleep;

use core::{marker::PhantomData, panic::Location, time::Duration};
use std::{collections::VecDeque, time::Instant};

use patience::Overrides;
pub use patience::Patience;

use crate::{
    AssertThat,
    actual::Actual,
    assert_that::DetachedChain,
    expectation::Expectation,
    failure::{AssertionFailure, Fact, FailureBuilder, FailureKind},
    matchers::satisfying,
    mode::{Capture, Panic},
    renderer::{Rendered, ValueRenderer},
};

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
/// assertion itself: `matches` or `satisfies`, or `try_matches` to return the failure instead of
/// panicking. They are available in panic mode, and the subject may be borrowed or owned.
///
/// The returned futures keep no chain records. They are `Send` whenever the observation, the
/// expectation, the renderer, and a [`giving_up_on`](Eventually::giving_up_on) closure are, and
/// they run in any async runtime. See
/// [`assertions::eventually`](crate::assertions::eventually) for an example.
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
pub struct Eventually<C, K, G = KeepRetrying> {
    chain: C,
    overrides: Overrides,
    kind: PhantomData<K>,
    give_up: G,
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

/// Retries every failed observation until the timeout: what
/// [`eventually_ok`](EventualAssertions::eventually_ok) does unless
/// [`giving_up_on`](Eventually::giving_up_on) says otherwise.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeepRetrying;

/// Ends the assertion at its first failed observation: what
/// [`giving_up_on_any_error`](Eventually::giving_up_on_any_error) selects.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyError;

/// Decides which failed observations end an [`eventually_ok`](EventualAssertions::eventually_ok)
/// assertion at once instead of being retried. Implemented by [`KeepRetrying`], [`AnyError`],
/// and closures `Fn(&E) -> bool`.
///
/// This trait is sealed. Pass a closure to [`giving_up_on`](Eventually::giving_up_on) for a
/// custom policy.
pub trait GiveUp<E>: sealed::Sealed<E> {
    /// Whether no retry can turn this observation's `error` into a value.
    fn gives_up_on(&self, error: &E) -> bool;
}

mod sealed {
    /// Restricts [`GiveUp`](super::GiveUp) to the policies of this module.
    pub trait Sealed<E> {}

    impl<E> Sealed<E> for super::KeepRetrying {}
    impl<E> Sealed<E> for super::AnyError {}
    impl<E, P: Fn(&E) -> bool> Sealed<E> for P {}
}

impl<E> GiveUp<E> for KeepRetrying {
    fn gives_up_on(&self, _: &E) -> bool {
        false
    }
}

impl<E> GiveUp<E> for AnyError {
    fn gives_up_on(&self, _: &E) -> bool {
        true
    }
}

impl<E, P: Fn(&E) -> bool> GiveUp<E> for P {
    fn gives_up_on(&self, error: &E) -> bool {
        self(error)
    }
}

impl<C, K> Eventually<C, K> {
    fn new(chain: C) -> Self {
        Self {
            chain,
            overrides: Overrides::default(),
            kind: PhantomData,
            give_up: KeepRetrying,
        }
    }
}

impl<C, K, G> Eventually<C, K, G> {
    /// Waits up to `timeout` for the expectation instead of the [`Patience`]'s
    /// [timeout](Patience::timeout).
    pub fn within(mut self, timeout: Duration) -> Self {
        self.overrides = self.overrides.with_timeout(timeout);
        self
    }

    /// Pauses `interval` between two observations instead of the [`Patience`]'s
    /// [interval](Patience::interval).
    pub fn polling_every(mut self, interval: Duration) -> Self {
        self.overrides = self.overrides.with_interval(interval);
        self
    }

    /// Starts from `patience` instead of the [global](Patience::global) patience.
    ///
    /// [`within`](Self::within) and [`polling_every`](Self::polling_every) override single
    /// settings of it, whether they are called before or after this method.
    pub fn with_patience(mut self, patience: Patience) -> Self {
        self.overrides = self.overrides.with_base(patience);
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

    /// Requires the assertion to hold for `duration` instead of the [`Patience`]'s
    /// [consistency duration](Patience::consistency_duration), for example past a timer the check
    /// must outlast.
    pub fn for_at_least(mut self, duration: Duration) -> Self {
        self.overrides = self.overrides.with_consistency_duration(duration);
        self
    }

    /// Waits up to `timeout` for each observation to complete instead of the [`Patience`]'s
    /// [observation timeout](Patience::observation_timeout). An observation still pending after
    /// it fails the assertion.
    pub fn each_observation_within(mut self, timeout: Duration) -> Self {
        self.overrides = self.overrides.with_observation_timeout(timeout);
        self
    }

    /// Pauses `interval` between two observations instead of the [`Patience`]'s
    /// [interval](Patience::interval).
    pub fn polling_every(mut self, interval: Duration) -> Self {
        self.overrides = self.overrides.with_interval(interval);
        self
    }

    /// Starts from `patience` instead of the [global](Patience::global) patience.
    ///
    /// [`for_at_least`](Self::for_at_least),
    /// [`each_observation_within`](Self::each_observation_within), and
    /// [`polling_every`](Self::polling_every) override single settings of it, whether they are
    /// called before or after this method.
    pub fn with_patience(mut self, patience: Patience) -> Self {
        self.overrides = self.overrides.with_base(patience);
        self
    }
}

impl<'t, F, Fut, K, G, R> Eventually<AssertThat<'t, F, Panic, R>, K, G>
where
    F: Fn() -> Fut,
    Fut: Future,
{
    /// Starts observing at the assertion's call, keeping the give-up policy for the extractor.
    #[track_caller]
    fn start(self) -> (EventualRun<'t, F, R>, G) {
        let run = EventualRun::start(self.chain, Location::caller(), self.overrides);
        (run, self.give_up)
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
        let (run, KeepRetrying) = self.start();
        let outcome = run.until(expected, plain);
        async move { outcome.await.into_chain() }
    }

    /// Returns the observed value or a structured assertion failure, without panicking.
    /// User code panics are not caught. Timing, retry policy, rendering and caller metadata
    /// are the same as `matches`. Pass `satisfying(|it| ..)` to use an assertion callback.
    ///
    /// # Errors
    /// Returns the structured assertion failure when observation or the expectation fails.
    ///
    /// # Examples
    /// A shared helper can propagate the failure with `?` instead of panicking:
    ///
    /// ```rust
    /// use assertr::{failure::AssertionFailure, matchers::eq, prelude::*};
    ///
    /// async fn wait_until_ready(
    ///     status: impl Fn() -> std::future::Ready<&'static str>,
    /// ) -> Result<(), Box<AssertionFailure>> {
    ///     assert_that!(status).eventually().try_matches(eq("ready")).await?;
    ///     Ok(())
    /// }
    ///
    /// # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
    /// assert_that!(wait_until_ready(|| std::future::ready("ready")).await).is_ok();
    /// assert_that!(wait_until_ready(|| std::future::ready("starting")).await).is_err();
    /// # });
    /// ```
    #[track_caller]
    pub fn try_matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = Result<T, Box<AssertionFailure>>>
    where
        R: ValueRenderer<T>,
    {
        let (run, KeepRetrying) = self.start();
        let outcome = run.until(expected, plain);
        async move { outcome.await.into_result() }
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
    /// Ends the assertion at the first failed observation whose error `give_up` accepts, instead
    /// of retrying it until the timeout: for errors no retry can fix, such as an element the page
    /// removed. The failure shows that error.
    ///
    /// ```rust
    /// use assertr::{matchers::eq, prelude::*};
    ///
    /// #[derive(Debug)]
    /// enum ReadError {
    ///     NotYetAvailable,
    ///     Removed,
    /// }
    ///
    /// # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
    /// let read = || async { Err::<u32, _>(ReadError::Removed) };
    /// let failure = assert_that!(read)
    ///     .eventually_ok()
    ///     .giving_up_on(|error| matches!(error, ReadError::Removed))
    ///     .try_matches(eq(1))
    ///     .await
    ///     .unwrap_err();
    /// assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
    /// # let _ = ReadError::NotYetAvailable;
    /// # });
    /// ```
    pub fn giving_up_on<G>(self, give_up: G) -> Eventually<AssertThat<'t, F, Panic, R>, Fallible, G>
    where
        G: Fn(&E) -> bool,
    {
        Eventually {
            chain: self.chain,
            overrides: self.overrides,
            kind: PhantomData,
            give_up,
        }
    }

    /// Ends the assertion at the first failed observation, instead of retrying it until the
    /// timeout: for observations of a fixed resource, such as one browser element, where every
    /// error is final. The failure shows that error. Only values that do not meet the
    /// expectation are observed again.
    pub fn giving_up_on_any_error(
        self,
    ) -> Eventually<AssertThat<'t, F, Panic, R>, Fallible, AnyError> {
        Eventually {
            chain: self.chain,
            overrides: self.overrides,
            kind: PhantomData,
            give_up: AnyError,
        }
    }
}

impl<'t, F, Fut, T: 't, E, R, G> Eventually<AssertThat<'t, F, Panic, R>, Fallible, G>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    G: GiveUp<E>,
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
        let (run, give_up) = self.start();
        let outcome = run.until(expected, fallible(give_up));
        async move { outcome.await.into_chain() }
    }

    /// Returns the observed value or a structured assertion failure, without panicking.
    /// User code panics are not caught. Timing, retry policy, rendering and caller metadata
    /// are the same as `matches`. Pass `satisfying(|it| ..)` to use an assertion callback.
    ///
    /// # Errors
    /// Returns the structured assertion failure when observation or the expectation fails.
    #[track_caller]
    pub fn try_matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = Result<T, Box<AssertionFailure>>>
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        let (run, give_up) = self.start();
        let outcome = run.until(expected, fallible(give_up));
        async move { outcome.await.into_result() }
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

impl<'t, F, Fut, K, R> Consistently<AssertThat<'t, F, Panic, R>, K>
where
    F: Fn() -> Fut,
    Fut: Future,
{
    /// Starts observing at the assertion's call.
    #[track_caller]
    fn start(self) -> EventualRun<'t, F, R> {
        EventualRun::start(self.chain, Location::caller(), self.overrides)
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
        let outcome = self.start().throughout(expected, plain);
        async move { outcome.await.into_chain() }
    }

    /// Returns the observed value or a structured assertion failure, without panicking.
    /// User code panics are not caught. Timing, retry policy, rendering and caller metadata
    /// are the same as `matches`. Pass `satisfying(|it| ..)` to use an assertion callback.
    ///
    /// # Errors
    /// Returns the structured assertion failure when observation or the expectation fails.
    #[track_caller]
    pub fn try_matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = Result<T, Box<AssertionFailure>>>
    where
        R: ValueRenderer<T>,
    {
        let outcome = self.start().throughout(expected, plain);
        async move { outcome.await.into_result() }
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
        let outcome = self.start().throughout(expected, fallible(AnyError));
        async move { outcome.await.into_chain() }
    }

    /// Returns the observed value or a structured assertion failure, without panicking.
    /// User code panics are not caught. Timing, retry policy, rendering and caller metadata
    /// are the same as `matches`. Pass `satisfying(|it| ..)` to use an assertion callback.
    ///
    /// # Errors
    /// Returns the structured assertion failure when observation or the expectation fails.
    #[track_caller]
    pub fn try_matches<D: Expectation<T, R>>(
        self,
        expected: D,
    ) -> impl Future<Output = Result<T, Box<AssertionFailure>>>
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
    {
        let outcome = self.start().throughout(expected, fallible(AnyError));
        async move { outcome.await.into_result() }
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

/// Takes the output of an observation that cannot fail as the observed value.
#[allow(clippy::unnecessary_wraps)] // Shares the extractor signature of `fallible`.
fn plain<T, R>(output: T, _: &DetachedChain<R>) -> Observed<T> {
    Ok(output)
}

/// Takes the `Ok` value of a fallible observation, rendering an `Err` and whether `give_up`
/// ends the assertion with it.
fn fallible<T, E, R: ValueRenderer<E>>(
    give_up: impl GiveUp<E>,
) -> impl Fn(Result<T, E>, &DetachedChain<R>) -> Observed<T> {
    move |output, chain| {
        output.map_err(|error| FailedObservation {
            gives_up: give_up.gives_up_on(&error),
            error: chain.render().value(&error),
        })
    }
}

/// The result of one observation: its value, or how it failed.
type Observed<T> = Result<T, FailedObservation>;

/// An observation that failed.
struct FailedObservation {
    /// The observation's error, rendered.
    error: Rendered,
    /// Whether the assertion ends with it instead of retrying.
    gives_up: bool,
}

/// What one observation led to.
enum Step<T> {
    /// The expectation holds for this value.
    Met(T),
    /// The expectation does not hold, and the assertion fails with this explanation.
    Failed(Box<FailureBuilder>),
    /// The expectation does not hold yet. The observation is kept unexplained, to explain it
    /// should no later observation complete before the timeout.
    Unmet(Observed<T>),
}

/// Shared execution result. Public boundaries choose propagation or panic presentation.
struct Outcome<T, R> {
    result: Result<T, FailureBuilder>,
    detached_chain: DetachedChain<R>,
    location: &'static Location<'static>,
}

impl<T, R> Outcome<T, R> {
    fn into_result(self) -> Result<T, Box<AssertionFailure>> {
        self.result
            .map_err(|failure| Box::new(self.detached_chain.complete_at(failure, self.location)))
    }

    fn into_chain<'t>(self) -> AssertThat<'t, T, Panic, R>
    where
        T: 't,
    {
        match self.result {
            Ok(value) => self.detached_chain.attach(Actual::Owned(value)),
            Err(failure) => self.detached_chain.raise_at(failure, self.location),
        }
    }
}

/// A started eventual assertion: the observation, separated from its [detached](DetachedChain)
/// chain. It holds no chain records, so its future is `Send` whenever the observation, the
/// expectation and the renderer are.
struct EventualRun<'t, F, R> {
    actual: Actual<'t, F>,
    detached_chain: DetachedChain<R>,
    location: &'static Location<'static>,
    patience: Patience,
}

impl<'t, F, Fut, R> EventualRun<'t, F, R>
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
        let (actual, detached_chain) = chain.into_parts();
        Self {
            actual,
            detached_chain,
            location,
            patience: overrides.resolve(),
        }
    }

    /// Observes until `expected` holds or the timeout passes, then fails with the last completed
    /// observation. An observation still pending at the timeout is abandoned. The assertion then
    /// fails with the observation before it, or as unobserved when none completed.
    async fn until<T: 't, D>(
        self,
        expected: D,
        extract: impl Fn(Fut::Output, &DetachedChain<R>) -> Observed<T>,
    ) -> Outcome<T, R>
    where
        D: Expectation<T, R>,
        R: ValueRenderer<T>,
    {
        let mut history = History::start();
        let deadline = history.started.checked_add(self.patience.timeout());
        let mut unmet = None;
        loop {
            let Some(output) = sleep::before((self.actual.borrowed())(), deadline).await else {
                let last =
                    unmet.map(|observed| judge(&self.detached_chain, observed, &expected, true));
                let failure = match last {
                    Some(Step::Met(value)) => return self.finish(Ok(value), &history),
                    Some(Step::Failed(failure)) => failure.fact(Fact::note(
                        "A later observation did not complete before the timeout.",
                    )),
                    Some(Step::Unmet(_)) => {
                        unreachable!("an explained rejection ends the assertion")
                    }
                    None => not_observed::<T>().fact(Fact::note(
                        "The observation did not complete before the timeout.",
                    )),
                };
                return self.finish(Err(failure.fact(history.waited())), &history);
            };
            let last_attempt = deadline.is_some_and(|deadline| Instant::now() >= deadline);
            let observed = extract(output, &self.detached_chain);
            match observe_once(
                &self.detached_chain,
                &mut history,
                observed,
                &expected,
                last_attempt,
            ) {
                Step::Met(value) => return self.finish(Ok(value), &history),
                Step::Failed(failure) => {
                    return self.finish(Err(failure.fact(history.waited())), &history);
                }
                Step::Unmet(observed) => {
                    unmet = Some(observed);
                    self.pause(deadline).await;
                }
            }
        }
    }

    /// Observes for the consistency duration, failing with the first observation that does not
    /// meet `expected`. An observation still pending after the observation timeout fails the
    /// assertion.
    async fn throughout<T: 't, D>(
        self,
        expected: D,
        extract: impl Fn(Fut::Output, &DetachedChain<R>) -> Observed<T>,
    ) -> Outcome<T, R>
    where
        D: Expectation<T, R>,
        R: ValueRenderer<T>,
    {
        let mut history = History::start();
        let deadline = history
            .started
            .checked_add(self.patience.consistency_duration());
        loop {
            let observation_deadline =
                Instant::now().checked_add(self.patience.observation_timeout());
            let Some(output) =
                sleep::before((self.actual.borrowed())(), observation_deadline).await
            else {
                let failure = not_observed::<T>().fact(Fact::note(format!(
                    "The observation did not complete within {}.",
                    format_duration(self.patience.observation_timeout())
                )));
                let held = history.held(history.observations);
                return self.finish(Err(failure.fact(held)), &history);
            };
            let observed = extract(output, &self.detached_chain);
            match observe_once(
                &self.detached_chain,
                &mut history,
                observed,
                &expected,
                true,
            ) {
                Step::Met(value) if deadline.is_some_and(|deadline| Instant::now() >= deadline) => {
                    return self.finish(Ok(value), &history);
                }
                Step::Met(_) => self.pause(deadline).await,
                Step::Failed(failure) => {
                    // The last observation is the one that did not hold.
                    let held = history.held(history.observations - 1);
                    return self.finish(Err(failure.fact(held)), &history);
                }
                Step::Unmet(_) => unreachable!("an explained rejection ends the assertion"),
            }
        }
    }

    /// Ends the assertion with `result`, adding the observed values to a failure.
    fn finish<T>(self, result: Result<T, FailureBuilder>, history: &History) -> Outcome<T, R> {
        Outcome {
            result: result.map_err(|failure| failure.facts(history.changes_fact())),
            detached_chain: self.detached_chain,
            location: self.location,
        }
    }

    /// The pause before the next observation: one interval, but never past `deadline`.
    fn pause(&self, deadline: Option<Instant>) -> sleep::Sleep {
        let next = Instant::now().checked_add(self.patience.interval());
        sleep::until(match (next, deadline) {
            (Some(next), Some(deadline)) => Some(next.min(deadline)),
            (next, deadline) => next.or(deadline),
        })
    }
}

/// The failure of an observation that produced no value.
pub(crate) fn not_observed<T>() -> FailureBuilder {
    FailureBuilder::new::<T>(FailureKind::Other).relation("could not be observed")
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
    history.record(match &observed {
        Ok(value) => chain.render().value(value),
        Err(failed) => failed.error.clone(),
    });
    judge(chain, observed, expected, explain)
}

/// Evaluates `expected` on an observation without recording it. A rejection is explained when
/// `explain` is set or a failed observation gives up, and kept as unmet otherwise.
fn judge<T, R, D>(
    chain: &DetachedChain<R>,
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
        Err(failed) if explain || failed.gives_up => {
            let failure = not_observed::<T>().fact(Fact::labelled("Error", failed.error));
            return Step::Failed(Box::new(failure));
        }
        Err(failed) => return Step::Unmet(Err(failed)),
    };
    let context = chain.assertion_context();
    let rejected = match expected.evaluate(&value, &context) {
        Ok(_) => false,
        Err(rejection) if explain => {
            return Step::Failed(Box::new(expected.explain(
                Some((&value, rejection)),
                FailureBuilder::new::<T>(D::KIND),
                &context,
            )));
        }
        Err(_) => true,
    };
    if rejected {
        Step::Unmet(Ok(value))
    } else {
        Step::Met(value)
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

    /// How long and how often the subject was observed.
    fn waited(&self) -> Fact {
        Fact::labelled(
            "Waited",
            format!(
                "{} ({})",
                format_duration(self.started.elapsed()),
                observation_count(self.observations)
            ),
        )
    }

    /// How long the expectation held, over its first `held` observations.
    fn held(&self, held: usize) -> Fact {
        let value = if held == 0 {
            "never".to_owned()
        } else {
            format!(
                "for {} ({}), then not",
                format_duration(self.started.elapsed()),
                observation_count(held)
            )
        };
        Fact::labelled("Held", value)
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

/// `1 observation`, `3 observations`.
fn observation_count(count: usize) -> String {
    if count == 1 {
        "1 observation".to_owned()
    } else {
        format!("{count} observations")
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
    use core::cell::Cell;

    use super::*;
    use crate::{
        failure::AssertionFailure,
        prelude::*,
        test_support::{LocationRecorder, recording_presentation},
    };

    /// A patience short enough for tests. Tests pass it explicitly, so a test changing the global
    /// patience never affects them.
    const QUICK: Patience = Patience::DEFAULT
        .with_timeout(Duration::from_millis(200))
        .with_interval(Duration::from_millis(5))
        .with_consistency_duration(Duration::from_millis(40))
        .with_observation_timeout(Duration::from_millis(200));

    /// An observation counting its calls, starting at 1.
    fn counter(calls: &Cell<u32>) -> impl Fn() -> core::future::Ready<u32> + '_ {
        move || {
            calls.set(calls.get() + 1);
            core::future::ready(calls.get())
        }
    }

    /// The structured failure that the eventual assertion built by `assertion` raises.
    ///
    /// The async counterpart of [`crate::test_support::raised_failure`] for `#[tokio::test]`s.
    async fn raised<Fut: Future>(
        assertion: impl FnOnce(LocationRecorder) -> Fut,
    ) -> AssertionFailure {
        let (presentation, slot) = recording_presentation();
        let pending = assertion(presentation);
        assert_that!(move || pending).panics_async().await;
        slot.lock()
            .unwrap()
            .take()
            .expect("the assertion raised a failure")
    }

    /// Passes `expected` through, recording `line`: the line of the call it is an argument of.
    fn on_line<D>(recorded: &Cell<u32>, line: u32, expected: D) -> D {
        recorded.set(line);
        expected
    }

    /// Asserts that `failure` was raised on `line` of this file.
    fn assert_raised_on(failure: &AssertionFailure, line: u32) {
        assert_that!(
            failure
                .location
                .map(|location| (location.file(), location.line()))
        )
        .is_equal_to(Some((file!(), line)));
    }

    /// Requires `future` to be `Send`, so a test awaiting it may move between threads.
    fn requires_send<F: Future + Send>(future: F) -> F {
        future
    }

    fn fact(failure: &AssertionFailure, label: &str) -> Option<String> {
        failure
            .facts
            .iter()
            .find(|fact| fact.label.as_deref() == Some(label))
            .map(|fact| fact.value.to_string())
    }

    fn note(failure: &AssertionFailure) -> Option<String> {
        failure
            .facts
            .iter()
            .find(|fact| fact.label.is_none())
            .map(|fact| fact.value.to_string())
    }

    mod eventually_try_matches {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn caller_location_is_as_expected() {
            let line = Cell::new(0);
            let failure = assert_that!(|| async { 1 })
                .eventually()
                .within(Duration::ZERO)
                .try_matches(on_line(&line, line!(), eq(2)))
                .await
                .unwrap_err();
            assert_raised_on(&failure, line.get());
        }

        #[tokio::test]
        async fn returns_the_value_or_the_failure() {
            let observe = || async { 1 };
            let passing = assert_that!(observe).eventually().within(Duration::ZERO);
            assert_that!(passing.try_matches(eq(1)).await.ok()).is_equal_to(Some(1));
            let failure = assert_that!(observe)
                .eventually()
                .within(Duration::ZERO)
                .try_matches(eq(2))
                .await
                .unwrap_err();
            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
        }
    }

    mod eventually_ok_try_matches {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn caller_location_is_as_expected() {
            let line = Cell::new(0);
            let failure = assert_that!(|| async { Ok::<_, &str>(1) })
                .eventually_ok()
                .within(Duration::ZERO)
                .try_matches(on_line(&line, line!(), eq(2)))
                .await
                .unwrap_err();
            assert_raised_on(&failure, line.get());
        }

        #[tokio::test]
        async fn returns_the_value_or_the_failure() {
            let observe = || async { Ok::<_, &str>(1) };
            let passing = assert_that!(observe).eventually_ok().within(Duration::ZERO);
            assert_that!(passing.try_matches(eq(1)).await.ok()).is_equal_to(Some(1));
            let failure = assert_that!(observe)
                .eventually_ok()
                .within(Duration::ZERO)
                .try_matches(eq(2))
                .await
                .unwrap_err();
            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
        }
    }

    mod consistently_try_matches {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn caller_location_is_as_expected() {
            let line = Cell::new(0);
            let failure = assert_that!(|| async { 1 })
                .consistently()
                .for_at_least(Duration::ZERO)
                .try_matches(on_line(&line, line!(), eq(2)))
                .await
                .unwrap_err();
            assert_raised_on(&failure, line.get());
        }

        #[tokio::test]
        async fn returns_the_value_or_the_failure() {
            let observe = || async { 1 };
            let passing = assert_that!(observe)
                .consistently()
                .for_at_least(Duration::ZERO);
            assert_that!(passing.try_matches(eq(1)).await.ok()).is_equal_to(Some(1));
            let failure = assert_that!(observe)
                .consistently()
                .for_at_least(Duration::ZERO)
                .try_matches(eq(2))
                .await
                .unwrap_err();
            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
        }
    }

    mod consistently_ok_try_matches {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn caller_location_is_as_expected() {
            let line = Cell::new(0);
            let failure = assert_that!(|| async { Ok::<_, &str>(1) })
                .consistently_ok()
                .for_at_least(Duration::ZERO)
                .try_matches(on_line(&line, line!(), eq(2)))
                .await
                .unwrap_err();
            assert_raised_on(&failure, line.get());
        }

        #[tokio::test]
        async fn returns_the_value_or_the_failure() {
            let observe = || async { Ok::<_, &str>(1) };
            let passing = assert_that!(observe)
                .consistently_ok()
                .for_at_least(Duration::ZERO);
            assert_that!(passing.try_matches(eq(1)).await.ok()).is_equal_to(Some(1));
            let failure = assert_that!(observe)
                .consistently_ok()
                .for_at_least(Duration::ZERO)
                .try_matches(eq(2))
                .await
                .unwrap_err();
            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
        }
    }

    /// Behavior shared by every `try_matches`, which all return through the same outcome.
    mod try_matches_execution {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn returned_failures_keep_the_chain_metadata_without_presenting_them() {
            let failure = assert_that!(|| async { 1 })
                .with_subject_name("sample")
                .with_detail_message("context")
                .with_panic_presentation(|_| panic!("must not present a returned failure"))
                .eventually()
                .within(Duration::ZERO)
                .try_matches(eq(2))
                .await
                .unwrap_err();
            assert_that!(failure.subject_name.as_deref()).is_equal_to(Some("sample"));
            assert_that!(failure.messages).contains("context".to_owned());
        }

        #[tokio::test]
        async fn returns_terminal_observation_errors_without_retrying() {
            let calls = Cell::new(0);
            let failure = assert_that!(|| {
                calls.set(calls.get() + 1);
                async { Err::<u8, _>("disconnected") }
            })
            .eventually_ok()
            .giving_up_on_any_error()
            .try_matches(eq(1))
            .await
            .unwrap_err();
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"disconnected\"".to_owned()));
        }

        #[tokio::test]
        async fn returned_futures_are_send_and_pending_reads_time_out() {
            let future = assert_that_owned!(std::future::pending::<u8>)
                .eventually()
                .with_patience(QUICK)
                .try_matches(eq(1));
            let failure = requires_send(future).await.unwrap_err();
            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
        }
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
                        .with_patience(QUICK.with_interval(Duration::from_millis(20)))
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
                .some()
                .ends_with(format!("({last} observations)"));
            assert_that!(fact(&failure, "Observed values"))
                .some()
                .starts_with("(")
                .contains(format!(": {last}"));
        }
    }

    mod futures {
        use alloc::sync::Arc;
        use std::sync::Mutex;

        use super::*;
        use crate::matchers::eq;

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
                    .with_patience(QUICK.with_timeout(Duration::ZERO))
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
                    .with_patience(QUICK.with_timeout(Duration::from_millis(20)))
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

        #[test]
        fn giving_up_keeps_the_caller_location() {
            let expected_line = Cell::new(0);
            let failure = crate::test_support::raised_failure(|presentation| {
                crate::test_support::block_on(async {
                    let observe = || async { Err::<u32, _>("gone") };
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually_ok()
                        .giving_up_on(|error: &&str| *error == "gone")
                        .matches(on_line(&expected_line, line!(), eq(1)))
                        .await;
                });
            })
            .expect("the assertion raised a failure");
            assert_raised_on(&failure, expected_line.get());
        }

        #[tokio::test]
        async fn gives_up_at_once_on_an_accepted_error() {
            let calls = Cell::new(0);
            let started = Instant::now();
            let failure = raised(|presentation| {
                let observe = || {
                    calls.set(calls.get() + 1);
                    core::future::ready(if calls.get() < 2 {
                        Err("not yet")
                    } else {
                        Err("gone")
                    })
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually_ok()
                        .with_patience(QUICK.with_timeout(Duration::from_secs(10)))
                        .giving_up_on(|error: &&str| *error == "gone")
                        .matches(eq(1_u32))
                        .await;
                }
            })
            .await;

            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(started.elapsed()).is_less_than(Duration::from_secs(1));
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"gone\"".to_owned()));
        }

        #[tokio::test]
        async fn retries_errors_it_does_not_give_up_on() {
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
                .giving_up_on(|error: &&str| *error == "gone")
                .matches(eq(3))
                .await;
        }

        #[tokio::test]
        async fn fails_with_the_error_when_observing_keeps_failing() {
            let failure = raised(|presentation| async move {
                let observe = || async { Err::<u32, _>("stale element") };
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .eventually_ok()
                    .with_patience(QUICK.with_timeout(Duration::from_millis(20)))
                    .matches(eq(1))
                    .await;
            })
            .await;

            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"stale element\"".to_owned()));
        }
    }

    mod giving_up_on_any_error {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn ends_at_the_first_error_but_observes_unmet_values_again() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = || {
                    calls.set(calls.get() + 1);
                    core::future::ready(if calls.get() < 3 {
                        Ok(calls.get())
                    } else {
                        Err("gone")
                    })
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually_ok()
                        .with_patience(QUICK.with_timeout(Duration::from_secs(10)))
                        .giving_up_on_any_error()
                        .matches(eq(0_u32))
                        .await;
                }
            })
            .await;

            assert_that!(calls.get()).is_equal_to(3);
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"gone\"".to_owned()));
        }
    }

    mod eventually_ok_satisfies {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { Ok::<_, &str>(1) };
            assert_caller_location!(
                async assert_that!(observe),
                eventually_ok().satisfies(|it| {
                    it.is_equal_to(2);
                })
            );
        }

        #[tokio::test]
        async fn applies_the_assertions_to_every_successful_observation() {
            let calls = Cell::new(0);
            let observe = || {
                calls.set(calls.get() + 1);
                core::future::ready(if calls.get() < 2 {
                    Err("not yet")
                } else {
                    Ok(calls.get())
                })
            };
            assert_that!(observe)
                .eventually_ok()
                .with_patience(QUICK)
                .satisfies(|count| {
                    count.is_greater_than(2);
                })
                .await
                .is_equal_to(3);
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
            assert_that!(started.elapsed()).is_greater_or_equal_to(QUICK.consistency_duration());
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
                        .with_patience(QUICK.with_consistency_duration(Duration::from_secs(10)))
                        .matches(lt(3))
                        .await;
                }
            })
            .await;

            assert_that!(calls.get()).is_equal_to(3);
            assert_that!(failure.actual.as_ref().map(ToString::to_string))
                .is_equal_to(Some("3".to_owned()));
            assert_that!(fact(&failure, "Held"))
                .some()
                .ends_with("(2 observations), then not");
            assert_that!(fact(&failure, "Observed values"))
                .some()
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

        #[tokio::test]
        async fn fails_with_the_failures_of_the_first_breaking_observation() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = counter(&calls);
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .consistently()
                        .with_patience(QUICK.with_consistency_duration(Duration::from_secs(10)))
                        .satisfies(|count| {
                            count.is_less_than(2);
                        })
                        .await;
                }
            })
            .await;

            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(failure.kind).is_equal_to(FailureKind::Matching);
            assert_that!(failure.children).has_length(1);
            assert_that!(failure.children[0].kind).is_equal_to(FailureKind::Ordering);
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
            assert_that!(fact(&failure, "Held")).is_equal_to(Some("never".to_owned()));
        }

        #[tokio::test]
        async fn counts_one_held_observation_in_the_singular() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = || {
                    calls.set(calls.get() + 1);
                    core::future::ready(if calls.get() < 2 { Ok(1) } else { Err("gone") })
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .consistently_ok()
                        .with_patience(QUICK.with_consistency_duration(Duration::from_secs(10)))
                        .matches(eq(1))
                        .await;
                }
            })
            .await;

            assert_that!(fact(&failure, "Held"))
                .some()
                .ends_with("(1 observation), then not");
        }
    }

    mod consistently_ok_satisfies {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let observe = || async { Ok::<_, &str>(1) };
            assert_caller_location!(
                async assert_that!(observe),
                consistently_ok().satisfies(|it| {
                    it.is_equal_to(2);
                })
            );
        }

        #[tokio::test]
        async fn applies_the_assertions_to_every_successful_observation() {
            let observe = || async { Ok::<_, &str>(1) };
            assert_that!(observe)
                .consistently_ok()
                .with_patience(QUICK)
                .satisfies(|value| {
                    value.is_less_than(2);
                })
                .await
                .is_equal_to(1);
        }
    }

    mod unfinished_observations {
        use super::*;
        use crate::matchers::eq;

        /// One observation, then a pause that the timeout cuts short, so the second observation
        /// starts at the timeout and is abandoned as soon as it is pending.
        const SECOND_ATTEMPT_AT_THE_TIMEOUT: Patience = QUICK
            .with_timeout(Duration::from_millis(50))
            .with_interval(Duration::from_secs(10));

        #[tokio::test]
        async fn fail_eventually_at_the_timeout() {
            let started = Instant::now();
            let failure = raised(|presentation| async move {
                let observe = core::future::pending::<u32>;
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .eventually()
                    .with_patience(QUICK.with_timeout(Duration::from_millis(20)))
                    .matches(eq(1))
                    .await;
            })
            .await;

            assert_that!(started.elapsed()).is_less_than(Duration::from_secs(1));
            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(note(&failure)).is_equal_to(Some(
                "The observation did not complete before the timeout.".to_owned(),
            ));
            assert_that!(fact(&failure, "Waited"))
                .some()
                .ends_with("(0 observations)");
        }

        #[tokio::test]
        async fn explain_eventually_the_last_completed_observation() {
            let calls = Cell::new(0);
            let failure = raised(|presentation| {
                let observe = || {
                    calls.set(calls.get() + 1);
                    let call = calls.get();
                    async move {
                        tokio::task::yield_now().await;
                        call
                    }
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually()
                        .with_patience(SECOND_ATTEMPT_AT_THE_TIMEOUT)
                        .matches(eq(0))
                        .await;
                }
            })
            .await;

            assert_that!(failure.kind).is_equal_to(FailureKind::Equality);
            assert_that!(failure.expected.as_ref().map(ToString::to_string))
                .is_equal_to(Some("0".to_owned()));
            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(failure.actual.as_ref().map(ToString::to_string))
                .is_equal_to(Some("1".to_owned()));
            assert_that!(fact(&failure, "Waited"))
                .some()
                .ends_with("(1 observation)");
            assert_that!(note(&failure)).is_equal_to(Some(
                "A later observation did not complete before the timeout.".to_owned(),
            ));
        }

        #[tokio::test]
        async fn explain_eventually_ok_the_last_completed_error() {
            let failure = raised(|presentation| {
                let observe = || async {
                    tokio::task::yield_now().await;
                    Err::<u32, _>("unavailable")
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually_ok()
                        .with_patience(SECOND_ATTEMPT_AT_THE_TIMEOUT)
                        .matches(eq(1))
                        .await;
                }
            })
            .await;

            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(fact(&failure, "Error")).is_equal_to(Some("\"unavailable\"".to_owned()));
            assert_that!(note(&failure)).is_equal_to(Some(
                "A later observation did not complete before the timeout.".to_owned(),
            ));
        }

        #[tokio::test]
        async fn fail_consistently_after_the_observation_timeout() {
            let calls = Cell::new(0);
            let started = Instant::now();
            let failure = raised(|presentation| {
                let observe = move || {
                    calls.set(calls.get() + 1);
                    let first = calls.get() == 1;
                    async move {
                        if !first {
                            core::future::pending::<()>().await;
                        }
                        1
                    }
                };
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .consistently()
                        .with_patience(QUICK)
                        .for_at_least(Duration::from_secs(10))
                        .each_observation_within(Duration::from_millis(20))
                        .matches(eq(1))
                        .await;
                }
            })
            .await;

            assert_that!(started.elapsed()).is_less_than(Duration::from_secs(1));
            assert_that!(failure.relation.as_deref()).is_equal_to(Some("could not be observed"));
            assert_that!(note(&failure)).is_equal_to(Some(
                "The observation did not complete within 20ms.".to_owned(),
            ));
            assert_that!(fact(&failure, "Held"))
                .some()
                .ends_with("(1 observation), then not");
        }
    }

    mod patience_bounds {
        use super::*;
        use crate::matchers::eq;

        #[tokio::test]
        async fn a_pause_ends_at_the_timeout() {
            let calls = Cell::new(0);
            let started = Instant::now();
            raised(|presentation| {
                let observe = counter(&calls);
                async move {
                    assert_that!(observe)
                        .with_panic_presentation(presentation)
                        .eventually()
                        .with_patience(
                            QUICK
                                .with_timeout(Duration::from_millis(20))
                                .with_interval(Duration::from_secs(60)),
                        )
                        .matches(eq(0))
                        .await;
                }
            })
            .await;

            assert_that!(started.elapsed()).is_less_than(Duration::from_secs(1));
            assert_that!(calls.get()).is_equal_to(2);
        }

        #[tokio::test]
        async fn a_pause_ends_with_the_consistency_duration() {
            let started = Instant::now();
            let observe = || async { 1 };
            assert_that!(observe)
                .consistently()
                .with_patience(QUICK.with_interval(Duration::from_secs(60)))
                .matches(eq(1))
                .await;
            assert_that!(started.elapsed()).is_less_than(Duration::from_secs(1));
        }

        #[tokio::test]
        async fn maximal_durations_do_not_overflow() {
            let observe = || async { 1 };
            assert_that!(observe)
                .eventually()
                .with_patience(
                    Patience::DEFAULT
                        .with_timeout(Duration::MAX)
                        .with_interval(Duration::MAX)
                        .with_consistency_duration(Duration::MAX)
                        .with_observation_timeout(Duration::MAX),
                )
                .matches(eq(1))
                .await;

            let failure = raised(|presentation| async move {
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .consistently()
                    .with_patience(
                        QUICK
                            .with_timeout(Duration::MAX)
                            .with_interval(Duration::MAX)
                            .with_consistency_duration(Duration::MAX)
                            .with_observation_timeout(Duration::MAX),
                    )
                    .matches(eq(2))
                    .await;
            })
            .await;
            assert_that!(fact(&failure, "Held")).is_equal_to(Some("never".to_owned()));
        }
    }

    mod renderer_contract {
        use core::future::Ready;

        use super::*;
        use crate::{
            matchers::eq,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        #[derive(PartialEq)]
        struct Secret(u32);

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, fn() -> Ready<u32>, Panic, NoRenderer> => EventualAssertions
            );
        }

        #[tokio::test]
        async fn builders_work_without_renderer_support() {
            let observe = || async { 1 };
            let _eventually = assert_that!(observe)
                .with_renderer(NoRenderer)
                .eventually()
                .within(Duration::ZERO)
                .polling_every(Duration::ZERO)
                .with_patience(QUICK);
            let _consistently = assert_that!(observe)
                .with_renderer(NoRenderer)
                .consistently_ok()
                .for_at_least(Duration::ZERO)
                .polling_every(Duration::ZERO)
                .with_patience(QUICK);
        }

        #[tokio::test]
        async fn failures_render_observed_values_with_the_active_renderer() {
            let failure = raised(|presentation| async move {
                let observe = || async { Secret(1) };
                assert_that!(observe)
                    .with_panic_presentation(presentation)
                    .with_renderer(SentinelRenderer)
                    .eventually()
                    .with_patience(QUICK.with_timeout(Duration::ZERO))
                    .matches(eq(Secret(2)))
                    .await;
            })
            .await;

            assert_that!(failure.actual.as_ref().map(ToString::to_string))
                .is_equal_to(Some(SENTINEL.to_owned()));
        }
    }
}
