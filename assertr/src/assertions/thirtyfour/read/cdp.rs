//! Browser-computed accessibility data. No local implementation of ARIA naming or role rules.
use thirtyfour::{
    WebElement,
    cdp::{Cdp, CdpCommand, Empty, RemoteObjectId},
};
use thirtyfour::{error::WebDriverError, prelude::WebDriverResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPartialTree {
    object_id: RemoteObjectId,
    fetch_relatives: bool,
}
impl CdpCommand for GetPartialTree {
    const METHOD: &'static str = "Accessibility.getPartialAXTree";
    type Returns = Tree;
}
#[derive(Deserialize)]
struct Tree {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    description: Option<Text>,
}
#[derive(Deserialize)]
struct Text {
    value: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseObject {
    object_id: RemoteObjectId,
}
impl CdpCommand for ReleaseObject {
    const METHOD: &'static str = "Runtime.releaseObject";
    type Returns = Empty;
}

/// Reads the Chromium accessibility-tree description. Unsupported commands remain errors.
///
/// Acquired remote objects are released after success or failure. Cancellation during a command
/// can leave an object until the caller closes its WebDriver session.
pub async fn accessible_description(element: &WebElement) -> WebDriverResult<String> {
    let object_id = element.cdp_remote_object_id().await?;
    let cdp = Cdp::new(element.handle().clone());
    let tree = cdp
        .send(GetPartialTree {
            object_id: object_id.clone(),
            fetch_relatives: false,
        })
        .await;
    // Release even if reading failed. Session teardown also handles cancellation during a command.
    let released = cdp.send(ReleaseObject { object_id }).await;
    let tree = tree?;
    released?;
    let Some(node) = tree.nodes.into_iter().next() else {
        return Err(WebDriverError::NotFound("accessibility node".into(), "browser returned no accessibility node".into()));
    };
    Ok(node.description.map_or_else(String::new, |text| text.value))
}
