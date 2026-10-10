//! Downstream-style coverage for the public assertion-authoring methods.
//!
//! These tests are written the way a downstream crate would write them: only through
//! `assertr::prelude::*`, without reaching into any private module. They pin the supported
//! routes for teaching assertr about your own types, plus the execution-adapter escape hatch:
//!
//! - **Composition** - delegate to existing assertions through `satisfies` and friends. Tracking,
//!   failure formatting and capture-mode behavior come from the assertions delegated to.
//! - **Reusable checks without an implementation** - build an `Expectation` from `predicate`,
//!   `field`, and the other matchers, and return it as `impl Expectation`.
//! - **Leaf assertions** - implement `Expectation`, then delegate to `matches` or `test_assertion`,
//!   which track, evaluate, and raise the explained failure.
//! - **Execution adapters** - for an operation the expectation protocol cannot express, call
//!   `track_assertion()` first, then raise a failure through the `failure(kind)` builder when the
//!   check does not hold.
//!
//! Custom traits are the supported shape. Assertr's own `*Assertions` traits are public so their
//! methods participate in method resolution, not as downstream implementation interfaces.

// The capture closures below wrap a single custom assertion on purpose: `capture(|it| it.is_x())`
// is the spelling users write and the one the documentation shows. Passing the method path instead
// would be shorter but would stop demonstrating the API.
#![allow(clippy::redundant_closure_for_method_calls)]

mod support;

use self::support::text_opt;

#[cfg(feature = "tokio")]
mod watch_trait_imports {
    use assertr::{
        assert_that,
        assertions::TokioWatchReceiverAssertions,
        prelude::{BoolAssertions, LengthAssertions},
    };

    fn require_changed<R, A: TokioWatchReceiverAssertions<i32, R>>(assertion: A) -> A {
        assertion.has_changed()
    }

    #[test]
    fn canonical_trait_supports_generic_and_qualified_calls_in_both_modes() {
        use assertr::prelude::TokioWatchReceiverAssertions as PreludeAssertions;

        struct NoRenderer;
        let (_sender, mut receiver) = tokio::sync::watch::channel(7);
        receiver.mark_changed();
        require_changed(assert_that!(receiver).with_renderer(NoRenderer));
        TokioWatchReceiverAssertions::has_changed(assert_that!(receiver));
        let failures = assert_that!(receiver)
            .with_renderer(NoRenderer)
            .capture(|it| require_changed(it).has_not_changed());
        assert_that!(failures).has_length(1);
        assert_that!(receiver.has_changed().unwrap()).is_true();

        receiver.mark_unchanged();
        PreludeAssertions::has_not_changed(assert_that!(receiver));
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn canonical_import_alone_supplies_fluent_aliases() {
        use assertr::FluentEntry;

        let (_sender, mut receiver) = tokio::sync::watch::channel(7);
        receiver.must().not_have_changed();
        receiver.mark_changed();
        receiver.must().have_changed();
        let failures = receiver.verify(|it| it.not_have_changed().have_changed());
        assert_that!(failures).has_length(1);
    }
}

#[cfg(feature = "std")]
mod path_renderer_bounds {
    use std::{fmt, io, path::PathBuf};

    use assertr::{matchers::path::DoesNotExist, prelude::*};

    use super::text_opt;

    // Deliberately neither Clone nor a renderer for any unrelated diagnostic type.
    struct PathAndErrorRenderer;

    impl ValueRenderer<PathBuf> for PathAndErrorRenderer {
        fn fmt(&self, _: &PathBuf, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("<path>")
        }
    }

    impl ValueRenderer<io::Error> for PathAndErrorRenderer {
        fn fmt(&self, _: &io::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("<inspection error>")
        }
    }

    fn check_absence<P, R, A>(assertion: A) -> A
    where
        P: std::ops::Deref<Target = std::path::Path>,
        R: ValueRenderer<P> + ValueRenderer<io::Error>,
        A: PathAssertions<P, R>,
    {
        assertion.does_not_exist()
    }

    #[test]
    fn absence_methods_and_expectations_require_only_path_and_error_rendering() {
        let path = PathBuf::from("invalid\0path");
        let failures = assert_that!(path)
            .with_renderer(PathAndErrorRenderer)
            .capture(|it| check_absence(it).matches(DoesNotExist));
        assert_that!(failures).has_length(2);
        for failure in &failures {
            assert_that!(failure.relation.as_deref())
                .is_equal_to(Some("could not determine whether the path exists"));
            assert_that!(text_opt(failure.actual.as_ref())).is_equal_to(Some("<path>"));
            assert_that!(failure.facts).has_length(1);
            assert_that!(text_opt(Some(&failure.facts[0].value)))
                .is_equal_to(Some("<inspection error>"));
        }

        #[cfg(feature = "fluent")]
        {
            let failures = assert_that!(path)
                .with_renderer(PathAndErrorRenderer)
                .capture(|it| it.not_exist());
            assert_that!(failures).has_length(1);
        }
    }
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

mod composed {
    use assertr::prelude::*;

    use super::{Metadata, Person};

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

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.to_string())
                    .contains("Expected: 30")
                    .contains("Actual: 12");
            },
        ]);
    }
}

mod reusable_without_impl {
    use assertr::{
        expectation::AssertionContext,
        matchers::{field, ge, predicate},
        prelude::*,
    };

