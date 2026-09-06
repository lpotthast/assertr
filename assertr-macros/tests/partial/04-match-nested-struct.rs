use renamed_assertr::prelude::*;

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
        address: partial!(Address { city: "Berlin", .. }),
    }));
}
