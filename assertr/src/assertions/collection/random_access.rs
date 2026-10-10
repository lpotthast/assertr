//! Indexed extraction for collections supporting constant-time random access.

use super::RandomAccess;
use crate::{
    AssertThat,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Checks constant-time indexed access and returns the borrowed element when present.
#[derive(Debug, Clone, Copy)]
pub struct HasElementAt(usize);
impl HasElementAt {
    /// Requires an element at this zero-based index.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
}

impl<C: RandomAccess + ?Sized, R> Expectation<C, R> for HasElementAt
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
        actual.element_at(self.0).ok_or_else(|| actual.length())
    }

    const KIND: FailureKind = FailureKind::Length;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a C, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("has an element at the index"),
            Some((actual, length)) => failure
                .actual(render.stable_collection(actual))
                .relation("has no element at the index")
                .fact(Fact::labelled("Actual length", render.value(&length))),
        };
        failure.expected(render.value(&self.0))
    }
}

/// Panic-mode indexed extraction from collections with [`RandomAccess`].
///
/// The method borrows the assertion chain and returns an assertion borrowing the selected element.
/// It is statically unavailable for collections without constant-time indexing, such as linked
/// lists, and for unordered collections such as sets.
///
/// ```compile_fail,E0599
/// use assertr::prelude::*;
/// use std::collections::LinkedList;
///
/// assert_that!(LinkedList::from([1, 2, 3])).at(1);
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RandomAccessExtractAssertions<'t, T, R = DebugRenderer> {
    /// Asserts that `index` is in bounds, then returns an assertion over that element.
    fn at(&'t self, index: usize) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>;
}

impl<'t, C, R> RandomAccessExtractAssertions<'t, C::Item, R> for AssertThat<'t, C, Panic, R>
where
    C: RandomAccess,
{
    #[track_caller]
    fn at(&'t self, index: usize) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone + ValueRenderer<usize>,
    {
        self.test_assertion(&HasElementAt::new(index));

        self.derive(|collection| {
            collection
                .element_at(index)
                .unwrap_or_else(|| unreachable!("validated index became unavailable"))
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            vec![1].must().at(0).be_equal_to(1);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => RandomAccessExtractAssertions<'static, i32, NoRenderer>
            );
        }

        #[test]
        fn index_and_length_use_the_active_renderer() {
            use crate::test_support::CustomValueRenderer;
            assert_that!(|| {
                assert_that!([7]).with_renderer(CustomValueRenderer).at(9);
            })
            .panics()
            .has_type::<String>()
            .contains("Expected: custom(9)")
            .contains("Actual length: custom(1)");
        }
    }

    mod at {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(vec![1, 2]), at(2));
        }

        #[test]
        fn returns_the_element_at_the_index() {
            assert_that!(vec![1, 2, 3]).at(1).is_equal_to(2);
        }

        #[test]
        fn panics_when_the_index_is_out_of_bounds() {
            assert_that!(|| {
                assert_that!(vec![1, 2]).with_location(false).at(2);
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

                    has no element at the index

                    Expected: 2

                    Details:
                      - Actual length: 2
                    -------- assertr --------
                "});
        }

        #[test]
        fn extraction_preserves_the_renderer_and_budget() {
            use crate::{renderer::RenderedBody, test_support::CustomValueRenderer};
            let subject = [7];
            let chain = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3));
            let failures = chain.at(0).capture(|it| it.is_equal_to(8));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.actual.as_ref().unwrap().body)
                        .is_equal_to(RenderedBody::Text {
                            text: "cus".into(),
                            omitted_characters: 6,
                        });
                    element.has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expected: cus... 6 more characters ...

                          Actual: cus... 6 more characters ...
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }
}
