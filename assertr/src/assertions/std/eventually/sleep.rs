//! A runtime-agnostic pause between two observations, and a deadline for one observation.
//!
//! Eventual assertions run in whatever async runtime awaits them. One background thread wakes
//! every pending pause when its deadline passes, so no runtime's timer is required.

use core::{
    future::poll_fn,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};
use std::{
    sync::{Condvar, Mutex, Once, PoisonError},
    time::Instant,
};

/// Completes once `deadline` has passed, or never without a deadline.
pub(super) fn until(deadline: Option<Instant>) -> Sleep {
    Sleep {
        deadline,
        registration: None,
    }
}

/// Completes with the output of `future`, or with `None` once `deadline` has passed first. An
/// output that is ready at the deadline still counts.
pub(super) async fn before<Fut: Future>(
    future: Fut,
    deadline: Option<Instant>,
) -> Option<Fut::Output> {
    let mut future = pin!(future);
    let mut timeout = until(deadline);
    poll_fn(|cx| {
        if let Poll::Ready(output) = future.as_mut().poll(cx) {
            return Poll::Ready(Some(output));
        }
        Pin::new(&mut timeout).poll(cx).map(|()| None)
    })
    .await
}

/// A pause until a deadline. It registers with the timer while pending and deregisters when
/// dropped, so an abandoned pause neither accumulates nor keeps its task alive.
pub(super) struct Sleep {
    deadline: Option<Instant>,
    /// The id of this pause's timer entry, once registered.
    registration: Option<u64>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let Some(deadline) = self.deadline else {
            return Poll::Pending;
        };
        if Instant::now() >= deadline {
            return Poll::Ready(());
        }
        let registration = timer().register(self.registration, deadline, cx.waker());
        self.registration = Some(registration);
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let Some(registration) = self.registration {
            timer().deregister(registration);
        }
    }
}

/// The deadlines of pending pauses and the background thread that wakes them.
struct Timer {
    pending: Mutex<Pending>,
    changed: Condvar,
}

struct Pending {
    next_id: u64,
    entries: Vec<Entry>,
}

struct Entry {
    id: u64,
    deadline: Instant,
    waker: Waker,
}

/// The timer, its thread started on first use.
fn timer() -> &'static Timer {
    static TIMER: Timer = Timer {
        pending: Mutex::new(Pending {
            next_id: 0,
            entries: Vec::new(),
        }),
        changed: Condvar::new(),
    };
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        std::thread::Builder::new()
            .name("assertr-timer".into())
            .spawn(|| TIMER.run())
            .expect("the timer thread of eventual assertions can be spawned");
    });
    &TIMER
}

impl Timer {
    /// Wakes `waker` at `deadline`, updating the entry `registration` if it is still pending.
    /// Returns the entry's id.
    fn register(&self, registration: Option<u64>, deadline: Instant, waker: &Waker) -> u64 {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(entry) =
            registration.and_then(|id| pending.entries.iter_mut().find(|entry| entry.id == id))
        {
            if !entry.waker.will_wake(waker) {
                entry.waker.clone_from(waker);
            }
            return entry.id;
        }
        let id = pending.next_id;
        pending.next_id += 1;
        pending.entries.push(Entry {
            id,
            deadline,
            waker: waker.clone(),
        });
        self.changed.notify_one();
        id
    }

    fn deregister(&self, registration: u64) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entries
            .retain(|entry| entry.id != registration);
    }

    fn run(&self) -> ! {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            let now = Instant::now();
            let due: Vec<Waker> = pending
                .entries
                .extract_if(.., |entry| entry.deadline <= now)
                .map(|entry| entry.waker)
                .collect();
            if !due.is_empty() {
                // Wake outside the lock: a waker may register the next pause right away.
                drop(pending);
                due.into_iter().for_each(Waker::wake);
                pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
                continue;
            }
            // Every remaining deadline lies after `now`.
            pending = match pending.entries.iter().map(|entry| entry.deadline).min() {
                None => self
                    .changed
                    .wait(pending)
                    .unwrap_or_else(PoisonError::into_inner),
                Some(deadline) => {
                    self.changed
                        .wait_timeout(pending, deadline - now)
                        .unwrap_or_else(PoisonError::into_inner)
                        .0
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::prelude::*;

    /// How many timer entries have the id `registration`.
    fn entries_of(registration: Option<u64>) -> usize {
        timer()
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entries
            .iter()
            .filter(|entry| Some(entry.id) == registration)
            .count()
    }

    #[test]
    fn repeated_polls_keep_one_entry_and_dropping_removes_it() {
        let mut cx = Context::from_waker(Waker::noop());
        let mut pause = until(Instant::now().checked_add(Duration::from_secs(3600)));
        for _ in 0..100 {
            assert_that!(Pin::new(&mut pause).poll(&mut cx).is_pending()).is_true();
        }
        let registration = pause.registration;
        assert_that!(registration).is_some();
        assert_that!(entries_of(registration)).is_equal_to(1);

        drop(pause);
        assert_that!(entries_of(registration)).is_equal_to(0);
    }

    #[test]
    fn without_a_deadline_it_never_completes_and_never_registers() {
        let mut cx = Context::from_waker(Waker::noop());
        let mut pause = until(None);
        assert_that!(Pin::new(&mut pause).poll(&mut cx).is_pending()).is_true();
        assert_that!(pause.registration).is_none();
    }

    #[tokio::test]
    async fn before_abandons_a_future_that_never_completes() {
        let deadline = Instant::now().checked_add(Duration::from_millis(20));
        let output = before(core::future::pending::<u32>(), deadline).await;
        assert_that!(output).is_none();
    }

    #[tokio::test]
    async fn before_prefers_an_output_ready_at_the_deadline() {
        let output = before(async { 1 }, Some(Instant::now())).await;
        assert_that!(output).is_equal_to(Some(1));
    }
}
