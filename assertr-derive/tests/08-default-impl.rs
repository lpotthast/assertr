#![allow(dead_code)]

use assertr::prelude::*;
use std::default::Default;

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub field_1: i32,
    pub field_2: i32,
    pub field_3: i32,
}

fn main() {
    let foo = Foo {
        field_1: 1,
        field_2: 2,
        field_3: 3,
    };

    assert_that(foo).is_equal_to(FooAssertrEq {
        field_1: eq(1), 
        ..Default::default()
    });
}
