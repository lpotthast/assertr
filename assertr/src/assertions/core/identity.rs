use crate::{
    AssertThat, AssertionContext, Expectation, Mode,
    failure::{FailureBuilder, FailureKind},
};

/// Defines an identity expectation comparing full pointers with [`core::ptr::eq`]. Diagnostics
/// render addresses through the identity adapter, never the targets' contents.
macro_rules! identity_expectation {
    (
        $(#[$attr:meta])*
        $name:ident: same = $same:literal, $relation:literal, $negated:literal, $operand:ident
    ) => {
        $(#[$attr])*
        pub struct $name<'e, T: ?Sized>(&'e T);

        impl<T: ?Sized> Clone for $name<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: ?Sized> Copy for $name<'_, T> {}

        /// Shows the expected target's address, because identity never inspects its contents.
        impl<T: ?Sized> core::fmt::Debug for $name<'_, T> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&core::ptr::from_ref(self.0))
                    .finish()
            }
        }

        impl<'e, T: ?Sized> $name<'e, T> {
            /// Borrows the reference whose full pointer is compared with the subject's pointer.
            #[must_use]
            pub const fn new(expected: &'e T) -> Self {
                Self(expected)
            }
        }

        impl<T: ?Sized, R> Expectation<T, R> for $name<'_, T> {
            type Success<'a>
                = ()
            where
                Self: 'a,
                T: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                T: 'a;

            fn evaluate(&self, actual: &T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
                if core::ptr::eq(actual, self.0) == $same {
                    Ok(())
                } else {
                    Err(())
                }
            }

            const KIND: FailureKind = FailureKind::Equality;

            fn explain(
                &self,
                rejected: Option<(&T, ())>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let identities = context.render().identities();
                failure
                    .relations(
                        rejected.map(|(actual, ())| identities.value(actual)),
                        $relation,
                        $negated,
                    )
                    .$operand(identities.value(self.0))
            }
        }
    };
}

identity_expectation!(
    /// Requires the subject and expected reference to have equal full pointers, without rendering
    /// their contents.
    IsSameInstanceAs: same = true, "is the same instance as", "is not the same instance as", expected
);

identity_expectation!(
    /// Checks that the subject is not the same instance as the expected reference, without
    /// comparing or rendering its contents.
    IsNotSameInstanceAs: same = false, "is not the same instance as", "is the same instance as", unexpected
);

/// Assertions comparing the address of the assertion subject with another reference.
///
/// Identity uses [`core::ptr::eq`] and requires no equality or rendering support. Diagnostics
/// display addresses, never the subject's contents. Distinct zero-sized values can share an
/// address, so pointer equality does not establish a unique allocation or logical object ID.
///
/// # Which reference is compared?
///
/// The subject is exactly the value returned by [`AssertThat::actual`]. The borrowing entry points
/// unwrap one reference level for sized targets: `assert_that!(value)` and `assert_that!(&value)`
/// both compare the address of `value`.
///
/// Owned references, reference-valued projections, and references to unsized targets retain their
/// reference type as the subject. For `AssertThat<&T>`, `expected` is `&&T` and the assertion
/// compares the storage of the reference itself. No additional dereferencing occurs. For
/// collections, the identity methods on
/// [`CollectionAssertions`](crate::assertions::collection::CollectionAssertions) and
/// [`StableOrderAssertions`](crate::assertions::collection::StableOrderAssertions) compare each
/// element's borrowed target instead.
///
/// ```
/// use assertr::prelude::*;
///
/// struct Key { _opaque: u8 }
/// let keys = [Key { _opaque: 1 }, Key { _opaque: 1 }];
/// let candidate = &keys[0];
/// assert_that!(candidate)
///     .is_same_instance_as(&keys[0])
///     .is_not_same_instance_as(&keys[1]);
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait IdentityAssertions<T> {
    /// Asserts that the subject and `expected` refer to the same instance.
    fn is_same_instance_as(self, expected: &T) -> Self;

    /// Asserts that the subject and `expected` refer to different instances.
    fn is_not_same_instance_as(self, expected: &T) -> Self;
}

