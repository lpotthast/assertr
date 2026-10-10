use crate::{
    AssertThat, Mode, Type,
    expectation::{AssertionContext, Expectation, passed},
    failure::{Fact, FailureBuilder, FailureKind},
};

/// Checks the conservative [`core::mem::needs_drop`] property of a represented type.
#[derive(Debug, Clone, Copy)]
pub struct NeedsDrop;

impl<T, R> Expectation<Type<T>, R> for NeedsDrop {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Type<T>: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Type<T>: 'a;
    fn evaluate<'a>(&'a self, actual: &'a Type<T>, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        passed(actual.needs_drop())
    }

    const KIND: FailureKind = FailureKind::Predicate;
    fn explain(
        &self,
        rejected: Option<(&Type<T>, ())>,
        failure: FailureBuilder,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("needs drop"),
            Some((actual, ())) => failure
                .actual(actual.type_name())
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
        self.matches(NeedsDrop)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            crate::Type::<String>::new().must().need_drop();
        }
    }

    mod renderer_contract {
        use crate::{
            Type,
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Type<i32>, Panic, NoRenderer> => MemAssertions
            );
            assert_trait_impl!(
                AssertThat<'static, Type<i32>, Capture, NoRenderer> => MemAssertions
            );
        }
    }

    mod needs_drop {
        use indoc::formatdoc;

        use crate::{assert_that_type, prelude::*};

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
        fn capture_continues_without_renderer_support() {
            use crate::test_support::NoRenderer;

            let failures = assert_that_type::<String>()
                .with_renderer(NoRenderer)
                .capture(MemAssertions::needs_drop);
            assert_that!(failures).is_empty();

            let failures = assert_that_type::<u32>()
                .with_renderer(NoRenderer)
                .capture(|it| {
                    let it = it.needs_drop().needs_drop();
                    assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                    it
                });
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_that!(failure.relation.as_deref()).is_equal_to(Some("does not need drop"));
                assert_that!(failure.kind).is_equal_to(crate::failure::FailureKind::Predicate);
            }
        }

        #[test]
        fn panics_when_type_does_not_need_drop() {
            struct DoesNotNeedDrop;

            assert_that!(|| {
                assert_that_type::<DoesNotNeedDrop>()
                    .with_location(false)
                    .needs_drop();
            }).panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `assertr::assertions::core::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoesNotNe...`

                    Actual: assertr::assertions::core::mem::tests::needs_drop::panics_when_type_does_not_need_drop::DoesNotNeedDrop

                    does not need drop

                    Details:
                      - Dropping a value of this type is guaranteed to have no side effect.
                      - You may have forgotten to `impl Drop` for this type.
                    -------- assertr --------
                "});
        }
    }
}
