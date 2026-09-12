use crate::failure::{Fact, FailureKind};
use crate::{AssertThat, Mode, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use tokio::sync::RwLock;

/// The immediate acquisition state of a Tokio read-write lock.
/// Acquired guards keep the observed value stable until the observation is consumed.
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
    fn explain<R: ValueRenderer<T>, Target>(
        self,
        actual: &RwLock<T>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match self {
            Self::Unlocked(guard) => failure
                .actual(render.struct_field(actual, "RwLock", "data", &*guard))
                .fact(Fact::labelled(LOCK_STATE, "unlocked")),
            Self::ReadLocked(guard) => failure
                .actual(render.struct_field(actual, "RwLock", "data", &*guard))
                .fact(Fact::labelled(LOCK_STATE, "read-locked")),
            Self::WriteLocked => failure
                .actual(render.unavailable_struct_field(actual, "RwLock", "data", "<locked>"))
                .fact(Fact::labelled(LOCK_STATE, "write-locked")),
        }
    }
}
/// Observes whether a Tokio read-write lock is not locked.
pub struct IsNotLocked;
impl<T, R> Expectation<RwLock<T>, R> for IsNotLocked {
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
        if matches!(observation, LockObservation::Unlocked(_)) {
            Ok(observation)
        } else {
            Err(observation)
        }
    }
}
impl<T, R> ExpectationDiagnostics<RwLock<T>, R> for IsNotLocked
where
    R: ValueRenderer<T>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a RwLock<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("is not locked"),
            Some((actual, observation)) => {
                observation.explain(actual, failure.relation("is unexpectedly locked"), context)
            }
        }
    }
}
/// Observes whether a Tokio read-write lock is read-locked.
pub struct IsReadLocked;
impl<T, R> Expectation<RwLock<T>, R> for IsReadLocked {
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
        if matches!(observation, LockObservation::ReadLocked(_)) {
            Ok(observation)
        } else {
            Err(observation)
        }
    }
}
impl<T, R> ExpectationDiagnostics<RwLock<T>, R> for IsReadLocked
where
    R: ValueRenderer<T>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a RwLock<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("is read-locked"),
            Some((actual, observation)) => {
                observation.explain(actual, failure.relation("is not read-locked"), context)
            }
        }
    }
}
/// Observes whether a Tokio read-write lock is write-locked.
pub struct IsWriteLocked;
impl<T, R> Expectation<RwLock<T>, R> for IsWriteLocked {
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
        if matches!(observation, LockObservation::WriteLocked) {
            Ok(observation)
        } else {
            Err(observation)
        }
    }
}
impl<T, R> ExpectationDiagnostics<RwLock<T>, R> for IsWriteLocked
where
    R: ValueRenderer<T>,
{
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a RwLock<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("is write-locked"),
            Some((actual, observation)) => {
                observation.explain(actual, failure.relation("is not write-locked"), context)
            }
        }
    }
}

/// Non-blocking assertions for Tokio's [`RwLock`] type.
///
/// State is inferred from immediate `try_read` and `try_write` attempts. Queued waiters and a
/// configured reader limit can affect those attempts, so these methods report acquisition state,
/// not a synchronized count of guards.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioRwLockAssertions<T, R> {
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
        self.apply_assertion(IsNotLocked)
    }

    #[track_caller]
    fn is_read_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsReadLocked)
    }

    #[track_caller]
    fn is_write_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsWriteLocked)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    mod observations {
        use super::super::{IsNotLocked, IsReadLocked, IsWriteLocked};
        use crate::{matchers::all_of, prelude::*};
        use tokio::sync::RwLock;

        #[test]
        fn composed_checks_release_rejected_and_successful_guards_between_siblings() {
            let lock = RwLock::new(7);
            let failures =
                assert_that!(lock).capture(|it| it.matches(all_of((IsReadLocked, IsWriteLocked))));
            assert_that!(failures[0].children).has_length(2);
            assert_that!(lock).matches(all_of((IsNotLocked, IsNotLocked)));
        }
    }

    use core::fmt;

    use crate::ValueRenderer;
    use tokio::sync::RwLock;

    struct WriteGuardCheckingRenderer<'a>(&'a RwLock<i32>);

    impl ValueRenderer<i32> for WriteGuardCheckingRenderer<'_> {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            assert_that!(self.0.try_read().is_err())
                .with_detail_message("the write guard must remain held while rendering")
                .is_true();
            write!(f, "guarded({value})")
        }
    }

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

    /// Synonym of `is_not_locked`. The fluent name and caller location are pinned here. The
    /// behavior is covered by that module.
    mod is_free {
        use crate::prelude::*;
        use tokio::sync::RwLock;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            RwLock::new(42).must().be_free();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let lock = RwLock::new(42);
            let _guard = lock.try_write().unwrap();
            assert_caller_location!(assert_that!(lock), is_free());
        }
    }

    mod is_read_locked {
        use super::WriteGuardCheckingRenderer;
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

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `rw_lock`

                Actual: RwLock {{
                    data: guarded(42),
                }}

                is not read-locked

                Details:
                  - Lock state: unlocked
                -------- assertr --------
            "});
                },
            ]);
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
        use super::WriteGuardCheckingRenderer;
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
