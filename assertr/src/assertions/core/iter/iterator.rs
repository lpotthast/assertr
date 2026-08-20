use alloc::vec::Vec;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode, actual::Actual, assertions::iterator,
    mode::Capture, tracking::AssertionTracking,
};

/// Terminal assertions for an owned iterator.
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
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u;

    fn contains_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn contains_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u;

    fn starts_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn starts_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn ends_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u;

    fn ends_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn ends_with_satisfying<'u, A>(self, assertions: impl AsRef<[A]>) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn contains_contiguous<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u;

    fn contains_contiguous_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn contains_contiguous_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u;

    fn does_not_contain_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u;

    fn contains_exactly_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn contains_exactly_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u;

    fn contains_exactly_in_any_order<'u>(
        self,
        expected: impl AsRef<[T]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[T]>,
        't: 'u;

    fn contains_exactly_in_any_order_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u;

    fn contains_exactly_in_any_order_satisfying<'u, A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
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
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
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
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_satisfying::<_, T, _, _, _, _>(&this, iter, &assertions);
        this
    }

    #[track_caller]
    fn starts_with<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_starts_with::<_, T, _, _, _, _, _>(&this, iter, expected, expected);
        this
    }

    #[track_caller]
    fn starts_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
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
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_ends_with::<_, T, _, _, _, _, _>(&this, iter, expected, expected);
        this
    }

    #[track_caller]
    fn ends_with_matching<'u, P>(self, predicates: impl AsRef<[P]>) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
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
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_contiguous::<_, T, _, _, _, _, _>(
            &this, iter, expected, expected,
        );
        this
    }

    #[track_caller]
    fn contains_contiguous_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
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
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain::<_, T, _, _, _, _>(&this, iter, &not_expected);
        this
    }

    #[track_caller]
    fn does_not_contain_matching<'u, P>(self, predicate: P) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain_matching::<_, T, _, _, _, _>(&this, iter, &predicate);
        this
    }

    #[track_caller]
    fn does_not_contain_satisfying<'u, A>(self, assertions: A) -> AssertThat<'u, (), M, R>
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u,
    {
        let (iter, this) = take_iterator(self);
        iterator::assert_does_not_contain_satisfying::<_, T, _, _, _, _>(&this, iter, &assertions);
        this
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly::<_, T, _, _, _, _, _>(&this, iter, expected, expected);
        this
    }

    #[track_caller]
    fn contains_exactly_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_satisfying::<_, T, _, _, _, _>(&this, iter, assertions);
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order<'u>(
        self,
        expected: impl AsRef<[T]>,
    ) -> AssertThat<'u, (), M, R>
    where
        T: PartialEq,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[T]>,
        't: 'u,
    {
        let expected = expected.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_in_any_order::<_, T, _, _, _, _>(
            &this, iter, expected, expected,
        );
        this
    }

    #[track_caller]
    fn contains_exactly_in_any_order_matching<'u, P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> AssertThat<'u, (), M, R>
    where
        P: Fn(&T) -> bool,
        R: AssertionRenderer<Vec<T>>,
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
        R: AssertionRenderer<Vec<T>> + Clone,
        't: 'u,
    {
        let assertions = assertions.as_ref();
        let (iter, this) = take_iterator(self);
        iterator::assert_contains_exactly_in_any_order_satisfying::<_, T, _, _, _, _>(
            &this, iter, assertions,
        );
        this
    }
}

