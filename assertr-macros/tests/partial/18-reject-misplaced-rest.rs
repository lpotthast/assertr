use renamed_assertr::prelude::*;

struct Point {
    x: i32,
}

fn main() {
    let _ = partial!(Point { .., x: renamed_assertr::matchers::eq(1) });
}
