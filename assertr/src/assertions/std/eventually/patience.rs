use core::time::Duration;
use std::sync::{PoisonError, RwLock};

/// How long and how often [eventual assertions](super::EventualAssertions) observe their subject.
///
/// | Setting | Meaning | Default | Per chain |
/// |---|---|---|---|
/// | [`timeout`](Self::timeout) | How long `eventually` waits for the expectation before failing. | 1 s | [`Eventually::within`](super::Eventually::within) |
/// | [`interval`](Self::interval) | The pause between two observations. | 10 ms | `polling_every` on both builders |
/// | [`consistency_duration`](Self::consistency_duration) | How long `consistently` requires the expectation to hold. | 100 ms | [`Consistently::for_at_least`](super::Consistently::for_at_least) |
/// | [`observation_timeout`](Self::observation_timeout) | How long `consistently` waits for one observation to complete. | 1 s | [`Consistently::each_observation_within`](super::Consistently::each_observation_within) |
///
/// `eventually` bounds every observation by its timeout instead of the observation timeout, so an
/// observation still pending when `eventually` gives up never extends the assertion.
///
/// Every chain starts from the [global](Self::global) patience, which defaults to
/// [`Patience::DEFAULT`]. Set it once for a test suite with [`set_global`](Self::set_global).
/// [`with_patience`](super::Eventually::with_patience) replaces it for one chain, and the per-chain
/// settings above override single values in any order.
///
/// ```rust
/// use assertr::prelude::*;
/// use std::time::Duration;
///
/// // A browser test suite: values arrive through a remote driver.
/// Patience::DEFAULT
///     .with_timeout(Duration::from_secs(10))
///     .with_interval(Duration::from_millis(50))
///     .with_consistency_duration(Duration::from_millis(300))
///     .with_observation_timeout(Duration::from_secs(2))
///     .set_global();
/// # Patience::DEFAULT.set_global();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Patience {
    timeout: Duration,
    interval: Duration,
    consistency_duration: Duration,
    observation_timeout: Duration,
}

static GLOBAL: RwLock<Patience> = RwLock::new(Patience::DEFAULT);

impl Patience {
    /// The patience every chain starts from until [`set_global`](Self::set_global) replaces it:
    /// `eventually` waits up to 1 second, `consistently` requires 100 milliseconds and waits up to
    /// 1 second for each observation, and both observe every 10 milliseconds.
    pub const DEFAULT: Self = Self {
        timeout: Duration::from_secs(1),
        interval: Duration::from_millis(10),
        consistency_duration: Duration::from_millis(100),
        observation_timeout: Duration::from_secs(1),
    };

    /// Sets how long `eventually` waits for the expectation before failing.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the pause between two observations.
    #[must_use]
    pub const fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets how long `consistently` requires the expectation to hold.
    #[must_use]
    pub const fn with_consistency_duration(mut self, duration: Duration) -> Self {
        self.consistency_duration = duration;
        self
    }

    /// Sets how long `consistently` waits for one observation to complete.
    #[must_use]
    pub const fn with_observation_timeout(mut self, timeout: Duration) -> Self {
        self.observation_timeout = timeout;
        self
    }

    /// How long `eventually` waits for the expectation before failing.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The pause between two observations.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// How long `consistently` requires the expectation to hold.
    #[must_use]
    pub const fn consistency_duration(&self) -> Duration {
        self.consistency_duration
    }

    /// How long `consistently` waits for one observation to complete.
    #[must_use]
    pub const fn observation_timeout(&self) -> Duration {
        self.observation_timeout
    }

    /// The patience that chains start from: [`Patience::DEFAULT`] unless
    /// [`set_global`](Self::set_global) replaced it.
    #[must_use]
    pub fn global() -> Self {
        *GLOBAL.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Makes this the patience every chain starts from, for the whole process. Chains already
    /// waiting keep the patience they started with.
    pub fn set_global(self) {
        *GLOBAL.write().unwrap_or_else(PoisonError::into_inner) = self;
    }
}

impl Default for Patience {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The patience settings of one chain, resolved when its assertion starts.
///
/// Single settings override the base patience regardless of the order in which they were set.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Overrides {
    /// The patience replacing the global one, if any.
    base: Option<Patience>,
    timeout: Option<Duration>,
    interval: Option<Duration>,
    consistency_duration: Option<Duration>,
    observation_timeout: Option<Duration>,
}

impl Overrides {
    pub(super) const fn with_base(mut self, patience: Patience) -> Self {
        self.base = Some(patience);
        self
    }

    pub(super) const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub(super) const fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub(super) const fn with_consistency_duration(mut self, duration: Duration) -> Self {
        self.consistency_duration = Some(duration);
        self
    }

    pub(super) const fn with_observation_timeout(mut self, timeout: Duration) -> Self {
        self.observation_timeout = Some(timeout);
        self
    }

    /// The base patience, or the global one, with the single settings applied.
    pub(super) fn resolve(self) -> Patience {
        let base = self.base.unwrap_or_else(Patience::global);
        Patience {
            timeout: self.timeout.unwrap_or(base.timeout),
            interval: self.interval.unwrap_or(base.interval),
            consistency_duration: self
                .consistency_duration
                .unwrap_or(base.consistency_duration),
            observation_timeout: self.observation_timeout.unwrap_or(base.observation_timeout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    const BASE: Patience = Patience::DEFAULT
        .with_timeout(Duration::from_millis(200))
        .with_interval(Duration::from_millis(5))
        .with_consistency_duration(Duration::from_millis(40))
        .with_observation_timeout(Duration::from_millis(50));

    #[test]
    fn resolves_the_global_patience_with_single_overrides() {
        let previous = Patience::global();
        BASE.with_timeout(Duration::from_millis(30)).set_global();
        let resolved = Overrides::default()
            .with_interval(Duration::from_millis(1))
            .resolve();
        previous.set_global();

        assert_that!(resolved).is_equal_to(
            BASE.with_timeout(Duration::from_millis(30))
                .with_interval(Duration::from_millis(1)),
        );
    }

    #[test]
    fn single_overrides_take_precedence_over_the_base_in_any_order() {
        let timeout = Duration::from_millis(7);
        let before = Overrides::default()
            .with_timeout(timeout)
            .with_base(BASE)
            .resolve();
        let after = Overrides::default()
            .with_base(BASE)
            .with_timeout(timeout)
            .resolve();

        assert_that!(before).is_equal_to(BASE.with_timeout(timeout));
        assert_that!(after).is_equal_to(before);
    }
}
