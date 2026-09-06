use renamed_assertr::{matchers::elements_are, prelude::*};

fn main() {
    // An empty list has no expectations from which to infer the element type.
    let empty = matchers![];
    assert_that!([0; 0]).matches(elements_are(&empty));
}
