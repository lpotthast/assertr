use core::time::Duration;
use std::sync::{PoisonError, RwLock};

/// How long and how often [eventual assertions](super::EventualAssertions) observe their subject.
///
/// - [`within`](Self::within): how long `eventually` waits for the expectation before failing, and
///   how long `consistently` waits for one observation to complete.
/// - [`polling_every`](Self::polling_every): the pause between two observations.
/// - [`consistently_for`](Self::consistently_for): how long `consistently` requires the expectation
///   to hold.
///
/// Every chain starts from the [global](Self::global) patience, which defaults to
/// [`Patience::DEFAULT`]. Set it once for a test suite with [`set_global`](Self::set_global), and
/// override single settings for one chain with `within`, `polling_every`, `for_at_least`, or
/// `with_patience` on the [`Eventually`](super::Eventually) and
/// [`Consistently`](super::Consistently) builders.
///
/// ```rust
/// use assertr::prelude::*;
/// use std::time::Duration;
///
/// // A browser test suite: values arrive through a remote driver.
/// Patience::DEFAULT
///     .within(Duration::from_secs(10))
///     .polling_every(Duration::from_millis(50))
///     .consistently_for(Duration::from_millis(300))
///     .set_global();
/// # Patience::DEFAULT.set_global();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Patience {
    within: Duration,
    polling_every: Duration,
    consistently_for: Duration,
}

static GLOBAL: RwLock<Patience> = RwLock::new(Patience::DEFAULT);

impl Patience {
    /// The patience every chain starts from until [`set_global`](Self::set_global) replaces it:
    /// `eventually` waits up to 1 second, `consistently` requires 100 milliseconds, and both
    /// observe every 10 milliseconds.
    pub const DEFAULT: Self = Self {
        within: Duration::from_secs(1),
        polling_every: Duration::from_millis(10),
        consistently_for: Duration::from_millis(100),
    };

    /// Sets how long `eventually` waits for the expectation before failing, and how long
    /// `consistently` waits for one observation to complete.
    #[must_use]
    pub const fn within(mut self, timeout: Duration) -> Self {
        self.within = timeout;
        self
    }

    /// Sets the pause between two observations.
    #[must_use]
    pub const fn polling_every(mut self, interval: Duration) -> Self {
        self.polling_every = interval;
        self
    }

    /// Sets how long `consistently` requires the expectation to hold.
    #[must_use]
    pub const fn consistently_for(mut self, duration: Duration) -> Self {
        self.consistently_for = duration;
        self
    }

    /// How long `eventually` waits for the expectation before failing, and how long
    /// `consistently` waits for one observation to complete.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.within
    }

    /// The pause between two observations.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.polling_every
    }

    /// How long `consistently` requires the expectation to hold.
    #[must_use]
    pub const fn consistency(&self) -> Duration {
        self.consistently_for
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

/// The overrides of one chain, applied to the global patience when its assertion starts.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Overrides {
    within: Option<Duration>,
    polling_every: Option<Duration>,
    consistently_for: Option<Duration>,
}

impl Overrides {
    pub(super) const fn within(mut self, timeout: Duration) -> Self {
        self.within = Some(timeout);
        self
    }

    pub(super) const fn polling_every(mut self, interval: Duration) -> Self {
        self.polling_every = Some(interval);
        self
    }

    pub(super) const fn consistently_for(mut self, duration: Duration) -> Self {
        self.consistently_for = Some(duration);
        self
    }

    pub(super) const fn all(patience: Patience) -> Self {
        Self {
            within: Some(patience.within),
            polling_every: Some(patience.polling_every),
            consistently_for: Some(patience.consistently_for),
        }
    }

    /// The global patience with these overrides applied.
    pub(super) fn resolve(self) -> Patience {
        let global = Patience::global();
        Patience {
            within: self.within.unwrap_or(global.within),
            polling_every: self.polling_every.unwrap_or(global.polling_every),
            consistently_for: self.consistently_for.unwrap_or(global.consistently_for),
        }
    }
}
