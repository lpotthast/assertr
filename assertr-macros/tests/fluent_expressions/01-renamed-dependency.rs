use renamed_assertr::prelude::*;

#[renamed_assertr::fluent_expressions]
fn main() {
    let failures = 42.verify(|it| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
}
