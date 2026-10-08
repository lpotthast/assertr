use crate::{
    AssertThat, DebugRenderer, Expectation, Mode, ValueRenderer,
    assertions::{
        collection::Placement,
        iterator::{
            Contains, ContainsExactlyInAnyOrder, ContainsMatching, DoesNotContain,
            DoesNotContainMatching, ElementsAreInAnyOrder, ElementsEqual, ElementsMatch,
            PositionReporting::YieldOrder, run,
        },
    },
    borrow_for::{BorrowFor, borrow_for},
    expectation::{MatcherList, lists::SatisfyingList, satisfying},
    mode::Capture,
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
/// assertions buffer at most that many elements. An exact [`Iterator::size_hint`] can reject a
/// length mismatch before consuming anything, but never establishes success on its own.
///
/// Equality diagnostics preview at most the last 16 consumed elements, regardless of how long the
/// scan ran. Unordered equality instead reports the buffered elements like
/// [`CollectionAssertions::contains_exactly_in_any_order`](crate::assertions::collection::CollectionAssertions::contains_exactly_in_any_order).
/// Matcher and `_satisfying` diagnostics retain evidence from the first rejected candidates that
/// fit the [rendering budget](crate::RenderingBudget). Later rejections are only counted. Matchers
/// evaluate `&T`, and `_satisfying` closures receive a capture-mode assertion borrowing each
/// candidate element.
///
/// Bulk value lists use [repeatable expected data](crate#expected-lists).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait IteratorAssertions<'t, T, M: Mode, R = DebugRenderer> {
    /// Asserts that the iterator contains an element equal to `expected`.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator contains an element matching `expected`.
    fn contains_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: Expectation<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator's prefix matches the expected matcher list in order.
    fn starts_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that the iterator's suffix matches the expected matcher list in order.
    fn ends_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that a contiguous subsequence matches the expected matcher list in order.
    ///
    /// A rejection retains one group per rejected window within the rendering budget. Each group
    /// carries a `Window start` fact with the window's zero-based starting index.
    fn contains_contiguous_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>;

    /// Asserts that a contiguous subsequence satisfies `assertions` in order.
    ///
    /// Rejections are grouped per window like
    /// [`contains_contiguous_matching`](Self::contains_contiguous_matching).
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that no iterator element matches `expected`.
    fn does_not_contain_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: Expectation<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts that each element matches the constraint at the same position, including length.
    fn contains_exactly_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u;

    /// Asserts one-to-one matching between elements and the expected matcher list, independent of
    /// order.
    fn contains_exactly_in_any_order_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
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
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            (actual, Contains::<T, _>::new(borrow_for::<T, _>(&expected)))
        });
        this
    }

    #[track_caller]
    fn contains_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: Expectation<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            (actual, ContainsMatching::<T, _>::new(expected, YieldOrder))
        });
        this
    }

    #[track_caller]
    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.contains_matching(satisfying(assertions))
    }

    #[track_caller]
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        equal(self, &expected, Placement::Prefix)
    }

    #[track_caller]
    fn starts_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        matching(self, || expected, Placement::Prefix)
    }

    #[track_caller]
    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        matching(
            self,
            || SatisfyingList::new(assertions.as_ref()),
            Placement::Prefix,
        )
    }

    #[track_caller]
    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        equal(self, &expected, Placement::Suffix)
    }

    #[track_caller]
    fn ends_with_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        matching(self, || expected, Placement::Suffix)
    }

    #[track_caller]
    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: Clone + ValueRenderer<usize>,
        't: 'u,
    {
        matching(
            self,
            || SatisfyingList::new(assertions.as_ref()),
            Placement::Suffix,
        )
    }

    #[track_caller]
    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        equal(self, &expected, Placement::Contiguous)
    }

    #[track_caller]
    fn contains_contiguous_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        matching(self, || expected, Placement::Contiguous)
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
        matching(
            self,
            || SatisfyingList::new(assertions.as_ref()),
            Placement::Contiguous,
        )
    }

    #[track_caller]
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            let unexpected = borrow_for::<T, _>(&not_expected);
            (actual, DoesNotContain::<T, _>::new(unexpected, YieldOrder))
        });
        this
    }

    #[track_caller]
    fn does_not_contain_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: Expectation<T, R>,
        't: 'u,
        R: ValueRenderer<usize> + ValueRenderer<T>,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            (
                actual,
                DoesNotContainMatching::<T, _>::new(expected, YieldOrder),
            )
        });
        this
    }

    #[track_caller]
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: ValueRenderer<T> + Clone + ValueRenderer<usize>,
        't: 'u,
    {
        self.does_not_contain_matching(satisfying(assertions))
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        equal(self, &expected, Placement::Exact)
    }

    #[track_caller]
    fn contains_exactly_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        matching(self, || expected, Placement::Exact)
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
        matching(
            self,
            || SatisfyingList::new(assertions.as_ref()),
            Placement::Exact,
        )
    }

    #[track_caller]
    fn contains_exactly_in_any_order<'u, E>(
        self,
        expected: impl AsRef<[E]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
        't: 'u,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            let scan = ContainsExactlyInAnyOrder::<T, _>::new(expected.as_ref());
            (actual, scan)
        });
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<'u, P>(self, expected: P) -> AssertThat<'u, (), M, R>
    where
        P: MatcherList<T, R>,
        't: 'u,
        R: ValueRenderer<usize>,
    {
        let (actual, this) = take_iterator(self);
        run(&this, || {
            (actual, ElementsAreInAnyOrder::<T, _>::new(expected))
        });
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
        let (actual, this) = take_iterator(self);
        run(&this, || {
            let expected = SatisfyingList::new(assertions.as_ref());
            (actual, ElementsAreInAnyOrder::<T, _>::new(expected))
        });
        this
    }
}

