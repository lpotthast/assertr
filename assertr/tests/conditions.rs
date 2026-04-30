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
    fn test(&self, value: &Person) -> Result<(), Self::Error> {
        if value.meta.alive {
            Ok(())
        } else {
            Err(format!("{:#?} is dead!", value.name))
        }
    }
}

struct HasName {
    expected: &'static str,
}
impl Condition<Person> for HasName {
    type Error = String;
    fn test(&self, value: &Person) -> Result<(), Self::Error> {
        if value.name == self.expected {
            Ok(())
        } else {
            Err(format!(
                "Expected name {:#?}, but Person has unexpected name {:#?}!",
                self.expected, value.name
            ))
        }
    }
}

struct HasNotName {
    unexpected: &'static str,
}
impl Condition<Person> for HasNotName {
    type Error = String;
    fn test(&self, value: &Person) -> Result<(), Self::Error> {
        if value.name == self.unexpected {
            Err(format!("Person has unexpected name {:#?}!", value.name))
        } else {
            Ok(())
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
    assert_that!(bob).is(alive).has(name_bob);
}

#[test]
fn is_able_to_use_custom_conditions_on_an_iterable_using_are_and_have() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: true },
    };
    let kevin = Person {
        name: "Kevin",
        meta: Metadata { alive: true },
    };
    let people = vec![bob, kevin];
    let alive = IsAlive {};
    let not_name_otto = HasNotName { unexpected: "Otto" };
    assert_that!(people).are(alive).have(not_name_otto);
}
