use assertr::prelude::*;
use indoc::formatdoc;

#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
}

/// Assertion-generated diagnostics are handed to the failure they belong to and are never stored
/// on the assertion, so a later failure on the same chain cannot pick them up.
#[test]
fn assertion_details_are_scoped_to_the_failure_that_produced_them() {
    use std::collections::VecDeque;

    let failures = assert_that!(VecDeque::from([1, 2, 3]))
        .with_location(false)
        .capture(|it| {
            it.contains_exactly_in_any_order_matching([
                |it: &i32| *it == 1,
                |it: &i32| *it == 2,
                |it: &i32| *it == 9,
            ])
            .contains(42)
        });

    assert_that!(&failures).has_length(2);
    assert_that!(failures[0].facts.as_slice())
        .contains_matching(|it: &assertr::Fact| it.label == "Elements not matched");
    assert_that!(TextReporter.report(&failures[1])).contains("does not contain\n\nExpected: 42");
    assert_that!(failures[1].facts.as_slice())
        .does_not_contain_matching(|it: &assertr::Fact| it.label == "Elements not matched");
}

#[cfg(feature = "derive")]
#[test]
fn equality_differences_are_scoped_to_the_failure_that_produced_them() {
    #[derive(Debug, PartialEq, AssertrEq)]
    struct Data {
        pub age: u32,
    }

    let failures = assert_that!(Data { age: 30 })
        .with_location(false)
        .capture(|it| {
            it.is_equal_to(DataAssertrEq { age: eq(31) })
                .is_equal_to(DataAssertrEq { age: eq(32) })
        });

    assert_that!(&failures).has_length(2);
    assert_that!(TextReporter.report(&failures[0]).as_str())
        .contains(r#""age": expected 31, but was 30"#);
    assert_that!(TextReporter.report(&failures[1]).as_str())
        .contains(r#""age": expected 32, but was 30"#)
        .does_not_contain("expected 31");
}

#[test]
fn detail_messages_are_scoped_to_the_assertion_that_adds_them() {
    let failures = assert_that!(Person { age: 42 })
        .with_location(false)
        .with_detail_message("Checking person...")
        .capture(|it| {
            it.is_equal_to(Person { age: 30 }).satisfies(
                |p| &p.age,
                |age| {
                    age.with_detail_message("Checking age...")
                        .is_greater_than(9000);
                },
            )
        });

    assert_that!(failures).contains_exactly_satisfying([
        |it: AssertThat<AssertionFailure, Capture>| {
            it.satisfies_owned(
                |failure| TextReporter.report(failure),
                |it| {
                    it.is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Person {{ age: 42 }}`

                Expected: Person {{
                    age: 30,
                }}

                  Actual: Person {{
                    age: 42,
                }}

                Messages:
                  - Checking person...
                -------- assertr --------
            "});
                },
            );
        },
        |it: AssertThat<AssertionFailure, Capture>| {
            it.satisfies_owned(
                |failure| TextReporter.report(failure),
                |it| {
                    it.is_equal_to(formatdoc! {r"
                -------- assertr --------
                Actual: 42

                is not greater than

                Expected: 9000

                Messages:
                  - Checking age...
                  - Checking person...
                -------- assertr --------
            "});
                },
            );
        },
    ]);
}
