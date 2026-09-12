use renamed_assertr::prelude::*;

struct Named {
    value: i32,
}

struct Tuple(i32);

fn main() {
    // Named and tuple fields both require explicit matchers such as `eq(1)`.
    assert_that!(Named { value: 1 }).matches(partial!(Named { value: 1 }));
    assert_that!(Tuple(1)).matches(partial!(Tuple(1)));
}
