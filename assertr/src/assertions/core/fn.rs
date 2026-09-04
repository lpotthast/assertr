use crate::actual::Actual;
use crate::failure::FailureKind;
use crate::mode::Panic;
use crate::{AssertThat, PanicValue};
use alloc::{boxed::Box, string::String};
use core::any::Any;
use core::panic::Location;
#[cfg(feature = "std")]
use core::task::Poll;

/// The message of a panic payload raised through `panic!` or `panic_any` with a `&str` or a
/// `String`. A payload of any other type carries no message that could be shown.
fn panic_message(payload: &(dyn Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

/// Raises the failure of a `does_not_panic` assertion at `location`, with the panic message as
/// evidence when the payload carries one.
fn raise_unexpected_panic<T, R>(
    this: &AssertThat<'_, T, Panic, R>,
    payload: &(dyn Any + Send),
    location: &'static Location<'static>,
) {
    let mut failure = this
        .failure_at(FailureKind::Panic, location)
        .relation("unexpectedly panicked");
    if let Some(message) = panic_message(payload) {
        failure = failure.fact("Panic message", format_args!("{message:?}"));
    }
    failure.raise();
}

/// Awaits `future`, catching a panic raised while it is polled.
///
/// This is the async counterpart of [`std::panic::catch_unwind`]. The future is pinned on the
/// heap, so the poll loop needs no unsafe pin projection, and every individual poll is wrapped
/// in `catch_unwind`. Once a poll panics, its payload is returned and the future is dropped
/// without ever being polled again.
#[cfg(feature = "std")]
async fn catch_unwind_future<Fut>(future: Fut) -> Result<Fut::Output, Box<dyn Any + Send>>
where
    Fut: Future,
{
    let mut future = Box::pin(future);
    core::future::poll_fn(move |cx| {
        match std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| future.as_mut().poll(cx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Err(panic_value) => Poll::Ready(Err(panic_value)),
        }
    })
    .await
}

/// Assertions that invoke a synchronous `FnOnce` subject.
///
/// These methods are available only in panic mode because they change the subject type.
/// Invoking `FnOnce` consumes it, so create the assertion with `assert_that_owned!` or
/// `.must_owned()`. Calling either method on a borrowed subject panics.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait FnOnceAssertions<'t, O, R = crate::DebugRenderer> {
    /// Asserts that invoking the function or dropping its output panics, then returns the payload.
    #[cfg(feature = "std")]
    fn panics(self) -> AssertThat<'t, PanicValue, Panic, R>;

    /// Asserts that invoking the function does not panic, then returns its output.
    ///
    /// Dropping the output is outside the caught unwind boundary.
    #[cfg(feature = "std")]
    fn does_not_panic(self) -> AssertThat<'t, O, Panic, R>;
}

impl<'t, O, R, F: FnOnce() -> O> FnOnceAssertions<'t, O, R> for AssertThat<'t, F, Panic, R> {
    #[track_caller]
    #[cfg(feature = "std")]
    fn panics(self) -> AssertThat<'t, PanicValue, Panic, R> {
        self.track_assertion();

        let this: AssertThat<Result<(), Box<dyn Any + Send + 'static>>, Panic, R> =
            self.map(|it| match it {
                Actual::Borrowed(_) => panic!(
                    "panics() consumes the function and can only be called on an owned FnOnce! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
                ),
                Actual::Owned(f) => {
                    // First, call the closure, receiving its output.
                    let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(f));

                    // Then, we drop the output,
                    // while catching any panics resulting from the `Drop` implementation.
                    let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(move || {
                        res.map(|value| drop(value))
                    }));

                    Actual::Owned(res.flatten())
                }
            });

        if this.actual().is_ok() {
            this.failure(FailureKind::Panic)
                .relation("did not panic")
                .raise();
        }

        this.map(|it| match it {
            Actual::Owned(Err(boxed_any)) => Actual::Owned(PanicValue(boxed_any)),
            Actual::Owned(Ok(())) => unreachable!("already checked"),
            Actual::Borrowed(_) => unreachable!("mapped assertion owns its subject"),
        })
    }

    #[track_caller]
    #[cfg(feature = "std")]
    fn does_not_panic(self) -> AssertThat<'t, O, Panic, R> {
        self.track_assertion();

        let this: AssertThat<Result<O, Box<dyn Any + Send + 'static>>, Panic, R> =
            self.map(|it| match it {
                Actual::Borrowed(_) => {
                    panic!(
                        "does_not_panic() consumes the function and can only be called on an owned FnOnce! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
                    )
                }
                Actual::Owned(f) => {
                    // Catch a panic from the function call but retain its output for further
                    // assertions. Dropping the output is therefore outside this unwind boundary.
                    let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(f));
                    Actual::Owned(res)
                }
            });

        if let Err(payload) = this.actual() {
            raise_unexpected_panic(&this, payload.as_ref(), Location::caller());
        }

        this.map(|it| match it {
            Actual::Owned(Ok(output)) => Actual::Owned(output),
            Actual::Owned(Err(_)) => unreachable!("already checked"),
            Actual::Borrowed(_) => unreachable!("mapped assertion owns its subject"),
        })
    }
}

