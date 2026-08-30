use crate::util::failure::join_failures;
use crate::{AssertThat, Mode, ValueRenderer};
use indoc::writedoc;
use std::fmt::Write;
use tokio::sync::Mutex;

/// Non-blocking assertions for Tokio's [`Mutex`] type.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioMutexAssertions<T, R> {
    /// Asserts that `try_lock` cannot acquire the mutex.
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that `try_lock` can acquire the mutex.
    fn is_not_locked(self) -> Self;

    /// Alias of [`TokioMutexAssertions::is_not_locked`].
    fn is_free(self) -> Self
    where
        Self: Sized,
    {
        self.is_not_locked()
    }

    /// Tries to acquire the mutex and runs assertions against its contained value.
    ///
    /// Fails the assertion if the mutex is currently locked. The closure receives a capture-mode
    /// assertion, and any failures it raises are attached to one mutex-level failure.
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, crate::mode::Capture, R>),
        R: ValueRenderer<T> + Clone;
}

impl<T, M: Mode, R> TokioMutexAssertions<T, R> for AssertThat<'_, Mutex<T>, M, R> {
    #[track_caller]
    fn is_locked(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        if let Ok(guard) = actual.try_lock() {
            let data = self.render_value(&*guard);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Mutex {{ data: {data:#?} }}

                    to be locked, but it wasn't!
                "}
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
                writedoc! {w, r"
                    Expected: Mutex {{ data: <locked> }}

                    to not be locked, but it was!
                "}
            });
        }
        self
    }

    #[track_caller]
    fn has_value_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, T, crate::mode::Capture, R>),
        R: ValueRenderer<T> + Clone,
    {
        self.track_assertion();
        {
            let actual = self.actual();
            match actual.try_lock() {
                Ok(guard) => {
                    let failures = self.collect_element_failures(&*guard, assertions);
                    if !failures.is_empty() {
                        let data = self.render_value(&*guard);
                        let details = [format!(
                            "Contained data failures:\n{}",
                            join_failures(&failures)
                        )];
                        self.fail_with_details(details, |w: &mut String| {
                            writedoc! {w, r"
                                Actual: Mutex {{ data: {data:#?} }}

                                contains data that does not satisfy the assertions.
                            "}
                        });
                    }
                }
                Err(_error) => {
                    self.fail(|w: &mut String| {
                        writedoc! {w, r"
                            Actual: Mutex {{ data: <locked> }}

                            could not be inspected because it is locked.
                        "}
                    });
                }
            }
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
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            mutex.must().be_locked();
            drop(guard);
        }

        #[tokio::test]
        async fn succeeds_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that!(&mutex).is_locked();
            drop(guard);
        }

        #[test]
        fn panics_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that_panic_by(|| assert_that!(mutex).with_location(false).is_locked())
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().not_be_locked();
        }

        #[test]
        fn succeeds_when_not_locked() {
            let mutex = Mutex::new(42);
            assert_that!(mutex).is_not_locked();
        }

        #[tokio::test]
        async fn panics_when_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;
            assert_that_panic_by(|| assert_that!(&mutex).with_location(false).is_not_locked())
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

    /// Synonym of `is_not_locked`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_free {
        #[cfg(feature = "fluent")]
        use tokio::sync::Mutex;

        #[cfg(feature = "fluent")]
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().be_free();
        }
    }

    mod has_value_satisfying {
        use indoc::formatdoc;
        use tokio::sync::Mutex;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Mutex::new(42).must().have_value_satisfying(|value| {
                value.is_equal_to(42);
            });
        }

        #[test]
        fn succeeds_when_the_mutex_is_available_and_the_value_satisfies_the_assertions() {
            let mutex = Mutex::new(String::from("value"));

            assert_that!(mutex).has_value_satisfying(|value| {
                value.contains("alu");
            });
        }

        #[test]
        fn accepts_an_fn_once_assertion_callback() {
            let mutex = Mutex::new(42);
            let captured = String::from("consumed");

            assert_that!(mutex).has_value_satisfying(move |value| {
                drop(captured);
                value.is_equal_to(42);
            });
        }

        #[test]
        fn panics_with_the_contained_failures_when_the_value_does_not_satisfy_the_assertions() {
            let mutex = Mutex::new(42);
            let indented_blank_line = "    ";

            assert_that_panic_by(|| {
                assert_that!(mutex)
                    .with_location(false)
                    .has_value_satisfying(|value| {
                        value.is_equal_to(43);
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: Mutex {{ data: 42 }}

                    contains data that does not satisfy the assertions.

                    Details: [
                        Contained data failures:
                        Expected: 43
                    {indented_blank_line}
                          Actual: 42,
                    ]
                    -------- assertr --------
                "});
        }

        #[tokio::test]
        async fn panics_when_the_mutex_is_locked() {
            let mutex = Mutex::new(42);
            let guard = mutex.lock().await;

            assert_that_panic_by(|| {
                assert_that!(mutex)
                    .with_location(false)
                    .has_value_satisfying(|value| {
                        value.is_equal_to(42);
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Actual: Mutex {{ data: <locked> }}

                    could not be inspected because it is locked.
                    -------- assertr --------
                "});

            drop(guard);
        }

        #[test]
        fn works_in_capture_mode() {
            let mutex = Mutex::new(42);

            let failures = assert_that!(mutex).with_location(false).capture(|it| {
                it.has_value_satisfying(|value| {
                    value.is_equal_to(43);
                })
            });

            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .satisfies(
                            |failure| &failure.description,
                            |description| {
                                description.is_equal_to(formatdoc! {r"
                                    Actual: Mutex {{ data: 42 }}

                                    contains data that does not satisfy the assertions.
                                "});
                            },
                        )
                        .satisfies(
                            |failure| &failure.details,
                            |details| {
                                details.contains_exactly([formatdoc! {r"
                                        Contained data failures:
                                        Expected: 43

                                          Actual: 42
                                    "}
                                .trim_end()]);
                            },
                        );
                },
            ]);
        }
    }
}
