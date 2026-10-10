use alloc::boxed::Box;
use core::{
    any::Any,
    panic::{AssertUnwindSafe, Location},
    task::Poll,
};

use crate::{
    AssertThat, PanicValue,
    actual::Actual,
    assertions::support::project_checked,
    entry::panic_message,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, ValueRenderer},
};

// Invocation and polling belong to the consuming adapters. These definitions inspect only the
// resulting observation, so explaining a rejection can never invoke or poll user code again.
type Invocation<O> = Result<O, Box<dyn Any + Send>>;

/// Requires a panic. `returned_future` adds a hint for a function whose output looks like a future
/// that the synchronous assertion never polled.
struct Panicked {
    returned_future: bool,
}

impl<R> Expectation<Invocation<()>, R> for Panicked {
    type Success<'a> = &'a Box<dyn Any + Send>;
    type Rejection<'a> = &'a ();

    fn evaluate<'a>(
        &'a self,
        actual: &'a Invocation<()>,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual {
            Ok(output) => Err(output),
            Err(payload) => Ok(payload),
        }
    }

    const KIND: FailureKind = FailureKind::Panic;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Invocation<()>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        _: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        if rejected.is_none() {
            return failure.relation("panics");
        }
        let failure = failure.relation("did not panic");
        if self.returned_future {
            failure.fact(Fact::note(
                "The function returned a future, which is never polled. Use `panics_async` to run it.",
            ))
        } else {
            failure
        }
    }
}

/// Whether `O` is probably a future, judged by its type name. The result only selects a hint, so
/// an unusual name merely omits or adds it.
fn looks_like_future<O>() -> bool {
    let name = core::any::type_name::<O>();
    name.contains("{async") || name.contains("future") || name.contains("Future")
}

struct DidNotPanic;

impl<O, R: ValueRenderer<str>> Expectation<Invocation<O>, R> for DidNotPanic {
    type Success<'a>
        = &'a O
    where
        Self: 'a,
        Invocation<O>: 'a;
    type Rejection<'a>
        = &'a Box<dyn Any + Send>
    where
        Self: 'a,
        Invocation<O>: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Invocation<O>,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.as_ref()
    }

    const KIND: FailureKind = FailureKind::Panic;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Invocation<O>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let Some((_, payload)) = rejected else {
            return failure.relation("does not panic");
        };
        let failure = failure.relation("unexpectedly panicked");
        match panic_message(payload.as_ref()) {
            Some(message) => failure.fact(Fact::labelled(
                "Panic message",
                context.render().value(message),
            )),
            None => failure,
        }
    }
}

/// Takes ownership of the function subject, which invoking `FnOnce` consumes.
fn owned_fn<F>(actual: Actual<'_, F>) -> F {
    match actual {
        Actual::Owned(function) => function,
        Actual::Borrowed(_) => panic!(
            "Function assertions consume the function and therefore need to own it. Write the closure inside `assert_that!(|| ...)`, or use `assert_that_owned!(f)` (or `f.must_owned()`) for a named closure."
        ),
    }
}

/// Drops a successful invocation's output, catching a panic raised by its `Drop` implementation.
fn drop_output<O>(invocation: Invocation<O>) -> Invocation<()> {
    std::panic::catch_unwind(AssertUnwindSafe(move || invocation.map(drop))).flatten()
}

