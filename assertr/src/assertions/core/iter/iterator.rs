use crate::{
    AssertThat, Mode,
    assertions::{
        collection::Placement,
        iterator::{
            ContainsAllScan, ContainsMatchingScan, ContainsScan, CountScan,
            DoesNotContainMatchingScan, DoesNotContainScan, ElementsEqualScan, ElementsMatchScan,
            IsExhaustedScan, IsNotExhaustedScan, Scan, UnorderedEqualScan, UnorderedMatchScan, run,
        },
    },
    borrow_for::{BorrowFor, borrow_for},
    expectation::{Expectation, MatcherList, lists::SatisfyingList},
    matchers::satisfying,
    mode::Capture,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Terminal assertions for an owned iterator.
///
/// Expected values use [`BorrowFor`] for the declared item type, including reference layers.
/// Use [`crate::matchers::dereferenced`] to match an item's pointee:
///
/// ```
/// use assertr::{prelude::*, matchers::{dereferenced, eq}};
/// let expected = String::from("hello");
/// assert_that_owned!([String::from("hello")].into_iter()).contains(&expected);
/// assert_that_owned!([&expected].into_iter()).contains(&expected);
/// assert_that_owned!([&expected].into_iter())
///     .contains_matching(dereferenced(eq("hello")));
/// ```
///
/// ```compile_fail
/// use assertr::prelude::*;
/// let expected = String::from("hello");
/// assert_that_owned!([&expected].into_iter()).contains(String::from("hello"));
/// ```
///
/// These assertions drive the iterator itself and therefore need to own it: create the assertion
/// with `assert_that_owned!(...)` (or the fluent `.must_owned()`). To assert on a borrowed
/// collection, use the collection assertions, or pass a borrowing iterator such as
/// `assert_that_owned!(values.iter())`.
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
/// assertions buffer at most that many elements. An exact [`Iterator::size_hint`] can reject a
/// length mismatch before consuming anything, but never establishes success on its own.
///
/// Equality diagnostics preview at most the last 16 consumed elements, regardless of how long the
/// scan ran. Unordered equality instead reports the buffered elements like
/// [`CollectionAssertions::contains_exactly_in_any_order`](crate::assertions::CollectionAssertions::contains_exactly_in_any_order).
/// Matcher and `_satisfying` diagnostics retain evidence from the first rejected candidates that
/// fit the [rendering budget](crate::renderer::RenderingBudget). Later rejections are only counted.
/// Matchers evaluate `&T`, and `_satisfying` closures receive a capture-mode assertion borrowing
/// each candidate element.
///
/// Bulk value lists use [repeatable expected data](crate#expected-lists).
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait IteratorAssertions<'t, T, M: Mode, R = DebugRenderer> {
    /// Asserts that the iterator contains an element equal to `expected`.
    fn contains<E>(self, expected: E) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that the iterator contains an element equal to each expected value.
    ///
    /// Extra elements are allowed and duplicate expectations may share one element, like
    /// [`CollectionAssertions::contains_all`](crate::assertions::CollectionAssertions::contains_all).
    /// The scan stops as soon as every expected value has been seen, so it can succeed on an
    /// infinite iterator. An empty expected list succeeds without advancing the iterator.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that_owned!((0..).map(|n| n * n)).contains_all([49, 4, 4]);
    /// ```
    fn contains_all<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that the iterator contains an element matching `expected`.
    fn contains_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator contains an element satisfying `assertions`.
    fn contains_satisfying<A>(self, assertions: A) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the iterator starts with elements equal to `expected`, in order.
    fn starts_with<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that the iterator's prefix matches the expected matcher list in order.
    fn starts_with_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator's prefix satisfies `assertions` in order.
    fn starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the iterator ends with elements equal to `expected`, in order.
    fn ends_with<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that the iterator's suffix matches the expected matcher list in order.
    fn ends_with_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts that the iterator's suffix satisfies `assertions` in order.
    fn ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the iterator contains `expected` as a contiguous subsequence.
    fn contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that a contiguous subsequence matches the expected matcher list in order.
    ///
    /// A rejection retains one group per rejected window within the rendering budget. Each group
    /// carries a `Window start` fact with the window's zero-based starting index.
    fn contains_contiguous_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts that a contiguous subsequence satisfies `assertions` in order.
    ///
    /// Rejections are grouped per window like
    /// [`contains_contiguous_matching`](Self::contains_contiguous_matching).
    fn contains_contiguous_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that no iterator element equals `not_expected`.
    fn does_not_contain<E>(self, not_expected: E) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that no iterator element matches the unwanted constraint `not_expected`.
    fn does_not_contain_matching<P>(self, not_expected: P) -> AssertThat<'t, (), M, R>
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize> + ValueRenderer<T>;

    /// Asserts that no iterator element satisfies `assertions`.
    fn does_not_contain_satisfying<A>(self, assertions: A) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>;

    /// Asserts positional equality with `expected`, including length.
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts that each element matches the constraint at the same position, including length.
    fn contains_exactly_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts that each element satisfies the assertions at the same position, including length.
    fn contains_exactly_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts multiset equality with `expected`, ignoring order but preserving duplicate counts.
    fn contains_exactly_in_any_order<E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>;

    /// Asserts one-to-one matching between elements and the expected matcher list, independent of
    /// order.
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>;

    /// Asserts one-to-one matching between elements and `assertions`, independent of order.
    fn contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>;

    /// Asserts that the iterator is exhausted: it yields no element.
    ///
    /// Consumes at most one element. A collection with a known length uses
    /// [`LengthAssertions::is_empty`](crate::assertions::LengthAssertions::is_empty) instead.
    fn is_exhausted(self) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T> + ValueRenderer<usize>;

    /// Asserts that the iterator is not exhausted: it yields at least one element.
    ///
    /// Consumes at most one element.
    fn is_not_exhausted(self) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T>;

    /// Asserts that the iterator yields exactly `expected` elements, like [`Iterator::count`].
    ///
    /// Reads at most `expected + 1` elements, so it also fails on an infinite iterator. An exact
    /// [`Iterator::size_hint`] can reject a mismatch before consuming anything. A collection with
    /// a known length uses
    /// [`LengthAssertions::has_length`](crate::assertions::LengthAssertions::has_length) instead.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let values = [1, 2, 3, 4];
    /// assert_that_owned!(values.iter().filter(|it| *it % 2 == 0)).has_count(2);
    /// ```
    fn has_count(self, expected: usize) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T> + ValueRenderer<usize>;
}

