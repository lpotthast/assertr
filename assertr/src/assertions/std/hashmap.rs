use crate::{tracking::AssertionTracking, AssertThat, AssertrPartialEq, EqContext, Mode};
use core::borrow::Borrow;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;
use std::{collections::HashMap, hash::Hash};

/// Assertions for generic [HashMap]s.
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

    fn contains_entry<E>(self, key: impl Borrow<K>, value: impl Borrow<E>) -> Self
    where
        K: Eq + Hash + Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;
}

impl<'t, K, V, M: Mode> HashMapAssertions<K, V> for AssertThat<'t, HashMap<K, V>, M> {
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
                writedoc! {w, r#"
                    Actual: HashMap {actual:#?}

                    does not contain expected key: {expected:#?}
                "#, actual = self.actual()}
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
                writedoc! {w, r#"
                    Actual: HashMap {actual:#?}

                    contains unexpected key: {not_expected:#?}
                "#, actual = self.actual()}
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
                writedoc! {w, r#"
                    Actual: HashMap {actual:#?}
                    
                    does not contain expected value: {expected:#?}
                "#, actual = self.actual()}
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
                        writedoc! {w, r#"
                            Actual: HashMap {actual:#?}

                            does not contain expected value at key: {expected_key:#?}

                            Expected value: {expected_value:#?}
                              Actual value: {actual_value:#?}
                            "#,
                        }
                    });
                }
            }
        }

        then
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
            assert_that(map).contains_key("foo");
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map).with_location(false).contains_key("baz");
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
            assert_that(map).does_not_contain_key("baz");
        }

        #[test]
        fn panics_when_key_is_present() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map)
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

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_value_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that(map).contains_value("bar");
        }

        #[test]
        fn panics_when_value_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map).with_location(false).contains_value("baz");
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
            assert_that(map).contains_value("bar".to_string());
        }
    }

    mod contains_entry {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_value_is_present() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            // TODO: Can we get rid of the requirement to explicitly define E as `&str` here?
            assert_that(map).contains_entry::<&str>("foo", "bar");
        }

        #[test]
        fn succeeds_when_value_is_present_with_complex_type_with_borrowable_values() {
            #[derive(Debug, PartialEq)]
            struct Person {
                age: u32,
            }
            let mut map = HashMap::<&str, Person>::new();
            map.insert("foo", Person { age: 42 });
            assert_that_ref(&map).contains_entry("foo", &Person { age: 42 });
            assert_that_ref(&map).contains_entry("foo", Person { age: 42 });
            assert_that_ref(&map).contains_entry("foo", Box::new(Person { age: 42 }));
        }

        #[test]
        fn panics_when_key_is_absent() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map)
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
                assert_that(map)
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
}
