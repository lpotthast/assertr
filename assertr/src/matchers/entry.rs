use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use crate::{
    ValueRenderer,
    assertions::map::{Map, MapKeyQuery, MapLookup},
    failure::PathSegment,
    renderer::IntoRendered,
};

/// A value matcher under one native map key query.
pub struct Entry<K, M> {
    pub(super) key: K,
    matcher: M,
}

/// Matches a value at a key using the map's native lookup relation.
pub fn entry<K, M>(key: K, matcher: M) -> Entry<K, M> {
    Entry { key, matcher }
}

impl<MapType, R, K, M> AssertrMatcher<MapType, R> for Entry<K, M>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: AssertrMatcher<MapType::Value, R>,
    R: ValueRenderer<K>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("has a matching entry")
            .expected(context.render().value(&self.key))
            .children([self.matcher.describe(context)])
    }

    fn evaluate(&self, actual: &MapType, context: &mut MatchContext<'_, R>) -> MatchResult {
        let evaluate = |context: &mut MatchContext<'_, R>| {
            if let Some((_, value)) = actual.get_key_value(self.key.as_query()) {
                self.matcher.evaluate(value, context)
            } else {
                context.outcome(false, |_| {
                    ConstraintDescription::new("contains the required key")
                })
            }
        };
        if context.is_diagnostic() {
            let path = PathSegment::Key(context.render().value(&self.key).into_rendered());
            context.scoped(path, evaluate)
        } else {
            evaluate(context)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::entry;
    use crate::{
        failure::{FailureKind, PathSegment},
        matchers::{anything, equal_to},
        prelude::*,
    };
    use alloc::{collections::BTreeMap, string::String};

    #[test]
    fn accepts_borrowed_key_queries() {
        let actual = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

        assert_that!(actual).matches(entry("a", equal_to(1)));
        assert_that!(actual).does_not_match(entry("a", equal_to(2)));
        assert_that!(actual).does_not_match(entry("missing", anything()));
    }

    #[test]
    fn scopes_value_failures_to_the_queried_key() {
        let failures = assert_that!(BTreeMap::from([("a", 1)]))
            .capture(|it| it.matches(entry("a", equal_to(2))));

        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children[0].path).has_length(1);
        assert_that!(failures[0].children[0].path[0]).is_matching(pattern!(PathSegment::Key(_)));
        assert_that!(failures[0].children[0].kind).is_equal_to(FailureKind::Equality);
    }
}
