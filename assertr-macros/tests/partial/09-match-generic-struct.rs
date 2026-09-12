use renamed_assertr::matchers::eq;
use renamed_assertr::{matchers::equal_to, prelude::*};

struct Record<'a, T, const N: usize> {
    value: T,
    label: &'a str,
    bytes: [u8; N],
}

fn check_record<T: PartialEq<T> + std::fmt::Debug>(record: &Record<'_, T, 1>, expected: T) {
    // `equal_to` compares the generic expected value through its `PartialEq` implementation.
    assert_that!(record).matches(partial!(Record::<T, 1> {
        value: equal_to(expected),
        label: eq("example"),
        ..
    }));
}

fn main() {
    let label = String::from("example");
    let record = Record {
        value: 2,
        label: &label,
        bytes: [0],
    };

    check_record(&record, 2);
}
