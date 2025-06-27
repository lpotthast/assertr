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

    person
        .must()
        .map(|it| it.borrowed().meta.alive.into())
        .be_equal_to(true);

    (-1.23)
        .must()
        .map_owned(|it| it.to_string())
        .has_length(5)
        .be_equal_to("-1.23".to_owned());
}
