use assertr::prelude::*;
use indoc::formatdoc;

#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
}

/// Assertion-generated diagnostics are handed to the failure they belong to and are never stored on
/// the assertion, so a later failure on the same chain cannot pick them up.
mod assertion_details {
    use super::*;

    #[test]
    fn assertion_details_are_scoped_to_the_failure_that_produced_them() {
        use std::collections::VecDeque;

        let failures = assert_that!(VecDeque::from([1, 2, 3]))
            .with_location(false)
            .capture(|it| {
                it.contains_exactly_in_any_order_matching(assertr::matchers::predicate_list([
                    |it: &i32| *it == 1,
                    |it: &i32| *it == 2,
                    |it: &i32| *it == 9,
                ]))
                .contains(42)
            });

        assert_that!(&failures).has_length(2);
        assert_that!(failures[0].children[0].relation.as_deref())
            .is_equal_to(Some("has no distinct matching element"));
        assert_that!(ToHumanReadableText.render(&failures[1]))
            .contains("does not contain\n\nExpected: 42");
        assert_that!(failures[1].children).is_empty();
        assert_that!(failures[1].constraint).is_none();
    }
}

#[cfg(feature = "matchers")]
mod matcher_differences {
    use super::*;

    #[test]
    fn matcher_differences_are_scoped_to_the_failure_that_produced_them() {
        #[derive(Debug, PartialEq)]
        struct Data {
            pub age: u32,
        }

        let failures = assert_that!(Data { age: 30 })
            .with_location(false)
            .capture(|it| {
                it.matches(partial!(Data { age: 31 }))
                    .matches(partial!(Data { age: 32 }))
            });

        assert_that!(&failures).has_length(2);
        assert_that!(ToHumanReadableText.render(&failures[0]).as_str()).contains("Expected: 31");
        assert_that!(ToHumanReadableText.render(&failures[1]).as_str())
            .contains("Expected: 32")
            .does_not_contain("Expected: 31");
    }
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
                |failure| ToHumanReadableText.render(failure),
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
                |failure| ToHumanReadableText.render(failure),
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
