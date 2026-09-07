// Intentionally no assertr prelude: preserve the inherent methods and their callback bounds.
struct User;
struct MutUser;
struct OnceUser;

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

fn double(value: i32) -> i32 {
    value * 2
}

#[renamed_assertr::fluent_expressions]
fn main() {
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
