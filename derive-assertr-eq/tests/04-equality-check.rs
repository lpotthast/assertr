#![allow(dead_code)]

use assertr::prelude::*;

// Deriving `Debug` is now necessary, as we want to actually use `Foo` in an assertion.
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

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: any(),
        name: any(),
        timestamp: any(),
    });

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: eq(1),
        name: eq("bob".to_string()),
        timestamp: any(),
    });
}
