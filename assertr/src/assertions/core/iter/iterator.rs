use crate::{AssertThat, Mode, ValueRenderer, actual::Actual, assertions::iterator, mode::Capture};

/// Terminal assertions for an owned iterator.
///
/// These assertions drive the iterator itself and therefore need to own it: create the assertion
/// with `assert_that_owned!(...)` (or the fluent `.must_owned()`). To assert on a borrowed
/// collection, use the collection assertions or the `into_iter_*` assertions instead.
///
/// Every method consumes only as much of the iterator as is needed to decide the assertion, then
/// renders any rejection while the iterator is still alive, drops the unconsumed remainder before
/// failure handling, and returns an assertion over `()`. Positive membership
/// assertions, prefix assertions, and contiguous-subsequence assertions therefore work with
/// potentially infinite iterators when a match is eventually produced. A positive match that never
/// occurs, a negative assertion whose forbidden match never occurs, or a non-empty suffix assertion
/// over a non-terminating iterator necessarily cannot terminate. Empty prefixes, suffixes, and
/// contiguous subsequences succeed without advancing the iterator.
///
/// Exact positional assertions read at most `expected.len() + 1` elements. Exact unordered
/// assertions buffer at most that many elements. Failure diagnostics retain only the last 16
/// consumed elements, regardless of how long the scan ran. Equality criteria accept comparable
/// expected element types, matchers evaluate `&T`, and `_satisfying` closures receive a
/// capture-mode assertion borrowing each candidate element.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait IteratorAssertions<'t, T, M: Mode, R> {
    /// Asserts that the iterator contains an element equal to `expected`.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator contains an element matching `expected`.
    fn contains_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::ExpectationDiagnostics<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator contains an element satisfying `assertions`.
    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator starts with elements equal to `expected`, in order.
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator's prefix matches the expected matcher list in order.
    fn starts_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator's prefix satisfies `assertions` in order.
    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator ends with elements equal to `expected`, in order.
    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator's suffix matches the expected matcher list in order.
    fn ends_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator's suffix satisfies `assertions` in order.
    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator contains `expected` as a contiguous subsequence.
    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that a contiguous subsequence matches the expected matcher list in order.
    fn contains_contiguous_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that a contiguous subsequence satisfies `assertions` in order.
    fn contains_contiguous_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that no iterator element equals `not_expected`.
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that no iterator element matches `expected`.
    fn does_not_contain_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::ExpectationDiagnostics<T, R>,
        't: 'u,
        R: ValueRenderer<usize> + ValueRenderer<T>;

    /// Asserts that no iterator element satisfies `assertions`.
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts positional equality with `expected`, including length.
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that each element matches the constraint at the same position, including length.
    fn contains_exactly_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that each element satisfies the assertions at the same position, including length.
    fn contains_exactly_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;

    /// Asserts multiset equality with `expected`, ignoring order but preserving duplicate counts.
    fn contains_exactly_in_any_order<'u, E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts one-to-one matching between elements and the expected matcher list, independent of
    /// order.
    fn contains_exactly_in_any_order_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts one-to-one matching between elements and `assertions`, independent of order.
    fn contains_exactly_in_any_order_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u;
}

