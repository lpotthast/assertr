// Import only equality assertions to preserve the inherent methods and their callback bounds.
use renamed_assertr::{assert_that, prelude::PartialEqAssertions};
struct User;
struct MutUser;
struct OnceUser;
struct PointerUser;
struct GenericUser;
struct NoArgumentUser;
struct TwoArgumentUser;
struct TokenUser;
struct UnconstrainedUser;

impl User {
    fn verify(self, _: impl Fn(i32) -> i32) {}

    fn verify_owned(self, _: impl Fn(i32) -> i32) {}
}

impl MutUser {
    fn verify(self, _: impl FnMut(i32) -> i32) {}

    fn verify_owned(self, _: impl FnMut(i32) -> i32) {}
}

impl OnceUser {
    fn verify(self, _: impl FnOnce(i32) -> i32) {}

    fn verify_owned(self, _: impl FnOnce(i32) -> i32) {}
}

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
fn main() {
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

    User.verify(double);
    User.verify_owned(double);

    let pointer: fn(i32) -> i32 = double;
    User.verify(pointer);
    User.verify_owned(pointer);

    let offset = Box::new(1);
    let callback = move |value| value + *offset;
    User.verify(&callback);
    User.verify_owned(callback);

    let mut total = 0;
    let mut callback = |value| {
        total += value;
        total
    };
    MutUser.verify(&mut callback);
    MutUser.verify_owned(callback);

    let owned = String::new();
    let callback = move |value| {
        drop(owned);
        value
    };
    OnceUser.verify(callback);

    let owned = String::new();
    let callback = move |value| {
        drop(owned);
        value
    };
    OnceUser.verify_owned(callback);
}
