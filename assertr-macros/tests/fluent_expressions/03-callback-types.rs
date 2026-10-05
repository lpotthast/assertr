// The attribute never rewrites callback arguments. Every callback keeps the type, call traits, and
// coercions that the selected `verify` method expects.
// Import only equality assertions to preserve the inherent methods and their callback bounds.
use renamed_assertr::{assert_that, prelude::PartialEqAssertions};

struct PointerUser;
struct GenericUser;
struct NoArgumentUser;
struct TwoArgumentUser;
struct TokenUser;
struct UnconstrainedUser;

impl PointerUser {
    fn verify(self, operation: fn(i32) -> i32) -> i32 {
        operation(21) + operation(21)
    }

    fn verify_owned(self, operation: fn(i32) -> i32) -> i32 {
        operation(21) + operation(21)
    }
}

impl GenericUser {
    fn verify<T>(self, operation: impl FnOnce(i32) -> T) -> T {
        operation(21)
    }

    fn verify_owned<T>(self, operation: impl FnOnce(i32) -> T) -> T {
        operation(21)
    }
}

impl NoArgumentUser {
    fn verify(self, operation: impl FnOnce() -> i32) -> i32 {
        operation()
    }
}

impl TwoArgumentUser {
    fn verify(self, operation: impl FnOnce(i32, i32) -> i32) -> i32 {
        operation(21, 21)
    }
}

impl TokenUser {
    fn verify(self, value: i32) -> i32 {
        value
    }
}

impl UnconstrainedUser {
    fn verify<A, B>(self, _: impl FnOnce(A) -> B) -> i32 {
        42
    }
}

fn double(value: i32) -> i32 {
    value * 2
}

fn make_pointer(creations: &mut usize) -> fn(i32) -> i32 {
    *creations += 1;
    double
}

