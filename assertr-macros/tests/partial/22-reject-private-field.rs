use renamed_assertr::{matchers::eq, prelude::*};

mod domain {
    pub struct Point {
        x: i32,
    }

    pub fn point() -> Point {
        Point { x: 1 }
    }
}

fn main() {
    // `..` skips other fields, but does not grant access to a selected private field.
    assert_that!(domain::point()).matches(partial!(domain::Point { x: eq(1), .. }));
}
