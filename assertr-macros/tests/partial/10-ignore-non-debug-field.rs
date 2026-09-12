use renamed_assertr::{matchers::eq, prelude::*};

// Neither this field nor the containing struct implements `Debug` or `PartialEq`.
struct Secret;

struct User {
    age: u32,
    secret: Secret,
}

fn main() {
    let user = User {
        age: 30,
        secret: Secret,
    };

    // Only selected fields need comparison and rendering support.
    assert_that!(user).matches(partial!(User { age: eq(30), .. }));
}
