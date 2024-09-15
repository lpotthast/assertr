use assertr::prelude::*;

struct Person {
    name: &'static str,
    meta: Metadata,
}

struct Metadata {
    alive: bool,
}

struct IsAlive {}
impl Condition<Person> for IsAlive {
    type Error = String;
    fn test<'a>(&self, value: &Person) -> Result<(), Self::Error> {
        match value.meta.alive {
            true => Ok(()),
            false => Err(format!("{:#?} is dead!", value.name)),
        }
    }
}

struct HasName {
    expected: &'static str,
}
impl Condition<Person> for HasName {
    type Error = String;
    fn test<'a>(&self, value: &Person) -> Result<(), Self::Error> {
        match value.name == self.expected {
            true => Ok(()),
            false => Err(format!(
                "Expected name {:#?}, but Person has unexpected name {:#?}!",
                self.expected, value.name
            )),
        }
    }
}

struct HasNotName {
    unexpected: &'static str,
}
impl Condition<Person> for HasNotName {
    type Error = String;
    fn test<'a>(&self, value: &Person) -> Result<(), Self::Error> {
        match value.name != self.unexpected {
            true => Ok(()),
            false => Err(format!("Person has unexpected name {:#?}!", value.name)),
        }
    }
}

#[test]
fn is_able_to_use_custom_conditions_using_is_and_has() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: true },
    };
    let alive = IsAlive {};
    let name_bob = HasName { expected: "Bob" };
    assert_that(bob).is(alive).has(name_bob);
}

#[test]
fn is_able_to_use_custom_conditions_on_an_iterable_using_are_and_have() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: true },
    };
    let kevin = Person {
        name: "Kevin",
        meta: Metadata { alive: false },
    };
    let people = vec![bob, kevin];
    let alive = IsAlive {};
    let not_name_otto = HasNotName { unexpected: "Otto" };
    assert_that(people).are(alive).have(not_name_otto);
}
