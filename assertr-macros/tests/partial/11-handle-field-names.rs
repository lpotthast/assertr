use renamed_assertr::{matchers::eq, prelude::*};

// Raw identifiers and names resembling macro internals must remain ordinary field names.
struct Fields {
    r#type: i32,
    projection: i32,
    path: i32,
    __assertr_expected_0: i32,
    __assertr_actual: i32,
    __assertr_value: i32,
}

fn main() {
    let fields = Fields {
        r#type: 1,
        projection: 2,
        path: 3,
        __assertr_expected_0: 4,
        __assertr_actual: 5,
        __assertr_value: 6,
    };

    assert_that!(fields).matches(partial!(Fields {
        r#type: eq(1),
        projection: eq(2),
        path: eq(3),
        __assertr_expected_0: eq(4),
        __assertr_actual: eq(5),
        __assertr_value: eq(6),
    }));
}
