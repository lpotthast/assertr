use renamed_assertr::{matchers::starts_with, prelude::*};

struct Named<'a> {
    text: &'a str,
    owned: String,
}

struct Tuple<'a>(&'a str, String);

struct Nested<'a> {
    inner: Named<'a>,
}

fn main() {
    // Keep the temporary Strings inside each assertion. Their borrowed slices must live
    // through the whole statement, including when passed to matchers or nested `partial!` calls.
    assert_that!(Named {
        text: "hello",
        owned: String::from("world"),
    })
    .matches(partial!(Named {
        text: starts_with(String::from("he").as_str()),
        owned: String::from("world").as_str(),
    }));

    assert_that!(Tuple("hello", String::from("world"))).matches(partial!(Tuple(
        starts_with(String::from("he").as_str()),
        String::from("world").as_str(),
    )));

    assert_that!(Nested {
        inner: Named {
            text: "hello",
            owned: String::from("world"),
        },
    })
    .matches(partial!(Nested {
        inner: partial!(Named {
            text: starts_with(String::from("he").as_str()),
            owned: String::from("world").as_str(),
        }),
    }));
}
