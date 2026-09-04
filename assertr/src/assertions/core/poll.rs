use core::task::Poll;

use crate::actual::Actual;
use crate::failure::FailureKind;
use crate::mode::{Mode, Panic};
use crate::{AssertThat, ValueRenderer};

/// Raises the failure of an assertion that found `Pending` where it expected `Ready`.
#[track_caller]
fn fail_pending<T, M: Mode, R>(this: &AssertThat<'_, Poll<T>, M, R>) {
    this.failure(FailureKind::Variant)
        .actual(format_args!("Pending"))
        .relation("is not the expected variant")
        .expected(format_args!("Poll::Ready"))
        .raise();
}

/// Non-extracting assertions for `Poll` subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PollAssertions<'t, T, M: Mode, R> {
    /// Asserts that the subject is `Ready`.
    ///
    /// Non-extracting: the subject stays the full `Poll`, so further assertions can be chained
    /// in any mode. Use [`PollExtractAssertions::get_ready`] to extract the contained value in
    /// panic mode, or [`PollAssertions::is_ready_satisfying`] to assert on it in any mode.
    fn is_ready(self) -> Self;

    /// Asserts that the subject is `Pending`.
    fn is_pending(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject is `Ready`, then runs `assertions` on its value.
    ///
    /// The closure receives an `AssertThat<T>` borrowing the contained value.
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>);
}

