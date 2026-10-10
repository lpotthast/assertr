use super::{
    Map, MapLookup, entries_are,
    entry_matcher_list::SatisfyingEntryList,
    imp,
    matching::{ContainsEntryMatching, ContainsValueMatching},
};
use crate::{
    AssertThat, Mode,
    assertions::map::EntryMatcherList,
    borrow_for::BorrowFor,
    expectation::Expectation,
    matchers::satisfying,
    mode::Capture,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Assertions over the keys, values, and entries of a map: `BTreeMap`, `HashMap`, and every type
/// implementing [`Map`].
///
/// Single-key assertions accept `&Q` directly. Bulk operands implement [`BorrowFor<K>`], selecting
/// `View` with the stored key type `K` as context. Both use native [`MapLookup<Q>`]. Selection
/// grants no lookup capability: hash maps require hashing and equality, tree maps require ordering,
/// and custom maps impose their own bounds. There is no fallback to an equality scan.
///
/// The renderer must render the selected query view, plus the stored keys and values used in
/// diagnostics. Operand wrappers need no renderer or `Clone`. Keyed matchers render the same query
/// for lookup and paths. Exact keyed checks also render stored keys to identify unexpected entries.
///
/// # Bulk query views
///
/// Standard owned, borrowed, mutable-reference, and smart-pointer key operands remain supported.
/// String keys accept `str` views, and vector keys accept slices:
///
/// ```
/// use assertr::{matchers::{entry, eq}, prelude::*};
/// use std::collections::BTreeMap;
/// let map = BTreeMap::from([(vec![1_u8, 2], 3)]);
/// let query = &[1_u8, 2][..];
/// assert_that!(map).contains_key(query).contains_keys([query])
///     .contains_exactly_entries([(query, 3)]).matches(entry(query, eq(3)));
/// ```
///
/// An array operand currently selects an array view in a vector context. That view does not grant
/// native lookup on vector keys. Pass an explicit slice as above instead:
///
/// ```compile_fail
/// use assertr::prelude::*;
/// use std::collections::BTreeMap;
/// let map = BTreeMap::from([(vec![1_u8, 2], 3)]);
/// assert_that!(map).contains_keys([[1_u8, 2]]);
/// ```
///
/// # Custom operands
///
/// A custom bulk operand implements `BorrowFor<K>` and [`Borrow<View>`](core::borrow::Borrow), as
/// described in [borrowed equality](crate#borrowed-expected-values). Custom renderers render
/// `View`, and the subject needs a `MapLookup` implementation for it.
///
/// Bulk value lists use [repeatable expected data](crate#expected-lists).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MapAssertions<K, V, R = DebugRenderer> {
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
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>;

    /// Asserts that no map value equals `not_expected`.
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>;

    /// Asserts that the map maps `key` to `value`.
    ///
    /// `value` can be owned or borrowed through [`BorrowFor<V>`](BorrowFor).
    /// Stored `String` values also accept literals. Reference-valued entries keep their declared
    /// type.
    ///
    /// This performs one assertion covering both key presence and value equality. A missing key is
    /// reported once.
    fn contains_entry<Q, E>(self, key: &Q, value: E) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E::View>;

    /// Asserts that the map has `key` and its value satisfies `assertions`.
    ///
    /// The assertions run in capture mode against the value. If any fail, this method raises one
    /// map-level failure carrying every captured value failure as a nested failure located at the
    /// key.
    fn contains_entry_satisfying<Q, A>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<Q> + Clone;

    /// Asserts that the map does not map `key` to `value`. This passes when the key is absent as
    /// well as when it is present with a different value. Like
    /// [`contains_entry`](MapAssertions::contains_entry), the operand selects a borrowed view
    /// through [`BorrowFor`] for the declared value type.
    fn does_not_contain_entry<Q, E>(self, key: &Q, value: E) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E::View>;

    /// Asserts that every expected key is present. Extra map keys are allowed.
    ///
    /// Accepts an array, slice, vector, or compatible wrapper. Expected data is accessed after
    /// tracking and may be borrowed again for diagnostics. Each query performs one native lookup.
    /// Successful checks allocate no query-view buffer. Collect generators explicitly first.
    fn contains_keys<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: BorrowFor<K>,
        Self::Map: MapLookup<E::View>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>;

    /// Asserts that the map contains exactly the given entries. There are no missing or unexpected
    /// keys, and every value is equal to its expectation.
    ///
    /// Each entry resolves its key and expected value, performs one native lookup, and compares
    /// the value when the key is present. A rejection reports missing keys, unexpected entries, and
    /// a nested failure at each key whose value differs, without repeating lookup or comparison.
    /// Repeated expected keys cannot stand in for missing entries. When they cover every entry
    /// while the lengths differ, the rejection reports both lengths instead.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use std::collections::BTreeMap;
    ///
    /// let map = BTreeMap::from([(String::from("retries"), 3), (String::from("timeout"), 30)]);
    /// assert_that!(map).contains_exactly_entries([("timeout", 30), ("retries", 3)]);
    /// ```
    fn contains_exactly_entries<EK, EV>(self, expected: impl AsRef<[(EK, EV)]>) -> Self
    where
        EK: BorrowFor<K>,
        Self::Map: MapLookup<EK::View>,
        V: PartialEq<EV::View>,
        EV: BorrowFor<V>,
        R: ValueRenderer<K>
            + ValueRenderer<V>
            + ValueRenderer<EK::View>
            + ValueRenderer<usize>
            + ValueRenderer<EV::View>;

    /// Asserts that the map contains exactly the given keys and that each value matches the
    /// matcher paired with its key. Missing and unexpected keys are failures.
    ///
    /// Accepts arrays, slices, and vectors of [`entry`](super::entry) matchers, or a `matchers!`
    /// list of entries with different matcher types. This is the method form of
    /// [`entries_are`](super::entries_are).
    ///
    /// ```
    /// use assertr::{matchers::{entry, eq, gt}, prelude::*};
    /// use std::collections::BTreeMap;
    ///
    /// let map = BTreeMap::from([("retries", 3), ("timeout", 30)]);
    /// assert_that!(map)
    ///     .contains_exactly_entries_matching([entry("retries", gt(0)), entry("timeout", gt(10))])
    ///     .contains_exactly_entries_matching(matchers![
    ///         entry("retries", eq(3)),
    ///         entry("timeout", gt(10)),
    ///     ]);
    /// let expected = [("retries", 3), ("timeout", 30)].map(|(key, value)| entry(key, eq(value)));
    /// assert_that!(map).contains_exactly_entries_matching(expected);
    /// ```
    fn contains_exactly_entries_matching<L>(self, expected: L) -> Self
    where
        L: EntryMatcherList<Self::Map, R>,
        R: ValueRenderer<K> + ValueRenderer<usize>;

    /// Asserts that the map contains exactly the given keys and that each value satisfies the
    /// assertions paired with its key. Missing and unexpected keys are failures.
    ///
    /// Each value's assertions run in capture mode. Every captured failure is retained as a nested
    /// failure of the map-level diagnostic, located at its expected key.
    ///
    /// Accepts an array, slice, vector, or compatible wrapper of key/callback pairs, like the other
    /// bulk methods. Collect generators explicitly first.
    fn contains_exactly_entries_satisfying<EK, A>(self, assertions: impl AsRef<[(EK, A)]>) -> Self
    where
        EK: BorrowFor<K>,
        Self::Map: MapLookup<EK::View>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<K> + ValueRenderer<EK::View> + ValueRenderer<usize> + Clone;

    /// Asserts that the map has `key` and its value matches `expected`.
    ///
    /// The key is looked up natively. A missing key or a rejected value fails with one nested
    /// failure located at the key. Extra entries are allowed.
    ///
    /// ```
    /// use assertr::{matchers::gt, prelude::*};
    /// use std::collections::BTreeMap;
    ///
    /// assert_that!(BTreeMap::from([("retries", 3)])).contains_entry_matching("retries", gt(0));
    /// ```
    fn contains_entry_matching<Q, E>(self, key: &Q, expected: E) -> Self
    where
        Q: ?Sized,
        Self::Map: MapLookup<Q>,
        E: Expectation<V, R>,
        R: ValueRenderer<Q>;

    /// Asserts that some map value matches `expected`.
    fn contains_value_matching<E>(self, expected: E) -> Self
    where
        E: Expectation<V, R>;
}