    #[derive(Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    fn is_adult<R: ValueRenderer<u32>>() -> impl Expectation<Person, R> + Clone {
        field("age", |person: &Person| &person.age, ge(18))
    }

    fn has_capitalized_name<R: ValueRenderer<String>>() -> impl Expectation<Person, R> + Clone {
        field(
            "name",
            |person: &Person| &person.name,
            predicate(|name: &String| name.starts_with(char::is_uppercase))
                .described_as("is capitalized")
                .rejected_as("is not capitalized"),
        )
    }

    #[test]
    fn checks_compose_as_assertions_and_matchers() {
        let ada = Person {
            name: "Ada".into(),
            age: 36,
        };
        assert_that!(&ada)
            .matches(is_adult())
            .matches(has_capitalized_name());
        assert_that!([ada]).contains_matching(matchers::all_of(matchers![
            is_adult(),
            has_capitalized_name()
        ]));
    }

    #[test]
    fn rejections_report_the_field_and_the_named_relation() {
        let minor = Person {
            name: "ada".into(),
            age: 16,
        };
        let failures = assert_that!(minor)
            .with_location(false)
            .capture(|it| it.matches(is_adult()).matches(has_capitalized_name()));

        assert_that!(failures).has_length(2);
        assert_that!(failures[1].to_string()).is_equal_to(indoc::indoc! {r#"
            -------- assertr --------
            Expression: `minor`

            does not match

            Nested failures:
              - At .name:
                Actual: "ada"

                is not capitalized
            -------- assertr --------
        "#});
    }

    struct IsEven;

    // The shortest full implementation: default `KIND`, no lifetime bounds for an owned subject,
    // and `relations` for the common explanation.
    impl<R: ValueRenderer<u32>> Expectation<u32, R> for IsEven {
        type Success<'a> = ();
        type Rejection<'a> = ();

        fn evaluate(&self, actual: &u32, _: &AssertionContext<'_, R>) -> Result<(), ()> {
            if actual.is_multiple_of(2) {
                Ok(())
            } else {
                Err(())
            }
        }

        fn explain(
            &self,
            rejected: Option<(&u32, ())>,
            failure: assertr::failure::FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> assertr::failure::FailureBuilder {
            failure.relations(
                rejected.map(|(actual, ())| context.render().value(actual)),
                "is even",
                "is odd",
            )
        }
    }

    #[test]
    fn a_minimal_implementation_uses_the_default_kind_and_relations() {
        assert_that!(4_u32).matches(IsEven);
        let failures = assert_that!(3_u32).capture(|it| it.matches(IsEven));
        assert_that!(failures[0].kind).is_equal_to(assertr::failure::FailureKind::Predicate);
        assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("is odd"));
    }
}

mod leaf {
    use core::fmt;

    use assertr::{
        failure::{Fact, FailureKind},
        prelude::*,
    };
    use indoc::formatdoc;

    use super::Person;

    trait PersonAssertions<R = DebugRenderer> {
        #[allow(clippy::wrong_self_convention)]
        fn is_adult(self) -> Self;
        #[allow(clippy::wrong_self_convention)]
        fn is_older_than(self, other: &Person) -> Self
        where
            R: ValueRenderer<Person> + ValueRenderer<u32>;
    }

    impl<M: Mode, R> PersonAssertions<R> for AssertThat<'_, Person, M, R> {
        #[track_caller]
        fn is_adult(self) -> Self {
            // Tracking comes first and happens unconditionally: a passing assertion must count just
            // as much as a failing one.
            self.track_assertion();

            let age = self.actual().age;
            if age < 18 {
                // A failure that renders no value needs no renderer capability. Prose notes are
                // verbatim text, not values.
                self.raise(
                    self.failure(FailureKind::Ordering)
                        .relation("is not an adult")
                        .fact(Fact::note("An adult is at least 18 years old.")),
                );
            }
            self
        }

        #[track_caller]
        fn is_older_than(self, other: &Person) -> Self
        where
            R: ValueRenderer<Person> + ValueRenderer<u32>,
        {
            self.track_assertion();

            let actual = self.actual();
            if actual.age <= other.age {
                // Facts belong to the failure, not to the chain: they must not reappear in a later
                // failure of the same chain.
                self.raise(
                    self.failure(FailureKind::Ordering)
                        .actual(self.render().value(actual))
                        .relation("is not older than")
                        .expected(self.render().value(other))
                        .fact(Fact::labelled(
                            "Actual age",
                            self.render().value(&actual.age),
                        ))
                        .fact(Fact::labelled(
                            "Expected age",
                            self.render().value(&other.age),
                        )),
                );
            }
            self
        }
    }

    fn age(value: u32) -> assertr::renderer::Rendered {
        assertr::renderer::RenderingContext::new(&DebugRenderer, RenderingBudget::default())
            .value(&value)
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
    impl ValueRenderer<u32> for AgeRenderer {
        fn fmt(&self, value: &u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{value} years")
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
        // The capture-mode test below pins the complete report.
        assert_that!(|| {
            assert_that!(person(12)).with_location(false).is_adult();
        })
        .panics()
        .has_message()
        .contains("is not an adult");
    }

    #[test]
    fn a_failing_leaf_assertion_is_collected_in_capture_mode() {
        let failures = assert_that!(person(12))
            .with_subject_name("child")
            .with_location(false)
            .capture(|it| it.is_adult());

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.subject_name.as_deref())
                    .is_equal_to(Some("child"));
                element
                    .derive(|value| &value.kind)
                    .is_equal_to(FailureKind::Ordering);
                element
                    .derive_owned(|value| value.relation.as_deref())
                    .is_equal_to(Some("is not an adult"));
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .contains_exactly([Fact::note("An adult is at least 18 years old.")]);
                element
                    .derive_owned(|value| value.to_string())
                    .contains(formatdoc! {"
            -------- assertr --------
            Subject: child
            Expression: `person(12)`

            is not an adult

            Details:
              - An adult is at least 18 years old.
            -------- assertr --------
        "});
            },
        ]);
    }

    #[test]
    fn facts_land_on_the_failure_that_raised_them_only() {
        let failures = assert_that!(person(12)).with_location(false).capture(|it| {
            it.is_older_than(&person(40)) // fails with facts
                .is_adult() // fails with one fact of its own
        });

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .contains_exactly([
                        Fact::labelled("Actual age", age(12)),
                        Fact::labelled("Expected age", age(40)),
                    ]);
            },
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .contains_exactly([Fact::note("An adult is at least 18 years old.")]);
            },
        ]);
    }

    #[test]
    fn leaf_assertion_values_use_the_active_renderer() {
        let failures = assert_that!(person(12))
            .with_renderer(AgeRenderer)
            .with_location(false)
            .capture(|it| it.is_older_than(&person(40)));

        assert_that!(super::text_opt(failures[0].actual.as_ref()))
            .is_equal_to(Some("Person(age=12)"));
        assert_that!(super::text_opt(failures[0].expected.as_ref()))
            .is_equal_to(Some("Person(age=40)"));
        assert_that!(failures[0].to_string()).contains(formatdoc! {"
            Actual: Person(age=12)

            is not older than

            Expected: Person(age=40)
        "});
        assert_that!(failures[0].to_string()).contains("- Actual age: 12 years");
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
        // Without `track_assertion`, `capture` would treat this closure as empty and panic. This is
        // why the tracking hook is public rather than internal.
        let failures = assert_that!(person(30)).capture(|it| it.is_adult());

        assert_that!(&failures[..]).is_empty();
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

#[cfg(feature = "fluent")]
mod generated_fluent_aliases {
    // The explicit method lifetime is the regression subject.
    #![allow(clippy::needless_lifetimes, clippy::wrong_self_convention)]
    #![deny(late_bound_lifetime_arguments)]

    use assertr::prelude::*;

    #[assertr_macros::fluent_aliases]
    trait BorrowAssertions {
        #[fluent_alias("borrow_as")]
        fn is_borrowed_as<'a>(self, expected: &'a str) -> Self;

        async fn is_valid(self) -> Self;

        async fn has_values<T, const N: usize>(self, expected: [T; N]) -> [T; N]
        where
            Self: Sized,
        {
            core::future::ready(expected).await
        }

        // Unnamed patterns receive hygienic names that cannot collide with these spellings.
        #[allow(non_upper_case_globals)]
        fn has_pair<const argument_3: usize>(
            self,
            argument_2: usize,
            (_left, _right): (usize, usize),
            _: bool,
        ) -> usize
        where
            Self: Sized,
        {
            argument_2 + argument_3
        }
    }

    impl<M: Mode, R> BorrowAssertions for AssertThat<'_, String, M, R> {
        #[track_caller]
        fn is_borrowed_as<'a>(self, _expected: &'a str) -> Self {
            self.track_assertion();
            self
        }

        async fn is_valid(self) -> Self {
            self.track_assertion();
            core::future::ready(self).await
        }
    }

    #[test]
    fn aliases_support_late_bound_lifetimes() {
        "value".to_owned().must().borrow_as("expected");
    }

    #[test]
    fn aliases_name_unnamed_patterns_hygienically() {
        let sum = "value".to_owned().must().have_pair::<2>(1, (3, 4), true);
        assert_that!(sum).is_equal_to(3);
    }

    #[tokio::test]
    async fn aliases_await_async_methods() {
        "value"
            .to_owned()
            .must()
            .be_valid()
            .await
            .borrow_as("expected");

        let values: [String; 2] = "value"
            .to_owned()
            .must()
            .have_values::<String, 2>(["first".to_owned(), "second".to_owned()])
            .await;
        assert_that!(values).contains_exactly(["first", "second"]);
    }
}

