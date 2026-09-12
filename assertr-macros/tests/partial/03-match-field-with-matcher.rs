use renamed_assertr::matchers::eq;
use renamed_assertr::{matchers::ge, prelude::*};

struct User {
    name: String,
    age: u32,
}

fn main() {
    let user = User {
        name: String::from("Alice"),
        age: 30,
    };

    // Values and matchers can be mixed in the same expectation.
    assert_that!(user).matches(partial!(User {
        name: eq("Alice"),
        age: ge(18),
    }));
}