impl<'t, T, I, M: Mode, R> IteratorAssertions<'t, T, M, R> for AssertThat<'t, I, M, R>
where
    I: Iterator<Item = T>,
{
    #[track_caller]
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_contains::<_, T, _, _, _, _>(&this, iter, &expected);
        this
    }

    #[track_caller]
    fn contains_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::ExpectationDiagnostics<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::membership::<_, T, _, _, _, _>(
            &this,
            iter,
            &expected,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.contains_matching(crate::expectation::satisfying(assertions))
    }

    #[track_caller]
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::assert_starts_with::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn starts_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::exact_or_prefix::<_, T, _, _, _, _>(&this, iter, &expected, false);
        this
    }

    #[track_caller]
    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = assertions
            .as_ref()
            .iter()
            .map(crate::expectation::satisfying)
            .collect::<alloc::vec::Vec<_>>();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::matchers::exact_or_prefix::<_, T, _, _, _, _>(&this, iter, &expected, false);
        this
    }

    #[track_caller]
    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::assert_ends_with::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn ends_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::suffix_or_contiguous::<_, T, _, _, _, _>(&this, iter, &expected, true);
        this
    }

    #[track_caller]
    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = assertions
            .as_ref()
            .iter()
            .map(crate::expectation::satisfying)
            .collect::<alloc::vec::Vec<_>>();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::matchers::suffix_or_contiguous::<_, T, _, _, _, _>(&this, iter, &expected, true);
        this
    }

    #[track_caller]
    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::assert_contains_contiguous::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_contiguous_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::suffix_or_contiguous::<_, T, _, _, _, _>(&this, iter, &expected, false);
        this
    }

    #[track_caller]
    fn contains_contiguous_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = assertions
            .as_ref()
            .iter()
            .map(crate::expectation::satisfying)
            .collect::<alloc::vec::Vec<_>>();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::matchers::suffix_or_contiguous::<_, T, _, _, _, _>(&this, iter, &expected, false);
        this
    }

    #[track_caller]
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
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
    fn does_not_contain_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::ExpectationDiagnostics<T, R>,
        't: 'u,
        R: ValueRenderer<usize> + ValueRenderer<T>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::no_membership::<_, T, _, _, _, _>(
            &this,
            iter,
            &expected,
            iterator::PositionReporting::YieldOrder,
        );
        this
    }

    #[track_caller]
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.does_not_contain_matching(crate::expectation::satisfying(assertions))
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::assert_contains_exactly::<_, T, _, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_exactly_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::exact_or_prefix::<_, T, _, _, _, _>(&this, iter, &expected, true);
        this
    }

    #[track_caller]
    fn contains_exactly_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = assertions
            .as_ref()
            .iter()
            .map(crate::expectation::satisfying)
            .collect::<alloc::vec::Vec<_>>();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::matchers::exact_or_prefix::<_, T, _, _, _, _>(&this, iter, &expected, true);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order<'u, E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E>,
        R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::assert_contains_exactly_in_any_order::<_, T, E, _, _, _>(&this, iter, expected);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: crate::expectation::MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (iter, this) = take_iterator(self);
        iterator::matchers::unordered::<_, T, _, _, _, _>(&this, iter, &expected);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.track_assertion();
        let expected = assertions
            .as_ref()
            .iter()
            .map(crate::expectation::satisfying)
            .collect::<alloc::vec::Vec<_>>();
        let (iter, this) = take_iterator_after_tracking(self);
        iterator::matchers::unordered::<_, T, _, _, _, _>(&this, iter, &expected);
        this
    }
}