/// Assertions that invoke an async `FnOnce` subject and poll its returned future.
///
/// These methods are available only in panic mode because they change the subject type.
/// Invoking `FnOnce` consumes it, so create the assertion with `assert_that_owned!` or
/// `.must_owned()`. Awaiting either method on a borrowed subject panics.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait AsyncFnOnceAssertions<'t, O, R = crate::DebugRenderer> {
    /// Asserts that invoking the function, polling its future, or dropping its output panics, then
    /// returns the payload.
    #[cfg(feature = "std")]
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, Panic, R>>;

    /// Asserts that invoking the function and polling its future do not panic, then returns its
    /// output.
    ///
    /// Dropping the output is outside the caught unwind boundary.
    #[cfg(feature = "std")]
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, O, Panic, R>>
    where
        O: 't;
}

impl<'t, Fut, O, R, F> AsyncFnOnceAssertions<'t, O, R> for AssertThat<'t, F, Panic, R>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = O>,
{
    #[track_caller]
    #[cfg(feature = "std")]
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, Panic, R>> {
        panics_async_at(self, Location::caller())
    }

    #[track_caller]
    #[cfg(feature = "std")]
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, O, Panic, R>>
    where
        O: 't,
    {
        does_not_panic_async_at(self, Location::caller())
    }
}

#[cfg(feature = "std")]
pub(crate) async fn panics_async_at<'t, Fut, O, R, F>(
    assertion: AssertThat<'t, F, Panic, R>,
    location: &'static Location<'static>,
) -> AssertThat<'t, PanicValue, Panic, R>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = O>,
{
    assertion.track_assertion();

    // Execute the user function
    let this: AssertThat<Result<(), Box<dyn Any + Send>>, Panic, R> = assertion
        .map_async(|it| {
            let f = match it {
                Actual::Borrowed(_) => {
                    panic!(
                        "panics_async() consumes the function and can only be called on an owned FnOnce! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
                    )
                }
                Actual::Owned(f) => f,
            };
            async move {
                let future = match std::panic::catch_unwind(core::panic::AssertUnwindSafe(f)) {
                    Ok(future) => future,
                    Err(payload) => return Err(payload),
                };

                // Poll the future, receiving its output.
                let res = catch_unwind_future(future).await;

                // Then, we drop the output,
                // while catching any panics resulting from the `Drop` implementation.
                let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(move || {
                    res.map(|value| drop(value))
                }));

                res.flatten()
            }
        })
        .await;

    if this.actual().is_ok() {
        this.failure_at(FailureKind::Panic, location)
            .relation("did not panic")
            .raise();
    }

    this.map(|it| match it {
        Actual::Owned(Err(boxed_any)) => Actual::Owned(PanicValue(boxed_any)),
        Actual::Owned(Ok(())) => unreachable!("already checked"),
        Actual::Borrowed(_) => unreachable!("mapped assertion owns its subject"),
    })
}

