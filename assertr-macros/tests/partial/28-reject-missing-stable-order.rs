use renamed_assertr::{matchers::eq, prelude::*};

fn main() {
    // Sorted rendering does not give a set the StableOrder capability required by elements_are!.
    assert_that!(std::collections::BTreeSet::from([1])).matches(elements_are![eq(1)]);
}
