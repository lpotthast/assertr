use alloc::vec::Vec;
use core::fmt::Debug;

use crate::actual::Actual;
use crate::{AssertThat, AssertrPartialEq, Mode, tracking::AssertionTracking};

pub trait IteratorAssertions<'t, T: Debug, M: Mode> {
    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: AssertrPartialEq<E>,
        't: 'u;

    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contain<'u, E>(self, expected: E) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: AssertrPartialEq<E>,
        't: 'u,
        Self: Sized,
    {
        self.contains(expected)
    }

    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: PartialEq<E>,
        T: AssertrPartialEq<E> + Debug,
        't: 'u;

    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contain_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: PartialEq<E>,
        T: AssertrPartialEq<E> + Debug,
        't: 'u,
        Self: Sized,
    {
        self.contains_exactly(expected)
    }
}

impl<'t, T, I, M: Mode> IteratorAssertions<'t, T, M> for AssertThat<'t, I, M>
where
    T: Debug,
    I: Iterator<Item = T>,
{
    #[track_caller]
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: Debug + AssertrPartialEq<E>,
        't: 'u,
    {
        self.track_assertion();
        // Any iterator can only be iterated once! Take it.
        let (actual, this) = self.replace_actual_with(Actual::Owned(()));

        let actual = actual.unwrap_owned().collect::<Vec<_>>();
        let expected = expected;
        if !actual
            .iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            this.fail(format_args!(
                "Actual: {actual:#?}\n\ndoes not contain expected: {expected:#?}\n",
            ));
        }
        this
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: PartialEq<E>, // TOOD: Why exactly do we need this bound? Can we get rid of it? It is required in order to util::slice::compare `&[&T]` with `&[&E]`...
        T: AssertrPartialEq<E> + Debug,
        't: 'u,
    {
        self.track_assertion();

        let (actual, this) = self.replace_actual_with(Actual::Owned(()));

        let actual = actual.unwrap_owned().collect::<Vec<_>>();
        let expected = expected.as_ref();

        let result = crate::util::slice::compare(actual.as_slice(), expected);

        if !result.strictly_equal {
            if !result.not_in_b.is_empty() {
                this.add_detail_message(format!("Elements not expected: {:#?}", result.not_in_b));
            }
            if !result.not_in_a.is_empty() {
                this.add_detail_message(format!("Elements not found: {:#?}", result.not_in_a));
            }
            if result.only_differing_in_order() {
                this.add_detail_message("The order of elements does not match!".to_owned());
            }

            this.fail(format_args!(
                "Actual: {actual:#?},\n\ndid not exactly match\n\nExpected: {expected:#?}\n",
            ));
        }
        this
    }
}

/// Assertions for any type convertable to some `Iterator` using the `IntoIterator` trait.
/// Assertions partly match the known assertions for slices, as an iterator can roughly be seen as
/// a collection (simply without random access to it and only the possibility to iterate once).
///
/// Assertions are prefixed to distinguish these assertions from more concrete implementations
/// on the actual type, like `Vec` for example.
pub trait IntoIteratorAssertions<T: Debug> {
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E>;

    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug,
        T: PartialEq<E>,
        T: AssertrPartialEq<E> + Debug;

    fn into_iter_iterator_is_empty(self) -> Self;
}

impl<T, I, M: Mode> IntoIteratorAssertions<T> for AssertThat<'_, I, M>
where
    T: Debug,
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: Debug + AssertrPartialEq<E>,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();
        let expected = expected;
        if !self
            .actual()
            .into_iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            self.fail(format_args!(
                "Actual: {actual:#?}\n\ndoes not contain expected: {expected:#?}\n",
            ));
        }
        self
    }

    #[track_caller]
    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug,
        T: PartialEq<E>, // TOOD: Why exactly do we need this bound? Can we get rid of it? It is required in order to util::slice::compare `&[&T]` with `&[&E]`...
        T: AssertrPartialEq<E> + Debug,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();
        let expected = expected.as_ref().iter().collect::<Vec<_>>();

        let result = crate::util::slice::compare(actual.as_slice(), expected.as_slice());

        if !result.strictly_equal {
            if !result.not_in_b.is_empty() {
                self.add_detail_message(format!("Elements not expected: {:#?}", result.not_in_b));
            }
            if !result.not_in_a.is_empty() {
                self.add_detail_message(format!("Elements not found: {:#?}", result.not_in_a));
            }
            if result.only_differing_in_order() {
                self.add_detail_message("The order of elements does not match!".to_owned());
            }

            self.fail(format_args!(
                "Actual: {actual:#?},\n\ndid not exactly match\n\nExpected: {expected:#?}\n",
            ));
        }
        self
    }

    // TODO: Should this exist? Should we create is_empty() impl's for concrete collection types instead?
    #[track_caller]
    fn into_iter_iterator_is_empty(self) -> Self {
        self.track_assertion();
        if self.actual().into_iter().count() != 0 {
            let actual = self.actual().into_iter().collect::<Vec<_>>();
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nIs not empty!\n",
                //actual = self.actual_ref(),
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod iterator_assertions {

        mod contains {
            use crate::prelude::*;

            #[test]
            fn succeeds_when_value_is_present() {
                let values = [1, 2, 3];
                let iter = values.iter();
                // TODO: We could also call must() which would lead to a panic, as we cannot
                //  "unwrap_owned" an owned Iterator anymore.
                iter.must_owned().contain(&1);
            }

            #[test]
            fn compiles_for_comparable_but_different_type() {
                let values = vec!["foo"];
                values.must().into_iter_contains("foo".to_string());
            }
        }
    }

    mod into_iterator_assertions {

        mod contains {
            use crate::prelude::*;

            #[test]
            fn succeeds_when_value_is_present() {
                let values = vec![1, 2, 3, 42];
                assert_that_owned(values)
                    .into_iter_contains(1)
                    .into_iter_contains(42)
                    .into_iter_contains(3)
                    .into_iter_contains(2);
            }

            #[test]
            fn compiles_for_comparable_but_different_type() {
                let values = vec!["foo"];
                assert_that_owned(values).into_iter_contains("foo".to_string());
            }
        }
    }
}
