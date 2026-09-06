use renamed_assertr::prelude::*;

struct Point {
    x: i32,
}

fn main() {
    assert_that!(Point { x: 1 }).matches(partial!(Point { x: "text" }));
}
