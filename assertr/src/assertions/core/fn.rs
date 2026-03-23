use crate::actual::Actual;
use crate::mode::Panic;
use crate::prelude::ResultExtractAssertions;
use crate::tracking::AssertionTracking;
use crate::{AssertThat, PanicValue};
use core::fmt::{Debug, Write};
use futures::FutureExt;
use indoc::writedoc;
use std::any::Any;
use std::panic::UnwindSafe;

/// Data-extracting assertions for `FnOnce` values.
/// Only available in Panic mode, as these transform the assertion subject type.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait FnOnceAssertions<'t, R> {
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "fluent", fluent_alias("panic"))]
    fn panics(self) -> AssertThat<'t, PanicValue, Panic>;

    #[cfg(feature = "std")]
    #[cfg_attr(feature = "fluent", fluent_alias("not_panic"))]
    fn does_not_panic(self) -> AssertThat<'t, R, Panic>
    where
        R: Debug;
}

impl<'t, R, F: FnOnce() -> R> FnOnceAssertions<'t, R> for AssertThat<'t, F, Panic> {
    #[track_caller]
    fn panics(self) -> AssertThat<'t, PanicValue, Panic> {
        self.track_assertion();

        let this: AssertThat<Result<(), Box<dyn Any + Send + 'static>>, Panic> =
            self.map(|it| match it {
                Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
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
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Function to panic when called.

                      Actual: No panic occurred!
                "}
            });
        }

        this.is_err()
            .with_detail_message("Function did not panic as expected!")
            .map(|it| {
                let boxed_any = it.unwrap_owned();
                PanicValue(boxed_any).into()
            })
    }

    #[track_caller]
    fn does_not_panic(self) -> AssertThat<'t, R, Panic>
    where
        R: Debug,
    {
        self.track_assertion();

        let this: AssertThat<Result<R, Box<dyn Any + Send + 'static>>, Panic> =
            self.map(|it| match it {
                Actual::Borrowed(_) => {
                    panic!("does_not_panic() can only be called on an owned FnOnce!")
                }
                Actual::Owned(f) => {
                    // Only capture the closures output while catching panics.
                    // We do not expect ANY panics, so just put the output in the returned
                    // assertion context. Should a `Drop` impl of R lead to a panic,
                    // the asserting code will see that panic.
                    // We cannot test for drop panics in a more deliberate way hare,
                    // e.g. by actually trying to drop the value, because we want the
                    // user to be a ble to issue further assertions on value of `R`.
                    let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(f));
                    Actual::Owned(res)
                }
            });

        if this.actual().is_err() {
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
                "}
            });
        }

        this.is_ok()
            .with_detail_message("Function panicked unexpectedly!")
            .map(|it| it.unwrap_owned().into())
    }
}

/// Data-extracting assertions for async `FnOnce` values.
/// Only available in Panic mode.
pub trait AsyncFnOnceAssertions<'t, R> {
    #[cfg(feature = "std")]
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, Panic>>;

    #[cfg(feature = "std")]
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, R, Panic>>
    where
        R: Debug + 't;
}

impl<'t, Fut, R, F> AsyncFnOnceAssertions<'t, R> for AssertThat<'t, F, Panic>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = R> + UnwindSafe,
{
    // #[track_caller] // This is implied in the default async desugaring.
    async fn panics_async(self) -> AssertThat<'t, PanicValue, Panic> {
        self.track_assertion();

        // Execute the user function
        let this: AssertThat<Result<(), Box<dyn Any + Send>>, Panic> = self
            .map_async(|it| {
                let f = match it {
                    Actual::Borrowed(_) => {
                        panic!("panics_async() can only be called on an owned FnOnce!")
                    }
                    Actual::Owned(f) => f,
                };
                async move {
                    // First, we await the future, receiving its output.
                    let res = FutureExt::catch_unwind(f()).await;

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
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Function to panic when called.

                      Actual: No panic occurred!
                "}
            });
        }

        this.is_err()
            .with_detail_message("Function did not panic as expected!")
            .map(|it| {
                let boxed_any: Box<dyn Any + Send> = it.unwrap_owned();
                Actual::Owned(PanicValue(boxed_any))
            })
    }

    // #[track_caller] // This is implied in the default async desugaring.
    async fn does_not_panic_async(self) -> AssertThat<'t, R, Panic>
    where
        R: Debug + 't,
    {
        self.track_assertion();

        let this: AssertThat<Result<R, Box<dyn Any + Send + 'static>>, Panic> = self
            .map_async(|it| {
                let f = match it {
                    Actual::Borrowed(_) => {
                        panic!("does_not_panic_async() can only be called on an owned FnOnce!")
                    }
                    Actual::Owned(f) => f,
                };
                async move {
                    // Only await the futures output while catching panics.
                    // We do not expect ANY panics, so just put the output in the returned
                    // assertion context. Should a `Drop` impl of R lead to a panic,
                    // the asserting code will see that panic.
                    // We cannot test for drop panics in a more deliberate way hare,
                    // e.g. by actually trying to drop the value, because we want the
                    // user to be a ble to issue further assertions on value of `R`.
                    FutureExt::catch_unwind(f()).await
                }
            })
            .await;

        if this.actual().is_err() {
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
                "}
            });
        }

        this.is_ok()
            .with_detail_message("Function panicked unexpectedly!")
            .map(|it| Actual::Owned(it.unwrap_owned()))
    }
}

#[cfg(test)]
mod tests {
    mod fn_once {
        mod panics {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_panic_occurs() {
                assert_that!(|| unimplemented!())
                    .panics()
                    .has_type::<&str>()
                    .is_equal_to("not implemented");
            }

            #[test]
            fn panics_when_no_panic_occurs() {
                assert_that_panic_by(|| assert_that!(|| 42).with_location(false).panics())
                    .has_type::<String>()
                    .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: Function to panic when called.

                          Actual: No panic occurred!
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_panic {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_no_panic_occurs() {
                assert_that!(|| 42).does_not_panic();
            }

            #[test]
            fn fails_when_panic_occurs() {
                assert_that_panic_by(|| {
                    assert_that!(|| unimplemented!())
                        .with_location(false)
                        .does_not_panic()
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
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
            async fn succeeds_when_panic_occurs() {
                assert_that!(async || unimplemented!())
                    .panics_async()
                    .await
                    .has_type::<&str>()
                    .is_equal_to("not implemented");
            }

            #[tokio::test]
            async fn panics_when_no_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that!(async || 42)
                        .with_location(false)
                        .panics_async()
                        .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                        -------- assertr --------
                        Expected: Function to panic when called.

                          Actual: No panic occurred!
                        -------- assertr --------
                    "#});
            }
        }

        mod does_not_panic {
            use crate::assert_that_panic_by_async;
            use crate::prelude::*;
            use indoc::formatdoc;

            #[tokio::test]
            async fn succeeds_when_no_panic_occurs() {
                assert_that!(|| 42).does_not_panic();
            }

            #[tokio::test]
            async fn fails_when_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that!(async || unimplemented!())
                        .with_location(false)
                        .does_not_panic_async()
                        .await
                })
                .await
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
                    -------- assertr --------
                "#});
            }
        }
    }
}
