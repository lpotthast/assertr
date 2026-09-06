use renamed_assertr::prelude::*;

struct Ready;

fn main() {
    assert_that!(Ready).matches(partial!(Ready));
}
