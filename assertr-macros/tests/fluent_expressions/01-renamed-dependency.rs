use renamed_assertr::prelude::*;

#[renamed_assertr::fluent_expressions]
fn main() {
    let failures = 42.verify(|it| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("42"));

    fn check(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
        it.is_equal_to(43)
    }
    let failures = 42.verify(check);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    let failures = 42.verify(|it: AssertThat<'_, i32, Capture>| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    let failures = [1, 2].into_iter().verify_owned(|it| it.contains(3));
    assert_that!(failures).has_length(1);
    assert_that!(failures[0].expression).is_equal_to(Some("[1, 2].into_iter()"));
}
