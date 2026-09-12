use assertr::failure::Fact;
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

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|item| item.relation.as_deref())
                .is_equal_to(Some("does not match the condition"));
            // The condition's error arrives as a typed rendered value as an unlabeled note of the
            // failure. No parsing of the description's framing text is required.
            element
                .derive_owned(|item| item.facts.as_slice())
                .contains_exactly([Fact::note(
                    assertr::renderer::RenderingContext::new(
                        &DebugRenderer,
                        RenderingBudget::default(),
                    )
                    .value(&String::from("\"Bob\" is dead!")),
                )]);
            element
                .derive_owned(|item| item.facts.as_slice())
                .contains_exactly([Fact::note(
                    assertr::renderer::RenderingContext::new(
                        &DebugRenderer,
                        RenderingBudget::default(),
                    )
                    .value(&String::from("\"Bob\" is dead!")),
                )]);
        },
    ]);
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

    assert_that!(&failures).contains_exactly_satisfying([
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|item| item.relation.as_deref())
                .is_equal_to(Some("does not match the condition"));
            element
                .derive_owned(|item| item.facts.as_slice())
                .contains_exactly([Fact::note(
                    assertr::renderer::RenderingContext::new(
                        &DebugRenderer,
                        RenderingBudget::default(),
                    )
                    .value(&String::from("\"Kevin\" is dead!")),
                )]);
        },
        |element: AssertThat<AssertionFailure, Capture>| {
            element
                .derive_owned(|item| item.relation.as_deref())
                .is_equal_to(Some("does not match the condition"));
            element
                .derive_owned(|item| item.facts.as_slice())
                .contains_exactly([Fact::note(
                    assertr::renderer::RenderingContext::new(
                        &DebugRenderer,
                        RenderingBudget::default(),
                    )
                    .value(&String::from("\"Otto\" is dead!")),
                )]);
        },
    ]);
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

    assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {r#"
        -------- assertr --------
        Expression: `bob`

        does not match the condition

        Details:
          - "\"Bob\" is dead!"
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

mod matcher_adapter {
    use super::*;

    #[test]
    fn conditions_compose_and_preserve_errors_without_subject_renderers() {
        use assertr::matchers::{all_of, condition};
        struct ErrorRenderer;
        impl ValueRenderer<String> for ErrorRenderer {
            fn fmt(&self, value: &String, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(value, f)
            }
        }
        let bob = Person {
            name: "Bob",
            meta: Metadata { alive: true },
        };
        let matcher = all_of((
            condition(IsAlive {}),
            condition(HasName { expected: "Bob" }),
        ));
        assert_that!(bob)
            .with_renderer(ErrorRenderer)
            .matches(&matcher);
        let failures = assert_that!(bob)
            .with_renderer(ErrorRenderer)
            .capture(|it| it.matches(condition(HasName { expected: "Alice" })));
        assert_that!(ToHumanReadableText.render(&failures[0]))
            .contains("Person has unexpected name");
    }
}
