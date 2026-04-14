use crate::{AssertThat, AssertrPartialEq, EqContext, Mode, tracking::AssertionTracking};
use core::borrow::Borrow;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;
use std::{collections::HashMap, hash::BuildHasher, hash::Hash};

/// Assertions for generic [`HashMap`]s.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait HashMapAssertions<K, V> {
    fn contains_key(self, expected: impl Borrow<K>) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug;

    fn does_not_contain_key(self, not_expected: impl Borrow<K>) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug;

    fn contains_value<E>(self, expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn contains_entry<E>(self, key: impl Borrow<K>, value: impl Borrow<E>) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn does_not_contain_entry<E>(self, key: impl Borrow<K>, value: impl Borrow<E>) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;

    fn contains_keys<E, I>(self, expected: I) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
        E: Borrow<K> + Debug,
        I: IntoIterator<Item = E>;

    fn contains_exactly_entries<EK, EV, I>(self, expected: I) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<EV> + Debug,
        EK: Borrow<K> + Debug,
        EV: Debug,
        I: IntoIterator<Item = (EK, EV)>;
}

impl<K, V, S: BuildHasher, M: Mode> HashMapAssertions<K, V>
    for AssertThat<'_, HashMap<K, V, S>, M>
{
    #[track_caller]
    fn contains_key(self, expected: impl Borrow<K>) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
    {
        self.track_assertion();

        let expected = expected.borrow();

        if !self.actual().contains_key(expected) {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    does not contain expected key: {expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn does_not_contain_key(self, not_expected: impl Borrow<K>) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
    {
        self.track_assertion();

        let not_expected = not_expected.borrow();

        if self.actual().contains_key(not_expected) {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    contains unexpected key: {not_expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains_value<E>(self, expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        if !self
            .actual()
            .values()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}
                    
                    does not contain expected value: {expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn does_not_contain_value<E>(self, not_expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        if self
            .actual()
            .values()
            .any(|it| AssertrPartialEq::eq(it, &not_expected, None))
        {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    contains unexpected value: {not_expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains_entry<E>(self, key: impl Borrow<K>, value: impl Borrow<E>) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        let then = self.contains_key(key.borrow());

        then.track_assertion();

        let actual = then.actual();
        let expected_key = key.borrow();
        let expected_value = value.borrow();

        match actual.get(expected_key) {
            None => { /* Ignored: contains_key() already created an error in this case... */ }
            Some(actual_value) => {
                let mut ctx = EqContext::new();
                if !AssertrPartialEq::eq(actual_value, expected_value, Some(&mut ctx)) {
                    if !ctx.differences.differences.is_empty() {
                        then.add_detail_message(format!("Differences: {:#?}", ctx.differences));
                    }
                    then.fail(|w: &mut String| {
                        writedoc! {w, r"
                            Actual: HashMap {actual:#?}

                            does not contain expected value at key: {expected_key:#?}

                            Expected value: {expected_value:#?}
                              Actual value: {actual_value:#?}
                            ",
                        }
                    });
                }
            }
        }

        then
    }

    #[track_caller]
    fn does_not_contain_entry<E>(self, key: impl Borrow<K>, value: impl Borrow<E>) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        let unexpected_key = key.borrow();
        let unexpected_value = value.borrow();

        if self
            .actual()
            .get(unexpected_key)
            .is_some_and(|actual_value| AssertrPartialEq::eq(actual_value, unexpected_value, None))
        {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    contains unexpected entry at key: {unexpected_key:#?}

                    Unexpected value: {unexpected_value:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains_keys<E, I>(self, expected: I) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
        E: Borrow<K> + Debug,
        I: IntoIterator<Item = E>,
    {
        self.track_assertion();

        let expected = expected.into_iter().collect::<Vec<_>>();
        let keys_not_found = expected
            .iter()
            .filter(|expected_key| !self.actual().contains_key((*expected_key).borrow()))
            .collect::<Vec<_>>();

        if !keys_not_found.is_empty() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    does not contain all expected keys

                    Expected keys: {expected:#?}

                    Keys not found: {keys_not_found:#?}
                ", actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains_exactly_entries<EK, EV, I>(self, expected: I) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<EV> + Debug,
        EK: Borrow<K> + Debug,
        EV: Debug,
        I: IntoIterator<Item = (EK, EV)>,
    {
        self.track_assertion();

        let expected = expected.into_iter().collect::<Vec<_>>();
        let mut keys_not_found = Vec::new();
        let mut keys_with_unexpected_values = Vec::new();

        for (expected_key, expected_value) in &expected {
            match self.actual().get((*expected_key).borrow()) {
                None => keys_not_found.push(expected_key),
                Some(actual_value) => {
                    let mut ctx = EqContext::new();
                    if !AssertrPartialEq::eq(actual_value, expected_value, Some(&mut ctx)) {
                        keys_with_unexpected_values.push(expected_key);
                        if !ctx.differences.differences.is_empty() {
                            self.add_detail_message(format!(
                                "Differences at key {expected_key:#?}: {:#?}",
                                ctx.differences
                            ));
                        }
                    }
                }
            }
        }

        let unexpected_entries = self
            .actual()
            .iter()
            .filter(|(actual_key, _actual_value)| {
                !expected
                    .iter()
                    .any(|(expected_key, _expected_value)| (*expected_key).borrow() == *actual_key)
            })
            .collect::<Vec<_>>();

        if !keys_not_found.is_empty()
            || !unexpected_entries.is_empty()
            || !keys_with_unexpected_values.is_empty()
        {
            if !keys_not_found.is_empty() {
                self.add_detail_message(format!("Keys not found: {keys_not_found:#?}"));
            }
            if !unexpected_entries.is_empty() {
                self.add_detail_message(format!("Unexpected entries: {unexpected_entries:#?}"));
            }
            if !keys_with_unexpected_values.is_empty() {
                self.add_detail_message(format!(
                    "Keys with unexpected values: {keys_with_unexpected_values:#?}"
                ));
            }

            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: HashMap {actual:#?}

                    does not exactly contain expected entries

                    Expected entries: {expected:#?}
                ", actual = self.actual()}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains_key {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_key_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not contain expected key: "baz"
                    -------- assertr --------
                "#});
        }
    }

    mod does_not_contain_key {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_key_is_absent() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    contains unexpected key: "foo"
                    -------- assertr --------
                "#});
        }
    }

    mod contains_value {
        use std::collections::HashMap;

        use crate::prelude::*;
        use indoc::formatdoc;

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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not contain expected value: "baz"
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

    mod does_not_contain_value {
        use std::collections::HashMap;

        use crate::prelude::*;
        use indoc::formatdoc;

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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    contains unexpected value: "bar"
                    -------- assertr --------
                "#});
        }
    }

    mod contains_entry {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        fn succeeds_when_value_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            // TODO: Can we get rid of the requirement to explicitly define E as `&str` here?
            assert_that!(map).contains_entry::<&str>("foo", "bar");
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
                    .contains_entry::<&str>("baz", "someValue");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not contain expected key: "baz"
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
                    .contains_entry::<&str>("foo", "someValue");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not contain expected value at key: "foo"

                    Expected value: "someValue"
                      Actual value: "bar"
                    -------- assertr --------
                "#});
        }
    }

    mod does_not_contain_entry {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        fn succeeds_when_key_is_absent() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_entry::<&str>("baz", "bar");
        }

        #[test]
        fn succeeds_when_value_differs() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).does_not_contain_entry::<&str>("foo", "baz");
        }

        #[test]
        fn panics_when_entry_is_present() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map)
                    .with_location(false)
                    .does_not_contain_entry::<&str>("foo", "bar");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    contains unexpected entry at key: "foo"

                    Unexpected value: "bar"
                    -------- assertr --------
                "#});
        }
    }

    mod contains_keys {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        fn succeeds_when_all_keys_are_present() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            assert_that!(map).contains_keys(["foo", "baz"]);
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not contain all expected keys

                    Expected keys: [
                        "foo",
                        "baz",
                    ]

                    Keys not found: [
                        "baz",
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod contains_exactly_entries {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::collections::HashMap;

        #[test]
        fn succeeds_when_entries_match() {
            let map = HashMap::from([("foo", "bar"), ("baz", "qux")]);
            assert_that!(&map).contains_exactly_entries([("foo", "bar"), ("baz", "qux")]);
            assert_that!(map)
                .contains_exactly_entries(HashMap::from([("foo", "bar"), ("baz", "qux")]));
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not exactly contain expected entries

                    Expected entries: [
                        (
                            "foo",
                            "bar",
                        ),
                        (
                            "baz",
                            "qux",
                        ),
                    ]

                    Details: [
                        Keys not found: [
                            "baz",
                        ],
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not exactly contain expected entries

                    Expected entries: []

                    Details: [
                        Unexpected entries: [
                            (
                                "foo",
                                "bar",
                            ),
                        ],
                    ]
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
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    does not exactly contain expected entries

                    Expected entries: [
                        (
                            "foo",
                            "baz",
                        ),
                    ]

                    Details: [
                        Keys with unexpected values: [
                            "foo",
                        ],
                    ]
                    -------- assertr --------
                "#});
        }
    }
}
