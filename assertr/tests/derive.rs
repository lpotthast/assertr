use assertr::prelude::*;

#[test]
fn extracting_does_not_retain_the_subject_borrow_until_the_context_is_dropped() {
    let mut values = vec![1];
    let assertion = assert_that!(values);
    assertion.get_first().is_equal_to(1);

    // Keep `assertion` in scope. Its last use, rather than its drop, must end the borrow.
    values.push(2);
    assert_eq!(values, [1, 2]);
}

#[test]
fn deriving_does_not_retain_the_subject_borrow_until_the_context_is_dropped() {
    let mut values = vec![1];
    let assertion = assert_that!(values);
    assertion.derive_owned(Vec::len).is_equal_to(1);

    values.push(2);
    assert_eq!(values, [1, 2]);
}

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

    assert_that!(person)
        .is_equal_to(Person {
            age: 30,
            meta: Metadata { alive: true },
        })
        .satisfies(
            |it| &it.age,
            |age| {
                age.is_greater_than(18);
            },
        )
        .satisfies_owned(
            |it| it.age,
            |age| {
                age.is_greater_than(18);
            },
        )
        .derive(|it| &it.meta.alive)
        .is_equal_to(true);
}
