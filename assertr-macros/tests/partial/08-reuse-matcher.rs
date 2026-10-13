use renamed_assertr::matchers::eq;
use renamed_assertr::{matchers::dereferenced, prelude::*};

struct User {
    age: u32,
}

fn main() {
    let user = User { age: 30 };
    let matcher = partial!(User { age: eq(30) });

    assert_that!(user).matches(&matcher);
    assert_that!(user).matches(&matcher);

    // A partial matcher accepts reference subjects directly. Other matchers need an adapter.
    assert_that_owned!(&user).matches(&matcher);
    assert_that_owned!(&user).matches(dereferenced(&matcher));
}
