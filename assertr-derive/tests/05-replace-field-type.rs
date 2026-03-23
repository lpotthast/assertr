#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, AssertrEq)]
pub struct Bar {
    pub id: i32,
}

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    #[assertr_eq(map_type = "BarAssertrEq")]
    pub bar: Bar,
}

fn main() {
    let foo = Foo {
        id: 1,
        bar: Bar { id: 42 },
    };

    foo.must().be_equal_to(FooAssertrEq {
        id: any(),
        bar: any(),
    });

    foo.must().be_equal_to(FooAssertrEq {
        id: eq(1),
        bar: eq(BarAssertrEq { id: eq(42) }),
    });
}
