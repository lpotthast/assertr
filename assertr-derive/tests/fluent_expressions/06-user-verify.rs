// Intentionally no assertr prelude: the attribute must not redirect this inherent method.
struct User(i32);

fn increment(value: i32) -> i32 {
    value + 1
}

impl User {
    fn verify(self, operation: impl FnOnce(i32) -> i32) -> i32 {
        operation(self.0)
    }
}

fn assert_is_42(actual: i32) {
    use renamed_assertr::prelude::*;

    assert_that!(actual).is_equal_to(42);
}

#[renamed_assertr::fluent_expressions]
fn main() {
    let result: i32 = User(41).verify(|value: i32| value + 1);
    assert_is_42(result);

    let result: i32 = User(41).verify(increment);
    assert_is_42(result);
}
