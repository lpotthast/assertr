//! Assertions, extractions, and map support for JSON values. Requires `serde-json`.

use alloc::{string::String, vec::Vec};
use core::borrow::Borrow;

use serde_json::Value;

use crate::{
    AssertThat, Mode,
    assertions::{HasLength, Map, MapLookup, StableEntryOrder},
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, RenderingOrder, ValueRenderer},
};

/// Requires a value at a JSON pointer, such as `/users/0/name`, returning the borrowed value.
///
/// Pointers follow RFC 6901: an empty pointer selects the whole value, and every other pointer
/// starts with `/`, separates object keys and array indexes with `/`, and escapes `~` as `~0`
/// and `/` as `~1`. The rejection shows the whole value and names the pointer.
///
/// ```
/// use assertr::{matchers::json::HasPointer, prelude::*};
/// use serde_json::json;
///
/// assert_that!(json!({ "users": [{ "name": "Ada" }] })).matches(HasPointer::new("/users/0/name"));
/// ```
#[derive(Debug, Clone)]
pub struct HasPointer<P>(P);

impl<P> HasPointer<P> {
    /// Expects a value at `pointer`.
    #[must_use]
    pub const fn new(pointer: P) -> Self {
        Self(pointer)
    }
}

impl<P, R> Expectation<Value, R> for HasPointer<P>
where
    P: AsRef<str>,
    R: ValueRenderer<Value> + ValueRenderer<str>,
{
    type Success<'a>
        = &'a Value
    where
        Self: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Value,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.pointer(self.0.as_ref()).ok_or(())
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Value, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let pointer = self.0.as_ref();
        let failure = failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "has a value at the JSON pointer",
                "has no value at the JSON pointer",
            )
            .expected(render.value(pointer));
        if pointer.is_empty() || pointer.starts_with('/') {
            failure
        } else {
            failure.fact(Fact::note(
                "a JSON pointer is empty or starts with `/`, as in `/users/0/name`",
            ))
        }
    }
}

/// Declares an expectation requiring one kind of JSON value and returning its content.
macro_rules! json_kind {
    (
        $(#[$attr:meta])*
        $name:ident, $success:ty, |$actual:ident| $extract:expr, $met:literal, $unmet:literal
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<R: ValueRenderer<Value>> Expectation<Value, R> for $name {
            type Success<'a>
                = $success
            where
                Self: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a;

            fn evaluate<'a>(
                &'a self,
                $actual: &'a Value,
                _: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                $extract.ok_or(())
            }

            const KIND: FailureKind = FailureKind::Variant;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a Value, Self::Rejection<'a>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                failure.relations(
                    rejected.map(|(actual, ())| context.render().value(actual)),
                    $met,
                    $unmet,
                )
            }
        }
    };
}

json_kind!(
    /// Requires JSON `null`.
    IsNull,
    (),
    |actual| actual.as_null(),
    "is JSON null",
    "is not JSON null"
);
json_kind!(
    /// Requires a JSON string, returning its contents.
    IsString,
    &'a str,
    |actual| actual.as_str(),
    "is a JSON string",
    "is not a JSON string"
);
json_kind!(
    /// Requires a JSON number representable as `i64`, returning it.
    IsI64,
    i64,
    |actual| actual.as_i64(),
    "is a JSON number fitting `i64`",
    "is not a JSON number fitting `i64`"
);
json_kind!(
    /// Requires a JSON number representable as `u64`, returning it.
    IsU64,
    u64,
    |actual| actual.as_u64(),
    "is a JSON number fitting `u64`",
    "is not a JSON number fitting `u64`"
);
json_kind!(
    /// Requires a JSON number, returning it as `f64`. Integers convert, possibly losing precision.
    IsF64,
    f64,
    |actual| actual.as_f64(),
    "is a JSON number",
    "is not a JSON number"
);
json_kind!(
    /// Requires a JSON boolean, returning it.
    IsBool,
    bool,
    |actual| actual.as_bool(),
    "is a JSON boolean",
    "is not a JSON boolean"
);
json_kind!(
    /// Requires a JSON array, returning its elements.
    IsArray,
    &'a Vec<Value>,
    |actual| actual.as_array(),
    "is a JSON array",
    "is not a JSON array"
);
json_kind!(
    /// Requires a JSON object, returning its entries.
    IsObject,
    &'a serde_json::Map<String, Value>,
    |actual| actual.as_object(),
    "is a JSON object",
    "is not a JSON object"
);

