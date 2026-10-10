//! Public API and Send boundaries, compiled without internal chain access.
#![cfg(feature = "thirtyfour")]
use assertr::prelude::*;
use thirtyfour::WebElement;

fn require_send(_: impl Future<Output = ()> + Send) {}

#[test]
fn async_composition_is_send_for_owned_and_borrowed_elements() {
    let _: fn(WebElement) = |element| {
        require_send(async move {
            assert_that!(&element)
                .has_attribute("aria-label")
                .await
                .starts_with("Open")
                .contains("menu");
            assert_that!(&element)
                .attribute("aria-controls")
                .await
                .is_none();
            assert_that_owned!(element)
                .has_attribute("id")
                .await
                .starts_with("menu-");
        });
    };
}

#[test]
fn transformed_observations_and_validated_snapshots_remain_send() {
    let _: fn(WebElement) = |element| {
        require_send(async move {
            assert_that!(&element)
                .inner_text()
                .await
                .map_owned(|text| text.parse::<u32>())
                .get_ok()
                .is_equal_to(3);
            assert_that!(&element)
                .attribute("aria-expanded")
                .await
                .derive_owned(|value| value.as_deref())
                .is_equal_to(Some("true"));
            let message = assert_that!(&element)
                .property("validationMessage")
                .await
                .get_some()
                .is_not_blank()
                .actual()
                .clone();
            assert_that!(&element)
                .accessible_name()
                .await
                .contains(message);
            let label_id = {
                let labels = assert_that!(&element)
                    .has_attribute("aria-labelledby")
                    .await;
                let ids = labels
                    .derive_owned(|labels| labels.split(' ').collect::<Vec<_>>())
                    .has_length(2);
                ids.derive_owned(|ids| ids[0]).is_equal_to("self");
                ids.actual()[1].to_owned()
            };
            assert_that!(&element)
                .has_attribute("id")
                .await
                .is_not_equal_to(label_id);
        });
    };
}
