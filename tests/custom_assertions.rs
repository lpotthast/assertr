#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
    meta: Metadata,
}

#[derive(Debug, PartialEq)]
struct Metadata {
    alive: bool,
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;

    use crate::Metadata;
    use crate::Person;

    trait PersonAssertions {
        fn has_age(self, expected: u32) -> Self;
        fn is_alive(self) -> Self;
    }

    impl<'t> PersonAssertions for AssertThat<'t, Person> {
        fn has_age(self, expected: u32) -> Self {
            assert_that(self.actual().age).is_equal_to(expected);
            self
        }

        fn is_alive(self) -> Self {
            assert_that(self.actual().meta.alive).is_true();
            self
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
}
