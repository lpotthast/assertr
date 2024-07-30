use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{AssertThat, failure::GenericFailure, Mode, tracking::AssertionTracking};

/// Assertions for generic `HashMap`s.∆
pub trait HashMapAssertions<K, V> {
    fn contains_key(self, expected: K) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug;

    fn contains_value(self, expected: V) -> Self
    where
        K: Debug,
        V: PartialEq + Debug;
}

impl<'t, K, V, M: Mode> HashMapAssertions<K, V> for AssertThat<'t, HashMap<K, V>, M> {
    #[track_caller]
    fn contains_key(self, expected: K) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug,
    {
        self.track_assertion();
        if !self.actual().contains_key(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\ndoes not contain expected key: {expected:#?}",
                    actual = self.actual(),
                ),
            });
        }
        self
    }

    #[track_caller]
    fn contains_value(self, expected: V) -> Self
    where
        K: Debug,
        V: PartialEq + Debug,
    {
        self.track_assertion();
        if !self.actual().values().any(|it| *it == expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\ndoes not contain expected value: {expected:#?}",
                    actual = self.actual(),
                ),
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
                    Actual: {{
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
                    Actual: {{
                        "foo": "bar",
                    }}

                    does not contain expected value: "baz"
                    -------- assertr --------
                "#});
        }
    }
}
