#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    // This field is not public and not annotated, it should not be present in FooAssertrEq.
    name: String,
}

fn main() {
    let _ = FooAssertrEq { id: any() };
}
