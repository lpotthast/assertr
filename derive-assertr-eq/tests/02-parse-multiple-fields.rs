#![allow(dead_code)]

use derive_assertr_eq::AssertrEq;

#[derive(AssertrEq)]
pub struct Foo {
    pub id: i32,
    pub name: String,
}

fn main() {}
