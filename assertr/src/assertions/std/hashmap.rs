use crate::failure::ExpectedActualFailure;
use crate::{
    failure::GenericFailure, tracking::AssertionTracking, AssertThat, AssertrPartialEq, EqContext,
    Mode,
};
use core::borrow::Borrow;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// Assertions for generic `HashMap`s.
pub trait HashMapAssertions<K, V> {
    fn has_length(self, expected: usize) -> Self
    where
        K: Debug,
        V: Debug;

    fn is_empty(self) -> Self
    where
        K: Debug,
        V: Debug;

    fn is_not_empty(self) -> Self
    where
        K: Debug,
        V: Debug;

    fn contains_key(self, expected: impl Borrow<K>) -> Self
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
    fn has_length(mut self, expected: usize) -> Self
    where
        K: Debug,
        V: Debug,
    {
        self.track_assertion();
        let actual = self.actual().len();
        if actual != expected {
            self = self.with_detail_message("HashMap was not of expected length!");
            self.fail(ExpectedActualFailure {
                expected: &expected,
                actual: &actual,
            });
        }
        self
    }

    #[track_caller]
    fn is_empty(self) -> Self
    where
        K: Debug,
        V: Debug,
    {
        self.track_assertion();
        if !self.actual().is_empty() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: HashMap {actual:#?}\n\nwas expected to be empty, but it is not!",
                    actual = self.actual(),
                ),
            });
        }
        self
    }

    #[track_caller]
    fn is_not_empty(self) -> Self
    where
        K: Debug,
        V: Debug,
    {
        self.track_assertion();
        if self.actual().is_empty() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: HashMap {actual:#?}\n\nwas expected NOT to be empty, but it IS empty!",
                    actual = self.actual(),
                ),
            });
        }
        self
    }

    #[track_caller]
    fn contains_key(self, expected: impl Borrow<K>) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
    {
        self.track_assertion();

        let expected = expected.borrow();

        if !self.actual().contains_key(expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: HashMap {actual:#?}\n\ndoes not contain expected key: {expected:#?}",
                    actual = self.actual(),
                ),
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
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: HashMap {actual:#?}\n\ndoes not contain expected value: {expected:#?}",
                    actual = self.actual(),
                ),
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
                    then.fail(GenericFailure {
                        arguments: format_args!(
                            "Actual: HashMap {actual:#?}\n\ndoes not contain expected value at key: {expected_key:#?}\n\nExpected value: {expected_value:#?}\n  Actual value: {actual_value:#?}",
                        ),
                    });
                }
            }
        }

        then
    }
}

#[cfg(test)]
mod tests {
    mod has_length {
        use indoc::formatdoc;
        use std::collections::HashMap;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            let map = HashMap::<(), ()>::new();
            assert_that(map).has_length(0);
        }

        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            map.insert("bar", "baz");
            map.insert("baz", "foo");
            assert_that(map).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map).with_location(false).has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 2

                      Actual: 1

                    Details: [
                        HashMap was not of expected length!,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_empty {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_map_is_empty() {
            let map = HashMap::<(), ()>::new();
            assert_that(map).is_empty();
        }

        #[test]
        fn panics_when_map_is_not_empty() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that(map).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }

    mod is_not_empty {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_map_is_empty() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that(map).is_not_empty();
        }

        #[test]
        fn panics_when_map_is_empty() {
            assert_that_panic_by(|| {
                let map = HashMap::<(), ()>::new();
                assert_that(map).with_location(false).is_not_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: HashMap {{}}

                    was expected NOT to be empty, but it IS empty!
                    -------- assertr --------
                "#});
        }
    }

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
            assert_that(&map).contains_entry("foo", &Person { age: 42 });
            assert_that(&map).contains_entry("foo", Person { age: 42 });
            assert_that(&map).contains_entry("foo", Box::new(Person { age: 42 }));
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
