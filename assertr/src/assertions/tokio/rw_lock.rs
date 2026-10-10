use tokio::sync::RwLock;

use crate::{
    AssertThat, Mode,
    assertions::std::mutex::{lock_data, locked_data},
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{DebugRenderer, ValueRenderer},
};

/// The immediate acquisition state of a Tokio read-write lock, observed by [`IsNotLocked`],
/// [`IsReadLocked`], and [`IsWriteLocked`].
///
/// These expectations return the observation both as their success and as their rejection.
/// Evaluation first tries to acquire a write guard, then a read guard. An acquired guard keeps the
/// observed value stable until the observation is dropped, so explanation renders the value that
/// matched the observed state. Match on the variant to read the state, and drop the observation
/// promptly, because its guard holds the lock.
#[derive(Debug)]
pub enum LockObservation<'a, T> {
    /// A write guard could be acquired.
    Unlocked(tokio::sync::RwLockWriteGuard<'a, T>),
    /// Only a read guard could be acquired.
    ReadLocked(tokio::sync::RwLockReadGuard<'a, T>),
    /// Neither guard could be acquired.
    WriteLocked,
}
impl<T> LockObservation<'_, T> {
    fn observe(actual: &RwLock<T>) -> LockObservation<'_, T> {
        match actual.try_write() {
            Ok(guard) => LockObservation::Unlocked(guard),
            Err(_) => match actual.try_read() {
                Ok(guard) => LockObservation::ReadLocked(guard),
                Err(_) => LockObservation::WriteLocked,
            },
        }
    }
    fn explain<R: ValueRenderer<T>>(
        self,
        actual: &RwLock<T>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        // Each guard is released at the end of its arm, after its data has been rendered.
        let (data, state) = match self {
            Self::Unlocked(guard) => (lock_data(render, actual, "RwLock", &*guard), "unlocked"),
            Self::ReadLocked(guard) => {
                (lock_data(render, actual, "RwLock", &*guard), "read-locked")
            }
            Self::WriteLocked => (locked_data(render, actual, "RwLock"), "write-locked"),
        };
        failure.actual(data).fact(Fact::labelled(LOCK_STATE, state))
    }
}
/// Generates a Tokio read-write lock expectation from its accepted state and relations.
macro_rules! lock_state_expectation {
    ($(#[$meta:meta])* $name:ident, $state:pat, $met:literal, $unmet:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl<T, R> Expectation<RwLock<T>, R> for $name
        where
            R: ValueRenderer<T>,
        {
            type Success<'a>
                = LockObservation<'a, T>
            where
                Self: 'a,
                RwLock<T>: 'a;
            type Rejection<'a>
                = LockObservation<'a, T>
            where
                Self: 'a,
                RwLock<T>: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a RwLock<T>,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                let observation = LockObservation::observe(actual);
                if matches!(observation, $state) {
                    Ok(observation)
                } else {
                    Err(observation)
                }
            }

            const KIND: FailureKind = FailureKind::Other;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a RwLock<T>, Self::Rejection<'a>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                match rejected {
                    None => failure.relation($met),
                    Some((actual, observation)) => {
                        observation.explain(actual, failure.relation($unmet), context)
                    }
                }
            }
        }

    };
}

lock_state_expectation!(
    /// Observes whether a Tokio read-write lock is not locked.
    IsNotLocked,
    LockObservation::Unlocked(_),
    "is not locked",
    "is unexpectedly locked"
);

lock_state_expectation!(
    /// Observes whether a Tokio read-write lock is read-locked.
    IsReadLocked,
    LockObservation::ReadLocked(_),
    "is read-locked",
    "is not read-locked"
);

lock_state_expectation!(
    /// Observes whether a Tokio read-write lock is write-locked.
    IsWriteLocked,
    LockObservation::WriteLocked,
    "is write-locked",
    "is not write-locked"
);

