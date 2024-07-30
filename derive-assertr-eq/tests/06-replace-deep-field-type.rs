#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, PartialEq, AssertrEq)]
pub struct Bar {
    pub id: i32,
}

#[derive(Debug, PartialEq, AssertrEq)]
pub struct Foo {
    pub id: i32,

    #[assertr_eq(map_type = "Vec<BarAssertrEq>")]
    pub bars: Vec<Bar>,
}

fn main() {
    let foo = Foo {
        id: 1,
        bars: vec![Bar {
            id: 42,
        }],
    };

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: any(),
        bars: any(),
    });

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: eq(1),
        bars: eq(vec![BarAssertrEq { id: eq(42) }]),
    });
}
