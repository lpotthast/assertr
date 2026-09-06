use renamed_assertr::prelude::*;

struct Point(i32, i32, i32);

fn main() {
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(1, 2, 3)));

    // `_` skips one position. A final `..` skips all remaining positions.
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(1, _, 3)));
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(1, ..)));
}
