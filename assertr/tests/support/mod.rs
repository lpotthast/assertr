//! Helpers shared by integration tests. Include with `mod support;`.

use assertr::renderer::{Rendered, RenderedBody};

/// The text of a rendered leaf.
///
/// # Panics
///
/// Panics when `value` is not a text node.
pub fn text(value: &Rendered) -> &str {
    match &value.body {
        RenderedBody::Text { text, .. } => text,
        body => panic!("expected a text node, got {body:?}"),
    }
}

/// The text of an optional rendered leaf. See [`text`].
pub fn text_opt(value: Option<&Rendered>) -> Option<&str> {
    value.map(text)
}
