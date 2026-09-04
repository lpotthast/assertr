//! Assertions and extraction for collections whose element order is meaningful.
//!
//! Order-free assertions live on [`CollectionAssertions`](super::CollectionAssertions). This
//! module's public extension traits require [`StableOrder`](StableOrder), so unordered subjects do
//! not implement a positional assertion family at all.

use super::{StableOrder, imp};
use crate::{
    AssertThat, AssertrPartialEq, Mode, ValueRenderer,
    failure::FailureKind,
    mode::{Capture, Panic},
};

/// Assertions over the elements of a collection whose order is stable and meaningful.
///
/// This trait is implemented only for [`StableOrder`] subjects. A call on a set reports
/// the missing capability and recommends `contains_exactly_in_any_order`.
///
/// The restriction is part of the trait implementation, not merely individual method bodies, so
/// a set cannot satisfy a generic `StableOrderAssertions` bound:
///
/// ```compile_fail,E0277
/// use std::collections::BTreeSet;
/// use assertr::prelude::*;
///
/// fn requires_stable_order<A: StableOrderAssertions<i32, DebugRenderer>>(_assertion: A) {}
///
/// requires_stable_order(assert_that!(BTreeSet::from([1, 2, 3])));
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait StableOrderAssertions<T, R> {
    /// Asserts that the collection starts with elements equal to `expected`, in order.
    fn starts_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that the collection's prefix matches `predicates` in order.
    fn starts_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that the collection's prefix satisfies `assertions` in order.
    fn starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;

    /// Asserts that the collection ends with elements equal to `expected`, in order.
    fn ends_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that the collection's suffix matches `predicates` in order.
    fn ends_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that the collection's suffix satisfies `assertions` in order.
    fn ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;

    /// Asserts that the collection contains `expected` as a contiguous subsequence.
    fn contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that a contiguous subsequence matches `predicates` in order.
    fn contains_contiguous_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>;

    /// Asserts that a contiguous subsequence satisfies `assertions` in order.
    fn contains_contiguous_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone;

    /// Asserts positional equality with `expected`, including length.
    ///
    /// `E` is the element type of the expected values, which only has to be comparable to `T`,
    /// not identical to it. The expected values are accepted as anything viewable as `&[E]`, so
    /// arrays, slices, and `Vec`s all work.
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>;

    /// Asserts that each element matches the predicate at the same position, including length.
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: ValueRenderer<T>,
        P: Fn(&T) -> bool;

    /// Asserts that each element satisfies the assertions at the same position, including length.
    ///
    /// On failure, each unsatisfied element's captured failures are reported.
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<T> + Clone,
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>);
}

