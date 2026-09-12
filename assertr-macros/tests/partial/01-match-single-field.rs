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

    // `eq` checks equality. `..` leaves the other fields unchecked.
    assert_that!(user).matches(partial!(User {
        name: eq("Alice"),
        ..
    }));
}
