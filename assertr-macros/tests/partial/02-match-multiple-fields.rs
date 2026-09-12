use renamed_assertr::{matchers::eq, prelude::*};

struct User {
    name: String,
    age: u32,
}

fn main() {
    let user = User {
        name: String::from("Alice"),
        age: 30,
    };

    assert_that!(user).matches(partial!(User {
        name: eq("Alice"),
        age: eq(30),
    }));
}
