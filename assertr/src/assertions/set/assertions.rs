use super::{Set, imp};
use crate::{AssertThat, Mode, ValueRenderer};

/// The set relations: subset, superset, and disjointness.
///
/// Other element assertions come from
/// [`CollectionAssertions`](crate::assertions::collection::CollectionAssertions).
///
/// Every relation accepts any other set type, so a `HashSet` can be
/// compared against a `BTreeSet`, and against a `HashSet` with a different hasher.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SetAssertions<T, R> {
    /// Asserts that every subject element belongs to `expected_superset`.
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: Set<Item = T>,
        R: ValueRenderer<T>;

    /// Asserts that every element of `expected_subset` belongs to the subject.
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: Set<Item = T>,
        R: ValueRenderer<T>;

    /// Asserts that the subject and `other` share no element.
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: Set<Item = T>,
        R: ValueRenderer<T>;
}

impl<S, M, R> SetAssertions<S::Item, R> for AssertThat<'_, S, M, R>
where
    S: Set,
    M: Mode,
{
    #[track_caller]
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: Set<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        imp::assert_is_subset_of(&self, &expected_superset);
        self
    }

    #[track_caller]
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: Set<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        imp::assert_is_superset_of(&self, &expected_subset);
        self
    }

    #[track_caller]
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: Set<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        imp::assert_is_disjoint_from(&self, &other);
        self
    }
}

#[cfg(test)]
mod tests {
    mod is_subset_of {
        use alloc::collections::BTreeSet;
        #[cfg(feature = "std")]
        use std::collections::{HashSet, hash_map::RandomState};
        #[cfg(feature = "std")]
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let set = BTreeSet::from(["foo"]);
            set.must().be_subset_of(BTreeSet::from(["foo", "bar"]));
        }

        #[test]
        fn succeeds_when_actual_is_subset() {
            assert_that!(BTreeSet::from(["foo"])).is_subset_of(BTreeSet::from(["foo", "bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_for_hash_sets() {
            assert_that!(HashSet::from(["foo"])).is_subset_of(HashSet::from(["foo", "bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo"]);
            let expected = HashSet::from(["foo", "bar"]);

            assert_that!(&actual).is_subset_of(&expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("foo");
            expected.insert("bar");

            assert_that!(actual).is_subset_of(expected);
        }

        /// The generic signature compares any set against any other, not only sets of one type.
        #[test]
        #[cfg(feature = "std")]
        fn succeeds_across_set_types() {
            assert_that!(HashSet::from(["foo"])).is_subset_of(BTreeSet::from(["foo", "bar"]));
            assert_that!(BTreeSet::from(["foo"])).is_subset_of(HashSet::from(["foo", "bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn panics_when_actual_contains_extra_elements() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["bar"]))
                    .with_location(false)
                    .is_subset_of(BTreeSet::<&str>::new());
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "bar",
                    }}

                    is not a subset of expected

                    Expected superset: BTreeSet {{}}

                    Elements not in expected: [
                        "bar",
                    ]

                    Details: [
                        The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.,
                    ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_naming_the_btree_set_type() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::from(["bar"]))
                    .with_location(false)
                    .is_subset_of(BTreeSet::<&str>::new());
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: BTreeSet {{
                        "bar",
                    }}

                    is not a subset of expected

                    Expected superset: BTreeSet {{}}

                    Elements not in expected: [
                        "bar",
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_superset_of {
        use alloc::collections::BTreeSet;
        #[cfg(feature = "std")]
        use std::collections::{HashSet, hash_map::RandomState};
        #[cfg(feature = "std")]
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let set = BTreeSet::from(["foo", "bar"]);
            set.must().be_superset_of(BTreeSet::from(["foo"]));
        }

        #[test]
        fn succeeds_when_actual_is_superset() {
            assert_that!(BTreeSet::from(["foo", "bar"])).is_superset_of(BTreeSet::from(["foo"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_for_hash_sets() {
            assert_that!(HashSet::from(["foo", "bar"])).is_superset_of(HashSet::from(["foo"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo", "bar"]);
            let expected = HashSet::from(["foo"]);

            assert_that!(&actual).is_superset_of(&expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo", "bar"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("foo");

            assert_that!(actual).is_superset_of(expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_across_set_types() {
            assert_that!(HashSet::from(["foo", "bar"])).is_superset_of(BTreeSet::from(["foo"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn panics_when_actual_is_missing_elements() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::<&str>::new())
                    .with_location(false)
                    .is_superset_of(BTreeSet::from(["bar"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{}}

                    is not a superset of expected

                    Expected subset: BTreeSet {{
                        "bar",
                    }}

                    Elements not in actual: [
                        "bar",
                    ]

                    Details: [
                        The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.,
                    ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_naming_the_btree_set_type() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::<&str>::new())
                    .with_location(false)
                    .is_superset_of(BTreeSet::from(["bar"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: BTreeSet {{}}

                    is not a superset of expected

                    Expected subset: BTreeSet {{
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
        use alloc::collections::BTreeSet;
        #[cfg(feature = "std")]
        use std::collections::{HashSet, hash_map::RandomState};
        #[cfg(feature = "std")]
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let set = BTreeSet::from(["foo"]);
            set.must().be_disjoint_from(BTreeSet::from(["bar"]));
        }

        #[test]
        fn succeeds_when_sets_are_disjoint() {
            assert_that!(BTreeSet::from(["foo"])).is_disjoint_from(BTreeSet::from(["bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_for_hash_sets() {
            assert_that!(HashSet::from(["foo"])).is_disjoint_from(HashSet::from(["bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_borrowed_actual_and_expected_sets() {
            let actual = HashSet::from(["foo"]);
            let expected = HashSet::from(["bar"]);

            assert_that!(&actual).is_disjoint_from(&expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_with_different_hashers() {
            let actual: HashSet<&str, RandomState> = HashSet::from(["foo"]);
            let mut expected: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            expected.insert("bar");

            assert_that!(actual).is_disjoint_from(expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn succeeds_across_set_types() {
            assert_that!(HashSet::from(["foo"])).is_disjoint_from(BTreeSet::from(["bar"]));
        }

        #[test]
        #[cfg(feature = "std")]
        fn panics_when_sets_overlap() {
            assert_that_panic_by(|| {
                assert_that!(HashSet::from(["foo"]))
                    .with_location(false)
                    .is_disjoint_from(BTreeSet::from(["foo"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashSet {{
                        "foo",
                    }}

                    is not disjoint from expected

                    Expected disjoint set: BTreeSet {{
                        "foo",
                    }}

                    Overlapping elements: [
                        "foo",
                    ]

                    Details: [
                        The sets have different types, but cross-type relations are supported. This assertion failed based on their elements.,
                    ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_naming_the_btree_set_type() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::from(["foo"]))
                    .with_location(false)
                    .is_disjoint_from(BTreeSet::from(["foo"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: BTreeSet {{
                        "foo",
                    }}

                    is not disjoint from expected

                    Expected disjoint set: BTreeSet {{
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
