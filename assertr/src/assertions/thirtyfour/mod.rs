//! Async browser observations that continue as ordinary value assertions.
//!
//! [`ThirtyfourWebElementAssertions::has_attribute`] checks presence and extracts a string.
//! Other projections retain optional values where the browser can report absence. A read is one
//! snapshot, not a wait. Use `eventually_ok` or `consistently_ok` with a fresh read closure for
//! repeated observations, and `giving_up_on(|_| true)` to terminate on WebDriver errors.
//!
//! These adapters require panic mode and detach before awaiting. Their futures are `Send` when
//! the renderer is. A returned assertion chain must be finished before the next suspension.

use alloc::{format, string::String};
use core::panic::Location;
use thirtyfour::{WebElement, error::WebDriverError, prelude::WebDriverResult};
use crate::{
    AssertThat, actual::Actual, assertions::core::option::IsSome,
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic, renderer::{DebugRenderer, ValueRenderer},
};

/// Browser-specific reads on a real thirtyfour element, followed by ordinary value assertions.
///
/// Protocol errors fail at the read call. Missing attributes only fail with `has_attribute`.
/// Borrowed handles remain available for further reads and actions.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ThirtyfourWebElementAssertions<'t, R = DebugRenderer> {
    /// Asserts presence and continues on the attribute's string, including an empty string.
    #[track_caller]
    fn has_attribute(self, name: impl Into<String>) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads an optional attribute. Use `.is_none()` for absence or `.get_some()` to extract it.
    #[track_caller]
    fn attribute(self, name: impl Into<String>) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where R: ValueRenderer<str> + ValueRenderer<WebDriverError>;
}

impl<'t, R> ThirtyfourWebElementAssertions<'t, R> for AssertThat<'t, WebElement, Panic, R> {
    #[track_caller]
    fn has_attribute(self, name: impl Into<String>) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where R: ValueRenderer<str> + ValueRenderer<WebDriverError> {
        let name = name.into();
        let location = Location::caller();
        let read = observe(self, format!("attribute {name}"), move |element| async move { element.attr(name).await });
        async move {
            let value = read.await.apply_assertion_after_tracking(IsSome, location);
            value.map(|actual| Actual::Owned(actual.unwrap_owned().expect("presence was checked")))
        }
    }

    #[track_caller]
    fn attribute(self, name: impl Into<String>) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where R: ValueRenderer<str> + ValueRenderer<WebDriverError> {
        let name = name.into();
        observe(self, format!("attribute {name}"), move |element| async move { element.attr(name).await })
    }
}

/// Detach before constructing the future. Never hold a chain across the browser await.
#[track_caller]
fn observe<'t, T: 't, R, F, Fut>(chain: AssertThat<'t, WebElement, Panic, R>, operation: String, read: F)
    -> impl Future<Output = AssertThat<'t, T, Panic, R>>
where
    R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    F: FnOnce(WebElement) -> Fut,
    Fut: Future<Output = WebDriverResult<T>>,
{
    let location = Location::caller();
    chain.track_assertion();
    let element = chain.actual().clone();
    let identity = element.element_id().to_string();
    let provenance = format!("Browser observation: {} on element {}", chain.render().value(operation.as_str()), chain.render().value(identity.as_str()));
    let (_, detached) = chain.with_detail_message(provenance).detach();
    async move {
        match read(element).await {
            Ok(value) => detached.attach(Actual::Owned(value)),
            Err(error) => {
                let render = detached.render();
                let failure = FailureBuilder::new::<WebElement>(FailureKind::Other)
                    .relation("could not be observed")
                    .fact(Fact::labelled("Operation", render.value(operation.as_str())))
                    .fact(Fact::labelled("Element", render.value(identity.as_str())))
                    .fact(Fact::labelled("WebDriver error", render.value(&error)));
                detached.raise_at(failure, location)
            }
        }
    }
}

#[cfg(test)]
mod tests;
