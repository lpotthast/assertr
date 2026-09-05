#![cfg(feature = "std")]

use core::{cell::Cell, convert::Infallible};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

use assertr::failure::adapter::{Adapter, AdapterExt, HumanReadableText, ToHumanReadableText};
use assertr::prelude::*;

const DEFAULT_MESSAGE: &str = "-------- assertr --------\nExpression: `1`\n\nExpected: 2\n\n  Actual: 1\n-------- assertr --------\n";

struct KindAdapter;

impl Adapter<AssertionFailure> for KindAdapter {
    type Output = HumanReadableText;
    type Error = Infallible;

    fn adapt(&self, failure: &AssertionFailure) -> Result<HumanReadableText, Infallible> {
        Ok(HumanReadableText::new(format!(
            "custom adapter: {:?}",
            failure.kind
        )))
    }
}

fn panic_text(action: impl FnOnce()) -> String {
    *catch_unwind(AssertUnwindSafe(action))
        .expect_err("the assertion should panic")
        .downcast::<String>()
        .expect("the panic payload should remain a String")
}

#[test]
fn a_context_presentation_produces_the_panic_payload() {
    let message = panic_text(|| {
        assert_that!(1)
            .with_panic_presentation(KindAdapter)
            .is_equal_to(2);
    });
    assert_eq!(message, "custom adapter: Equality");
}

#[test]
fn an_adapter_with_string_errors_can_be_used_directly_for_presentation() {
    struct Presentation;

    impl Adapter<AssertionFailure> for Presentation {
        type Output = HumanReadableText;
        type Error = String;

        fn adapt(&self, failure: &AssertionFailure) -> Result<HumanReadableText, String> {
            Ok(ToHumanReadableText.render(failure))
        }
    }

    let message = panic_text(|| {
        assert_that!(1)
            .with_location(false)
            .with_panic_presentation(Presentation)
            .is_equal_to(2);
    });
    assert_eq!(message, DEFAULT_MESSAGE);
}

struct AddContext(String);

impl Adapter<HumanReadableText> for AddContext {
    type Output = HumanReadableText;
    type Error = Infallible;

    fn adapt(&self, text: &HumanReadableText) -> Result<HumanReadableText, Infallible> {
        Ok(HumanReadableText::new(format!("{}\n{text}", self.0)))
    }
}

#[test]
fn an_adapter_chain_can_own_a_copy_of_local_context() {
    let context = String::from("Integration check failed:");
    let adapter = ToHumanReadableText.then(AddContext(context.clone()));
    let message = panic_text(|| {
        assert_that!(1)
            .with_location(false)
            .with_panic_presentation(adapter)
            .is_equal_to(2);
    });
    assert_eq!(message, format!("{context}\n{DEFAULT_MESSAGE}"));
}

struct CountPresentations(Rc<Cell<usize>>);

impl Adapter<AssertionFailure> for CountPresentations {
    type Output = HumanReadableText;
    type Error = Infallible;

    fn adapt(&self, failure: &AssertionFailure) -> Result<HumanReadableText, Infallible> {
        self.0.set(self.0.get() + 1);
        ToHumanReadableText.adapt(failure)
    }
}

#[test]
fn an_owned_presentation_does_not_extend_the_subject_borrow_until_drop() {
    let count = Rc::new(Cell::new(0));
    let mut values = vec![1];
    let assertion =
        assert_that!(values).with_panic_presentation(CountPresentations(Rc::clone(&count)));
    let first = assertion.get_first();
    assert_eq!(first.actual(), &1);

    // Both contexts remain in scope, including the owned presentation's destructor.
    values.push(2);
    assert_eq!(values, [1, 2]);
    assert_eq!(count.get(), 0);
}

#[test]
fn a_non_clone_presentation_is_shared_with_derived_assertions() {
    let count = Rc::new(Cell::new(0));
    let assertion = assert_that_owned!(1)
        .with_location(false)
        .with_panic_presentation(CountPresentations(Rc::clone(&count)));

    let child_message = panic_text(|| {
        assertion.derive_owned(|value| *value).is_equal_to(2);
    });
    assert!(child_message.contains("Expected: 2\n\n  Actual: 1"));
    assert_eq!(count.get(), 1);

    let parent_message = panic_text(|| {
        assertion.is_equal_to(2);
    });
    assert_eq!(parent_message, DEFAULT_MESSAGE);
    assert_eq!(count.get(), 2);
    // All contexts have dropped, releasing the presentation's shared state.
    assert_eq!(Rc::strong_count(&count), 1);
}

#[test]
fn capture_and_success_do_not_invoke_a_non_sync_presentation() {
    let count = Rc::new(Cell::new(0));
    let adapter = CountPresentations(Rc::clone(&count));
    assert_that!(1)
        .with_panic_presentation(CountPresentations(Rc::clone(&count)))
        .is_equal_to(1);
    let failures = assert_that!(1)
        .with_location(false)
        .with_panic_presentation(CountPresentations(Rc::clone(&count)))
        .capture(|it| it.is_equal_to(2));

    assert_eq!(count.get(), 0);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        adapter.adapt(&failures[0]).unwrap().as_str(),
        DEFAULT_MESSAGE
    );
    assert_eq!(count.get(), 1);
}

#[test]
fn presentation_is_inherited_by_projections_and_renderer_changes() {
    let message = panic_text(|| {
        assert_that_owned!(1)
            .with_panic_presentation(KindAdapter)
            .with_renderer(assertr::DebugRenderer)
            .map_owned(|_| 2)
            .is_equal_to(3);
    });
    assert_eq!(message, "custom adapter: Equality");
}

