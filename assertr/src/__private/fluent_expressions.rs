//! Macro-only expression-aware support for terminal fluent entry points.

use core::panic::Location;

use crate::failure::AssertionFailures;

/// Completes expression attachment without changing the original call's result type.
///
/// Keeping `T` on both sides supplies the generated closure's expected input type before its
/// specialized attachment is resolved. The macro places this call at the original method span,
/// so its caller location identifies the same entry as the runtime's tracked fluent method.
#[track_caller]
pub fn finish<T>(result: T, attach: impl FnOnce(T, &'static Location<'static>) -> T) -> T {
    attach(result, Location::caller())
}

/// Borrows a completed result for type-directed expression attachment.
pub struct AttachExpression<'a, T>(&'a mut T);

impl<'a, T> AttachExpression<'a, T> {
    /// Wraps a result without changing its type or taking ownership of it.
    #[must_use]
    pub fn new(result: &'a mut T) -> Self {
        Self(result)
    }
}

impl AttachExpression<'_, AssertionFailures> {
    /// Attaches the expression only to failures originating at this fluent entry.
    pub fn attach(self, expression: &'static str, location: &'static Location<'static>) {
        self.0.attach_expression(expression, location);
    }
}

/// Fallback for results from unrelated methods with fluent entry names.
pub trait AttachExpressionFallback {
    /// Leaves an unrelated result unchanged.
    fn attach(self, expression: &'static str, location: &'static Location<'static>);
}

impl<T> AttachExpressionFallback for AttachExpression<'_, T> {
    fn attach(self, _: &'static str, _: &'static Location<'static>) {}
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn assert_string_panic_contains(panic: std::thread::Result<()>, expected: &str) {
        let panic = assert_that_owned!(panic).get_err().unwrap_inner();
        let message = assert_that_owned!(panic.downcast::<String>().map_err(|_| ()))
            .get_ok()
            .unwrap_inner();
        assert_that!(message.as_str()).contains(expected);
    }

    #[test]
    fn plain_fluent_entry_points_do_not_invent_expressions() {
        let failures = 42.verify(|it| it.is_equal_to(43));
        assert_that!(failures[0].expression).is_none();

        let failures = 42.verify_owned(|it| it.is_equal_to(43));
        assert_that!(failures[0].expression).is_none();
    }

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
    #[test]
    fn fluent_attribute_attaches_only_pending_root_expressions() {
        struct NoRenderer;

        fn check(root: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i64, Capture, NoRenderer> {
            let root = root.is_equal_to(43);
            root.derive_owned(|value| value + 1).is_equal_to(45);
            root.derive_owned(|value| value + 1)
                .with_expression("child override")
                .is_equal_to(45);

            let mapped = root
                .with_renderer(NoRenderer)
                .map(|actual| crate::actual::Actual::Owned(i64::from(*actual.borrowed())))
                .with_renderer(DebugRenderer)
                .is_equal_to(43);
            mapped
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

    #[crate::fluent_expressions]
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

        #[crate::fluent_expressions]
        fn run() {
            let failures = User(21).verify(|value| value * 2);
            assert_that!(failures[0].expression).is_none();
            let failures = User(21).verify_owned(|value| value * 2);
            assert_that!(failures[0].expression).is_none();
        }

        run();
    }

    /// Pins the documented limitation: the attribute identifies failures by their entry location,
    /// so a tracked user-defined `verify` that forwards its call site to an inner verification
    /// lets those failures receive the outer receiver expression.
    #[test]
    fn fluent_attribute_attaches_to_tracked_verification_at_the_same_call_site() {
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

        #[crate::fluent_expressions]
        fn run() {
            let failures = User(21).verify(|value| value * 2);
            assert_that!(failures[0].expression).is_equal_to(Some("User(21)"));
            let offset = Box::new(21);
            let failures = User(21).verify_owned(move |value| value + *offset);
            assert_that!(failures[0].expression).is_equal_to(Some("User(21)"));
        }

        run();
    }

    #[test]
    fn fluent_attribute_attaches_expressions_inside_tracked_functions() {
        #[track_caller]
        #[crate::fluent_expressions]
        fn check(actual: i32) -> [AssertionFailures; 2] {
            let borrowed = actual.verify(|it| it.is_equal_to(43));
            let owned = actual.verify_owned(|it| it.is_equal_to(43));
            [borrowed, owned]
        }

        for failures in check(42) {
            assert_that!(failures[0].expression).is_equal_to(Some("actual"));
        }
    }

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
    mod macro_callbacks {
        use super::*;

        macro_rules! checks {
            () => {
                |it| it.is_equal_to(1)
            };
        }

        #[test]
        fn capture_the_receiver_expression() {
            let failures = 2.verify(checks!());
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].expression).is_equal_to(Some("2"));

            let failures = 2.verify_owned(checks!());
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].expression).is_equal_to(Some("2"));
        }

        #[test]
        fn preserve_explicit_expressions() {
            macro_rules! explicit_checks {
                () => {
                    |it| it.with_expression("explicit").is_equal_to(1)
                };
            }

            let borrowed = 2.verify(explicit_checks!());
            let owned = 2.verify_owned(explicit_checks!());
            for failures in [borrowed, owned] {
                assert_that!(failures).has_length(1);
                assert_that!(failures[0].expression).is_equal_to(Some("explicit"));
            }
        }

        #[test]
        fn preserve_evaluation_order_and_evaluate_callbacks_once() {
            use core::cell::RefCell;

            fn receiver(events: &RefCell<Vec<&'static str>>) -> i32 {
                events.borrow_mut().push("receiver");
                2
            }

            macro_rules! recorded_checks {
                ($events:ident) => {{
                    $events.borrow_mut().push("callback");
                    |it| {
                        $events.borrow_mut().push("call");
                        it.is_equal_to(1)
                    }
                }};
            }

            let events = RefCell::new(Vec::new());
            let borrowed = receiver(&events).verify(recorded_checks!(events));
            let owned = receiver(&events).verify_owned(recorded_checks!(events));
            for failures in [borrowed, owned] {
                assert_that!(failures).has_length(1);
                assert_that!(failures[0].expression).is_equal_to(Some("receiver(&events)"));
            }
            assert_that!(events.into_inner()).contains_exactly([
                "receiver", "callback", "call", "receiver", "callback", "call",
            ]);
        }

        #[test]
        fn capture_nested_entries_independently() {
            let failures = 2.verify({
                let nested = 3.verify(|it| it.is_equal_to(1));
                assert_that!(nested[0].expression).is_equal_to(Some("3"));
                checks!()
            });
            assert_that!(failures[0].expression).is_equal_to(Some("2"));

            let failures = 2.verify_owned({
                3.must().is_equal_to(3);
                |it| it.is_equal_to(1)
            });
            assert_that!(failures[0].expression).is_equal_to(Some("2"));
        }
    }

    #[crate::fluent_expressions]
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
        assert_that!(failures[0].expression).is_equal_to(Some("42"));
        let failures = 42.verify_owned(|it| fail_at_caller!(it, expected));
        assert_that!(failures[0].location).is_equal_to(expected);
        assert_that!(failures[0].expression).is_equal_to(Some("42"));
    }

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
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

    #[crate::fluent_expressions]
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
}
