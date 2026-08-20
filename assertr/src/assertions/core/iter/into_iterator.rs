use alloc::vec::Vec;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode, assertions::iterator, mode::Capture,
    tracking::AssertionTracking,
};

/// Chainable assertions over a fresh borrowed iteration of a collection-like value.
///
/// Each method calls `IntoIterator::into_iter(&actual)` exactly once and returns the original
/// assertion. Chaining therefore performs one fresh borrowed traversal per assertion. Streaming,
/// bounded-preview, sequence-criteria, and potential-nontermination behavior matches
/// [`super::IteratorAssertions`].
/// Method names are prefixed to avoid collisions with more specific collection assertion traits.
#[allow(clippy::return_self_not_must_use)]
pub trait IntoIteratorAssertions<T, R> {
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>;
    fn into_iter_contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_starts_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>;
    fn into_iter_starts_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_ends_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>;
    fn into_iter_ends_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>;
    fn into_iter_contains_contiguous_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_contains_contiguous_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>;
    fn into_iter_does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>;
    fn into_iter_contains_exactly_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_contains_exactly_in_any_order(self, expected: impl AsRef<[T]>) -> Self
    where
        T: PartialEq,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_contains_exactly_in_any_order_matching<P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone;
    fn into_iter_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_is_not_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>;

    #[deprecated(since = "0.6.2", note = "renamed to `into_iter_is_empty`")]
    /// Deprecated forwarding name for [`IntoIteratorAssertions::into_iter_is_empty`].
    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
}

