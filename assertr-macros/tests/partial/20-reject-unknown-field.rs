use renamed_assertr::{matchers::eq, prelude::*};

struct Point {
    x: i32,
}

fn main() {
    assert_that!(Point { x: 1 }).matches(partial!(Point { y: eq(1), .. }));
}
