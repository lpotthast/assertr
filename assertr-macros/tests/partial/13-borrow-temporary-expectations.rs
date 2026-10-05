use renamed_assertr::matchers::eq;
use renamed_assertr::{matchers::string::StartsWith, prelude::*};

struct Named<'a> {
    text: &'a str,
    owned: String,
}

struct Tuple<'a>(&'a str, String);

struct Nested<'a> {
    inner: Named<'a>,
}

fn main() {
    // Keep the temporary Strings inside each assertion. Their borrowed operands must live
    // through the whole statement, including when passed to matchers or nested `partial!` calls.
    assert_that!(Named {
        text: "hello",
        owned: String::from("world"),
    })
    .matches(partial!(Named {
        text: StartsWith::new(String::from("he").as_str()),
        owned: eq(&String::from("world")),
    }));

    assert_that!(Tuple("hello", String::from("world"))).matches(partial!(Tuple(
        StartsWith::new(String::from("he").as_str()),
        eq(&String::from("world")),
    )));

    assert_that!(Nested {
        inner: Named {
            text: "hello",
            owned: String::from("world"),
        },
    })
    .matches(partial!(Nested {
        inner: partial!(Named {
            text: StartsWith::new(String::from("he").as_str()),
            owned: eq(&String::from("world")),
        }),
    }));
}