/// The panic payload of an invocation that [`Panicked`] accepted.
fn panic_value(invocation: Actual<'_, Invocation<()>>) -> Actual<'_, PanicValue> {
    project_checked(invocation, |it| it.err().map(PanicValue), |_| None)
}

/// The output of an invocation that [`DidNotPanic`] accepted.
fn output<O>(invocation: Actual<'_, Invocation<O>>) -> Actual<'_, O> {
    project_checked(invocation, Result::ok, |it| it.as_ref().ok())
}

/// Invokes `function` and awaits its future, catching a panic raised by either.
///
/// This is the async counterpart of [`std::panic::catch_unwind`]. The future is pinned on the heap,
/// so the poll loop needs no unsafe pin projection, and every individual poll is wrapped in
/// `catch_unwind`. Once a poll panics, its payload is returned and the future is dropped without
/// ever being polled again.
async fn invoke_async<F, Fut>(function: F) -> Invocation<Fut::Output>
where
    F: FnOnce() -> Fut,
    Fut: Future,
{
    let mut future = Box::pin(std::panic::catch_unwind(AssertUnwindSafe(function))?);
    core::future::poll_fn(move |cx| {
        match std::panic::catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(cx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Err(payload) => Poll::Ready(Err(payload)),
        }
    })
    .await
}

/// Assertions that invoke a synchronous `FnOnce` subject.
///
/// These methods are available only in panic mode because they change the subject type. Invoking
/// `FnOnce` consumes it, so the chain must own the closure. `assert_that!(|| ..)` owns a closure
/// literal. Use `assert_that_owned!(f)` or `f.must_owned()` for a named closure. Calling either
/// method on a borrowed subject panics.
///
/// ```
/// use assertr::prelude::*;
///
/// assert_that!(|| "x".parse::<u32>().unwrap()).panics();
/// assert_that!(|| 1 + 1).does_not_panic().is_equal_to(2);
/// ```
///
/// # Functions returning a future
///
/// A function returning a future, such as `async || ..` or `|| async { .. }`, only creates the
/// future here. Its body never runs, so `does_not_panic` passes and `panics` fails whatever the
/// body does. Use [`AsyncFnOnceAssertions`] (`panics_async` and `does_not_panic_async`) to await
/// it. A `panics` failure hints at this when the output looks like a future.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait FnOnceAssertions<'t, O, R = DebugRenderer> {
    /// Asserts that invoking the function or dropping its output panics, then returns the payload.
    ///
    /// A returned future is not polled. See
    /// [functions returning a future](Self#functions-returning-a-future).
    fn panics(self) -> AssertThat<'t, PanicValue, Panic, R>;

    /// Asserts that invoking the function does not panic, then returns its output.
    ///
    /// Dropping the output is outside the caught unwind boundary. A returned future is not polled,
    /// so this passes without running its body. See
    /// [functions returning a future](Self#functions-returning-a-future).
    fn does_not_panic(self) -> AssertThat<'t, O, Panic, R>
    where
        R: ValueRenderer<str>;
}

impl<'t, O, R, F: FnOnce() -> O> FnOnceAssertions<'t, O, R> for AssertThat<'t, F, Panic, R> {
    #[track_caller]
    fn panics(self) -> AssertThat<'t, PanicValue, Panic, R> {
        self.track_assertion();
        self.map(|function| {
            let invocation = std::panic::catch_unwind(AssertUnwindSafe(owned_fn(function)));
            Actual::Owned(drop_output(invocation))
        })
        .apply_assertion_after_tracking(
            Panicked {
                returned_future: looks_like_future::<O>(),
            },
            Location::caller(),
        )
        .map(panic_value)
    }

    #[track_caller]
    fn does_not_panic(self) -> AssertThat<'t, O, Panic, R>
    where
        R: ValueRenderer<str>,
    {
        self.track_assertion();
        // The output remains available for later assertions, so dropping it is outside this
        // unwind boundary.
        self.map(|function| {
            Actual::Owned(std::panic::catch_unwind(AssertUnwindSafe(owned_fn(
                function,
            ))))
        })
        .apply_assertion_after_tracking(DidNotPanic, Location::caller())
        .map(output)
    }
}

/// Assertions that invoke an async `FnOnce` subject and poll its returned future.
///
/// These methods are available only in panic mode because they change the subject type. Invoking
/// `FnOnce` consumes it, so the chain must own the closure. `assert_that!(async || ..)` owns a
/// closure literal. Use `assert_that_owned!(f)` or `f.must_owned()` for a named closure. Awaiting
/// either method on a borrowed subject panics.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait AsyncFnOnceAssertions<'t, O, R = DebugRenderer> {
    /// Asserts that invoking the function, polling its future, or dropping its output panics, then
    /// returns the payload.
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, Panic, R>>;

    /// Asserts that invoking the function and polling its future do not panic, then returns its
    /// output.
    ///
    /// Dropping the output is outside the caught unwind boundary.
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, O, Panic, R>>
    where
        O: 't,
        R: ValueRenderer<str>;
}

// The caller is captured at the method call. Tracking and invocation happen on the first poll.
impl<'t, Fut, O, R, F> AsyncFnOnceAssertions<'t, O, R> for AssertThat<'t, F, Panic, R>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = O>,
{
    #[track_caller]
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, Panic, R>> {
        let location = Location::caller();
        async move {
            self.track_assertion();
            self.map_async(|function| {
                let function = owned_fn(function);
                async move { drop_output(invoke_async(function).await) }
            })
            .await
            .apply_assertion_after_tracking(
                Panicked {
                    returned_future: false,
                },
                location,
            )
            .map(panic_value)
        }
    }

