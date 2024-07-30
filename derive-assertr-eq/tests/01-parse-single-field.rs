#![allow(dead_code)]

use derive_assertr_eq::AssertrEq;

#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,
}

fn main() {}