// These helpers deliberately take `T` by value to model the signature available to downstream
// generic code, rather than proving the assertion bounds only for `&T`.
#[allow(clippy::needless_pass_by_value)]
#[cfg(feature = "num")]
mod generic_num_traits_bounds {
    use core::{
        fmt::Debug,
        ops::{Add, Div, Mul, Neg, Rem, Sub},
    };

    use assertr::{assertions::NumericDistance, prelude::*};
    use num_traits::{Num, One, Signed, Zero};

    #[derive(Debug, PartialEq, PartialOrd)]
    struct Money(i64);

    impl Add for Money {
        type Output = Self;

        fn add(self, rhs: Self) -> Self {
            Self(self.0 + rhs.0)
        }
    }

    impl Sub for Money {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self {
            Self(self.0 - rhs.0)
        }
    }

    impl Mul for Money {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self {
            Self(self.0 * rhs.0)
        }
    }

    impl Div for Money {
        type Output = Self;

        fn div(self, rhs: Self) -> Self {
            Self(self.0 / rhs.0)
        }
    }

    impl Rem for Money {
        type Output = Self;

        fn rem(self, rhs: Self) -> Self {
            Self(self.0 % rhs.0)
        }
    }

    impl Neg for Money {
        type Output = Self;

        fn neg(self) -> Self {
            Self(-self.0)
        }
    }

    impl Zero for Money {
        fn zero() -> Self {
            Self(0)
        }

        fn is_zero(&self) -> bool {
            self.0 == 0
        }
    }

    impl One for Money {
        fn one() -> Self {
            Self(1)
        }
    }

    impl Num for Money {
        type FromStrRadixErr = <i64 as Num>::FromStrRadixErr;

        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            i64::from_str_radix(str, radix).map(Self)
        }
    }

    impl NumericDistance for Money {
        fn zero_distance() -> Self {
            Self(0)
        }

        fn checked_distance(&self, other: &Self) -> Option<Self> {
            self.0.checked_distance(&other.0).map(Self)
        }
    }

    impl Signed for Money {
        fn abs(&self) -> Self {
            Self(self.0.abs())
        }

        fn abs_sub(&self, other: &Self) -> Self {
            Self(Signed::abs_sub(&self.0, &other.0))
        }

        fn signum(&self) -> Self {
            Self(self.0.signum())
        }

        fn is_positive(&self) -> bool {
            self.0.is_positive()
        }

        fn is_negative(&self) -> bool {
            self.0.is_negative()
        }
    }

    fn assert_identities<T: Num + Debug>(zero: T, one: T) {
        assert_that!(zero).is_zero();
        assert_that!(one).is_one();
    }

    fn assert_close_to<T: Num + NumericDistance + Debug>(value: T, expected: T, deviation: T) {
        assert_that!(value).is_close_to(expected, deviation);
    }

    fn assert_sign<T: Num + Signed + Debug>(negative: T, positive: T) {
        assert_that!(negative).is_negative();
        assert_that!(positive).is_positive();
    }

    #[test]
    fn numeric_trait_is_available_without_renderer_support() {
        struct NoRenderer;
        fn accepts_numeric_assertions<T: Num, A: NumAssertions<T, NoRenderer>>(_: &A) {}

        let value = Money(42);
        let assertion = assert_that!(value).with_renderer(NoRenderer);
        accepts_numeric_assertions::<Money, _>(&assertion);
    }

    #[test]
    fn a_custom_distance_preserves_the_renderer_without_clone_bounds() {
        struct Cents;
        impl ValueRenderer<Money> for Cents {
            fn fmt(&self, value: &Money, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{} cents", value.0)
            }
        }

        let failures = assert_that!(Money(42))
            .with_renderer(Cents)
            .capture(|it| it.is_close_to(Money(40), Money(1)));
        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|item| super::text_opt(item.actual.as_ref()))
                    .is_equal_to(Some("42 cents"));
                element
                    .derive_owned(|item| super::text_opt(item.expected.as_ref()))
                    .is_equal_to(Some("40 cents"));
                element
                    .derive_owned(|item| super::text_opt(Some(&item.facts[0].value)))
                    .is_equal_to(Some("2 cents"));
                element
                    .derive_owned(|item| super::text_opt(Some(&item.facts[1].value)))
                    .is_equal_to(Some("1 cents"));
            },
        ]);
    }

    #[test]
    fn a_user_defined_numeric_type_reaches_assertions_through_generic_bounds() {
        assert_identities(Money(0), Money(1));
        assert_close_to(Money(42), Money(40), Money(2));
        assert_sign(Money(-7), Money(7));
    }
}

mod callback_renderer_bounds {
    use std::collections::{BTreeMap, LinkedList};

    use assertr::{
        assertions::{Collection, HasLength},
        prelude::*,
        renderer::CollectionPresentation,
    };

    struct Secret;

    #[derive(Clone)]
    struct NoRenderer;

    #[derive(Clone)]
    struct CountRenderer;

