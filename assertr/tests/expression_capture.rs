use assertr::prelude::*;
use indoc::formatdoc;

#[cfg(feature = "fluent")]
fn assert_string_panic_contains(panic: std::thread::Result<()>, expected: &str) {
    let panic = assert_that_owned!(panic).get_err().unwrap_inner();
    let message = assert_that_owned!(panic.downcast::<String>().map_err(|_| ()))
        .get_ok()
        .unwrap_inner();
    assert_that!(message.as_str()).contains(expected);
}

#[test]
fn macro_entry_points_capture_the_asserted_expression() {
    let answer = 42;
    let failures = assert_that!(answer + 1)
        .with_location(false)
        .capture(|it| it.is_equal_to(42));
    assert_that!(failures[0].expression).is_equal_to(Some("answer + 1"));

    let failures = assert_that_owned!(String::from("actual"))
        .with_location(false)
        .capture(|it| it.is_equal_to("expected"));
    assert_that!(failures[0].expression).is_equal_to(Some("String::from(\"actual\")"));
}

#[test]
#[cfg(feature = "std")]
fn type_entry_point_uses_the_asserted_type_name() {
    let failures = assert_that_type::<u8>()
        .with_location(false)
        .capture(MemAssertions::needs_drop);

    assert_that!(failures[0].expression).is_equal_to(Some(core::any::type_name::<u8>()));
}

#[test]
fn the_human_readable_adapter_renders_a_subject_name_and_expression_as_separate_fields() {
    let failures = assert_that!(42)
        .with_subject_name("answer")
        .with_location(false)
        .capture(|it| it.is_equal_to(43));

    assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {"
            -------- assertr --------
            Subject: answer
            Expression: `42`

            Expected: 43

              Actual: 42
            -------- assertr --------
        "});
}

#[test]
fn the_human_readable_adapter_caps_expressions_to_one_line_and_one_hundred_characters() {
    const LONG_EXPRESSION: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvw";

    let failures = assert_that!(42)
        .with_expression("first line\nsecond line")
        .with_location(false)
        .capture(|it| it.is_equal_to(43));
    assert_that!(ToHumanReadableText.render(&failures[0]))
        .contains("Expression: `first line...`\n\n");

    let failures = assert_that!(42)
        .with_expression(LONG_EXPRESSION)
        .with_location(false)
        .capture(|it| it.is_equal_to(43));
    let rendered = ToHumanReadableText.render(&failures[0]);
    let expression_line = rendered
        .lines()
        .find(|line| line.starts_with("Expression:"))
        .expect("expression line");
    assert_that!(expression_line.chars().count()).is_equal_to("Expression: ``".len() + 100);
    assert_that!(expression_line).ends_with("...`");
    assert_that!(failures[0].expression).is_equal_to(Some(LONG_EXPRESSION));
}

#[test]
fn derived_chains_start_without_the_root_expression() {
    let root = assert_that!(("value", 42)).with_location(false);
    let failures = root
        .derive_owned(|value| value.1)
        .capture(|it| it.is_equal_to(43));

    assert_that!(failures[0].expression).is_none();
    assert_that!(ToHumanReadableText.render(&failures[0])).does_not_contain("Subject:");
    assert_that!(ToHumanReadableText.render(&failures[0])).does_not_contain("Expression:");
}

#[cfg(feature = "fluent")]
#[test]
fn plain_fluent_entry_points_do_not_invent_expressions() {
    let failures = 42.verify(|it| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_none();

    let failures = 42.verify_owned(|it| it.is_equal_to(43));
    assert_that!(failures[0].expression).is_none();
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_preserves_unrelated_verify_methods() {
    struct User(i32);

    impl User {
        fn verify(self, operation: impl FnOnce(i32) -> i32) -> i32 {
            operation(self.0)
        }

        fn verify_owned(self, operation: impl FnOnce(i32) -> i32) -> i32 {
            operation(self.0)
        }
    }

    let verified: i32 = User(41).verify(|value: i32| value + 1);
    assert_that!(verified).is_equal_to(42);

    let verified_owned: i32 = User(41).verify_owned(|value: i32| value + 1);
    assert_that!(verified_owned).is_equal_to(42);
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_captures_all_four_entry_receivers() {
    macro_rules! answer {
        () => {
            42
        };
    }

    assert_string_panic_contains(
        std::panic::catch_unwind(|| {
            42.must().be_equal_to(43);
        }),
        "Expression: `42`\n\n",
    );

    assert_string_panic_contains(
        std::panic::catch_unwind(|| {
            String::from("actual").must_owned().be_equal_to("expected");
        }),
        "Expression: `String::from(\"actual\")`\n\n",
    );

    let failures = 42.verify(|it: AssertThat<'_, i32, Capture>| it.be_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("42"));

    let failures = String::from("actual")
        .verify_owned(|it: AssertThat<'_, String, Capture>| it.be_equal_to("expected"));
    assert_that!(failures[0].expression).is_equal_to(Some("String::from(\"actual\")"));

    let failures = answer!().verify(|it| it.be_equal_to(43));
    assert_that!(failures[0].expression).is_equal_to(Some("answer!()"));

    let mut value = String::from("actual");
    let reference = &mut value;
    let failures = reference.verify(|it| it.be_equal_to("expected"));
    assert_that!(failures[0].expression).is_equal_to(Some("reference"));
    reference.push('!');
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_does_not_capture_entry_calls_generated_by_macros() {
    macro_rules! verify_failure {
        () => {
            42.verify(|it| it.is_equal_to(43))
        };
    }

    let failures = verify_failure!();
    assert_that!(failures[0].expression).is_none();
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
mod fluent_module_scope {
    use super::*;

    mod nested {
        use super::*;

        #[test]
        fn captures_receivers_in_nested_modules() {
            let failures = 42.verify(|it| it.is_equal_to(43));
            assert_that!(failures[0].expression).is_equal_to(Some("42"));
        }
    }
}