/// Takes the iterator out of `this`, returning it together with the terminal `()` assertion.
///
/// The assertion itself must run directly inside the calling `#[track_caller]` trait method,
/// not inside a closure passed to a helper, so failure locations point at the user's call site.
fn take_iterator<'t, 'u, T, I, M: Mode, R>(
    this: AssertThat<'t, I, M, R>,
) -> (I, AssertThat<'u, (), M, R>)
where
    I: Iterator<Item = T>,
    't: 'u,
{
    this.track_assertion();
    let (actual, terminal) = this.replace_actual_with(Actual::Owned(()));
    (actual.unwrap_owned(), terminal)
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!([1, 2, 3].into_iter()).contains(2);
        }

        #[test]
        fn succeeds_on_an_unbounded_iterator_when_a_match_occurs() {
            assert_that!(0..).contains(3);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo".to_owned()].into_iter()).contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains(4);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain expected: 4

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!([1, 2, 3].into_iter()).contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_matching(|it: &i32| *it > 7);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain an element matching the predicate.

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_satisfying {
        use crate::prelude::*;

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!([1, 2, 3].into_iter()).contains_satisfying(is_two);
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].into_iter())
                    .with_location(false)
                    .contains_satisfying(is_seven);
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the assertions.")
            .contains("Element at index 0 does not satisfy the assertions:\n    Expected: 7")
            .contains("Element at index 1 does not satisfy the assertions:\n    Expected: 7");
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!([1, 2, 3].into_iter()).does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo".to_owned()].into_iter()).does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ]

                    contains unexpected: 2

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!([1, 2, 3].into_iter()).does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_matching(|it: &i32| *it % 2 == 0);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ]

                    contains an element matching the predicate.

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod does_not_contain_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_no_element_satisfies() {
            assert_that!([1, 2, 3].into_iter()).does_not_contain_satisfying(is_seven);
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .does_not_contain_satisfying(is_two);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ]

                    contains an element satisfying the assertions.

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_prefix_matches() {
            assert_that!([1, 2, 3].into_iter()).starts_with([1, 2]);
        }

        #[test]
        fn succeeds_for_an_empty_prefix() {
            assert_that!([1, 2, 3].into_iter()).starts_with::<i32>([]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()].into_iter()).starts_with(["a"]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with([1, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ]

                    does not start with expected prefix: [
                        1,
                        9,
                    ]

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod starts_with_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that!([1, 2, 3].into_iter()).starts_with_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_matching([is_one, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ]

                    does not start with elements matching the predicates.

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod starts_with_satisfying {
        use crate::prelude::*;

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_prefix_satisfies() {
            assert_that!([1, 2, 3].into_iter()).starts_with_satisfying([is_one]);
        }

        #[test]
        fn panics_when_prefix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .starts_with_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("does not start with elements satisfying the assertions.")
            .contains(
                "Element at index 1 does not satisfy its prefix assertions:\n    Expected: 9",
            );
        }
    }

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_suffix_matches() {
            assert_that!([1, 2, 3].into_iter()).ends_with([2, 3]);
        }

        #[test]
        fn succeeds_for_an_empty_suffix() {
            assert_that!([1, 2, 3].into_iter()).ends_with::<i32>([]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()].into_iter()).ends_with(["b"]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with([2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not end with expected suffix: [
                        2,
                        9,
                    ]

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod ends_with_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that!([1, 2, 3].into_iter()).ends_with_matching([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_matching([is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not end with elements matching the predicates.

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
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

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_suffix_satisfies() {
            assert_that!([1, 2, 3].into_iter()).ends_with_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .ends_with_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not end with elements satisfying the assertions.")
            .contains(
                "Suffix element at index 2 does not satisfy its assertions:\n    Expected: 9",
            );
        }
    }

    mod contains_contiguous {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_a_contiguous_match_exists() {
            assert_that!([1, 2, 3].into_iter()).contains_contiguous([2, 3]);
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that!([1, 1, 2].into_iter()).contains_contiguous([1, 2]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_contiguous(["a", "b"]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous([2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain contiguous expected elements: [
                        2,
                        9,
                    ]

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_contiguous_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that!([1, 2, 3].into_iter()).contains_contiguous_matching([is_one, is_two]);
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that!([1, 1, 2].into_iter()).contains_contiguous_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_matching([is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ]

                    does not contain contiguous elements matching the predicates.

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_contiguous_satisfying {
        use crate::prelude::*;

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
            assert_that!([1, 2, 3].into_iter()).contains_contiguous_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_contiguous_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not contain contiguous elements satisfying the assertions.")
            .contains("The final contiguous candidate did not satisfy the assertions:");
        }
    }

    mod contains_exactly {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_elements_match_exactly() {
            assert_that!([1, 2, 3].into_iter()).contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()].into_iter())
                .contains_exactly(["a", "b"]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 9, 3]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ],

                    did not exactly match

                    Expected: [
                        1,
                        9,
                        3,
                    ]

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_without_consumption_when_a_known_length_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly([1, 2]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [],

                    did not exactly match

                    Expected: [
                        1,
                        2,
                    ]

                    Details: [
                        Consumed 0 element(s).,
                        Iterator reported an exact remaining length of 3; expected 2.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that!([1, 2, 3].into_iter())
                .contains_exactly_matching([is_one, is_two, is_three]);
        }

        #[test]
        fn panics_when_an_element_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_matching([is_one, is_nine, is_three]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                    ],

                    did not exactly match predicates.

                    Details: [
                        Consumed 2 element(s).,
                        Decisive element is at zero-based index 1.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_satisfying {
        use crate::prelude::*;

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
            assert_that!([1, 2].into_iter()).contains_exactly_satisfying([is_one, is_two]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!([1, 2].into_iter())
                    .with_location(false)
                    .contains_exactly_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions.")
            .contains("Element at index 1 does not satisfy its assertions:\n    Expected: 9");
        }
    }

    mod contains_exactly_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that!([2, 1, 1].into_iter()).contains_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order([1, 2, 9]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    Elements expected: [
                        1,
                        2,
                        9,
                    ]

                    The elements did not match exactly in any order.

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

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
            assert_that!([1, 2].into_iter())
                .contains_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that!([1, 2, 3].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_matching([is_one, is_two, is_nine]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                        2,
                        3,
                    ],

                    did not exactly match predicates in any order.

                    Details: [
                        Consumed 3 element(s).,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        fn positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        fn negative(it: AssertThat<i32, Capture>) {
            it.is_less_than(0);
        }

        #[test]
        fn succeeds_when_a_maximum_matching_exists() {
            assert_that!([-1, 1].into_iter())
                .contains_exactly_in_any_order_satisfying([positive, negative]);
        }

        #[test]
        fn panics_when_an_element_satisfies_no_assertion() {
            assert_that_panic_by(|| {
                assert_that!([1, -1, 2].into_iter())
                    .with_location(false)
                    .contains_exactly_in_any_order_satisfying([positive, positive, positive]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions in any order.")
            .contains("Element at index 1 did not satisfy any available assertion:");
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn sequence_assertions_have_fluent_aliases() {
            assert_that!(0..).start_with([0, 1]);
            assert_that!(0..3).end_with([1, 2]);
            assert_that!(0..).contain_contiguous([2, 3]);
        }
    }
}
