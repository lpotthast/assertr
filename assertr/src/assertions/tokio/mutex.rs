use crate::{AssertThat, Mode, tracking::AssertionTracking};
use core::fmt::Debug;
use indoc::writedoc;
use std::fmt::Write;
use tokio::sync::Mutex;

// TODO: Add possibility to easily assert on the contained value (when the lock can be acquired).
/// Assertions for tokio's [Mutex] type.
pub trait TokioMutexAssertions<T: Debug> {
    fn is_locked(self) -> Self;
    fn be_locked(self) -> Self
    where
        Self: Sized,
    {
        self.is_locked()
    }

    fn is_not_locked(self) -> Self;
    fn not_be_locked(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }

    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }
    fn be_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_free()
    }
}

impl<T: Debug, M: Mode> TokioMutexAssertions<T> for AssertThat<'_, Mutex<T>, M> {
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
            mutex.must().be_locked();
            drop(guard);
        }

        #[test]
        fn panics_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that_panic_by(|| mutex.must().with_location(false).be_locked())
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
            mutex.must().not_be_locked();
        }

        #[tokio::test]
        async fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that_panic_by(|| mutex.must().with_location(false).not_be_locked())
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
            mutex.must().be_free();
        }
    }
}
