#![cfg(feature = "std")]

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{
    panic::{AssertUnwindSafe, RefUnwindSafe, catch_unwind},
    rc::Rc,
};

use assertr::prelude::*;

const DEFAULT_MESSAGE: &str = "-------- assertr --------\nExpression: `1`\n\nExpected: 2\n\n  Actual: 1\n-------- assertr --------\n";

fn kind(failure: &AssertionFailure) -> String {
    format!("custom presentation: {:?}", failure.kind)
}

fn panic_text(action: impl FnOnce()) -> String {
    *catch_unwind(AssertUnwindSafe(action))
        .expect_err("the assertion should panic")
        .downcast::<String>()
        .expect("the panic payload should remain a String")
}

/// Counts its invocations and produces the default report. The closure is neither `Send`,
/// `Sync`, nor `Clone`.
fn counting(
    count: &Rc<AtomicUsize>,
) -> impl Fn(&AssertionFailure) -> String + RefUnwindSafe + 'static {
    let count = Rc::clone(count);
    move |failure| {
        count.fetch_add(1, Ordering::Relaxed);
        failure.to_string()
    }
}

mod presentation {
    use super::*;

    #[test]
    fn default_presentation_neither_logs_nor_blocks_on_stdout() {
        const CHILD: &str = "ASSERTR_TEST_DEFAULT_PRESENTATION_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let stdout = std::io::stdout().lock();
            let message = panic_text(|| {
                assert_that!(1).with_location(false).is_equal_to(2);
            });
            drop(stdout);
            assert_that!(message).is_equal_to(DEFAULT_MESSAGE);
            return;
        }

        // Isolate the lock regression so a broken default cannot hang the entire test suite.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "presentation::default_presentation_neither_logs_nor_blocks_on_stdout",
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
        assert_that!(output.status.success())
            .with_detail_message(String::from_utf8_lossy(&output.stderr))
            .is_true();
        assert_that!(String::from_utf8_lossy(&output.stdout)).does_not_contain("-------- assertr");
    }
}

mod inheritance {
    use super::*;

    #[test]
    fn presentation_is_inherited_by_projections_and_renderer_changes() {
        let message = panic_text(|| {
            assert_that_owned!(1)
                .with_panic_presentation(kind)
                .with_renderer(DebugRenderer)
                .map_owned(|_| 2)
                .is_equal_to(3);
        });
        assert_that!(message).is_equal_to("custom presentation: Equality");
    }

    #[test]
    fn presentation_is_inherited_by_derived_assertions() {
        let message = panic_text(|| {
            assert_that_owned!(1)
                .with_panic_presentation(kind)
                .satisfies_owned(
                    |_| 2,
                    |derived| {
                        derived.is_equal_to(3);
                    },
                );
        });
        assert_that!(message).is_equal_to("custom presentation: Equality");
    }

    #[test]
    fn a_child_can_override_presentation_without_changing_its_parent() {
        let assertion = assert_that_owned!(1).with_panic_presentation(kind);
        let child_message = panic_text(|| {
            assertion
                .derive_owned(|value| *value)
                .with_location(false)
                .with_panic_presentation(ToString::to_string)
                .is_equal_to(2);
        });
        assert_that!(child_message).contains("Expected: 2\n\n  Actual: 1");
        let parent_message = panic_text(|| {
            assertion.is_equal_to(2);
        });
        assert_that!(parent_message).is_equal_to("custom presentation: Equality");
    }

    #[test]
    fn a_non_clone_presentation_is_shared_with_derived_assertions() {
        let count = Rc::new(AtomicUsize::new(0));
        let assertion = assert_that_owned!(1)
            .with_location(false)
            .with_panic_presentation(counting(&count));

        let child_message = panic_text(|| {
            assertion.derive_owned(|value| *value).is_equal_to(2);
        });
        assert_that!(child_message).contains("Expected: 2\n\n  Actual: 1");
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(1);

        let parent_message = panic_text(|| {
            assertion.is_equal_to(2);
        });
        assert_that!(parent_message).is_equal_to(DEFAULT_MESSAGE);
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(2);
        // All contexts have dropped, releasing the presentation's shared state.
        assert_that!(Rc::strong_count(&count)).is_equal_to(1);
    }
}

mod ownership {
    use super::*;

    #[test]
    fn an_erased_presentation_preserves_context_unwind_safety() {
        let count = Rc::new(AtomicUsize::new(0));
        let context = assert_that!(1).with_panic_presentation(counting(&count));
        assert_that!(
            catch_unwind(|| {
                context.derive(|value| value).is_equal_to(2);
            })
            .is_err()
        )
        .is_true();
        assert_that!(catch_unwind(move || context.is_equal_to(3)).is_err()).is_true();
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(2);
    }

    #[test]
    fn an_owned_presentation_does_not_extend_the_subject_borrow_until_drop() {
        let count = Rc::new(AtomicUsize::new(0));
        let mut values = vec![1];
        let assertion = assert_that!(values).with_panic_presentation(counting(&count));
        let first = assertion.get_first();
        assert_that!(first.actual()).is_equal_to(1);

        // Both contexts remain in scope, including the owned presentation's destructor.
        values.push(2);
        assert_that!(values).contains_exactly([1, 2]);
        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(0);
    }
}

mod fallback {
    use super::*;

    #[test]
    fn a_presentation_panic_preserves_the_failure_and_adds_a_diagnostic() {
        for (opaque, detail) in [
            (false, "presentation exploded"),
            (true, "non-string panic payload"),
        ] {
            let message = panic_text(|| {
                assert_that!(1)
                    .with_location(false)
                    .with_panic_presentation(move |_| {
                        if opaque {
                            std::panic::panic_any(7_u8);
                        }
                        panic!("presentation exploded")
                    })
                    .is_equal_to(2);
            });
            assert_that!(message).is_equal_to(format!(
                "{DEFAULT_MESSAGE}\n-------- assertr presentation diagnostic --------\nThe failure presentation panicked: {detail}\n------ end assertr presentation diagnostic ------\n"
            ));
        }
    }
}

mod capture {
    use super::*;

    #[test]
    fn capture_and_success_do_not_invoke_the_presentation() {
        let count = Rc::new(AtomicUsize::new(0));
        assert_that!(1)
            .with_panic_presentation(counting(&count))
            .is_equal_to(1);
        let failures = assert_that!([1, 2])
            .with_panic_presentation(counting(&count))
            .capture(|it| {
                it.matches(elements_are![
                    matchers::eq(1),
                    matchers::predicate(|x: &i32| *x == 3)
                ])
            });

        assert_that!(count.load(Ordering::Relaxed)).is_equal_to(0);
        let child = &failures[0].children[0];
        assert_that!(child.path).is_equal_to([assertr::failure::PathSegment::Index(1)]);
        assert_that!(child.constraint.as_ref().unwrap().relation.as_deref())
            .is_equal_to(Some("satisfies the predicate"));
        assert_that!(failures[0].to_string()).contains("At [1]:");
    }
}
