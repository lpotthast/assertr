use crate::actual::Actual;
use crate::mode::Mode;
use crate::prelude::ResultAssertions;
use crate::tracking::AssertionTracking;
use crate::{AssertThat, PanicValue};
use core::fmt::{Debug, Write};
use futures::FutureExt;
use indoc::writedoc;
use std::any::Any;
use std::panic::UnwindSafe;

pub trait FnOnceAssertions<'t, R, M: Mode> {
    #[cfg(feature = "std")]
    fn panics(self) -> AssertThat<'t, PanicValue, M>;

    #[cfg(feature = "std")]
    fn does_not_panic(self) -> AssertThat<'t, R, M>
    where
        R: Debug;
}

impl<'t, R, F: FnOnce() -> R, M: Mode> FnOnceAssertions<'t, R, M> for AssertThat<'t, F, M> {
    #[track_caller]
    fn panics(self) -> AssertThat<'t, PanicValue, M> {
        self.track_assertion();

        let this: AssertThat<Result<(), Box<dyn Any + Send + 'static>>, M> =
            self.map(|it| match it {
                Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
                Actual::Owned(f) => Actual::Owned(
                    std::panic::catch_unwind(core::panic::AssertUnwindSafe(f)).map(|it| ()),
                ),
            });

        if this.actual().is_ok() {
            this.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Function to panic when called.

                      Actual: No panic occurred!
                "#}
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
    fn does_not_panic(self) -> AssertThat<'t, R, M>
    where
        R: Debug,
    {
        self.track_assertion();

        let this: AssertThat<Result<R, Box<dyn Any + Send + 'static>>, M> =
            self.map(|it| match it {
                Actual::Borrowed(_) => {
                    panic!("does_not_panic() can only be called on an owned FnOnce!")
                }
                Actual::Owned(f) => {
                    Actual::Owned(std::panic::catch_unwind(core::panic::AssertUnwindSafe(f)))
                }
            });

        if this.actual().is_err() {
            this.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
                "#}
            });
        }

        this.is_ok()
            .with_detail_message("Function panicked unexpectedly!")
            .map(|it| it.unwrap_owned().into())
    }
}

pub trait AsyncFnOnceAssertions<'t, R, M: Mode> {
    #[cfg(feature = "std")]
    fn panics_async(self) -> impl Future<Output = AssertThat<'t, PanicValue, M>>;

    #[cfg(feature = "std")]
    fn does_not_panic_async(self) -> impl Future<Output = AssertThat<'t, R, M>>
    where
        R: Debug + 't;
}

impl<'t, Fut, R, F, M: Mode> AsyncFnOnceAssertions<'t, R, M> for AssertThat<'t, F, M>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = R> + UnwindSafe,
{
    // #[track_caller] // This is implied in the default async desugaring.
    async fn panics_async(self) -> AssertThat<'t, PanicValue, M> {
        self.track_assertion();

        // Execute the user function
        let this: AssertThat<Result<(), Box<dyn Any + Send>>, M> = self
            .map_async2(async |it| match it {
                Actual::Borrowed(_) => {
                    panic!("panics_async() can only be called on an owned FnOnce!")
                }
                Actual::Owned(f) => Actual::Owned(f().catch_unwind().await.map(|_| ())),
            })
            .await;

        if this.actual().is_ok() {
            this.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Function to panic when called.

                      Actual: No panic occurred!
                "#}
            });
        }

        this.is_err()
            .with_detail_message("Function did not panic as expected!")
            .map(|it| {
                let boxed_any = it.unwrap_owned();
                PanicValue(boxed_any).into()
            })
    }

    // #[track_caller] // This is implied in the default async desugaring.
    async fn does_not_panic_async(self) -> AssertThat<'t, R, M>
    where
        R: Debug + 't,
    {
        self.track_assertion();

        let this: AssertThat<Result<R, Box<dyn Any + Send + 'static>>, M> = self
            .map_async2(async |it| match it {
                Actual::Borrowed(_) => {
                    panic!("does_not_panic_async() can only be called on an owned FnOnce!")
                }
                Actual::Owned(f) => {
                    Actual::Owned(core::panic::AssertUnwindSafe(f()).catch_unwind().await)
                }
            })
            .await;

        if this.actual().is_err() {
            this.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: Function to not panic when called.

                      Actual: Function panicked unexpectedly!
                "#}
            });
        }

        this.is_ok()
            .with_detail_message("Function panicked unexpectedly!")
            .map(|it| it.unwrap_owned().into())
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
                assert_that(|| unimplemented!())
                    .panics()
                    .has_type::<&str>()
                    .is_equal_to("not implemented");
            }

            #[test]
            fn panics_when_no_panic_occurs() {
                assert_that_panic_by(|| assert_that(|| 42).with_location(false).panics())
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
                assert_that(|| 42).does_not_panic();
            }

            #[test]
            fn fails_when_panic_occurs() {
                assert_that_panic_by(|| {
                    assert_that(|| unimplemented!())
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
                assert_that(async || unimplemented!())
                    .panics_async()
                    .await
                    .has_type::<&str>()
                    .is_equal_to("not implemented");
            }

            #[tokio::test]
            async fn panics_when_no_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that(async || 42)
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
                assert_that(|| 42).does_not_panic();
            }

            #[tokio::test]
            async fn fails_when_panic_occurs() {
                assert_that_panic_by_async(async || {
                    assert_that(async || unimplemented!())
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
