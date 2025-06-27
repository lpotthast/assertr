#![allow(dead_code)]

use assertr::prelude::*;
use indoc::formatdoc;

// Deriving `Debug` is now necessary, as we want to actually use `Foo` in an assertion.
#[derive(Debug, AssertrEq)]
pub struct Foo {
    pub id: i32,

    pub name: String,

    pub data: (u32, u32),
}

fn main() {
    let foo = Foo {
        id: 1,
        name: "bob".to_string(),
        data: (42, 100),
    };

    foo.must().be_equal_to(FooAssertrEq {
        id: any(),
        name: any(),
        data: any(),
    });

    foo.must().be_equal_to(FooAssertrEq {
        id: eq(1),
        name: eq("bob".to_string()),
        data: any(),
    });

    assert_that_panic_by(|| {
        foo.must()
            .with_location(false)
            .be_equal_to(FooAssertrEq {
                id: eq(1),
                name: eq("otto".to_string()),
                data: any(),
            })
    })
    .has_type::<String>()
    .is_equal_to(formatdoc! {r#"
            -------- assertr --------
            Expected: FooAssertrEq {{
                id: Eq::Eq(1),
                name: Eq::Eq("otto"),
                data: Eq::Any,
            }}

              Actual: Foo {{
                id: 1,
                name: "bob",
                data: (
                    42,
                    100,
                ),
            }}

            Details: [
                Differences: [
                    "name": expected "otto", but was "bob",
                ],
            ]
            -------- assertr --------
        "#});
}
