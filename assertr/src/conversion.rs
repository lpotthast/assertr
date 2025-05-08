#[cfg(feature = "serde")]
use crate::AssertThat;
#[cfg(feature = "serde")]
use crate::actual::Actual;
#[cfg(feature = "serde")]
use crate::mode::Mode;

/// A conversion function that can be used with `map` to easily convert any `serde::Serialize`able
/// type into its JSON representation for further checks.
/// Uses `serde_json` to perform the conversion.
/// Panics when the conversion fails.
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// struct Person {
///     age: u32,
/// }
///
/// let person = Person { age: 42 };
///
/// assert_that(person)
///     .map(json())
///     .is_equal_to(r#"{"age":42}"#);
/// ```
#[cfg(feature = "serde")]
pub fn json<S: serde::Serialize>() -> impl FnOnce(Actual<S>) -> Actual<String> {
    |it| {
        serde_json::to_string(it.borrowed())
            .expect("JSON conversion to succeed")
            .into()
    }
}

/// A conversion function that can be used with `map` to easily convert any `serde::Serialize`able
/// type into its TOML representation for further checks.
/// Uses `toml` to perform the conversion.
/// Panics when the conversion fails.
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// struct Config {
///     value: u32,
/// }
///
/// let config = Config { value: 42 };
///
/// assert_that(config)
///     .map(toml())
///     .is_equal_to("value = 42\n");
/// ```
#[cfg(feature = "serde")]
pub fn toml<S: serde::Serialize>() -> impl FnOnce(Actual<S>) -> Actual<String> {
    |it| {
        toml::to_string(it.borrowed())
            .expect("TOML conversion to succeed")
            .into()
    }
}

#[cfg(feature = "serde")]
impl<'t, T, M: Mode> AssertThat<'t, T, M>
where
    T: serde::Serialize,
{
    pub fn as_json(self) -> AssertThat<'t, String, M> {
        self.map(json())
    }

    pub fn as_toml(self) -> AssertThat<'t, String, M> {
        self.map(toml())
    }
}