impl<'t, T, I, M: Mode, R> IteratorAssertions<'t, T, M, R> for AssertThat<'t, I, M, R>
where
    I: Iterator<Item = T>,
{
    #[track_caller]
    fn contains<E>(self, expected: E) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            ContainsScan::<T, _>::new(borrow_for::<T, _>(&expected))
        })
    }

    #[track_caller]
    fn contains_all<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || ContainsAllScan::<T, _>::new(expected.as_ref()))
    }

    #[track_caller]
    fn contains_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || ContainsMatchingScan::<T, _>::new(expected))
    }

    #[track_caller]
    fn contains_satisfying<A>(self, assertions: A) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        self.contains_matching(satisfying(assertions))
    }

    #[track_caller]
    fn starts_with<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsEqualScan::<T, _>::new(expected.as_ref(), Placement::Prefix)
        })
    }

    #[track_caller]
    fn starts_with_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(expected, Placement::Prefix)
        })
    }

    #[track_caller]
    fn starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(
                SatisfyingList::new(assertions.as_ref()),
                Placement::Prefix,
            )
        })
    }

    #[track_caller]
    fn ends_with<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsEqualScan::<T, _>::new(expected.as_ref(), Placement::Suffix)
        })
    }

    #[track_caller]
    fn ends_with_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(expected, Placement::Suffix)
        })
    }

    #[track_caller]
    fn ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(
                SatisfyingList::new(assertions.as_ref()),
                Placement::Suffix,
            )
        })
    }

    #[track_caller]
    fn contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsEqualScan::<T, _>::new(expected.as_ref(), Placement::Contiguous)
        })
    }

    #[track_caller]
    fn contains_contiguous_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(expected, Placement::Contiguous)
        })
    }

    #[track_caller]
    fn contains_contiguous_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(
                SatisfyingList::new(assertions.as_ref()),
                Placement::Contiguous,
            )
        })
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            DoesNotContainScan::<T, _>::new(borrow_for::<T, _>(&not_expected))
        })
    }

    #[track_caller]
    fn does_not_contain_matching<P>(self, not_expected: P) -> AssertThat<'t, (), M, R>
    where
        P: Expectation<T, R>,
        R: ValueRenderer<usize> + ValueRenderer<T>,
    {
        consume(self, || {
            DoesNotContainMatchingScan::<T, _>::new(not_expected)
        })
    }

    #[track_caller]
    fn does_not_contain_satisfying<A>(self, assertions: A) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
    {
        self.does_not_contain_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsEqualScan::<T, _>::new(expected.as_ref(), Placement::Exact)
        })
    }

    #[track_caller]
    fn contains_exactly_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(expected, Placement::Exact)
        })
    }

    #[track_caller]
    fn contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        consume(self, || {
            ElementsMatchScan::<T, _>::new(
                SatisfyingList::new(assertions.as_ref()),
                Placement::Exact,
            )
        })
    }

    #[track_caller]
    fn contains_exactly_in_any_order<E>(self, expected: impl AsRef<[E]>) -> AssertThat<'t, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    {
        consume(self, || UnorderedEqualScan::<T, _>::new(expected.as_ref()))
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<P>(self, expected: P) -> AssertThat<'t, (), M, R>
    where
        P: MatcherList<T, R>,
        R: ValueRenderer<usize>,
    {
        consume(self, || UnorderedMatchScan::<T, _>::new(expected))
    }

    #[track_caller]
    fn contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'t, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
    {
        consume(self, || {
            UnorderedMatchScan::<T, _>::new(SatisfyingList::new(assertions.as_ref()))
        })
    }

    #[track_caller]
    fn is_exhausted(self) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        consume(self, IsExhaustedScan::<T>::new)
    }

    #[track_caller]
    fn is_not_exhausted(self) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T>,
    {
        consume(self, IsNotExhaustedScan::<T>::new)
    }

    #[track_caller]
    fn has_count(self, expected: usize) -> AssertThat<'t, (), M, R>
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        consume(self, || CountScan::<T>::new(expected))
    }
}

