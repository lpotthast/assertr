use crate::{tracking::AssertionTracking, AssertThat, Mode};
use core::fmt::Debug;
use tokio::sync::RwLock;

/// Assertions for tokio's [RwLock] type.
pub trait TokioRwLockAssertions<T: Debug> {
    fn is_not_locked(self) -> Self;

    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }

    fn is_read_locked(self) -> Self;

    fn is_write_locked(self) -> Self;
}

impl<'t, T: Debug, M: Mode> TokioRwLockAssertions<T> for AssertThat<'t, RwLock<T>, M> {
    #[track_caller]
    fn is_not_locked(self) -> Self
    where
        T: Debug,
    {
        self.track_assertion();
        if self.actual().try_write().is_err() {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_err() {
                // RwLock allows multiple readers, but we cannot read again, so existing lock must be write-lock!
                self.fail(format_args!(
                    "Actual: {actual:?}\n\nwas expected to not be read- or write-locked, but it is!\n\nIt is currently write-locked!\n",
                    actual = self.actual(),
                ));
            } else {
                self.fail(format_args!(
                    "Actual: {actual:?}\n\nwas expected to not be read- or write-locked, but it is!\n\nIt is currently read-locked!\n",
                    actual = self.actual(),
                ));
            }
        }
        self
    }

    #[track_caller]
    fn is_read_locked(self) -> Self
    where
        T: Debug,
    {
        self.track_assertion();
        if self.actual().try_write().is_ok() {
            // Can be locked for writing, must have zero locks than!
            self.fail(format_args!(
                "Actual: {actual:?}\n\nwas expected to be read-locked, but it is not!\n\nIt is not locked at all!\n",
                actual = self.actual(),
            ));
        } else {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_err() {
                // RwLock allows multiple readers, but we cannot read again, so existing lock must be write-lock!
                self.fail(format_args!(
                    "Actual: {actual:?}\n\nwas expected to be read-locked, but it is not!\n\nIt is currently write-locked!\n",
                    actual = self.actual(),
                ));
            }
        }
        self
    }

    #[track_caller]
    fn is_write_locked(self) -> Self
    where
        T: Debug,
    {
        self.track_assertion();
        if self.actual().try_write().is_ok() {
            // Can be locked for writing, must have zero locks than!
            self.fail(format_args!(
                "Actual: {actual:?}\n\nwas expected to be write-locked, but it is not!\n",
                actual = self.actual(),
            ));
        } else {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_ok() {
                // RwLock allows multiple readers, and we can read again, so existing lock must be read-lock!
                self.fail(format_args!(
                    "Actual: {actual:?}\n\nwas expected to be write-locked, but it is not!\n\nIt is currently read-locked!\n",
                    actual = self.actual(),
                ));
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod is_not_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[test]
        fn succeeds_when_not_locked() {
            let rw_lock = RwLock::new(42);
            assert_that(rw_lock).is_not_locked();
        }

        #[tokio::test]
        async fn panics_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| {
                assert_that::<RwLock<u32>>(&rw_lock)
                    .with_location(false)
                    .is_not_locked()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: <locked> }}

                    was expected to not be read- or write-locked, but it is!

                    It is currently write-locked!
                    -------- assertr --------
                "#});

            drop(rw_lock_write_guard);
        }

        #[tokio::test]
        async fn panics_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;

            assert_that_panic_by(|| {
                assert_that::<RwLock<u32>>(&rw_lock)
                    .with_location(false)
                    .is_not_locked()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: 42 }}

                    was expected to not be read- or write-locked, but it is!

                    It is currently read-locked!
                    -------- assertr --------
                "#});

            drop(rw_lock_read_guard);
        }
    }

    mod is_read_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[tokio::test]
        async fn succeeds_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;
            assert_that(&rw_lock).is_read_locked();
            drop(rw_lock_read_guard);
        }

        #[tokio::test]
        async fn panics_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| assert_that(&rw_lock).with_location(false).is_read_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: <locked> }}

                    was expected to be read-locked, but it is not!

                    It is currently write-locked!
                    -------- assertr --------
                "#});

            drop(rw_lock_write_guard);
        }

        #[test]
        fn panics_when_not_locked_at_all() {
            let rw_lock = RwLock::new(42);

            assert_that_panic_by(|| assert_that(rw_lock).with_location(false).is_read_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: 42 }}

                    was expected to be read-locked, but it is not!

                    It is not locked at all!
                    -------- assertr --------
                "#});
        }
    }

    mod is_write_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[tokio::test]
        async fn succeeds_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;
            assert_that(&rw_lock).is_write_locked();
            drop(rw_lock_write_guard);
        }

        #[tokio::test]
        async fn panics_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;

            assert_that_panic_by(|| assert_that(&rw_lock).with_location(false).is_write_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: 42 }}

                    was expected to be write-locked, but it is not!

                    It is currently read-locked!
                    -------- assertr --------
                "#});

            drop(rw_lock_read_guard);
        }

        #[test]
        fn panics_when_not_write_locked() {
            let rw_lock = RwLock::new(42);

            assert_that_panic_by(|| assert_that(rw_lock).with_location(false).is_write_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: RwLock {{ data: 42 }}

                    was expected to be write-locked, but it is not!
                    -------- assertr --------
                "#});
        }
    }
}
