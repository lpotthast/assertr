use assertr::IntoAssertContext;
use assertr::prelude::*;
use indoc::formatdoc;

#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
}

#[test]
fn test() {
    assert_that_owned(Person { age: 42 })
        .with_location(false)
        .with_capture()
        .with_detail_message("Checking person...")
        .is_equal_to(Person { age: 30 })
        .satisfies(
            |p| p.age,
            |age| {
                age.with_detail_message("Checking age...")
                    .is_greater_than(9000);
            },
        )
        .capture_failures()
        .assert()
        .contains_exactly::<String>([
            formatdoc! {r#"
                -------- assertr --------
                Expected: Person {{
                    age: 30,
                }}

                  Actual: Person {{
                    age: 42,
                }}

                Details: [
                    Checking person...,
                ]
                -------- assertr --------
            "#},
            formatdoc! {r#"
                -------- assertr --------
                Actual: 42

                is not greater than

                Expected: 9000

                Details: [
                    Checking age...,
                    Checking person...,
                ]
                -------- assertr --------
            "#},
        ]);
}
