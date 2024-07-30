#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    pub name: String,

    pub timestamp: std::time::Instant,
}

fn main() {
    let foo = Foo {
        id: 1,
        name: "bob".to_string(),
        timestamp: std::time::Instant::now(),
    };

    assert_that(&foo).is_equal_to_assertr(FooAssertrEq {
        id: any(),
        name: any(),
        timestamp: any(),
    });

    assert_that(&foo).is_equal_to_assertr(FooAssertrEq {
        id: eq(1),
        name: eq("bob".to_string()),
        timestamp: any(),
    });
}
