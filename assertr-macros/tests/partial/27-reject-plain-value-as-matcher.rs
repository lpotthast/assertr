use renamed_assertr::prelude::*;

fn main() {
    // Bare values become equality matchers inside `partial!`, but `.matches` requires a matcher.
    assert_that!(1).matches(1);
}
