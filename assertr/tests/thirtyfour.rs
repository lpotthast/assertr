//! Public API and Send boundaries, compiled without internal chain access.
#![cfg(feature = "thirtyfour")]
use assertr::{assertions::ThirtyfourWebElementAssertions, prelude::*};
use thirtyfour::WebElement;

#[test]
fn async_composition_is_send_for_owned_and_borrowed_elements() {
    fn require_send(_: impl Future<Output = ()> + Send) {}
    let _example: fn(WebElement) = |element| {
        require_send(async move {
            assert_that!(&element).has_attribute("aria-label").await.starts_with("Open").contains("menu");
            assert_that!(&element).attribute("aria-controls").await.is_none();
            assert_that_owned!(element).has_attribute("id").await.starts_with("menu-");
        });
    };
}

#[test]
fn trait_is_available_without_renderer_support() {
    struct NoRenderer;
    fn available<'a, T: ThirtyfourWebElementAssertions<'a, NoRenderer>>() {}
    available::<AssertThat<'_, WebElement, Panic, NoRenderer>>();
}
