use renamed_assertr::{AssertionFailure, prelude::*};

// `#[non_exhaustive]` requires `..` only outside the defining crate, so use an external type.
// A function parameter avoids unrelated setup to construct an AssertionFailure.
fn check(failure: &AssertionFailure) {
    assert_that!(failure).matches(partial!(AssertionFailure {}));
}

fn main() {}
