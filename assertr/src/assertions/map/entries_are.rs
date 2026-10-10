use super::entry::key_segment;
use crate::{
    assertions::map::{EntryMatcherList, FoundEntries, Map},
    expectation::{AssertionContext, Evidence, Expectation, composite_items},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::ValueRenderer,
};

/// Exact keyed matching. Duplicate queries cannot replace a missing distinct entry.
#[derive(Debug, Clone)]
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
    R: ValueRenderer<MapType::Key> + ValueRenderer<usize>,
    L: EntryMatcherList<MapType, R>,
{
    composite_items!(MapType);
    fn evaluate(
        &self,
        actual: &MapType,
        settings: &AssertionContext<'_, R>,
    ) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let mut entries_match = true;
        let mut found = FoundEntries::new();
        for index in 0..self.0.len() {
            let (entry_matches, key) = self.0.evaluate_entry_at(index, actual, &mut context);
            entries_match &= entry_matches;
            if let Some(key) = key {
                found.record(key);
            }
        }
        let mut extras = context.isolated_for_order(MapType::RENDERING_ORDER);
        for (key, _) in found.unexpected_entries(actual) {
            entries_match = false;
            extras.record(|context| {
                FailureBuilder::new::<MapType>(FailureKind::Matching)
                    .path([key_segment(context.render(), key)])
                    .relation("has an unexpected key")
                    .build()
            });
        }
        context.append(extras.into_evidence());
        // Distinct expected keys make every length difference a missing or unexpected key. Only
        // duplicate queries can match every entry while the lengths differ.
        let (actual_length, expected_length) = (actual.length(), self.0.len());
        if entries_match && actual_length != expected_length {
            context.record(|context| {
                let render = context.render();
                FailureBuilder::new::<MapType>(FailureKind::Length)
                    .relation("does not have the required number of entries")
                    .fact(Fact::labelled(
                        "Actual length",
                        render.value(&actual_length),
                    ))
                    .fact(Fact::labelled(
                        "Expected length",
                        render.value(&expected_length),
                    ))
                    .build()
            });
        }
        let matched = entries_match && actual_length == expected_length;
        context.finish(matched, |context| context.describe::<MapType, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&MapType, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => context.describe_list::<MapType, _>(
                &self.0,
                failure.relation("has exactly the matching entries"),
            ),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}

/// Exact keyed matching with explicit value expectations.
///
/// Keys are lookup operands. Use [`eq`](crate::matchers::eq) or
/// [`eq`](crate::matchers::eq) for value equality.
#[macro_export]
macro_rules! entries_are {
    (@list) => {
        $crate::__private::Nil
    };
    (@list ($key:expr, $value:expr) $(, ($tail_key:expr, $tail_value:expr))* $(,)?) => {
        $crate::__private::Cons(
            $crate::matchers::entry($key, $value),
            $crate::entries_are!(@list $(($tail_key, $tail_value)),*)
        )
    };
    ($(($key:expr, $value:expr)),* $(,)?) => {
        $crate::matchers::entries_are($crate::entries_are!(@list $(($key, $value)),*))
    };
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use crate::{matchers::eq, prelude::*, test_support::UnorderedMap};

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
            let expected: [matchers::Entry<&str, matchers::EqualTo<i32>>; 0] = [];
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
        assert_that!(failures[0].children).has_length(1);
        assert_that!(failures[0].children[0].relation.as_deref())
            .is_equal_to(Some("has an unexpected key"));
    }

    #[test]
    fn length_differences_count_as_omitted_evidence_at_zero_budget() {
        let failures = assert_that!(BTreeMap::from([(1, 1)]))
            .with_rendering_budget(RenderingBudget::default().with_max_items(0))
            .capture(|it| it.matches(entries_are![(1, eq(1)), (1, eq(1))]));

        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children).is_empty();
        assert_that!(failures[0].omitted_children).is_equal_to(1);
    }

    #[test]
    fn limits_repeated_value_evidence_to_the_rendering_budget() {
        let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3),]))
            .with_rendering_budget(RenderingBudget::default().with_max_items(1))
            .with_location(false)
            .capture(|it| it.matches(entries_are![("a", eq(0)), ("b", eq(0)), ("c", eq(0))]));

        assert_that!(failures[0].children).has_length(1);
        assert_that!(failures[0].children[0].path).contains_exactly_matching([
            pattern!(crate::failure::PathSegment::Key(key) if format!("{key:#}") == "\"a\""),
        ]);
        assert_that!(failures[0].omitted_children).is_equal_to(2);
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
        assert_that!(actual[0].to_string()).is_equal_to(expected[0].to_string());
    }
}