#[renamed_assertr::fluent_expressions]
#[allow(unused_parens, unused_braces)]
fn assertion_callbacks() {
    use renamed_assertr::prelude::*;

    type Check = fn(AssertThat<'static, i32, Capture>) -> AssertThat<'static, i32, Capture>;

    macro_rules! checks {
        () => {
            |it| it.is_equal_to(1)
        };
    }

    fn first(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
        it.is_equal_to(1)
    }

    fn second(it: AssertThat<'_, i32, Capture>) -> AssertThat<'_, i32, Capture> {
        it.is_equal_to(3)
    }

    fn expect_capture(failures: AssertionFailures) {
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].expression).is_equal_to(Some("2"));
    }

    expect_capture(2.verify(|it| it.is_equal_to(1)));
    expect_capture(2.verify_owned(|it: AssertThat<'_, i32, Capture>| it.is_equal_to(1)));
    expect_capture(2.verify(first));
    expect_capture(2.verify_owned(first as Check));
    let pointer: Check = first;
    expect_capture(2.verify(pointer));
    expect_capture(2.verify_owned(pointer));

    for choose_first in [true, false] {
        expect_capture(2.verify(if choose_first { first } else { second }));
        expect_capture(2.verify_owned(match choose_first {
            true => first as Check,
            false => |it| second(it),
        }));
    }

    let callback = |it: AssertThat<'static, i32, Capture>| it.is_equal_to(1);
    expect_capture(2.verify(&callback));
    expect_capture(2.verify_owned(callback));
    let mut calls = 0;
    let mut callback = |it: AssertThat<'static, i32, Capture>| {
        calls += 1;
        it.is_equal_to(1)
    };
    expect_capture(2.verify(&mut callback));
    expect_capture(2.verify_owned(callback));
    assert_that!(calls).is_equal_to(2);

    expect_capture(2.verify(checks!()));
    expect_capture(2.verify_owned(checks!()));
    expect_capture(2.verify((checks!())));
    expect_capture(2.verify_owned({ checks!() }));
    expect_capture(2.verify(&(checks!() as Check)));
    expect_capture(2.verify_owned(&mut (checks!() as Check)));
}

struct Shared;
struct Mutable;
struct Once;

impl Shared {
    fn verify(self, operation: impl Fn(i32) -> i32) -> i32 {
        operation(21) + operation(21)
    }

    fn verify_owned(self, operation: impl Fn(i32) -> i32) -> i32 {
        self.verify(operation)
    }
}

impl Mutable {
    fn verify(self, mut operation: impl FnMut(i32) -> i32) -> i32 {
        operation(21) + operation(21)
    }

    fn verify_owned(self, operation: impl FnMut(i32) -> i32) -> i32 {
        self.verify(operation)
    }
}

impl Once {
    fn verify(self, operation: impl FnOnce(i32) -> i32) -> i32 {
        operation(21)
    }

    fn verify_owned(self, operation: impl FnOnce(i32) -> i32) -> i32 {
        self.verify(operation)
    }
}

#[renamed_assertr::fluent_expressions]
fn macro_callbacks() {
    macro_rules! double {
        () => {
            |value| value.saturating_mul(2)
        };
    }

    let result = Shared.verify(double!());
    assert_that!(result).is_equal_to(84);
    let result = Shared.verify_owned(double!());
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify(double!());
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(double!());
    assert_that!(result).is_equal_to(84);

    // A concrete function-pointer parameter provides enough context for unannotated branches.
    for choose_macro in [true, false] {
        let result = PointerUser.verify(if choose_macro {
            double!()
        } else {
            |value| value.saturating_mul(2)
        });
        assert_that!(result).is_equal_to(84);
        let result = PointerUser.verify_owned(match choose_macro {
            true => |value| value.saturating_mul(2),
            false => double!(),
        });
        assert_that!(result).is_equal_to(84);
    }

    macro_rules! accumulate {
        ($total:ident) => {
            |value| {
                $total += value.abs();
                $total
            }
        };
    }

    let mut total = 0;
    let result = Mutable.verify(accumulate!(total));
    assert_that!(result).is_equal_to(63);
    let result = Mutable.verify_owned(accumulate!(total));
    assert_that!(result).is_equal_to(147);
    assert_that!(total).is_equal_to(84);

    macro_rules! consume {
        ($owned:ident) => {
            move |value| {
                drop($owned);
                value.abs()
            }
        };
    }

    let owned = String::from("consumed");
    let result = Once.verify(consume!(owned));
    assert_that!(result).is_equal_to(21);
    let owned = String::from("consumed");
    let result = Once.verify_owned(consume!(owned));
    assert_that!(result).is_equal_to(21);
}

#[renamed_assertr::fluent_expressions]
fn unrelated_callbacks() {
    // Keep every rewritten call outside assert_that!, whose contents the attribute cannot visit.
    let result = PointerUser.verify(double);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(double);
    assert_that!(result).is_equal_to(84);

    let pointer: fn(i32) -> i32 = double;
    let result = PointerUser.verify(pointer);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(pointer);
    assert_that!(result).is_equal_to(84);

    let result = PointerUser.verify(double as fn(i32) -> i32);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(double as fn(i32) -> i32);
    assert_that!(result).is_equal_to(84);

    let result = PointerUser.verify(|value| value * 2);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(|value| value * 2);
    assert_that!(result).is_equal_to(84);
    let operation = |value| value * 2;
    let result = PointerUser.verify(operation);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(operation);
    assert_that!(result).is_equal_to(84);

    let mut block_creations = 0;
    let result = PointerUser.verify({
        block_creations += 1;
        |value| value * 2
    });
    assert_that!(block_creations).is_equal_to(1);
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify(if true { double } else { |value| value * 2 });
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(match false {
        true => double,
        false => |value| value * 2,
    });
    assert_that!(result).is_equal_to(84);

    let result = NoArgumentUser.verify(|| 42);
    assert_that!(result).is_equal_to(42);
    let callback = || 42;
    let result = NoArgumentUser.verify(callback);
    assert_that!(result).is_equal_to(42);
    let result = TwoArgumentUser.verify(|left, right| left + right);
    assert_that!(result).is_equal_to(42);
    let callback = |left, right| left + right;
    let result = TwoArgumentUser.verify(callback);
    assert_that!(result).is_equal_to(42);
    let result = TokenUser.verify(42);
    assert_that!(result).is_equal_to(42);
    let result = UnconstrainedUser.verify(|value: i32| value);
    assert_that!(result).is_equal_to(42);

    let mut creations = 0;
    let result = PointerUser.verify(make_pointer(&mut creations));
    assert_that!(result).is_equal_to(84);
    let result = PointerUser.verify_owned(make_pointer(&mut creations));
    assert_that!(result).is_equal_to(84);
    assert_that!(creations).is_equal_to(2);

    let result: i32 = GenericUser.verify(|_| Default::default());
    assert_that!(result).is_equal_to(0);
    let result: i32 = GenericUser.verify_owned(|_| Default::default());
    assert_that!(result).is_equal_to(0);
    let result = GenericUser.verify::<i32>(|_| Default::default());
    assert_that!(result).is_equal_to(0);

    let value = String::from("borrowed");
    let result: &str = GenericUser.verify(|_| value.as_str());
    assert_that!(result).is_equal_to("borrowed");
    let result: &str = GenericUser.verify_owned(|_| value.as_str());
    assert_that!(result).is_equal_to("borrowed");

    Shared.verify(double);
    Shared.verify_owned(double);

    let pointer: fn(i32) -> i32 = double;
    Shared.verify(pointer);
    Shared.verify_owned(pointer);

    let offset = Box::new(1);
    let callback = move |value| value + *offset;
    Shared.verify(&callback);
    Shared.verify_owned(callback);

    let mut total = 0;
    let mut callback = |value| {
        total += value;
        total
    };
    Mutable.verify(&mut callback);
    Mutable.verify_owned(callback);

    let owned = String::new();
    let callback = move |value| {
        drop(owned);
        value
    };
    Once.verify(callback);

    let owned = String::new();
    let callback = move |value| {
        drop(owned);
        value
    };
    Once.verify_owned(callback);
}

fn main() {
    assertion_callbacks();
    macro_callbacks();
    unrelated_callbacks();
}
