use renamed_assertr::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static CALLS: AtomicUsize = AtomicUsize::new(0);
type StaticCheck = fn(AssertThat<'static, i32, Capture>) -> AssertThat<'static, i32, Capture>;

fn first(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
    CALLS.fetch_add(1, Ordering::Relaxed);
    it.is_equal_to(0)
}

fn second(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
    CALLS.fetch_add(1, Ordering::Relaxed);
    it.is_equal_to(1)
}

fn check(failures: &AssertionFailures, before: usize) {
    assert_that!(failures).has_length(1);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(before + 1);
}

fn reusable_callback()
-> impl for<'a> Fn(AssertThat<'a, i32, Capture>) -> AssertThat<'a, i32, Capture> {
    let owned = String::from("still available");
    move |it| {
        assert_that!(owned).is_equal_to("still available");
        it.is_equal_to(0)
    }
}

#[renamed_assertr::fluent_expressions]
fn main() {
    for choose_first in [true, false] {
        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify(if choose_first { first } else { second });
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify_owned(if choose_first { first } else { second });
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify(match choose_first {
            true => first,
            false => second,
        });
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify_owned(match choose_first {
            true => first,
            false => second,
        });
        check(&failures, before);

        for nested in [true, false] {
            let before = CALLS.load(Ordering::Relaxed);
            let failures = 42.verify(if choose_first {
                match nested {
                    true => first,
                    false => second,
                }
            } else {
                second
            });
            check(&failures, before);

            let before = CALLS.load(Ordering::Relaxed);
            let failures = 42.verify_owned(match choose_first {
                true => {
                    if nested {
                        first
                    } else {
                        second
                    }
                }
                false => second,
            });
            check(&failures, before);
        }

        // Retaining a reference must not leave probes for its discarded tentative rewrite.
        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify(&(if choose_first { first } else { second }));
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify_owned(
            &(match choose_first {
                true => first,
                false => second,
            }),
        );
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify(if choose_first {
            first as StaticCheck
        } else {
            |it| second(it)
        });
        check(&failures, before);

        let before = CALLS.load(Ordering::Relaxed);
        let failures = 42.verify_owned(match choose_first {
            true => |it| first(it),
            false => second as StaticCheck,
        });
        check(&failures, before);
    }

    let mut order = Vec::new();
    let before = CALLS.load(Ordering::Relaxed);
    let failures = {
        order.push("receiver");
        42
    }
    .verify(
        if {
            order.push("condition");
            true
        } {
            order.push("chosen branch");
            first
        } else {
            order.push("unchosen branch");
            second
        },
    );
    assert_that!(failures).has_length(1);
    assert_that!(order).contains_exactly(["receiver", "condition", "chosen branch"]);
    assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(before + 1);

    // Branch probes borrow the callback exactly as the original expression did.
    let callback = reusable_callback();
    let failures = 42.verify(if true { &callback } else { &callback });
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
    let failures = 42.verify_owned(&callback);
    assert_that!(failures[0].expression).is_equal_to(Some("42"));
}
