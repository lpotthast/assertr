//! Downstream-style coverage for the public assertion-authoring methods.
//!
//! These tests are written the way a downstream crate would write them: only through
//! `assertr::prelude::*`, without reaching into any private module. They pin the two supported
//! routes for teaching assertr about your own types:
//!
//! - **Composition** - delegate to existing assertions through `satisfies` and friends. Tracking,
//!   failure formatting and capture-mode behavior come from the assertions delegated to.
//! - **Leaf assertions** - decide the outcome yourself: call `track_assertion()` first, then
//!   `fail(...)` or `fail_with_details(...)` when the check does not hold.
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
        assert_that!(&failures[0].description)
            .contains("Expected: 30")
            .contains("Actual: 12");
    }
}

mod leaf {
    use super::Person;
    use assertr::prelude::*;
    use indoc::formatdoc;

    trait PersonAssertions {
        #[allow(clippy::wrong_self_convention)]
        fn is_adult(self) -> Self;
        #[allow(clippy::wrong_self_convention)]
        fn is_older_than(self, other: &Person) -> Self;
    }

    impl<M: Mode, R> PersonAssertions for AssertThat<'_, Person, M, R> {
        #[track_caller]
        fn is_adult(self) -> Self {
            // Tracking comes first and happens unconditionally: a passing assertion must count
            // just as much as a failing one.
            self.track_assertion();

            let age = self.actual().age;
            if age < 18 {
                self.fail(format_args!(
                    "Expected the person to be an adult, but they are only {age} years old!"
                ));
            }
            self
        }

        #[track_caller]
        fn is_older_than(self, other: &Person) -> Self {
            self.track_assertion();

            let actual = self.actual();
            if actual.age <= other.age {
                // Per-failure diagnostics belong to the failure, not to the chain: they must not
                // reappear in a later failure of the same chain.
                self.fail_with_details(
                    [
                        format!("Actual person: {actual:?}"),
                        format!("Compared to:   {other:?}"),
                    ],
                    format_args!(
                        "Expected an age greater than {expected}, but was {age}!",
                        expected = other.age,
                        age = actual.age,
                    ),
                );
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

            Expected the person to be an adult, but they are only 12 years old!
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
        assert_that!(&failures[0].description)
            .is_equal_to("Expected the person to be an adult, but they are only 12 years old!");
        assert_that!(failures[0].details.as_slice()).is_empty();
        assert_that!(failures[0].to_string()).is_equal_to(formatdoc! {"
            -------- assertr --------
            Subject: child
            Expression: `person(12)`

            Expected the person to be an adult, but they are only 12 years old!
            -------- assertr --------
        "});
    }

    #[test]
    fn details_handed_to_fail_with_details_land_on_that_failure_only() {
        let failures = assert_that!(person(12)).with_location(false).capture(|it| {
            it.is_older_than(&person(40)) // fails with details
                .is_adult() // fails without details
        });

        assert_that!(&failures).has_length(2);
        assert_that!(failures[0].details.as_slice()).has_length(2);
        assert_that!(&failures[0].details[0]).contains("Actual person:");
        assert_that!(failures[1].details.as_slice()).is_empty();
    }

    #[test]
    fn a_leaf_assertion_reports_its_own_call_site() {
        // `#[track_caller]` on the custom method has to reach through the public `fail`, or every
        // custom assertion would blame a line inside assertr.
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

mod custom_failure_writer {
    use assertr::failure::Failure;
    use assertr::prelude::*;

    /// A downstream implementation of the [`Failure`] trait, proving it stays open for the
    /// zero-allocation "write the description yourself" route.
    struct Bullets<'a>(&'a [&'a str]);

    impl Failure for Bullets<'_> {
        fn write_to(self, target: &mut String) -> core::fmt::Result {
            use core::fmt::Write as _;
            for line in self.0 {
                writeln!(target, "- {line}")?;
            }
            Ok(())
        }
    }

    trait BulletAssertions {
        fn fails_with_bullets(self) -> Self;
    }

    impl<M: Mode, R> BulletAssertions for AssertThat<'_, u32, M, R> {
        #[track_caller]
        fn fails_with_bullets(self) -> Self {
            self.track_assertion();
            self.fail(Bullets(&["first", "second"]));
            self
        }
    }

    #[test]
    fn a_downstream_failure_impl_writes_the_description() {
        let failures = assert_that!(1u32)
            .with_location(false)
            .capture(|it| it.fails_with_bullets());

        assert_that!(&failures).has_length(1);
        assert_that!(&failures[0].description).is_equal_to("- first\n- second\n");
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
