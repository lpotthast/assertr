use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Fact, Mode, Type,
    failure::{FailureBuilder, FailureKind},
};

/// Checks the conservative [`core::mem::needs_drop`] property of a represented type.
pub struct NeedsDrop;

impl<T, R> Expectation<Type<T>, R> for NeedsDrop {
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;
    fn evaluate<'a>(&'a self, actual: &'a Type<T>, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.needs_drop() { Ok(()) } else { Err(()) }
    }
}

impl<T, R> ExpectationDiagnostics<Type<T>, R> for NeedsDrop {
    const KIND: FailureKind = FailureKind::Other;
    fn explain<Target>(
        &self,
        rejected: Option<(&Type<T>, ())>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("needs drop"),
            Some((actual, ())) => failure
                .actual(actual.get_type_name())
                .relation("does not need drop")
                .fact(Fact::note(
                    "Dropping a value of this type is guaranteed to have no side effect.",
                ))
                .fact(Fact::note(
                    "You may have forgotten to `impl Drop` for this type.",
                )),
        }
    }
}

/// Static memory assertions for any type.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MemAssertions {
    /// Asserts that [`core::mem::needs_drop`] returns `true` for the represented type.
    ///
    /// This is a conservative signal. It does not guarantee that dropping the type runs code.
    fn needs_drop(self) -> Self;
}

impl<T, M: Mode, R> MemAssertions for AssertThat<'_, Type<T>, M, R> {
    #[track_caller]
    fn needs_drop(self) -> Self {
        self.apply_assertion(NeedsDrop)
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

            assert_trait_impl!(super::super::NeedsDrop => crate::ExpectationDiagnostics<crate::Type<i32>, NoRenderer>);
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_type::<u32>(), needs_drop());
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
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `assertr::assertions::std::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoeNotNeed...`

                    Actual: assertr::assertions::std::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoeNotNeedDrop

                    does not need drop

                    Details:
                      - Dropping a value of this type is guaranteed to have no side effect.
                      - You may have forgotten to `impl Drop` for this type.
                    -------- assertr --------
                "});
        }
    }
}
