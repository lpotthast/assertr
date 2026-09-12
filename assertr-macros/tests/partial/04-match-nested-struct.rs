use renamed_assertr::{matchers::eq, prelude::*};

struct Address {
    city: String,
    street: String,
}

struct User {
    address: Address,
}

fn main() {
    let user = User {
        address: Address {
            city: String::from("Berlin"),
            street: String::from("Example Street"),
        },
    };

    assert_that!(user).matches(partial!(User {
        address: partial!(Address {
            city: eq("Berlin"),
            ..
        }),
    }));
}
