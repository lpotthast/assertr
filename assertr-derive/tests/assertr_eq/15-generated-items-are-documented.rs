#![deny(missing_docs)]
//! Verifies that `AssertrEq` documents its generated public items.

use assertr_derive::AssertrEq;

/// A documented public type used to verify generated documentation.
#[derive(AssertrEq)]
pub struct Documented {
    /// A documented source field.
    pub value: u32,
}

fn main() {
    let _ = DocumentedAssertrEq::default();
}
