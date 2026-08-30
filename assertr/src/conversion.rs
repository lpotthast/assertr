#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
use crate::AssertThat;
#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
use crate::actual::Actual;
#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
use crate::mode::Mode;

/// Returns a `map` adapter that serializes the subject as JSON with `serde_json`.
///
/// # Panics
///
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
/// assert_that!(person)
///     .map(json())
///     .is_equal_to(r#"{"age":42}"#);
/// ```
#[cfg(feature = "serde-json")]
pub fn json<S: serde::Serialize>() -> impl FnOnce(Actual<S>) -> Actual<String> {
    |it| {
        serde_json::to_string(it.borrowed())
            .expect("JSON conversion to succeed")
            .into()
    }
}

/// Returns a `map` adapter that serializes the subject as TOML with `toml`.
///
/// # Panics
///
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
/// assert_that!(config)
///     .map(toml())
///     .is_equal_to("value = 42\n");
/// ```
#[cfg(feature = "serde-toml")]
pub fn toml<S: serde::Serialize>() -> impl FnOnce(Actual<S>) -> Actual<String> {
    |it| {
        toml::to_string(it.borrowed())
            .expect("TOML conversion to succeed")
            .into()
    }
}

#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R>
where
    T: serde::Serialize,
{
    /// Converts the subject to JSON for further assertions.
    #[cfg(feature = "serde-json")]
    #[must_use]
    pub fn as_json(self) -> AssertThat<'t, String, M, R> {
        self.map(json())
    }

    /// Converts the subject to TOML for further assertions.
    #[cfg(feature = "serde-toml")]
    #[must_use]
    pub fn as_toml(self) -> AssertThat<'t, String, M, R> {
        self.map(toml())
    }
}