/// Takes the iterator out of `this`, returning it together with the terminal `()` assertion.
#[track_caller]
fn take_iterator<'t, 'u, I, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
) -> (I, AssertThat<'u, (), M, R>)
where
    't: 'u,
{
    this.take_owned(
        "Iterator assertions consume the iterator and therefore need to own it. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
    )
}

/// Runs a positional equality scan, accessing the expected list only after tracking.
#[track_caller]
fn equal<'t, 'u, T, I, E, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
    expected: &impl AsRef<[E]>,
    placement: Placement,
) -> AssertThat<'u, (), M, R>
where
    I: Iterator<Item = T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
    't: 'u,
{
    let (actual, this) = take_iterator(this);
    run(&this, || {
        (
            actual,
            ElementsEqual::<T, _>::new(expected.as_ref(), placement),
        )
    });
    this
}

/// Runs a positional matcher scan, creating the list only after tracking.
#[track_caller]
fn matching<'t, 'u, T, I, L, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
    expected: impl FnOnce() -> L,
    placement: Placement,
) -> AssertThat<'u, (), M, R>
where
    I: Iterator<Item = T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
    't: 'u,
{
    let (actual, this) = take_iterator(this);
    run(&this, || {
        (actual, ElementsMatch::<T, _>::new(expected(), placement))
    });
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
        use crate::prelude::*;
        use indoc::formatdoc;

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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_matching(crate::expectation::predicate(|it: &i32| *it > 7))
            );
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
        use crate::prelude::*;
        use indoc::formatdoc;

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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                does_not_contain_matching(crate::expectation::predicate(|it: &i32| *it % 2 == 0))
            );
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
        use crate::prelude::*;
        use indoc::formatdoc;

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

        fn is_nine(value: &i32) -> bool {
            *value == 9
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
        use crate::prelude::*;
        use indoc::formatdoc;

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
                      - At [2]:
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                ends_with_matching(crate::expectation::predicate_list([is_two, is_nine]))
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
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_matching(crate::expectation::predicate_list([is_two, is_nine]));
            })
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
        use crate::prelude::*;
        use indoc::formatdoc;

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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_contiguous_matching(crate::expectation::predicate_list(
                    [is_two, is_nine,]
                ))
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
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 2]);
            })
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
        use crate::prelude::*;
        use indoc::formatdoc;

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

        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        fn is_nine(value: &i32) -> bool {
            *value == 9
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
            assert_that_panic_by(|| {
                assert_that_owned!([1, 2, 3].into_iter().filter(|_| true))
                    .with_location(false)
                    .contains_exactly_matching(crate::expectation::predicate_list([
                        |it: &i32| *it == 1,
                        |it: &i32| *it == 2,
                    ]));
            })
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
        use crate::prelude::*;

        use indoc::formatdoc;

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
            assert_that_panic_by(|| {
                assert_that_owned!((1..).filter(|_| true))
                    .with_location(false)
                    .contains_exactly_in_any_order([2, 1]);
            })
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
        use crate::prelude::*;
        use indoc::formatdoc;

        /// An item without `PartialEq`, matched by predicates over its field.
        #[derive(Debug)]
        struct Opaque(u8);

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that_owned!([1, 2, 3].into_iter()),
                contains_exactly_in_any_order_matching(crate::expectation::predicate_list([
                    is_one, is_two, is_nine,
                ]))
            );
        }

        #[test]
        fn matches_items_without_equality_through_predicates() {
            assert_that_owned!([Opaque(1), Opaque(2)].into_iter())
                .contains_exactly_in_any_order_matching(crate::expectation::predicate_list([
                    |it: &Opaque| it.0 == 2,
                    |it: &Opaque| it.0 == 1,
                ]));
        }

        #[test]
        fn assigns_overlapping_predicates_one_to_one() {
            assert_that_owned!([Opaque(2), Opaque(1)].into_iter())
                .contains_exactly_in_any_order_matching(crate::expectation::predicate_list([
                    |_: &Opaque| true,
                    |it: &Opaque| it.0 == 2,
                ]));
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