    #[track_caller]
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, O, Panic, R>>
    where
        O: 't,
        R: ValueRenderer<str>,
    {
        let location = Location::caller();
        async move {
            self.track_assertion();
            self.map_async(|function| invoke_async(owned_fn(function)))
                .await
                .apply_assertion_after_tracking(DidNotPanic, location)
                .map(output)
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[tokio::test]
        async fn are_as_expected() {
            (|| unimplemented!()).must_owned().panic();
            (|| 42).must_owned().not_panic();
            (async || unimplemented!()).must_owned().panic_async().await;
            (async || 42).must_owned().not_panic_async().await;
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, fn() -> (), Panic, NoRenderer>
                    => FnOnceAssertions<'static, (), NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, fn() -> core::future::Ready<()>, Panic, NoRenderer>
                    => AsyncFnOnceAssertions<'static, (), NoRenderer>
            );
        }

        #[tokio::test]
        async fn successful_panic_observations_need_no_renderer() {
            let synchronous = assert_that!(|| panic!("sync"))
                .with_renderer(NoRenderer)
                .panics();
            assert_that!(synchronous.actual().0.is::<&str>()).is_true();
            assert_that!(synchronous.state.records.assertion_count()).is_equal_to(1);
            let asynchronous = assert_that!(|| async { panic!("async") })
                .with_renderer(NoRenderer)
                .panics_async()
                .await;
            assert_that!(asynchronous.actual().0.is::<&str>()).is_true();
            assert_that!(asynchronous.state.records.assertion_count()).is_equal_to(1);
        }
    }

    mod fn_once {
        mod panics {
            use indoc::formatdoc;

            use crate::prelude::*;

            #[test]
            fn caller_location_is_as_expected() {
                assert_caller_location!(assert_that!(|| 42), panics());
            }

            #[test]
            fn succeeds_when_panic_occurs() {
                assert_that!(|| unimplemented!())
                    .panics()
                    .has_message()
                    .is_equal_to("not implemented");
            }

            #[test]
            fn succeeds_when_dropping_the_output_panics() {
                struct PanicsOnDrop;

                impl Drop for PanicsOnDrop {
                    fn drop(&mut self) {
                        panic!("output drop");
                    }
                }

                assert_that!(|| PanicsOnDrop)
                    .panics()
                    .has_message()
                    .is_equal_to("output drop");
            }