/// Takes the iterator out of `this`, then tracks the assertion and lets `scan` create the scan.
///
/// `scan` runs after tracking, so expected operands and lists are accessed only then. The
/// iterator is released before failure handling, and the terminal `()` assertion is returned.
#[track_caller]
fn consume<I, D, M: Mode, R>(
    this: AssertThat<'_, I, M, R>,
    scan: impl FnOnce() -> D,
) -> AssertThat<'_, (), M, R>
where
    I: Iterator,
    D: Scan<I, R>,
{
    let (actual, this) = this.take_owned(
        "Iterator assertions consume the iterator and therefore need to own it. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
    );
    run(&this, || (actual, scan()));
    this
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::{matchers::eq, prelude::*};

        fn is(expected: i32) -> impl Fn(AssertThat<'_, i32, Capture>) {
            move |it| {
                it.is_equal_to(expected);
            }
        }

        #[test]
        fn are_as_expected() {
            let values = || [1, 2, 3].into_iter().must_owned();
            values().contain(2);
            values().contain_all([3, 1]);
            values().contain_matching(eq(2));
            values().contain_satisfying(is(2));
            values().not_contain(4);
            values().not_contain_matching(eq(4));
            values().not_contain_satisfying(is(4));
            values().start_with([1, 2]);
            values().start_with_matching([eq(1), eq(2)]);
            values().start_with_satisfying([is(1)]);
            values().end_with([2, 3]);
            values().end_with_matching([eq(2), eq(3)]);
            values().end_with_satisfying([is(2), is(3)]);
            values().contain_contiguous([2, 3]);
            values().contain_contiguous_matching([eq(1), eq(2)]);
            values().contain_contiguous_satisfying([is(2), is(3)]);
            values().contain_exactly([1, 2, 3]);
            values().contain_exactly_matching([eq(1), eq(2), eq(3)]);
            values().contain_exactly_satisfying([is(1), is(2), is(3)]);
            values().contain_exactly_in_any_order([3, 1, 2]);
            values().contain_exactly_in_any_order_matching([eq(3), eq(1), eq(2)]);
            values().contain_exactly_in_any_order_satisfying([is(3), is(1), is(2)]);
            core::iter::empty::<i32>().must_owned().be_exhausted();
            values().not_be_exhausted();
            values().have_count(3);
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{
                ComparisonRenderer, NoRenderer, RendererActual, RendererExpected, assert_trait_impl,
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
        fn numeric_evidence_uses_the_active_renderer() {
            use crate::{
                matchers::eq,
                test_support::{CustomValueRenderer, assert_custom_fact},
            };

            // Iterators with and without an exact size hint.
            let hinted = || [1, 2].into_iter();
            let unhinted = || [1, 2].into_iter().filter(|_| true);
            macro_rules! case {
                ($iterator:expr, $call:ident($($arg:expr),*), $label:literal, $value:expr) => {{
                    let failures = assert_that_owned!($iterator)
                        .with_renderer(CustomValueRenderer)
                        .capture(|it| it.$call($($arg),*));
                    assert_custom_fact(&failures[0], $label, $value);
                }};
            }
            case!(hinted(), contains(9), "Consumed elements", 2);
            case!(hinted(), contains_matching(eq(9)), "Consumed elements", 2);
            case!(hinted(), starts_with([1, 2, 3]), "Reported length", 2);
            case!(hinted(), contains_exactly([1, 2, 3]), "Expected length", 3);
            case!(unhinted(), contains_exactly([1, 9]), "Decisive index", 1);
            case!(
                unhinted(),
                contains_exactly([1]),
                "Extra element at index",
                1
            );
            case!(hinted(), ends_with([1, 2, 3]), "Suffix length", 3);
            case!(
                hinted(),
                contains_exactly_in_any_order([1]),
                "Expected length",
                1
            );
            let failures = assert_that_owned!(hinted())
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.contains_contiguous_matching([eq(1), eq(9)]));
            assert_custom_fact(&failures[0].children[0], "Window start", 0);
        }

        #[test]
        fn membership_and_sequence_equality_use_the_active_renderer_type() {
            assert_that_owned!(vec![RendererActual(1), RendererActual(2)].into_iter())
                .with_renderer(ComparisonRenderer)
                .contains(RendererExpected::new(2));

            assert_that_owned!(vec![RendererActual(1), RendererActual(2)].into_iter())
                .with_renderer(ComparisonRenderer)
                .starts_with([RendererExpected::new(1)]);
        }
    }

    mod contains {
        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["foo".to_owned()].into_iter()).contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains(4);
            })
            .panics()
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

    mod contains_all {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_all([2, 4])
            );
        }

        #[test]
        fn succeeds_when_every_expected_value_occurs() {
            assert_that_owned!([1, 2, 3].into_iter()).contains_all([3, 1, 1]);
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_all(["b", "a"]);
        }

        #[test]
        fn stops_on_an_infinite_iterator_once_every_value_occurred() {
            let mut iterator = 0..;
            assert_that_owned!(&mut iterator).contains_all([3, 1]);
            assert_that!(iterator.next()).is_equal_to(Some(4));
        }

        #[test]
        fn panics_when_any_expected_value_is_absent() {
            assert_that!(|| {
                assert_that_owned!(vec![1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_all([2, 4]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `vec![1, 2, 3].into_iter()`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain all of

                    Expected: [
                        2,
                        4,
                    ]

                    Details:
                      - Elements not found: [
                            4,
                        ]
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }
    }

    mod is_exhausted {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!([1].into_iter()), is_exhausted());
        }

        #[test]
        fn succeeds_without_elements() {
            assert_that_owned!([1, 2].iter().filter(|it| **it > 2)).is_exhausted();
        }

        #[test]
        fn panics_with_the_first_element() {
            assert_that!(|| {
                assert_that_owned!([1, 2].into_iter())
                    .with_location(false)
                    .is_exhausted();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2].into_iter()`

                    Actual: [
                        1,
                    ]

                    is not exhausted

                    Details:
                      - Consumed elements: 1
                    -------- assertr --------
                "});
        }
    }

    mod is_not_exhausted {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!(core::iter::empty::<i32>()),
                is_not_exhausted()
            );
        }

        #[test]
        fn succeeds_on_an_infinite_iterator() {
            assert_that_owned!(0..).is_not_exhausted();
        }

        #[test]
        fn panics_without_elements() {
            assert_that!(|| {
                assert_that_owned!(core::iter::empty::<i32>())
                    .with_location(false)
                    .is_not_exhausted();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `core::iter::empty::<i32>()`

                    Actual: []

                    is unexpectedly exhausted
                    -------- assertr --------
                "});
        }
    }

    mod has_count {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!([1, 2, 3].into_iter()), has_count(2));
        }

        #[test]
        fn succeeds_when_the_count_matches() {
            assert_that_owned!([1, 2, 3].iter().filter(|it| **it > 1)).has_count(2);
        }

        #[test]
        fn fails_on_an_infinite_iterator_after_one_extra_element() {
            let mut iterator = 0..;
            let failures = assert_that_owned!(&mut iterator).capture(|it| it.has_count(2));
            assert_that!(failures).has_length(1);
            assert_that!(iterator.next()).is_equal_to(Some(3));
        }

        #[test]
        fn panics_with_the_counted_elements() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].iter().filter(|it| **it > 0))
                    .with_location(false)
                    .has_count(2);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].iter().filter(|it| **it > 0)`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not have the expected count

                    Expected: 2

                    Details:
                      - Minimum actual count: 3
                    -------- assertr --------
                "});
        }

        #[test]
        fn rejects_an_exact_size_hint_without_consuming() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .has_count(2);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    does not have the expected count

                    Expected: 2

                    Details:
                      - Reported length: 3
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_matching(matchers::predicate(|it: &i32| *it > 7))
            );
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_matching(matchers::predicate(|it: &i32| *it > 7));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not contain a matching element

                Details:
                  - Consumed elements: 3
                Nested failures:
                  - At [0]:
                    Actual: 1

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate

                  - At [1]:
                    Actual: 2

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate

                  - At [2]:
                    Actual: 3

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2].into_iter()),
                contains_satisfying(is_seven)
            );
        }

        #[test]
        fn borrows_non_clone_items_and_returns_a_terminal_capture_chain() {
            struct Item(i32);
            let calls = core::cell::Cell::new(0);

            let failures = assert_that_owned!([Item(1), Item(2), Item(3)].into_iter()).capture(
                |it| -> AssertThat<'_, (), Capture> {
                    it.contains_satisfying(|item| {
                        calls.set(calls.get() + 1);
                        item.derive(|item| &item.0).is_equal_to(2);
                    })
                },
            );

            assert_that!(calls.get()).is_equal_to(2);
            assert_that!(failures).is_empty();
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }
    }

    mod does_not_contain {
        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["foo".to_owned()].into_iter()).does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain(2);
            })
            .panics()
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
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                does_not_contain_matching(matchers::predicate(|it: &i32| *it % 2 == 0))
            );
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_matching(matchers::predicate(|it: &i32| *it % 2 == 0));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                contains an unexpected matching element

                Details:
                  - Consumed elements: 2
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
    }

    mod starts_with {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                starts_with([1, 9])
            );
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter()).starts_with(["a"]);
        }
    }

    mod starts_with_matching {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                starts_with_matching([is_one, is_nine].map(matchers::predicate))
            );
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_matching([is_one, is_nine].map(matchers::predicate));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match the required position

                Details:
                  - Consumed elements: 2
                Nested failures:
                  - At [1]:
                    Actual: 2

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod starts_with_satisfying {
        use crate::prelude::*;

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
    }

    mod ends_with {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that_owned!([1, 2, 3].into_iter()), ends_with([2, 9]));
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter()).ends_with(["b"]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with([2, 9]);
            })
            .panics()
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
                      - At [2]:
                        Expected: 9

                          Actual: 3
                    -------- assertr --------
                "});
        }
    }

    mod ends_with_matching {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                ends_with_matching([is_two, is_nine].map(matchers::predicate))
            );
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_matching([is_two, is_nine].map(matchers::predicate));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not end with matching elements

                Details:
                  - Consumed elements: 3
                Nested failures:
                  - At [2]:
                    Actual: 3

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod ends_with_satisfying {
        use crate::prelude::*;

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

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }
    }

    mod contains_contiguous {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous([2, 9])
            );
        }

        #[test]
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_contiguous(["a", "b"]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous([2, 9]);
            })
            .panics()
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
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous_matching([is_two, is_nine,].map(matchers::predicate))
            );
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_matching([is_two, is_nine].map(matchers::predicate));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not contain these elements contiguously

                Details:
                  - Consumed elements: 3
                Nested failures:
                  - does not match in this window

                    Details:
                      - Window start: 0
                    Nested failures:
                      - At [0]:
                        Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - At [1]:
                        Actual: 2

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                  - does not match in this window

                    Details:
                      - Window start: 1
                    Nested failures:
                      - At [2]:
                        Actual: 3

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn stops_after_finding_a_window_in_an_infinite_iterator() {
            assert_that_owned!(0..)
                .contains_contiguous_matching(matchers![matchers::eq(5), matchers::eq(6)]);
        }
    }

    mod contains_contiguous_satisfying {
        use crate::prelude::*;

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

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }
    }

    mod contains_exactly {
        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn compiles_for_string_values() {
            assert_that_owned!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_exactly(["a", "b"]);
        }

        #[test]
        fn panics_without_consumption_when_a_known_length_differs() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 2]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `[1, 2, 3].into_iter()`

                    does not contain exactly

                    Expected: [
                        1,
                        2,
                    ]

                    Details:
                      - Reported length: 3
                      - Expected length: 2
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_matching {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_matching([is_one, is_nine, is_three,].map(matchers::predicate))
            );
        }

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
        }

        #[test]
        fn panics_when_an_element_does_not_match() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_matching(
                        [is_one, is_nine, is_three].map(matchers::predicate),
                    );
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match the required position

                Details:
                  - Consumed elements: 2
                Nested failures:
                  - At [1]:
                    Actual: 2

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_an_extra_element_follows() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter().filter(|_| true))
                    .with_location(false)
                    .contains_exactly_matching(
                        [|it: &i32| *it == 1, |it: &i32| *it == 2].map(matchers::predicate),
                    );
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter().filter(|_| true)`

                has an extra element

                Details:
                  - Consumed elements: 3
                  - Extra element at index: 2
                -------- assertr --------
            "});
        }
    }

    mod contains_exactly_satisfying {
        use crate::prelude::*;

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

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }
    }

    mod contains_exactly_in_any_order {
        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn reports_the_collection_differences_and_the_consumption_of_a_longer_input() {
            assert_that!(|| {
                assert_that_owned!((1..).filter(|_| true))
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 1]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Expression: `(1..).filter(|_| true)`

                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain exactly in any order

                    Expected: [
                        2,
                        1,
                    ]

                    Details:
                      - Elements not expected: [
                            3,
                        ]
                      - Consumed elements: 3
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order([1, 2, 9]);
            })
            .panics()
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
                      - Elements not found: [
                            9,
                        ]
                      - Elements not expected: [
                            3,
                        ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use indoc::formatdoc;

        use crate::prelude::*;

        /// An item without `PartialEq`, matched by predicates over its field.
        #[derive(Debug)]
        struct Opaque(u8);

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_in_any_order_matching(
                    [is_one, is_two, is_nine,].map(matchers::predicate)
                )
            );
        }

        #[test]
        fn matches_items_without_equality_through_predicates() {
            assert_that_owned!([Opaque(1), Opaque(2)].into_iter())
                .contains_exactly_in_any_order_matching(
                    [|it: &Opaque| it.0 == 2, |it: &Opaque| it.0 == 1].map(matchers::predicate),
                );
        }

        #[test]
        fn assigns_overlapping_predicates_one_to_one() {
            assert_that_owned!([Opaque(2), Opaque(1)].into_iter())
                .contains_exactly_in_any_order_matching(
                    [|_: &Opaque| true, |it: &Opaque| it.0 == 2].map(matchers::predicate),
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
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that!(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching(
                        [is_one, is_two, is_nine].map(matchers::predicate),
                    );
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                does not match exactly in any order

                Details:
                  - Consumed elements: 3
                Nested failures:
                  - is missing an element matching this expectation

                    Constraint:
                        satisfies the predicate

                    Details:
                      - At slot: 2
                    Nested failures:
                      - Actual: 1

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 2

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 3

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                  - has unexpected elements

                    Details:
                      - Unexpected count: 1
                    Nested failures:
                      - Actual: 3

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate

                      - Actual: 3

                        does not satisfy the constraint

                        Constraint:
                            satisfies the predicate
                -------- assertr --------
            "});
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

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
    }
}
