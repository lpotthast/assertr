use super::entry_matcher_list::sealed as entry_list_sealed;
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    assertions::map::{EntryMatcherList, FoundEntries, Map},
    expectation::{Evidence, MatcherList, lists::sealed as list_sealed},
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};

/// Exact keyed matching. Duplicate queries cannot replace a missing distinct entry.
pub struct EntriesAre<L>(L);

/// Applies an exact keyed matcher list. Use arrays, slices, or vectors of [`Entry`](super::Entry)
/// values, or `entries_are!` for heterogeneous entries.
///
/// ```
/// use assertr::{matchers::{entries_are, entry, eq}, prelude::*};
/// use std::collections::BTreeMap;
///
/// let expected = [entry("key", eq(1))];
/// assert_that!(BTreeMap::from([("key", 1)])).matches(entries_are(&expected[..]));
/// ```
pub fn entries_are<L>(list: L) -> EntriesAre<L> {
    EntriesAre(list)
}

impl<MapType: Map + ?Sized, R, L> Expectation<MapType, R> for EntriesAre<L>
where
    R: crate::ValueRenderer<MapType::Key>,
    L: EntryMatcherList<MapType, R>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        MapType: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        MapType: 'a;
    fn evaluate(
        &self,
        actual: &MapType,
        settings: &AssertionContext<'_, R>,
    ) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let mut matched = actual.length() == self.0.len();
        let mut found = FoundEntries::new();
        for index in 0..self.0.len() {
            let (entry_matches, key) = self.0.evaluate_entry_at(index, actual, &mut context);
            matched &= entry_matches;
            if let Some(key) = key {
                found.record(key);
            }
        }
        let mut extras = context.isolated_for_order(MapType::RENDERING_ORDER);
        for (key, _) in actual.entries() {
            if !found.contains(key) {
                matched = false;
                if extras.is_diagnostic() {
                    extras.record(
                        FailureBuilder::detached::<MapType>(FailureKind::Matching)
                            .path([PathSegment::Key(extras.render().value(key).into_rendered())])
                            .relation("has an unexpected key")
                            .build(),
                    );
                } else {
                    extras.outcome(false, |_| {
                        FailureBuilder::detached::<()>(FailureKind::Matching)
                            .relation("has an unexpected key")
                            .build()
                    });
                }
            }
        }
        context.append(extras.into_evidence());
        if matched {
            Ok(())
        } else {
            if context.evidence.is_empty() {
                context.outcome(false, |context| context.describe::<MapType, _>(self));
            }
            Err(context.into_evidence())
        }
    }
}
impl<MapType: Map + ?Sized, R, L> ExpectationDiagnostics<MapType, R> for EntriesAre<L>
where
    R: ValueRenderer<MapType::Key>,
    L: EntryMatcherList<MapType, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&MapType, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => context.describe_list::<MapType, _, _>(
                &self.0,
                failure.relation("has exactly the matching entries"),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

impl<L> entry_list_sealed::Sealed for EntriesAre<L> {}

impl<L> list_sealed::Sealed for EntriesAre<L> {}

impl<MapType, R, L> MatcherList<MapType, R> for EntriesAre<L>
where
    MapType: Map + ?Sized,
    L: EntryMatcherList<MapType, R>,
{
    fn len(&self) -> usize {
        self.0.len()
    }

    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure {
        self.0.describe_at(index, context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> bool {
        self.0.evaluate_at(index, actual, context)
    }
}

impl<MapType, R, L> EntryMatcherList<MapType, R> for EntriesAre<L>
where
    MapType: Map + ?Sized,
    L: EntryMatcherList<MapType, R>,
{
    fn evaluate_entry_at<'a>(
        &'a self,
        index: usize,
        actual: &'a MapType,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a MapType::Key>) {
        self.0.evaluate_entry_at(index, actual, context)
    }
}

/// Exact keyed matching with explicit value expectations.
///
/// Keys are lookup operands. Use [`eq`](crate::matchers::eq) or
/// [`equal_to`](crate::matchers::equal_to) for value equality.
#[macro_export]
macro_rules! entries_are {
    (@list) => {
        $crate::__private::Nil
    };
    (@list ($key:expr, $value:expr) $(, ($tail_key:expr, $tail_value:expr))* $(,)?) => {
        $crate::__private::Cons(
            $crate::assertions::map::entry($key, $value),
            $crate::entries_are!(@list $(($tail_key, $tail_value)),*)
        )
    };
    ($(($key:expr, $value:expr)),* $(,)?) => {
        $crate::assertions::map::entries_are($crate::entries_are!(@list $(($key, $value)),*))
    };
}

#[cfg(test)]
mod tests {
    use crate::{matchers::eq, prelude::*, test_support::UnorderedMap};
    use alloc::collections::BTreeMap;

    mod homogeneous_lists {
        use super::*;
        use crate::matchers::{entries_are, entry, eq};

        #[test]
        fn arrays_and_slices_preserve_vector_matching_and_diagnostics() {
            let actual = BTreeMap::from([("a", 1), ("b", 2)]);
            let expected = [entry("a", eq(1)), entry("b", eq(2))];
            assert_that!(actual).matches(entries_are(&expected[..]));
            assert_that!(actual).matches(entries_are(expected));

            let expected = [entry("a", eq(9)), entry("c", eq(3))];
            let slice_failures = assert_that!(actual)
                .with_location(false)
                .capture(|it| it.matches(entries_are(&expected[..])));
            let array_failures = assert_that!(actual)
                .with_location(false)
                .capture(|it| it.matches(entries_are(expected)));
            let vector_failures = assert_that!(actual).with_location(false).capture(|it| {
                it.matches(entries_are(alloc::vec![
                    entry("a", eq(9)),
                    entry("c", eq(3))
                ]))
            });
            assert_that!(slice_failures).has_length(1);
            assert_that!(slice_failures).is_equal_to(vector_failures.clone());
            assert_that!(array_failures).is_equal_to(vector_failures);
        }

        #[test]
        fn empty_arrays_and_slices_require_an_empty_map() {
            let expected: [crate::matchers::Entry<&str, crate::matchers::EqualTo<i32>>; 0] = [];
            let empty = BTreeMap::<&str, i32>::new();
            assert_that!(empty).matches(entries_are(&expected[..]));
            assert_that!(empty).matches(entries_are(&expected));
            let failures = assert_that!(BTreeMap::from([("a", 1)]))
                .capture(|it| it.matches(entries_are(expected)));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].relation.as_deref())
                .is_equal_to(Some("has an unexpected key"));
        }
    }

    #[test]
    fn duplicate_keys_do_not_hide_unexpected_entries() {
        let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
            .capture(|it| it.matches(entries_are![("a", eq(1)), ("a", eq(1))]));

        assert_that!(failures).has_length(1);
    }

    mod retained_lookup {
        use super::*;
        use crate::{
            assertions::{
                HasLength,
                map::{Map, MapKeyQuery, MapLookup},
            },
            failure::{FailureKind, PathSegment},
            renderer::RenderingOrder,
        };
        use core::cell::Cell;

        struct ObservedMap {
            entries: [(u32, i32); 2],
            lookups: Cell<usize>,
        }
        impl HasLength for ObservedMap {
            fn length(&self) -> usize {
                self.entries.len()
            }
        }
        impl Map for ObservedMap {
            type Key = u32;
            type Value = i32;
            const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;
            fn entries(&self) -> impl Iterator<Item = (&u32, &i32)> {
                self.entries.iter().map(|(key, value)| (key, value))
            }
        }
        impl MapLookup<u32> for ObservedMap {
            fn get_key_value(&self, query: &u32) -> Option<(&u32, &i32)> {
                self.lookups.set(self.lookups.get() + 1);
                self.entries().find(|(key, _)| *key == query)
            }
        }

        #[derive(Debug)]
        struct Query<'a>(&'a Cell<usize>);
        impl MapKeyQuery<u32> for Query<'_> {
            type Query = u32;
            fn as_query(&self) -> &u32 {
                let previous = self.0.get();
                self.0.set(previous + 1);
                if previous == 0 { &1 } else { &2 }
            }
        }

        #[test]
        fn converts_and_looks_up_once_even_when_the_value_rejects() {
            for expected in [1, 99] {
                let conversions = Cell::new(0);
                let actual = ObservedMap {
                    entries: [(1, 1), (2, 2)],
                    lookups: Cell::new(0),
                };
                let failures = assert_that!(actual).capture(|it| {
                    it.matches(entries_are![
                        (Query(&conversions), eq(expected)),
                        (2_u32, eq(2))
                    ])
                });

                assert_that!(conversions.get()).is_equal_to(1);
                assert_that!(actual.lookups.get()).is_equal_to(2);
                if expected == 1 {
                    assert_that!(failures).is_empty();
                } else {
                    assert_that!(failures).has_length(1);
                    // The rejected value occupies its original key. There is no unexpected key.
                    assert_that!(failures[0].children).has_length(1);
                    let child = &failures[0].children[0];
                    assert_that!(child.kind).is_equal_to(FailureKind::Equality);
                    assert_that!(child.path)
                        .contains_exactly_matching([pattern!(PathSegment::Key(_))]);
                }
            }
        }
    }

    #[test]
    fn sorts_unexpected_keys_before_limiting_them() {
        let capture = |entries| {
            let actual = UnorderedMap(entries);
            assert_that!(actual)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .capture(|it| it.matches(entries_are![]))
        };
        let expected = capture(vec![(1, 0), (2, 0), (3, 0)]);
        let actual = capture(vec![(3, 0), (2, 0), (1, 0)]);

        assert_that!(actual[0].children).has_length(1);
        assert_that!(actual[0].omitted_children).is_equal_to(2);
        assert_that!(ToHumanReadableText.render(&actual[0]))
            .is_equal_to(ToHumanReadableText.render(&expected[0]));
    }
}
