use renamed_assertr::{matchers::eq, prelude::*};

struct User {
    name: String,
}

fn main() {
    let matcher = {
        let name = String::from("Alice");
        partial!(User {
            name: eq(name.as_str())
        })
    }; // `name` is dropped here, but the matcher still borrows it.

    let user = User {
        name: String::from("Alice"),
    };
    assert_that!(user).matches(matcher);
}
