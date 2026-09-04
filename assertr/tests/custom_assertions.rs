//! Downstream-style coverage for the public assertion-authoring methods.
//!
//! These tests are written the way a downstream crate would write them: only through
//! `assertr::prelude::*`, without reaching into any private module. They pin the two supported
//! routes for teaching assertr about your own types:
//!
//! - **Composition** - delegate to existing assertions through `satisfies` and friends. Tracking,
//!   failure formatting and capture-mode behavior come from the assertions delegated to.
//! - **Leaf assertions** - decide the outcome yourself: call `track_assertion()` first, then raise
//!   a failure through the `failure(kind)` builder when the check does not hold.
//!
//! Custom traits are the supported shape. Assertr's own `*Assertions` traits are public so their
//! methods participate in method resolution, not as downstream implementation interfaces.

// The capture closures below wrap a single custom assertion on purpose: `capture(|it| it.is_x())`
// is the spelling users write and the one the documentation shows. Passing the method path
// instead would be shorter but would stop demonstrating the API.
#![allow(clippy::redundant_closure_for_method_calls)]

#[derive(Debug, PartialEq)]
struct Person {
    age: u32,
    meta: Metadata,
}

#[derive(Debug, PartialEq)]
struct Metadata {
    alive: bool,
}

mod composed {
    use super::{Metadata, Person};
    use assertr::prelude::*;

    trait PersonAssertions<R = DebugRenderer> {
        fn has_age(self, expected: u32) -> Self
        where
            R: Clone + ValueRenderer<u32>;

        #[allow(clippy::wrong_self_convention)]
        fn is_alive(self) -> Self
        where
            R: Clone + ValueRenderer<bool>;
    }

    impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
        #[track_caller]
        fn has_age(self, expected: u32) -> Self
        where
            R: Clone + ValueRenderer<u32>,
        {
            self.satisfies(
                |p| &p.age,
                |age| {
                    age.is_equal_to(expected);
                },
            )
        }

        #[track_caller]
        fn is_alive(self) -> Self
        where
            R: Clone + ValueRenderer<bool>,
        {
            self.satisfies(
                |p| &p.meta.alive,
                |alive| {
                    alive.is_true();
                },
            )
        }
    }

    struct NoRenderer;

    fn assert_trait_is_implemented<T: PersonAssertions<NoRenderer>>(_: &T) {}

    #[test]
    fn trait_is_implemented_without_renderer_support() {
        let assertion = assert_that!(Person {
            age: 30,
            meta: Metadata { alive: true },
        })
        .with_renderer(NoRenderer);

        assert_trait_is_implemented(&assertion);
    }

    #[test]
    fn a_composed_assertion_chains_like_a_built_in_one() {
        let person = Person {
            age: 30,
            meta: Metadata { alive: true },
        };

        assert_that!(&person)
            .is_equal_to(Person {
                age: 30,
                meta: Metadata { alive: true },
            })
            .has_age(30)
            .is_alive();
    }

    #[test]
    fn a_composed_assertion_reports_through_the_delegated_assertion() {
        let person = Person {
            age: 12,
            meta: Metadata { alive: true },
        };

        let failures = assert_that!(&person)
            .with_location(false)
            .capture(|it| it.has_age(30));

        assert_that!(&failures).has_length(1);
        assert_that!(failures[0].description())
            .contains("Expected: 30")
            .contains("Actual: 12");
    }
}

mod leaf {
    use super::Person;
    use assertr::prelude::*;
    use assertr::renderer::TypeHint;
    use assertr::{Fact, FailureKind};
    use core::fmt;
    use indoc::formatdoc;

    trait PersonAssertions<R = DebugRenderer> {
        #[allow(clippy::wrong_self_convention)]
        fn is_adult(self) -> Self;
        #[allow(clippy::wrong_self_convention)]
        fn is_older_than(self, other: &Person) -> Self
        where
            R: ValueRenderer<Person>;
    }

    impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
        #[track_caller]
        fn is_adult(self) -> Self {
            // Tracking comes first and happens unconditionally: a passing assertion must count
            // just as much as a failing one.
            self.track_assertion();

            let age = self.actual().age;
            if age < 18 {
                // A failure that renders no value needs no renderer capability.
                self.failure(FailureKind::Ordering)
                    .relation("is not an adult")
                    .fact("Age", age)
                    .raise();
            }
            self
        }

