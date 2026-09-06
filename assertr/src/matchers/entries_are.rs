use super::{
    AssertrMatcher, Description, EntryMatcherList, MatchContext, MatchResult, MatcherList,
    entry_matcher_list::sealed as entry_list_sealed, lists::sealed as list_sealed,
};
use crate::{
    ValueRenderer,
    assertions::map::{FoundEntries, Map},
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};

/// Exact keyed matching. Duplicate queries cannot replace a missing distinct entry.
pub struct EntriesAre<L>(L);

/// Applies an exact keyed matcher list. Prefer `entries_are!` for heterogeneous values.
pub fn entries_are<L>(list: L) -> EntriesAre<L> {
    EntriesAre(list)
}

impl<MapType, R, L> AssertrMatcher<MapType, R> for EntriesAre<L>
where
    MapType: Map + ?Sized,
    L: EntryMatcherList<MapType, R>,
    R: ValueRenderer<MapType::Key>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        Description::new("has exactly the matching entries")
            .omitted_children(self.0.len().saturating_sub(context.render().max_items()))
            .children(
                (0..self.0.len().min(context.render().max_items()))
                    .map(|index| self.0.describe_at(index, context)),
            )
    }

    fn evaluate(&self, actual: &MapType, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut group = context.isolated();
        let mut matched = actual.length() == self.0.len();
        let mut found = FoundEntries::new();
        for index in 0..self.0.len() {
            matched &= self.0.evaluate_at(index, actual, &mut group).matched;
            if let Some(key) = self.0.found_key(index, actual) {
                found.record(key);
            }
        }
        let mut extras = group.isolated_for_order(MapType::RENDERING_ORDER);
        for (key, _) in actual.entries() {
            if !found.contains(key) {
                matched = false;
                if context.is_positive() {
                    if extras.is_diagnostic() {
                        extras.record(
                            FailureBuilder::detached::<MapType>(FailureKind::Matching)
                                .path([PathSegment::Key(
                                    extras.render().value(key).into_rendered(),
                                )])
                                .relation("has an unexpected key")
                                .build(),
                        );
                    } else {
                        extras.outcome(false, |_| Description::new("has an unexpected key"));
                    }
                }
            }
        }
        group.append(extras);
        if matched != context.is_positive() {
            if group.evidence.is_empty() && group.omitted == 0 {
                group.outcome(matched, |context| {
                    <Self as AssertrMatcher<MapType, R>>::describe(self, context)
                });
            }
            context.append(group);
        }
        MatchResult::new(matched)
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

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        self.0.describe_at(index, context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &MapType,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        self.0.evaluate_at(index, actual, context)
    }
}

impl<MapType, R, L> EntryMatcherList<MapType, R> for EntriesAre<L>
where
    MapType: Map + ?Sized,
    L: EntryMatcherList<MapType, R>,
{
    fn found_key<'a>(&self, index: usize, actual: &'a MapType) -> Option<&'a MapType::Key> {
        self.0.found_key(index, actual)
    }
}

/// Exact keyed matching with equality shorthand for value expressions.
#[macro_export]
macro_rules! entries_are {
    (@list) => {
        $crate::__private::Nil
    };
    (@list ($key:expr, $value:expr) $(, ($tail_key:expr, $tail_value:expr))* $(,)?) => {
        $crate::__private::Cons(
            $crate::matchers::entry($key, $crate::__private::normalize($value)),
            $crate::entries_are!(@list $(($tail_key, $tail_value)),*)
        )
    };
    ($(($key:expr, $value:expr)),* $(,)?) => {
        $crate::matchers::entries_are($crate::entries_are!(@list $(($key, $value)),*))
    };
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, test_support::UnorderedMap};
    use alloc::collections::BTreeMap;

    #[test]
    fn duplicate_keys_do_not_hide_unexpected_entries() {
        let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
            .capture(|it| it.matches(entries_are![("a", 1), ("a", 1)]));

        assert_that!(failures).has_length(1);
    }

    #[test]
    fn sorts_unexpected_keys_before_limiting_them() {
        let capture = |entries| {
            let actual = UnorderedMap(entries);
            assert_that!(actual)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
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
