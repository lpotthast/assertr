use super::{SetLookup, imp};
use crate::{AssertThat, Mode, renderer::DebugRenderer, renderer::ValueRenderer};

/// The set relations: subset, superset, and disjointness.
///
/// Other element assertions come from
/// [`CollectionAssertions`](crate::assertions::CollectionAssertions).
///
/// Every relation accepts any other set type, so a `HashSet` can be compared against a `BTreeSet`,
/// and against a `HashSet` with a different hasher.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait SetAssertions<T, R = DebugRenderer> {
    /// Asserts that every subject element belongs to `expected_superset`.
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: SetLookup<Item = T>,
        R: ValueRenderer<T>;

    /// Asserts that every element of `expected_subset` belongs to the subject.
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: SetLookup<Item = T>,
        R: ValueRenderer<T>;

    /// Asserts that the subject and `other` share no element.
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: SetLookup<Item = T>,
        R: ValueRenderer<T>;
}

impl<S, M, R> SetAssertions<S::Item, R> for AssertThat<'_, S, M, R>
where
    S: SetLookup,
    M: Mode,
{
    #[track_caller]
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: SetLookup<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        self.matches(imp::IsSubsetOf::new(expected_superset))
    }

    #[track_caller]
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: SetLookup<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        self.matches(imp::IsSupersetOf::new(expected_subset))
    }

    #[track_caller]
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: SetLookup<Item = S::Item>,
        R: ValueRenderer<S::Item>,
    {
        self.matches(imp::IsDisjointFrom::new(other))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;
        use alloc::collections::BTreeSet;

        #[test]
        fn are_as_expected() {
            let set = BTreeSet::from(["foo"]);
            set.must().be_subset_of(BTreeSet::from(["foo", "bar"]));
            BTreeSet::from(["foo", "bar"])
                .must()
                .be_superset_of(set.clone());
            set.must().be_disjoint_from(BTreeSet::from(["bar"]));
        }
    }

    mod renderer_contract {
        use alloc::collections::BTreeSet;

        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, BTreeSet<i32>, Panic, NoRenderer>
                    => SetAssertions<i32, NoRenderer>
            );
        }
    }

    /// Every relation accepts hash sets, borrowed sets, other hashers, and other set types.
    #[cfg(feature = "std")]
    mod set_types {
        use alloc::collections::BTreeSet;
        use std::collections::{HashSet, hash_map::RandomState};
        use std::hash::{BuildHasherDefault, DefaultHasher};

        use crate::prelude::*;

        #[test]
        fn relations_accept_any_set_type() {
            let small: HashSet<&str, RandomState> = HashSet::from(["foo"]);
            let mut large: HashSet<&str, BuildHasherDefault<DefaultHasher>> =
                HashSet::with_hasher(BuildHasherDefault::default());
            large.extend(["foo", "bar"]);
            let other = BTreeSet::from(["baz"]);
            assert_that!(&small)
                .is_subset_of(&large)
                .is_disjoint_from(&other);
            assert_that!(&large)
                .is_superset_of(&small)
                .is_superset_of(BTreeSet::from(["bar"]));
            assert_that!(other)
                .is_subset_of(HashSet::from(["baz", "qux"]))
                .is_disjoint_from(small);
        }
    }

    mod is_subset_of {
        use alloc::collections::BTreeSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeSet::from(["bar"])),
                is_subset_of(BTreeSet::<&str>::new())
            );
        }

        #[test]
        fn succeeds_when_actual_is_subset() {
            assert_that!(BTreeSet::from(["foo"])).is_subset_of(BTreeSet::from(["foo", "bar"]));
        }

        #[test]
        fn panics_with_the_elements_not_in_expected() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::from(["bar"]))
                    .with_location(false)
                    .is_subset_of(BTreeSet::<&str>::new());
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeSet::from(["bar"])`

                    Actual: BTreeSet {{
                        "bar",
                    }}

                    is not a subset of

                    Expected: BTreeSet {{}}

                    Details:
                      - Elements not in expected: [
                            "bar",
                        ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_superset_of {
        use alloc::collections::BTreeSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeSet::<&str>::new()),
                is_superset_of(BTreeSet::from(["bar"]))
            );
        }

        #[test]
        fn succeeds_when_actual_is_superset() {
            assert_that!(BTreeSet::from(["foo", "bar"])).is_superset_of(BTreeSet::from(["foo"]));
        }

        #[test]
        fn panics_with_the_elements_not_in_actual() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::<&str>::new())
                    .with_location(false)
                    .is_superset_of(BTreeSet::from(["bar"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeSet::<&str>::new()`

                    Actual: BTreeSet {{}}

                    is not a superset of

                    Expected: BTreeSet {{
                        "bar",
                    }}

                    Details:
                      - Elements not in actual: [
                            "bar",
                        ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_disjoint_from {
        use alloc::collections::BTreeSet;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeSet::from(["foo"])),
                is_disjoint_from(BTreeSet::from(["foo"]))
            );
        }

        #[test]
        fn succeeds_when_sets_are_disjoint() {
            assert_that!(BTreeSet::from(["foo"])).is_disjoint_from(BTreeSet::from(["bar"]));
        }

        #[test]
        fn panics_with_the_overlapping_elements() {
            assert_that_panic_by(|| {
                assert_that!(BTreeSet::from(["foo"]))
                    .with_location(false)
                    .is_disjoint_from(BTreeSet::from(["foo"]));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeSet::from(["foo"])`

                    Actual: BTreeSet {{
                        "foo",
                    }}

                    is not disjoint from

                    Expected: BTreeSet {{
                        "foo",
                    }}

                    Details:
                      - Overlapping elements: [
                            "foo",
                        ]
                    -------- assertr --------
                "#});
        }
    }
}
