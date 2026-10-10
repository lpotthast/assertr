//! A runtime-agnostic pause between two observations.
//!
//! Eventual assertions run in whatever async runtime awaits them. One background thread wakes
//! every pending pause when its deadline passes, so no runtime's timer is required.

use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};
use std::{
    sync::{Condvar, Mutex, OnceLock, PoisonError},
    time::Instant,
};

/// Completes once `duration` has passed.
pub(super) fn sleep(duration: Duration) -> Sleep {
    Sleep {
        deadline: Instant::now() + duration,
    }
}

pub(super) struct Sleep {
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }
        timer().register(self.deadline, cx.waker().clone());
        Poll::Pending
    }
}

/// The deadlines of pending pauses and the background thread that wakes them.
struct Timer {
    pending: Mutex<Vec<(Instant, Waker)>>,
    changed: Condvar,
}

fn timer() -> &'static Timer {
    static TIMER: OnceLock<&'static Timer> = OnceLock::new();
    TIMER.get_or_init(|| {
        let timer: &'static Timer = Box::leak(Box::new(Timer {
            pending: Mutex::new(Vec::new()),
            changed: Condvar::new(),
        }));
        std::thread::Builder::new()
            .name("assertr-timer".into())
            .spawn(|| timer.run())
            .expect("the timer thread of eventual assertions can be spawned");
        timer
    })
}

impl Timer {
    fn register(&self, deadline: Instant, waker: Waker) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((deadline, waker));
        self.changed.notify_one();
    }

    fn run(&self) -> ! {
        loop {
            // Wake outside the lock: a waker may register the next pause right away.
            let due = {
                let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
                let now = Instant::now();
                let mut due = Vec::new();
                pending.retain(|(deadline, waker)| {
                    let is_due = *deadline <= now;
                    if is_due {
                        due.push(waker.clone());
                    }
                    !is_due
                });
                due
            };
            for waker in due {
                waker.wake();
            }
            let pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
            let next = pending.iter().map(|(deadline, _)| *deadline).min();
            let now = Instant::now();
            drop(match next {
                None => self
                    .changed
                    .wait(pending)
                    .unwrap_or_else(PoisonError::into_inner),
                Some(deadline) if deadline > now => {
                    self.changed
                        .wait_timeout(pending, deadline - now)
                        .unwrap_or_else(PoisonError::into_inner)
                        .0
                }
                Some(_) => pending,
            });
        }
    }
}