    impl ValueRenderer<usize> for CountRenderer {
        fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "count({value})")
        }
    }

    #[derive(Clone)]
    struct QueryRenderer;

    impl ValueRenderer<str> for QueryRenderer {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "key({value})")
        }
    }

    fn is_some<R>(it: AssertThat<'_, Option<Secret>, Capture, R>) {
        it.is_some();
    }

    // This subject grants order-free traversal only, without StableOrder or IntoIterator.
    struct Bag([Option<Secret>; 1]);

    impl HasLength for Bag {
        fn length(&self) -> usize {
            self.0.len()
        }
    }

    impl Collection for Bag {
        type Item = Option<Secret>;
        const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

        fn elements(&self) -> impl Iterator<Item = &Self::Item> {
            self.0.iter()
        }
    }

    // Borrowed traversal does not require any collection capability.
    struct BorrowedItems([Option<Secret>; 1]);

    impl<'a> IntoIterator for &'a BorrowedItems {
        type Item = &'a Option<Secret>;
        type IntoIter = core::slice::Iter<'a, Option<Secret>>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    #[test]
    fn collection_callbacks_need_only_the_renderers_their_failures_use() {
        let values = Bag([Some(Secret)]);
        assert_that!(values)
            .with_renderer(NoRenderer)
            .contains_satisfying(is_some);
        assert_that!(values)
            .with_renderer(CountRenderer)
            .contains_exactly_in_any_order_satisfying([is_some]);
    }

    #[test]
    fn stable_order_callbacks_need_only_a_count_renderer() {
        let values = LinkedList::from([Some(Secret)]);
        assert_that!(values)
            .with_renderer(CountRenderer)
            .starts_with_satisfying([is_some])
            .ends_with_satisfying([is_some])
            .contains_contiguous_satisfying([is_some])
            .contains_exactly_satisfying([is_some]);
    }

    #[test]
    fn direct_iterator_callbacks_need_only_a_count_renderer() {
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .contains_satisfying(is_some);
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .starts_with_satisfying([is_some]);
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .ends_with_satisfying([is_some]);
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .contains_contiguous_satisfying([is_some]);
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .contains_exactly_satisfying([is_some]);
        assert_that_owned!([Some(Secret)].into_iter())
            .with_renderer(CountRenderer)
            .contains_exactly_in_any_order_satisfying([is_some]);
    }

    #[test]
    fn borrowed_iterator_callbacks_need_only_a_count_renderer() {
        assert_that!(BorrowedItems([Some(Secret)]))
            .with_renderer(CountRenderer)
            .into_iter_contains_satisfying(is_some)
            .into_iter_contains_exactly_in_any_order_satisfying([is_some]);
    }

    #[test]
    fn map_entry_callback_needs_only_a_query_renderer() {
        // QueryRenderer cannot render the stored String keys or Option<Secret> values.
        assert_that!(BTreeMap::from([(String::from("secret"), Some(Secret))]))
            .with_renderer(QueryRenderer)
            .contains_entry_satisfying("secret", is_some);
    }

    #[test]
    fn exact_map_callbacks_need_only_key_renderers() {
        assert_that!(BTreeMap::from([(1_usize, Some(Secret))]))
            .with_renderer(CountRenderer)
            .contains_exactly_entries_satisfying([(1_usize, is_some)]);
    }

    #[cfg(feature = "fluent")]
    #[test]
    fn fluent_callbacks_keep_the_relaxed_bounds() {
        Bag([Some(Secret)])
            .must()
            .with_renderer(NoRenderer)
            .contain_satisfying(is_some);
        LinkedList::from([Some(Secret)])
            .must()
            .with_renderer(CountRenderer)
            .contain_exactly_satisfying([is_some]);
        [Some(Secret)]
            .into_iter()
            .must_owned()
            .with_renderer(CountRenderer)
            .contain_satisfying(is_some);
        BorrowedItems([Some(Secret)])
            .must()
            .with_renderer(CountRenderer)
            .into_iter_contain_satisfying(is_some);
        BTreeMap::from([(String::from("secret"), Some(Secret))])
            .must()
            .with_renderer(QueryRenderer)
            .contain_entry_satisfying("secret", is_some);
    }
}

mod matcher_authoring {
    use core::{
        cell::{Cell, Ref, RefCell},
        fmt,
    };

    use assertr::{
        expectation::{AssertionContext, Evidence},
        failure::{Fact, FailureBuilder, FailureKind},
        matchers::{all_of, each},
        prelude::*,
        renderer::RenderedBody,
    };

    use super::text_opt;

