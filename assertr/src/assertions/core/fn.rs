use crate::actual::Actual;
use crate::mode::Mode;
use crate::prelude::ResultAssertions;
use crate::tracking::AssertionTracking;
use crate::{AssertThat, PanicValue};
use core::fmt::{Debug, Write};
use indoc::writedoc;

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

        let this = self.map(|it| match it {
            Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
            Actual::Owned(f) => Actual::Owned(std::panic::catch_unwind(
                core::panic::AssertUnwindSafe(move || {
                    f();
                }),
            )),
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

        let this = self.map(|it| match it {
            Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
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

#[cfg(test)]
mod tests {

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