        #[track_caller]
        fn is_older_than(self, other: &Person) -> Self
        where
            R: ValueRenderer<Person>,
        {
            self.track_assertion();

            let actual = self.actual();
            if actual.age <= other.age {
                // Facts belong to the failure, not to the chain: they must not reappear in a
                // later failure of the same chain.
                self.failure(FailureKind::Ordering)
                    .actual(self.render().value(actual))
                    .relation("is not older than")
                    .expected(self.render().value(other))
                    .fact("Actual age", actual.age)
                    .fact("Expected age", other.age)
                    .raise();
            }
            self
        }
    }

    fn person(age: u32) -> Person {
        Person {
            age,
            meta: super::Metadata { alive: true },
        }
    }

    struct NoRenderer;

    #[derive(Clone, Copy)]
    struct AgeRenderer;

    impl ValueRenderer<Person> for AgeRenderer {
        fn fmt(&self, value: &Person, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Person(age={})", value.age)
        }
    }

    #[test]
    fn leaf_assertions_do_not_require_renderer_support() {
        assert_that!(person(30))
            .with_renderer(NoRenderer)
            .is_adult();
    }

    #[test]
    fn a_passing_leaf_assertion_chains_like_a_built_in_one() {
        assert_that!(person(30))
            .is_adult()
            .is_older_than(&person(18));
    }

    #[test]
    // `panics()` is only available with the `std` feature.
    #[cfg(feature = "std")]
    fn a_failing_leaf_assertion_panics_in_panic_mode() {
        let panic = assert_that_owned!(|| {
            assert_that!(person(12)).with_location(false).is_adult();
        })
        .panics();

        panic.has_type::<String>().is_equal_to(formatdoc! {"
            -------- assertr --------
            Expression: `person(12)`

            is not an adult

            Details:
              - Age: 12
            -------- assertr --------
        "});
    }

    #[test]
    fn a_failing_leaf_assertion_is_collected_in_capture_mode() {
        let failures = assert_that!(person(12))
            .with_subject_name("child")
            .with_location(false)
            .capture(|it| it.is_adult());

        assert_that!(&failures).has_length(1);
        assert_that!(failures[0].subject_name.as_deref()).is_equal_to(Some("child"));
        assert_that!(failures[0].kind).is_equal_to(FailureKind::Ordering);
        assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("is not an adult"));
        assert_that!(failures[0].facts.as_slice()).contains_exactly([Fact::new("Age", "12")]);
        assert_that!(failures[0].to_string()).is_equal_to(formatdoc! {"
            -------- assertr --------
            Subject: child
            Expression: `person(12)`

            is not an adult

            Details:
              - Age: 12
            -------- assertr --------
        "});
    }

    #[test]
    fn facts_land_on_the_failure_that_raised_them_only() {
        let failures = assert_that!(person(12)).with_location(false).capture(|it| {
            it.is_older_than(&person(40)) // fails with facts
                .is_adult() // fails with one fact of its own
        });

        assert_that!(&failures).has_length(2);
        assert_that!(failures[0].facts.as_slice()).contains_exactly([
            Fact::new("Actual age", "12"),
            Fact::new("Expected age", "40"),
        ]);
        assert_that!(failures[1].facts.as_slice()).contains_exactly([Fact::new("Age", "12")]);
    }

    #[test]
    fn leaf_assertion_values_use_the_active_renderer() {
        let failures = assert_that!(person(12))
            .with_renderer(AgeRenderer)
            .with_location(false)
            .capture(|it| it.is_older_than(&person(40)));

        assert_that!(failures[0].actual.as_deref()).is_equal_to(Some("Person(age=12)"));
        assert_that!(failures[0].expected.as_deref()).is_equal_to(Some("Person(age=40)"));
        assert_that!(failures[0].description()).is_equal_to(formatdoc! {"
            Actual: Person(age=12)

            is not older than

            Expected: Person(age=40)
        "});
    }

    #[test]
    fn rendered_values_can_customize_and_show_type_hints() {
        let person = person(12);
        let assertion = assert_that!(&person).with_renderer(AgeRenderer);
        let rendered = assertion
            .render()
            .value(assertion.actual())
            .with_type_hint(TypeHint::Label("Subject"))
            .show_type_hint(true);

        assert_that!(format!("{rendered:?}")).is_equal_to("Subject Person(age=12)");
    }

    #[test]
    fn a_leaf_assertion_reports_its_own_call_site() {
        // `#[track_caller]` on the custom method has to reach through the public `failure`, or
        // every custom assertion would blame a line inside assertr.
        let failures = assert_that!(person(12)).capture(|it| it.is_adult());

        let location = failures[0].location.expect("location captured by default");
        assert_that!(location.file()).ends_with("custom_assertions.rs");
    }

    #[test]
    fn a_passing_leaf_assertion_counts_as_an_assertion() {
        // Without `track_assertion`, `capture` would treat this closure as empty and panic. This
        // is why the tracking hook is public rather than internal.
        let failures = assert_that!(person(30)).capture(|it| it.is_adult());

        assert_that!(failures.as_slice()).is_empty();
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn leaf_assertions_work_through_the_fluent_entry_points() {
        person(30).must().is_adult();

        let failures = person(12).verify(|it| it.is_adult());
        assert_that!(&failures).has_length(1);

        let mut adult = person(30);
        (&mut adult).must().is_adult();

        let mut child = person(12);
        let failures = (&mut child).verify(|it| it.is_adult());
        assert_that!(&failures).has_length(1);
    }
}

mod nested {
    use super::Person;
    use assertr::failure::FailureBuilder;
    use assertr::prelude::*;
    use assertr::{Fact, FailureKind};
    use indoc::formatdoc;

    /// A downstream assertion over a group of people that reports each rejected member as a
    /// nested failure located by its index, the way the built-in positional assertions do.
    trait GroupAssertions<R = DebugRenderer> {
        #[allow(clippy::wrong_self_convention)]
        fn are_adults(self) -> Self
        where
            R: ValueRenderer<Person>;
    }

    impl<M: Mode, R> GroupAssertions<R> for AssertThat<'_, Vec<Person>, M, R> {
        #[track_caller]
        fn are_adults(self) -> Self
        where
            R: ValueRenderer<Person>,
        {
            self.track_assertion();

            let minors = self
                .actual()
                .iter()
                .enumerate()
                .filter(|(_, person)| person.age < 18)
                .map(|(index, person)| {
                    FailureBuilder::detached::<Person>(FailureKind::Ordering)
                        .actual(self.render().value(person))
                        .relation("is not an adult")
                        .build()
                        .located_at(Fact::index(index))
                })
                .collect::<Vec<_>>();
            if !minors.is_empty() {
                self.failure(FailureKind::Predicate)
                    .relation("contains people who are not adults")
                    .children(minors)
                    .raise();
            }
            self
        }
    }

    fn person(age: u32) -> Person {
        Person {
            age,
            meta: super::Metadata { alive: true },
        }
    }

    #[derive(Clone, Copy)]
    struct AgeRenderer;

    impl ValueRenderer<Person> for AgeRenderer {
        fn fmt(&self, value: &Person, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Person(age={})", value.age)
        }
    }

    #[test]
    fn a_downstream_assertion_attaches_located_children() {
        let failures = assert_that!(vec![person(30), person(12)])
            .with_renderer(AgeRenderer)
            .with_location(false)
            .capture(|it| it.are_adults());

        assert_that!(&failures).has_length(1);
        let child = &failures[0].children[0];
        assert_that!(child.kind).is_equal_to(FailureKind::Ordering);
        assert_that!(child.actual.as_deref()).is_equal_to(Some("Person(age=12)"));
        assert_that!(child.facts.as_slice()).contains_exactly([Fact::index(1)]);
        assert_that!(failures[0].to_string()).is_equal_to(formatdoc! {"
            -------- assertr --------
            Expression: `vec![person(30), person(12)]`

            contains people who are not adults

            Nested failures:
              - At index 1:
                Actual: Person(age=12)

                is not an adult
            -------- assertr --------
        "});
    }
}

#[cfg(feature = "fluent")]
mod generated_fluent_aliases {
    // The explicit method lifetime is the regression subject.
    #![allow(clippy::needless_lifetimes, clippy::wrong_self_convention)]
    #![deny(late_bound_lifetime_arguments)]

    use assertr::prelude::*;

    #[assertr_derive::fluent_aliases]
    trait BorrowAssertions {
        #[fluent_alias("borrow_as")]
        fn is_borrowed_as<'a>(self, expected: &'a str) -> Self;
    }

    impl<M: Mode, R> BorrowAssertions for AssertThat<'_, String, M, R> {
        #[track_caller]
        fn is_borrowed_as<'a>(self, _expected: &'a str) -> Self {
            self.track_assertion();
            self
        }
    }

    #[test]
    fn aliases_support_late_bound_lifetimes() {
        "value".to_owned().must().borrow_as("expected");
    }
}