/// Takes the iterator out of `this`, returning it together with the terminal `()` assertion.
///
/// The assertion itself must run directly inside the calling `#[track_caller]` trait method, not
/// inside a closure passed to a helper, so failure locations point at the user's call site.
#[track_caller]
fn take_iterator<'t, 'u, T, I, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
) -> (I, AssertThat<'u, (), M, R>)
where
    I: Iterator<Item = T>,
    't: 'u,
    R: ValueRenderer<usize>,
{
    this.track_assertion();
    take_iterator_after_tracking(this)
}

#[track_caller]
fn take_iterator_after_tracking<'t, 'u, T, I, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
) -> (I, AssertThat<'u, (), M, R>)
where
    I: Iterator<Item = T>,
    't: 'u,
{
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
        use crate::{
            prelude::*,
            test_support::{
                NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
            },
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!([1, 2, 3].into_iter()), contains(4));
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

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_fact};
            use indoc::formatdoc;

            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains(9));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                Actual: [
                    custom(1),
                    custom(2),
                ]

                does not contain

                Expected: custom(9)

                Details:
                  - Consumed elements: custom(2)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Consumed elements", 2);
                },
            ]);
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
                .contain_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 0));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_matching(crate::expectation::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that_owned!([1, 2, 3].into_iter())
                .contains_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 0));
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_matching(crate::expectation::predicate(|it: &i32| *it > 7));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not contain a matching element

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - At [0]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - At [1]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                  - At [2]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn stops_consuming_after_the_first_match() {
            let calls = core::cell::Cell::new(0);
            let iterator = (0..).inspect(|_| calls.set(calls.get() + 1));

            assert_that_owned!(iterator)
                .contains_matching(crate::assertions::core::partial_eq::equal_to(3));

            assert_that!(calls.get()).is_equal_to(4);
        }

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::{
                assertions::core::partial_eq::equal_to,
                test_support::{CustomValueRenderer, assert_custom_fact},
            };
            use indoc::formatdoc;
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_matching(equal_to(9)));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                does not contain a matching element

                Details:
                  - Consumed: custom(2)
                  - Preview starts at: custom(0)
                Nested failures:
                  - At [0]:
                    Expected: custom(9)

                      Actual: custom(1)
                  - At [1]:
                    Expected: custom(9)

                      Actual: custom(2)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Consumed", 2);
                    assert_custom_fact(element.actual(), "Preview starts at", 0);
                },
            ]);
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2].into_iter()),
                contains_satisfying(is_seven)
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                does not contain a matching element

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [0]:
                    Expected: 7

                      Actual: 1
                  - At [1]:
                    Expected: 7

                      Actual: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn rendering_budget_limits_unsatisfied_element_children() {
            let failures = assert_that_owned!(0..20)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| {
                    it.contains_satisfying(|element| {
                        element.is_equal_to(99);
                    })
                });

            assert_that!(failures[0].children.as_slice()).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| &value.path)
                        .is_equal_to([crate::failure::PathSegment::Index(4)]);
                },
            ]);
            assert_that!(failures[0].omitted_children).is_equal_to(19);
        }

        #[test]
        fn opaque_callback_failures_preserve_custom_rendering_and_budgets() {
            use crate::test_support::{CustomValueRenderer, assert_custom_value};

            struct Opaque(usize);

            fn check(it: AssertThat<'_, Opaque, Capture, CustomValueRenderer>) {
                it.satisfies(
                    |value| &value.0,
                    |value| {
                        value.is_equal_to(9);
                    },
                );
            }

            let values = [Opaque(1), Opaque(2)];
            let failures = assert_that_owned!(values.into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .capture(|it| it.contains_satisfying(check));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].omitted_children).is_equal_to(1);
            let child = &failures[0].children[0];
            assert_custom_value(child.actual.as_ref().unwrap(), &1_usize);
            assert_custom_value(child.expected.as_ref().unwrap(), &9_usize);
            assert_that!(child.path).contains_exactly([crate::failure::PathSegment::Index(0)]);
            crate::test_support::assert_custom_fact(&failures[0], "Consumed", 2);
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                does_not_contain(2)
            );
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
                .not_contain_matching(crate::expectation::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                does_not_contain_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 0))
            );
        }

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that_owned!([1, 2, 3].into_iter())
                .does_not_contain_matching(crate::expectation::predicate(|it: &i32| *it > 7));
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_matching(crate::expectation::predicate(|it: &i32| {
                        *it % 2 == 0
                    }));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                contains an unexpected matching element

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    Actual: 2

                    matches the unwanted constraint

                    Constraint:
                        satisfies the predicate
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                does_not_contain_satisfying(is_two)
            );
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
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                contains an unexpected matching element

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    Actual: 2

                    matches the unwanted constraint

                    Constraint:
                        satisfies the assertions
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                starts_with([1, 9])
            );
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

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_fact};
            use indoc::formatdoc;
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.starts_with([1, 2, 3]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                Actual: []

                does not start with

                Expected: [
                    custom(1),
                    custom(2),
                    custom(3),
                ]

                Details:
                  - Consumed elements: custom(0)
                  - Reported length: custom(2)
                  - Prefix length: custom(3)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Prefix length", 3);
                },
            ]);
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
                .start_with_matching(crate::expectation::predicate_list([is_one, is_two]));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                starts_with_matching(crate::expectation::predicate_list([is_one, is_nine]))
            );
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
            assert_that_owned!([1, 2, 3].into_iter())
                .starts_with_matching(crate::expectation::predicate_list([is_one, is_two]));
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_matching(crate::expectation::predicate_list([is_one, is_nine]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match the required position

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn empty_prefix_does_not_consume_the_iterator() {
            let calls = core::cell::Cell::new(0);
            assert_that_owned!((0..).inspect(|_| calls.set(calls.get() + 1)))
                .starts_with_matching(matchers![]);

            assert_that!(calls.get()).is_equal_to(0);
        }

        #[test]
        fn short_iterators_report_first_missing_matcher_and_length() {
            use crate::assertions::core::partial_eq::equal_to;

            let iterators: [Box<dyn Iterator<Item = i32>>; 2] = [
                Box::new(core::iter::from_fn(|| None)),
                Box::new([1].into_iter().filter(|_| true)),
            ];
            for (consumed, iterator) in iterators.into_iter().enumerate() {
                let expected = if consumed == 0 { 1 } else { 987_654 };
                let failures = assert_that_owned!(iterator)
                    .with_location(false)
                    .capture(|it| it.starts_with_matching([equal_to(1), equal_to(987_654)]));

                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure.has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `iterator`

                    is missing a matching position

                    Details:
                      - Consumed: {consumed}
                      - Preview starts at: 0
                      - Expected length: 2
                    Nested failures:
                      - At [{consumed}]:
                        does not satisfy the constraint

                        Constraint:
                            is equal to

                            Expected: {expected}
                    -------- assertr --------
                    "});
                    },
                ]);
            }
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                starts_with_satisfying([is_one, is_nine])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match the required position

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    Expected: 9

                      Actual: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn missing_position_describes_assertions_without_running_them() {
            let calls = core::cell::Cell::new(0);
            let assertions = [1, 987_654].map(|expected| {
                let calls = &calls;
                move |it: AssertThat<i32, Capture>| {
                    calls.set(calls.get() + 1);
                    it.is_equal_to(expected);
                }
            });
            let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                .capture(|it| it.starts_with_satisfying(assertions));

            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(AssertionFailure::children)
                        .contains_exactly_satisfying([
                            |child: AssertThat<AssertionFailure, Capture>| {
                                child
                                    .derive(|child| &child.path)
                                    .is_equal_to([crate::failure::PathSegment::Index(1)]);
                                child
                                    .derive_owned(AssertionFailure::constraint)
                                    .is_some_satisfying(|constraint| {
                                        constraint
                                            .derive_owned(|constraint| constraint.relation())
                                            .is_equal_to(Some("satisfies the assertions"));
                                    });
                            },
                        ]);
                },
            ]);
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!([1, 2, 3].into_iter()), ends_with([2, 9]));
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

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_fact};
            use indoc::formatdoc;
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.ends_with([1, 2, 3]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                Actual: [
                    custom(1),
                    custom(2),
                ]

                does not end with

                Expected: [
                    custom(1),
                    custom(2),
                    custom(3),
                ]

                Details:
                  - Consumed elements: custom(2)
                  - Suffix length: custom(3)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Suffix length", 3);
                },
            ]);
        }
    }

    mod ends_with_matching {
        use crate::{assertions::core::partial_eq::equal_to, prelude::*};
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .end_with_matching(crate::expectation::predicate_list([is_two, is_three]));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                ends_with_matching(crate::expectation::predicate_list([is_two, is_nine]))
            );
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
            assert_that_owned!([1, 2, 3].into_iter())
                .ends_with_matching(crate::expectation::predicate_list([is_two, is_three]));
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_matching(crate::expectation::predicate_list([is_two, is_nine]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not end with matching positions

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - At [2]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn short_iterators_report_matcher_descriptions_and_length() {
            for values in [&[][..], &[1][..]] {
                let consumed = values.len();
                let failures = assert_that_owned!(values.iter().copied())
                    .with_location(false)
                    .capture(|it| it.ends_with_matching([equal_to(9), equal_to(10)]));

                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure.has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `values.iter().copied()`

                    does not end with matching positions

                    Constraint:
                        ends with these positions

                        Nested failures:
                          - is equal to

                            Expected: 9
                          - is equal to

                            Expected: 10

                    Details:
                      - Consumed: {consumed}
                      - Preview starts at: 0
                      - Expected length: 2
                    -------- assertr --------
                    "});
                    },
                ]);
            }
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                ends_with_satisfying([is_two, is_nine])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not end with matching positions

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - At [2]:
                    Expected: 9

                      Actual: 3
                -------- assertr --------
            "});
        }

        #[test]
        fn limits_repeated_suffix_evidence_to_the_rendering_budget() {
            let failures = assert_that_owned!([1, 2, 3].into_iter())
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.ends_with_satisfying([is_zero; 3]));

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].omitted_children).is_equal_to(2);
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous([2, 9])
            );
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
        use crate::{assertions::core::partial_eq::equal_to, prelude::*};
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3]
                .into_iter()
                .must_owned()
                .contain_contiguous_matching(crate::expectation::predicate_list([is_one, is_two]));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous_matching(crate::expectation::predicate_list(
                    [is_two, is_nine,]
                ))
            );
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
                .contains_contiguous_matching(crate::expectation::predicate_list([is_one, is_two]));
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that_owned!([1, 1, 2].into_iter())
                .contains_contiguous_matching(crate::expectation::predicate_list([is_one, is_two]));
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_matching(crate::expectation::predicate_list([
                        is_two, is_nine,
                    ]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not contain matching contiguous positions

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - At [2]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn stops_after_finding_a_window_in_an_infinite_iterator() {
            assert_that_owned!(0..).contains_contiguous_matching(matchers![
                crate::matchers::eq(5),
                crate::matchers::eq(6)
            ]);
        }

        #[test]
        fn short_iterators_report_matcher_descriptions_and_length() {
            for values in [&[][..], &[1][..]] {
                let consumed = values.len();
                let failures = assert_that_owned!(values.iter().copied())
                    .with_location(false)
                    .capture(|it| it.contains_contiguous_matching([equal_to(9), equal_to(10)]));

                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure.has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `values.iter().copied()`

                    does not contain matching contiguous positions

                    Constraint:
                        contains these contiguous positions

                        Nested failures:
                          - is equal to

                            Expected: 9
                          - is equal to

                            Expected: 10

                    Details:
                      - Consumed: {consumed}
                      - Preview starts at: 0
                      - Expected length: 2
                    -------- assertr --------
                    "});
                    },
                ]);
            }
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous_satisfying([is_two, is_nine])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not contain matching contiguous positions

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - At [2]:
                    Expected: 9

                      Actual: 3
                -------- assertr --------
            "});
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly([1, 2])
            );
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

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_fact};
            use indoc::formatdoc;
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly([1]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                Actual: []

                does not contain exactly

                Expected: [
                    custom(1),
                ]

                Details:
                  - Consumed elements: custom(0)
                  - Reported length: custom(2)
                  - Expected length: custom(1)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Reported length", 2);
                    assert_custom_fact(element.actual(), "Expected length", 1);
                },
            ]);
        }
    }

    mod contains_exactly_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            [1, 2, 3].into_iter().must_owned().contain_exactly_matching(
                crate::expectation::predicate_list([is_one, is_two, is_three]),
            );
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_matching(crate::expectation::predicate_list([
                    is_one, is_nine, is_three,
                ]))
            );
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
            assert_that_owned!([1, 2, 3].into_iter()).contains_exactly_matching(
                crate::expectation::predicate_list([is_one, is_two, is_three]),
            );
        }

        #[test]
        fn panics_when_an_element_does_not_match() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_matching(crate::expectation::predicate_list([
                        is_one, is_nine, is_three,
                    ]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match the required position

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn consumes_at_most_one_more_than_the_expected_length() {
            let calls = core::cell::Cell::new(0);
            let iterator = (0..).inspect(|_| calls.set(calls.get() + 1));
            let failures = assert_that_owned!(iterator).capture(|it| {
                it.contains_exactly_matching(matchers![
                    crate::matchers::eq(0),
                    crate::matchers::eq(1)
                ])
            });

            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(3);
        }

        #[test]
        fn short_iterators_report_first_missing_matcher_and_length() {
            use crate::assertions::core::partial_eq::equal_to;

            let iterators: [Box<dyn Iterator<Item = i32>>; 2] = [
                Box::new(core::iter::from_fn(|| None)),
                Box::new([1].into_iter().filter(|_| true)),
            ];
            for (consumed, iterator) in iterators.into_iter().enumerate() {
                let expected = if consumed == 0 { 1 } else { 987_654 };
                let failures = assert_that_owned!(iterator)
                    .with_location(false)
                    .capture(|it| it.contains_exactly_matching([equal_to(1), equal_to(987_654)]));

                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure.has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `iterator`

                    is missing a matching position

                    Details:
                      - Consumed: {consumed}
                      - Preview starts at: 0
                      - Expected length: 2
                    Nested failures:
                      - At [{consumed}]:
                        does not satisfy the constraint

                        Constraint:
                            is equal to

                            Expected: {expected}
                    -------- assertr --------
                    "});
                    },
                ]);
            }
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2].into_iter()),
                contains_exactly_satisfying([is_one, is_nine])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                does not match the required position

                Details:
                  - Consumed: 2
                  - Preview starts at: 0
                Nested failures:
                  - At [1]:
                    Expected: 9

                      Actual: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn missing_position_describes_assertions_without_running_them() {
            let calls = core::cell::Cell::new(0);
            let assertions = [1, 987_654].map(|expected| {
                let calls = &calls;
                move |it: AssertThat<i32, Capture>| {
                    calls.set(calls.get() + 1);
                    it.is_equal_to(expected);
                }
            });
            let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                .capture(|it| it.contains_exactly_satisfying(assertions));

            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(AssertionFailure::children)
                        .contains_exactly_satisfying([
                            |child: AssertThat<AssertionFailure, Capture>| {
                                child
                                    .derive(|child| &child.path)
                                    .is_equal_to([crate::failure::PathSegment::Index(1)]);
                                child
                                    .derive_owned(AssertionFailure::constraint)
                                    .is_some_satisfying(|constraint| {
                                        constraint
                                            .derive_owned(|constraint| constraint.relation())
                                            .is_equal_to(Some("satisfies the assertions"));
                                    });
                            },
                        ]);
                },
            ]);
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;

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

        impl PartialEq<Expected> for Actual {
            fn eq(&self, other: &Expected) -> bool {
                self.0 == other.0
            }
        }

        impl PartialEq<WildcardExpected> for Actual {
            fn eq(&self, other: &WildcardExpected) -> bool {
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_in_any_order([1, 2, 9])
            );
        }

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that_owned!([2, 1, 1].into_iter()).contains_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn supports_heterogeneous_partial_eq() {
            assert_that_owned!([Actual(1), Actual(2)].into_iter())
                .contains_exactly_in_any_order([Expected(2), Expected(1)]);
        }

        #[test]
        fn supports_non_equivalence_partial_eq() {
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

        #[test]
        fn renders_numeric_evidence_with_the_active_renderer() {
            use crate::test_support::{CustomValueRenderer, assert_custom_fact};
            use indoc::formatdoc;
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly_in_any_order([1]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2].into_iter()`

                Actual: []

                does not contain exactly in any order

                Expected: [
                    custom(1),
                ]

                Details:
                  - Consumed elements: custom(0)
                  - Reported length: custom(2)
                  - Expected length: custom(1)
                -------- assertr --------
            "});

                    assert_custom_fact(element.actual(), "Expected length", 1);
                },
            ]);
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
                .contain_exactly_in_any_order_matching(crate::expectation::predicate_list([
                    is_at_most_two,
                    is_one,
                ]));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_in_any_order_matching(crate::expectation::predicate_list([
                    is_one, is_two, is_nine,
                ]))
            );
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
            assert_that_owned!([1, 2].into_iter()).contains_exactly_in_any_order_matching(
                crate::expectation::predicate_list([is_at_most_two, is_one]),
            );
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(crate::expectation::predicate_list([
                        is_one, is_two, is_nine,
                    ]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match exactly in any order

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - at slot: 2
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                      - does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, -1, 2].into_iter()),
                contains_exactly_in_any_order_satisfying([positive, positive, positive])
            );
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `[1, -1, 2].into_iter()`

                does not match exactly in any order

                Details:
                  - Consumed: 3
                  - Preview starts at: 0
                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the assertions

                    Details:
                      - at slot: 2
                    Nested failures:
                      - Actual: -1

                        is not greater than

                        Expected: 0
                  - has unexpected elements

                    Details:
                      - unexpected count: 1
                    Nested failures:
                      - Actual: -1

                        is not greater than

                        Expected: 0
                      - Actual: -1

                        is not greater than

                        Expected: 0
                -------- assertr --------
            "});
        }
    }
}
