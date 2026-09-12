use renamed_assertr::prelude::*;

fn main() {
    // Like macro expectations, `.matches` requires an explicit matcher such as `eq(1)`.
    assert_that!(1).matches(1);
}
