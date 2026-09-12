use alloc::vec::Vec;
use core::borrow::Borrow;

use super::{Map, MapKeyQuery, MapLookup, imp};
use crate::{
    AssertThat, Mode, ValueRenderer, assertions::map::EntryMatcherList,
    expectation::ExpectationDiagnostics, mode::Capture,
};

/// Assertions over the keys, values, and entries of a map: `BTreeMap`, `HashMap`, and every type
/// implementing [`Map`].
///
/// Single-key assertions accept `&Q`. Bulk keys implement [`MapKeyQuery<K>`], whose associated
/// query type keeps borrowed lookup inference-friendly. Both go through the map's native
/// [`MapLookup<Q>`], so the bounds are the map's own: `Q: Hash + Eq` on a `HashMap`, `Q: Ord` on a
/// `BTreeMap`. Because the standard maps look up any `Q` their key borrows as, a `HashMap<String,
/// _>` can be queried with a `str`. The [`MapAssertions::Map`] associated type names the subject
/// map so methods can express that requirement. Assertr renders the map structure. The renderer
/// needs capabilities only for keys and values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MapAssertions<K, V, R> {
    /// The subject map type. Assertions that query a key require it to implement [`MapLookup`] for
    /// the query type.
    type Map: Map<Key = K, Value = V>;

    /// Asserts that the map has an entry under `expected`, regardless of its value.
    fn contains_key<Q>(self, expected: &Q) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q>;

    /// Asserts that the map has no entry under `not_expected`.
    fn does_not_contain_key<Q>(self, not_expected: &Q) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q>;

    /// Asserts that at least one map value equals `expected`.
    fn contains_value<E>(self, expected: E) -> Self
    where
        V: PartialEq<E>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E>;

    /// Asserts that no map value equals `not_expected`.
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        V: PartialEq<E>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E>;

    /// Asserts that the map maps `key` to `value`.
    ///
    /// `value` is accepted owned or borrowed. When the stored value type is itself a reference,
    /// such as `&str`, a borrowed argument satisfies both `Borrow<str>` and `Borrow<&str>` and the
    /// expected type `E` cannot be inferred. Name it with `contains_entry::<&str, _>("k", "v")`.
    ///
    /// This performs one assertion covering both key presence and value equality. A missing key is
    /// reported once.
    fn contains_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: PartialEq<E>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E>;

    /// Asserts that the map has `key` and its value satisfies `assertions`.
    ///
    /// The assertions run in capture mode against the value. If any fail, this method raises one
    /// map-level failure carrying every captured value failure as a nested failure located at the
    /// key.
    fn contains_entry_satisfying<A, Q>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<Q> + Clone;

    /// Asserts that the map does not map `key` to `value`. This passes when the key is absent as
    /// well as when it is present with a different value. Like
    /// [`contains_entry`](MapAssertions::contains_entry), it needs the expected type named when the
    /// stored value type is a reference. Use `does_not_contain_entry::<&str, _>("k", "v")`.
    fn does_not_contain_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: PartialEq<E>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E>;

    /// Asserts that every expected key is present. Extra map keys are allowed.
    fn contains_keys<E, I>(self, expected: I) -> Self
    where
        E: MapKeyQuery<K>,
        Self::Map: MapLookup<<E as MapKeyQuery<K>>::Query>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E>;

    /// Asserts that the map contains exactly the given entries. There are no missing or unexpected
    /// keys, and every value is equal to its expectation.
    fn contains_exactly_entries<EK, EV, I>(self, expected: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Self::Map: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        V: PartialEq<EV>,
        I: IntoIterator<Item = (EK, EV)>,
        R: ValueRenderer<K>
            + ValueRenderer<V>
            + ValueRenderer<EK>
            + ValueRenderer<EV>
            + ValueRenderer<usize>;

    /// Asserts that the map contains exactly the given keys and that each value matches the
    /// predicate paired with its key. Missing and unexpected keys are failures.
    fn contains_exactly_entries_matching<L>(self, expected: L) -> Self
    where
        L: EntryMatcherList<Self::Map, R>,
        R: ValueRenderer<K>;

    /// Asserts that the map contains exactly the given keys and that each value satisfies the
    /// assertions paired with its key. Missing and unexpected keys are failures.
    ///
    /// Each value's assertions run in capture mode. Every captured failure is retained as a nested
    /// failure of the map-level diagnostic, located at its expected key.
    fn contains_exactly_entries_satisfying<EK, A, I>(self, assertions: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Self::Map: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        I: IntoIterator<Item = (EK, A)>,
        R: ValueRenderer<K> + ValueRenderer<EK> + Clone;
    /// Asserts that the value at a key satisfies a matcher.
    #[track_caller]
    fn contains_entry_matching<Q: ?Sized, E>(self, key: &Q, expected: E) -> Self
    where
        Self::Map: MapLookup<Q>,
        E: ExpectationDiagnostics<V, R>,
        R: ValueRenderer<Q>;

    /// Asserts that some map value satisfies a matcher.
    #[track_caller]
    fn contains_value_matching<E>(self, expected: E) -> Self
    where
        E: ExpectationDiagnostics<V, R>;
}

// `K` and `V` are explicit impl parameters rather than written as `Mp::Key` / `Mp::Value`: a method
// bound `Mp: MapLookup<Mp::Key>` is a bounds cycle (E0391), `Mp: MapLookup<K>` is not.
impl<Mp, K, V, M, R> MapAssertions<K, V, R> for AssertThat<'_, Mp, M, R>
where
    Mp: Map<Key = K, Value = V>,
    M: Mode,
{
    type Map = Mp;

    #[track_caller]
    fn contains_key<Q>(self, expected: &Q) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
    {
        self.apply_assertion(imp::ContainsKey::new(expected))
    }

    #[track_caller]
    fn does_not_contain_key<Q>(self, not_expected: &Q) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
    {
        self.apply_assertion(imp::DoesNotContainKey::new(not_expected))
    }

    #[track_caller]
    fn contains_value<E>(self, expected: E) -> Self
    where
        Mp::Value: PartialEq<E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        self.apply_assertion(imp::ContainsValue::new(expected))
    }

    #[track_caller]
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        Mp::Value: PartialEq<E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        self.apply_assertion(imp::DoesNotContainValue::new(not_expected))
    }

    #[track_caller]
    fn contains_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        Mp::Value: PartialEq<E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
    {
        self.apply_assertion(imp::ContainsEntry::with_storage(key, value))
    }

    #[track_caller]
    fn contains_entry_satisfying<A, Q>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
        R: ValueRenderer<Q> + Clone,
    {
        self.contains_entry_matching(key, crate::expectation::satisfying(assertions))
    }

    #[track_caller]
    fn does_not_contain_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        Mp::Value: PartialEq<E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
    {
        self.apply_assertion(imp::DoesNotContainEntry::with_storage(key, value))
    }

    #[track_caller]
    fn contains_keys<E, I>(self, expected: I) -> Self
    where
        E: MapKeyQuery<K>,
        Mp: MapLookup<<E as MapKeyQuery<K>>::Query>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        self.track_assertion();
        let expected = expected.into_iter().collect::<Vec<_>>();
        self.apply_assertion_after_tracking(imp::ContainsKeys::new(expected))
    }

    #[track_caller]
    fn contains_exactly_entries<EK, EV, I>(self, expected: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Mp: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        Mp::Value: PartialEq<EV>,
        I: IntoIterator<Item = (EK, EV)>,
        R: ValueRenderer<Mp::Key>
            + ValueRenderer<usize>
            + ValueRenderer<Mp::Value>
            + ValueRenderer<EK>
            + ValueRenderer<EV>,
    {
        self.track_assertion();
        let expected = expected.into_iter().collect::<Vec<_>>();
        self.apply_assertion_after_tracking(imp::ContainsExactlyEntries::new(expected))
    }

    #[track_caller]
    fn contains_exactly_entries_matching<L>(self, expected: L) -> Self
    where
        L: EntryMatcherList<Self::Map, R>,
        R: ValueRenderer<K>,
    {
        self.track_assertion();

        self.apply_assertion_after_tracking(crate::assertions::map::entries_are(expected))
    }

    #[track_caller]
    fn contains_exactly_entries_satisfying<EK, A, I>(self, assertions: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Mp: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
        I: IntoIterator<Item = (EK, A)>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<EK> + Clone,
    {
        self.track_assertion();
        let expected = crate::assertions::map::entry_matchers(
            assertions
                .into_iter()
                .map(|(key, assertions)| (key, crate::expectation::satisfying(assertions))),
        );
        self.apply_assertion_after_tracking(crate::assertions::map::entries_are(expected))
    }

    #[track_caller]
    fn contains_entry_matching<Q: ?Sized, E>(self, key: &Q, expected: E) -> Self
    where
        Self::Map: MapLookup<Q>,
        E: ExpectationDiagnostics<V, R>,
        R: ValueRenderer<Q>,
    {
        self.apply_assertion(super::matching::ContainsEntryMatching::new(key, expected))
    }

    #[track_caller]
    fn contains_value_matching<E>(self, expected: E) -> Self
    where
        E: ExpectationDiagnostics<V, R>,
    {
        self.apply_assertion(super::matching::ContainsValueMatching::new(expected))
    }
}

#[cfg(test)]
mod tests {
    mod contains_entry_matching {
        use crate::{assertions::core::partial_eq::equal_to, expectation::anything, prelude::*};
        use alloc::collections::BTreeMap;

        #[cfg(feature = "fluent")]
        #[test]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([("a", 1)])
                .must()
                .contain_entry_matching("a", equal_to(1));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_entry_matching("a", equal_to(2))
            );
        }

        #[test]
        fn preserves_borrowed_lookup_and_key_paths() {
            let map = BTreeMap::from([(alloc::string::String::from("a"), 1)]);
            assert_that!(map).contains_entry_matching("a", equal_to(1));
            let failures =
                assert_that!(map).capture(|it| it.contains_entry_matching("b", anything()));
            assert_that!(&failures[0].children[0].path[0])
                .is_matching(pattern!(crate::failure::PathSegment::Key(_)));
        }

        #[test]
        fn zero_budget_does_not_render_keys() {
            struct NeverRender;
            impl ValueRenderer<str> for NeverRender {
                fn fmt(&self, _: &str, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("omitted key rendered")
                }
            }
            let map = BTreeMap::from([("a", 1)]);
            let failures = assert_that!(map)
                .with_renderer(NeverRender)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.contains_entry_matching("b", anything()));
            assert_that!(failures[0].omitted_children).is_equal_to(1);
        }
    }

    mod contains_value_matching {
        use crate::{expectation::predicate, prelude::*};
        use alloc::collections::BTreeMap;

        #[cfg(feature = "fluent")]
        #[test]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([("a", 1)])
                .must()
                .contain_value_matching(predicate(|x: &i32| *x == 1));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_value_matching(crate::assertions::core::partial_eq::equal_to(2))
            );
        }

        #[test]
        fn requires_neither_key_nor_value_rendering() {
            assert_that!(BTreeMap::from([("a", 1)]))
                .with_renderer(crate::test_support::NoRenderer)
                .contains_value_matching(predicate(|x: &i32| *x == 1));
        }

        #[test]
        fn sorts_candidate_evidence_before_limiting_it() {
            use crate::{assertions::core::partial_eq::equal_to, test_support::UnorderedMap};

            let capture = |entries| {
                let actual = UnorderedMap(entries);
                assert_that!(actual)
                    .with_location(false)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                    .capture(|it| it.contains_value_matching(equal_to(9)))
            };
            let expected = capture(vec![(1, 1), (2, 2), (3, 3)]);
            let actual = capture(vec![(3, 3), (2, 2), (1, 1)]);
            assert_that!(actual[0].children).has_length(1);
            assert_that!(actual[0].omitted_children).is_equal_to(2);
            assert_that!(ToHumanReadableText.render(&actual[0]))
                .is_equal_to(ToHumanReadableText.render(&expected[0]));
        }
    }

    mod renderer_contract {
        use alloc::collections::BTreeMap;

        use crate::{
            prelude::*,
            test_support::{
                NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
            },
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, BTreeMap<i32, i32>, Panic, NoRenderer>
                    => MapAssertions<i32, i32, NoRenderer>
            );
        }

        #[test]
        fn equality_uses_the_active_renderer_type() {
            assert_that!(BTreeMap::from([("a", RendererActual(1))]))
                .with_renderer(SentinelRenderer)
                .contains_value(RendererExpected(1))
                .contains_entry("a", RendererExpected(1))
                .contains_exactly_entries([("a", RendererExpected(1))]);
        }
    }

    mod rendering_budget {
        use alloc::collections::BTreeMap;

        use crate::prelude::*;

        #[test]
        fn limits_complete_expected_and_unexpected_entry_groups() {
            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("x", 10), ("y", 20), ("z", 30)]));

            assert_that!(ToHumanReadableText.render(&failures[0]))
                .contains("Expected: [")
                .contains("] (... 2 more entries ...)");
            assert_that!(failures[0].facts.iter().any(|fact| {
                fact.label == "Unexpected entries"
                    && rendered_text(&fact.value).starts_with('[')
                    && rendered_text(&fact.value).contains("] (... 2 more entries ...)")
            }))
            .is_true();
        }
    }

    #[cfg(feature = "std")]
    mod contains_key {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().contain_key("foo");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(assert_that!(map), contains_key("baz"));
        }

        #[test]
        fn succeeds_when_key_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).contains_key("foo");
        }

        #[test]
        fn accepts_str_query_for_string_key() {
            let map = HashMap::from([(String::from("foo"), "bar")]);
            assert_that!(map).contains_key("foo");
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map).with_location(false).contains_key("baz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain key

                    Expected: "baz"
                    -------- assertr --------
                "#});
        }
    }

    #[cfg(feature = "std")]
    mod does_not_contain_key {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().not_contain_key("baz");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(assert_that!(map), does_not_contain_key("foo"));
        }

        #[test]
        fn succeeds_when_key_is_absent() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_key("baz");
        }

        #[test]
        fn accepts_str_query_for_string_key() {
            let map = HashMap::from([(String::from("foo"), "bar")]);
            assert_that!(map).does_not_contain_key("baz");
        }

        #[test]
        fn panics_when_key_is_present() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_key("foo");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    contains key

                    Unexpected: "foo"
                    -------- assertr --------
                "#});
        }
    }

    #[cfg(feature = "std")]
    mod contains_value {
        use std::collections::HashMap;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().contain_value("bar");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(assert_that!(map), contains_value("baz"));
        }

        #[test]
        fn succeeds_when_value_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).contains_value("bar");
        }

        #[test]
        fn panics_when_value_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map).with_location(false).contains_value("baz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain value

                    Expected: "baz"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn compiles_with_any_type_comparable_to_the_actual_value_type() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).contains_value("bar".to_string());
        }

        #[test]
        fn can_check_for_custom_type() {
            #[derive(Debug, PartialEq)]
            struct Data {
                data: u32,
            }

            let mut map = HashMap::new();
            map.insert("foo", Data { data: 0 });
            assert_that!(&map).contains_value(Data { data: 0 });
            assert_that!(&map).contains_value(Data { data: 0 });
        }
    }

    #[cfg(feature = "std")]
    mod does_not_contain_value {
        use std::collections::HashMap;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().not_contain_value("baz");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(assert_that!(map), does_not_contain_value("bar"));
        }

        #[test]
        fn succeeds_when_value_is_absent() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_value("baz");
        }

        #[test]
        fn panics_when_value_is_present() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_value("bar");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    contains value

                    Unexpected: "bar"
                    -------- assertr --------
                "#});
        }
    }

    #[cfg(feature = "std")]
    mod contains_entry {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().contain_entry::<&str, _>("foo", "bar");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(
                assert_that!(map),
                contains_entry::<&str, _>("baz", "someValue")
            );
        }

        #[test]
        fn succeeds_when_value_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            // `&str` borrows both `str` and `&str`, so the equality target must be named.
            assert_that!(map).contains_entry::<&str, _>("foo", "bar");
        }

        #[test]
        fn tracks_one_assertion_for_key_presence_and_value_equality() {
            let map = HashMap::from([("foo", 1)]);

            let assertion = assert_that!(map).contains_entry("foo", 1);

            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn accepts_str_query_for_string_key() {
            let map = HashMap::from([(String::from("foo"), 1)]);
            assert_that!(map).contains_entry("foo", 1);
        }

        #[test]
        fn succeeds_when_value_is_present_with_complex_type_with_borrowable_values() {
            #[derive(Debug, PartialEq)]
            struct Person {
                age: u32,
            }
            let mut map = HashMap::<&str, Person>::new();
            map.insert("foo", Person { age: 42 });
            assert_that!(&map).contains_entry("foo", &Person { age: 42 });
            assert_that!(&map).contains_entry("foo", Person { age: 42 });
            assert_that!(&map).contains_entry("foo", Box::new(Person { age: 42 }));
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .contains_entry::<&str, _>("baz", "someValue");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain key

                    Expected: "baz"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_key_is_present_but_value_is_not_equal() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .contains_entry::<&str, _>("foo", "someValue");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain the expected value at a key

                    Nested failures:
                      - At key "foo":
                        Expected: "someValue"

                          Actual: "bar"
                    -------- assertr --------
                "#});
        }
    }

    mod contains_entry_satisfying {
        use alloc::collections::BTreeMap;

        use indoc::formatdoc;

        use crate::{
            prelude::*,
            test_support::{RendererActual, RendererExpected, SENTINEL, SentinelRenderer},
        };

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        fn is_positive_and_large(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0).is_greater_than(10);
        }

        fn is_renderer_expected_two(it: AssertThat<RendererActual, Capture, SentinelRenderer>) {
            it.is_equal_to(RendererExpected(2));
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([("retries", 3)])
                .must()
                .contain_entry_satisfying("retries", is_three);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("retries", 3)])),
                contains_entry_satisfying("timeout", is_three)
            );
        }

        #[test]
        fn succeeds_when_the_value_satisfies_all_assertions() {
            assert_that!(BTreeMap::from([("retries", 12)]))
                .contains_entry_satisfying("retries", is_positive_and_large);
        }

        #[test]
        fn accepts_str_query_for_string_key() {
            let map = BTreeMap::from([(String::from("retries"), 3)]);
            assert_that!(map).contains_entry_satisfying("retries", is_three);
        }

        #[test]
        fn panics_when_the_key_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("retries", 3)]))
                    .with_location(false)
                    .contains_entry_satisfying("timeout", is_three);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("retries", 3)])`

                does not contain a matching entry

                Nested failures:
                  - At ["timeout"]:
                    does not satisfy the constraint

                    Constraint:
                        contains the required key
                -------- assertr --------
            "#});
        }

        #[test]
        fn reports_every_unsatisfied_value_assertion() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("retries", -3)]))
                    .with_location(false)
                    .contains_entry_satisfying("retries", is_positive_and_large);
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("retries", -3)])`

                does not contain a matching entry

                Nested failures:
                  - At ["retries"]:
                    Actual: -3

                    is not greater than

                    Expected: 0
                  - At ["retries"]:
                    Actual: -3

                    is not greater than

                    Expected: 10
                -------- assertr --------
            "#});
        }

        #[test]
        fn nested_failures_use_the_active_renderer() {
            let failures = assert_that!(BTreeMap::from([("value", RendererActual(1))]))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains_entry_satisfying("value", is_renderer_expected_two));

            assert_that!(ToHumanReadableText.render(&failures[0].children[0])).contains(SENTINEL);
        }

        #[test]
        fn opaque_callback_failures_preserve_custom_rendering_and_key_paths() {
            use crate::test_support::{CustomValueRenderer, assert_custom_value};

            struct Opaque(usize);

            fn check(it: AssertThat<'_, Opaque, Capture, CustomValueRenderer>) {
                it.satisfies(
                    |value| &value.0,
                    |value| {
                        value.is_equal_to(9);
                    },
                );
            }

            let values = BTreeMap::from([("value", Opaque(1))]);
            let failures = assert_that!(values)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_entry_satisfying("value", check));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            let child = &failures[0].children[0];
            assert_custom_value(child.actual.as_ref().unwrap(), &1_usize);
            assert_custom_value(child.expected.as_ref().unwrap(), &9_usize);
            assert_that!(child.path).has_length(1);
            let crate::failure::PathSegment::Key(key) = &child.path[0] else {
                panic!("expected the callback failure at its map key");
            };
            assert_custom_value(key, "value");
        }
    }

    #[cfg(feature = "std")]
    mod does_not_contain_entry {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            map.must().not_contain_entry::<&str, _>("baz", "bar");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_caller_location!(
                assert_that!(map),
                does_not_contain_entry::<&str, _>("foo", "bar")
            );
        }

        #[test]
        fn succeeds_when_key_is_absent() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_entry::<&str, _>("baz", "bar");
        }

        #[test]
        fn succeeds_when_value_differs() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_entry::<&str, _>("foo", "baz");
        }

        #[test]
        fn accepts_str_query_for_string_key() {
            let map = HashMap::from([(String::from("foo"), 1)]);
            assert_that!(map).does_not_contain_entry("foo", 2);
        }

        #[test]
        fn panics_when_entry_is_present() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_entry::<&str, _>("foo", "bar");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    contains the entry

                    Unexpected: (
                        "foo",
                        "bar",
                    )
                    -------- assertr --------
                "#});
        }
    }

    #[cfg(feature = "std")]
    mod contains_keys {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            map.must().contain_keys(["foo", "baz"]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let map = HashMap::from([("foo", "bar")]);
            assert_caller_location!(assert_that!(map), contains_keys(["foo", "baz"]));
        }

        #[test]
        fn succeeds_when_all_keys_are_present() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            assert_that!(map).contains_keys(["foo", "baz"]);
        }

        #[test]
        fn accepts_str_queries_for_string_keys() {
            let map = HashMap::from([(String::from("foo"), 1), (String::from("bar"), 2)]);
            assert_that!(map).contains_keys(["foo", "bar"]);
        }

        #[test]
        fn panics_when_a_key_is_missing() {
            assert_that_panic_by(|| {
                let map = HashMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_keys(["foo", "baz"]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain all of

                    Expected: [
                        "foo",
                        "baz",
                    ]

                    Details:
                      - Keys not found: [
                            "baz",
                        ]
                    -------- assertr --------
                "#});
        }
    }

    #[cfg(feature = "std")]
    mod contains_exactly_entries {
        use crate::{Fact, prelude::*};
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            map.must()
                .contain_exactly_entries([("foo", "bar"), ("baz", "qux")]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            let map = HashMap::from([("a", 1)]);
            assert_caller_location!(
                assert_that!(map),
                contains_exactly_entries([("a", 1), ("a", 1)])
            );
        }

        #[test]
        fn succeeds_when_entries_match() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            assert_that!(&map).contains_exactly_entries([("foo", "bar"), ("baz", "qux")]);
            assert_that!(map)
                .contains_exactly_entries(HashMap::from([("foo", "bar"), ("baz", "qux")]));
        }

        #[test]
        fn accepts_str_queries_for_string_keys() {
            let map = HashMap::from([(String::from("foo"), 1), (String::from("bar"), 2)]);
            assert_that!(map).contains_exactly_entries([("foo", 1), ("bar", 2)]);
        }

        #[test]
        fn panics_when_an_expected_key_is_missing() {
            assert_that_panic_by(|| {
                let map = HashMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_exactly_entries([("foo", "bar"), ("baz", "qux")]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain exactly

                    Expected: [
                        (
                            "foo",
                            "bar",
                        ),
                        (
                            "baz",
                            "qux",
                        ),
                    ]

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                      - Keys not found: [
                            "baz",
                        ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_the_expected_entries_repeat_a_key() {
            assert_that_panic_by(|| {
                let map = HashMap::from([("a", 1)]);
                assert_that!(map)
                    .with_location(false)
                    .contains_exactly_entries([("a", 1), ("a", 1)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "a": 1,
                    }} (sorted for rendering)

                    does not contain exactly

                    Expected: [
                        (
                            "a",
                            1,
                        ),
                        (
                            "a",
                            1,
                        ),
                    ]

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                    -------- assertr --------
                "#});
        }

        /// A repeated expected key makes the counts agree, so the extra actual entry can only be
        /// found by knowing which stored entries the expectations resolved to.
        #[test]
        fn panics_when_a_repeated_key_hides_an_unexpected_entry_behind_matching_counts() {
            use alloc::collections::BTreeMap;

            assert_that_panic_by(|| {
                let map = BTreeMap::from([("a", 1), ("b", 2)]);
                assert_that!(map)
                    .with_location(false)
                    .contains_exactly_entries([("a", 1), ("a", 1)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: BTreeMap {{
                        "a": 1,
                        "b": 2,
                    }}

                    does not contain exactly

                    Expected: [
                        (
                            "a",
                            1,
                        ),
                        (
                            "a",
                            1,
                        ),
                    ]

                    Details:
                      - Unexpected entries: [
                            (
                                "b",
                                2,
                            ),
                        ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_an_unexpected_entry_is_present() {
            assert_that_panic_by(|| {
                let map = HashMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_exactly_entries(HashMap::<&str, &str>::new());
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain exactly

                    Expected: []

                    Details:
                      - Actual length: 1
                      - Expected length: 0
                      - Unexpected entries: [
                            (
                                "foo",
                                "bar",
                            ),
                        ] (sorted for rendering)
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_an_expected_value_differs() {
            assert_that_panic_by(|| {
                let map = HashMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_exactly_entries([("foo", "baz")]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }} (sorted for rendering)

                    does not contain exactly

                    Expected: [
                        (
                            "foo",
                            "baz",
                        ),
                    ]

                    Details:
                      - Keys with unexpected values: [
                            "foo",
                        ]
                    Nested failures:
                      - At key "foo":
                        Expected: "baz"

                          Actual: "bar"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn nested_failures_keep_the_order_of_the_expected_entries_for_an_ordered_map() {
            use alloc::collections::BTreeMap;

            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("b", 0), ("a", 0)]));

            let keys = failures[0]
                .children
                .iter()
                .map(|child| rendered_text(&child.facts[0].value))
                .collect::<Vec<_>>();
            assert_that!(keys).contains_exactly(["\"b\"".to_owned(), "\"a\"".to_owned()]);
        }

        #[test]
        fn nested_failures_are_sorted_by_their_text_for_a_map_rendered_in_sorted_order() {
            let failures = assert_that!(HashMap::from([("a", 1), ("b", 2)]))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("b", 0), ("a", 0)]));

            let keys = failures[0]
                .children
                .iter()
                .map(|child| rendered_text(&child.facts[0].value))
                .collect::<Vec<_>>();
            assert_that!(keys).contains_exactly(["\"a\"".to_owned(), "\"b\"".to_owned()]);
        }

        #[test]
        fn limits_the_nested_value_failures_to_the_rendering_budget() {
            use alloc::collections::BTreeMap;

            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("a", 0), ("b", 0), ("c", 0)]));

            assert_that!(failures[0].children.as_slice()).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|item| rendered_text(&item.facts[0].value))
                        .is_equal_to("\"a\"");
                },
            ]);
            assert_that!(failures[0].facts.as_slice())
                .contains(Fact::note("... 2 more unexpected values ..."));
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    mod contains_exactly_entries_matching {
        use alloc::collections::BTreeMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn is_two(value: &i32) -> bool {
            *value == 2
        }

        type Predicate = fn(&i32) -> bool;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([("a", 1)])
                .must()
                .contain_exactly_entries_matching(crate::assertions::map::entry_matchers(
                    ([("a", is_one)])
                        .into_iter()
                        .map(|(key, p)| (key, crate::expectation::predicate(p))),
                ));
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_exactly_entries_matching(crate::entries_are![(
                    "a",
                    crate::matchers::eq(2)
                )])
            );
        }

        #[test]
        fn succeeds_when_the_keys_are_exact_and_each_value_matches() {
            let predicates: [(&str, Predicate); 2] = [("b", is_two), ("a", is_one)];

            assert_that!(BTreeMap::from([("a", 1), ("b", 2)])).contains_exactly_entries_matching(
                crate::assertions::map::entry_matchers(
                    (predicates)
                        .into_iter()
                        .map(|(key, p)| (key, crate::expectation::predicate(p))),
                ),
            );
        }

        #[test]
        fn accepts_str_queries_for_string_keys() {
            let predicates: [(&str, Predicate); 2] = [("b", is_two), ("a", is_one)];
            let map = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

            assert_that!(map).contains_exactly_entries_matching(
                crate::assertions::map::entry_matchers(
                    (predicates)
                        .into_iter()
                        .map(|(key, p)| (key, crate::expectation::predicate(p))),
                ),
            );
        }

        #[test]
        fn panics_when_an_expected_key_is_missing() {
            let predicates: [(&str, Predicate); 2] = [("a", is_two), ("missing", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 2)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(crate::assertions::map::entry_matchers(
                        (predicates)
                            .into_iter()
                            .map(|(key, p)| (key, crate::expectation::predicate(p))),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 2)])`

                does not match

                Nested failures:
                  - At ["missing"]:
                    does not satisfy the constraint

                    Constraint:
                        contains the required key
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_the_expected_keys_repeat_a_key() {
            let predicates: [(&str, Predicate); 2] = [("a", is_one), ("a", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(crate::assertions::map::entry_matchers(
                        (predicates)
                            .into_iter()
                            .map(|(key, p)| (key, crate::expectation::predicate(p))),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1)])`

                does not match

                Nested failures:
                  - does not satisfy the constraint

                    Constraint:
                        has exactly the matching entries

                        Nested failures:
                          - contains a matching entry

                            Expected: "a"

                            Nested failures:
                              - satisfies the predicate
                          - contains a matching entry

                            Expected: "a"

                            Nested failures:
                              - satisfies the predicate
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_an_unexpected_entry_is_present() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1), ("extra", 9)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(crate::assertions::map::entry_matchers(
                        ([("a", is_one)])
                            .into_iter()
                            .map(|(key, p)| (key, crate::expectation::predicate(p))),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1), ("extra", 9)])`

                does not match

                Nested failures:
                  - At ["extra"]:
                    has an unexpected key
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_an_expected_value_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(crate::assertions::map::entry_matchers(
                        ([("a", is_two)])
                            .into_iter()
                            .map(|(key, p)| (key, crate::expectation::predicate(p))),
                    ));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1)])`

                does not match

                Nested failures:
                  - At ["a"]:
                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "#});
        }

        #[test]
        fn keyed_lists_can_be_reused_by_reference() {
            let map = alloc::collections::BTreeMap::from([("a", 1)]);
            let expected = entries_are![("a", crate::matchers::eq(1))];
            assert_that!(map)
                .contains_exactly_entries_matching(&expected)
                .contains_exactly_entries_matching(&expected);
        }

        #[test]
        fn supports_large_homogeneous_keyed_lists() {
            let map: BTreeMap<_, _> = (0..4096).map(|key| (key, key)).collect();
            let expected = crate::assertions::map::entry_matchers(
                (0..4096)
                    .rev()
                    .map(|key| (key, crate::assertions::core::partial_eq::equal_to(key))),
            );

            assert_that!(map).contains_exactly_entries_matching(expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn requires_no_ordering_for_hash_map_keys() {
            #[derive(Debug, PartialEq, Eq, Hash)]
            struct Key(u8);

            let map = std::collections::HashMap::from([(Key(1), 10), (Key(2), 20)]);
            assert_that!(map).contains_exactly_entries_matching(entries_are![
                (Key(2), crate::matchers::eq(20)),
                (Key(1), crate::matchers::eq(10))
            ]);
        }
    }

    mod contains_exactly_entries_satisfying {
        use alloc::collections::BTreeMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        fn is_one(it: AssertThat<i32, Capture>) {
            it.is_equal_to(1);
        }

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        fn is_zero(it: AssertThat<i32, Capture>) {
            it.is_equal_to(0);
        }

        type ValueAssertions = fn(AssertThat<i32, Capture>);

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([("a", 1)])
                .must()
                .contain_exactly_entries_satisfying([("a", is_one)]);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_exactly_entries_satisfying([("a", is_two)])
            );
        }

        #[test]
        fn succeeds_when_the_keys_are_exact_and_each_value_satisfies() {
            let assertions: [(&str, ValueAssertions); 2] = [("b", is_two), ("a", is_one)];

            assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                .contains_exactly_entries_satisfying(assertions);
        }

        #[test]
        fn supports_large_homogeneous_keyed_lists() {
            let map: BTreeMap<_, _> = (0..4096).map(|key| (key, key)).collect();
            let checks = (0..4096).rev().map(|key| {
                (key, move |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(key);
                })
            });

            assert_that!(map).contains_exactly_entries_satisfying(checks);
        }

        #[test]
        fn accepts_str_queries_for_string_keys() {
            let assertions: [(&str, ValueAssertions); 2] = [("b", is_two), ("a", is_one)];
            let map = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

            assert_that!(map).contains_exactly_entries_satisfying(assertions);
        }

        #[test]
        fn panics_when_an_expected_key_is_missing() {
            let assertions: [(&str, ValueAssertions); 2] = [("a", is_two), ("missing", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 2)]))
                    .with_location(false)
                    .contains_exactly_entries_satisfying(assertions);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 2)])`

                does not match

                Nested failures:
                  - At ["missing"]:
                    does not satisfy the constraint

                    Constraint:
                        contains the required key
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_the_expected_keys_repeat_a_key() {
            let assertions: [(&str, ValueAssertions); 2] = [("a", is_one), ("a", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_satisfying(assertions);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1)])`

                does not match

                Nested failures:
                  - does not satisfy the constraint

                    Constraint:
                        has exactly the matching entries

                        Nested failures:
                          - contains a matching entry

                            Expected: "a"

                            Nested failures:
                              - satisfies the assertions
                          - contains a matching entry

                            Expected: "a"

                            Nested failures:
                              - satisfies the assertions
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_an_unexpected_entry_is_present() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1), ("extra", 9)]))
                    .with_location(false)
                    .contains_exactly_entries_satisfying([("a", is_one)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1), ("extra", 9)])`

                does not match

                Nested failures:
                  - At ["extra"]:
                    has an unexpected key
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_an_expected_value_does_not_satisfy() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_satisfying([("a", is_two)]);
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `BTreeMap::from([("a", 1)])`

                does not match

                Nested failures:
                  - At ["a"]:
                    Expected: 2

                      Actual: 1
                -------- assertr --------
            "#});
        }

        #[test]
        fn limits_repeated_value_evidence_to_the_rendering_budget() {
            let assertions: [(&str, ValueAssertions); 3] =
                [("a", is_zero), ("b", is_zero), ("c", is_zero)];
            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries_satisfying(assertions));

            assert_that!(failures[0].children.as_slice()).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| &value.path[0]).is_matching(
                pattern!(crate::failure::PathSegment::Key(key) if rendered_text(key) == "\"a\""),
            );
                },
            ]);
            assert_that!(failures[0].omitted_children).is_equal_to(2);
        }

        #[test]
        fn opaque_callback_failures_preserve_custom_rendering_and_key_paths() {
            use crate::test_support::{CustomValueRenderer, assert_custom_value};

            struct Opaque(usize);

            fn check(it: AssertThat<'_, Opaque, Capture, CustomValueRenderer>) {
                it.satisfies(
                    |value| &value.0,
                    |value| {
                        value.is_equal_to(9);
                    },
                );
            }

            let values = BTreeMap::from([("value", Opaque(1))]);
            let failures = assert_that!(values)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly_entries_satisfying([("value", check)]));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            let child = &failures[0].children[0];
            assert_custom_value(child.actual.as_ref().unwrap(), &1_usize);
            assert_custom_value(child.expected.as_ref().unwrap(), &9_usize);
            assert_that!(child.path).has_length(1);
            let crate::failure::PathSegment::Key(key) = &child.path[0] else {
                panic!("expected the callback failure at its map key");
            };
            assert_custom_value(key, &"value");
        }
    }

    /// The same assertions against a `BTreeMap`, which is `alloc` rather than `std`: this is the
    /// suite that runs in a `no_std` build, where maps had no assertions at all before.
    mod btree_map {
        use alloc::collections::BTreeMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        fn map() -> BTreeMap<&'static str, &'static str> {
            BTreeMap::from([("foo", "bar")])
        }

        fn is_bar(value: &&str) -> bool {
            *value == "bar"
        }

        fn satisfies_bar(it: AssertThat<&str, Capture>) {
            it.is_equal_to("bar");
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_aliases_are_as_expected() {
            map()
                .must()
                .contain_key("foo")
                .not_contain_key("baz")
                .contain_value("bar")
                .not_contain_value("baz")
                .contain_entry::<&str, _>("foo", "bar")
                .contain_entry_satisfying("foo", satisfies_bar)
                .not_contain_entry::<&str, _>("foo", "baz")
                .contain_keys(["foo"])
                .contain_exactly_entries([("foo", "bar")])
                .contain_exactly_entries_matching(crate::assertions::map::entry_matchers(
                    ([("foo", is_bar)])
                        .into_iter()
                        .map(|(key, p)| (key, crate::expectation::predicate(p))),
                ))
                .contain_exactly_entries_satisfying([("foo", satisfies_bar)]);
        }

        #[test]
        fn succeeds_for_every_assertion_of_the_family() {
            assert_that!(map())
                .contains_key("foo")
                .does_not_contain_key("baz")
                .contains_value("bar")
                .does_not_contain_value("baz")
                .contains_entry::<&str, _>("foo", "bar")
                .contains_entry_satisfying("foo", satisfies_bar)
                .does_not_contain_entry::<&str, _>("foo", "baz")
                .contains_keys(["foo"])
                .contains_exactly_entries([("foo", "bar")])
                .contains_exactly_entries_matching(crate::assertions::map::entry_matchers(
                    ([("foo", is_bar)])
                        .into_iter()
                        .map(|(key, p)| (key, crate::expectation::predicate(p))),
                ))
                .contains_exactly_entries_satisfying([("foo", satisfies_bar)]);
        }

        #[test]
        fn panics_naming_the_btree_map_type() {
            assert_that_panic_by(|| {
                assert_that!(map()).with_location(false).contains_key("baz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map()`

                    Actual: BTreeMap {{
                        "foo": "bar",
                    }}

                    does not contain key

                    Expected: "baz"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn reports_a_value_mismatch_at_a_present_key() {
            assert_that_panic_by(|| {
                assert_that!(map())
                    .with_location(false)
                    .contains_entry::<&str, _>("foo", "baz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map()`

                    Actual: BTreeMap {{
                        "foo": "bar",
                    }}

                    does not contain the expected value at a key

                    Nested failures:
                      - At key "foo":
                        Expected: "baz"

                          Actual: "bar"
                    -------- assertr --------
                "#});
        }

        #[test]
        fn a_missing_key_is_reported_once_not_twice() {
            let failures = assert_that!(map())
                .with_location(false)
                .capture(|it| it.contains_entry::<&str, _>("baz", "bar"));

            assert_that!(failures).has_length(1);
        }
    }

    mod contains_exactly_entries_on_btree_map {
        use crate::prelude::*;
        use alloc::collections::BTreeMap;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            BTreeMap::from([(1, 2)])
                .must()
                .contain_exactly_entries([(1, 2)]);
        }
        #[test]
        fn renders_length_facts_with_the_active_renderer() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, assert_custom_value};
            let subject = BTreeMap::from([(1, 2)]);
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([] as [(i32, i32); 0]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|item| item).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: BTreeMap {{
                    custom(1): custom(2),
                }}

                does not contain exactly

                Expected: []

                Details:
                  - Actual length: custom(1)
                  - Expected length: custom(0)
                  - Unexpected entries: [
                        (
                            custom(1),
                            custom(2),
                        ),
                    ]
                -------- assertr --------
            "});

                    assert_custom_value(&element.actual().facts[0].value, &1_usize);
                    assert_custom_value(&element.actual().facts[1].value, &0_usize);
                },
            ]);
        }
    }

    mod evaluation {
        use crate::{
            assertions::{
                HasLength,
                map::{self, Map, MapLookup},
            },
            prelude::*,
            renderer::RenderingOrder,
        };
        use alloc::collections::BTreeMap;
        use core::{borrow::Borrow, cell::Cell};

        #[derive(Debug)]
        struct Value<'a> {
            value: i32,
            comparisons: &'a Cell<usize>,
        }
        impl PartialEq<i32> for Value<'_> {
            fn eq(&self, expected: &i32) -> bool {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value == *expected
            }
        }
        struct ObservedMap<'a> {
            entries: [(i32, Value<'a>); 2],
            lookups: &'a Cell<usize>,
        }
        impl HasLength for ObservedMap<'_> {
            fn length(&self) -> usize {
                self.entries.len()
            }
        }
        impl<'a> Map for ObservedMap<'a> {
            type Key = i32;
            type Value = Value<'a>;
            const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;
            fn entries(&self) -> impl Iterator<Item = (&i32, &Self::Value)> {
                self.entries.iter().map(|(key, value)| (key, value))
            }
        }
        impl MapLookup<i32> for ObservedMap<'_> {
            fn get_key_value(&self, query: &i32) -> Option<(&i32, &Self::Value)> {
                self.lookups.set(self.lookups.get() + 1);
                self.entries().find(|(key, _)| *key == query)
            }
        }
        struct Expected<'a> {
            value: i32,
            borrows: &'a Cell<usize>,
        }
        impl Borrow<i32> for Expected<'_> {
            fn borrow(&self) -> &i32 {
                self.borrows.set(self.borrows.get() + 1);
                &self.value
            }
        }

        #[test]
        fn entry_checks_borrow_and_look_up_once_including_rejections() {
            for key in [1, 9] {
                let lookups = Cell::new(0);
                let comparisons = Cell::new(0);
                let borrows = Cell::new(0);
                let map = ObservedMap {
                    entries: [
                        (
                            1,
                            Value {
                                value: 10,
                                comparisons: &comparisons,
                            },
                        ),
                        (
                            2,
                            Value {
                                value: 20,
                                comparisons: &comparisons,
                            },
                        ),
                    ],
                    lookups: &lookups,
                };
                let failures = assert_that!(map).capture(|it| {
                    let it = it.contains_entry(
                        &key,
                        Expected {
                            value: 99,
                            borrows: &borrows,
                        },
                    );
                    assert_that!((lookups.get(), borrows.get())).is_equal_to((1, 1));
                    it.does_not_contain_entry(
                        &key,
                        Expected {
                            value: 10,
                            borrows: &borrows,
                        },
                    )
                });
                assert_that!((lookups.get(), borrows.get())).is_equal_to((2, 2));
                assert_that!(comparisons.get()).is_equal_to(if key == 1 { 2 } else { 0 });
                assert_that!(failures).has_length(if key == 1 { 2 } else { 1 });
            }
        }

        #[test]
        fn exact_entry_diagnostics_reuse_native_queries_and_comparisons() {
            let lookups = Cell::new(0);
            let comparisons = Cell::new(0);
            let map = ObservedMap {
                entries: [
                    (
                        1,
                        Value {
                            value: 10,
                            comparisons: &comparisons,
                        },
                    ),
                    (
                        2,
                        Value {
                            value: 20,
                            comparisons: &comparisons,
                        },
                    ),
                ],
                lookups: &lookups,
            };
            let failures = assert_that!(map)
                .capture(|it| it.contains_exactly_entries([(1, 99), (2, 99), (9, 99)]));
            assert_that!((lookups.get(), comparisons.get())).is_equal_to((3, 2));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(2);
        }

        #[test]
        fn missing_subject_descriptions_preserve_negative_operand_roles() {
            let map = BTreeMap::<i32, i32>::new();
            let root = assert_that!(map);
            let context = root.assertion_context();
            let descriptions = [
                context.describe::<BTreeMap<i32, i32>, _>(&map::DoesNotContainKey::new(&1)),
                context.describe::<BTreeMap<i32, i32>, _>(&map::DoesNotContainValue::new(2)),
                context.describe::<BTreeMap<i32, i32>, _>(&map::DoesNotContainEntry::new(&1, 2)),
            ];
            for description in descriptions {
                assert_that!(description.expected).is_none();
                assert_that!(description.unexpected).is_some();
            }
        }

        #[test]
        #[cfg(feature = "std")]
        fn tracks_before_expected_borrow_and_iteration_can_panic() {
            struct PanickingInput;
            impl Borrow<i32> for PanickingInput {
                fn borrow(&self) -> &i32 {
                    panic!("expected value borrow");
                }
            }
            struct PanickingIterator<T>(core::marker::PhantomData<fn() -> T>);
            impl<T> IntoIterator for PanickingIterator<T> {
                type Item = T;
                type IntoIter = core::iter::Empty<T>;
                fn into_iter(self) -> Self::IntoIter {
                    panic!("expected entries conversion");
                }
            }
            let map = BTreeMap::from([(1, 2)]);
            let failures = assert_that!(map).capture(|root| {
                let child = root.derive(|value| value);
                let panic = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    child.contains_entry::<i32, _>(&1, PanickingInput);
                }));
                assert_that!(panic).is_err();
                assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                let child = root.derive(|value| value);
                let panic = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    child.contains_exactly_entries(PanickingIterator::<(i32, i32)>(
                        core::marker::PhantomData,
                    ));
                }));
                assert_that!(panic).is_err();
                assert_that!(root.state.records.assertion_count()).is_equal_to(2);
                let child = root.derive(|value| value);
                let panic = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    child.contains_keys(PanickingIterator::<i32>(core::marker::PhantomData));
                }));
                assert_that!(panic).is_err();
                assert_that!(root.state.records.assertion_count()).is_equal_to(3);
                let child = root.derive(|value| value);
                let panic = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    type Callback = for<'a> fn(AssertThat<'a, i32, Capture>);
                    child.contains_exactly_entries_satisfying(
                        PanickingIterator::<(i32, Callback)>(core::marker::PhantomData),
                    );
                }));
                assert_that!(panic).is_err();
                assert_that!(root.state.records.assertion_count()).is_equal_to(4);
                root
            });
            assert_that!(failures).is_empty();
        }
    }
}
