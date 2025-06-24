use crate::AssertThat;
use crate::assertions::HasLength;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;

pub trait LengthAssertions {
    fn is_empty(self) -> Self;

    fn is_not_empty(self) -> Self;

    fn have_length(self, expected: usize) -> Self;

    fn has_length(self, expected: usize) -> Self
    where
        Self: Sized,
    {
        self.have_length(expected)
    }
}

impl<T: HasLength + Debug, M: Mode> LengthAssertions for AssertThat<'_, T, M> {
    #[track_caller]
    fn is_empty(self) -> Self {
        self.track_assertion();
        if !self.actual().is_empty() {
            let actual = self.actual();
            let type_name = core::any::type_name::<T>();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual: {type_name} {actual:#?}

                    was expected to be empty, but it is not!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_not_empty(self) -> Self {
        self.track_assertion();
        if self.actual().is_empty() {
            let actual = self.actual();
            let type_name = core::any::type_name::<T>();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual: {type_name} {actual:#?}

                    was expected not to be empty, but it is!
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn have_length(self, expected: usize) -> Self {
        self.track_assertion();
        let actual_len = self.actual().length();
        if actual_len != expected {
            let type_name = core::any::type_name::<T>();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual: {type_name} {actual:#?}

                    does not have the correct length

                    Expected: {expected:?}
                      Actual: {actual_len:?}
                "#,actual = self.actual()}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod is_empty_on_array {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_empty() {
            let arr: [i32; 0] = [];
            assert_that(arr).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| assert_that([1, 2, 3]).with_location(false).is_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: [i32; 3] [
                    1,
                    2,
                    3,
                ]

                was expected to be empty, but it is not!
                -------- assertr --------
            "#});
        }
    }

    mod is_empty_on_slice {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn with_slice_succeeds_when_empty() {
            let slice: &[i32] = [].as_slice();
            assert_that(slice).is_empty();
        }

        #[test]
        fn with_slice_panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that([42].as_slice()).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: &[i32] [
                        42,
                    ]

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }

    mod is_empty_on_str_slice {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_empty() {
            assert_that("").is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that("foo").with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: &str "foo"

                was expected to be empty, but it is not!
                -------- assertr --------
            "#});
        }
    }

    mod is_empty_on_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_empty() {
            assert_that(String::from("")).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo"))
                    .with_location(false)
                    .is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: alloc::string::String "foo"

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }

    mod is_empty_on_vec {
        use crate::prelude::*;
        use alloc::format;
        use alloc::string::String;
        use alloc::vec;
        use alloc::vec::Vec;
        use indoc::formatdoc;

        #[test]
        fn with_slice_succeeds_when_empty() {
            let vec = Vec::<i32>::new();
            assert_that(vec).is_empty();
        }

        #[test]
        fn with_slice_panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that(vec![42]).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: alloc::vec::Vec<i32> [
                        42,
                    ]

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }

    mod is_empty_on_hashmap {
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
                    Actual: std::collections::hash::map::HashMap<&str, &str> {{
                        "foo": "bar",
                    }}

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }

    mod is_not_empty_on_hashmap {
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
                    Actual: std::collections::hash::map::HashMap<(), ()> {{}}

                    was expected not to be empty, but it is!
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_str_slice {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_length_matches() {
            assert_that("foo bar").has_length(7);
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that("foo bar").with_location(false).has_length(42);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: &str "foo bar"
    
                    does not have the correct length
    
                    Expected: 42
                      Actual: 7
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_length_matches() {
            assert_that(String::from("foo bar")).has_length(7);
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo bar"))
                    .with_location(false)
                    .has_length(42);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: alloc::string::String "foo bar"

                    does not have the correct length

                    Expected: 42
                      Actual: 7
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_slice {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            let slice: &[i32] = [].as_slice();
            assert_that(slice).has_length(0);
        }
        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            let slice: &[i32] = [1, 2, 3].as_slice();
            assert_that(slice).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that([42].as_slice())
                    .with_location(false)
                    .has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: &[i32] [
                        42,
                    ]

                    does not have the correct length

                    Expected: 2
                      Actual: 1
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_vec {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            let vec = Vec::<i32>::new();
            assert_that(vec).has_length(0);
        }
        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            let vec = vec![1, 2, 3];
            assert_that(vec).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that(vec![42]).with_location(false).has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: alloc::vec::Vec<i32> [
                        42,
                    ]

                    does not have the correct length

                    Expected: 2
                      Actual: 1
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_hashmap {
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
                Actual: std::collections::hash::map::HashMap<&str, &str> {{
                    "foo": "bar",
                }}

                does not have the correct length
                
                Expected: 2
                  Actual: 1
                -------- assertr --------
            "#});
        }
    }
}
