use renamed_assertr::prelude::*;

macro_rules! answer {
    () => {
        42
    };
}

#[renamed_assertr::fluent_expressions]
fn main() {
    answer!().must().is_equal_to(42);
    let failures = answer!().verify(|it| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("answer!()"));
}