/// Checks of JSON values that work in every mode.
///
/// ```
/// use assertr::prelude::*;
/// use serde_json::json;
///
/// assert_that!(json!({ "name": "Ada", "manager": null }))
///     .has_pointer("/name")
///     .matches(assertr::matchers::json::HasPointer::new("/manager"));
/// assert_that!(json!(null)).is_null();
/// ```
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait JsonValueAssertions<R = DebugRenderer> {
    /// Asserts that `pointer` selects a value. See [`HasPointer`] for the pointer syntax.
    fn has_pointer(self, pointer: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Value> + ValueRenderer<str>;

    /// Asserts that the value is JSON `null`.
    fn is_null(self) -> Self
    where
        R: ValueRenderer<Value>;
}

impl<M: Mode, R> JsonValueAssertions<R> for AssertThat<'_, Value, M, R> {
    #[track_caller]
    fn has_pointer(self, pointer: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<Value> + ValueRenderer<str>,
    {
        self.matches(HasPointer::new(pointer))
    }

    #[track_caller]
    fn is_null(self) -> Self
    where
        R: ValueRenderer<Value>,
    {
        self.matches(IsNull)
    }
}

/// Panic-mode extractions from JSON values.
///
/// Each method checks that the value has the requested shape, then continues with its content,
/// so the general assertion families apply: string assertions to a string, collection
/// assertions to an array, and map assertions to an object. A JSON value itself compares with
/// strings, numbers, and booleans, so `pointer("/name").is_equal_to("Ada")` needs no extraction. A
/// failed extraction cannot produce a value, so the family is unavailable in capture mode. Use
/// [`JsonValueAssertions`] or the [`json` matchers](crate::matchers::json) there.
///
/// ```
/// use assertr::prelude::*;
/// use serde_json::json;
///
/// let response = json!({
///     "sections": [{ "category": "qemu_toolchain", "checks": 3 }],
///     "healthy": true,
/// });
/// let it = assert_that!(response);
/// it.pointer("/sections/0/category").is_equal_to("qemu_toolchain");
/// it.pointer("/sections/0/category").str().starts_with("qemu");
/// it.pointer("/sections/0/checks").u64().is_equal_to(3);
/// it.pointer("/sections").array().has_length(1);
/// it.pointer("/healthy").bool().is_true();
/// it.object().contains_key("sections");
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait JsonValueExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that `pointer` selects a value, then continues with it. See [`HasPointer`] for the
    /// pointer syntax.
    fn pointer(&'t self, pointer: impl AsRef<str>) -> AssertThat<'t, Value, Panic, R>
    where
        R: ValueRenderer<Value> + ValueRenderer<str> + Clone;

    /// Asserts that the value is a string, then continues with its contents.
    fn str(&'t self) -> AssertThat<'t, &'t str, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is a number fitting `i64`, then continues with it.
    fn i64(&'t self) -> AssertThat<'t, i64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is a number fitting `u64`, then continues with it.
    fn u64(&'t self) -> AssertThat<'t, u64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is a number, then continues with it as `f64`.
    fn f64(&'t self) -> AssertThat<'t, f64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is a boolean, then continues with it.
    fn bool(&'t self) -> AssertThat<'t, bool, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is an array, then continues with its elements.
    fn array(&'t self) -> AssertThat<'t, Vec<Value>, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;

    /// Asserts that the value is an object, then continues with its entries, which support the
    /// map assertions.
    fn object(&'t self) -> AssertThat<'t, serde_json::Map<String, Value>, Panic, R>
    where
        R: ValueRenderer<Value> + Clone;
}

impl<'t, R> JsonValueExtractAssertions<'t, R> for AssertThat<'t, Value, Panic, R> {
    #[track_caller]
    fn pointer(&'t self, pointer: impl AsRef<str>) -> AssertThat<'t, Value, Panic, R>
    where
        R: ValueRenderer<Value> + ValueRenderer<str> + Clone,
    {
        let pointer = pointer.as_ref();
        self.test_assertion(&HasPointer::new(pointer));
        self.derive_as("pointer(..)", |value| {
            value
                .pointer(pointer)
                .unwrap_or_else(|| unreachable!("validated pointer became unavailable"))
        })
    }

    #[track_caller]
    fn str(&'t self) -> AssertThat<'t, &'t str, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsString);
        self.derive_owned_as("str()", |_| value)
    }

    #[track_caller]
    fn i64(&'t self) -> AssertThat<'t, i64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsI64);
        self.derive_owned_as("i64()", |_| value)
    }

    #[track_caller]
    fn u64(&'t self) -> AssertThat<'t, u64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsU64);
        self.derive_owned_as("u64()", |_| value)
    }

    #[track_caller]
    fn f64(&'t self) -> AssertThat<'t, f64, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsF64);
        self.derive_owned_as("f64()", |_| value)
    }

    #[track_caller]
    fn bool(&'t self) -> AssertThat<'t, bool, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsBool);
        self.derive_owned_as("bool()", |_| value)
    }

    #[track_caller]
    fn array(&'t self) -> AssertThat<'t, Vec<Value>, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsArray);
        self.derive_as("array()", |_| value)
    }

    #[track_caller]
    fn object(&'t self) -> AssertThat<'t, serde_json::Map<String, Value>, Panic, R>
    where
        R: ValueRenderer<Value> + Clone,
    {
        let value = self.require(&IsObject);
        self.derive_as("object()", |_| value)
    }
}

impl HasLength for serde_json::Map<String, Value> {
    fn length(&self) -> usize {
        self.len()
    }
}

/// A JSON object is a map from its keys to its values.
impl Map for serde_json::Map<String, Value> {
    type Key = String;
    type Value = Value;
    const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

    fn entries(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.iter()
    }
}

/// A JSON object always yields its entries in a guaranteed order: sorted by key, or in insertion
/// order with `serde_json`'s `preserve_order` feature.
impl StableEntryOrder for serde_json::Map<String, Value> {}

impl<Q> MapLookup<Q> for serde_json::Map<String, Value>
where
    String: Borrow<Q>,
    Q: Ord + Eq + core::hash::Hash + ?Sized,
{
    fn get_key_value(&self, key: &Q) -> Option<(&String, &Value)> {
        serde_json::Map::get_key_value(self, key)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    fn document() -> Value {
        json!({
            "name": "Ada",
            "age": 36,
            "balance": -2,
            "ratio": 0.5,
            "admin": true,
            "manager": null,
            "roles": ["reader", "editor"],
            "address": { "city": "London" },
        })
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::document;
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            document()
                .must()
                .have_pointer("/name")
                .pointer("/manager")
                .be_null();
        }
    }

    mod renderer_contract {
        use serde_json::Value;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Value, Capture, NoRenderer> => JsonValueAssertions<NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Value, Panic, NoRenderer>
                    => JsonValueExtractAssertions<'static, NoRenderer>
            );
        }
    }

    mod has_pointer {
        use indoc::formatdoc;
        use serde_json::json;

        use super::document;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(document()), has_pointer("/missing"));
        }

        #[test]
        fn succeeds_for_present_values_including_null() {
            assert_that!(document())
                .has_pointer("")
                .has_pointer("/roles/1")
                .has_pointer("/address/city")
                .has_pointer("/manager");
        }

        #[test]
        fn panics_with_the_value_and_the_pointer() {
            assert_that!(|| {
                assert_that!(json!({ "a": [1] }))
                    .with_location(false)
                    .has_pointer("/a/1");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `json!({{ "a": [1] }})`

                Actual: Object {{
                    "a": Array [
                        Number(1),
                    ],
                }}

                has no value at the JSON pointer

                Expected: "/a/1"
                -------- assertr --------
            "#});
        }

        #[test]
        fn explains_a_pointer_without_leading_slash() {
            let failures = assert_that!(document()).capture(|it| it.has_pointer("name"));
            assert_that!(failures[0].to_string())
                .contains("a JSON pointer is empty or starts with `/`");
        }
    }

    mod is_null {
        use super::document;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(document()), is_null());
        }

        #[test]
        fn succeeds_for_null_only() {
            assert_that!(serde_json::Value::Null).is_null();
            let failures = assert_that!(document()).capture(JsonValueAssertions::is_null);
            assert_that!(failures[0].to_string()).contains("is not JSON null");
        }
    }

    mod pointer {
        use indoc::formatdoc;

        use super::document;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(document()).pointer("/name"),
                is_equal_to(serde_json::json!("Grace"))
            );
        }

        #[test]
        fn continues_with_the_selected_value() {
            assert_that!(document())
                .pointer("/address/city")
                .is_equal_to(serde_json::json!("London"));
        }

        #[test]
        fn failures_of_the_selected_value_name_the_projection() {
            assert_that!(|| {
                let value = document();
                assert_that!(value)
                    .with_location(false)
                    .pointer("/name")
                    .str()
                    .is_equal_to("Grace");
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `value`
                Projection: `.pointer(..).str()`

                Expected: "Grace"

                  Actual: "Ada"
                -------- assertr --------
            "#});
        }
    }

    mod kinds {
        use indoc::formatdoc;

        use super::document;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(document()).pointer("/age"), str());
            assert_caller_location!(assert_that!(document()).pointer("/name"), i64());
            assert_caller_location!(assert_that!(document()).pointer("/balance"), u64());
            assert_caller_location!(assert_that!(document()).pointer("/name"), f64());
            assert_caller_location!(assert_that!(document()).pointer("/name"), bool());
            assert_caller_location!(assert_that!(document()).pointer("/name"), array());
            assert_caller_location!(assert_that!(document()).pointer("/name"), object());
        }

        #[test]
        fn continue_with_the_content_of_each_kind() {
            let value = document();
            let it = assert_that!(value);
            it.pointer("/name").str().starts_with("A");
            it.pointer("/age").i64().is_equal_to(36);
            it.pointer("/balance").i64().is_less_than(0);
            it.pointer("/age").u64().is_equal_to(36);
            it.pointer("/ratio").f64().is_equal_to(0.5);
            it.pointer("/admin").bool().is_true();
            it.pointer("/roles")
                .array()
                .contains_exactly(["reader", "editor"]);
            it.pointer("/name").is_equal_to("Ada");
            it.pointer("/age").is_equal_to(36);
            it.pointer("/address")
                .object()
                .contains_key("city")
                .keys()
                .contains_exactly(["city"]);
        }

        #[test]
        fn panics_when_the_value_has_another_kind() {
            assert_that!(|| {
                let value = document();
                assert_that!(value)
                    .with_location(false)
                    .pointer("/age")
                    .str();
            })
            .panics()
            .message()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `value`
                Projection: `.pointer(..)`

                Actual: Number(36)

                is not a JSON string
                -------- assertr --------
            "});
        }
    }

    mod json_objects {
        use serde_json::json;

        use crate::prelude::*;

        #[test]
        fn support_map_assertions_with_borrowed_keys() {
            let value = json!({ "retries": 3, "timeout": 30 });
            let object = value.as_object().unwrap();
            assert_that!(object)
                .has_length(2)
                .contains_key("retries")
                .contains_entry("timeout", json!(30))
                .contains_entry("timeout", 30)
                .contains_exactly_entries([("retries", 3), ("timeout", 30)])
                .does_not_contain_key("verbose");
        }
    }
}
