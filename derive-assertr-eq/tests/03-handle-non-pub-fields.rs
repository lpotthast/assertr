#![allow(dead_code)]

use derive_assertr_eq::AssertrEq;

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    // This field is not public and not annotated, it should not be present in FooAssertrEq.
    name: String,
}

fn main() {
    let _ = FooAssertrEq { id: assertr::Eq::Any };
}
