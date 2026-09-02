use crate::mode::Panic;
use crate::prelude::*;
use alloc::string::String;
use core::borrow::Borrow;
use core::fmt::Write;
use indoc::writedoc;

/// Non-extracting assertions for [`tokio::sync::watch::Receiver`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioWatchReceiverAssertions<T, R = crate::DebugRenderer> {
    /// Asserts that the receiver's current value equals `expected` without marking it seen.
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq,
        R: ValueRenderer<T>;
}

impl<T, M: Mode, R> TokioWatchReceiverAssertions<T, R>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, M, R>
{
    #[track_caller]
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        let actual = tokio::sync::watch::Receiver::borrow(self.actual());
        let expected = expected.borrow();
        if *actual != *expected {
            let actual = self.render_value(&*actual);
            let expected = self.render_value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
        }
        drop(actual);
        self
    }
}

/// Panic-mode assertions over a watch receiver's change state.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait TokioWatchReceiverExtractAssertions<T, R = crate::DebugRenderer> {
    /// Asserts that the current value has not been seen by this receiver.
    ///
    /// A closed channel fails with its [`RecvError`](tokio::sync::watch::error::RecvError).
    fn has_changed(self) -> Self
    where
        R: ValueRenderer<tokio::sync::watch::error::RecvError> + ValueRenderer<bool> + Clone;

    /// Asserts that the current value has already been seen by this receiver.
    ///
    /// A closed channel fails with its [`RecvError`](tokio::sync::watch::error::RecvError).
    fn has_not_changed(self) -> Self
    where
        R: ValueRenderer<tokio::sync::watch::error::RecvError> + ValueRenderer<bool> + Clone;
}

impl<T, R> TokioWatchReceiverExtractAssertions<T, R>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, Panic, R>
{
    #[track_caller]
    fn has_changed(self) -> Self
    where
        R: ValueRenderer<tokio::sync::watch::error::RecvError> + ValueRenderer<bool> + Clone,
    {
        self.derive_owned(tokio::sync::watch::Receiver::has_changed)
            .with_detail_message("Expected a tokio `watch` channel to have changed.")
            .get_ok()
            .is_true();
        self
    }

    #[track_caller]
    fn has_not_changed(self) -> Self
    where
        R: ValueRenderer<tokio::sync::watch::error::RecvError> + ValueRenderer<bool> + Clone,
    {
        self.derive_owned(tokio::sync::watch::Receiver::has_changed)
            .with_detail_message("Expected a tokio `watch` channel to have not changed.")
            .get_ok()
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
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let (_tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.must().have_current_value(Person { name: "bob".into() });
        }

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
                    Expression: `rx`

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
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();
            rx.must().have_changed();
        }

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
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: true

                      Actual: false

                    Details: [
                        Expected a tokio `watch` channel to have changed.,
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod has_not_changed {
        use super::Person;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();
            rx.must().not_have_changed();
        }

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
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: false

                      Actual: true

                    Details: [
                        Expected a tokio `watch` channel to have not changed.,
                    ]
                    -------- assertr --------
                "});
        }
    }
}
