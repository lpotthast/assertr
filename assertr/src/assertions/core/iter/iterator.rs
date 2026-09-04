use crate::{
    AssertThat, AssertrPartialEq, Mode, ValueRenderer, actual::Actual, assertions::iterator,
    mode::Capture,
};

/// Terminal assertions for an owned iterator.
///
/// These assertions drive the iterator itself and therefore need to own it: create the assertion
/// with `assert_that_owned!(...)` (or the fluent `.must_owned()`). To assert on a borrowed
/// collection, use the collection assertions or the `into_iter_*` assertions instead.
///
/// Every method consumes only as much of the iterator as is needed to decide the assertion,
/// then drops the unconsumed remainder and returns an assertion over `()`. Positive membership
/// assertions, prefix assertions, and contiguous-subsequence assertions therefore work with
/// potentially infinite iterators when a match is eventually produced. A positive match that
/// never occurs, a negative assertion whose forbidden match never occurs, or a non-empty suffix
/// assertion over a non-terminating iterator necessarily cannot terminate. Empty prefixes,
/// suffixes, and contiguous subsequences succeed without advancing the iterator.
///
/// Exact positional assertions read at most `expected.len() + 1` elements. Exact unordered
/// assertions buffer at most that many elements. Failure diagnostics retain only the last 16
/// consumed elements, regardless of how long the scan ran. Equality criteria accept comparable
/// expected element types, predicates receive `&T`, and `_satisfying` closures receive a
/// capture-mode assertion borrowing each candidate element.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait IteratorAssertions<'t, T, M: Mode, R> {
    /// Asserts that the iterator contains an element equal to `expected`.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that the iterator contains an element matching `predicate`.
    fn contains_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that the iterator contains an element satisfying `assertions`.
    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts that the iterator starts with elements equal to `expected`, in order.
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that the iterator's prefix matches `predicates` in order.
    fn starts_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that the iterator's prefix satisfies `assertions` in order.
    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts that the iterator ends with elements equal to `expected`, in order.
    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that the iterator's suffix matches `predicates` in order.
    fn ends_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that the iterator's suffix satisfies `assertions` in order.
    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts that the iterator contains `expected` as a contiguous subsequence.
    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that a contiguous subsequence matches `predicates` in order.
    fn contains_contiguous_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that a contiguous subsequence satisfies `assertions` in order.
    fn contains_contiguous_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts that no iterator element equals `not_expected`.
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that no iterator element matches `predicate`.
    fn does_not_contain_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that no iterator element satisfies `assertions`.
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts positional equality with `expected`, including length.
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts that each element matches the predicate at the same position, including length.
    fn contains_exactly_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts that each element satisfies the assertions at the same position, including length.
    fn contains_exactly_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;

    /// Asserts multiset equality with `expected`, ignoring order but preserving duplicate counts.
    fn contains_exactly_in_any_order<'u, E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u;

    /// Asserts one-to-one matching between elements and `predicates`, independent of order.
    fn contains_exactly_in_any_order_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u;

    /// Asserts one-to-one matching between elements and `assertions`, independent of order.
    fn contains_exactly_in_any_order_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u;
}

