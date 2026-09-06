use renamed_assertr::prelude::*;

struct User {
    name: String,
    age: u32,
}

fn main() {
    let user = User {
        name: String::from("Alice"),
        age: 30,
    };

    // A bare value checks equality. `..` leaves the other fields unchecked.
    assert_that!(user).matches(partial!(User { name: "Alice", .. }));
}
