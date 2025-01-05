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

trait PersonAssertions {
    fn has_age(self, expected: u32) -> Self;
    #[allow(clippy::wrong_self_convention)]
    fn is_alive(self) -> Self;
}

impl<M: Mode> PersonAssertions for AssertThat<'_, Person, M> {
    fn has_age(self, expected: u32) -> Self {
        self.satisfies(
            |p| p.age,
            |age| {
                age.is_equal_to(expected);
            },
        )
    }

    fn is_alive(self) -> Self {
        self.satisfies(
            |p| p.meta.alive,
            |alive| {
                alive.is_true();
            },
        )
    }
}

#[test]
fn is_able_to_use_custom_has_age_assertion() {
    let person = Person {
        age: 30,
        meta: Metadata { alive: true },
    };

    assert_that(person)
        .is_equal_to(Person {
            age: 30,
            meta: Metadata { alive: true },
        })
        .has_age(30)
        .is_alive();
}