impl<C, M, R> StableOrderAssertions<C::Item, R> for AssertThat<'_, C, M, R>
where
    C: StableOrder,
    M: Mode,
{
    #[track_caller]
    fn starts_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_starts_with(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn starts_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&C::Item) -> bool,
        R: ValueRenderer<C::Item>,
    {
        imp::assert_starts_with_matching(&self, predicates.as_ref());
        self
    }

    #[track_caller]
    fn starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
        R: ValueRenderer<C::Item> + Clone,
    {
        imp::assert_starts_with_satisfying(&self, assertions.as_ref());
        self
    }

    #[track_caller]
    fn ends_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_ends_with(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn ends_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&C::Item) -> bool,
        R: ValueRenderer<C::Item>,
    {
        imp::assert_ends_with_matching(&self, predicates.as_ref());
        self
    }

    #[track_caller]
    fn ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
        R: ValueRenderer<C::Item> + Clone,
    {
        imp::assert_ends_with_satisfying(&self, assertions.as_ref());
        self
    }

    #[track_caller]
    fn contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_contains_contiguous(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_contiguous_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&C::Item) -> bool,
        R: ValueRenderer<C::Item>,
    {
        imp::assert_contains_contiguous_matching(&self, predicates.as_ref());
        self
    }

    #[track_caller]
    fn contains_contiguous_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
        R: ValueRenderer<C::Item> + Clone,
    {
        imp::assert_contains_contiguous_satisfying(&self, assertions.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        C::Item: AssertrPartialEq<E, R>,
        R: ValueRenderer<C::Item> + ValueRenderer<E>,
    {
        imp::assert_contains_exactly(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_matching<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        R: ValueRenderer<C::Item>,
        P: Fn(&C::Item) -> bool,
    {
        imp::assert_contains_exactly_matching(&self, expected.as_ref());
        self
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        R: ValueRenderer<C::Item> + Clone,
        A: for<'a> Fn(AssertThat<'a, C::Item, Capture, R>),
    {
        imp::assert_contains_exactly_satisfying(&self, assertions.as_ref());
        self
    }
}

/// Panic-mode element extraction from collections with [`StableOrder`].
///
/// These methods borrow the assertion chain and return an assertion borrowing the selected
/// element. A failed extraction cannot produce an element, so the family is intentionally
/// unavailable in capture mode.
///
/// Unordered collections do not expose first or last elements:
///
/// ```compile_fail,E0599
/// use assertr::prelude::*;
/// use std::collections::BTreeSet;
///
/// assert_that!(BTreeSet::from([1, 2, 3])).get_first();
/// ```
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait StableOrderExtractAssertions<'t, T, R> {
    /// Asserts that the collection is non-empty, then returns an assertion over its first element.
    fn get_first(&'t self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone;

    /// Asserts that the collection is non-empty, then returns an assertion over its last element.
    fn get_last(&'t self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone;

    /// Asserts that the collection contains exactly one element, then returns an assertion over it.
    fn get_single(&'t self) -> AssertThat<'t, T, Panic, R>
    where
        R: ValueRenderer<T> + Clone;
}

impl<'t, C, R> StableOrderExtractAssertions<'t, C::Item, R> for AssertThat<'t, C, Panic, R>
where
    C: StableOrder,
{
    #[track_caller]
    fn get_first(&'t self) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone,
    {
        self.track_assertion();
        if self.actual().length() == 0 {
            self.failure(FailureKind::Length)
                .actual(self.render().stable_collection(self.actual()))
                .relation("has no first element")
                .raise();
        }

        self.derive(|collection| {
            collection
                .elements()
                .next()
                .unwrap_or_else(|| unreachable!("non-empty collection had no first element"))
        })
    }

    #[track_caller]
    fn get_last(&'t self) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone,
    {
        self.track_assertion();
        if self.actual().length() == 0 {
            self.failure(FailureKind::Length)
                .actual(self.render().stable_collection(self.actual()))
                .relation("has no last element")
                .raise();
        }

        self.derive(|collection| {
            collection
                .elements()
                .last()
                .unwrap_or_else(|| unreachable!("non-empty collection had no last element"))
        })
    }

    #[track_caller]
    fn get_single(&'t self) -> AssertThat<'t, C::Item, Panic, R>
    where
        R: ValueRenderer<C::Item> + Clone,
    {
        self.track_assertion();
        if self.actual().length() != 1 {
            self.failure(FailureKind::Length)
                .actual(self.render().stable_collection(self.actual()))
                .relation("does not contain exactly one element")
                .fact("Actual length", self.actual().length())
                .raise();
        }

        self.derive(|collection| {
            collection
                .elements()
                .next()
                .unwrap_or_else(|| unreachable!("single-element collection had no element"))
        })
    }
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    mod renderer_contract {
        use crate::assertions::{
            HasLength,
            collection::{Collection, StableOrder},
        };
        use crate::prelude::*;
        use crate::renderer::{CollectionPresentation, RenderingOrder};
        use crate::test_support::{
            NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
        };

        struct SortedPresentation(Vec<i32>);

        impl HasLength for SortedPresentation {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl Collection for SortedPresentation {
            type Item = i32;
            const PRESENTATION: CollectionPresentation =
                CollectionPresentation::list().with_order(RenderingOrder::SortByRenderedText);

            fn elements(&self) -> impl Iterator<Item = &i32> {
                self.0.iter()
            }
        }

        impl StableOrder for SortedPresentation {}

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => StableOrderAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Vec<i32>, Panic, NoRenderer>
                    => StableOrderExtractAssertions<'static, i32, NoRenderer>
            );
        }

        #[test]
        fn methods_use_the_active_renderer_type() {
            assert_that!([RendererActual(1), RendererActual(2)].as_slice())
                .with_renderer(SentinelRenderer)
                .starts_with([RendererExpected(1)])
                .ends_with([RendererExpected(2)])
                .contains_contiguous([RendererExpected(1), RendererExpected(2)])
                .contains_exactly([RendererExpected(1), RendererExpected(2)]);
        }

        #[test]
        fn positional_diagnostics_override_sorted_presentation() {
            assert_that_panic_by(|| {
                assert_that!(SortedPresentation(vec![3, 1, 2]))
                    .with_location(false)
                    .contains_exactly([3, 1, 9]);
            })
            .has_type::<String>()
            .contains("Actual: [\n    3,\n    1,\n    2,\n]")
            .does_not_contain("sorted for rendering");
        }
    }

    mod get_first {
        use alloc::{collections::LinkedList, vec::Vec};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1].must().get_first().be_equal_to(1);
        }

        #[test]
        fn returns_the_first_element_of_a_stable_order_collection() {
            assert_that!(LinkedList::from([1, 2, 3]))
                .get_first()
                .is_equal_to(1);
        }

        #[test]
        fn panics_for_an_empty_collection() {
            assert_that_panic_by(|| {
                assert_that!(Vec::<i32>::new())
                    .with_location(false)
                    .get_first();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `Vec::<i32>::new()`

                    Actual: []

                    has no first element
                    -------- assertr --------
                "});
        }
    }

    mod get_last {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1].must().get_last().be_equal_to(1);
        }

        #[test]
        fn returns_the_last_element() {
            assert_that!(vec![1, 2, 3]).get_last().is_equal_to(3);
        }
    }

    mod get_single {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1].must().get_single().be_equal_to(1);
        }

        #[test]
        fn returns_the_only_element() {
            assert_that!(vec![2]).get_single().is_equal_to(2);
        }

        #[test]
        fn panics_when_there_is_more_than_one_element() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2]).with_location(false).get_single();
            })
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

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].must().start_with([1, 2]);
        }

        #[test]
        fn succeeds_for_a_matching_prefix() {
            assert_that!([1, 2, 3]).starts_with([1, 2]);
        }

        #[test]
        fn reports_the_whole_collection_and_first_differing_position() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .starts_with([1, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[1, 2, 3]`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not start with

                    Expected: [
                        1,
                        9,
                    ]

                    Nested failures:
                      - At index 1:
                        Expected: 9

                          Actual: 2
                    -------- assertr --------
                "});
        }
    }

    mod starts_with_matching {
        use crate::prelude::*;

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].must().start_with_matching([is_one, is_two]);
        }

        #[test]
        fn succeeds_for_matching_prefix_predicates() {
            assert_that!([1, 2, 3]).starts_with_matching([is_one, is_two]);
        }

        #[test]
        fn reports_the_mismatching_position() {
            assert_that_panic_by(|| {
                assert_that!([1, 3])
                    .with_location(false)
                    .starts_with_matching([is_one, is_two]);
            })
            .has_type::<String>()
            .contains("does not start with elements matching the predicates")
            .contains("Nested failures:\n  - At index 1:\n    Actual: 3\n\n    does not match its predicate\n");
        }
    }

    mod starts_with_satisfying {
        use crate::prelude::*;

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].must().start_with_satisfying([is_one, is_two]);
        }

        #[test]
        fn succeeds_when_the_prefix_satisfies_the_assertions() {
            assert_that!([1, 2, 3]).starts_with_satisfying([is_one, is_two]);
        }

        #[test]
        fn reports_nested_failures() {
            assert_that_panic_by(|| {
                assert_that!([1, 3])
                    .with_location(false)
                    .starts_with_satisfying([is_one, is_two]);
            })
            .has_type::<String>()
            .contains("does not start with elements satisfying the assertions")
            .contains("Nested failures:\n  - At index 1:\n    Expected: 2\n\n      Actual: 3\n");
        }
    }

    mod ends_with {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].must().end_with([2, 3]);
        }

        #[test]
        fn succeeds_for_a_matching_suffix() {
            assert_that!([1, 2, 3]).ends_with([2, 3]);
        }

        #[test]
        fn reports_the_collection_position_that_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .ends_with([2, 9]);
            })
            .has_type::<String>()
            .contains("does not end with\n\nExpected: [\n    2,\n    9,\n]")
            .contains("Nested failures:\n  - At index 2:\n    Expected: 9\n\n      Actual: 3\n");
        }
    }

    mod ends_with_matching {
        use crate::prelude::*;

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].must().end_with_matching([is_two, is_three]);
        }

        #[test]
        fn succeeds_for_matching_suffix_predicates() {
            assert_that!([1, 2, 3]).ends_with_matching([is_two, is_three]);
        }

        #[test]
        fn reports_a_suffix_predicate_mismatch() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 4])
                    .with_location(false)
                    .ends_with_matching([is_two, is_three]);
            })
            .has_type::<String>()
            .contains("does not end with elements matching the predicates")
            .contains("Nested failures:\n  - At index 2:\n    Actual: 4\n\n    does not match its predicate\n");
        }
    }

    mod ends_with_satisfying {
        use crate::prelude::*;

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].must().end_with_satisfying([is_two, is_three]);
        }

        #[test]
        fn succeeds_when_the_suffix_satisfies_the_assertions() {
            assert_that!([1, 2, 3]).ends_with_satisfying([is_two, is_three]);
        }

        #[test]
        fn reports_nested_suffix_failures() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 4])
                    .with_location(false)
                    .ends_with_satisfying([is_two, is_three]);
            })
            .has_type::<String>()
            .contains("does not end with elements satisfying the assertions")
            .contains("Nested failures:\n  - At index 2:\n    Expected: 3\n\n      Actual: 4\n");
        }
    }

    mod contains_contiguous {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].must().contain_contiguous([2, 3]);
        }

        #[test]
        fn succeeds_for_a_contiguous_subsequence() {
            assert_that!([1, 1, 2]).contains_contiguous([1, 2]);
        }

        #[test]
        fn reports_the_whole_collection_when_not_found() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .contains_contiguous([1, 3]);
            })
            .has_type::<String>()
            .contains("Actual: [\n    1,\n    2,\n    3,\n]")
            .contains(
                "does not contain the contiguous subsequence\n\nExpected: [\n    1,\n    3,\n]",
            );
        }
    }

    mod contains_contiguous_matching {
        use crate::prelude::*;

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 3]
                .must()
                .contain_contiguous_matching([is_one, is_three]);
        }

        #[test]
        fn succeeds_for_contiguous_matching_elements() {
            assert_that!([0, 1, 3]).contains_contiguous_matching([is_one, is_three]);
        }

        #[test]
        fn reports_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3])
                    .with_location(false)
                    .contains_contiguous_matching([is_one, is_three]);
            })
            .has_type::<String>()
            .contains("does not contain a contiguous subsequence matching the predicates");
        }
    }

    mod contains_contiguous_satisfying {
        use crate::prelude::*;

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 3]
                .must()
                .contain_contiguous_satisfying([is_one, is_three]);
        }

        #[test]
        fn succeeds_for_contiguous_satisfying_elements() {
            assert_that!([0, 1, 3]).contains_contiguous_satisfying([is_one, is_three]);
        }

        #[test]
        fn reports_nested_failures_from_the_final_candidate() {
            assert_that_panic_by(|| {
                assert_that!([1, 2])
                    .with_location(false)
                    .contains_contiguous_satisfying([is_one, is_three]);
            })
            .has_type::<String>()
            .contains("does not contain a contiguous subsequence satisfying the assertions")
            .contains("Nested failures:\n  - At index 1:\n    Expected: 3\n\n      Actual: 2\n");
        }
    }

    mod contains_exactly {
        use crate::prelude::*;
        use crate::{AssertrPartialEq, EqContext};
        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        enum WildcardExpected {
            Any,
            Value(u8),
        }

        impl<R> AssertrPartialEq<WildcardExpected, R> for Actual {
            fn eq(&self, other: &WildcardExpected, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
                match other {
                    WildcardExpected::Any => true,
                    WildcardExpected::Value(expected) => self.0 == *expected,
                }
            }
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_exactly([1, 2, 3]);
        }

        #[test]
        fn succeeds_when_exact_match() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_different_type_combinations() {
            assert_that!(["foo".to_owned()].as_slice()).contains_exactly(["foo"]);
            assert_that!(["foo"].as_slice()).contains_exactly(["foo"]);
            assert_that!(["foo"].as_slice()).contains_exactly(["foo".to_owned()]);
            assert_that!(["foo"].as_slice()).contains_exactly(vec!["foo".to_owned()]);
            assert_that!(vec!["foo"].as_slice())
                .contains_exactly(vec!["foo".to_owned()].into_iter());
        }

        #[test]
        fn succeeds_when_exact_match_provided_as_slice() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn panics_when_not_exact_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly([2, 3, 4]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly

                    Expected: [
                        2,
                        3,
                        4,
                    ]

                    Details:
                      - Elements not expected: [
                            1,
                        ]
                      - Elements not found: [
                            4,
                        ]
                    -------- assertr --------
                "});
        }

        #[test]
        #[cfg(feature = "derive")]
        fn reports_assertr_eq_field_differences() {
            #[derive(Debug, AssertrEq)]
            struct Record {
                pub id: u32,
            }

            assert_that_panic_by(|| {
                assert_that!([Record { id: 1 }].as_slice())
                    .with_location(false)
                    .contains_exactly([RecordAssertrEq { id: eq(2) }]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `[Record {{ id: 1 }}].as_slice()`

                    Actual: [
                        Record {{
                            id: 1,
                        }},
                    ]

                    does not contain exactly

                    Expected: [
                        RecordAssertrEq {{
                            id: Eq::Eq(2),
                        }},
                    ]

                    Details:
                      - Differences: [
                            "id": expected 2, but was 1,
                        ]
                      - Elements not expected: [
                            Record {{
                                id: 1,
                            }},
                        ]
                      - Elements not found: [
                            RecordAssertrEq {{
                                id: Eq::Eq(2),
                            }},
                        ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_with_detail_message_when_only_differing_in_order() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly([3, 2, 1]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly

                    Expected: [
                        3,
                        2,
                        1,
                    ]

                    Details:
                      - Only the order of the elements differs.
                    -------- assertr --------
                "});
        }

        #[test]
        #[cfg(feature = "derive")]
        fn does_not_report_positional_field_differences_when_only_differing_in_order() {
            #[derive(Debug, AssertrEq)]
            struct Record {
                pub id: u32,
            }

            assert_that_panic_by(|| {
                assert_that!([Record { id: 1 }, Record { id: 2 }].as_slice())
                    .with_location(false)
                    .contains_exactly([
                        RecordAssertrEq { id: eq(2) },
                        RecordAssertrEq { id: eq(1) },
                    ]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[Record {{ id: 1 }}, Record {{ id: 2 }}].as_slice()`

                    Actual: [
                        Record {{
                            id: 1,
                        }},
                        Record {{
                            id: 2,
                        }},
                    ]

                    does not contain exactly

                    Expected: [
                        RecordAssertrEq {{
                            id: Eq::Eq(2),
                        }},
                        RecordAssertrEq {{
                            id: Eq::Eq(1),
                        }},
                    ]

                    Details:
                      - Only the order of the elements differs.
                    -------- assertr --------
                "});
        }

        #[test]
        fn recognizes_non_equivalence_matches_that_only_differ_in_order() {
            assert_that_panic_by(|| {
                assert_that!([Actual(2), Actual(1)].as_slice())
                    .with_location(false)
                    .contains_exactly([WildcardExpected::Any, WildcardExpected::Value(2)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[Actual(2), Actual(1)].as_slice()`

                    Actual: [
                        Actual(
                            2,
                        ),
                        Actual(
                            1,
                        ),
                    ]

                    does not contain exactly

                    Expected: [
                        Any,
                        Value(
                            2,
                        ),
                    ]

                    Details:
                      - Only the order of the elements differs.
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_with_missing_and_unexpected_elements_when_multiplicities_differ() {
            assert_that_panic_by(|| {
                assert_that!([1, 1, 2].as_slice())
                    .with_location(false)
                    .contains_exactly([1, 2, 2]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[1, 1, 2].as_slice()`

                    Actual: [
                        1,
                        1,
                        2,
                    ]

                    does not contain exactly

                    Expected: [
                        1,
                        2,
                        2,
                    ]

                    Details:
                      - Elements not expected: [
                            1,
                        ]
                      - Elements not found: [
                            2,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].as_slice().must().contain_exactly_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 2,
                |it: &i32| *it == 3,
            ]);
        }

        #[test]
        fn succeeds_when_each_element_matches_its_predicate() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_matching([
                move |it: &i32| *it == 1,
                move |it: &i32| *it < 3,
                move |it: &i32| *it > 2,
            ]);
        }

        #[test]
        fn panics_when_elements_only_match_in_a_different_order() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_matching([
                        move |it: &i32| *it == 1,
                        move |it: &i32| *it == 3,
                        move |it: &i32| *it == 2,
                    ]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not exactly match the predicates

                    Nested failures:
                      - At index 1:
                        Actual: 2

                        does not match its predicate
                      - At index 2:
                        Actual: 3

                        does not match its predicate
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_lengths_differ() {
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it == 1, |it| *it == 2];
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_matching(predicates);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].as_slice()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not exactly match the predicates

                    Details:
                      - Actual length: 3
                      - Expected length: 2
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_satisfying {
        use crate::prelude::*;

        fn is_zero(it: AssertThat<i32, Capture>) {
            it.is_equal_to(0);
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2].as_slice().must().contain_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
            ]);
        }

        #[test]
        fn succeeds_when_each_element_satisfies_its_assertions() {
            assert_that!([1, 2, 3].as_slice()).contains_exactly_satisfying([
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(2);
                },
                |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                },
            ]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy_its_positional_assertions() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].as_slice())
                    .with_location(false)
                    .contains_exactly_satisfying([
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(1);
                        },
                        |it: AssertThat<i32, Capture>| {
                            it.is_equal_to(3);
                        },
                    ]);
            })
            .has_type::<String>()
            .contains("does not exactly satisfy the assertions")
            .contains("Nested failures:\n  - At index 1:\n    Expected: 3\n\n      Actual: 2\n");
        }

        #[test]
        fn panics_when_lengths_differ() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].as_slice())
                    .with_location(false)
                    .contains_exactly_satisfying([|it: AssertThat<i32, Capture>| {
                        it.is_equal_to(1);
                    }]);
            })
            .has_type::<String>()
            .contains("Details:\n  - Actual length: 3\n  - Expected length: 1\n");
        }

        #[test]
        fn limits_repeated_element_evidence_to_the_rendering_budget() {
            let failures = assert_that!([1, 2, 3])
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| it.contains_exactly_satisfying([is_zero; 3]));

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].facts.as_slice())
                .contains_exactly([crate::Fact::note("... 2 more unsatisfied elements ...")]);
        }
    }
}
