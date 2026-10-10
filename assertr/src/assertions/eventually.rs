//! Builders and retry policies of eventual assertions on a value that changes over time.
//!
//! [`eventually`](crate::assertions::EventualAssertions::eventually) observes until an
//! expectation holds, and [`consistently`](crate::assertions::EventualAssertions::consistently)
//! requires it to keep holding. The subject is an observation: a closure returning a future of the
//! current value, such as `|| log.text()`. The assertion observes it repeatedly, with the
//! [`Patience`](crate::assertions::Patience) configured globally or for the chain, and applies any
//! [matcher](mod@crate::matchers) or assertion callback to every observed value.
//!
//! [`EventualAssertions`](crate::assertions::EventualAssertions) and
//! [`Patience`](crate::assertions::Patience) live in [`assertions`](crate::assertions) and the
//! prelude. This module holds the builders they return, [`Eventually`] and [`Consistently`], and
//! the [`GiveUp`] policies deciding which failed observations of
//! [`eventually_ok`](crate::assertions::EventualAssertions::eventually_ok) end the assertion.
//!
//! The futures of eventual assertions hold the observation, the expectation, the renderer, and an
//! `eventually_ok` give-up closure, but no chain records. They are `Send` whenever those are, so a
//! test may move between threads while it waits. They run in any async runtime.
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

pub use super::std::eventually::{
    AnyError, Consistently, Eventually, Fallible, GiveUp, KeepRetrying, Plain,
};
