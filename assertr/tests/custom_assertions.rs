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
// is the spelling users write and the one the documentation shows. Passing the method path instead
// would be shorter but would stop demonstrating the API.
#![allow(clippy::redundant_closure_for_method_calls)]

fn text_opt(value: Option<&assertr::renderer::Rendered>) -> Option<&str> {
    value.map(|value| match &value.body {
        assertr::renderer::RenderedBody::Text { text, .. } => text.as_str(),
        body => panic!("expected a text node, got {body:?}"),
    })
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

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| ToHumanReadableText.render(value))
                    .contains("Expected: 30")
                    .contains("Actual: 12");
            },
        ]);
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
            // Tracking comes first and happens unconditionally: a passing assertion must count just
            // as much as a failing one.
            self.track_assertion();

            let age = self.actual().age;
            if age < 18 {
                // A failure that renders no value needs no renderer capability.
                self.failure(FailureKind::Ordering)
                    .relation("is not an adult")
                    .fact(Fact::labelled("Age", age))
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
                // Facts belong to the failure, not to the chain: they must not reappear in a later
                // failure of the same chain.
                self.failure(FailureKind::Ordering)
                    .actual(self.render().value(actual))
                    .relation("is not older than")
                    .expected(self.render().value(other))
                    .fact(Fact::labelled("Actual age", actual.age))
                    .fact(Fact::labelled("Expected age", other.age))
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
                    .contains_exactly([Fact::labelled("Age", "12")]);
                element
                    .derive_owned(|value| ToHumanReadableText.render(value))
                    .contains(formatdoc! {"
            -------- assertr --------
            Subject: child
            Expression: `person(12)`

            is not an adult

            Details:
              - Age: 12
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
                        Fact::labelled("Actual age", "12"),
                        Fact::labelled("Expected age", "40"),
                    ]);
            },
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .contains_exactly([Fact::labelled("Age", "12")]);
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
        assert_that!(ToHumanReadableText.render(&failures[0])).contains(formatdoc! {"
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
        // Without `track_assertion`, `capture` would treat this closure as empty and panic. This is
        // why the tracking hook is public rather than internal.
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

    /// A downstream assertion over a group of people that reports each rejected member as a nested
    /// failure located by its index, the way the built-in positional assertions do.
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

        assert_that!(failures).contains_exactly_satisfying([
            |failure: AssertThat<AssertionFailure, Capture>| {
                failure
                    .derive(|failure| &failure.children)
                    .contains_exactly_satisfying([
                        |child: AssertThat<AssertionFailure, Capture>| {
                            child
                                .derive(|child| &child.kind)
                                .is_equal_to(FailureKind::Ordering);
                            child
                                .derive_owned(|child| super::text_opt(child.actual.as_ref()))
                                .is_equal_to(Some("Person(age=12)"));
                            child
                                .derive(|child| &child.facts)
                                .contains_exactly([Fact::index(1)]);
                        },
                    ]);
                failure
                    .derive_owned(|failure| ToHumanReadableText.render(failure))
                    .is_equal_to(formatdoc! {"
            -------- assertr --------
            Expression: `vec![person(30), person(12)]`

            contains people who are not adults

            Nested failures:
              - At index 1:
                Actual: Person(age=12)

                is not an adult
            -------- assertr --------
            "});
            },
        ]);
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
        assert_that!(values).is_equal_to(["first", "second"]);
    }
}

mod matcher_authoring {
    use assertr::{
        matchers::{ConstraintDescription, MatchContext, MatchResult},
        prelude::*,
    };

    struct AgeAtLeast(u32);
    impl<R: ValueRenderer<u32>> AssertrMatcher<super::Person, R> for AgeAtLeast {
        fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
            ConstraintDescription::new("has at least the required age")
                .expected(context.render().value(&self.0))
        }
        fn evaluate(
            &self,
            actual: &super::Person,
            context: &mut MatchContext<'_, R>,
        ) -> MatchResult {
            context.scoped(assertr::failure::PathSegment::Field("age"), |context| {
                assertr::matchers::ge(self.0).evaluate(&actual.age, context)
            })
        }
    }
    struct AgeRenderer;
    impl ValueRenderer<u32> for AgeRenderer {
        fn fmt(&self, value: &u32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "age={value}")
        }
    }
    #[test]
    fn downstream_matchers_compose_without_parent_rendering() {
        let person = super::Person {
            age: 12,
            meta: super::Metadata { alive: true },
        };
        let matcher = assertr::matchers::all_of((AgeAtLeast(18), AgeAtLeast(21)));
        let failures = assert_that!(person)
            .with_renderer(AgeRenderer)
            .capture(|it| it.matches(&matcher));
        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive(|value| &value.children)
                    .contains_exactly_satisfying(
                        [|child: AssertThat<AssertionFailure, Capture>| {
                            child
                                .derive_owned(|child| super::text_opt(child.actual.as_ref()))
                                .is_equal_to(Some("age=12"));
                        }; 2],
                    );
            },
        ]);
        assert_that!(person)
            .with_renderer(AgeRenderer)
            .does_not_match(&matcher);
    }
}

// These helpers deliberately take `T` by value to model the signature available to downstream
// generic code, rather than proving the assertion bounds only for `&T`.
#[allow(clippy::needless_pass_by_value)]
#[cfg(feature = "num")]
mod generic_num_traits_bounds {
    use core::fmt::Debug;
    use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

    use num_traits::{Num, One, Signed, Zero};

    use assertr::assertions::num::NumericDistance;
    use assertr::prelude::*;

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
        assert_that!(zero).is_zero().is_additive_identity();
        assert_that!(one).is_one().is_multiplicative_identity();
    }

    fn assert_close_to<T: NumericDistance + Debug>(value: T, expected: T, deviation: T) {
        assert_that!(value).is_close_to(expected, deviation);
    }

    fn assert_sign<T: Num + Signed + Debug>(negative: T, positive: T) {
        assert_that!(negative).is_negative();
        assert_that!(positive).is_positive();
    }

    #[test]
    fn numeric_trait_is_available_without_renderer_support() {
        struct NoRenderer;
        fn accepts_numeric_assertions<T: Num, A: NumAssertions<T>>(_: &A) {}

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
