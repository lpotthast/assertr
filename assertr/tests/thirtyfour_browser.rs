//! Real browser contracts, explicitly invoked with an existing Chromium `WebDriver` endpoint.
#![cfg(feature = "thirtyfour-cdp")]
use assertr::prelude::*;
use thirtyfour::{By, DesiredCapabilities, WebDriver, prelude::*};

#[tokio::test]
#[ignore = "requires ASSERTR_WEBDRIVER_URL and a Chromium browser; see just test-browser"]
async fn browser_contracts() -> WebDriverResult<()> {
    let endpoint = std::env::var("ASSERTR_WEBDRIVER_URL").expect("set ASSERTR_WEBDRIVER_URL");
    let mut caps = DesiredCapabilities::chrome();
    if let Ok(binary) = std::env::var("ASSERTR_CHROME_BINARY") {
        caps.set_binary(&binary)?;
    }
    caps.add_arg("--headless=new")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    let driver = WebDriver::new(endpoint, caps).await?;
    driver
        .goto(concat!(
            "data:text/html,",
            "<button id='trigger' aria-label='Open menu' aria-describedby='help'>Launch</button>",
            "<span id='help'>Useful details</span>",
            "<button id='override' role='link'>Override</button>",
            "<div id='contents' style='display:contents'><span>Contents text</span></div>",
            "<input id='check' type='checkbox' checked>",
            "<button id='hidden' hidden>Hidden</button>",
            "<div inert><button id='inert'>Inert</button></div>"
        ))
        .await?;
    let trigger = driver.find(By::Id("trigger")).await?;
    assert_that!(&trigger)
        .has_attribute("aria-label")
        .await
        .starts_with("Open")
        .contains("menu");
    assert_that!(&trigger).attribute("missing").await.is_none();
    assert_that!(&trigger)
        .accessible_name()
        .await
        .is_equal_to("Open menu");
    assert_that!(&trigger)
        .accessible_role()
        .await
        .is_equal_to("button");
    assert_that!(&trigger)
        .accessible_description()
        .await
        .is_equal_to("Useful details");
    assert_that!(&trigger).text().await.is_equal_to("Launch");
    assert_that!(&trigger)
        .text_content()
        .await
        .get_some()
        .is_equal_to("Launch");
    assert_that!(&trigger).displayed().await.is_true();
    assert_that!(&trigger).enabled().await.is_true();
    trigger.click().await?;
    assert_that!(&trigger).focused().await.is_true();
    let other = driver.find(By::Id("override")).await?;
    assert_that!(&other)
        .accessible_role()
        .await
        .is_equal_to("link");
    assert_that!(&other).focused().await.is_false();
    let contents = driver.find(By::Id("contents")).await?;
    assert_that!(&contents)
        .inner_text()
        .await
        .is_equal_to("Contents text");
    let check = driver.find(By::Id("check")).await?;
    assert_that!(&check).selected().await.is_true();
    let hidden = driver.find(By::Id("hidden")).await?;
    assert_that!(&hidden).displayed().await.is_false();
    assert_that!(&hidden).accessible_name().await.is_empty();
    let inert = driver.find(By::Id("inert")).await?;
    assert_that!(&inert).accessible_name().await.is_empty();
    driver.refresh().await?;
    assert_that!(trigger.attr("aria-label").await.is_err()).is_true();
    assert_that!(
        assertr::assertions::thirtyfour::read::focused(&trigger)
            .await
            .is_err()
    )
    .is_true();
    driver.quit().await
}