#[cfg(feature = "std")]
async fn does_not_panic_async_at<'t, Fut, O, R, F>(
    assertion: AssertThat<'t, F, Panic, R>,
    location: &'static Location<'static>,
) -> AssertThat<'t, O, Panic, R>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = O>,
    O: 't,
{
    assertion.track_assertion();

    let this: AssertThat<Result<O, Box<dyn Any + Send + 'static>>, Panic, R> = assertion
        .map_async(|it| {
            let f = match it {
                Actual::Borrowed(_) => {
                    panic!(
                        "does_not_panic_async() consumes the function and can only be called on an owned FnOnce! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
                    )
                }
                Actual::Owned(f) => f,
            };
            async move {
                let future = match std::panic::catch_unwind(core::panic::AssertUnwindSafe(f)) {
                    Ok(future) => future,
                    Err(payload) => return Err(payload),
                };

                // The output remains available for later assertions, so dropping it is outside
                // this unwind boundary.
                catch_unwind_future(future).await
            }
        })
        .await;

    if let Err(payload) = this.actual() {
        raise_unexpected_panic(&this, payload.as_ref(), location);
    }

    this.map(|it| match it {
        Actual::Owned(Ok(output)) => Actual::Owned(output),
        Actual::Owned(Err(_)) => unreachable!("already checked"),
        Actual::Borrowed(_) => unreachable!("mapped assertion owns its subject"),
    })
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

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
    }

    mod fn_once {
        mod panics {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                (|| unimplemented!()).must_owned().panic();
            }

            #[test]
            fn succeeds_when_panic_occurs() {
                assert_that_owned!(|| unimplemented!())
                    .panics()
                    .has_type::<&str>()
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

                assert_that_owned!(|| PanicsOnDrop)
                    .panics()
                    .has_type::<&str>()
                    .is_equal_to("output drop");
            }

            #[test]
            fn later_failure_does_not_report_that_the_function_did_not_panic() {
                assert_that_panic_by(|| {
                    assert_that_owned!(|| panic!("boom"))
                        .with_location(false)
                        .panics()
                        .has_type::<String>();
                })
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
            fn panics_when_no_panic_occurs() {
                assert_that_panic_by(|| assert_that_owned!(|| 42).with_location(false).panics())
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
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            #[cfg(feature = "fluent")]
            fn fluent_alias_is_as_expected() {
                (|| 42).must_owned().not_panic();
            }

            #[test]
            fn succeeds_when_no_panic_occurs() {
                assert_that_owned!(|| 42).does_not_panic();
            }

            #[test]
            fn later_failure_does_not_report_that_the_function_panicked() {
                assert_that_panic_by(|| {
                    assert_that_owned!(|| "actual")
                        .with_location(false)
                        .does_not_panic()
                        .is_equal_to("expected");
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| "actual"`

                    Expected: "expected"

                      Actual: "actual"
                    -------- assertr --------
                "#});
            }

            #[test]
            fn fails_when_panic_occurs() {
                assert_that_panic_by(|| {
                    assert_that_owned!(|| unimplemented!())
                        .with_location(false)
                        .does_not_panic()
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| unimplemented!()`

                    unexpectedly panicked

                    Details:
                      - Panic message: "not implemented"
                    -------- assertr --------
                "#});
            }

            #[test]
            fn failure_includes_string_panic_payload() {
                assert_that_panic_by(|| {
                    assert_that_owned!(|| std::panic::panic_any(String::from("owned boom")))
                        .with_location(false)
                        .does_not_panic()
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| std::panic::panic_any(String::from("owned boom"))`

                    unexpectedly panicked

                    Details:
                      - Panic message: "owned boom"
                    -------- assertr --------
                "#});
            }
        }
    }

    mod async_fn_once {
        mod panics {
            use crate::assert_that_panic_by_async;
            use crate::prelude::*;
            use indoc::formatdoc;

            #[tokio::test]
            #[cfg(feature = "fluent")]
            async fn fluent_alias_is_as_expected() {
                (async || unimplemented!()).must_owned().panic_async().await;
            }

            #[tokio::test]
            async fn succeeds_when_panic_occurs() {
                assert_that_owned!(async || unimplemented!())
                    .panics_async()
                    .await
                    .has_type::<&str>()
                    .is_equal_to("not implemented");
            }

            #[tokio::test]
            async fn succeeds_when_panic_occurs_after_yielding() {
                assert_that_owned!(async || {
                    tokio::task::yield_now().await;
                    panic!("boom");
                })
                .panics_async()
                .await
                .has_type::<&str>()
                .is_equal_to("boom");
            }

            #[tokio::test]
            async fn succeeds_when_function_panics_before_returning_its_future() {
                assert_that_owned!(|| -> core::future::Ready<()> { panic!("before future") })
                    .panics_async()
                    .await
                    .has_type::<&str>()
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

                assert_that_owned!(async || PanicsOnDrop)
                    .panics_async()
                    .await
                    .has_type::<&str>()
                    .is_equal_to("output drop");
            }

            #[tokio::test]
            async fn later_failure_does_not_report_that_the_function_did_not_panic() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(async || panic!("boom"))
                        .with_location(false)
                        .panics_async()
                        .await
                        .has_type::<String>();
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `async || panic!("boom")`

                    Actual: &str

                    is not of the expected type

                    Expected: alloc::string::String
                    -------- assertr --------
                "#});
            }

            #[tokio::test]
            async fn panics_when_no_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(async || 42)
                        .with_location(false)
                        .panics_async()
                        .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                        -------- assertr --------
                        Expression: `async || 42`

                        did not panic
                        -------- assertr --------
                    "});
            }

            #[tokio::test]
            async fn failure_location_points_at_the_callers_assertion() {
                let expected_line = line!() + 2;
                let panic = assert_that_panic_by_async(async || {
                    assert_that_owned!(async || 42).panics_async().await;
                })
                .await;

                panic
                    .has_type::<String>()
                    .contains(format!("Assertion failed at {}:{expected_line}:", file!()));
            }
        }

        mod does_not_panic {
            use crate::assert_that_panic_by_async;
            use crate::prelude::*;
            use indoc::formatdoc;

            #[tokio::test]
            #[cfg(feature = "fluent")]
            async fn fluent_alias_is_as_expected() {
                (async || 42).must_owned().not_panic_async().await;
            }

            #[tokio::test]
            async fn succeeds_when_no_panic_occurs() {
                assert_that_owned!(async || 42).does_not_panic_async().await;
            }

            #[tokio::test]
            async fn succeeds_when_future_yields_before_completing() {
                assert_that_owned!(async || {
                    tokio::task::yield_now().await;
                    42
                })
                .does_not_panic_async()
                .await
                .is_equal_to(42);
            }

            #[tokio::test]
            async fn later_failure_does_not_report_that_the_function_panicked() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(async || "actual")
                        .with_location(false)
                        .does_not_panic_async()
                        .await
                        .is_equal_to("expected");
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `async || "actual"`

                    Expected: "expected"

                      Actual: "actual"
                    -------- assertr --------
                "#});
            }

            #[tokio::test]
            async fn fails_when_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(async || unimplemented!())
                        .with_location(false)
                        .does_not_panic_async()
                        .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `async || unimplemented!()`

                    unexpectedly panicked

                    Details:
                      - Panic message: "not implemented"
                    -------- assertr --------
                "#});
            }

            #[tokio::test]
            async fn fails_when_function_panics_before_returning_its_future() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(|| -> core::future::Ready<()> { panic!("before future") })
                        .with_location(false)
                        .does_not_panic_async()
                        .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `|| -> core::future::Ready<()> {{ panic!("before future") }}`

                    unexpectedly panicked

                    Details:
                      - Panic message: "before future"
                    -------- assertr --------
                "#});
            }

            #[tokio::test]
            async fn failure_includes_string_panic_payload() {
                assert_that_panic_by_async(async || {
                    assert_that_owned!(async || {
                        std::panic::panic_any(String::from("owned boom"))
                    })
                    .with_location(false)
                    .does_not_panic_async()
                    .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `async || {{ std::panic::panic_any(String::from("owned boom")) }}`

                    unexpectedly panicked

                    Details:
                      - Panic message: "owned boom"
                    -------- assertr --------
                "#});
            }

            #[tokio::test]
            async fn failure_location_points_at_the_callers_assertion() {
                let expected_line = line!() + 3;
                let panic = assert_that_panic_by_async(async || {
                    assert_that_owned!(async || panic!("boom"))
                        .does_not_panic_async()
                        .await;
                })
                .await;

                panic
                    .has_type::<String>()
                    .contains(format!("Assertion failed at {}:{expected_line}:", file!()));
            }
        }
    }
}
