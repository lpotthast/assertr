//! Indexed extraction for collections supporting constant-time random access.

use alloc::string::String;
use core::fmt::Write;

use indoc::writedoc;

use super::RandomAccess;
use crate::{AssertThat, ValueRenderer, mode::Panic};

/// Panic-mode indexed extraction from collections with [`RandomAccess`].
///
/// The method borrows the assertion chain and returns an assertion borrowing the selected
/// element. It is statically unavailable for stable-order collections such as linked lists and
/// for unordered collections such as sets.
///
/// ```compile_fail,E0599
/// use assertr::prelude::*;
/// use std::collections::LinkedList;
///
/// assert_that!(LinkedList::from([1, 2, 3])).get_at(1);
/// ```
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait RandomAccessExtractAssertions<'t, T, R> {
    /// Asserts that `index` is in bounds, then returns an assertion over that element.
    fn get_at(&'t self, index: usize) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone;
}

impl<'t, C, R> RandomAccessExtractAssertions<'t, C::Item, R> for AssertThat<'t, C, Panic, R>
where
    C: RandomAccess,
{
    #[track_caller]
    fn get_at(&'t self, index: usize) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone,
    {
        self.track_assertion();
        if self.actual().element_at(index).is_none() {
            let actual = self.render().stable_collection(self.actual());
            let length = self.actual().length();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    has no element at index {index}. Its length is {length}.
                "}
            });
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

                    has no element at index 2. Its length is 2.
                    -------- assertr --------
                "});
        }
    }
}
