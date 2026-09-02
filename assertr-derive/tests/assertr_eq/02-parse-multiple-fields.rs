#![allow(dead_code)]

use renamed_assertr::prelude::*;
use assertr_derive::AssertrEq;

#[derive(AssertrEq)]
pub struct Foo {
    pub id: i32,
    pub name: String,
}

fn main() {
    let _ = FooAssertrEq {
        id: any(),
        name: any(),
    };
}
