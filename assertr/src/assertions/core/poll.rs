use core::task::Poll;

use crate::{
    AssertThat,
    assertions::support::{explain_variant, project_checked},
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
    mode::{Mode, Panic},
    renderer::{DebugRenderer, ValueRenderer},
};

fn ready<T>(poll: Poll<T>) -> Option<T> {
    match poll {
        Poll::Ready(value) => Some(value),
        Poll::Pending => None,
    }
}

fn ready_ref<T>(poll: &Poll<T>) -> Option<&T> {
    match poll {
        Poll::Ready(value) => Some(value),
        Poll::Pending => None,
    }
}

/// Checks for `Ready` and returns the borrowed value on success.
/// Checks, extraction, and ordinary callbacks execute this same definition.
#[derive(Debug, Clone, Copy)]
pub struct IsReady;

impl<T, R> Expectation<Poll<T>, R> for IsReady {
    type Success<'a>
        = &'a T
    where
        Self: 'a,
        Poll<T>: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Poll<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Poll<T>,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a T, ()> {
        ready_ref(actual).ok_or(())
    }

    const KIND: FailureKind = FailureKind::Variant;
    fn explain(
        &self,
        rejected: Option<(&Poll<T>, ())>,
        failure: FailureBuilder,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_variant(failure, rejected.map(|_| "Pending"), "Poll::Ready")
    }
}

/// Checks for `Pending`, retaining the unexpected ready value on rejection.
#[derive(Debug, Clone, Copy)]
pub struct IsPending;

impl<T, R: ValueRenderer<T>> Expectation<Poll<T>, R> for IsPending {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Poll<T>: 'a;
    type Rejection<'a>
        = &'a T
    where
        Self: 'a,
        Poll<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Poll<T>,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), &'a T> {
        match actual {
            Poll::Pending => Ok(()),
            Poll::Ready(value) => Err(value),
        }
    }

    const KIND: FailureKind = FailureKind::Variant;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Poll<T>, &'a T)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        explain_variant(
            failure,
            rejected.map(|(actual, value)| render.variant(actual, "Ready", value)),
            "Poll::Pending",
        )
    }
}

/// Non-extracting assertions for `Poll` subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PollAssertions<T, M: Mode, R = DebugRenderer> {
    /// Asserts that the subject is `Ready`.
    ///
    /// Non-extracting: the subject stays the full `Poll`, so further assertions can be chained in
    /// any mode. Use [`PollExtractAssertions::get_ready`] to extract the contained value in panic
    /// mode, or [`PollAssertions::is_ready_satisfying`] to assert on it in any mode.
    fn is_ready(self) -> Self;

    /// Asserts that the subject is `Pending`.
    fn is_pending(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject is `Ready`, then runs `assertions` on its value.
    ///
    /// The closure receives an `AssertThat<T>` borrowing the contained value.
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>);
}

impl<T, M: Mode, R> PollAssertions<T, M, R> for AssertThat<'_, Poll<T>, M, R> {
    #[track_caller]
    fn is_ready(self) -> Self {
        self.matches(IsReady)
    }

    #[track_caller]
    fn is_pending(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsPending)
    }

    #[track_caller]
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
    {
        self.satisfy_success(self.test_assertion(&IsReady), assertions);
        self
    }
}

/// Panic-mode extraction from `Poll` subjects.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PollExtractAssertions<'t, T, R = DebugRenderer> {
    /// Asserts that the subject is `Ready`, then returns an assertion over its value.
    ///
    /// A borrowed subject yields a borrowed value. An owned subject yields an owned value.
    ///
    /// This is available only in `Panic` mode because `Pending` cannot produce a `T`. Use
    /// [`PollAssertions::is_ready_satisfying`] for capture mode, or the non-extracting
    /// [`PollAssertions::is_ready`] when the contained value is irrelevant.
    fn get_ready(self) -> AssertThat<'t, T, Panic, R>;
}

impl<'t, T, R> PollExtractAssertions<'t, T, R> for AssertThat<'t, Poll<T>, Panic, R> {
    #[track_caller]
    fn get_ready(self) -> AssertThat<'t, T, Panic, R> {
        self.matches(IsReady)
            .map(|actual| project_checked(actual, ready, ready_ref))
    }
}

