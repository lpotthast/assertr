use crate::{AssertThat, Mode, Type};
use core::fmt::Write;
use indoc::writedoc;

/// Static memory assertions for any type.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait MemAssertions {
    /// Asserts that [`core::mem::needs_drop`] returns `true` for the represented type.
    ///
    /// This is a conservative signal. It does not guarantee that dropping the type runs code.
    fn needs_drop(self) -> Self;
}

impl<T, M: Mode, R> MemAssertions for AssertThat<'_, Type<T>, M, R> {
    #[track_caller]
    fn needs_drop(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.needs_drop() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Type {actual:#?} was expected to need drop,

                    but dropping it is guaranteed to have no side effect.

                    You may have forgotten to `impl Drop` for this type.
                ", actual = actual.get_type_name()}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::Type;
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Type<i32>, Panic, NoRenderer> => MemAssertions
            );
        }
    }

    mod needs_drop {
        use crate::assert_that_type;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            crate::Type::<String>::new().must().need_drop();
        }

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
                    Expression: `assertr::assertions::std::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoeNotNeed...`

                    Type "assertr::assertions::std::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoeNotNeedDrop" was expected to need drop,

                    but dropping it is guaranteed to have no side effect.

                    You may have forgotten to `impl Drop` for this type.
                    -------- assertr --------
                "#});
        }
    }
}
