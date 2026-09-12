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
fn fluent_attribute_captures_expressions_with_callback_values() {
    fn check(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
        it.is_equal_to(43)
    }

    let actual = 42;
    let callback: fn(AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> = check;
    let failures = actual.verify(callback);
    assert_that!(failures[0].expression).is_equal_to(Some("actual"));
    let failures = actual.verify_owned(callback);
    assert_that!(failures[0].expression).is_equal_to(Some("actual"));

    let mut calls = 0;
    let mut callback = |it: AssertThat<'static, i32, Capture>| {
        calls += 1;
        it.is_equal_to(43)
    };
    let failures = 42.verify(&mut callback);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    let failures = 42.verify_owned(callback);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    assert_that!(calls).is_equal_to(2);

    let expected = String::from("expected");
    let callback = move |it: AssertThat<'static, String, Capture>| it.is_equal_to(expected);
    let failures = String::from("actual").verify_owned(callback);
    assert_that!(failures[0].expression).is_equal_to(Some("String::from(\"actual\")"));
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_attaches_only_pending_root_expressions() {
    struct NoRenderer;

    fn check(root: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i64, Capture, NoRenderer> {
        let root = root.is_equal_to(43);
        root.derive_owned(|value| value + 1).is_equal_to(45);
        root.derive_owned(|value| value + 1)
            .with_expression("child override")
            .is_equal_to(45);

        root.with_renderer(NoRenderer)
            .map(|actual| assertr::actual::Actual::Owned(i64::from(*actual.borrowed())))
            .with_renderer(DebugRenderer)
            .is_equal_to(43)
            .with_expression("root override")
            .is_equal_to(44)
            .with_renderer(NoRenderer)
    }

    let actual = 42;
    let borrowed = actual.verify(check);
    let owned = actual.verify_owned(check);
    for failures in [borrowed, owned] {
        let expressions: Vec<_> = failures.iter().map(|failure| failure.expression).collect();
        assert_that!(expressions).contains_exactly([
            Some("actual"),
            None,
            Some("child override"),
            Some("actual"),
            Some("root override"),
        ]);
    }
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_preserves_nested_capture_expressions() {
    let actual = 42;
    let mut nested = AssertionFailures::default();
    let failures = actual.verify(|it| {
        let inner = 12;
        nested = inner.verify_owned(|it| it.is_equal_to(13));
        it.is_equal_to(43)
    });
    assert_that!(failures[0].expression).is_equal_to(Some("actual"));
    assert_that!(nested[0].expression).is_equal_to(Some("inner"));
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_attribute_does_not_attach_to_aggregates_from_other_calls() {
    struct User(i32);
    impl User {
        fn verify(self, operation: fn(i32) -> i32) -> AssertionFailures {
            operation(self.0).verify(|it| it.is_equal_to(43))
        }

        fn verify_owned(self, operation: fn(i32) -> i32) -> AssertionFailures {
            operation(self.0).verify_owned(|it| it.is_equal_to(43))
        }
    }

    #[assertr::fluent_expressions]
    fn run() {
        let failures = User(21).verify(|value| value * 2);
        assert_that!(failures[0].expression).is_none();
        let failures = User(21).verify_owned(|value| value * 2);
        assert_that!(failures[0].expression).is_none();
    }

    run();
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_attribute_does_not_attach_to_unrelated_tracked_verification() {
    struct User(i32);
    impl User {
        #[track_caller]
        fn verify(self, operation: fn(i32) -> i32) -> AssertionFailures {
            operation(self.0).verify(|it| it.is_equal_to(43))
        }

        #[track_caller]
        fn verify_owned(self, operation: impl FnOnce(i32) -> i32) -> AssertionFailures {
            operation(self.0).verify_owned(|it| it.is_equal_to(43))
        }
    }

    #[assertr::fluent_expressions]
    fn run() {
        fn double(value: i32) -> i32 {
            value * 2
        }

        let failures = User(21).verify(|value| value * 2);
        assert_that!(failures[0].expression).is_none();
        let callback: fn(i32) -> i32 = |value| value * 2;
        let failures = User(21).verify(callback);
        assert_that!(failures[0].expression).is_none();
        let failures = User(21).verify(double);
        assert_that!(failures[0].expression).is_none();

        let offset = Box::new(21);
        let failures = User(21).verify_owned(move |value| value + *offset);
        assert_that!(failures[0].expression).is_none();
        let offset = Box::new(21);
        let callback = move |value| value + *offset;
        let failures = User(21).verify_owned(callback);
        assert_that!(failures[0].expression).is_none();
    }

    run();
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_attribute_attaches_expressions_inside_tracked_functions() {
    #[track_caller]
    #[assertr::fluent_expressions]
    fn check(actual: i32) -> [AssertionFailures; 2] {
        let borrowed = actual.verify(|it| it.is_equal_to(43));
        let owned = actual.verify_owned(|it| it.is_equal_to(43));
        [borrowed, owned]
    }

    for failures in check(42) {
        assert_that!(failures[0].expression).is_equal_to(Some("actual"));
    }
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_preserves_callback_blocks() {
    let actual = 42;
    let mut creations = 0;
    let failures = actual.verify({
        creations += 1;
        |it| it.is_equal_to(43)
    });
    assert_that!(creations).is_equal_to(1);
    assert_that!(failures[0].expression).is_equal_to(Some("actual"));
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_preserves_assertion_caller_locations() {
    macro_rules! fail_at_caller {
        ($it:ident, $expected:ident) => {{
            $expected = Some(core::panic::Location::caller());
            $it.is_equal_to(43)
        }};
    }

    let mut expected = None;
    let failures = 42.verify(|it| fail_at_caller!(it, expected));
    assert_that!(failures[0].location).is_equal_to(expected);
    let failures = 42.verify_owned(|it| fail_at_caller!(it, expected));
    assert_that!(failures[0].location).is_equal_to(expected);
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_preserves_receiver_and_callback_evaluation_order() {
    use core::cell::RefCell;

    struct User(i32);
    impl User {
        fn verify(self, operation: fn(i32) -> i32) -> i32 {
            operation(self.0) + operation(self.0)
        }

        fn verify_owned(self, operation: fn(i32) -> i32) -> i32 {
            operation(self.0) + operation(self.0)
        }
    }

    fn receiver(events: &RefCell<Vec<&'static str>>) -> User {
        events.borrow_mut().push("receiver");
        User(21)
    }

    fn callback(events: &RefCell<Vec<&'static str>>) -> fn(i32) -> i32 {
        events.borrow_mut().push("callback");
        |value| value * 2
    }

    let events = RefCell::new(Vec::new());
    let result = receiver(&events).verify(callback(&events));
    assert_that!(result).is_equal_to(84);
    let result = receiver(&events).verify_owned(callback(&events));
    assert_that!(result).is_equal_to(84);
    assert_that!(events.into_inner())
        .contains_exactly(["receiver", "callback", "receiver", "callback"]);
}

#[cfg(feature = "fluent")]
#[assertr::fluent_expressions]
#[test]
fn fluent_attribute_evaluates_callback_once_before_repeated_calls() {
    struct User(i32);

    impl User {
        fn verify(self, mut operation: impl FnMut(i32) -> i32) -> i32 {
            operation(self.0) + operation(self.0)
        }

        fn verify_owned(self, operation: impl FnMut(i32) -> i32) -> i32 {
            Self::verify(self, operation)
        }
    }

    let mut events = Vec::new();
    let result = User(21).verify({
        events.push("create");
        |value| {
            events.push("call");
            value * 2
        }
    });
    assert_that!(result).is_equal_to(84);
    assert_that!(events).contains_exactly(["create", "call", "call"]);

    events.clear();
    let result = User(21).verify_owned({
        events.push("create");
        |value| {
            events.push("call");
            value * 2
        }
    });
    assert_that!(result).is_equal_to(84);
    assert_that!(events).contains_exactly(["create", "call", "call"]);
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
