//! Order-free extraction for any collection.

use super::Collection;
use crate::{
    AssertThat,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Requires that a collection contains exactly one element, returning its borrowed element.
///
/// The element of a one-element collection needs no order, so this works for sets and map views
/// too.
#[derive(Debug, Clone, Copy)]
pub struct HasSingle;

impl<C: Collection + ?Sized, R> Expectation<C, R> for HasSingle
where
    R: ValueRenderer<C::Item> + ValueRenderer<usize>,
{
    type Success<'a>
        = &'a C::Item
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = usize
    where
        Self: 'a,
        C: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a C,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let length = actual.length();
        if length != 1 {
            return Err(length);
        }
        Ok(actual
            .elements()
            .next()
            .unwrap_or_else(|| unreachable!("validated collection had no element")))
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure.relation("contains exactly one element"),
            Some((actual, rejection)) => failure
                .actual(render.collection(actual))
                .relation("does not contain exactly one element")
                .fact(Fact::labelled("Actual length", render.value(&rejection))),
        }
    }
}

/// Panic-mode element extraction from any [`Collection`].
///
/// The method borrows the assertion chain and returns an assertion borrowing the selected element.
/// A failed extraction cannot produce an element, so the family is intentionally unavailable in
/// capture mode. Positional extraction (`first`, `last`, `at`) requires
/// [`StableOrderExtractAssertions`](crate::assertions::StableOrderExtractAssertions) or
/// [`RandomAccessExtractAssertions`](crate::assertions::RandomAccessExtractAssertions).
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CollectionExtractAssertions<'t, T, R = DebugRenderer> {
    /// Asserts that the collection contains exactly one element, then returns an assertion over it.
    fn single(&'t self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>;
}

impl<'t, C, R> CollectionExtractAssertions<'t, C::Item, R> for AssertThat<'t, C, Panic, R>
where
    C: Collection,
{
    #[track_caller]
    fn single(&'t self) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone + ValueRenderer<usize>,
    {
        let element = self.require(&HasSingle);
        self.derive(|_| element)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            vec![1].must().single().be_equal_to(1);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{CustomValueRenderer, NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => CollectionExtractAssertions<'static, i32, NoRenderer>
            );
        }

        #[test]
        fn length_uses_the_active_renderer() {
            assert_that!(|| {
                assert_that!([1, 2])
                    .with_renderer(CustomValueRenderer)
                    .single();
            })
            .panics()
            .has_type::<String>()
            .contains("Actual length: custom(2)");
        }
    }

    mod single {
        use alloc::collections::{BTreeMap, BTreeSet};
        #[cfg(feature = "std")]
        use std::collections::HashSet;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2]), single());
        }

        #[test]
        fn returns_the_only_element() {
            assert_that!(vec![2]).single().is_equal_to(2);
        }

        #[test]
        fn returns_the_only_element_of_an_unordered_collection() {
            assert_that!(BTreeSet::from([2])).single().is_equal_to(2);
            assert_that!(BTreeMap::from([("key", 2)]))
                .values()
                .single()
                .is_equal_to(2);
        }

        #[cfg(feature = "std")]
        #[test]
        fn renders_an_unordered_collection_with_its_presentation() {
            assert_that!(|| {
                assert_that!(HashSet::from([3, 1, 2]))
                    .with_location(false)
                    .single();
            })
            .panics()
            .has_type::<String>()
            .contains("Actual: HashSet {\n    1,\n    2,\n    3,\n}");
        }

        #[test]
        fn panics_when_there_is_more_than_one_element() {
            assert_that!(|| {
                assert_that!(vec![1, 2]).with_location(false).single();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `vec![1, 2]`

                    Actual: [
                        1,
                        2,
                    ]

                    does not contain exactly one element

                    Details:
                      - Actual length: 2
                    -------- assertr --------
                "});
        }
    }
}
