#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, PartialEq, AssertrEq)]
pub struct Bar {
    pub id: i32,
}

#[derive(Debug, PartialEq, AssertrEq)]
pub struct Foo {
    pub id: i32,

    #[assertr_eq(map_type = "BarAssertrEq")]
    pub bar: Bar,
}

fn main() {
    let foo = Foo {
        id: 1,
        bar: Bar {
            id: 42,
        },
    };

    assert_that(&foo).is_equal_to_assertr(FooAssertrEq {
        id: any(),
        bar: any(),
    });

    assert_that(&foo).is_equal_to_assertr(FooAssertrEq {
        id: eq(1),
        bar: eq(BarAssertrEq { id: eq(42) }),
    });
}
