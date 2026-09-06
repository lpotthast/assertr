use renamed_assertr::{matchers::anything, prelude::*};

struct NoRenderer;

struct User {
    age: u32,
}

fn main() {
    // `anything()` needs no rendering, whether used directly or inside `partial!`.
    assert_that!(())
        .with_renderer(NoRenderer)
        .matches(anything());
    assert_that!(User { age: 30 })
        .with_renderer(NoRenderer)
        .matches(partial!(User { age: anything() }));
}
