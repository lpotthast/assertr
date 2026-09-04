use alloc::vec::Vec;
use core::borrow::Borrow;

use super::{Map, MapKeyQuery, MapLookup, imp};
use crate::{AssertThat, AssertrPartialEq, Mode, ValueRenderer, mode::Capture};

/// Assertions over the keys, values, and entries of a map: `BTreeMap`, `HashMap`, and every type
/// implementing [`Map`].
///
/// Single-key assertions accept `&Q`. Bulk keys implement [`MapKeyQuery<K>`], whose
/// associated query type keeps borrowed lookup inference-friendly. Both go through the map's native
/// [`MapLookup<Q>`], so the bounds are the map's own: `Q: Hash + Eq` on a `HashMap`, `Q: Ord` on a
/// `BTreeMap`. Because the standard maps look up any `Q` their key borrows as, a
/// `HashMap<String, _>` can be queried with a `str`. The [`MapAssertions::Map`] associated type
/// names the subject map so methods can express that requirement. Assertr renders the map
/// structure. The renderer needs capabilities only for keys and values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait MapAssertions<K, V, R> {
    /// The subject map type. Assertions that query a key require it to implement
    /// [`MapLookup`] for the query type.
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
        V: AssertrPartialEq<E, R>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E>;

    /// Asserts that no map value equals `not_expected`.
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        V: AssertrPartialEq<E, R>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E>;

    /// Asserts that the map maps `key` to `value`.
    ///
    /// `value` is accepted owned or borrowed. When the stored value type is itself a reference,
    /// such as `&str`, a borrowed argument satisfies both `Borrow<str>` and `Borrow<&str>` and
    /// the expected type `E` cannot be inferred. Name it with
    /// `contains_entry::<&str, _>("k", "v")`.
    ///
    /// This performs one assertion covering both key presence and value equality. A missing key
    /// is reported once.
    fn contains_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: AssertrPartialEq<E, R>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E>;

    /// Asserts that the map has `key` and its value satisfies `assertions`.
    ///
    /// The assertions run in capture mode against the value. If any fail, this method raises one
    /// map-level failure carrying every captured value failure as a nested failure located at
    /// the key.
    fn contains_entry_satisfying<A, Q>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + Clone;

    /// Asserts that the map does not map `key` to `value`. This passes when the key is absent as
    /// well as when it is present with a different value. Like
    /// [`contains_entry`](MapAssertions::contains_entry), it needs the expected type named when
    /// the stored value type is a reference. Use
    /// `does_not_contain_entry::<&str, _>("k", "v")`.
    fn does_not_contain_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: AssertrPartialEq<E, R>,
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
        V: AssertrPartialEq<EV, R>,
        I: IntoIterator<Item = (EK, EV)>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<EK> + ValueRenderer<EV>;

    /// Asserts that the map contains exactly the given keys and that each value matches the
    /// predicate paired with its key. Missing and unexpected keys are failures.
    fn contains_exactly_entries_matching<EK, P, I>(self, predicates: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Self::Map: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        P: Fn(&V) -> bool,
        I: IntoIterator<Item = (EK, P)>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<EK>;

    /// Asserts that the map contains exactly the given keys and that each value satisfies the
    /// assertions paired with its key. Missing and unexpected keys are failures.
    ///
    /// Each value's assertions run in capture mode. Every captured failure is retained as a
    /// nested failure of the map-level diagnostic, located at its expected key.
    fn contains_exactly_entries_satisfying<EK, A, I>(self, assertions: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Self::Map: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        I: IntoIterator<Item = (EK, A)>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<EK> + Clone;
}

// `K` and `V` are explicit impl parameters rather than written as `Mp::Key` / `Mp::Value`: a
// method bound `Mp: MapLookup<Mp::Key>` is a bounds cycle (E0391), `Mp: MapLookup<K>` is not.
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
        imp::assert_contains_key(&self, expected);
        self
    }

    #[track_caller]
    fn does_not_contain_key<Q>(self, not_expected: &Q) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
    {
        imp::assert_does_not_contain_key(&self, not_expected);
        self
    }

    #[track_caller]
    fn contains_value<E>(self, expected: E) -> Self
    where
        Mp::Value: AssertrPartialEq<E, R>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        imp::assert_contains_value(&self, &expected);
        self
    }

    #[track_caller]
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        Mp::Value: AssertrPartialEq<E, R>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        imp::assert_does_not_contain_value(&self, &not_expected);
        self
    }

    #[track_caller]
    fn contains_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        Mp::Value: AssertrPartialEq<E, R>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
    {
        imp::assert_contains_entry(&self, key, value.borrow());
        self
    }

    #[track_caller]
    fn contains_entry_satisfying<A, Q>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + Clone,
    {
        imp::assert_contains_entry_satisfying(&self, key, &assertions);
        self
    }

    #[track_caller]
    fn does_not_contain_entry<E, Q>(self, key: &Q, value: impl Borrow<E>) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        Mp::Value: AssertrPartialEq<E, R>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
    {
        imp::assert_does_not_contain_entry(&self, key, value.borrow());
        self
    }

    #[track_caller]
    fn contains_keys<E, I>(self, expected: I) -> Self
    where
        E: MapKeyQuery<K>,
        Mp: MapLookup<<E as MapKeyQuery<K>>::Query>,
        I: IntoIterator<Item = E>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
    {
        let expected = expected.into_iter().collect::<Vec<_>>();
        imp::assert_contains_keys(&self, expected.as_slice());
        self
    }

    #[track_caller]
    fn contains_exactly_entries<EK, EV, I>(self, expected: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Mp: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        Mp::Value: AssertrPartialEq<EV, R>,
        I: IntoIterator<Item = (EK, EV)>,
        R: ValueRenderer<Mp::Key>
            + ValueRenderer<Mp::Value>
            + ValueRenderer<EK>
            + ValueRenderer<EV>,
    {
        let expected = expected.into_iter().collect::<Vec<_>>();
        imp::assert_contains_exactly_entries(&self, expected.as_slice());
        self
    }

    #[track_caller]
    fn contains_exactly_entries_matching<EK, P, I>(self, predicates: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Mp: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        P: Fn(&Mp::Value) -> bool,
        I: IntoIterator<Item = (EK, P)>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<EK>,
    {
        let predicates = predicates.into_iter().collect::<Vec<_>>();
        imp::assert_contains_exactly_entries_matching(&self, predicates.as_slice());
        self
    }

    #[track_caller]
    fn contains_exactly_entries_satisfying<EK, A, I>(self, assertions: I) -> Self
    where
        EK: MapKeyQuery<K>,
        Mp: MapLookup<<EK as MapKeyQuery<K>>::Query>,
        A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
        I: IntoIterator<Item = (EK, A)>,
        R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<EK> + Clone,
    {
        let assertions = assertions.into_iter().collect::<Vec<_>>();
        imp::assert_contains_exactly_entries_satisfying(&self, assertions.as_slice());
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use alloc::collections::BTreeMap;

        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, RendererActual, RendererExpected, SentinelRenderer, assert_trait_impl,
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
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("x", 10), ("y", 20), ("z", 30)]));

            assert_that!(failures[0].description())
                .contains("Expected: [")
                .contains("] (... 2 more entries ...)");
            assert_that!(failures[0].facts.iter().any(|fact| {
                fact.label == "Unexpected entries"
                    && fact.value.starts_with('[')
                    && fact.value.contains("] (... 2 more entries ...)")
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
        #[cfg(feature = "derive")]
        fn can_check_for_derived_type() {
            #[derive(Debug, PartialEq, AssertrEq)]
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

            assert_that!(assertion.state.number_of_assertions.borrow().0).is_equal_to(1);
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

        use crate::prelude::*;
        use crate::test_support::{RendererActual, RendererExpected, SENTINEL, SentinelRenderer};

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

                    Actual: BTreeMap {{
                        "retries": 3,
                    }}

                    does not contain key

                    Expected: "timeout"
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
            .contains("does not contain a value satisfying the assertions at a key")
            .contains(
                "Nested failures:\n  - At key \"retries\":\n    Actual: -3\n\n    is not greater than\n\n    Expected: 0\n  - At key \"retries\":\n    Actual: -3\n\n    is not greater than\n\n    Expected: 10\n",
            );
        }

        #[test]
        fn nested_failures_use_the_active_renderer() {
            let failures = assert_that!(BTreeMap::from([("value", RendererActual(1))]))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains_entry_satisfying("value", is_renderer_expected_two));

            assert_that!(failures[0].children[0].description()).contains(SENTINEL);
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
        use crate::prelude::*;
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
                .map(|child| child.facts[0].value.as_str())
                .collect::<Vec<_>>();
            assert_that!(keys).contains_exactly(["\"b\"", "\"a\""]);
        }

        #[test]
        fn nested_failures_are_sorted_by_their_text_for_a_map_rendered_in_sorted_order() {
            let failures = assert_that!(HashMap::from([("a", 1), ("b", 2)]))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("b", 0), ("a", 0)]));

            let keys = failures[0]
                .children
                .iter()
                .map(|child| child.facts[0].value.as_str())
                .collect::<Vec<_>>();
            assert_that!(keys).contains_exactly(["\"a\"", "\"b\""]);
        }

        #[test]
        fn limits_the_nested_value_failures_to_the_rendering_budget() {
            use alloc::collections::BTreeMap;

            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("a", 0), ("b", 0), ("c", 0)]));

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].children[0].facts[0].value.as_str()).is_equal_to("\"a\"");
            assert_that!(failures[0].facts.as_slice())
                .contains(crate::Fact::note("... 2 more unexpected values ..."));
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
                .contain_exactly_entries_matching([("a", is_one)]);
        }

        #[test]
        fn succeeds_when_the_keys_are_exact_and_each_value_matches() {
            let predicates: [(&str, Predicate); 2] = [("b", is_two), ("a", is_one)];

            assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                .contains_exactly_entries_matching(predicates);
        }

        #[test]
        fn accepts_str_queries_for_string_keys() {
            let predicates: [(&str, Predicate); 2] = [("b", is_two), ("a", is_one)];
            let map = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

            assert_that!(map).contains_exactly_entries_matching(predicates);
        }

        #[test]
        fn panics_when_an_expected_key_is_missing() {
            let predicates: [(&str, Predicate); 2] = [("a", is_two), ("missing", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 2)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(predicates);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeMap::from([("a", 2)])`

                    Actual: BTreeMap {{
                        "a": 2,
                    }}

                    does not exactly contain entries matching the predicates

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                      - Keys not found: [
                            "missing",
                        ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_the_expected_keys_repeat_a_key() {
            let predicates: [(&str, Predicate); 2] = [("a", is_one), ("a", is_one)];

            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_matching(predicates);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeMap::from([("a", 1)])`

                    Actual: BTreeMap {{
                        "a": 1,
                    }}

                    does not exactly contain entries matching the predicates

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_an_unexpected_entry_is_present() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1), ("extra", 9)]))
                    .with_location(false)
                    .contains_exactly_entries_matching([("a", is_one)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeMap::from([("a", 1), ("extra", 9)])`

                    Actual: BTreeMap {{
                        "a": 1,
                        "extra": 9,
                    }}

                    does not exactly contain entries matching the predicates

                    Details:
                      - Actual length: 2
                      - Expected length: 1
                      - Unexpected entries: [
                            (
                                "extra",
                                9,
                            ),
                        ]
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_an_expected_value_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(BTreeMap::from([("a", 1)]))
                    .with_location(false)
                    .contains_exactly_entries_matching([("a", is_two)]);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeMap::from([("a", 1)])`

                    Actual: BTreeMap {{
                        "a": 1,
                    }}

                    does not exactly contain entries matching the predicates

                    Nested failures:
                      - At key "a":
                        Actual: 1

                        does not match its predicate
                    -------- assertr --------
                "#});
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
        fn succeeds_when_the_keys_are_exact_and_each_value_satisfies() {
            let assertions: [(&str, ValueAssertions); 2] = [("b", is_two), ("a", is_one)];

            assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                .contains_exactly_entries_satisfying(assertions);
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

                    Actual: BTreeMap {{
                        "a": 2,
                    }}

                    does not exactly contain entries satisfying the assertions

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                      - Keys not found: [
                            "missing",
                        ]
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

                    Actual: BTreeMap {{
                        "a": 1,
                    }}

                    does not exactly contain entries satisfying the assertions

                    Details:
                      - Actual length: 1
                      - Expected length: 2
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

                    Actual: BTreeMap {{
                        "a": 1,
                        "extra": 9,
                    }}

                    does not exactly contain entries satisfying the assertions

                    Details:
                      - Actual length: 2
                      - Expected length: 1
                      - Unexpected entries: [
                            (
                                "extra",
                                9,
                            ),
                        ]
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
            .contains("does not exactly contain entries satisfying the assertions")
            .contains("Nested failures:\n  - At key \"a\":\n    Expected: 2\n\n      Actual: 1\n");
        }

        #[test]
        fn limits_repeated_value_evidence_to_the_rendering_budget() {
            let assertions: [(&str, ValueAssertions); 3] =
                [("a", is_zero), ("b", is_zero), ("c", is_zero)];
            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::builder().max_items(1).build())
                .with_location(false)
                .capture(|it| it.contains_exactly_entries_satisfying(assertions));

            assert_that!(failures[0].children.as_slice()).has_length(1);
            assert_that!(failures[0].children[0].facts[0].value.as_str()).is_equal_to("\"a\"");
            assert_that!(failures[0].facts.as_slice())
                .contains_exactly([crate::Fact::note("... 2 more unsatisfied values ...")]);
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
                .contain_exactly_entries_matching([("foo", is_bar)])
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
                .contains_exactly_entries_matching([("foo", is_bar)])
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
}
