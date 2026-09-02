use renamed_assertr::prelude::*;

fn fails_i32<'t>(it: AssertThat<'t, i32, Capture>) -> AssertThat<'t, i32, Capture> {
    it.is_equal_to(43)
}

#[renamed_assertr::fluent_expressions]
fn main() {
    let failures = 42.verify(
        |it: AssertThat<'_, i32, Capture>| it.is_equal_to(43),
    );
    assert_that!(failures[0].expression).is_equal_to(Some("42"));

    let failures = String::from("actual").verify_owned(
        |it: AssertThat<'_, String, Capture>| it.is_equal_to("expected"),
    );
    assert_that!(failures[0].expression).is_equal_to(Some("String::from(\"actual\")"));

    let failures = 42.verify(fails_i32);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
}
