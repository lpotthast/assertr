//! Fallible browser reads usable in ordinary code and assertr observation closures.
//!
//! Native reads remain available directly on `WebElement`. These functions supply operations
//! missing from thirtyfour or with explicitly different text semantics. They never poll or
//! reacquire a node, and retain `WebDriver` errors for the caller's retry policy.

#[cfg(feature = "thirtyfour-cdp")]
mod cdp;

#[cfg(feature = "thirtyfour-cdp")]
pub use cdp::accessible_description;
use thirtyfour::{
    ElementId, RequestData, SessionId, WebElement, common::command::FormatRequestData,
    prelude::WebDriverResult,
};

/// A standard command reading a browser-computed property of an element.
#[derive(Debug)]
struct Computed {
    element: ElementId,
    endpoint: &'static str,
}

impl FormatRequestData for Computed {
    fn format_request(&self, session: &SessionId) -> RequestData {
        RequestData::new(
            http::Method::GET,
            format!(
                "/session/{session}/element/{}/{}",
                self.element, self.endpoint
            ),
        )
    }
}

/// Reads the computed property behind `endpoint`.
async fn computed(element: &WebElement, endpoint: &'static str) -> WebDriverResult<String> {
    let command = Computed {
        element: element.element_id(),
        endpoint,
    };
    element.handle().cmd(command).await?.value()
}

/// Reads the browser-computed accessible name using the standard computed-label endpoint.
///
/// # Errors
/// Returns the original browser error if the command fails or its response cannot be decoded.
pub async fn accessible_name(element: &WebElement) -> WebDriverResult<String> {
    computed(element, "computedlabel").await
}

/// Reads the browser-computed role using the standard computed-role endpoint.
///
/// # Errors
/// Returns the original browser error if the command fails or its response cannot be decoded.
pub async fn accessible_role(element: &WebElement) -> WebDriverResult<String> {
    computed(element, "computedrole").await
}

/// Reads and trims DOM `innerText`, including text under `display: contents`.
///
/// This deliberately differs from `WebDriver`'s rendered-text command. A missing property is an
/// error, rather than evidence of empty text (for example on elements without `innerText`).
///
/// # Errors
/// Returns the browser error if the command fails or cannot be decoded, or `NotFound` when
/// the element has no `innerText` property.
pub async fn inner_text(element: &WebElement) -> WebDriverResult<String> {
    element
        .prop("innerText")
        .await?
        .map(|text| text.trim().to_owned())
        .ok_or_else(|| {
            thirtyfour::error::WebDriverError::NotFound(
                "innerText".into(),
                "element has no innerText property".into(),
            )
        })
}

/// Checks focus by comparing the active node's remote identity within the element's session.
///
/// # Errors
/// Returns the original browser error if the command fails or its response cannot be decoded.
pub async fn focused(element: &WebElement) -> WebDriverResult<bool> {
    // Check the fixed handle first: a stale handle must fail, not become a negative focus result.
    element.tag_name().await?;
    let active = element.handle().active_element().await?;
    Ok(active.element_id() == element.element_id())
}
