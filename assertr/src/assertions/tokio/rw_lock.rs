use crate::{AssertThat, Mode, ValueRenderer};
use alloc::string::String;
use core::fmt::Write;
use indoc::writedoc;
use tokio::sync::RwLock;

/// Non-blocking assertions for Tokio's [`RwLock`] type.
///
/// State is inferred from immediate `try_read` and `try_write` attempts. Queued waiters and a
/// configured reader limit can affect those attempts, so these methods report acquisition state,
/// not a synchronized count of guards.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioRwLockAssertions<T, R> {
    /// Asserts that `try_write` can acquire the lock.
    fn is_not_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Alias of [`TokioRwLockAssertions::is_not_locked`].
    fn is_free(self) -> Self
    where
        Self: Sized,
        R: ValueRenderer<T>,
    {
        self.is_not_locked()
    }

    /// Asserts that `try_write` fails while `try_read` succeeds.
    fn is_read_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that both `try_write` and `try_read` fail.
    fn is_write_locked(self) -> Self
    where
        R: ValueRenderer<T>;
}

impl<T, M: Mode, R> TokioRwLockAssertions<T, R> for AssertThat<'_, RwLock<T>, M, R> {
    #[track_caller]
    fn is_not_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_write().is_err() {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_err() {
                // RwLock allows multiple readers, but we cannot read again, so existing lock must be write-lock!
                let actual = Self::render_unavailable_struct_field("RwLock", "data", "<locked>");
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:?}

                        was expected to not be read- or write-locked, but it is!

                        It is currently write-locked!
                    "}
                });
            } else {
                let value = self
                    .actual()
                    .try_read()
                    .expect("the lock-state check already succeeded");
                let actual = self.render_struct_field("RwLock", "data", &*value);
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:?}

                        was expected to not be read- or write-locked, but it is!

                        It is currently read-locked!
                    "}
                });
            }
        }
        self
    }

    #[track_caller]
    fn is_read_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_write().is_ok() {
            // Can be locked for writing, must have zero locks than!
            let value = self
                .actual()
                .try_read()
                .expect("the lock-state check already succeeded");
            let actual = self.render_struct_field("RwLock", "data", &*value);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:?}

                    was expected to be read-locked, but it is not!

                    It is not locked at all!
                "}
            });
        } else {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_err() {
                // RwLock allows multiple readers, but we cannot read again, so existing lock must be write-lock!
                let actual = Self::render_unavailable_struct_field("RwLock", "data", "<locked>");
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:?}

                        was expected to be read-locked, but it is not!

                        It is currently write-locked!
                    "}
                });
            }
        }
        self
    }

    #[track_caller]
    fn is_write_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_write().is_ok() {
            // Can be locked for writing, must have zero locks than!
            let value = self
                .actual()
                .try_read()
                .expect("the lock-state check already succeeded");
            let actual = self.render_struct_field("RwLock", "data", &*value);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:?}

                    was expected to be write-locked, but it is not!
                "}
            });
        } else {
            // Cannot be locked for writing, must already be read- or write-locked than!
            if self.actual().try_read().is_ok() {
                // RwLock allows multiple readers, and we can read again, so existing lock must be read-lock!
                let value = self
                    .actual()
                    .try_read()
                    .expect("the lock-state check already succeeded");
                let actual = self.render_struct_field("RwLock", "data", &*value);
                self.fail(|w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:?}

                        was expected to be write-locked, but it is not!

                        It is currently read-locked!
                    "}
                });
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            RwLock::new(42).must().not_be_locked();
        }

        #[test]
        fn succeeds_when_not_locked() {
            let rw_lock = RwLock::new(42);
            assert_that!(rw_lock).is_not_locked();
        }

        #[tokio::test]
        async fn panics_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| assert_that!(&rw_lock).with_location(false).is_not_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&rw_lock`

                    Actual: RwLock {{ data: <locked> }}

                    was expected to not be read- or write-locked, but it is!

                    It is currently write-locked!
                    -------- assertr --------
                "});

            drop(rw_lock_write_guard);
        }

        #[tokio::test]
        async fn panics_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;

            assert_that_panic_by(|| assert_that!(&rw_lock).with_location(false).is_not_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&rw_lock`

                    Actual: RwLock {{ data: 42 }}

                    was expected to not be read- or write-locked, but it is!

                    It is currently read-locked!
                    -------- assertr --------
                "});

            drop(rw_lock_read_guard);
        }
    }

    /// Synonym of `is_not_locked`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_free {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;
        #[cfg(feature = "fluent")]
        use tokio::sync::RwLock;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            RwLock::new(42).must().be_free();
        }
    }

    mod is_read_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;
            rw_lock.must().be_read_locked();
            drop(rw_lock_read_guard);
        }

        #[tokio::test]
        async fn succeeds_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;
            assert_that!(&rw_lock).is_read_locked();
            drop(rw_lock_read_guard);
        }

        #[tokio::test]
        async fn panics_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;

            assert_that_panic_by(|| assert_that!(&rw_lock).with_location(false).is_read_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&rw_lock`

                    Actual: RwLock {{ data: <locked> }}

                    was expected to be read-locked, but it is not!

                    It is currently write-locked!
                    -------- assertr --------
                "});

            drop(rw_lock_write_guard);
        }

        #[test]
        fn panics_when_not_locked_at_all() {
            let rw_lock = RwLock::new(42);

            assert_that_panic_by(|| assert_that!(rw_lock).with_location(false).is_read_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rw_lock`

                    Actual: RwLock {{ data: 42 }}

                    was expected to be read-locked, but it is not!

                    It is not locked at all!
                    -------- assertr --------
                "});
        }
    }

    mod is_write_locked {
        use crate::prelude::*;
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;
            rw_lock.must().be_write_locked();
            drop(rw_lock_write_guard);
        }

        #[tokio::test]
        async fn succeeds_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;
            assert_that!(&rw_lock).is_write_locked();
            drop(rw_lock_write_guard);
        }

        #[tokio::test]
        async fn panics_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;

            assert_that_panic_by(|| {
                assert_that!(&rw_lock)
                    .with_location(false)
                    .is_write_locked()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&rw_lock`

                    Actual: RwLock {{ data: 42 }}

                    was expected to be write-locked, but it is not!

                    It is currently read-locked!
                    -------- assertr --------
                "});

            drop(rw_lock_read_guard);
        }

        #[test]
        fn panics_when_not_write_locked() {
            let rw_lock = RwLock::new(42);

            assert_that_panic_by(|| assert_that!(rw_lock).with_location(false).is_write_locked())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rw_lock`

                    Actual: RwLock {{ data: 42 }}

                    was expected to be write-locked, but it is not!
                    -------- assertr --------
                "});
        }
    }
}
