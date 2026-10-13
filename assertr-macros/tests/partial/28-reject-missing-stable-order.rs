use renamed_assertr::{matchers::eq, prelude::*};

fn main() {
    // Sorted rendering does not give a hash set the StableOrder capability required by
    // elements_are!, because its iteration order is unspecified.
    assert_that!(std::collections::HashSet::from([1])).matches(elements_are![eq(1)]);
}
