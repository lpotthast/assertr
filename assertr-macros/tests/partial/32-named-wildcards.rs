use renamed_assertr::{matchers::eq, prelude::*};

struct NotDebug;

struct User {
    name: &'static str,
    session: NotDebug,
}

enum Event {
    Login { user: &'static str, session: NotDebug },
}

fn main() {
    // `_` lists a field without checking it. Unlike `..`, the pattern stays exhaustive.
    assert_that!(User {
        name: "Ada",
        session: NotDebug,
    })
    .matches(partial!(User {
        name: eq("Ada"),
        session: _,
    }));
    assert_that!(Event::Login {
        user: "Ada",
        session: NotDebug,
    })
    .matches(partial!(Event::Login {
        user: eq("Ada"),
        session: _
    }));
}
