use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;
use std::{
    collections::HashSet,
    hash::{BuildHasher, Hash},
};

use crate::{AssertThat, AssertrPartialEq, Mode, tracking::AssertionTracking};

/// Assertions for generic [`HashSet`]s.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HashSetAssertions<T> {
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
        I: IntoIterator<Item = E>;

    fn is_subset_of<S2>(self, expected_superset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher;

    fn is_superset_of<S2>(self, expected_subset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher;

    fn is_disjoint_from<S2>(self, other: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher;
}

impl<T, S: BuildHasher, M: Mode> HashSetAssertions<T> for AssertThat<'_, HashSet<T, S>, M> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        if !self
            .actual()
            .iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    does not contain expected: {expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn does_not_contain<E>(self, not_expected: E) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        if self
            .actual()
            .iter()
            .any(|it| AssertrPartialEq::eq(it, &not_expected, None))
        {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    contains unexpected: {not_expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains_all<E, I>(self, expected: I) -> Self
    where
        T: AssertrPartialEq<E> + Debug,
        E: Debug,
        I: IntoIterator<Item = E>,
    {
        self.track_assertion();

        let expected = expected.into_iter().collect::<Vec<_>>();
        let elements_not_found = expected
            .iter()
            .filter(|expected| {
                !self
                    .actual()
                    .iter()
                    .any(|actual| AssertrPartialEq::eq(actual, expected, None))
            })
            .collect::<Vec<_>>();

        if !elements_not_found.is_empty() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    does not contain all expected elements

                    Expected: {expected:#?}

                    Elements not found: {elements_not_found:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_subset_of<S2>(self, expected_superset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher,
    {
        self.track_assertion();

        let expected_superset = expected_superset.borrow();
        let elements_not_in_expected = self
            .actual()
            .iter()
            .filter(|actual| !expected_superset.contains(*actual))
            .collect::<Vec<_>>();

        if !elements_not_in_expected.is_empty() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not a subset of expected

                    Expected superset: {expected_superset:#?}

                    Elements not in expected: {elements_not_in_expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_superset_of<S2>(self, expected_subset: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher,
    {
        self.track_assertion();

        let expected_subset = expected_subset.borrow();
        let elements_not_in_actual = expected_subset
            .iter()
            .filter(|expected| !self.actual().contains(*expected))
            .collect::<Vec<_>>();

        if !elements_not_in_actual.is_empty() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not a superset of expected

                    Expected subset: {expected_subset:#?}

                    Elements not in actual: {elements_not_in_actual:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_disjoint_from<S2>(self, other: impl Borrow<HashSet<T, S2>>) -> Self
    where
        T: Eq + Hash + Debug,
        S2: BuildHasher,
    {
        self.track_assertion();

        let other = other.borrow();
        let overlapping_elements = self
            .actual()
            .iter()
            .filter(|actual| other.contains(*actual))
            .collect::<Vec<_>>();

        if !overlapping_elements.is_empty() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashSet {actual:#?}

                    is not disjoint from expected

                    Expected disjoint set: {other:#?}

                    Overlapping elements: {overlapping_elements:#?}
                ", actual = self.actual()}
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
