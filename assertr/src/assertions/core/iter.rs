use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;

use crate::actual::Actual;
use crate::{AssertThat, AssertionRenderer, AssertrPartialEq, Mode, tracking::AssertionTracking};

#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait IteratorAssertions<'t, T, M: Mode, R> {
    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u;

    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u;

    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
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
        self.track_assertion();
        // Any iterator can only be iterated once! Take it.
        let (actual, this) = self.replace_actual_with(Actual::Owned(()));

        let actual = actual.unwrap_owned().collect::<Vec<_>>();
        let expected = expected;
        if !actual.iter().any(|it| {
            let mut ctx = this.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &expected, Some(&mut ctx))
        }) {
            let actual = this.render_value(&actual);
            let expected = this.render_value(&expected);
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    does not contain expected: {expected:#?}
                "}
            });
        }
        this
    }

    #[track_caller]
    fn does_not_contain<'u, E>(self, not_expected: E) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<E>,
        't: 'u,
    {
        self.track_assertion();

        let (actual, this) = self.replace_actual_with(Actual::Owned(()));
        let actual = actual.unwrap_owned().collect::<Vec<_>>();

        if actual.iter().any(|it| {
            let mut ctx = this.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &not_expected, Some(&mut ctx))
        }) {
            let actual = this.render_value(&actual);
            let not_expected = this.render_value(&not_expected);
            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    contains unexpected: {not_expected:#?}
                "}
            });
        }
        this
    }

    #[track_caller]
    fn contains_exactly<'u, E>(self, expected: impl AsRef<[E]>) -> AssertThat<'u, (), M, R>
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<Vec<T>> + AssertionRenderer<[E]>,
        't: 'u,
    {
        self.track_assertion();

        let (actual, this) = self.replace_actual_with(Actual::Owned(()));

        let actual = actual.unwrap_owned().collect::<Vec<_>>();
        let expected = expected.as_ref();

        let mut ctx = this.eq_context();
        let result =
            crate::util::slice::compare_with_context(actual.as_slice(), expected, Some(&mut ctx));

        if !result.strictly_equal {
            if result.only_differing_in_order() {
                this.add_detail_message("The order of elements does not match!".to_owned());
            }
            let actual = this.render_value(&actual);
            let expected = this.render_value(expected);

            this.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?},

                    did not exactly match

                    Expected: {expected:#?}
                "}
            });
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
#[allow(clippy::return_self_not_must_use)]
pub trait IntoIteratorAssertions<T, R> {
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>;

    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>;

    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E>,
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>;

    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>;
}

impl<T, I, M: Mode, R> IntoIteratorAssertions<T, R> for AssertThat<'_, I, M, R>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn into_iter_contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();
        let expected = expected;
        if !self.actual().into_iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(&actual);
            let expected = self.render_value(&expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    does not contain expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn into_iter_does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + AssertionRenderer<E>,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();

        if actual.iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(*it, &not_expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(&actual);
            let not_expected = self.render_value(&not_expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    contains unexpected: {not_expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn into_iter_contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        T: PartialEq<E>, // TOOD: Why exactly do we need this bound? Can we get rid of it? It is required in order to util::slice::compare `&[&T]` with `&[&E]`...
        T: AssertrPartialEq<E, R>,
        R: for<'a> AssertionRenderer<Vec<&'a T>> + for<'a> AssertionRenderer<Vec<&'a E>>,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();
        let expected = expected.as_ref().iter().collect::<Vec<_>>();

        let mut ctx = self.eq_context();
        let result = crate::util::slice::compare_with_context(
            actual.as_slice(),
            expected.as_slice(),
            Some(&mut ctx),
        );

        if !result.strictly_equal {
            if result.only_differing_in_order() {
                self.add_detail_message("The order of elements does not match!".to_owned());
            }
            let actual = self.render_value(&actual);
            let expected = self.render_value(&expected);

            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?},

                    did not exactly match

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }

    // TODO: Should this exist? Should we create is_empty() impl's for concrete collection types instead?
    #[track_caller]
    fn into_iter_iterator_is_empty(self) -> Self
    where
        R: for<'a> AssertionRenderer<Vec<&'a T>>,
    {
        self.track_assertion();
        if self.actual().into_iter().count() != 0 {
            let actual = self.actual().into_iter().collect::<Vec<_>>();
            let actual = self.render_value(&actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    Is not empty!
                "}
            });
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
                assert_that!(iter).contains(&1);
            }

            #[test]
            fn compiles_for_comparable_but_different_type() {
                let values = vec!["foo"];
                assert_that!(values).into_iter_contains("foo".to_string());
            }
        }

        mod does_not_contain {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_value_is_absent() {
                let values = [1, 2, 3];
                let iter = values.iter();
                assert_that!(iter).does_not_contain(&4);
            }

            #[test]
            fn panics_when_value_is_present() {
                assert_that_panic_by(|| {
                    let values = [1, 2, 3];
                    let iter = values.iter();
                    assert_that!(iter).with_location(false).does_not_contain(&2);
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                        -------- assertr --------
                        Actual: [
                            1,
                            2,
                            3,
                        ]

                        contains unexpected: 2
                        -------- assertr --------
                    "});
            }
        }
    }

    mod into_iterator_assertions {

        mod contains {
            use crate::prelude::*;

            #[test]
            fn succeeds_when_value_is_present() {
                let values = vec![1, 2, 3, 42];
                assert_that!(values)
                    .into_iter_contains(1)
                    .into_iter_contains(42)
                    .into_iter_contains(3)
                    .into_iter_contains(2);
            }

            #[test]
            fn compiles_for_comparable_but_different_type() {
                let values = vec!["foo"];
                assert_that!(values).into_iter_contains("foo".to_string());
            }
        }

        mod does_not_contain {
            use crate::prelude::*;
            use indoc::formatdoc;

            #[test]
            fn succeeds_when_value_is_absent() {
                let values = vec![1, 2, 3, 42];
                assert_that!(values)
                    .into_iter_does_not_contain(5)
                    .into_iter_does_not_contain(99);
            }

            #[test]
            fn compiles_for_comparable_but_different_type() {
                let values = vec!["foo"];
                assert_that!(values).into_iter_does_not_contain("bar".to_string());
            }

            #[test]
            fn panics_when_value_is_present() {
                assert_that_panic_by(|| {
                    let values = vec![1, 2, 3];
                    assert_that!(values)
                        .with_location(false)
                        .into_iter_does_not_contain(2);
                })
                .has_type::<String>()
                .is_equal_to(formatdoc! {"
                        -------- assertr --------
                        Actual: [
                            1,
                            2,
                            3,
                        ]

                        contains unexpected: 2
                        -------- assertr --------
                    "});
            }
        }
    }
}
