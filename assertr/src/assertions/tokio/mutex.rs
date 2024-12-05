use crate::{tracking::AssertionTracking, AssertThat, Mode};
use core::fmt::Debug;
use indoc::writedoc;
use std::fmt::Write;
use tokio::sync::Mutex;

// TODO: Add possibility to easily assert on the contained value (when the lock can be acquired).
/// Assertions for tokio's [Mutex] type.
pub trait TokioMutexAssertions<T: Debug> {
    fn is_locked(self) -> Self;

    fn is_not_locked(self) -> Self;

    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }
}

impl<'t, T: Debug, M: Mode> TokioMutexAssertions<T> for AssertThat<'t, Mutex<T>, M> {
    #[track_caller]
    fn is_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if let Ok(guard) = actual.try_lock() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Mutex {{ data: {guard:#?} }}

                    to be locked, but it wasn't!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_locked(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if let Err(_err) = actual.try_lock() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Mutex {{ data: <locked> }}

                    to not be locked, but it was!
                "#}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod is_locked {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[tokio::test]
        async fn succeeds_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that_ref(&mutex).is_locked();
            drop(guard);
        }

        #[test]
        fn panics_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that_panic_by(|| assert_that(mutex).with_location(false).is_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected: Mutex {{ data: 42 }}

                    to be locked, but it wasn't!
                    -------- assertr --------
                "});
        }
    }

    mod is_not_locked {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that(mutex).is_not_locked();
        }

        #[tokio::test]
        async fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that_panic_by(|| {
                assert_that::<Mutex<i32>>(&mutex)
                    .with_location(false)
                    .is_not_locked()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expected: Mutex {{ data: <locked> }}

                    to not be locked, but it was!
                    -------- assertr --------
                "});
            drop(guard);
        }
    }

    mod is_free {
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that(mutex).is_free();
        }
    }
}
