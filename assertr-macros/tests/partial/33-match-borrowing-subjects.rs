use std::{rc::Rc, sync::Arc};

use renamed_assertr::{
    matchers::{each, eq},
    prelude::*,
};

struct User {
    name: &'static str,
}

struct Session<'a> {
    user: &'a User,
}

struct Wrapper<T> {
    value: T,
}

fn main() {
    let users = vec![User { name: "Alice" }];
    let matcher = partial!(User { name: eq("Alice") });

    // `Vec::first` returns a reference, which a partial matcher checks without `dereferenced`.
    assert_that!(users.first()).some().matches(&matcher);
    assert_that_owned!(&users[0]).matches(&matcher);
    assert_that!(Box::new(User { name: "Alice" })).matches(&matcher);
    assert_that!(Rc::new(User { name: "Alice" })).matches(&matcher);
    assert_that!(Arc::new(User { name: "Alice" })).matches(&matcher);
    assert_that!(users.iter().collect::<Vec<_>>()).matches(each(&matcher));

    // A nested matcher checks a reference-typed field.
    let session = Session { user: &users[0] };
    assert_that!(session).matches(partial!(Session {
        user: partial!(User { name: eq("Alice") }),
    }));

    // Generic parameters are still inferred from the subject.
    assert_that_owned!(&Wrapper { value: 3 }).matches(partial!(Wrapper { value: eq(3) }));
}