#[test]
fn presentation_is_inherited_by_derived_assertions() {
    let message = panic_text(|| {
        assert_that_owned!(1)
            .with_panic_presentation(KindAdapter)
            .satisfies_owned(
                |_| 2,
                |derived| {
                    derived.is_equal_to(3);
                },
            );
    });
    assert_eq!(message, "custom adapter: Equality");
}

#[test]
fn a_child_can_override_presentation_without_changing_its_parent() {
    let assertion = assert_that_owned!(1).with_panic_presentation(KindAdapter);
    let child_message = panic_text(|| {
        assertion
            .derive_owned(|value| *value)
            .with_location(false)
            .with_panic_presentation(ToHumanReadableText)
            .is_equal_to(2);
    });
    assert!(child_message.contains("Expected: 2\n\n  Actual: 1"));
    let parent_message = panic_text(|| {
        assertion.is_equal_to(2);
    });
    assert_eq!(parent_message, "custom adapter: Equality");
}

#[test]
fn presentation_configuration_does_not_require_a_value_renderer() {
    struct NoRenderer;
    let _assertion = assert_that!(1)
        .with_renderer(NoRenderer)
        .with_panic_presentation(ToHumanReadableText);
}

struct ReturnsError;

impl Adapter<AssertionFailure> for ReturnsError {
    type Output = HumanReadableText;
    type Error = &'static str;

    fn adapt(&self, _: &AssertionFailure) -> Result<HumanReadableText, &'static str> {
        Err("presentation unavailable")
    }
}

#[test]
fn a_presentation_error_preserves_the_failure_and_adds_a_diagnostic() {
    let message = panic_text(|| {
        assert_that!(1)
            .with_location(false)
            .with_panic_presentation(ReturnsError)
            .is_equal_to(2);
    });
    assert_eq!(
        message,
        format!(
            "{DEFAULT_MESSAGE}\n-------- assertr presentation diagnostic --------\nThe failure presentation returned an error: presentation unavailable\n------ end assertr presentation diagnostic ------\n"
        )
    );
}

#[test]
fn a_panicking_error_formatter_preserves_the_original_failure() {
    struct FormattingPanic;

    impl core::fmt::Display for FormattingPanic {
        fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            panic!("error formatting exploded")
        }
    }

    struct ReturnsUnformattableError;

    impl Adapter<AssertionFailure> for ReturnsUnformattableError {
        type Output = HumanReadableText;
        type Error = FormattingPanic;

        fn adapt(&self, _: &AssertionFailure) -> Result<HumanReadableText, FormattingPanic> {
            Err(FormattingPanic)
        }
    }

    let message = panic_text(|| {
        assert_that!(1)
            .with_location(false)
            .with_panic_presentation(ReturnsUnformattableError)
            .is_equal_to(2);
    });
    assert_eq!(
        message,
        format!(
            "{DEFAULT_MESSAGE}\n-------- assertr presentation diagnostic --------\nThe failure presentation panicked: error formatting exploded\n------ end assertr presentation diagnostic ------\n"
        )
    );
}

struct Panics(bool);

impl Adapter<AssertionFailure> for Panics {
    type Output = HumanReadableText;
    type Error = Infallible;

    fn adapt(&self, _: &AssertionFailure) -> Result<HumanReadableText, Infallible> {
        if self.0 {
            std::panic::panic_any(7_u8);
        }
        panic!("presentation exploded")
    }
}

#[test]
fn a_presentation_panic_preserves_the_failure_and_adds_a_diagnostic() {
    for (opaque, detail) in [
        (false, "presentation exploded"),
        (true, "non-string panic payload"),
    ] {
        let message = panic_text(|| {
            assert_that!(1)
                .with_location(false)
                .with_panic_presentation(Panics(opaque))
                .is_equal_to(2);
        });
        assert_eq!(
            message,
            format!(
                "{DEFAULT_MESSAGE}\n-------- assertr presentation diagnostic --------\nThe failure presentation panicked: {detail}\n------ end assertr presentation diagnostic ------\n"
            )
        );
    }
}

#[test]
fn default_presentation_neither_logs_nor_blocks_on_stdout() {
    const CHILD: &str = "ASSERTR_TEST_DEFAULT_PRESENTATION_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let stdout = std::io::stdout().lock();
        let message = panic_text(|| {
            assert_that!(1).with_location(false).is_equal_to(2);
        });
        drop(stdout);
        assert_eq!(message, DEFAULT_MESSAGE);
        return;
    }

    // Isolate the lock regression so a broken default cannot hang the entire test suite.
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "default_presentation_neither_logs_nor_blocks_on_stdout",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("default panic presentation blocked while stdout was locked");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("-------- assertr"));
}

struct TextLength;

impl Adapter<HumanReadableText> for TextLength {
    type Output = usize;
    type Error = Infallible;

    fn adapt(&self, text: &HumanReadableText) -> Result<usize, Infallible> {
        Ok(text.len())
    }
}

struct RecordLength<'a>(&'a Cell<Option<usize>>);

impl Adapter<usize> for RecordLength<'_> {
    type Output = ();
    type Error = Infallible;

    fn adapt(&self, length: &usize) -> Result<(), Infallible> {
        self.0.set(Some(*length));
        Ok(())
    }
}

#[test]
fn captured_failures_support_explicit_chains_with_arbitrary_outputs() {
    let failures = assert_that!(1)
        .with_location(false)
        .capture(|it| it.is_equal_to(2));
    let failure = &failures[0];
    let chain = ToHumanReadableText.then(TextLength);

    assert_eq!(chain.adapt(failure).unwrap(), DEFAULT_MESSAGE.len());

    let recorded = Cell::new(None);
    chain.then(RecordLength(&recorded)).adapt(failure).unwrap();
    assert_eq!(recorded.get(), Some(DEFAULT_MESSAGE.len()));
}
