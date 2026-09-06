use renamed_assertr::prelude::*;

fn main() {
    // A bare identifier must resolve to a constructor, not become a catch-all pattern binding.
    assert_that!(123).matches(partial!(ThisDoesNotExist));
    assert_that!(123).matches(partial!(variant ThisDoesNotExist));
}