// Explicit stored key and value parameters keep mutually dependent method bounds cycle-free.
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
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q>,
    {
        self.matches(imp::ContainsKey::new(expected))
    }

    #[track_caller]
    fn does_not_contain_key<Q>(self, not_expected: &Q) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q>,
    {
        self.matches(imp::DoesNotContainKey::new(not_expected))
    }

    #[track_caller]
    fn contains_value<E>(self, expected: E) -> Self
    where
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>,
    {
        self.matches(imp::ContainsValue::new(expected))
    }

    #[track_caller]
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>,
    {
        self.matches(imp::DoesNotContainValue::new(not_expected))
    }

    #[track_caller]
    fn contains_entry<Q, E>(self, key: &Q, value: E) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E::View>,
    {
        self.matches(imp::ContainsEntry::new(key, value))
    }

    #[track_caller]
    fn contains_entry_satisfying<Q, A>(self, key: &Q, assertions: A) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<Q> + Clone,
    {
        self.contains_entry_matching(key, satisfying(assertions))
    }

    #[track_caller]
    fn does_not_contain_entry<Q, E>(self, key: &Q, value: E) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        V: PartialEq<E::View>,
        E: BorrowFor<V>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<Q> + ValueRenderer<E::View>,
    {
        self.matches(imp::DoesNotContainEntry::new(key, value))
    }

    #[track_caller]
    fn contains_keys<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: BorrowFor<K>,
        Mp: MapLookup<E::View>,
        R: ValueRenderer<K> + ValueRenderer<V> + ValueRenderer<E::View>,
    {
        self.matches(imp::ContainsKeys::new(expected))
    }

    #[track_caller]
    fn contains_exactly_entries<EK, EV>(self, expected: impl AsRef<[(EK, EV)]>) -> Self
    where
        EK: BorrowFor<K>,
        Mp: MapLookup<EK::View>,
        V: PartialEq<EV::View>,
        EV: BorrowFor<V>,
        R: ValueRenderer<K>
            + ValueRenderer<usize>
            + ValueRenderer<V>
            + ValueRenderer<EK::View>
            + ValueRenderer<EV::View>,
    {
        self.matches(imp::ContainsExactlyEntries::new(expected))
    }

    #[track_caller]
    fn contains_exactly_entries_matching<L>(self, expected: L) -> Self
    where
        L: EntryMatcherList<Self::Map, R>,
        R: ValueRenderer<K> + ValueRenderer<usize>,
    {
        self.matches(entries_are(expected))
    }

    #[track_caller]
    fn contains_exactly_entries_satisfying<EK, A>(self, assertions: impl AsRef<[(EK, A)]>) -> Self
    where
        EK: BorrowFor<K>,
        Mp: MapLookup<EK::View>,
        A: for<'a> Fn(AssertThat<'a, V, Capture, R>),
        R: ValueRenderer<K> + ValueRenderer<EK::View> + ValueRenderer<usize> + Clone,
    {
        self.matches(entries_are(SatisfyingEntryList::new(assertions)))
    }

    #[track_caller]
    fn contains_entry_matching<Q, E>(self, key: &Q, expected: E) -> Self
    where
        Q: ?Sized,
        Mp: MapLookup<Q>,
        E: Expectation<V, R>,
        R: ValueRenderer<Q>,
    {
        self.matches(ContainsEntryMatching::new(key, expected))
    }

    #[track_caller]
    fn contains_value_matching<E>(self, expected: E) -> Self
    where
        E: Expectation<V, R>,
    {
        self.matches(ContainsValueMatching::new(expected))
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use crate::prelude::*;

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;
        use crate::matchers::{entry, eq};

        fn is_bar(it: AssertThat<&str, Capture>) {
            it.is_equal_to("bar");
        }

        #[test]
        fn are_as_expected() {
            BTreeMap::from([("foo", "bar")])
                .must()
                .contain_key("foo")
                .not_contain_key("baz")
                .contain_value("bar")
                .not_contain_value("baz")
                .contain_entry::<_, &str>("foo", "bar")
                .contain_entry_satisfying("foo", is_bar)
                .not_contain_entry::<_, &str>("foo", "baz")
                .contain_keys(["foo"])
                .contain_exactly_entries([("foo", "bar")])
                .contain_exactly_entries_matching([entry("foo", eq("bar"))])
                .contain_exactly_entries_satisfying([("foo", is_bar)])
                .contain_entry_matching("foo", eq("bar"))
                .contain_value_matching(eq("bar"));
        }
    }

    mod renderer_contract {
        use super::*;
        use crate::test_support::{
            ComparisonRenderer, CustomValueRenderer, NoRenderer, RendererActual, RendererExpected,
            assert_custom_fact, assert_trait_impl,
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
                .with_renderer(ComparisonRenderer)
                .contains_value(RendererExpected::new(1))
                .contains_entry("a", RendererExpected::new(1))
                .contains_exactly_entries([("a", RendererExpected::new(1))]);
        }

        #[test]
        fn length_facts_use_the_active_renderer() {
            let failures = assert_that!(BTreeMap::from([(1, 2)]))
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.contains_exactly_entries([(1, 2), (1, 2)]));
            assert_custom_fact(&failures[0], "Actual length", 1);
            assert_custom_fact(&failures[0], "Expected length", 2);
        }

        #[test]
        fn value_matching_requires_neither_key_nor_value_rendering() {
            assert_that!(BTreeMap::from([("a", 1)]))
                .with_renderer(NoRenderer)
                .contains_value_matching(crate::test_support::opaque_predicate(|x: &i32| *x == 1));
        }
    }

    mod rendering_budget {
        use super::*;

        #[test]
        fn limits_complete_expected_and_unexpected_entry_groups() {
            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2), ("c", 3)]))
                .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                .with_location(false)
                .capture(|it| it.contains_exactly_entries([("x", 10), ("y", 20), ("z", 30)]));

            assert_that!(failures[0].to_string())
                .contains("Expected: [")
                .contains("] (... 2 more elements ...)");
            assert_that!(failures[0].facts.iter().any(|fact| {
                fact.label.as_deref() == Some("Unexpected entries")
                    && format!("{:#}", fact.value).starts_with('[')
                    && format!("{:#}", fact.value).contains("] (... 2 more elements ...)")
            }))
            .is_true();
        }

        #[test]
        fn zero_budget_does_not_render_keys() {
            struct NeverRender;
            impl ValueRenderer<str> for NeverRender {
                fn fmt(&self, _: &str, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("omitted key rendered")
                }
            }
            let failures = assert_that!(BTreeMap::from([("a", 1)]))
                .with_renderer(NeverRender)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.contains_entry_matching("b", matchers::anything()));
            assert_that!(failures[0].omitted_children).is_equal_to(1);
        }
    }

    mod contains_key {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", 1)])),
                contains_key("baz")
            );
        }

        #[test]
        fn succeeds_when_key_is_present() {
            assert_that!(BTreeMap::from([("foo", "bar")])).contains_key("foo");
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map).with_location(false).contains_key("baz");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                does not contain key

                Expected: "baz"
                -------- assertr --------
            "#});
        }
    }

    mod does_not_contain_key {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", 1)])),
                does_not_contain_key("foo")
            );
        }

        #[test]
        fn succeeds_when_key_is_absent() {
            assert_that!(BTreeMap::from([("foo", "bar")])).does_not_contain_key("baz");
        }

        #[test]
        fn panics_when_key_is_present() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_key("foo");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                contains key

                Unexpected: "foo"
                -------- assertr --------
            "#});
        }
    }

    mod contains_value {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", "bar")])),
                contains_value("baz")
            );
        }

        #[test]
        fn succeeds_when_value_is_present() {
            #[derive(Debug, PartialEq)]
            struct Data {
                data: u32,
            }

            assert_that!(BTreeMap::from([("foo", "bar")])).contains_value("bar");
            let map = BTreeMap::from([("foo", Data { data: 0 })]);
            assert_that!(&map).contains_value(Data { data: 0 });
        }

        #[test]
        fn panics_when_value_is_absent() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map).with_location(false).contains_value("baz");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                does not contain value

                Expected: "baz"
                -------- assertr --------
            "#});
        }
    }

    mod does_not_contain_value {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", "bar")])),
                does_not_contain_value("bar")
            );
        }

        #[test]
        fn succeeds_when_value_is_absent() {
            assert_that!(BTreeMap::from([("foo", "bar")])).does_not_contain_value("baz");
        }

        #[test]
        fn panics_when_value_is_present() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_value("bar");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                contains value

                Unexpected: "bar"
                -------- assertr --------
            "#});
        }
    }

    mod contains_entry {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", "bar")])),
                contains_entry("baz", "someValue")
            );
        }

        #[test]
        fn succeeds_when_entry_is_present() {
            #[derive(Debug, PartialEq)]
            struct Person {
                age: u32,
            }

            let map = BTreeMap::from([("foo", Person { age: 42 })]);
            assert_that!(&map)
                .contains_entry("foo", &Person { age: 42 })
                .contains_entry("foo", Person { age: 42 })
                .contains_entry("foo", Box::new(Person { age: 42 }));
        }

        #[test]
        fn reports_a_missing_key_once() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_entry("baz", "someValue");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                does not contain key

                Expected: "baz"
                -------- assertr --------
            "#});
        }

        #[test]
        fn reports_a_value_mismatch_at_its_key() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_entry("foo", "someValue");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                does not contain the expected value at a key

                Nested failures:
                  - At ["foo"]:
                    Expected: "someValue"

                      Actual: "bar"
                -------- assertr --------
            "#});
        }
    }

    /// Delegates to `contains_entry_matching`, which covers the behavior.
    mod contains_entry_satisfying {
        use super::*;

        fn is_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("retries", 3)])),
                contains_entry_satisfying("timeout", is_three)
            );
        }

        #[test]
        fn succeeds_when_the_value_satisfies_the_assertions() {
            assert_that!(BTreeMap::from([("retries", 3)]))
                .contains_entry_satisfying("retries", is_three);
        }
    }

    mod does_not_contain_entry {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", "bar")])),
                does_not_contain_entry("foo", "bar")
            );
        }

        #[test]
        fn succeeds_when_key_is_absent_or_value_differs() {
            assert_that!(BTreeMap::from([("foo", "bar")]))
                .does_not_contain_entry("baz", "bar")
                .does_not_contain_entry("foo", "baz");
        }

        #[test]
        fn panics_when_entry_is_present() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_entry("foo", "bar");
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

                contains the entry

                Unexpected: (
                    "foo",
                    "bar",
                )
                -------- assertr --------
            "#});
        }
    }

    mod contains_keys {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("foo", "bar")])),
                contains_keys(["foo", "baz"])
            );
        }

        #[test]
        fn succeeds_when_all_keys_are_present() {
            assert_that!(BTreeMap::from([("foo", 1), ("bar", 2)])).contains_keys(["foo", "bar"]);
        }

        #[test]
        fn panics_when_a_key_is_missing() {
            assert_that!(|| {
                let map = BTreeMap::from([("foo", "bar")]);
                assert_that!(map)
                    .with_location(false)
                    .contains_keys(["foo", "baz"]);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "foo": "bar",
                }}

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

    mod contains_exactly_entries {
        use indoc::formatdoc;

        use super::*;

        fn report(expected: &[(&'static str, i32)]) -> String {
            let map = BTreeMap::from([("a", 1)]);
            assert_that!(map)
                .with_location(false)
                .capture(|it| it.contains_exactly_entries(expected))[0]
                .to_string()
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_exactly_entries([("a", 1), ("a", 1)])
            );
        }

        #[test]
        fn succeeds_when_entries_match() {
            let map = BTreeMap::from([("foo", "bar"), ("baz", "qux")]);
            assert_that!(&map).contains_exactly_entries([("foo", "bar"), ("baz", "qux")]);
            assert_that!(map).contains_exactly_entries(map.clone().into_iter().collect::<Vec<_>>());
        }

        #[test]
        fn reports_missing_keys() {
            assert_that!(report(&[("a", 1), ("b", 2)])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "a": 1,
                }}

                does not contain exactly

                Expected: [
                    (
                        "a",
                        1,
                    ),
                    (
                        "b",
                        2,
                    ),
                ]

                Details:
                  - Keys not found: [
                        "b",
                    ]
                -------- assertr --------
            "#});
        }

        #[test]
        fn reports_unexpected_entries() {
            assert_that!(report(&[])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "a": 1,
                }}

                does not contain exactly

                Expected: []

                Details:
                  - Unexpected entries: [
                        (
                            "a",
                            1,
                        ),
                    ]
                -------- assertr --------
            "#});
        }

        #[test]
        fn reports_lengths_only_when_repeated_keys_match_every_entry() {
            assert_that!(report(&[("a", 1), ("a", 1)])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "a": 1,
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
                  - Actual length: 1
                  - Expected length: 2
                -------- assertr --------
            "#});
            let failures = assert_that!(BTreeMap::from([("a", 1)]))
                .capture(|it| it.contains_exactly_entries([("a", 2), ("a", 2)]));
            assert_that!(failures[0].facts).is_empty();
            assert_that!(failures[0].children).has_length(2);
        }

        /// A repeated expected key makes the counts agree, so the extra actual entry can only be
        /// found by knowing which stored entries the expectations resolved to.
        #[test]
        fn finds_an_unexpected_entry_hidden_behind_matching_counts() {
            let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                .capture(|it| it.contains_exactly_entries([("a", 1), ("a", 1)]));
            assert_that!(
                failures[0]
                    .facts
                    .iter()
                    .map(|fact| fact.label.as_deref())
                    .collect::<Vec<_>>()
            )
            .contains_exactly([Some("Unexpected entries")]);
        }

        #[test]
        fn reports_value_mismatches_at_their_keys() {
            assert_that!(report(&[("a", 2)])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: BTreeMap {{
                    "a": 1,
                }}

                does not contain exactly

                Expected: [
                    (
                        "a",
                        2,
                    ),
                ]

                Nested failures:
                  - At ["a"]:
                    Expected: 2

                      Actual: 1
                -------- assertr --------
            "#});
        }

        #[test]
        fn limits_nested_value_failures_in_the_maps_diagnostic_order() {
            use crate::assertions::map::MapLookup;

            fn check(
                map: impl MapLookup<&'static str, Key = &'static str, Value = i32>,
                keys: [&str; 3],
            ) {
                for limit in [0, 1, 2, 3, 4, usize::MAX] {
                    let failures = assert_that!(map)
                        .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                        .with_location(false)
                        .capture(|it| it.contains_exactly_entries([("b", 0), ("a", 0), ("c", 0)]));
                    let retained = limit.min(keys.len());
                    let actual_keys = failures[0]
                        .children
                        .iter()
                        .map(|child| {
                            assert_that!(child.facts).is_empty();
                            let [crate::failure::PathSegment::Key(key)] = child.path.as_slice()
                            else {
                                panic!("expected one key path segment");
                            };
                            format!("{key:#}")
                        })
                        .collect::<Vec<_>>();
                    assert_that!(actual_keys).is_equal_to(&keys[..retained]);
                    assert_that!(failures[0].omitted_children).is_equal_to(keys.len() - retained);
                    assert_that!(failures[0].facts).is_empty();
                }
            }
            let entries = [("a", 3), ("b", 2), ("c", 1)];
            check(BTreeMap::from(entries), ["\"b\"", "\"a\"", "\"c\""]);
            #[cfg(feature = "std")]
            check(
                std::collections::HashMap::from(entries),
                ["\"a\"", "\"b\"", "\"c\""],
            );
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    mod contains_exactly_entries_matching {
        use indoc::formatdoc;

        use super::*;
        use crate::matchers::{entry, eq, predicate};

        fn is_one(value: &i32) -> bool {
            *value == 1
        }

        fn report(actual: &[(&'static str, i32)], keys: &[&'static str]) -> String {
            let map = BTreeMap::from_iter(actual.iter().copied());
            assert_that!(map).with_location(false).capture(|it| {
                it.contains_exactly_entries_matching(
                    keys.iter()
                        .map(|key| entry(*key, predicate(is_one)))
                        .collect::<Vec<_>>(),
                )
            })[0]
                .to_string()
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_exactly_entries_matching([entry("a", eq(2))])
            );
        }

        #[test]
        fn succeeds_when_the_keys_are_exact_and_each_value_matches() {
            assert_that!(BTreeMap::from([
                (String::from("a"), 1),
                (String::from("b"), 2)
            ]))
            .contains_exactly_entries_matching([entry("b", eq(2)), entry("a", eq(1))])
            .contains_exactly_entries_matching(crate::matchers![
                entry("b", eq(2)),
                entry("a", predicate(is_one)),
            ]);
        }

        #[test]
        fn reports_missing_keys() {
            assert_that!(report(&[("a", 1)], &["a", "missing"])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

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
        fn reports_lengths_only_when_repeated_keys_match_every_entry() {
            assert_that!(report(&[("a", 1)], &["a", "a"])).is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `map`

                does not match

                Nested failures:
                  - does not have the required number of entries

                    Details:
                      - Actual length: 1
                      - Expected length: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn reports_unexpected_keys() {
            assert_that!(report(&[("a", 1), ("extra", 9)], &["a"])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                does not match

                Nested failures:
                  - At ["extra"]:
                    has an unexpected key
                -------- assertr --------
            "#});
        }

        #[test]
        fn reports_value_mismatches_at_their_keys() {
            assert_that!(report(&[("a", 2)], &["a"])).is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                does not match

                Nested failures:
                  - At ["a"]:
                    Actual: 2

                    does not satisfy the constraint

                    Constraint:
                        satisfies the predicate
                -------- assertr --------
            "#});
        }

        #[test]
        fn keyed_lists_can_be_reused_by_reference() {
            let map = BTreeMap::from([("a", 1)]);
            let expected = crate::matchers![entry("a", eq(1))];
            assert_that!(map)
                .contains_exactly_entries_matching(&expected)
                .contains_exactly_entries_matching(&expected);
        }

        #[test]
        fn supports_large_homogeneous_keyed_lists() {
            let map: BTreeMap<_, _> = (0..4096).map(|key| (key, key)).collect();
            let expected = (0..4096)
                .rev()
                .map(|key| entry(key, eq(key)))
                .collect::<Vec<_>>();

            assert_that!(map).contains_exactly_entries_matching(expected);
        }

        #[test]
        #[cfg(feature = "std")]
        fn requires_no_ordering_for_hash_map_keys() {
            #[derive(Debug, PartialEq, Eq, Hash)]
            struct Key(u8);

            let map = std::collections::HashMap::from([(Key(1), 10), (Key(2), 20)]);
            assert_that!(map)
                .contains_exactly_entries_matching([entry(Key(2), eq(20)), entry(Key(1), eq(10))]);
        }
    }

    mod contains_exactly_entries_satisfying {
        use super::*;

        fn is_two(it: AssertThat<i32, Capture>) {
            it.is_equal_to(2);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_exactly_entries_satisfying([("a", is_two)])
            );
        }

        #[test]
        fn adapts_collected_keyed_callbacks_with_borrowed_queries() {
            let map = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);
            let expected = [(String::from("b"), 2), (String::from("a"), 1)];
            let calls = core::cell::Cell::new(0);
            let checks = expected
                .iter()
                .map(|(key, value)| {
                    let calls = &calls;
                    (key.as_str(), move |it: AssertThat<i32, Capture>| {
                        calls.set(calls.get() + 1);
                        it.is_equal_to(*value);
                    })
                })
                .collect::<Vec<_>>();

            assert_that!(map).contains_exactly_entries_satisfying(&checks);
            assert_that!(map).contains_exactly_entries_satisfying(checks.as_slice());
            assert_that!(map).contains_exactly_entries_satisfying(checks);
            assert_that!(calls.get()).is_equal_to(6);
        }

        #[test]
        fn reports_like_entry_matchers() {
            let map = BTreeMap::from([(1, 1), (2, 2)]);
            let adapted = assert_that!(map)
                .with_location(false)
                .capture(|it| it.contains_exactly_entries_satisfying([(1, is_two), (3, is_two)]));
            let prepared = assert_that!(map).with_location(false).capture(|it| {
                it.contains_exactly_entries_matching([
                    matchers::entry(1, matchers::satisfying(is_two)),
                    matchers::entry(3, matchers::satisfying(is_two)),
                ])
            });
            assert_that!(adapted).has_length(1);
            assert_that!(adapted).is_equal_to(prepared);
        }
    }

    mod contains_entry_matching {
        use super::*;
        use crate::{assertions::core::partial_eq::eq, matchers::anything};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_entry_matching("a", eq(2))
            );
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that!(|| {
                let map = BTreeMap::from([("a", 1)]);
                assert_that!(map)
                    .with_location(false)
                    .contains_entry_matching("b", eq(1));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                does not contain a matching entry

                Nested failures:
                  - At ["b"]:
                    does not satisfy the constraint

                    Constraint:
                        contains the required key
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_when_value_does_not_match() {
            assert_that!(|| {
                let map = BTreeMap::from([("a", 1)]);
                assert_that!(map)
                    .with_location(false)
                    .contains_entry_matching("a", eq(2));
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                does not contain a matching entry

                Nested failures:
                  - At ["a"]:
                    Expected: 2

                      Actual: 1
                -------- assertr --------
            "#});
        }

        #[test]
        fn preserves_borrowed_lookup_and_key_paths() {
            let map = BTreeMap::from([(String::from("a"), 1)]);
            assert_that!(map).contains_entry_matching("a", eq(1));
            let failures =
                assert_that!(map).capture(|it| it.contains_entry_matching("b", anything()));
            assert_that!(&failures[0].children[0].path[0])
                .is_matching(pattern!(crate::failure::PathSegment::Key(_)));
        }
    }

    mod contains_value_matching {
        use super::*;
        use crate::matchers::predicate;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])),
                contains_value_matching(crate::assertions::core::partial_eq::eq(2))
            );
        }

        #[test]
        fn stays_one_nested_group_inside_compositions() {
            use crate::{assertions::map::ContainsValueMatching, matchers::all_of};

            let failures = assert_that!(BTreeMap::from([("a", 2), ("b", 3)]))
                .with_location(false)
                .capture(|it| {
                    it.matches(all_of(matchers![
                        ContainsValueMatching::new(predicate(|x: &i32| *x == 1)),
                        ContainsValueMatching::new(predicate(|x: &i32| *x == 4))
                    ]))
                });
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure.has_text_report(indoc::indoc! {r#"
                    -------- assertr --------
                    Expression: `BTreeMap::from([("a", 2), ("b", 3)])`

                    does not match

                    Nested failures:
                      - does not contain a matching value

                        Nested failures:
                          - Actual: 2

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate

                          - Actual: 3

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate

                      - does not contain a matching value

                        Nested failures:
                          - Actual: 2

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate

                          - Actual: 3

                            does not satisfy the constraint

                            Constraint:
                                satisfies the predicate
                    -------- assertr --------
                    "#});
                },
            ]);
        }

        #[test]
        fn sorts_candidate_evidence_before_limiting_it() {
            use crate::{assertions::core::partial_eq::eq, test_support::UnorderedMap};

            let capture = |entries| {
                let actual = UnorderedMap(entries);
                assert_that!(actual)
                    .with_location(false)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                    .capture(|it| it.contains_value_matching(eq(9)))
            };
            let expected = capture(vec![(1, 1), (2, 2), (3, 3)]);
            let actual = capture(vec![(3, 3), (2, 2), (1, 1)]);
            assert_that!(actual[0].children).has_length(1);
            assert_that!(actual[0].omitted_children).is_equal_to(2);
            assert_that!(actual[0].to_string()).is_equal_to(expected[0].to_string());
        }
    }

    mod operands {
        use core::cell::Cell;

        use super::*;
        use crate::{
            assertions::map::{
                ContainsEntry, ContainsExactlyEntries, ContainsValue, DoesNotContainEntry,
                DoesNotContainValue,
            },
            matchers::{entries_are, entry, eq},
            test_support::{StrOperand, StringRenderer},
        };

        #[test]
        fn borrowed_and_unsized_views_work_with_native_string_key_queries() {
            let expected = String::from("value");
            let absent = String::from("absent");
            let values = BTreeMap::from([(String::from("key"), String::from("value"))]);
            assert_that!(values)
                .contains_key("key")
                .contains_keys(["key"])
                .contains_value(&expected)
                .does_not_contain_value(&absent)
                .contains_entry("key", &expected)
                .does_not_contain_entry("key", &absent)
                .contains_exactly_entries([("key", &expected)])
                .contains_value("value")
                .does_not_contain_value("absent")
                .contains_entry("key", "value")
                .does_not_contain_entry("key", "absent")
                .contains_exactly_entries([("key", "value")])
                .matches(ContainsValue::new(&expected))
                .matches(DoesNotContainValue::new("absent"))
                .matches(ContainsEntry::new("key", &expected))
                .matches(DoesNotContainEntry::new("key", "absent"))
                .matches(ContainsExactlyEntries::new([("key", &expected)]));
        }

        #[test]
        fn value_views_are_borrowed_once_and_retained_for_diagnostics() {
            for method in 0..5 {
                let calls = Cell::new(0);
                let expected = StrOperand {
                    value: if method == 1 || method == 3 {
                        "hello"
                    } else {
                        "world"
                    },
                    observe: || calls.set(calls.get() + 1),
                };
                let failures = assert_that!(BTreeMap::from([(0_usize, String::from("hello"))]))
                    .with_renderer(StringRenderer)
                    .capture(|it| match method {
                        0 => it.contains_value(expected),
                        1 => it.does_not_contain_value(expected),
                        2 => it.contains_entry(&0, expected),
                        3 => it.does_not_contain_entry(&0, expected),
                        _ => it.contains_exactly_entries([(0, expected)]),
                    });
                // Bulk expected data may be borrowed again for diagnostics.
                if method < 4 {
                    assert_that!(calls.get()).is_equal_to(1);
                } else {
                    assert_that!(calls.get()).is_greater_than(0);
                }
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn slice_queries_work_for_bulk_keys_and_all_keyed_compositions() {
            let actual = BTreeMap::from([(alloc::vec![1_u8, 2], 3)]);
            let query = &[1_u8, 2][..];
            assert_that!(actual)
                .contains_key(query)
                .contains_keys([query])
                .contains_exactly_entries([(query, 3)])
                .matches(entry(query, eq(3)))
                .matches(entries_are([entry(query, eq(3))]))
                .contains_exactly_entries_matching([entry(query, eq(3))])
                .contains_exactly_entries_satisfying([(query, |it: AssertThat<i32, Capture>| {
                    it.is_equal_to(3);
                })])
                .matches(matchers::all_of([entry(query, eq(3))]));
        }

        #[test]
        fn missing_subject_descriptions_preserve_negative_operand_roles() {
            use crate::assertions::map::DoesNotContainKey;

            let map = BTreeMap::<i32, i32>::new();
            let root = assert_that!(map);
            let context = root.assertion_context();
            let descriptions = [
                context.describe::<BTreeMap<i32, i32>, _>(&DoesNotContainKey::new(&1)),
                context.describe::<BTreeMap<i32, i32>, _>(&DoesNotContainValue::new(2)),
                context.describe::<BTreeMap<i32, i32>, _>(&DoesNotContainEntry::new(&1, 2)),
            ];
            for description in descriptions {
                assert_that!(description.expected).is_none();
                assert_that!(description.unexpected).is_some();
            }
        }
    }

    /// Native lookups, comparisons, and operand borrows, observed through the shared fixture.
    mod observation {
        use core::cell::Cell;

        use super::*;
        use crate::{
            assertions::map::{
                ContainsExactlyEntries, ContainsKeys,
                fixture::{
                    Events, Inputs, ObservedMap, Query, Value, count, take_observations, value,
                },
            },
            matchers::{anything, entries_are, entry, eq, satisfying},
            test_support::StringRenderer,
        };

        #[test]
        fn single_entry_checks_borrow_and_look_up_once() {
            for key in ["a", "missing"] {
                let compares = usize::from(key == "a");
                let events = Events::default();
                let map = ObservedMap::new(&events, &[("a", "value")]);
                let observed = || {
                    let observed = (
                        count(&events, "value"),
                        count(&events, "lookup"),
                        count(&events, "compare"),
                    );
                    events.take();
                    observed
                };
                let it = || assert_that!(map).with_renderer(StringRenderer);

                let failures = it().capture(|it| it.contains_entry(key, value(&events, "other")));
                assert_that!(failures).has_length(1);
                assert_that!(observed()).is_equal_to((1, 1, compares));

                let failures =
                    it().capture(|it| it.does_not_contain_entry(key, value(&events, "value")));
                assert_that!(failures).has_length(compares);
                assert_that!(observed()).is_equal_to((1, 1, compares));

                let failures = it().capture(|it| {
                    it.matches(entry(
                        Query::once(key, &events),
                        eq(value(&events, "other")),
                    ))
                });
                assert_that!(failures[0].to_string()).does_not_contain("later");
                assert_that!(count(&events, "key")).is_equal_to(1);
                assert_that!(observed()).is_equal_to((compares, 1, compares));
            }
        }

        #[test]
        fn bulk_checks_look_up_each_key_once_in_input_order() {
            let events = Events::default();
            let map = ObservedMap::new(&events, &[("a", "value")]);
            let failures = assert_that!(map)
                .with_renderer(StringRenderer)
                .capture(|it| {
                    it.contains_keys(Inputs {
                        values: alloc::vec![Query::new("a", &events), Query::new("b", &events)],
                        events: &events,
                    })
                });
            assert_that!(failures[0].to_string()).contains("\"b\"");
            assert_that!(take_observations(&events)).contains_exactly(["lookup", "lookup"]);

            let failures = assert_that!(map)
                .with_renderer(StringRenderer)
                .capture(|it| {
                    it.contains_exactly_entries(Inputs {
                        values: alloc::vec![
                            (Query::new("a", &events), value(&events, "wrong")),
                            (Query::new("b", &events), value(&events, "value")),
                        ],
                        events: &events,
                    })
                });
            assert_that!(failures[0].children).has_length(1);
            // Each entry resolves its key before its value and looks up the key once. The
            // diagnostics repeat no lookup or comparison.
            assert_that!(&events.borrow()[..4]).contains_exactly([
                "container",
                "key",
                "value",
                "lookup",
            ]);
            assert_that!(take_observations(&events))
                .contains_exactly(["lookup", "compare", "lookup"]);
        }

        #[test]
        fn keyed_matcher_lists_look_up_each_query_once() {
            for text in ["value", "wrong"] {
                let events = Events::default();
                let map = ObservedMap::new(&events, &[("a", "value"), ("b", "value")]);
                let failures = assert_that!(map)
                    .with_renderer(StringRenderer)
                    .capture(|it| {
                        it.matches(entries_are(crate::matchers![
                            entry(Query::once("a", &events), eq(value(&events, text))),
                            entry("b", anything()),
                        ]))
                    });
                assert_that!(count(&events, "key")).is_equal_to(1);
                assert_that!(count(&events, "lookup")).is_equal_to(2);
                if text == "value" {
                    assert_that!(failures).is_empty();
                } else {
                    // The rejected value occupies its original key. There is no unexpected key.
                    assert_that!(failures[0].children).has_length(1);
                    let child = &failures[0].children[0];
                    assert_that!(child.kind).is_equal_to(crate::failure::FailureKind::Equality);
                    assert_that!(&child.path).contains_exactly_matching([pattern!(
                        crate::failure::PathSegment::Key(key) if format!("{key:#}") == "\"a\""
                    )]);
                }
            }
        }

        #[test]
        fn probes_resolve_queries_anew_and_render_nothing() {
            let events = Events::default();
            let map = ObservedMap::new(&events, &[("a", "value")]);
            let root = assert_that!(map).with_renderer(crate::test_support::PanickingRenderer(
                "probe rendered a diagnostic leaf",
            ));
            let context = root.assertion_context();
            let matcher = entry(Query::once("a", &events), anything());
            assert_that!(context.probe(&map, &matcher)).is_true();
            assert_that!(events.take()).contains_exactly(["key", "lookup"]);
            // The query now resolves to a different key.
            assert_that!(context.probe(&map, &matcher)).is_false();
            assert_that!(events.take()).contains_exactly(["key", "lookup"]);
            let exact = entries_are([entry(Query::new("missing", &events), anything())]);
            assert_that!(context.probe(&map, &exact)).is_false();
            assert_that!(events.take()).contains_exactly(["key", "lookup"]);
            let keys = ContainsKeys::new([Query::new("missing", &events)]);
            assert_that!(context.probe(&map, &keys)).is_false();
            assert_that!(take_observations(&events)).contains_exactly(["lookup"]);
            let exact =
                ContainsExactlyEntries::new([(Query::new("a", &events), value(&events, "wrong"))]);
            assert_that!(context.probe(&map, &exact)).is_false();
            assert_that!(take_observations(&events)).contains_exactly(["lookup", "compare"]);
        }

        #[test]
        fn descriptions_borrow_only_budgeted_queries_without_lookups_or_callbacks() {
            let events = Events::default();
            let callbacks = Cell::new(0);
            let map = ObservedMap::new(&events, &[("a", "value")]);
            let root = assert_that!(map).with_renderer(StringRenderer);
            let context = root.assertion_context();
            let matcher = entry(
                Query::new("a", &events),
                satisfying(|_: AssertThat<Value, Capture, StringRenderer>| {
                    callbacks.set(callbacks.get() + 1);
                }),
            );
            let description = context.describe::<ObservedMap, _>(&matcher);
            assert_that!(description.relation.as_deref())
                .is_equal_to(Some("contains a matching entry"));
            assert_that!(events.take()).contains_exactly(["key"]);
            let _ = context.describe::<ObservedMap, _>(&entries_are([matcher]));
            assert_that!(events.take()).contains_exactly(["key"]);
            assert_that!(callbacks.get()).is_equal_to(0);

            let omitted = Events::default();
            let keys = [Query::new("a", &events), Query::new("b", &omitted)];
            let entries = [
                (Query::new("a", &events), Query::new("value", &events)),
                (Query::new("b", &omitted), Query::new("value", &omitted)),
            ];
            let context = AssertionContext::new(
                &StringRenderer,
                RenderingBudget::default().with_max_items(1),
            );
            let _ = context.describe::<BTreeMap<String, String>, _>(&ContainsKeys::new(&keys));
            let _ = context
                .describe::<BTreeMap<String, String>, _>(&ContainsExactlyEntries::new(&entries));
            assert_that!(&*events.borrow()).contains("key");
            assert_that!(&*omitted.borrow()).is_empty();
        }

        #[test]
        fn selected_views_match_literal_diagnostics_including_paths() {
            let actual = BTreeMap::from([(String::from("a"), String::from("value"))]);
            let events = Events::default();
            let wrapped = assert_that!(actual)
                .with_renderer(StringRenderer)
                .with_location(false)
                .capture(|it| {
                    it.contains_keys([Query::new("missing", &events)])
                        .contains_exactly_entries([(Query::new("a", &events), "wrong")])
                        .matches(entry(Query::new("a", &events), eq("wrong")))
                        .matches(entries_are([entry(Query::new("a", &events), eq("wrong"))]))
                        .contains_exactly_entries_satisfying([(
                            Query::new("a", &events),
                            |it: AssertThat<String, Capture, StringRenderer>| {
                                it.is_equal_to("wrong");
                            },
                        )])
                });
            let literal = assert_that!(actual)
                .with_renderer(StringRenderer)
                .with_location(false)
                .capture(|it| {
                    it.contains_keys(["missing"])
                        .contains_exactly_entries([("a", "wrong")])
                        .matches(entry("a", eq("wrong")))
                        .matches(entries_are([entry("a", eq("wrong"))]))
                        .contains_exactly_entries_satisfying([(
                            "a",
                            |it: AssertThat<String, Capture, StringRenderer>| {
                                it.is_equal_to("wrong");
                            },
                        )])
                });
            assert_that!(wrapped).is_equal_to(literal);
            assert_that!(&*events.borrow()).contains("key");
        }
    }
}
