#![allow(dead_code)]

use assertr::prelude::*;
use derive_assertr_eq::AssertrEq;

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
        id: assertr::Eq::Any,
        name: assertr::Eq::Any,
        timestamp: assertr::Eq::Any,
    });

    assert_that(&foo).is_equal_to_assertr(FooAssertrEq {
        id: assertr::Eq::Eq(1),
        name: assertr::Eq::Eq("bob".to_string()),
        timestamp: assertr::Eq::Any,
    });
}
