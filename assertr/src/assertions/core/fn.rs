use core::fmt::Debug;

use crate::actual::Actual;
use crate::mode::Mode;
use crate::prelude::ResultAssertions;
use crate::tracking::AssertionTracking;
use crate::{AssertThat, PanicValue};

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
        self.map(|it| match it {
            Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
            Actual::Owned(f) => Actual::Owned(std::panic::catch_unwind(
                core::panic::AssertUnwindSafe(move || {
                    f();
                }),
            )),
        })
        .with_detail_message("Function did not panic as expected!")
        .is_err()
        .map(|it| PanicValue(it.unwrap_owned()).into())
    }

    #[track_caller]
    fn does_not_panic(self) -> AssertThat<'t, R, M>
    where
        R: Debug,
    {
        self.track_assertion();
        self.map(|it| match it {
            Actual::Borrowed(_) => panic!("panics() can only be called on an owned FnOnce!"),
            Actual::Owned(f) => Actual::Owned(std::panic::catch_unwind(
                core::panic::AssertUnwindSafe(move || f()),
            )),
        })
        .with_detail_message("Function panicked unexpectedly!")
        .is_ok()
        .map(|it| it.unwrap_owned().into())
    }
}

#[cfg(test)]
mod tests {

    mod panics {}

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
                Actual: Err(
                    Any {{ .. }},
                )
                
                is not of expected variant: Result:Ok
                
                Details: [
                    Function panicked unexpectedly!,
                ]
                -------- assertr --------
            "#});
        }
    }
}
