use assertr::prelude::*;

#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
    meta: Metadata,
}

#[derive(Debug, PartialEq)]
struct Metadata {
    alive: bool,
}

#[test]
fn is_able_to_access_derived_properties_without_breaking_the_call_chain() {
    let person = Person {
        age: 30,
        meta: Metadata { alive: true },
    };

    person
        .must()
        .be_equal_to(Person {
            age: 30,
            meta: Metadata { alive: true },
        })
        .satisfy(
            |it| it.age,
            |age| {
                age.is_greater_than(18);
            },
        )
        .derive(|it| it.meta.alive)
        .be_equal_to(true);
}
