use super::{SetLookup, imp};
use crate::{
    AssertThat, Mode,
    assertions::collection::Collection,
    expectation::ExpectedOperand,
    renderer::{DebugRenderer, ValueRenderer},
};

/// The set relations: subset, superset, and disjointness.
///
/// Other element assertions come from
/// [`CollectionAssertions`](crate::assertions::CollectionAssertions).
///
/// The other side of every relation can be any collection: another set of any kind, an array, a
/// slice, or a vector. Its elements select a borrowed view of the subject's element type through
/// [`BorrowFor`](crate::borrow_for::BorrowFor), like other expected values, and are looked up in
/// the subject through its native [`SetLookup`]. So a `HashSet<String>` can be compared against
/// `["read", "write"]`, and a `HashSet` against a `BTreeSet`.
///
/// ```
/// use std::collections::{BTreeSet, HashSet};
///
/// use assertr::prelude::*;
///
/// # #[cfg(feature = "std")] {
/// let granted = HashSet::from([String::from("read"), String::from("write")]);
/// assert_that!(&granted)
///     .is_subset_of(["read", "write", "admin"])
///     .is_superset_of(BTreeSet::from(["read"]))
///     .is_disjoint_from(vec!["delete"]);
///
/// let failures = assert_that!(granted)
///     .with_location(false)
///     .capture(|it| it.is_disjoint_from(["write"]));
/// assert_that!(failures[0].to_string())
///     .contains("is not disjoint from")
///     .contains("Overlapping elements");
/// # }
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait SetAssertions<T, R = DebugRenderer> {
    /// The subject set type. The relations require it to implement [`SetLookup`] for the query
    /// type the other side's elements select.
    type Set: Collection<Item = T> + ?Sized;

    /// Asserts that every subject element is named by an element of `expected_superset`.
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        Self::Set: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>;

    /// Asserts that every element of `expected_subset` belongs to the subject.
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        Self::Set: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>;

    /// Asserts that no element of `other` belongs to the subject.
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        Self::Set: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>;
}

// An explicit element parameter keeps the mutually dependent method bounds cycle-free.
impl<S, T, M, R> SetAssertions<T, R> for AssertThat<'_, S, M, R>
where
    S: Collection<Item = T> + SetLookup<T>,
    M: Mode,
{
    type Set = S;

    #[track_caller]
    fn is_subset_of<O>(self, expected_superset: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        S: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>,
    {
        self.matches(imp::IsSubsetOf::new(expected_superset))
    }

    #[track_caller]
    fn is_superset_of<O>(self, expected_subset: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        S: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>,
    {
        self.matches(imp::IsSupersetOf::new(expected_subset))
    }

    #[track_caller]
    fn is_disjoint_from<O>(self, other: O) -> Self
    where
        O: Collection,
        O::Item: ExpectedOperand<T>,
        S: SetLookup<<O::Item as ExpectedOperand<T>>::View>,
        R: ValueRenderer<T> + ValueRenderer<O::Item>,
    {
        self.matches(imp::IsDisjointFrom::new(other))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use alloc::collections::BTreeSet;

        use crate::prelude::*;

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

        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

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
        use std::{
            collections::{HashSet, hash_map::RandomState},
            hash::{BuildHasherDefault, DefaultHasher},
        };

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

        use indoc::formatdoc;

        use crate::prelude::*;

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
        fn accepts_list_operands_with_borrowed_views() {
            let set = BTreeSet::from([String::from("foo"), String::from("bar")]);
            assert_that!(set)
                .is_subset_of(["foo", "bar", "baz"])
                .is_subset_of(vec!["bar", "foo", "foo"]);
        }

        #[test]
        fn panics_with_the_elements_a_list_does_not_name() {
            assert_that!(|| {
                assert_that!(BTreeSet::from([1, 2, 3]))
                    .with_location(false)
                    .is_subset_of([3, 1]);
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `BTreeSet::from([1, 2, 3])`

                Actual: BTreeSet {{
                    1,
                    2,
                    3,
                }}

                is not a subset of

                Expected: [
                    3,
                    1,
                ]

                Details:
                  - Elements not in expected: [
                        2,
                    ]
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_with_the_elements_not_in_expected() {
            assert_that!(|| {
                assert_that!(BTreeSet::from(["bar"]))
                    .with_location(false)
                    .is_subset_of(BTreeSet::<&str>::new());
            })
            .panics()
            .message()
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

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeSet::<&str>::new()),
                is_superset_of(BTreeSet::from(["bar"]))
            );
        }

        #[test]
        fn accepts_list_operands_with_borrowed_views() {
            let set = BTreeSet::from([String::from("foo"), String::from("bar")]);
            assert_that!(set)
                .is_superset_of(["foo"])
                .is_superset_of(vec!["bar", "bar"]);
            let failures = assert_that!(set).capture(|it| it.is_superset_of(["baz"]));
            assert_that!(failures[0].to_string())
                .contains("Elements not in actual: [\n        \"baz\",");
        }

        #[test]
        fn succeeds_when_actual_is_superset() {
            assert_that!(BTreeSet::from(["foo", "bar"])).is_superset_of(BTreeSet::from(["foo"]));
        }

        #[test]
        fn panics_with_the_elements_not_in_actual() {
            assert_that!(|| {
                assert_that!(BTreeSet::<&str>::new())
                    .with_location(false)
                    .is_superset_of(BTreeSet::from(["bar"]));
            })
            .panics()
            .message()
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

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeSet::from(["foo"])),
                is_disjoint_from(BTreeSet::from(["foo"]))
            );
        }

        #[test]
        fn accepts_list_operands_and_reports_each_overlap_once() {
            let set = BTreeSet::from([String::from("foo"), String::from("bar")]);
            assert_that!(set).is_disjoint_from(["baz", "qux"]);
            let failures = assert_that!(set).capture(|it| it.is_disjoint_from(["foo", "foo"]));
            assert_that!(failures[0].to_string())
                .contains("Overlapping elements: [\n        \"foo\",\n    ]");
        }

        #[test]
        fn succeeds_when_sets_are_disjoint() {
            assert_that!(BTreeSet::from(["foo"])).is_disjoint_from(BTreeSet::from(["bar"]));
        }

        #[test]
        fn panics_with_the_overlapping_elements() {
            assert_that!(|| {
                assert_that!(BTreeSet::from(["foo"]))
                    .with_location(false)
                    .is_disjoint_from(BTreeSet::from(["foo"]));
            })
            .panics()
            .message()
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
