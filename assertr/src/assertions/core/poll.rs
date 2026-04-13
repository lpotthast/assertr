use alloc::string::String;
use core::fmt::{Debug, Write};
use core::task::Poll;
use indoc::writedoc;

use crate::AssertThat;
use crate::actual::Actual;
use crate::mode::{Mode, Panic};
use crate::tracking::AssertionTracking;

/// Non-extracting assertions for `Poll` values.
/// These work in any mode (Panic or Capture).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PollAssertions<'t, T, M: Mode> {
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>);

    fn is_pending(self) -> Self;
}

impl<'t, T: Debug, M: Mode> PollAssertions<'t, T, M> for AssertThat<'t, Poll<T>, M> {
    #[track_caller]
    fn is_ready_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>),
    {
        self.track_assertion();
        let actual = self.actual();
        if actual.is_ready() {
            self.satisfies_ref(
                |it| match it {
                    Poll::Ready(t) => t,
                    Poll::Pending => unreachable!("already checked"),
                },
                assertions,
            )
        } else {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not yet ready.
                "}
            });
            self
        }
    }

    #[track_caller]
    fn is_pending(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_pending() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not pending.
                "}
            });
        }
        self
    }
}

/// Data-extracting assertion for `Poll` values.
/// Only available in Panic mode, as the extracted `T` cannot be produced when the poll is pending.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PollExtractAssertions<'t, T> {
    /// Use `PollAssertions::is_ready_satisfying` for capture mode.
    fn is_ready(self) -> AssertThat<'t, T, Panic>;
}

impl<'t, T: Debug> PollExtractAssertions<'t, T> for AssertThat<'t, Poll<T>, Panic> {
    #[track_caller]
    fn is_ready(self) -> AssertThat<'t, T, Panic> {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_ready() {
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not yet ready.
                "}
            });
        }
        self.map(|it| match it {
            Actual::Owned(p) => Actual::Owned(match p {
                Poll::Ready(t) => t,
                Poll::Pending => panic!("is pending"),
            }),
            Actual::Borrowed(p) => Actual::Borrowed(match p {
                Poll::Ready(t) => t,
                Poll::Pending => panic!("is pending"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
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
        fn succeeds_when_ready() {
            assert_that!(Poll::Ready(Foo { val: 42 }))
                .is_ready()
                .is_equal_to(Foo { val: 42 });
        }

        #[test]
        fn panics_when_not_ready() {
            assert_that_panic_by(|| {
                assert_that!(Poll::<Foo>::Pending)
                    .with_location(false)
                    .is_ready();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: Pending
                
                is not yet ready.
                -------- assertr --------
            "#});
        }
    }

    mod is_ready_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

        #[test]
        fn succeeds_when_ready_and_assertions_pass() {
            assert_that!(Poll::Ready(42)).is_ready_satisfying(|ready| {
                ready.is_equal_to(&42);
            });
        }

        #[test]
        fn captures_inner_failure_when_ready_and_assertion_fails() {
            let failures = assert_that!(Poll::Ready(42))
                .with_capture()
                .with_location(false)
                .is_ready_satisfying(|ready| {
                    ready.is_greater_than(&9000);
                })
                .capture_failures();

            assert_that!(failures).contains_exactly::<String>([formatdoc! {"
                -------- assertr --------
                Actual: 42

                is not greater than

                Expected: 9000
                -------- assertr --------
            "}]);
        }

        #[test]
        fn captures_variant_failure_when_pending() {
            let failures = assert_that!(Poll::<i32>::Pending)
                .with_capture()
                .with_location(false)
                .is_ready_satisfying(|_| panic!("assertions should not run"))
                .capture_failures();

            assert_that!(failures).contains_exactly::<String>([formatdoc! {r#"
                -------- assertr --------
                Actual: Pending
                
                is not yet ready.
                -------- assertr --------
            "#}]);
        }

        #[test]
        fn panics_when_pending() {
            assert_that_panic_by(|| {
                assert_that!(Poll::<i32>::Pending)
                    .with_location(false)
                    .is_ready_satisfying(|_| panic!("assertions should not run"));
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: Pending
                
                is not yet ready.
                -------- assertr --------
            "#});
        }
    }

    mod is_pending {
        use super::Foo;
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::task::Poll;

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
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: Ready(
                    Foo {{
                        val: 42,
                    }},
                )
                
                is not pending.
                -------- assertr --------
            "#});
        }
    }
}
