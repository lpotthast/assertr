use assertr::prelude::*;

#[derive(Debug, PartialEq)]
struct Person {
    meta: Metadata,
}

#[derive(Debug, PartialEq)]
struct Metadata {
    alive: bool,
}

#[test]
fn is_able_to_access_derived_properties_without_breaking_the_call_chain() {
    let person = Person {
        meta: Metadata { alive: true },
    };

    assert_that(person)
        .map(|it| it.borrowed().meta.alive.into())
        .is_equal_to(true);

    assert_that(-3.14)
        .map_owned(|it| it.to_string())
        .has_length(5)
        .is_equal_to("-3.14".to_owned());
}