/// Non-blocking assertions for Tokio's [`RwLock`] type.
///
/// State is inferred from immediate `try_read` and `try_write` attempts. Queued waiters and a
/// configured reader limit can affect those attempts, so these methods report acquisition state,
/// not a synchronized count of guards.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioRwLockAssertions<T, R = DebugRenderer> {
    /// Asserts that `try_write` can acquire the lock.
    fn is_not_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Alias of [`TokioRwLockAssertions::is_not_locked`].
    #[track_caller]
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
        self.matches(IsNotLocked)
    }

    #[track_caller]
    fn is_read_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsReadLocked)
    }

    #[track_caller]
    fn is_write_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsWriteLocked)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use tokio::sync::RwLock;

    use crate::{prelude::*, renderer::ValueRenderer};

    /// Renders values only while their lock's write guard is held.
    struct WriteGuardCheckingRenderer<'a>(&'a RwLock<i32>);

    impl ValueRenderer<i32> for WriteGuardCheckingRenderer<'_> {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            assert_that!(self.0.try_read())
                .with_detail_message("the write guard must remain held while rendering")
                .is_err();
            write!(f, "guarded({value})")
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use tokio::sync::RwLock;

        use crate::prelude::*;

        #[tokio::test]
        async fn are_as_expected() {
            RwLock::new(42).must().not_be_locked();
            RwLock::new(42).must().be_free();
            {
                let rw_lock = RwLock::new(42);
                let rw_lock_read_guard = rw_lock.read().await;
                rw_lock.must().be_read_locked();
                drop(rw_lock_read_guard);
            }
            {
                let rw_lock = RwLock::new(42);
                let rw_lock_write_guard = rw_lock.write().await;
                rw_lock.must().be_write_locked();
                drop(rw_lock_write_guard);
            }
        }
    }

    mod observations {
        use tokio::sync::RwLock;

        use super::super::{IsNotLocked, IsReadLocked, IsWriteLocked};
        use crate::{matchers::all_of, prelude::*};

        #[test]
        fn composed_checks_release_rejected_and_successful_guards_between_siblings() {
            let lock = RwLock::new(7);
            let failures = assert_that!(lock)
                .capture(|it| it.matches(all_of(matchers![IsReadLocked, IsWriteLocked])));
            assert_that!(failures[0].children).has_length(2);
            assert_that!(lock).matches(all_of(matchers![IsNotLocked, IsNotLocked]));
        }
    }

    mod renderer_contract {
        use tokio::sync::RwLock;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

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

            assert_that!(
                failures[0]
                    .actual
                    .as_ref()
                    .map(|value| format!("{value:#}"))
            )
            .is_equal_to(Some(format!("RwLock {{\n    data: {SENTINEL},\n}}")));
        }
    }

    mod is_not_locked {
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let lock = RwLock::new(42);
            let _guard = lock.try_write().unwrap();
            assert_caller_location!(assert_that!(lock), is_not_locked());
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

            assert_that!(|| assert_that!(&rw_lock).with_location(false).is_not_locked())
                .panics()
                .has_message()
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

            assert_that!(|| assert_that!(&rw_lock).with_location(false).is_not_locked())
                .panics()
                .has_message()
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

    /// Synonym of `is_not_locked`. The fluent name and caller location are pinned here. The
    /// behavior is covered by that module.
    mod is_free {
        use tokio::sync::RwLock;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let lock = RwLock::new(42);
            let _guard = lock.try_write().unwrap();
            assert_caller_location!(assert_that!(lock), is_free());
        }
    }

    mod is_read_locked {
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        use super::WriteGuardCheckingRenderer;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let rw_lock = RwLock::new(42);
            assert_caller_location!(assert_that!(rw_lock), is_read_locked());
        }

        #[test]
        fn retains_the_write_guard_until_the_failure_is_rendered() {
            let rw_lock = RwLock::new(42);
            let failures = assert_that!(rw_lock)
                .with_renderer(WriteGuardCheckingRenderer(&rw_lock))
                .with_location(false)
                .capture(|it| it.is_read_locked().is_not_locked());

            // The renderer checks the guard. `panics_when_not_locked_at_all` pins the report.
            assert_that!(failures).has_length(1);
            assert_that!(
                failures[0]
                    .actual
                    .as_ref()
                    .map(|value| format!("{value:#}"))
            )
            .is_equal_to(Some("RwLock {\n    data: guarded(42),\n}".to_owned()));
        }

        #[tokio::test]
        async fn succeeds_when_read_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_read_guard = rw_lock.read().await;
            assert_that!(&rw_lock).is_read_locked();
            drop(rw_lock_read_guard);
        }

        #[test]
        fn panics_when_not_locked_at_all() {
            let rw_lock = RwLock::new(42);

            assert_that!(|| assert_that!(rw_lock).with_location(false).is_read_locked())
                .panics()
                .has_message()
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
        use indoc::formatdoc;
        use tokio::sync::RwLock;

        use super::WriteGuardCheckingRenderer;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let rw_lock = RwLock::new(42);
            assert_caller_location!(assert_that!(rw_lock), is_write_locked());
        }

        #[test]
        fn retains_the_write_guard_until_the_failure_is_rendered() {
            let rw_lock = RwLock::new(42);
            let failures = assert_that!(rw_lock)
                .with_renderer(WriteGuardCheckingRenderer(&rw_lock))
                .with_location(false)
                .capture(|it| it.is_write_locked().is_not_locked());

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `rw_lock`

                Actual: RwLock {{
                    data: guarded(42),
                }}

                is not write-locked

                Details:
                  - Lock state: unlocked
                -------- assertr --------
            "});
                },
            ]);
        }

        #[tokio::test]
        async fn succeeds_when_write_locked() {
            let rw_lock = RwLock::new(42);
            let rw_lock_write_guard = rw_lock.write().await;
            assert_that!(&rw_lock).is_write_locked();
            drop(rw_lock_write_guard);
        }
    }
}