    // Retain a guarded observation, as assertions over cells, locks, and receivers need to do.
    struct HasText<'e>(&'e str);

    impl<R: ValueRenderer<str>> Expectation<RefCell<String>, R> for HasText<'_> {
        type Success<'a>
            = ()
        where
            Self: 'a,
            RefCell<String>: 'a;
        type Rejection<'a>
            = Ref<'a, String>
        where
            Self: 'a,
            RefCell<String>: 'a;

        fn evaluate<'a>(
            &'a self,
            actual: &'a RefCell<String>,
            _: &AssertionContext<'_, R>,
        ) -> Result<(), Self::Rejection<'a>> {
            // An exclusive acquisition also catches a guard retained by a preceding pair.
            drop(actual.borrow_mut());
            let observed = actual.borrow();
            if observed.as_str() == self.0 {
                Ok(())
            } else {
                Err(observed)
            }
        }

        const KIND: FailureKind = FailureKind::Equality;

        fn explain<'a>(
            &'a self,
            rejected: Option<(&'a RefCell<String>, Ref<'a, String>)>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            let render = context.render();
            let failure = match rejected {
                None => failure.relation("has text"),
                Some((_, observed)) => failure.actual(render.value(observed.as_str())),
            };
            failure.expected(render.value(self.0))
        }
    }

    struct Subject(u32);
    struct OpaqueError(u32);

    #[derive(Default)]
    struct Reject {
        observations: Cell<usize>,
    }

    impl<R: ValueRenderer<OpaqueError>> Expectation<Subject, R> for Reject {
        type Success<'a> = ();
        type Rejection<'a> = OpaqueError;

        fn evaluate(
            &self,
            actual: &Subject,
            _: &AssertionContext<'_, R>,
        ) -> Result<(), OpaqueError> {
            self.observations.set(self.observations.get() + 1);
            Err(OpaqueError(actual.0))
        }

        const KIND: FailureKind = FailureKind::Predicate;

        fn explain(
            &self,
            rejected: Option<(&Subject, OpaqueError)>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            match rejected {
                None => failure.relation("is accepted"),
                Some((_, error)) => failure
                    .relation("is rejected")
                    .fact(Fact::note(context.render().value(&error))),
            }
        }
    }

    // No Clone, subject renderer, or formatting traits on the subject or rejection are needed.
    struct ErrorRenderer;
    impl ValueRenderer<OpaqueError> for ErrorRenderer {
        fn fmt(&self, error: &OpaqueError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "error({})", error.0)
        }
    }

    // A downstream composite must use only the public context and retain owned child evidence.
    struct Twice<D>(D);

    impl<T: ?Sized, R, D: Expectation<T, R>> Expectation<T, R> for Twice<D> {
        type Success<'a>
            = ()
        where
            Self: 'a,
            T: 'a;
        type Rejection<'a>
            = Evidence
        where
            Self: 'a,
            T: 'a;

        fn evaluate(&self, actual: &T, context: &AssertionContext<'_, R>) -> Result<(), Evidence> {
            let mut children = context.isolated();
            let first = children.evaluate(actual, &self.0);
            let second = children.evaluate(actual, &self.0);
            if first && second {
                Ok(())
            } else {
                Err(children.into_evidence())
            }
        }

        const KIND: FailureKind = D::KIND;

        fn explain<'a>(
            &'a self,
            rejected: Option<(&'a T, Evidence)>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            match rejected {
                None => self.0.explain(None, failure, context),
                Some((_, children)) => failure.evidence(children),
            }
        }
    }

    mod guarded_observations {
        use super::*;

        #[test]
        fn diagnostics_use_the_retained_observation_and_release_its_guard() {
            let expected = String::from("expected");
            let definition = HasText(&expected);
            let actual = RefCell::new(String::from("observed"));
            let failures = assert_that!(actual)
                .matches(HasText("observed"))
                .capture(|it| it.matches(&definition).matches(&definition));
            assert_that!(failures).has_length(2);
            *actual.borrow_mut() = String::from("changed");
            for failure in &failures {
                assert_that!(text_opt(failure.actual.as_ref())).is_equal_to(Some("\"observed\""));
                assert_that!(text_opt(failure.expected.as_ref())).is_equal_to(Some("\"expected\""));
            }
        }

        #[test]
        fn unordered_samples_explain_each_guarded_observation_before_the_next_pair() {
            for limit in [0, 1, 2, usize::MAX] {
                let actual = [
                    RefCell::new(String::from("first")),
                    RefCell::new(String::from("second")),
                ];
                let failures = assert_that!(actual)
                    .with_rendering_budget(RenderingBudget::unlimited().with_max_items(limit))
                    .capture(|it| {
                        it.matches(elements_are_in_any_order![
                            HasText("expected"),
                            HasText("expected")
                        ])
                    });
                assert_that!(failures).has_length(1);
                assert_that!(actual[0].borrow().as_str()).is_equal_to("first");
                assert_that!(actual[1].borrow().as_str()).is_equal_to("second");
                actual[0].borrow_mut().clear();
                actual[1].borrow_mut().clear();
                if limit > 0 {
                    let missing = &failures[0].children[0];
                    assert_that!(missing.children).has_length(limit.min(2));
                    assert_that!(text_opt(missing.children[0].actual.as_ref()))
                        .is_equal_to(Some("\"first\""));
                    assert_that!(missing.omitted_children).is_equal_to(2 - limit.min(2));
                }
            }
        }

        #[test]
        fn downstream_compositions_retain_children_with_the_supplied_context() {
            let actual = RefCell::new(String::from("observed"));
            let matcher = Twice(HasText("expected"));
            let failures =
                assert_that!(actual).capture(|it| it.matches(&matcher).matches(&matcher));
            assert_that!(failures).has_length(2);
            *actual.borrow_mut() = String::from("released");
            for failure in &failures {
                assert_that!(failure.children).has_length(2);
                for child in &failure.children {
                    assert_that!(text_opt(child.actual.as_ref())).is_equal_to(Some("\"observed\""));
                    assert_that!(text_opt(child.expected.as_ref()))
                        .is_equal_to(Some("\"expected\""));
                }
            }
        }
    }

    mod owned_rejections {
        use super::*;

        fn assert_error(failure: &AssertionFailure, text: &str, omitted_characters: usize) {
            assert_that!(failure.kind).is_equal_to(FailureKind::Predicate);
            assert_that!(failure.relation.as_deref()).is_equal_to(Some("is rejected"));
            assert_that!(failure.actual).is_none();
            assert_that!(failure.facts).has_length(1);
            let fact = &failure.facts[0];
            assert_that!(fact.label).is_none();
            assert_that!(fact.value.type_name)
                .is_equal_to(Some(core::any::type_name::<OpaqueError>()));
            let RenderedBody::Text {
                text: retained,
                omitted_characters: omitted,
                ..
            } = &fact.value.body
            else {
                panic!("expected a text node, got {:?}", fact.value.body);
            };
            assert_that!(retained.as_str()).is_equal_to(text);
            assert_that!(*omitted).is_equal_to(omitted_characters);
        }

        #[test]
        fn direct_execution_reuses_definitions_and_renders_original_errors_with_the_budget() {
            let definition = Reject::default();
            let failures = assert_that!(Subject(42))
                .with_renderer(ErrorRenderer)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(3))
                .capture(|it| it.matches(&definition).matches(&definition));

            assert_that!(definition.observations.get()).is_equal_to(2);
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_error(failure, "err", 6);
            }
        }

        #[test]
        fn composition_retains_typed_errors_without_rendering_subjects() {
            let definition = Reject::default();
            let failures = assert_that!(Subject(42))
                .with_renderer(ErrorRenderer)
                .capture(|it| it.matches(Twice(&definition)));

            assert_that!(definition.observations.get()).is_equal_to(2);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(2);
            for child in &failures[0].children {
                assert_error(child, "error(42)", 0);
            }

            let failures = assert_that!([Subject(42), Subject(43)])
                .with_renderer(ErrorRenderer)
                .capture(|it| it.matches(each(&definition)));

            assert_that!(definition.observations.get()).is_equal_to(4);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(2);
            assert_error(&failures[0].children[0], "error(42)", 0);
            assert_error(&failures[0].children[1], "error(43)", 0);
        }

        #[test]
        fn exhausted_child_evidence_does_not_render_errors_or_skip_evaluation() {
            struct NeverRender;
            impl ValueRenderer<OpaqueError> for NeverRender {
                fn fmt(&self, _: &OpaqueError, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                    panic!("omitted errors must not be rendered")
                }
            }

            let definition = Reject::default();
            let failures = assert_that!(Subject(42))
                .with_renderer(NeverRender)
                .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                .capture(|it| it.matches(all_of(matchers![&definition, &definition])));

            assert_that!(definition.observations.get()).is_equal_to(2);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).is_empty();
            assert_that!(failures[0].omitted_children).is_equal_to(2);
        }
    }
}

mod structural_rendering {
    use core::fmt;

    use assertr::{
        assertions::{Collection, HasLength},
        expectation::AssertionContext,
        failure::{FailureBuilder, FailureKind, PathSegment},
        prelude::*,
        renderer::{CollectionPresentation, RenderedBody},
    };

    // Neither subjects nor leaves implement Debug. The renderer is deliberately not Clone.
    struct Token(&'static str);
    struct LeafRenderer;

    impl ValueRenderer<Token> for LeafRenderer {
        fn fmt(&self, value: &Token, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "token({})", value.0)
        }
    }

