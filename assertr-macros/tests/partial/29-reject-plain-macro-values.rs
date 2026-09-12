use renamed_assertr::prelude::*;
use std::collections::BTreeMap;

fn main() {
    // Collection entries and keyed value expectations all require explicit matchers.
    assert_that!([1]).matches(elements_are![1]);
    assert_that!([1]).matches(elements_are_in_any_order![1]);
    assert_that!([1]).contains_exactly_matching(matchers![1]);
    assert_that!(BTreeMap::from([("value", 1)])).matches(entries_are![("value", 1)]);
}
