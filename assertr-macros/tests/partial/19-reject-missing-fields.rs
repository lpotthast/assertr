use renamed_assertr::{matchers::eq, prelude::*};

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    // Omitting `y` requires an explicit `..`.
    assert_that!(Point { x: 1, y: 2 }).matches(partial!(Point { x: eq(1) }));
}