    impl ValueRenderer<str> for LeafRenderer {
        fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "text({value})")
        }
    }

    struct Bag<T>(Vec<T>);

    impl<T> HasLength for Bag<T> {
        fn length(&self) -> usize {
            self.0.len()
        }
    }

    impl<T> Collection for Bag<T> {
        type Item = T;
        const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
        fn elements(&self) -> impl Iterator<Item = &T> {
            self.0.iter()
        }
    }

    // Detailed tree metadata, ordering, and omission behavior live in renderer::context tests.
    #[test]
    fn chain_settings_reach_custom_collection_rendering() {
        let values = Bag(vec![Token("first"), Token("second")]);
        let budget = RenderingBudget::unlimited()
            .with_max_items(1)
            .with_max_leaf_characters(5);
        let failures = assert_that!(values)
            .with_renderer(LeafRenderer)
            .with_subject_name("tokens")
            .with_location(false)
            .with_rendering_budget(budget)
            .capture(|it| {
                it.track_assertion();
                assert_that!(it.render().budget()).is_equal_to(budget);
                it.raise(
                    it.failure(FailureKind::Length)
                        .actual(it.render().collection(it.actual()))
                        .relation("is not empty"),
                );
                it
            });
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].subject_name.as_deref()).is_equal_to(Some("tokens"));
        assert_that!(failures[0].location).is_none();
        let RenderedBody::Group { items, omitted, .. } = &failures[0].actual.as_ref().unwrap().body
        else {
            panic!("expected a collection");
        };
        assert_that!(*omitted).is_equal_to(1);
        assert_that!(items).has_length(1);
        let RenderedBody::Text {
            text,
            omitted_characters,
            ..
        } = &items[0].body
        else {
            panic!("expected a text node, got {:?}", items[0].body);
        };
        assert_that!(text.as_str()).is_equal_to("token");
        assert_that!(*omitted_characters).is_equal_to(7);
    }

    struct ForwardEvidence(RenderingBudget);

    impl<R: ValueRenderer<Token>> Expectation<Token, R> for ForwardEvidence {
        type Success<'a> = ();
        type Rejection<'a> = ();

        fn evaluate(&self, _: &Token, context: &AssertionContext<'_, R>) -> Result<(), ()> {
            assert_that!(context.render().budget()).is_equal_to(self.0);
            Err(())
        }

        const KIND: FailureKind = FailureKind::Predicate;

        fn explain<'a>(
            &'a self,
            rejected: Option<(&'a Token, ())>,
            failure: FailureBuilder,
            context: &AssertionContext<'_, R>,
        ) -> FailureBuilder {
            let render = context.render();
            assert_that!(render.budget()).is_equal_to(self.0);
            let failure = failure.relation("has rejected evidence");
            let Some((token, ())) = rejected else {
                return failure;
            };
            // Fixed evidence exercises the downstream builder contract without a local collector.
            failure
                .omitted_children(2)
                .children([FailureBuilder::new::<Token>(FailureKind::Predicate)
                    .actual(render.value(token))
                    .relation("is rejected")
                    .path([PathSegment::Index(1)])
                    .build()])
        }
    }

    #[test]
    fn custom_expectation_receives_the_budget_and_forwards_located_children_and_omissions() {
        let budget = RenderingBudget::default()
            .with_max_items(1)
            .with_max_leaf_characters(8);
        let failures = assert_that!(Token("a"))
            .with_renderer(LeafRenderer)
            .with_rendering_budget(budget)
            .capture(|it| it.matches(ForwardEvidence(budget)));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].children).has_length(1);
        assert_that!(failures[0].omitted_children).is_equal_to(2);
        let child = &failures[0].children[0];
        assert_that!(super::text_opt(child.actual.as_ref())).is_equal_to(Some("token(a)"));
        assert_that!(child.kind).is_equal_to(FailureKind::Predicate);
        assert_that!(child.relation.as_deref()).is_equal_to(Some("is rejected"));
        assert_that!(child.path).contains_exactly([PathSegment::Index(1)]);
        assert_that!(child.facts).is_empty();
    }
}

mod borrowed_views {
    use core::{borrow::Borrow, cmp::Ordering, fmt};

    use assertr::{
        borrow_for::BorrowFor,
        matchers::{
            eq, lt,
            range::{ContainsElement, DoesNotContainElement},
        },
        prelude::*,
    };

    #[derive(PartialEq, PartialOrd)]
    struct Measurement(i32);
    struct Operand(i32);
    impl Borrow<i32> for Operand {
        fn borrow(&self) -> &i32 {
            &self.0
        }
    }
    impl BorrowFor<Measurement> for Operand {
        type View = i32;
    }
    impl PartialEq<i32> for Measurement {
        fn eq(&self, other: &i32) -> bool {
            self.0 == *other
        }
    }
    impl PartialOrd<i32> for Measurement {
        fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
            self.0.partial_cmp(other)
        }
    }
    impl PartialEq<Measurement> for i32 {
        fn eq(&self, other: &Measurement) -> bool {
            *self == other.0
        }
    }
    impl PartialOrd<Measurement> for i32 {
        fn partial_cmp(&self, other: &Measurement) -> Option<Ordering> {
            self.partial_cmp(&other.0)
        }
    }
    struct Renderer;
    impl ValueRenderer<Measurement> for Renderer {
        fn fmt(&self, value: &Measurement, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "measurement({})", value.0)
        }
    }
    impl ValueRenderer<i32> for Renderer {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{value}")
        }
    }

    fn check_equal<T, E, R>(actual: &T, expected: E, renderer: R)
    where
        E: BorrowFor<T>,
        T: PartialEq<E::View>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        assert_that!(actual)
            .with_renderer(renderer)
            .is_equal_to(expected);
    }

    #[test]
    fn custom_views_work_in_generic_helpers_and_comparison_families() {
        check_equal(&Measurement(2), Operand(2), Renderer);
        check_equal(&String::from("hello"), "hello", DebugRenderer);
        let expected = Operand(2);
        let matcher = eq(expected);
        assert_that!(Measurement(2))
            .with_renderer(Renderer)
            .matches(&matcher);
        assert_that!(Measurement(2))
            .with_renderer(Renderer)
            .matches(&matcher);
        let ordering = lt(Operand(3));
        assert_that!(Measurement(2))
            .with_renderer(Renderer)
            .is_less_than(Operand(3))
            .matches(&ordering);
        assert_that!([Measurement(2)])
            .with_renderer(Renderer)
            .contains(Operand(2))
            .contains_exactly([Operand(2)]);
        let operands = vec![Operand(2)];
        let expected = matchers::collection::ContainsAll::new(&operands);
        assert_that!([Measurement(2)])
            .with_renderer(Renderer)
            .contains_all(&operands)
            .matches(&expected);
        assert_that!(vec![Measurement(2)])
            .with_renderer(Renderer)
            .matches(&expected);
        assert_that!(Measurement(1)..Measurement(3))
            .with_renderer(Renderer)
            .contains_element(Operand(2))
            .does_not_contain_element(Operand(3));
        let contained = ContainsElement::<Measurement>::borrowing(Operand(2));
        let excluded = DoesNotContainElement::<Measurement>::borrowing(Operand(3));
        assert_that!(Measurement(1)..Measurement(3))
            .with_renderer(Renderer)
            .matches(&contained)
            .matches(&excluded);
        let lower = Measurement(1);
        let upper = Measurement(3);
        assert_that!(&lower..&upper)
            .with_renderer(Renderer)
            .matches(&contained)
            .matches(&excluded);
    }

    #[test]
    fn diagnostics_render_the_actual_and_selected_view_without_wrapper_support() {
        let failures = assert_that!(Measurement(1))
            .with_renderer(Renderer)
            .with_location(false)
            .capture(|it| it.is_equal_to(Operand(2)));
        assert_that!(failures).has_length(1);
        let text = failures[0].to_string();
        assert_that!(text)
            .contains("measurement(1)")
            .contains("Expected: 2");
    }
}

