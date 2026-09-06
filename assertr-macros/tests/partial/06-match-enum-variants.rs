use renamed_assertr::prelude::*;

enum Message {
    Named { value: i32 },
    Tuple(i32),
    Unit,
}

fn main() {
    assert_that!(Message::Named { value: 1 }).matches(partial!(Message::Named { value: 1 }));
    assert_that!(Message::Tuple(2)).matches(partial!(Message::Tuple(2)));
    assert_that!(Message::Unit).matches(partial!(Message::Unit));

    // The optional `variant` prefix also includes the variant in diagnostic paths.
    assert_that!(Some(3)).matches(partial!(variant Some(3)));
}