impl<T, I, M: Mode, R> IntoIteratorAssertions<T, R> for AssertThat<'_, I, M, R>
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    #[track_caller]
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        iterator::assert_contains::<_, T, _, _, _, _>(&self, self.actual().into_iter(), &expected);
        self
    }
    #[track_caller]
    fn into_iter_contains_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_contains_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &predicate,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &assertions,
        );
        self
    }
    #[track_caller]
    fn into_iter_starts_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let rendered = expected.iter().collect::<Vec<_>>();
        iterator::assert_starts_with::<_, T, _, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
            &rendered,
        );
        self
    }
    #[track_caller]
    fn into_iter_starts_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_starts_with_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_starts_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_starts_with_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_ends_with<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let rendered = expected.iter().collect::<Vec<_>>();
        iterator::assert_ends_with::<_, T, _, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
            &rendered,
        );
        self
    }
    #[track_caller]
    fn into_iter_ends_with_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_ends_with_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_ends_with_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_ends_with_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_contiguous<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let rendered = expected.iter().collect::<Vec<_>>();
        iterator::assert_contains_contiguous::<_, T, _, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
            &rendered,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_contiguous_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_contains_contiguous_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_contiguous_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_contiguous_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        iterator::assert_does_not_contain::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &not_expected,
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_matching<P>(self, predicate: P) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_does_not_contain_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &predicate,
        );
        self
    }
    #[track_caller]
    fn into_iter_does_not_contain_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_does_not_contain_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            &assertions,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let rendered = expected.iter().collect::<Vec<_>>();
        iterator::assert_contains_exactly::<_, T, _, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
            &rendered,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_matching<P>(self, predicates: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_satisfying<A>(self, assertions: impl AsRef<[A]>) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order(self, expected: impl AsRef<[T]>) -> Self
    where
        T: PartialEq,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        let expected = expected.as_ref();
        let rendered = expected.iter().collect::<Vec<_>>();
        iterator::assert_contains_exactly_in_any_order::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            expected,
            &rendered,
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_matching<P>(
        self,
        predicates: impl AsRef<[P]>,
    ) -> Self
    where
        P: Fn(&T) -> bool,
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_in_any_order_matching::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            predicates.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_contains_exactly_in_any_order_satisfying<A>(
        self,
        assertions: impl AsRef<[A]>,
    ) -> Self
    where
        A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
        R: for<'a> AssertionRenderer<Vec<&'a T>> + Clone,
    {
        self.track_assertion();
        iterator::assert_contains_exactly_in_any_order_satisfying::<_, T, _, _, _, _>(
            &self,
            self.actual().into_iter(),
            assertions.as_ref(),
        );
        self
    }
    #[track_caller]
    fn into_iter_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_is_empty::<_, T, _, _, _>(&self, self.actual().into_iter());
        self
    }
    #[track_caller]
    fn into_iter_is_not_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_is_not_empty::<_, T, _, _, _>(&self, self.actual().into_iter());
        self
    }
    #[track_caller]
    fn into_iter_has_length(self, expected: usize) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        iterator::assert_has_length::<_, T, _, _, _>(&self, self.actual().into_iter(), expected);
        self
    }
    #[track_caller]
    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.into_iter_is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    mod into_iter_contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_contains(2);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo".to_owned()]).into_iter_contains("foo");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains(4);
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

    mod into_iter_contains_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_an_element_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_matching(|it: &i32| *it % 2 == 0);
        }

        #[test]
        fn panics_when_no_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_matching(|it: &i32| *it > 7);
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

    mod into_iter_contains_satisfying {
        use crate::prelude::*;

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_seven(it: AssertThat<i32, Capture>) {
            it.is_equal_to(7);
        }

        #[test]
        fn succeeds_when_an_element_satisfies() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_satisfying(is_two);
        }

        #[test]
        fn panics_when_no_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2])
                    .with_location(false)
                    .into_iter_contains_satisfying(is_seven);
            })
            .has_type::<String>()
            .contains("does not contain an element satisfying the assertions.")
            .contains("Element at index 0 does not satisfy the assertions:\n    Expected: 7")
            .contains("Element at index 1 does not satisfy the assertions:\n    Expected: 7");
        }
    }

    mod into_iter_does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain(4);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["foo".to_owned()]).into_iter_does_not_contain("bar");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain(2);
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

    mod into_iter_does_not_contain_matching {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_no_element_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain_matching(|it: &i32| *it > 7);
        }

        #[test]
        fn panics_when_an_element_matches() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain_matching(|it: &i32| *it % 2 == 0);
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

    mod into_iter_does_not_contain_satisfying {
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
            assert_that!(vec![1, 2, 3]).into_iter_does_not_contain_satisfying(is_seven);
        }

        #[test]
        fn panics_when_an_element_satisfies() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_does_not_contain_satisfying(is_two);
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

    mod into_iter_starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_prefix_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_starts_with([1, 2]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()]).into_iter_starts_with(["a"]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_starts_with([1, 9]);
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

    mod into_iter_starts_with_matching {
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
            assert_that!(vec![1, 2, 3]).into_iter_starts_with_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_prefix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_starts_with_matching([is_one, is_nine]);
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

    mod into_iter_starts_with_satisfying {
        use crate::prelude::*;

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_nine(it: AssertThat<i32, Capture>) {
            it.is_equal_to(9);
        }

        #[test]
        fn succeeds_when_prefix_satisfies() {
            assert_that!(vec![1, 2, 3]).into_iter_starts_with_satisfying([is_one]);
        }

        #[test]
        fn panics_when_prefix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_starts_with_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("does not start with elements satisfying the assertions.")
            .contains(
                "Element at index 1 does not satisfy its prefix assertions:\n    Expected: 9",
            );
        }
    }

    mod into_iter_ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_suffix_matches() {
            assert_that!(vec![1, 2, 3]).into_iter_ends_with([2, 3]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()]).into_iter_ends_with(["b"]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_ends_with([2, 9]);
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

    mod into_iter_ends_with_matching {
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
            assert_that!(vec![1, 2, 3]).into_iter_ends_with_matching([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_ends_with_matching([is_two, is_nine]);
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

    mod into_iter_ends_with_satisfying {
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
            assert_that!(vec![1, 2, 3]).into_iter_ends_with_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_suffix_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_ends_with_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not end with elements satisfying the assertions.")
            .contains(
                "Suffix element at index 2 does not satisfy its assertions:\n    Expected: 9",
            );
        }
    }

    mod into_iter_contains_contiguous {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_a_contiguous_match_exists() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_contiguous([2, 3]);
        }

        #[test]
        fn succeeds_when_candidates_overlap() {
            assert_that!(vec![1, 1, 2]).into_iter_contains_contiguous([1, 2]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()])
                .into_iter_contains_contiguous(["a", "b"]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_contiguous([2, 9]);
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

    mod into_iter_contains_contiguous_matching {
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
            assert_that!(vec![1, 2, 3]).into_iter_contains_contiguous_matching([is_one, is_two]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_contiguous_matching([is_two, is_nine]);
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

    mod into_iter_contains_contiguous_satisfying {
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
            assert_that!(vec![1, 2, 3])
                .into_iter_contains_contiguous_satisfying([is_two, is_three]);
        }

        #[test]
        fn panics_when_no_contiguous_match_exists() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_contiguous_satisfying([is_two, is_nine]);
            })
            .has_type::<String>()
            .contains("does not contain contiguous elements satisfying the assertions.")
            .contains("The final contiguous candidate did not satisfy the assertions:");
        }
    }

    mod into_iter_contains_exactly {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_elements_match_exactly() {
            assert_that!(vec![1, 2, 3]).into_iter_contains_exactly([1, 2, 3]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(vec!["a".to_owned(), "b".to_owned()])
                .into_iter_contains_exactly(["a", "b"]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly([1, 9, 3]);
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
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly([1, 2]);
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

    mod into_iter_contains_exactly_matching {
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
            assert_that!(vec![1, 2, 3])
                .into_iter_contains_exactly_matching([is_one, is_two, is_three]);
        }

        #[test]
        fn panics_when_an_element_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_matching([is_one, is_nine, is_three]);
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

    mod into_iter_contains_exactly_satisfying {
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
            assert_that!(vec![1, 2]).into_iter_contains_exactly_satisfying([is_one, is_two]);
        }

        #[test]
        fn panics_when_an_element_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2])
                    .with_location(false)
                    .into_iter_contains_exactly_satisfying([is_one, is_nine]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions.")
            .contains("Element at index 1 does not satisfy its assertions:\n    Expected: 9");
        }
    }

    mod into_iter_contains_exactly_in_any_order {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_elements_match_in_another_order() {
            assert_that!(vec![2, 1, 1]).into_iter_contains_exactly_in_any_order([1, 2, 1]);
        }

        #[test]
        fn panics_when_an_element_differs() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order([1, 2, 9]);
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

    mod into_iter_contains_exactly_in_any_order_matching {
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
            assert_that!(vec![1, 2])
                .into_iter_contains_exactly_in_any_order_matching([is_at_most_two, is_one]);
        }

        #[test]
        fn panics_when_a_predicate_stays_unmatched() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order_matching([is_one, is_two, is_nine]);
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

    mod into_iter_contains_exactly_in_any_order_satisfying {
        use crate::prelude::*;

        fn positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        fn negative(it: AssertThat<i32, Capture>) {
            it.is_less_than(0);
        }

        #[test]
        fn succeeds_when_a_maximum_matching_exists() {
            assert_that!(vec![-1, 1])
                .into_iter_contains_exactly_in_any_order_satisfying([positive, negative]);
        }

        #[test]
        fn panics_when_an_element_satisfies_no_assertion() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, -1, 2])
                    .with_location(false)
                    .into_iter_contains_exactly_in_any_order_satisfying([
                        positive, positive, positive,
                    ]);
            })
            .has_type::<String>()
            .contains("did not exactly satisfy the assertions in any order.")
            .contains("Element at index 1 did not satisfy any available assertion:");
        }
    }

    mod into_iter_is_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_empty() {
            assert_that!(Vec::<i32>::new()).into_iter_is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!(vec![1])
                    .with_location(false)
                    .into_iter_is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: [
                        1,
                    ]

                    is not empty.

                    Details: [
                        Consumed 1 element(s).,
                        Decisive element is at zero-based index 0.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_is_not_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_not_empty() {
            assert_that!(vec![1]).into_iter_is_not_empty();
        }

        #[test]
        fn panics_when_empty() {
            assert_that_panic_by(|| {
                assert_that!(Vec::<i32>::new())
                    .with_location(false)
                    .into_iter_is_not_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: []

                    is empty.
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_has_length {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_length_matches() {
            assert_that!(vec![1, 2]).into_iter_has_length(2);
        }

        #[test]
        fn panics_when_length_differs() {
            assert_that_panic_by(|| {
                assert_that!(vec![1, 2, 3])
                    .with_location(false)
                    .into_iter_has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                    -------- assertr --------
                    Actual: []

                    does not have the correct length

                    Expected: 2
                    Observed: 3

                    Details: [
                        Iterator reported an exact remaining length of 3; no elements were consumed.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod into_iter_iterator_is_empty {
        use crate::prelude::*;

        #[test]
        #[allow(deprecated)]
        fn deprecated_name_remains_available() {
            assert_that!(Vec::<i32>::new()).into_iter_iterator_is_empty();
        }
    }

    mod chaining {
        use crate::prelude::*;

        #[test]
        fn assertions_chain_on_the_original_subject() {
            assert_that!(vec![1, 2])
                .into_iter_contains(1)
                .into_iter_does_not_contain(3)
                .into_iter_starts_with([1])
                .into_iter_ends_with([2])
                .into_iter_has_length(2)
                .into_iter_is_not_empty();
        }
    }
}