mod map_query_operands {
    use core::{borrow::Borrow, fmt};
    use std::collections::BTreeMap;

    use assertr::{
        borrow_for::BorrowFor,
        matchers::{entry, eq},
        prelude::*,
    };

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct StoredKey(String);
    impl Borrow<str> for StoredKey {
        fn borrow(&self) -> &str {
            &self.0
        }
    }
    // The operand and stored key are deliberately opaque to renderers.
    struct Query<'a>(&'a str);
    impl Borrow<str> for Query<'_> {
        fn borrow(&self) -> &str {
            self.0
        }
    }
    impl BorrowFor<StoredKey> for Query<'_> {
        type View = str;
    }
    // Reference operands opt in separately. BorrowFor does not forward a wrapper's selection.
    impl Borrow<str> for &Query<'_> {
        fn borrow(&self) -> &str {
            self.0
        }
    }
    impl BorrowFor<StoredKey> for &Query<'_> {
        type View = str;
    }

    #[derive(Clone)]
    struct QueryRenderer;
    impl ValueRenderer<str> for QueryRenderer {
        fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "query({value})")
        }
    }
    impl ValueRenderer<i32> for QueryRenderer {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{value}")
        }
    }

    #[test]
    fn entry_requires_only_the_selected_query_and_nested_value_renderers() {
        let actual = BTreeMap::from([(StoredKey(String::from("a")), 1)]);
        let query = Query("a");
        assert_that!(actual)
            .with_renderer(QueryRenderer)
            .matches(entry(&query, eq(1)));
        let failures = assert_that!(actual)
            .with_renderer(QueryRenderer)
            .capture(|it| {
                it.matches(entry(Query("a"), eq(2)))
                    // A single native &str query needs no BorrowFor registration for &str.
                    .contains_entry_matching("a", eq(2))
            });
        assert_that!(failures).has_length(2);
        for failure in &failures {
            assert_that!(failure.to_string()).contains("At [query(a)]:");
        }
    }

    #[derive(Clone)]
    struct BulkRenderer;
    impl ValueRenderer<str> for BulkRenderer {
        fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            QueryRenderer.fmt(value, f)
        }
    }
    impl ValueRenderer<i32> for BulkRenderer {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            QueryRenderer.fmt(value, f)
        }
    }
    impl ValueRenderer<StoredKey> for BulkRenderer {
        fn fmt(&self, value: &StoredKey, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "stored({})", value.0)
        }
    }
    impl ValueRenderer<usize> for BulkRenderer {
        fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{value}")
        }
    }

    #[test]
    fn bulk_methods_use_registered_query_views_and_preserve_stored_evidence() {
        let actual = BTreeMap::from([(StoredKey(String::from("a")), 1)]);
        let failures = assert_that!(actual)
            .with_renderer(BulkRenderer)
            .capture(|it| {
                it.contains_keys([Query("missing")])
                    .contains_exactly_entries([(Query("a"), 2)])
                    .contains_exactly_entries_matching([entry(Query("missing"), eq(1))])
                    .contains_exactly_entries_satisfying([(
                        Query("a"),
                        |it: AssertThat<i32, Capture, BulkRenderer>| {
                            it.is_equal_to(2);
                        },
                    )])
            });
        assert_that!(failures).has_length(4);
        let membership = failures[0].to_string();
        assert_that!(membership)
            .contains("query(missing)")
            .contains("stored(a)");
        assert_that!(failures[1].to_string()).contains("At [query(a)]:");
        assert_that!(failures[2].to_string()).contains("At [stored(a)]:");
    }
}

/// Downstream subjects that implement only the public capability traits.
mod custom_capabilities {
    use assertr::prelude::*;

    struct NumericRenderer;

