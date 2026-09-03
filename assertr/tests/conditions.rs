use assertr::prelude::*;
use indoc::formatdoc;

struct Person {
    name: &'static str,
    meta: Metadata,
}

struct Metadata {
    alive: bool,
}

struct IsAlive {}
impl AssertrCondition<Person> for IsAlive {
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
impl AssertrCondition<Person> for HasName {
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
impl AssertrCondition<Person> for HasNotName {
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

#[test]
fn conditions_are_reusable_when_passed_by_reference() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: true },
    };
    let kevin = Person {
        name: "Kevin",
        meta: Metadata { alive: true },
    };
    let alive = IsAlive {};

    assert_that!(bob).is(&alive);
    assert_that!(kevin).is(&alive);
    assert_that!(vec![bob, kevin]).are(&alive);

    // The condition was never consumed and can finally be passed by value.
    let otto = Person {
        name: "Otto",
        meta: Metadata { alive: true },
    };
    assert_that!(otto).is(alive);
}

#[test]
fn a_failing_condition_exposes_its_error_as_a_failure_detail() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: false },
    };

    let failures = assert_that!(bob)
        .with_location(false)
        .capture(|it| it.is(IsAlive {}));

    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].description.as_str()).is_equal_to("Condition did not match.\n");
    // The condition's error arrives verbatim as a per-failure detail; no parsing of the
    // description's framing text is required.
    assert_that!(failures[0].details.as_slice()).contains_exactly(["\"Bob\" is dead!"]);
}

#[test]
fn each_failing_element_raises_its_own_failure_without_inventing_an_index() {
    let people = vec![
        Person {
            name: "Bob",
            meta: Metadata { alive: true },
        },
        Person {
            name: "Kevin",
            meta: Metadata { alive: false },
        },
        Person {
            name: "Otto",
            meta: Metadata { alive: false },
        },
    ];

    let failures = assert_that!(people)
        .with_location(false)
        .capture(|it| it.are(IsAlive {}));

    assert_that!(&failures).has_length(2);
    assert_that!(failures[0].description.as_str())
        .is_equal_to("Condition did not match for an element.\n");
    assert_that!(failures[0].details.as_slice()).contains_exactly(["\"Kevin\" is dead!"]);
    assert_that!(failures[1].description.as_str())
        .is_equal_to("Condition did not match for an element.\n");
    assert_that!(failures[1].details.as_slice()).contains_exactly(["\"Otto\" is dead!"]);
}

#[test]
fn a_condition_failure_renders_the_error_under_details() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: false },
    };

    let failures = assert_that!(bob)
        .with_location(false)
        .capture(|it| it.is(IsAlive {}));

    assert_that!(failures[0].to_string()).is_equal_to(formatdoc! {r#"
        -------- assertr --------
        Expression: `bob`

        Condition did not match.

        Details:
          - "Bob" is dead!
        -------- assertr --------
    "#});
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_chains_use_be_for_values_and_have_for_iterables() {
    let bob = Person {
        name: "Bob",
        meta: Metadata { alive: true },
    };
    bob.must().be(IsAlive {});

    let people = vec![
        Person {
            name: "Bob",
            meta: Metadata { alive: true },
        },
        Person {
            name: "Kevin",
            meta: Metadata { alive: true },
        },
    ];
    people.must().have(HasNotName { unexpected: "Otto" });
}
