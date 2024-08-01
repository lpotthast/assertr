#![allow(dead_code)]

use std::collections::HashMap;

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

    #[assertr_eq(
        map_type = "HashMap<String, BarAssertrEq>",
        compare_with = "::assertr::util::hashmap::compare"
    )]
    pub bars2: HashMap<String, Bar>,
}

fn main() {
    let foo = Foo {
        id: 1,
        bars: vec![Bar {
            id: 42,
        }],
        bars2: HashMap::new(),
    };

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: any(),
        bars: any(),
        bars2: any(),
    });

    assert_that::<Foo>(&foo).is_equal_to(FooAssertrEq {
        id: eq(1),
        bars: eq(vec![BarAssertrEq { id: eq(42) }]),
        bars2: eq(HashMap::new()),
    });
}