            #[test]
            fn later_failure_does_not_report_that_the_function_did_not_panic() {
                assert_that!(|| {
                    assert_that!(|| panic!("boom"))
                        .with_location(false)
                        .panics()
                        .has_type::<String>();
                })
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| panic!("boom")`

                    Actual: &str

                    is not of the expected type

                    Expected: alloc::string::String
                    -------- assertr --------
                "#});
            }

            #[test]
            fn hints_at_panics_async_when_the_function_returns_a_future() {
                const HINT: &str = "The function returned a future, which is never polled. Use `panics_async` to run it.";
                assert_that!(|| assert_that!(async || panic!("never runs")).panics())
                    .panics()
                    .has_type::<String>()
                    .contains(HINT);
                assert_that!(|| assert_that!(|| async { panic!("never runs") }).panics())
                    .panics()
                    .has_type::<String>()
                    .contains(HINT);
                assert_that!(|| assert_that!(|| 42).panics())
                    .panics()
                    .has_type::<String>()
                    .does_not_contain(HINT);
            }

            #[test]
            fn panics_when_no_panic_occurs() {
                assert_that!(|| assert_that!(|| 42).with_location(false).panics())
                    .panics()
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r"
                        -------- assertr --------
                        Expression: `|| 42`

                        did not panic
                        -------- assertr --------
                    "});
            }
        }

        mod does_not_panic {
            use indoc::formatdoc;

            use crate::prelude::*;

            #[test]
            fn caller_location_is_as_expected() {
                assert_caller_location!(assert_that!(|| panic!("subject panic")), does_not_panic());
            }

            #[test]
            fn string_payloads_use_the_active_renderer() {
                use crate::test_support::{CustomValueRenderer, RedactingRenderer};
                let message = "private-panic-value";
                for owned in [false, true] {
                    assert_that!(|| {
                        assert_that!(|| if owned {
                            std::panic::panic_any(message.to_owned());
                        } else {
                            std::panic::panic_any(message);
                        })
                        .with_renderer(CustomValueRenderer)
                        .with_location(false)
                        .does_not_panic();
                    }).panics()
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `|| if owned {{ std::panic::panic_any(message.to_owned()); }} else...`

                        unexpectedly panicked

                        Details:
                          - Panic message: custom("private-panic-value")
                        -------- assertr --------
                    "#});
                    assert_that!(|| {
                        assert_that!(|| if owned {
                            std::panic::panic_any(message.to_owned());
                        } else {
                            std::panic::panic_any(message);
                        })
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .does_not_panic();
                    }).panics()
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r"
                        -------- assertr --------
                        Expression: `|| if owned {{ std::panic::panic_any(message.to_owned()); }} else...`

                        unexpectedly panicked

                        Details:
                          - Panic message: <redacted>
                        -------- assertr --------
                    "});
                }
            }

            #[test]
            fn succeeds_when_no_panic_occurs() {
                assert_that!(|| 42).does_not_panic();
            }

            #[test]
            fn invokes_once_after_tracking_and_retains_the_output() {
                use core::cell::Cell;

                struct Output<'a>(&'a Cell<usize>);
                impl Drop for Output<'_> {
                    fn drop(&mut self) {
                        self.0.set(self.0.get() + 1);
                    }
                }

                let root = assert_that!(());
                let drops = Cell::new(0);
                let mut invocations = 0;
                let assertion = root
                    .derive_owned(|()| {
                        || {
                            invocations += 1;
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            Output(&drops)
                        }
                    })
                    .does_not_panic();
                assert_that!(drops.get()).is_equal_to(0);
                assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
                drop(assertion);
                assert_that!(invocations).is_equal_to(1);
                assert_that!(drops.get()).is_equal_to(1);
            }

            #[test]
            fn later_failure_does_not_report_that_the_function_panicked() {
                assert_that!(|| {
                    assert_that!(|| "actual")
                        .with_location(false)
                        .does_not_panic()
                        .is_equal_to("expected");
                })
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| "actual"`

                    Expected: "expected"

                      Actual: "actual"
                    -------- assertr --------
                "#});
            }
        }
    }

    mod async_fn_once {
        mod observations {
            use core::{
                cell::Cell,
                pin::Pin,
                task::{Context, Poll},
            };

            use crate::prelude::*;

            struct PanickingFuture<'a> {
                polls: &'a Cell<usize>,
                drops: &'a Cell<usize>,
            }

