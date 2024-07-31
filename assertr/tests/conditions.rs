use assertr::prelude::*;

use std::fmt::Arguments;

struct Person {
    meta: Metadata,
}

struct Metadata {
    alive: bool,
}

struct IsAlive {}
impl Condition<Person> for IsAlive {
    fn test<'a>(&self, value: &Person) -> Result<(), Arguments<'a>> {
        match value.meta.alive {
            true => Ok(()),
            false => Err(format_args!("Person is dead!")),
        }
    }
}

const ALIVE: IsAlive = IsAlive {};

#[test]
fn is_able_to_use_custom_alive_condition() {
    let person = Person {
        meta: Metadata { alive: true },
    };
    assert_that(person).is(ALIVE);
}