    impl ValueRenderer<usize> for NumericRenderer {
        fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(value, f)
        }
    }

    #[test]
    fn a_custom_collection_can_compare_borrowed_instances_without_a_renderer() {
        use core::borrow::Borrow;

        use assertr::{
            assertions::{Collection, HasLength, StableOrder},
            renderer::CollectionPresentation,
        };

        struct Key {
            _byte: u8,
        }
        struct Stored {
            key: Key,
        }
        impl Borrow<Key> for Stored {
            fn borrow(&self) -> &Key {
                &self.key
            }
        }
        struct Candidates([Stored; 2]);
        impl HasLength for Candidates {
            fn length(&self) -> usize {
                self.0.len()
            }
        }
        impl Collection for Candidates {
            type Item = Stored;
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
            fn elements(&self) -> impl Iterator<Item = &Stored> {
                self.0.iter()
            }
        }
        impl StableOrder for Candidates {}
        struct NoRenderer;

        let candidates = Candidates([
            Stored {
                key: Key { _byte: 1 },
            },
            Stored {
                key: Key { _byte: 1 },
            },
        ]);
        let other = Key { _byte: 1 };
        let expected = [&candidates.0[0].key, &candidates.0[1].key];
        assert_that!(expected[0])
            .with_renderer(NoRenderer)
            .is_same_instance_as(expected[0])
            .is_not_same_instance_as(expected[1]);
        assert_that!(candidates)
            .with_renderer(NumericRenderer)
            .contains_same_instance_as(expected[0])
            .does_not_contain_same_instance_as(&other)
            .contains_exactly_same_instances(expected)
            .contains_exactly_same_instances_in_any_order([expected[1], expected[0]]);
        let failures = assert_that!(candidates)
            .with_renderer(NoRenderer)
            .capture(|it| it.contains_same_instance_as(&other));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn a_custom_collection_gets_every_collection_assertion() {
        use assertr::{
            assertions::{Collection, HasLength, RandomAccess, StableOrder},
            renderer::CollectionPresentation,
        };

        /// A downstream collection type, implementing only the extension traits.
        #[derive(Debug)]
        struct Ring(Vec<i32>);

        impl HasLength for Ring {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl Collection for Ring {
            type Item = i32;
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

            fn elements(&self) -> impl Iterator<Item = &i32> {
                self.0.iter()
            }
        }

        impl StableOrder for Ring {}

        impl RandomAccess for Ring {
            fn element_at(&self, index: usize) -> Option<&i32> {
                self.0.get(index)
            }
        }

        let ring = Ring(vec![1, 2, 3]);
        assert_that!(ring)
            .contains(2)
            .does_not_contain(4)
            .starts_with([1, 2])
            .ends_with([2, 3])
            .contains_contiguous([1, 2])
            .contains_exactly([1, 2, 3])
            .contains_exactly_in_any_order([3, 2, 1])
            .has_length(3);
        assert_that!(ring).first().is_equal_to(1);
        assert_that!(ring).last().is_equal_to(3);
        assert_that!(ring).at(1).is_equal_to(2);
        assert_that!(Ring(vec![42])).single().is_equal_to(42);

        let failures = assert_that!(Ring(vec![1, 2, 3]))
            .with_location(false)
            .capture(|it| it.contains(4));
        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(ToString::to_string)
                    .contains("Actual: [");
            },
        ]);

        #[cfg(feature = "fluent")]
        {
            let mut ring = Ring(vec![1, 2, 3]);
            (&mut ring)
                .must()
                .contain(2)
                .contain_exactly([1, 2, 3])
                .have_length(3);
        }
    }

    #[test]
    fn a_custom_bag_gets_only_order_free_collection_assertions() {
        use assertr::{
            assertions::{Collection, HasLength},
            renderer::{CollectionPresentation, RenderingOrder},
        };

        #[derive(Debug)]
        struct Multiset(Vec<i32>);

        impl HasLength for Multiset {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl Collection for Multiset {
            type Item = i32;
            const PRESENTATION: CollectionPresentation = CollectionPresentation::list()
                .with_type_hint(true)
                .with_order(RenderingOrder::SortByRenderedText);

            fn elements(&self) -> impl Iterator<Item = &i32> {
                self.0.iter()
            }
        }

        assert_that!(Multiset(vec![2, 1, 1]))
            .contains(1)
            .contains_exactly_in_any_order([1, 2, 1])
            .has_length(3);

        let failures = assert_that!(Multiset(vec![2, 1, 1]))
            .with_location(false)
            .capture(|it| it.contains(3));
        assert_that!(failures[0].to_string())
            .contains("Actual: Multiset [")
            .contains("(sorted for rendering)");
    }

    #[test]
    fn a_custom_set_gets_every_set_and_collection_assertion() {
        use assertr::{
            assertions::{Collection, HasLength, SetLookup},
            renderer::CollectionPresentation,
        };

        /// A downstream set type, implementing only the extension traits.
        #[derive(Debug)]
        struct CustomSet(Vec<i32>);

        impl HasLength for CustomSet {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl Collection for CustomSet {
            type Item = i32;
            const PRESENTATION: CollectionPresentation =
                CollectionPresentation::set().with_type_hint(true);

            fn elements(&self) -> impl Iterator<Item = &i32> {
                self.0.iter()
            }
        }

        impl SetLookup for CustomSet {
            fn contains_element(&self, element: &i32) -> bool {
                self.0.contains(element)
            }
        }

        let set = CustomSet(vec![1, 2, 3]);
        assert_that!(set)
            .contains(2)
            .does_not_contain(4)
            .contains_all([1, 3])
            .contains_exactly_in_any_order([3, 2, 1])
            .is_subset_of(CustomSet(vec![1, 2, 3, 4]))
            .is_superset_of(CustomSet(vec![1]))
            .is_disjoint_from(CustomSet(vec![9]))
            .has_length(3);

        let failures = assert_that!(CustomSet(vec![1, 2, 3]))
            .with_location(false)
            .capture(|it| it.contains(4));
        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(ToString::to_string)
                    .contains("Actual: CustomSet {");
            },
        ]);

        let relation_failures = assert_that!(CustomSet(vec![1, 2]))
            .with_location(false)
            .capture(|it| it.is_subset_of(CustomSet(vec![1])));
        assert_that!(&relation_failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(ToString::to_string)
                    .contains("Actual: CustomSet {")
                    .contains("Expected: CustomSet {");
            },
        ]);

        #[cfg(feature = "fluent")]
        {
            let mut set = CustomSet(vec![1, 2, 3]);
            (&mut set)
                .must()
                .contain(2)
                .be_subset_of(CustomSet(vec![1, 2, 3, 4]))
                .have_length(3);
        }
    }

    #[test]
    fn a_custom_map_gets_every_map_assertion() {
        use core::borrow::Borrow;
        use std::collections::BTreeMap;

        use assertr::{
            assertions::{HasLength, Map, MapLookup},
            renderer::RenderingOrder,
        };

        /// A downstream map type, implementing only the extension traits.
        #[derive(Debug)]
        struct Config(BTreeMap<String, i32>);

        impl HasLength for Config {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl Map for Config {
            type Key = String;
            type Value = i32;
            const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

            fn entries(&self) -> impl Iterator<Item = (&String, &i32)> {
                self.0.iter()
            }
        }

        /// One generic impl carrying the wrapped map's own lookup bounds.
        impl<Q> MapLookup<Q> for Config
        where
            Q: Ord + ?Sized,
            String: Borrow<Q>,
        {
            fn get_key_value(&self, key: &Q) -> Option<(&String, &i32)> {
                self.0.get_key_value(key)
            }
        }

        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_three(value: &i32) -> bool {
            *value == 3
        }

        fn satisfies_three(it: AssertThat<i32, Capture>) {
            it.is_equal_to(3);
        }

        let config = Config(BTreeMap::from([("retries".to_owned(), 3)]));
        assert_that!(config)
            .contains_key("retries")
            .does_not_contain_key("timeout")
            .contains_value(3)
            .does_not_contain_value(9)
            .contains_entry("retries", 3)
            .contains_entry_satisfying("retries", satisfies_three)
            .contains_keys(["retries"])
            .contains_exactly_entries([("retries", 3)])
            .contains_exactly_entries_matching(
                [("retries", is_three)]
                    .map(|(key, p)| matchers::entry(key, matchers::predicate(p))),
            )
            .contains_exactly_entries_satisfying([("retries", satisfies_three)])
            .has_length(1);

        #[cfg(feature = "fluent")]
        {
            let mut config = Config(BTreeMap::from([("retries".to_owned(), 3)]));
            (&mut config)
                .must()
                .contain_key("retries")
                .contain_entry("retries", 3)
                .have_length(1);
        }
    }
}
