use core::fmt::{Debug, Write};
use core::task::Poll;
use indoc::writedoc;

use crate::AssertThat;
use crate::actual::Actual;
use crate::mode::{Mode, Panic};
use crate::tracking::AssertionTracking;

/// Non-extracting assertions for `Poll` values.
/// These work in any mode (Panic or Capture).
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PollAssertions<'t, T, M: Mode> {
    fn is_pending(self) -> Self;
}

impl<'t, T: Debug, M: Mode> PollAssertions<'t, T, M> for AssertThat<'t, Poll<T>, M> {
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