#[cfg(test)]
mod tests {
    use core::task::Poll;

    use indoc::formatdoc;

    use crate::{failure::FailureKind, prelude::*, test_support::rejected_kind};

    #[derive(Debug, PartialEq)]
    struct Foo {
        val: u32,
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        fn are_as_expected() {
            Poll::Ready(42).must().be_ready();
            Poll::<i32>::Pending.must().be_pending();
            Poll::Ready(42).must().be_ready_satisfying(|ready| {
                ready.is_equal_to(42);
            });
        }
    }

    mod renderer_contract {
        use super::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};

        struct Secret;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Poll<()>, Panic, NoRenderer>
                    => PollAssertions<(), Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Poll<()>, Panic, NoRenderer>
                    => PollExtractAssertions<'static, (), NoRenderer>
            );
        }

        #[test]
        fn ready_variant_is_rendered_from_its_leaf_value() {
            let failures = assert_that!(Poll::Ready(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(PollAssertions::is_pending);

            assert_that!(failures[0].to_string())
                .contains("Ready(")
                .contains(SENTINEL);
        }
    }

    mod is_ready {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Poll::<i32>::Pending), is_ready());
        }

        #[test]
        fn succeeds_when_ready_and_retains_the_subject() {
            assert_that!(Poll::Ready(Foo { val: 42 }))
                .is_ready()
                .is_equal_to(Poll::Ready(Foo { val: 42 }));
        }

        #[test]
        fn panics_when_not_ready() {
            assert_that!(|| {
                assert_that!(Poll::<Foo>::Pending)
                    .with_location(false)
                    .is_ready();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::<Foo>::Pending`

                Actual: Pending

                is not the expected variant

                Expected: Poll::Ready
                -------- assertr --------
            "});
        }

        #[test]
        fn continues_in_capture_mode() {
            let failures =
                assert_that!(Poll::<i32>::Pending).capture(|it| it.is_ready().is_pending());
            assert_that!(failures).has_length(1);
        }
    }

    mod get_ready {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Poll::<i32>::Pending), get_ready());
        }

        #[test]
        fn extracts_borrowed_and_owned_values() {
            let poll = Poll::Ready(Foo { val: 42 });
            assert_that!(poll).get_ready().is_equal_to(Foo { val: 42 });
            assert_that_owned!(poll)
                .get_ready()
                .is_equal_to(Foo { val: 42 });
        }

        #[test]
        fn rejects_pending_like_is_ready() {
            rejected_kind(FailureKind::Variant, || {
                let _ = assert_that!(Poll::<i32>::Pending)
                    .with_panic_presentation(|failure| format!("{:?}", failure.kind))
                    .get_ready();
            });
        }
    }

    mod is_ready_satisfying {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(Poll::<i32>::Pending),
                is_ready_satisfying(|_| {})
            );
        }

        #[test]
        fn retains_fn_once_callbacks_and_tracks_the_variant_once() {
            let owned = String::from("consumed by callback");
            let assertion = assert_that!(Poll::Ready(3)).is_ready_satisfying(
                |it: AssertThat<'_, i32, Panic>| {
                    drop(owned);
                    it.is_equal_to(3);
                },
            );
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(2);
        }

        #[test]
        fn does_not_run_the_callback_when_pending() {
            let failures = assert_that!(Poll::<i32>::Pending)
                .capture(|it| it.is_ready_satisfying(|_| panic!("assertions should not run")));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].kind).is_equal_to(FailureKind::Variant);
        }
    }

    mod is_pending {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(Poll::Ready(1)), is_pending());
        }

        #[test]
        fn succeeds_when_pending() {
            assert_that!(Poll::<Foo>::Pending).is_pending();
        }

        #[test]
        fn panics_when_ready() {
            assert_that!(|| {
                assert_that!(Poll::Ready(Foo { val: 42 }))
                    .with_location(false)
                    .is_pending();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::Ready(Foo {{ val: 42 }})`

                Actual: Ready(
                    Foo {{
                        val: 42,
                    }},
                )

                is not the expected variant

                Expected: Poll::Pending
                -------- assertr --------
            "});
        }
    }
}
