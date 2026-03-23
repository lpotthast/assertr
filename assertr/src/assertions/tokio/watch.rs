use crate::mode::Panic;
use crate::prelude::*;
use core::borrow::Borrow;
use core::fmt::Debug;
use std::ops::Deref;

/// Non-extracting assertions for the `tokio::sync::watch::Receiver` type.
/// These work in any mode (Panic or Capture).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioWatchReceiverAssertions<T: Debug> {
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq;
}

impl<T: Debug, M: Mode> TokioWatchReceiverAssertions<T>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, M>
{
    #[track_caller]
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq,
    {
        self.derive(|it| it.borrow())
            .derive(Deref::deref)
            .is_equal_to(expected.borrow());
        self
    }
}

/// Assertions for tokio watch receivers that use data-extracting operations internally.
/// Only available in Panic mode.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioWatchReceiverExtractAssertions<T: Debug> {
    fn has_changed(self) -> Self;

    fn has_not_changed(self) -> Self;
}

impl<T: Debug> TokioWatchReceiverExtractAssertions<T>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, Panic>
{
    fn has_changed(self) -> Self {
        self.derive(tokio::sync::watch::Receiver::has_changed)
            .with_detail_message("Expected a tokio `watch` channel to have changed.")
            .is_ok()
            .is_true();
        self
    }

    fn has_not_changed(self) -> Self {
        self.derive(tokio::sync::watch::Receiver::has_changed)
            .with_detail_message("Expected a tokio `watch` channel to have not changed.")
            .is_ok()
            .is_false();
        self
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq)]
    struct Person {
        name: String,
    }

    mod has_current_value {
        use super::Person;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[tokio::test]
        async fn succeeds_when_equal() {
            let (tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            tx.send(Person {
                name: "kevin".into(),
            })
            .unwrap();

            assert_that!(rx).has_current_value(Person {
                name: "kevin".into(),
            });
        }

        #[tokio::test]
        async fn panics_when_not_equal() {
            let (_tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });

            assert_that_panic_by(|| {
                assert_that!(rx)
                    .with_location(false)
                    .has_current_value(Person {
                        name: "alice".into(),
                    })
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: Person {{
                        name: "alice",
                    }}

                      Actual: Person {{
                        name: "bob",
                    }}
                    -------- assertr --------
                "#});
        }
    }

    mod has_changed {
        use super::Person;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[tokio::test]
        async fn succeeds_when_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();

            assert_that!(rx).has_changed();
        }

        #[tokio::test]
        async fn panics_when_not_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: true

                      Actual: false

                    Details: [
                        Expected a tokio `watch` channel to have changed.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod has_not_changed {
        use super::Person;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[tokio::test]
        async fn succeeds_when_not_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();

            assert_that!(rx).has_not_changed();
        }

        #[tokio::test]
        async fn panics_when_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_not_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: false

                      Actual: true

                    Details: [
                        Expected a tokio `watch` channel to have not changed.,
                    ]
                    -------- assertr --------
                "#});
        }
    }
}