            impl Future for PanickingFuture<'_> {
                type Output = ();
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                    self.polls.set(self.polls.get() + 1);
                    if self.polls.get() == 1 {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    } else {
                        panic!("poll panic");
                    }
                }
            }

            impl Drop for PanickingFuture<'_> {
                fn drop(&mut self) {
                    self.drops.set(self.drops.get() + 1);
                }
            }

            #[tokio::test]
            async fn invocation_is_lazy_and_panicked_futures_are_never_repolled() {
                for expects_panic in [true, false] {
                    let root = assert_that!(());
                    let invocations = Cell::new(0);
                    let polls = Cell::new(0);
                    let drops = Cell::new(0);
                    let child = root.derive_owned(|()| {
                        || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            invocations.set(invocations.get() + 1);
                            PanickingFuture {
                                polls: &polls,
                                drops: &drops,
                            }
                        }
                    });
                    if expects_panic {
                        let pending = child.panics_async();
                        assert_that!(root.state.records.assertion_count()).is_equal_to(0);
                        assert_that!(invocations.get()).is_equal_to(0);
                        let result = pending.await;
                        assert_that!(result.state.records.assertion_count()).is_equal_to(1);
                        result.has_type::<&str>().is_equal_to("poll panic");
                    } else {
                        let pending = child.does_not_panic_async();
                        assert_that!(root.state.records.assertion_count()).is_equal_to(0);
                        assert_that!(invocations.get()).is_equal_to(0);
                        assert_that!(async || {
                            pending.await;
                        })
                        .panics_async()
                        .await
                        .has_type::<String>()
                        .contains("unexpectedly panicked");
                        assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    }
                    assert_that!(invocations.get()).is_equal_to(1);
                    assert_that!(polls.get()).is_equal_to(2);
                    assert_that!(drops.get()).is_equal_to(1);
                }
            }
        }

        mod panics {
            use crate::prelude::*;

            #[test]
            fn caller_location_is_as_expected() {
                assert_caller_location!(async assert_that!(|| async {}), panics_async());
            }

            #[tokio::test]
            async fn succeeds_when_panic_occurs() {
                assert_that!(async || unimplemented!())
                    .panics_async()
                    .await
                    .has_message()
                    .is_equal_to("not implemented");
            }

            #[tokio::test]
            async fn succeeds_when_panic_occurs_after_yielding() {
                assert_that!(async || {
                    tokio::task::yield_now().await;
                    panic!("boom");
                })
                .panics_async()
                .await
                .has_message()
                .is_equal_to("boom");
            }

            #[tokio::test]
            async fn succeeds_when_function_panics_before_returning_its_future() {
                assert_that!(|| -> core::future::Ready<()> { panic!("before future") })
                    .panics_async()
                    .await
                    .has_message()
                    .is_equal_to("before future");
            }

            #[tokio::test]
            async fn succeeds_when_dropping_the_output_panics() {
                struct PanicsOnDrop;

                impl Drop for PanicsOnDrop {
                    fn drop(&mut self) {
                        panic!("output drop");
                    }
                }

                assert_that!(async || PanicsOnDrop)
                    .panics_async()
                    .await
                    .has_message()
                    .is_equal_to("output drop");
            }

            #[tokio::test]
            async fn panics_when_no_panic_occurs() {
                assert_that!(async || { assert_that!(async || 42).panics_async().await })
                    .panics_async()
                    .await
                    .has_type::<String>()
                    .contains("did not panic");
            }

            #[tokio::test]
            async fn later_failure_does_not_report_that_the_function_did_not_panic() {
                assert_that!(async || {
                    assert_that!(async || panic!("boom"))
                        .panics_async()
                        .await
                        .has_type::<String>();
                })
                .panics_async()
                .await
                .has_type::<String>()
                .contains("is not of the expected type");
            }
        }

        mod does_not_panic {
            use crate::prelude::*;

            #[test]
            fn caller_location_is_as_expected() {
                assert_caller_location!(async assert_that!(|| async { panic!("subject panic") }), does_not_panic_async());
            }

            #[tokio::test]
            async fn string_payloads_use_the_active_renderer() {
                use crate::test_support::{CustomValueRenderer, RedactingRenderer};

                let message = "private-async-panic-value";
                for owned in [false, true] {
                    let panic = || -> () {
                        if owned {
                            std::panic::panic_any(message.to_owned())
                        }
                        std::panic::panic_any(message)
                    };
                    assert_that!(async || {
                        assert_that!(async || panic())
                            .with_renderer(CustomValueRenderer)
                            .does_not_panic_async()
                            .await;
                    })
                    .panics_async()
                    .await
                    .has_type::<String>()
                    .contains(r#"Panic message: custom("private-async-panic-value")"#);
                    assert_that!(async || {
                        assert_that!(async || panic())
                            .with_renderer(RedactingRenderer)
                            .does_not_panic_async()
                            .await;
                    })
                    .panics_async()
                    .await
                    .has_type::<String>()
                    .contains("Panic message: <redacted>");
                }
            }

            #[tokio::test]
            async fn succeeds_when_no_panic_occurs() {
                assert_that!(async || 42).does_not_panic_async().await;
            }

            #[tokio::test]
            async fn succeeds_when_future_yields_before_completing() {
                assert_that!(async || {
                    tokio::task::yield_now().await;
                    42
                })
                .does_not_panic_async()
                .await
                .is_equal_to(42);
            }

            #[tokio::test]
            async fn later_failure_does_not_report_that_the_function_panicked() {
                assert_that!(async || {
                    assert_that!(async || "actual")
                        .does_not_panic_async()
                        .await
                        .is_equal_to("expected");
                })
                .panics_async()
                .await
                .has_type::<String>()
                .contains(r#"Expected: "expected""#);
            }

            #[tokio::test]
            async fn fails_when_function_panics_before_returning_its_future() {
                assert_that!(async || {
                    assert_that!(|| -> core::future::Ready<()> { panic!("before future") })
                        .does_not_panic_async()
                        .await
                })
                .panics_async()
                .await
                .has_type::<String>()
                .contains(r#"Panic message: "before future""#);
            }
        }
    }
}
