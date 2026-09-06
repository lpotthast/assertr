#![deny(missing_docs)]
//! Macro expansion must compile in a downstream crate that denies missing documentation.

use renamed_assertr::prelude::*;

struct User {
    age: u32,
}

fn main() {
    assert_that!(User { age: 30 }).matches(partial!(User { age: 30 }));
}
