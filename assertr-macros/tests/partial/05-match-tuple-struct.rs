use renamed_assertr::{matchers::eq, prelude::*};

struct Point(i32, i32, i32);

fn main() {
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(eq(1), eq(2), eq(3))));

    // `_` skips one position. A final `..` skips all remaining positions.
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(eq(1), _, eq(3))));
    assert_that!(Point(1, 2, 3)).matches(partial!(Point(eq(1), ..)));
}
