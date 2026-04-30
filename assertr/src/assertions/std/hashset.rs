use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::Write;
use indoc::writedoc;
use std::{
    collections::HashSet,
    hash::{BuildHasher, Hash},
};

use crate::{AssertThat, AssertionRenderer, AssertrPartialEq, Mode, tracking::AssertionTracking};

/// Assertions for generic [`HashSet`]s.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HashSetAssertions<T, S, R> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<E>;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<E>;

    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: AssertrPartialEq<E, R>,
        I: IntoIterator<Item = E>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<[E]> + AssertionRenderer<E>;

    fn is_subset_of<S2>(self, expected_superset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>;

    fn is_superset_of<S2>(self, expected_subset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>;

    fn is_disjoint_from<S2>(self, other: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>;
}

impl<T, S: BuildHasher, M: Mode, R> HashSetAssertions<T, S, R>
    for AssertThat<'_, HashSet<T, S>, M, R>
{
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<E>,
    {
        self.track_assertion();

        if !self.actual().iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(self.actual());
            let expected = self.render_value(&expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    does not contain expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E, R>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<E>,
    {
        self.track_assertion();

        if self.actual().iter().any(|it| {
            let mut ctx = self.eq_context();
            <_ as AssertrPartialEq<_, R>>::eq(it, &not_expected, Some(&mut ctx))
        }) {
            let actual = self.render_value(self.actual());
            let not_expected = self.render_value(&not_expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    contains unexpected: {not_expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: AssertrPartialEq<E, R>,
        I: IntoIterator<Item = E>,
        R: AssertionRenderer<HashSet<T, S>> + AssertionRenderer<[E]> + AssertionRenderer<E>,
    {
        self.track_assertion();

        let expected = expected.into_iter().collect::<Vec<_>>();
        let elements_not_found = expected
            .iter()
            .filter(|expected| {
                !self.actual().iter().any(|actual| {
                    let mut ctx = self.eq_context();
                    <_ as AssertrPartialEq<_, R>>::eq(actual, expected, Some(&mut ctx))
                })
            })
            .collect::<Vec<_>>();

        if !elements_not_found.is_empty() {
            let actual = self.render_value(self.actual());
            let expected_rendered = self.render_value(expected.as_slice());
            let elements_rendered = self.render_values(elements_not_found.as_slice());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    does not contain all expected elements

                    Expected: {expected_rendered:#?}

                    Elements not found: {elements_rendered:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_subset_of<S2>(self, expected_superset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>,
    {
        self.track_assertion();

        let expected_superset = expected_superset.borrow();
        let elements_not_in_expected = self
            .actual()
            .iter()
            .filter(|actual| !expected_superset.contains(*actual))
            .collect::<Vec<_>>();

        if !elements_not_in_expected.is_empty() {
            let actual = self.render_value(self.actual());
            let expected_superset_rendered = self.render_value(expected_superset);
            let elements_rendered = self.render_values(elements_not_in_expected.as_slice());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not a subset of expected

                    Expected superset: {expected_superset_rendered:#?}

                    Elements not in expected: {elements_rendered:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_superset_of<S2>(self, expected_subset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>,
    {
        self.track_assertion();

        let expected_subset = expected_subset.borrow();
        let elements_not_in_actual = expected_subset
            .iter()
            .filter(|expected| !self.actual().contains(*expected))
            .collect::<Vec<_>>();

        if !elements_not_in_actual.is_empty() {
            let actual = self.render_value(self.actual());
            let expected_subset_rendered = self.render_value(expected_subset);
            let elements_rendered = self.render_values(elements_not_in_actual.as_slice());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not a superset of expected

                    Expected subset: {expected_subset_rendered:#?}

                    Elements not in actual: {elements_rendered:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_disjoint_from<S2>(self, other: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash,
        S2: BuildHasher,
        R: AssertionRenderer<HashSet<T, S>>
            + AssertionRenderer<HashSet<T, S2>>
            + AssertionRenderer<T>,
    {
        self.track_assertion();

        let other = other.borrow();
        let overlapping_elements = self
            .actual()
            .iter()
            .filter(|actual| other.contains(*actual))
            .collect::<Vec<_>>();

        if !overlapping_elements.is_empty() {
            let actual = self.render_value(self.actual());
            let other_rendered = self.render_value(other);
            let elements_rendered = self.render_values(overlapping_elements.as_slice());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not disjoint from expected

                    Expected disjoint set: {other_rendered:#?}

                    Overlapping elements: {elements_rendered:#?}
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains {
        use std::collections::HashSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_present() {
            assert_that!(HashSet::from(["foo"])).contains("foo");
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(HashSet::from(["foo"])).contains("foo".to_owned());
            assert_that!(HashSet::from(["foo".to_owned()])).contains("foo");
        }

        #[test]
        fn panics_when_expected_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["foo"]))
                    .with_location(false)
                    .contains("bar");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "foo",
                    }}

                    does not contain expected: "bar"
                    -------- assertr --------
                "#});
        }
    }

    mod does_not_contain {
        use std::collections::HashSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_unexpected_is_absent() {
            assert_that!(HashSet::from(["foo"])).does_not_contain("bar");
        }

        #[test]
        fn panics_when_unexpected_is_present() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["foo"]))
                    .with_location(false)
                    .does_not_contain("foo");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "foo",
                    }}

                    contains unexpected: "foo"
                    -------- assertr --------
                "#});
        }
    }

    mod contains_all {
        use std::collections::HashSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_all_expected_values_are_present() {
            assert_that!(HashSet::from(["foo", "bar"])).contains_all(["foo", "bar"]);
        }

        #[test]
        fn succeeds_with_vec_input() {
            assert_that!(HashSet::from(["foo", "bar"])).contains_all(vec!["foo", "bar"]);
        }

        #[test]
        fn compiles_for_comparable_but_different_type() {
            assert_that!(HashSet::from(["foo"])).contains_all(["foo".to_owned()]);
            assert_that!(HashSet::from(["foo".to_owned()])).contains_all(["foo"]);
        }

        #[test]
        fn panics_when_any_expected_value_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["foo"]))
                    .with_location(false)
                    .contains_all(["foo", "bar"]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "foo",
                    }}

                    does not contain all expected elements

                    Expected: [
                        "foo",
                        "bar",
                    ]

                    Elements not found: [
                        "bar",
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_subset_of {
        use std::collections::{HashSet, hash_map::RandomState};
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_actual_is_subset() {
            assert_that!(HashSet::from(["foo"])).is_subset_of(HashSet::from(["foo", "bar"]));
        }

        #[test]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo"]);
            let expected = HashSet::from(["foo", "bar"]);

            assert_that!(&actual).is_subset_of(&expected);
        }

        #[test]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("foo");
            expected.insert("bar");

            assert_that!(actual).is_subset_of(expected);
        }

        #[test]
        fn panics_when_actual_contains_extra_elements() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["bar"]))
                    .with_location(false)
                    .is_subset_of(HashSet::<&str>::new());
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "bar",
                    }}

                    is not a subset of expected

                    Expected superset: {{}}

                    Elements not in expected: [
                        "bar",
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_superset_of {
        use std::collections::{HashSet, hash_map::RandomState};
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_actual_is_superset() {
            assert_that!(HashSet::from(["foo", "bar"])).is_superset_of(HashSet::from(["foo"]));
        }

        #[test]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo", "bar"]);
            let expected = HashSet::from(["foo"]);

            assert_that!(&actual).is_superset_of(&expected);
        }

        #[test]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo", "bar"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("foo");

            assert_that!(actual).is_superset_of(expected);
        }

        #[test]
        fn panics_when_actual_is_missing_elements() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::<&str>::new())
                    .with_location(false)
                    .is_superset_of(HashSet::from(["bar"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{}}

                    is not a superset of expected

                    Expected subset: {{
                        "bar",
                    }}

                    Elements not in actual: [
                        "bar",
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_disjoint_from {
        use std::collections::{HashSet, hash_map::RandomState};
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_sets_are_disjoint() {
            assert_that!(HashSet::from(["foo"])).is_disjoint_from(HashSet::from(["bar"]));
        }

        #[test]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo"]);
            let expected = HashSet::from(["bar"]);

            assert_that!(&actual).is_disjoint_from(&expected);
        }

        #[test]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("bar");

            assert_that!(actual).is_disjoint_from(expected);
        }

        #[test]
        fn panics_when_sets_overlap() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["foo"]))
                    .with_location(false)
                    .is_disjoint_from(HashSet::from(["foo"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "foo",
                    }}

                    is not disjoint from expected

                    Expected disjoint set: {{
                        "foo",
                    }}

                    Overlapping elements: [
                        "foo",
                    ]
                    -------- assertr --------
                "#});
        }
    }
}
