use renamed_assertr::{matchers::equal_to, prelude::*};

struct Record<'a, T, const N: usize> {
    value: T,
    label: &'a str,
    bytes: [u8; N],
}

fn check_record<T: PartialEq<T> + std::fmt::Debug>(record: &Record<'_, T, 1>, expected: T) {
    // `T` could also implement the matcher trait. Explicit equality removes that ambiguity.
    assert_that!(record).matches(partial!(Record::<T, 1> {
        value: equal_to(expected),
        label: "example",
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
