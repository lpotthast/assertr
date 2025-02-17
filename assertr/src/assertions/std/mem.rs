use crate::{tracking::AssertionTracking, AssertThat, Mode, TypeHolder};
use core::fmt::Write;
use indoc::writedoc;

/// Static memory assertions for any type.
pub trait MemAssertions {
    fn needs_drop(self) -> Self;
}

impl<T, M: Mode> MemAssertions for AssertThat<'_, TypeHolder<T>, M> {
    #[track_caller]
    fn needs_drop(self) -> Self {
        self.track_assertion();

        if !std::mem::needs_drop::<T>() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Type {actual:#?} was expected to need drop,
                    
                    but dropping it is guaranteed to have no side effect.

                    You may forgot to `impl Drop` for this type.
                "#, actual = self.actual().get_type_name()}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod needs_drop {
        use crate::assert_that_type;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_type_needs_drop() {
            struct NeedsDrop;
            impl Drop for NeedsDrop {
                fn drop(&mut self) {
                    // placeholder...
                }
            }

            assert_that_type::<NeedsDrop>().needs_drop();
        }

        #[test]
        fn panics_when_type_does_not_need_drop() {
            struct DoeNotNeedDrop;

            assert_that_panic_by(|| {
                assert_that_type::<DoeNotNeedDrop>()
                    .with_location(false)
                    .needs_drop();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Type "assertr::assertions::std::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoeNotNeedDrop" was expected to need drop,

                    but dropping it is guaranteed to have no side effect.

                    You may forgot to `impl Drop` for this type.
                    -------- assertr --------
                "#});
        }
    }
}