impl<T, M: Mode, R> IdentityAssertions<T> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_same_instance_as(self, expected: &T) -> Self {
        self.apply_assertion(IsSameInstanceAs::new(expected))
    }

    #[track_caller]
    fn is_not_same_instance_as(self, expected: &T) -> Self {
        self.apply_assertion(IsNotSameInstanceAs::new(expected))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            let values = [1, 1];
            values[0].must().be_same_instance_as(&values[0]);
            values[0].must().not_be_same_instance_as(&values[1]);
        }
    }

    use crate::{
        FailureKind,
        prelude::*,
        renderer::RenderedBody,
        test_support::{NoRenderer, assert_trait_impl},
    };
    use indoc::formatdoc;

    struct Opaque {
        _byte: u8,
    }

    mod renderer_contract {
        use super::*;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Opaque, Panic, NoRenderer> => IdentityAssertions<Opaque>
            );
        }

        #[test]
        fn pointer_evidence_respects_the_leaf_budget_and_retains_type_information() {
            let values = [Opaque { _byte: 1 }, Opaque { _byte: 1 }];
            let failures = assert_that!(values[0])
                .with_renderer(NoRenderer)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(2))
                .capture(|it| it.is_same_instance_as(&values[1]));
            for (rendered, value) in [
                (failures[0].actual.as_ref().unwrap(), &values[0]),
                (failures[0].expected.as_ref().unwrap(), &values[1]),
            ] {
                assert_that!(rendered.type_name)
                    .is_equal_to(Some(core::any::type_name::<Opaque>()));
                assert_that!(rendered.body).is_equal_to(RenderedBody::Text {
                    text: "0x".into(),
                    omitted_characters: format!("{value:p}").len() - 2,
                });
            }
        }
    }

    mod is_same_instance_as {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let values = [Opaque { _byte: 1 }, Opaque { _byte: 1 }];
            let actual = &values[0];
            let expected = &values[1];
            assert_caller_location!(
                assert_that!(actual).with_renderer(NoRenderer),
                is_same_instance_as(expected)
            );
        }

        #[test]
        fn accepts_opaque_values_and_normalizes_borrowed_references() {
            let mut value = Opaque { _byte: 1 };
            assert_that!(value).is_same_instance_as(&value);
            assert_that!(&value).is_same_instance_as(&value);
            let reference = &mut value;
            assert_that!(reference).is_same_instance_as(reference);
            let candidates = [&value];
            let assertion = assert_that!(candidates[0])
                .with_renderer(NoRenderer)
                .is_same_instance_as(&value)
                .is_same_instance_as(&value);
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(2);
        }

        #[test]
        fn does_not_unwrap_reference_valued_subjects() {
            let values = [1, 1];
            let references = [&values[0], &values[0]];
            let assertion = assert_that!(references);
            assertion
                .get_first()
                .is_same_instance_as(&references[0])
                .is_not_same_instance_as(&references[1]);

            let text = "text";
            assert_that!(&text).is_same_instance_as(&text);
            assert_that_owned!(references[0]).is_not_same_instance_as(&references[0]);
        }

        #[test]
        fn captures_distinct_equal_values_and_preserves_context() {
            let values = [42, 42];
            let mut expected_line = 0;
            let failures = assert_that!(values[0])
                .with_renderer(NoRenderer)
                .with_subject_name("candidate")
                .with_detail_message("identity matters")
                .capture(|it| {
                    expected_line = line!() + 1;
                    let it = it.is_same_instance_as(&values[1]);
                    it.is_same_instance_as(&values[0])
                });
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| value.location.unwrap().line())
                        .is_equal_to(expected_line);
                    element
                        .derive(|value| &value.kind)
                        .is_equal_to(FailureKind::Equality);
                    element
                        .derive_owned(|value| value.subject_name.as_deref())
                        .is_equal_to(Some("candidate"));
                    element
                        .derive(|value| &value.expression)
                        .is_equal_to(Some("values[0]"));
                    element
                        .derive(|value| &value.messages)
                        .contains_exactly(["identity matters"]);
                    element
                        .derive_owned(|value| value.unexpected.is_none())
                        .is_true();
                },
            ]);
        }

        #[test]
        fn panics_with_pointer_evidence_without_rendering_the_values() {
            let values = [Opaque { _byte: 1 }, Opaque { _byte: 1 }];
            let actual = &values[0];
            let expected = &values[1];
            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_renderer(NoRenderer)
                    .with_location(false)
                    .is_same_instance_as(expected)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `actual`

                Actual: {actual:p}

                is not the same instance as

                Expected: {expected:p}
                -------- assertr --------
            "});
        }

        #[test]
        fn accepts_a_zero_sized_value_at_the_same_address() {
            let value = ();
            assert_that!(value)
                .with_renderer(NoRenderer)
                .is_same_instance_as(&value);
        }
    }

    mod is_not_same_instance_as {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let value = Opaque { _byte: 1 };
            assert_caller_location!(
                assert_that!(value).with_renderer(NoRenderer),
                is_not_same_instance_as(&value)
            );
        }

        #[test]
        fn accepts_distinct_opaque_instances_and_preserves_the_renderer() {
            let values = [Opaque { _byte: 1 }, Opaque { _byte: 1 }];
            let assertion: AssertThat<'_, Opaque, Panic, NoRenderer> = assert_that!(values[0])
                .with_renderer(NoRenderer)
                .is_not_same_instance_as(&values[1]);
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn captures_identity_with_unexpected_evidence() {
            let value = Opaque { _byte: 1 };
            let failures = assert_that!(value)
                .with_renderer(NoRenderer)
                .with_location(false)
                .capture(|it| it.is_not_same_instance_as(&value));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|item| item.expected.is_none())
                        .is_true();
                    element.derive(|item| item).has_text_report(formatdoc! {"
                -------- assertr --------
                Expression: `value`

                Actual: {value:p}

                is the same instance as

                Unexpected: {value:p}
                -------- assertr --------
            ", value = &value});
                },
            ]);
        }
    }
}
