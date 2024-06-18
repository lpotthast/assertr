use crate::{failure::GenericFailure, AssertThat};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// Assertions for generic maps.
impl<'t, K, V> AssertThat<'t, HashMap<K, V>> {
    #[track_caller]
    pub fn contains_key(self, expected: K) -> Self
    where
        K: Eq + Hash + Debug,
        V: PartialEq + Debug,
    {
        if !self.actual.borrowed().contains_key(&expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\ndoes not contain expected key: {expected:#?}",
                    actual = self.actual.borrowed(),
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn contains_value(self, expected: V) -> Self
    where
        K: Debug,
        V: PartialEq + Debug,
    {
        if !self
            .actual
            .borrowed()
            .values()
            .any(|it| *it == expected)
        {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\ndoes not contain expected value: {expected:#?}",
                    actual = self.actual.borrowed(),
                ),
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn contains_key_succeeds_when_key_is_present() {
        let mut map = HashMap::new();
        map.insert("foo", "bar");
        assert_that(map).contains_key("foo");
    }

    #[test]
    fn contains_key_panics_when_key_is_absent() {
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

    #[test]
    fn contains_value_succeeds_when_value_is_present() {
        let mut map = HashMap::new();
        map.insert("foo", "bar");
        assert_that(map).contains_value("bar");
    }

    #[test]
    fn contains_value_panics_when_value_is_absent() {
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
