use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{AssertrPartialEq, AssertThat, EqContext, failure::GenericFailure, Mode, tracking::AssertionTracking};

/// Assertions for generic `HashMap`s.∆
pub trait HashMapAssertions<K, V> {
    fn contains_key(self, expected: K) -> Self
    where
        K: Eq + Hash + Debug,
        V: Debug;

    fn contains_value<E>(self, expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug;
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
    fn contains_value<E>(self, expected: E) -> Self
    where
        K: Debug,
        V: AssertrPartialEq<E> + Debug,
        E: Debug,
    {
        self.track_assertion();

        let mut ctx = EqContext::new();

        if !self.actual().values().any(|it| AssertrPartialEq::eq(it, &expected, &mut ctx)) {
            // TODO: Use context. NO: We are actually not interested in differences at this point!
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

        #[test]
        fn compiles_with_any_type_comparable_to_the_actual_value_type() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that(map).contains_value("bar".to_string());
        }
    }
}