impl<'t, T, I, M: Mode, R> IteratorAssertions<'t, T, M, R> for AssertThat<'t, I, M, R>
where
    I: Iterator<Item = T>,
{
    #[track_caller]
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_contains::<_, T, _, _, _, _>(&this, iter, &expected);
        this
    }

    #[track_caller]
    fn contains_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_matching::<_, T, _, _, _, _>(&this, iter, &predicate);
        this
    }

    #[track_caller]
    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_satisfying::<_, T, _, _, _, _>(
            &this,
            iter,
            &assertions,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_starts_with::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn starts_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let predicates = predicates.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_starts_with_matching::<_, T, _, _, _, _>(&this, iter, predicates);
        this
    }

    #[track_caller]
    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_starts_with_satisfying::<_, T, _, _, _, _>(&this, iter, assertions);
        this
    }

    #[track_caller]
    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_ends_with::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn ends_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let predicates = predicates.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_ends_with_matching::<_, T, _, _, _, _>(&this, iter, predicates);
        this
    }

    #[track_caller]
    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_ends_with_satisfying::<_, T, _, _, _, _>(&this, iter, assertions);
        this
    }

    #[track_caller]
    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_contiguous::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_contiguous_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let predicates = predicates.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_contiguous_matching::<_, T, _, _, _, _>(&this, iter, predicates);
        this
    }

    #[track_caller]
    fn contains_contiguous_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_contiguous_satisfying::<_, T, _, _, _, _>(
            &this, iter, assertions,
        );
        this
    }

    #[track_caller]
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain::<_, T, _, _, _, _>(
            &this,
            iter,
            &not_expected,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn does_not_contain_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain_matching::<_, T, _, _, _, _>(
            &this,
            iter,
            &predicate,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain_satisfying::<_, T, _, _, _, _>(
            &this,
            iter,
            &assertions,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_exactly_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let predicates = predicates.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_matching::<_, T, _, _, _, _>(&this, iter, predicates);
        this
    }

    #[track_caller]
    fn contains_exactly_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_satisfying::<_, T, _, _, _, _>(&this, iter, assertions);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order<'u, E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: ValueRenderer<T> + ValueRenderer<E>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_in_any_order::<_, T, E, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: ValueRenderer<T>,
        't: 'u,
    {
        let predicates = predicates.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_in_any_order_matching::<_, T, _, _, _, _>(
            &this, iter, predicates,
        );
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_in_any_order_satisfying::<_, T, _, _, _, _>(
            &this,
            iter,
            assertions,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }
}

/// Takes the iterator out of `this`, returning it together with the terminal `()` assertion.
///
/// The assertion itself must run directly inside the calling `#[track_caller]` trait method,
/// not inside a closure passed to a helper, so failure locations point at the user's call site.
///
#[track_caller]
fn take_iterator<'t, 'u, T, I, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
) -> (I, AssertThat<'u, (), M, R>)
where
    I: Iterator<Item = T>,
    't: 'u,
{
    this.track_assertion();
    let (actual, terminal) = this.replace_actual_with(Actual::Owned(()));
    match actual {
        Actual::Owned(iterator) => (iterator, terminal),
        Actual::Borrowed(_) => panic!(
            "Iterator assertions consume the iterator and therefore need to own it. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
        ),
    }
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, core::ops::Range<i32>, Panic, NoRenderer>
                    => IteratorAssertions<'static, i32, Panic, NoRenderer>
            );
        }

        #[test]
        fn membership_and_sequence_equality_use_the_active_renderer_type() {
            assert_that_owned!(vec![RendererActual(1), RendererActual(2)].into_iter())
                .with_renderer(SentinelRenderer)
                .contains(RendererExpected(2));

            assert_that_owned!(vec![RendererActual(1), RendererActual(2)].into_iter())
                .with_renderer(SentinelRenderer)
                .starts_with([RendererExpected(1)]);
        }
    }

    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].into_iter().must_owned().contain(2);
        }

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that_owned!([1, 2, 3].into_iter()).contains(2);
        }

        #[test]
        fn succeeds_on_an_unbounded_iterator_when_a_match_occurs() {
            assert_that_owned!(0..).contains(3);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["foo".to_owned()].into_iter()).contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains(4);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain

                    Expected: 4

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_matching(|it: &i32| *it > 7);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain an element matching the predicate

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_satisfying(is_two);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that_owned!([1, 2, 3].into_iter()).contains_satisfying(is_two);
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2].into_iter())
                    .with_location(false)
                    .contains_satisfying(is_seven);
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the assertions")
            .contains(
                "Nested failures:\n  - At index 0:\n    Expected: 7\n\n      Actual: 1\n  - At index 1:\n    Expected: 7\n\n      Actual: 2\n",
            );
        }

        #[test]
        fn rendering_budget_limits_unsatisfied_element_children() {
            let failures = assert_that_owned!(0..20)
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| {
                    it.contains_satisfying(|element| {
                        element.is_equal_to(99);
                    })
                });

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].children[0].facts.as_slice())
                .contains(crate::Fact::new(crate::Fact::INDEX, "4"));
            assert_that!(
                failures[0]
                    .facts
                    .last()
                    .map(|fact| rendered_text(&fact.value))
            )
            .is_equal_to(Some("... 19 more unsatisfied elements ...".to_owned()));
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].into_iter().must_owned().not_contain(4);
        }

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that_owned!([1, 2, 3].into_iter()).does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["foo".to_owned()].into_iter()).does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    contains

                    Unexpected: 2

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_matching(|it: &i32| *it % 2 == 0);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    contains an element matching the predicate

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .not_contain_satisfying(is_seven);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that_owned!([1, 2, 3].into_iter()).does_not_contain_satisfying(is_seven);
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_satisfying(is_two);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    contains an element satisfying the assertions

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
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
            [1, 2, 3].into_iter().must_owned().start_with([1, 2]);
        }

        #[test]
        fn succeeds_when_prefix_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).starts_with([1, 2]);
        }

        #[test]
        fn succeeds_for_an_empty_prefix() {
            assert_that_owned!([1, 2, 3].into_iter()).starts_with::<i32>([]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter()).starts_with(["a"]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with([1, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not start with

                    Expected: [
                        1,
                        9,
                    ]

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
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
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .start_with_matching([is_one, is_two]);
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn succeeds_when_prefix_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).starts_with_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_matching([is_one, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not start with elements matching the predicates

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
                    Nested failures:
                      - At index 1:
                        Actual: 2

                        does not match its predicate
                    -------- assertr --------
                "});
        }
    }

    mod starts_with_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .start_with_satisfying([is_one]);
        }

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_prefix_satisfies() {
            assert_that_owned!([1, 2, 3].into_iter()).starts_with_satisfying([is_one]);
        }

        #[test]
        fn panics_when_prefix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("does not start with elements satisfying the assertions")
            .contains("Nested failures:\n  - At index 1:\n    Expected: 9\n\n      Actual: 2\n");
        }
    }

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].into_iter().must_owned().end_with([2, 3]);
        }

        #[test]
        fn succeeds_when_suffix_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).ends_with([2, 3]);
        }

        #[test]
        fn succeeds_for_an_empty_suffix() {
            assert_that_owned!([1, 2, 3].into_iter()).ends_with::<i32>([]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter()).ends_with(["b"]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with([2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not end with

                    Expected: [
                        2,
                        9,
                    ]

                    Details:
                      - Consumed elements: 3
                    Nested failures:
                      - At index 2:
                        Expected: 9

                          Actual: 3
                    -------- assertr --------
                "});
        }
    }

    mod ends_with_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .end_with_matching([is_two, is_three]);
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn succeeds_when_suffix_matches() {
            assert_that_owned!([1, 2, 3].into_iter()).ends_with_matching([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_matching([is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not end with elements matching the predicates

                    Details:
                      - Consumed elements: 3
                    Nested failures:
                      - At index 2:
                        Actual: 3

                        does not match its predicate
                    -------- assertr --------
                "});
        }
    }

    mod ends_with_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .end_with_satisfying([is_two, is_three]);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        fn is_zero(it: AssertThat<i32, Capture>) {
            it.is_equal_to(0);
        }

        #[test]
        fn succeeds_when_suffix_satisfies() {
            assert_that_owned!([1, 2, 3].into_iter()).ends_with_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not end with elements satisfying the assertions")
            .contains("Nested failures:\n  - At index 2:\n    Expected: 9\n\n      Actual: 3\n");
        }

        #[test]
        fn limits_repeated_suffix_evidence_to_the_rendering_budget() {
            let failures = assert_that_owned!([1, 2, 3].into_iter())
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| it.ends_with_satisfying([is_zero; 3]));

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].facts.as_slice())
                .contains(crate::Fact::note("... 2 more unsatisfied elements ..."));
        }
    }

    mod contains_contiguous {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_contiguous([2, 3]);
        }

        #[test]
        fn succeeds_when_a_contiguous_match_exists() {
            assert_that_owned!([1, 2, 3].into_iter()).contains_contiguous([2, 3]);
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that_owned!([1, 1, 2].into_iter()).contains_contiguous([1, 2]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_contiguous(["a", "b"]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous([2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain the contiguous subsequence

                    Expected: [
                        2,
                        9,
                    ]

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_contiguous_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_contiguous_matching([is_one, is_two]);
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn succeeds_when_a_contiguous_match_exists() {
            assert_that_owned!([1, 2, 3].into_iter())
                .contains_contiguous_matching([is_one, is_two]);
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that_owned!([1, 1, 2].into_iter())
                .contains_contiguous_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_matching([is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain a contiguous subsequence matching the predicates

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_contiguous_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_contiguous_satisfying([is_two, is_three]);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_a_contiguous_match_exists() {
            assert_that_owned!([1, 2, 3].into_iter())
                .contains_contiguous_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not contain a contiguous subsequence satisfying the assertions")
            .contains("Nested failures:\n  - At index 2:\n    Expected: 9\n\n      Actual: 3\n");
        }
    }

    mod contains_exactly {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_exactly([1, 2, 3]);
        }

        #[test]
        fn succeeds_when_elements_match_exactly() {
            assert_that_owned!([1, 2, 3].into_iter()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_exactly(["a", "b"]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 9, 3]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not contain exactly

                    Expected: [
                        1,
                        9,
                        3,
                    ]

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
                    Nested failures:
                      - At index 1:
                        Expected: 9

                          Actual: 2
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_without_consumption_when_a_known_length_differs() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 2]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: []

                    does not contain exactly

                    Expected: [
                        1,
                        2,
                    ]

                    Details:
                      - Consumed elements: 0
                      - Reported length: 3
                      - Expected length: 2
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
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_exactly_matching([is_one, is_two, is_three]);
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn succeeds_when_all_predicates_match_in_order() {
            assert_that_owned!([1, 2, 3].into_iter())
                .contains_exactly_matching([is_one, is_two, is_three]);
        }

        #[test]
        fn panics_when_an_element_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_matching([is_one, is_nine, is_three]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                    ]

                    does not exactly match the predicates

                    Details:
                      - Consumed elements: 2
                      - Decisive index: 1
                    Nested failures:
                      - At index 1:
                        Actual: 2

                        does not match its predicate
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2]
                .into_iter()
                .must_owned()
                .contain_exactly_satisfying([is_one, is_two]);
        }

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_all_assertions_are_satisfied_in_order() {
            assert_that_owned!([1, 2].into_iter()).contains_exactly_satisfying([is_one, is_two]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2].into_iter())
                    .with_location(false)
                    .contains_exactly_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("does not exactly satisfy the assertions")
            .contains("Nested failures:\n  - At index 1:\n    Expected: 9\n\n      Actual: 2\n");
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;
        use crate::{AssertrPartialEq, EqContext};
        use indoc::formatdoc;

        #[derive(Debug)]
        struct Actual(u8);

        #[derive(Debug)]
        struct Expected(u8);

        #[derive(Debug)]
        enum WildcardExpected {
            Any,
            Value(u8),
        }

        impl<R> AssertrPartialEq<Expected, R> for Actual {
            fn eq(&self, other: &Expected, _ctx: Option<&mut EqContext<'_, R>>) -> bool {
                self.0 == other.0
            }
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
            [2, 1, 1]
                .into_iter()
                .must_owned()
                .contain_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that_owned!([2, 1, 1].into_iter()).contains_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn supports_assertr_partial_eq_without_partial_eq() {
            assert_that_owned!([Actual(1), Actual(2)].into_iter())
                .contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn supports_non_equivalence_assertr_partial_eq() {
            assert_that_owned!([Actual(2), Actual(1)].into_iter())
                .contains_exactly_in_any_order([WildcardExpected::Any, WildcardExpected::Value(2)]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order([1, 2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly in any order

                    Expected: [
                        1,
                        2,
                        9,
                    ]

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2]
                .into_iter()
                .must_owned()
                .contain_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        fn is_at_most_two(value: &i32) -> bool {
            *value <= 2
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn succeeds_when_a_maximum_matching_exists_for_overlapping_predicates() {
            assert_that_owned!([1, 2].into_iter())
                .contains_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching([is_one, is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not exactly match the predicates in any order

                    Details:
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [-1, 1]
                .into_iter()
                .must_owned()
                .contain_exactly_in_any_order_satisfying([positive, negative]);
        }

        fn positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        fn negative(it: AssertThat<i32, Capture>) {
            it.is_less_than(0);
        }

        #[test]
        fn succeeds_when_a_maximum_matching_exists() {
            assert_that_owned!([-1, 1].into_iter())
                .contains_exactly_in_any_order_satisfying([positive, negative]);
        }

        #[test]
        fn panics_when_an_element_satisfies_no_assertion() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, -1, 2].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_satisfying([positive, positive, positive]);
            })
            .has_type::<String>()
            .contains("does not exactly satisfy the assertions in any order")
            .contains(
                "Nested failures:\n  - At index 1:\n    Actual: -1\n\n    is not greater than\n\n    Expected: 0\n",
            );
        }
    }
}
