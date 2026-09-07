use crate::AssertThat;
use crate::ValueRenderer;
use crate::assertions::HasLength;
use crate::failure::{Fact, FailureKind};
use crate::mode::Mode;

/// Assertions for subjects implementing [`HasLength`].
///
/// [`HasLength`]: crate::assertions::HasLength
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait LengthAssertions {
    /// The assertion subject whose length is checked and whose failures are rendered.
    type Subject: HasLength;

    /// The renderer carried by the assertion chain.
    type Renderer;

    /// Asserts that the subject has length zero.
    fn is_empty(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject has nonzero length.
    fn is_not_empty(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject has exactly `expected` elements or bytes.
    fn has_length(self, expected: usize) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject> + ValueRenderer<usize>;
}

impl<T: HasLength, M: Mode, R> LengthAssertions for AssertThat<'_, T, M, R> {
    type Renderer = R;
    type Subject = T;

    #[track_caller]
    fn is_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if !self.actual().is_empty() {
            self.failure(FailureKind::Length)
                .actual(self.render().value(self.actual()).show_type_hint(true))
                .relation("is not empty")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_not_empty(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        if self.actual().is_empty() {
            self.failure(FailureKind::Length)
                .actual(self.render().value(self.actual()).show_type_hint(true))
                .relation("is unexpectedly empty")
                .raise();
        }
        self
    }

    #[track_caller]
    fn has_length(self, expected: usize) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<usize>,
    {
        self.track_assertion();
        let actual_len = self.actual().length();
        if actual_len != expected {
            self.failure(FailureKind::Length)
                .actual(self.render().value(self.actual()).show_type_hint(true))
                .relation("does not have the expected length")
                .expected(self.render().value(&expected))
                .fact(Fact::labelled(
                    "Actual length",
                    self.render().value(&actual_len),
                ))
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Vec<u8>, Panic, NoRenderer> => LengthAssertions
            );
        }
    }

    mod is_empty_on_array {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let arr: [i32; 0] = [];
            arr.must().be_empty();
        }

        #[test]
        fn succeeds_when_empty() {
            let arr: [i32; 0] = [];
            assert_that!(arr).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| assert_that!([1, 2, 3]).with_location(false).is_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2, 3]`

                Actual: [i32; 3] [
                    1,
                    2,
                    3,
                ]

                is not empty
                -------- assertr --------
            "});
        }

        #[test]
        fn requires_no_numeric_renderer() {
            struct SubjectRenderer;
            impl ValueRenderer<[i32; 0]> for SubjectRenderer {
                fn fmt(&self, _: &[i32; 0], _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("success rendered")
                }
            }
            assert_that!([] as [i32; 0])
                .with_renderer(SubjectRenderer)
                .with_location(false)
                .is_empty();
        }
    }

    mod is_empty_on_slice {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let slice: &[i32] = [].as_slice();
            slice.must().be_empty();
        }

        #[test]
        fn with_slice_succeeds_when_empty() {
            let slice: &[i32] = [].as_slice();
            assert_that!(slice).is_empty();
        }

        #[test]
        fn with_slice_panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!([42].as_slice())
                    .with_location(false)
                    .is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[42].as_slice()`

                    Actual: [i32] [
                        42,
                    ]

                    is not empty
                    -------- assertr --------
                "});
        }
    }

    mod is_empty_on_str_slice {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "".must().be_empty();
        }

        #[test]
        fn succeeds_when_empty() {
            assert_that!("").is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!("foo").with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo"`

                Actual: str "foo"

                is not empty
                -------- assertr --------
            "#});
        }
    }

    mod is_empty_on_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            String::new().must().be_empty();
        }

        #[test]
        fn succeeds_when_empty() {
            assert_that!(String::new()).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!(String::from("foo"))
                    .with_location(false)
                    .is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `String::from("foo")`

                    Actual: String "foo"

                    is not empty
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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Vec::<i32>::new().must().be_empty();
        }

        #[test]
        fn with_slice_succeeds_when_empty() {
            let vec = Vec::<i32>::new();
            assert_that!(vec).is_empty();
        }

        #[test]
        fn with_slice_panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!(vec![42]).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `vec![42]`

                    Actual: Vec [
                        42,
                    ]

                    is not empty
                    -------- assertr --------
                "});
        }
    }

    #[cfg(feature = "std")]
    #[allow(clippy::zero_sized_map_values)]
    mod is_empty_on_hashmap {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            HashMap::<(), ()>::new().must().be_empty();
        }

        #[test]
        fn succeeds_when_map_is_empty() {
            let map = HashMap::<(), ()>::new();
            assert_that!(map).is_empty();
        }

        #[test]
        fn panics_when_map_is_not_empty() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{
                        "foo": "bar",
                    }}

                    is not empty
                    -------- assertr --------
                "#});
        }
    }

    mod is_empty_on_vec_deque {
        use alloc::collections::VecDeque;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            VecDeque::<i32>::new().must().be_empty();
        }

        #[test]
        fn succeeds_when_empty() {
            assert_that!(VecDeque::<i32>::new()).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that!(VecDeque::from([42]))
                    .with_location(false)
                    .is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `VecDeque::from([42])`

                    Actual: VecDeque [
                        42,
                    ]

                    is not empty
                    -------- assertr --------
                "});
        }
    }

    #[cfg(feature = "std")]
    #[allow(clippy::zero_sized_map_values)]
    mod is_not_empty_on_hashmap {
        use std::collections::HashMap;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            HashMap::from([("foo", "bar")]).must().not_be_empty();
        }

        #[test]
        fn succeeds_when_map_is_empty() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            assert_that!(map).is_not_empty();
        }

        #[test]
        fn panics_when_map_is_empty() {
            assert_that_panic_by(|| {
                let map = HashMap::<(), ()>::new();
                assert_that!(map).with_location(false).is_not_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `map`

                    Actual: HashMap {{}}

                    is unexpectedly empty
                    -------- assertr --------
                "});
        }
    }

    mod is_not_empty_on_vec_deque {
        use alloc::collections::VecDeque;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            VecDeque::from([42]).must().not_be_empty();
        }

        #[test]
        fn succeeds_when_not_empty() {
            assert_that!(VecDeque::from([42])).is_not_empty();
        }

        #[test]
        fn succeeds_for_borrowed_vec_deque() {
            let deque = VecDeque::from([42]);

            assert_that!(&deque).is_not_empty();
        }

        #[test]
        fn panics_when_empty() {
            assert_that_panic_by(|| {
                assert_that!(VecDeque::<i32>::new())
                    .with_location(false)
                    .is_not_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `VecDeque::<i32>::new()`

                    Actual: VecDeque []

                    is unexpectedly empty
                    -------- assertr --------
                "});
        }
    }

    mod has_length_on_str_slice {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo bar".must().have_length(7);
        }

        #[test]
        fn succeeds_when_expected_length_matches() {
            assert_that!("foo bar").has_length(7);
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!("foo bar").with_location(false).has_length(42);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `"foo bar"`

                    Actual: str "foo bar"

                    does not have the expected length

                    Expected: 42

                    Details:
                      - Actual length: 7
                    -------- assertr --------
                "#});
        }
    }

    mod has_length_on_string {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            String::from("foo bar").must().have_length(7);
        }

        #[test]
        fn succeeds_when_expected_length_matches() {
            assert_that!(String::from("foo bar")).has_length(7);
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(String::from("foo bar"))
                    .with_location(false)
                    .has_length(42);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `String::from("foo bar")`

                    Actual: String "foo bar"

                    does not have the expected length

                    Expected: 42

                    Details:
                      - Actual length: 7
                    -------- assertr --------
                "#});
        }
    }

    mod length_assertions_on_other_string_like_types {
        use crate::prelude::*;
        use alloc::{borrow::Cow, boxed::Box, string::String};

        #[test]
        fn boxed_str_supports_length_assertions() {
            assert_that!(Box::<str>::from("foo"))
                .is_not_empty()
                .has_length(3);
            assert_that!(Box::<str>::default()).is_empty().has_length(0);
        }

        #[test]
        fn cow_str_supports_length_assertions() {
            assert_that!(Cow::Borrowed("foo"))
                .is_not_empty()
                .has_length(3);
            assert_that!(Cow::<str>::Owned(String::new()))
                .is_empty()
                .has_length(0);
        }
    }

    mod has_length_on_slice {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let slice: &[i32] = [1, 2, 3].as_slice();
            slice.must().have_length(3);
        }

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            let slice: &[i32] = [].as_slice();
            assert_that!(slice).has_length(0);
        }
        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            let slice: &[i32] = [1, 2, 3].as_slice();
            assert_that!(slice).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!([42].as_slice())
                    .with_location(false)
                    .has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `[42].as_slice()`

                    Actual: [i32] [
                        42,
                    ]

                    does not have the expected length

                    Expected: 2

                    Details:
                      - Actual length: 1
                    -------- assertr --------
                "});
        }

        #[test]
        fn renders_original_length_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, assert_custom_value};
            let failures = assert_that!([1, 2])
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.has_length(3));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `[1, 2]`

                Actual: [i32; 2] custom([1, 2])

                does not have the expected length

                Expected: custom(3)

                Details:
                  - Actual length: custom(2)
                -------- assertr --------
            "});

            assert_custom_value(failures[0].expected.as_ref().unwrap(), &3_usize);
            assert_custom_value(&failures[0].facts[0].value, &2_usize);
        }

        #[test]
        fn a_matching_length_does_not_render() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("success rendered")
                }
            }
            assert_that!([1, 2])
                .with_renderer(NeverRender)
                .with_location(false)
                .has_length(2);
        }
    }

    mod has_length_on_vec {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            vec![1, 2, 3].must().have_length(3);
        }

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            assert_that!(Vec::<i32>::new()).has_length(0);
        }
        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            assert_that!(vec![1, 2, 3]).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(vec![42]).with_location(false).has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `vec![42]`

                    Actual: Vec [
                        42,
                    ]

                    does not have the expected length

                    Expected: 2

                    Details:
                      - Actual length: 1
                    -------- assertr --------
                "});
        }
    }

    mod has_length_on_vec_deque {
        use alloc::collections::VecDeque;

        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            VecDeque::from([1, 2, 3]).must().have_length(3);
        }

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            assert_that!(VecDeque::<i32>::new()).has_length(0);
        }

        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            assert_that!(VecDeque::from([1, 2, 3])).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that!(VecDeque::from([42]))
                    .with_location(false)
                    .has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `VecDeque::from([42])`

                    Actual: VecDeque [
                        42,
                    ]

                    does not have the expected length

                    Expected: 2

                    Details:
                      - Actual length: 1
                    -------- assertr --------
                "});
        }
    }

    #[cfg(feature = "std")]
    #[allow(clippy::zero_sized_map_values)]
    mod has_length_on_hashmap {
        use indoc::formatdoc;
        use std::collections::HashMap;

        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            HashMap::from([("foo", "bar")]).must().have_length(1);
        }

        #[test]
        fn succeeds_when_length_matches_and_empty() {
            assert_that!(HashMap::<(), ()>::new()).has_length(0);
        }

        #[test]
        fn succeeds_when_length_matches_and_non_empty() {
            let mut map = HashMap::new();
            map.insert("foo", "bar");
            map.insert("bar", "baz");
            map.insert("baz", "foo");
            assert_that!(map).has_length(3);
        }

        #[test]
        fn panics_when_length_does_not_match() {
            assert_that_panic_by(|| {
                let mut map = HashMap::new();
                map.insert("foo", "bar");
                assert_that!(map).with_location(false).has_length(2);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `map`

                Actual: HashMap {{
                    "foo": "bar",
                }}

                does not have the expected length
                
                Expected: 2
                
                Details:
                  - Actual length: 1
                -------- assertr --------
            "#});
        }
    }
}
