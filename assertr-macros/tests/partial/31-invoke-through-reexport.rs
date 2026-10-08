// A facade crate may re-export assertr. The macro forwards its own crate path to the generated
// code instead of looking up the dependency name of the calling crate.
mod facade {
    pub use renamed_assertr as assertr;
}

use facade::assertr::{assert_that, matchers::eq};

struct User {
    name: &'static str,
    age: u32,
}

fn main() {
    let user = User {
        name: "Alice",
        age: 30,
    };
    assert_that!(user).matches(facade::assertr::partial!(User { name: eq("Alice"), .. }));
    assert_that!(user).matches(facade::assertr::prelude::partial!(User { age: eq(30), .. }));
}