impl<'t, T, M: Mode, R> PollAssertions<'t, T, M, R> for AssertThat<'t, Poll<T>, M, R> {
    #[track_caller]
    fn is_ready(self) -> Self {
        self.track_assertion();
        if !self.actual().is_ready() {
            fail_pending(&self);
        }
        self
    }

    #[track_caller]
    fn is_pending(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_pending() {
            let actual = match actual {
                Poll::Ready(value) => self.render().variant(actual, "Ready", value),
                Poll::Pending => unreachable!("already checked"),
            };
            self.failure(FailureKind::Variant)
                .actual(actual)
                .relation("is not the expected variant")
                .expected(format_args!("Poll::Pending"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        R: Clone,
        A: for<'a> FnOnce(AssertThat<'a, T, M, R>),
    {
        self.track_assertion();
        if self.actual().is_ready() {
            self.satisfies(
                |it| match it {
                    Poll::Ready(t) => t,
                    Poll::Pending => unreachable!("already checked"),
                },
                assertions,
            )
        } else {
            fail_pending(&self);
            self
        }
    }
}

/// Panic-mode extraction from `Poll` subjects.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PollExtractAssertions<'t, T, R> {
    /// Asserts that the subject is `Ready`, then returns an assertion over its value.
    ///
    /// A borrowed subject yields a borrowed value. An owned subject yields an owned value.
    ///
    /// This is available only in `Panic` mode because `Pending` cannot produce a `T`. Use
    /// [`PollAssertions::is_ready_satisfying`] for capture mode, or the
    /// non-extracting [`PollAssertions::is_ready`] when the contained value is irrelevant.
    fn get_ready(self) -> AssertThat<'t, T, Panic, R>;
}

impl<'t, T, R> PollExtractAssertions<'t, T, R> for AssertThat<'t, Poll<T>, Panic, R> {
    #[track_caller]
    fn get_ready(self) -> AssertThat<'t, T, Panic, R> {
        self.track_assertion();
        if !self.actual().is_ready() {
            fail_pending(&self);
        }
        self.map(|it| match it {
            Actual::Owned(p) => Actual::Owned(match p {
                Poll::Ready(t) => t,
                Poll::Pending => unreachable!("already checked"),
            }),
            Actual::Borrowed(p) => Actual::Borrowed(match p {
                Poll::Ready(t) => t,
                Poll::Pending => unreachable!("already checked"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use core::task::Poll;

        struct Secret;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Poll<()>, Panic, NoRenderer>
                    => PollAssertions<'static, (), Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, Poll<()>, Panic, NoRenderer>
                    => PollExtractAssertions<'static, (), NoRenderer>
            );
        }

        #[test]
        fn ready_variant_is_rendered_from_its_leaf_value() {
            let failures = assert_that!(Poll::Ready(Secret))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(PollAssertions::is_pending);

            assert_that!(TextReporter.report(&failures[0]))
                .contains("Ready(")
                .contains(SENTINEL);
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Foo {
        val: u32,
    }

    mod is_ready {
        use super::Foo;
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Poll::Ready(Foo { val: 42 }).must().be_ready();
        }

        #[test]
        fn succeeds_when_ready_and_retains_the_subject() {
            assert_that!(Poll::Ready(Foo { val: 42 }))
                .is_ready()
                .is_equal_to(Poll::Ready(Foo { val: 42 }));
        }

        #[test]
        fn panics_when_not_ready() {
            assert_that_panic_by(|| {
                assert_that!(Poll::<Foo>::Pending)
                    .with_location(false)
                    .is_ready();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::<Foo>::Pending`

                Actual: Pending

                is not the expected variant

                Expected: Poll::Ready
                -------- assertr --------
            "});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(Poll::<i32>::Pending)
                .with_location(false)
                .capture(|it| it.is_ready().is_pending());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Poll::<i32>::Pending`

                        Actual: Pending

                        is not the expected variant

                        Expected: Poll::Ready
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }

    mod get_ready {
        use super::Foo;
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Poll::Ready(42).must().get_ready().is_equal_to(42);
        }

        #[test]
        fn extracts_the_borrowed_inner_value() {
            let poll = Poll::Ready(Foo { val: 42 });

            assert_that!(poll).get_ready().is_equal_to(Foo { val: 42 });

            // The poll was only borrowed and remains usable.
            assert_that!(poll).is_ready();
        }

        #[test]
        fn extracts_the_owned_inner_value() {
            assert_that_owned!(Poll::Ready(Foo { val: 42 }))
                .get_ready()
                .is_equal_to(Foo { val: 42 });
        }

        #[test]
        fn panics_when_not_ready() {
            assert_that_panic_by(|| {
                assert_that!(Poll::<Foo>::Pending)
                    .with_location(false)
                    .get_ready();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::<Foo>::Pending`

                Actual: Pending

                is not the expected variant

                Expected: Poll::Ready
                -------- assertr --------
            "});
        }
    }

    mod is_ready_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Poll::Ready(42).must().be_ready_satisfying(|ready| {
                ready.is_equal_to(42);
            });
        }

        #[test]
        fn succeeds_when_ready_and_assertions_pass() {
            assert_that!(Poll::Ready(42)).is_ready_satisfying(|ready| {
                ready.is_equal_to(42);
            });
        }

        #[test]
        fn hands_out_a_value_typed_assertion_supporting_type_specific_assertions() {
            assert_that!(Poll::Ready(String::from("value"))).is_ready_satisfying(|ready| {
                ready.contains("alu").starts_with("v");
            });
        }

        #[test]
        fn captures_inner_failure_when_ready_and_assertion_fails() {
            let failures = assert_that!(Poll::Ready(42))
                .with_location(false)
                .capture(|it| {
                    it.is_ready_satisfying(|ready| {
                        ready.is_greater_than(9000);
                    })
                });

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {"
                        -------- assertr --------
                        Actual: 42

                        is not greater than

                        Expected: 9000
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn captures_variant_failure_when_pending() {
            let failures = assert_that!(Poll::<i32>::Pending)
                .with_location(false)
                .capture(|it| it.is_ready_satisfying(|_| panic!("assertions should not run")));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r"
                        -------- assertr --------
                        Expression: `Poll::<i32>::Pending`

                        Actual: Pending

                        is not the expected variant

                        Expected: Poll::Ready
                        -------- assertr --------
                    "});
                },
            ]);
        }

        #[test]
        fn panics_when_pending() {
            assert_that_panic_by(|| {
                assert_that!(Poll::<i32>::Pending)
                    .with_location(false)
                    .is_ready_satisfying(|_| panic!("assertions should not run"));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::<i32>::Pending`

                Actual: Pending

                is not the expected variant

                Expected: Poll::Ready
                -------- assertr --------
            "});
        }
    }

    mod is_pending {
        use super::Foo;
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Poll::<Foo>::Pending.must().be_pending();
        }

        #[test]
        fn succeeds_when_pending() {
            assert_that!(Poll::<Foo>::Pending).is_pending();
        }

        #[test]
        fn panics_when_ready() {
            assert_that_panic_by(|| {
                assert_that!(Poll::Ready(Foo { val: 42 }))
                    .with_location(false)
                    .is_pending();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `Poll::Ready(Foo {{ val: 42 }})`

                Actual: Ready(
                    Foo {{
                        val: 42,
                    }},
                )

                is not the expected variant

                Expected: Poll::Pending
                -------- assertr --------
            "});
        }
    }
}
