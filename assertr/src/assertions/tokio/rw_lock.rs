use crate::failure::FailureKind;
use crate::{AssertThat, Mode, ValueRenderer};
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

/// The label of the fact naming the observed lock state.
const LOCK_STATE: &str = "Lock state";

impl<T, M: Mode, R> TokioRwLockAssertions<T, R> for AssertThat<'_, RwLock<T>, M, R> {
    #[track_caller]
    fn is_not_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().try_write().is_err() {
            // Cannot be locked for writing, so it is already read- or write-locked.
            match self.actual().try_read() {
                // RwLock allows multiple readers. It cannot be read again, so the existing lock
                // is a write lock.
                Err(_) => {
                    self.failure(FailureKind::Other)
                        .actual(self.render().unavailable_struct_field(
                            self.actual(),
                            "RwLock",
                            "data",
                            "<locked>",
                        ))
                        .relation("is unexpectedly locked")
                        .fact(LOCK_STATE, "write-locked")
                        .raise();
                }
                Ok(value) => {
                    self.failure(FailureKind::Other)
                        .actual(self.render().struct_field(
                            self.actual(),
                            "RwLock",
                            "data",
                            &*value,
                        ))
                        .relation("is unexpectedly locked")
                        .fact(LOCK_STATE, "read-locked")
                        .raise();
                }
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
            // Can be locked for writing, so it holds no lock at all.
            let value = self
                .actual()
                .try_read()
                .expect("the lock-state check already succeeded");
            self.failure(FailureKind::Other)
                .actual(
                    self.render()
                        .struct_field(self.actual(), "RwLock", "data", &*value),
                )
                .relation("is not read-locked")
                .fact(LOCK_STATE, "unlocked")
                .raise();
        } else if self.actual().try_read().is_err() {
            // Cannot be locked for writing, and RwLock allows multiple readers, so a lock that
            // cannot be read again is a write lock.
            self.failure(FailureKind::Other)
                .actual(self.render().unavailable_struct_field(
                    self.actual(),
                    "RwLock",
                    "data",
                    "<locked>",
                ))
                .relation("is not read-locked")
                .fact(LOCK_STATE, "write-locked")
                .raise();
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
            // Can be locked for writing, so it holds no lock at all.
            let value = self
                .actual()
                .try_read()
                .expect("the lock-state check already succeeded");
            self.failure(FailureKind::Other)
                .actual(
                    self.render()
                        .struct_field(self.actual(), "RwLock", "data", &*value),
                )
                .relation("is not write-locked")
                .fact(LOCK_STATE, "unlocked")
                .raise();
        } else if let Ok(value) = self.actual().try_read() {
            // Cannot be locked for writing, and RwLock allows multiple readers, so a lock that
            // can be read again is a read lock.
            self.failure(FailureKind::Other)
                .actual(
                    self.render()
                        .struct_field(self.actual(), "RwLock", "data", &*value),
                )
                .relation("is not write-locked")
                .fact(LOCK_STATE, "read-locked")
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use tokio::sync::RwLock;

        struct Secret;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, RwLock<i32>, Panic, NoRenderer>
                    => TokioRwLockAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn failures_render_the_inner_value_with_the_active_renderer() {
            let failures = assert_that!(RwLock::new(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(TokioRwLockAssertions::is_read_locked);

            assert_that!(failures[0].actual.as_ref().map(rendered_text))
                .is_equal_to(Some(format!("RwLock {{\n    data: {SENTINEL},\n}}")));
        }
    }

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

                    Actual: RwLock {{
                        data: <locked>,
                    }}

                    is unexpectedly locked

                    Details:
                      - Lock state: write-locked
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

                    Actual: RwLock {{
                        data: 42,
                    }}

                    is unexpectedly locked

                    Details:
                      - Lock state: read-locked
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

                    Actual: RwLock {{
                        data: <locked>,
                    }}

                    is not read-locked

                    Details:
                      - Lock state: write-locked
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

                    Actual: RwLock {{
                        data: 42,
                    }}

                    is not read-locked

                    Details:
                      - Lock state: unlocked
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

                    Actual: RwLock {{
                        data: 42,
                    }}

                    is not write-locked

                    Details:
                      - Lock state: read-locked
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

                    Actual: RwLock {{
                        data: 42,
                    }}

                    is not write-locked

                    Details:
                      - Lock state: unlocked
                    -------- assertr --------
                "});
        }
    }
}
