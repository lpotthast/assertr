//! Indexed extraction for collections supporting constant-time random access.

use super::RandomAccess;
use crate::{AssertThat, Fact, ValueRenderer, failure::FailureKind, mode::Panic};

/// Panic-mode indexed extraction from collections with [`RandomAccess`].
///
/// The method borrows the assertion chain and returns an assertion borrowing the selected element.
/// It is statically unavailable for stable-order collections such as linked lists and for unordered
/// collections such as sets.
///
/// ```compile_fail,E0599
/// use assertr::prelude::*;
/// use std::collections::LinkedList;
///
/// assert_that!(LinkedList::from([1, 2, 3])).get_at(1);
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RandomAccessExtractAssertions<'t, T, R> {
    /// Asserts that `index` is in bounds, then returns an assertion over that element.
    fn get_at(&'t self, index: usize) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>;
}

impl<'t, C, R> RandomAccessExtractAssertions<'t, C::Item, R> for AssertThat<'t, C, Panic, R>
where
    C: RandomAccess,
{
    #[track_caller]
    fn get_at(&'t self, index: usize) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone + ValueRenderer<usize>,
    {
        self.track_assertion();
        if self.actual().element_at(index).is_none() {
            self.failure(FailureKind::Length)
                .actual(self.render().stable_collection(self.actual()))
                .relation("has no element at the index")
                .expected(self.render().value(&index))
                .fact(Fact::labelled(
                    "Actual length",
                    self.render().value(&self.actual().length()),
                ))
                .raise();
        }

        self.derive(|collection| {
            collection
                .element_at(index)
                .unwrap_or_else(|| unreachable!("validated index became unavailable"))
        })
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => RandomAccessExtractAssertions<'static, i32, NoRenderer>
            );
        }
    }

    mod get_at {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1].must().get_at(0).be_equal_to(1);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2]), get_at(2));
        }

        #[test]
        fn returns_the_element_at_the_index() {
            assert_that!(vec![1, 2, 3]).get_at(1).is_equal_to(2);
        }

        #[test]
        fn panics_when_the_index_is_out_of_bounds() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2]).with_location(false).get_at(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `vec![1, 2]`

                    Actual: [
                        1,
                        2,
                    ]

                    has no element at the index

                    Expected: 2

                    Details:
                      - Actual length: 2
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_with_rendered_index_and_length() {
            use crate::test_support::CustomValueRenderer;
            assert_that_panic_by(|| {
                assert_that!([7])
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .get_at(9);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[7]`

                Actual: [
                    custom(7),
                ]

                has no element at the index

                Expected: custom(9)

                Details:
                  - Actual length: custom(1)
                -------- assertr --------
            "});
        }

        #[test]
        fn extraction_preserves_the_renderer_and_budget() {
            use indoc::formatdoc;

            use crate::{renderer::RenderedBody, test_support::CustomValueRenderer};
            let subject = [7];
            let chain = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(3).build());
            let failures = chain.get_at(0).capture(|it| it.is_equal_to(8));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expected: cus... 6 more characters ...

                  Actual: cus... 6 more characters ...
                -------- assertr --------
            "});

            assert_eq!(
                failures[0].actual.as_ref().unwrap().body,
                RenderedBody::Text {
                    text: "cus".into(),
                    omitted_characters: 6
                }
            );
        }
    }
}
