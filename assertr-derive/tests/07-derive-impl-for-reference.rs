#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Debug, AssertrEq)]
pub struct Bar {
    pub id: i32,
}

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    #[assertr_eq(
        map_type = "Vec<BarAssertrEq>",
        compare_with = "::assertr::cmp::slice::compare"
    )]
    pub bars: Vec<Bar>,
}

fn main() {
    let foo = Foo {
        id: 1,
        bars: vec![Bar { id: 42 }],
    };

    let bars_refs = foo.bars.iter().collect::<Vec<_>>();

    // This must compile without errors.
    // It should only compile when AssertrPartialEq
    // was not only implemented for Bar, but also for &Bar!
    bars_refs.must().contain(BarAssertrEq { id: any() });
}
