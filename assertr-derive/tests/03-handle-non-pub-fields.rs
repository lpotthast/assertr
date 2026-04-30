#![allow(dead_code)]

use assertr::prelude::*;

#[derive(Clone, Copy)]
struct Renderer;

struct Secret;

#[derive(AssertrEq)]
pub struct Foo {
    pub id: i32,

    // This field is not public, so it should neither be present in FooAssertrEq nor in generated
    // renderer bounds!
    secret: Secret,
}

impl AssertionRenderer<Foo> for Renderer {
    fn fmt(&self, _value: &Foo, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Foo(..)")
    }
}

impl AssertionRenderer<FooAssertrEq> for Renderer {
    fn fmt(&self, _value: &FooAssertrEq, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("FooAssertrEq(..)")
    }
}

impl AssertionRenderer<i32> for Renderer {
    fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(value, f)
    }
}

fn main() {
    let _ = FooAssertrEq { id: any() };

    Foo {
        id: 1,
        secret: Secret,
    }
    .must()
    .with_renderer(Renderer)
    .be_equal_to(FooAssertrEq { id: eq(1) });
}
