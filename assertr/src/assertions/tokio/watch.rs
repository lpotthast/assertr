use crate::borrow_for::{BorrowFor, borrow_for};
use crate::failure::FailureKind;
use crate::prelude::*;
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};

/// Compares the current watch value without marking it seen.
pub struct HasCurrentValue<E>(E);
impl<T, E, R> Expectation<tokio::sync::watch::Receiver<T>, R> for HasCurrentValue<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    type Rejection<'a>
        = (tokio::sync::watch::Ref<'a, T>, &'a E::View)
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a tokio::sync::watch::Receiver<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let actual = tokio::sync::watch::Receiver::borrow(actual);
        let expected = borrow_for::<T, _>(&self.0);
        if *actual == *expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}
impl<T, E, R> ExpectationDiagnostics<tokio::sync::watch::Receiver<T>, R> for HasCurrentValue<E>
where
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a tokio::sync::watch::Receiver<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has the current value")
                .expected(render.value(borrow_for::<T, _>(&self.0))),
            Some((_, (actual, expected))) => failure
                .actual(render.value(&*actual))
                .expected(render.value(expected)),
        }
    }
}
impl<E> HasCurrentValue<E> {
    /// Expects this current value using its borrowed comparison view.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}
/// Checks whether the receiver has changed, rejecting closed channels.
pub struct HasChanged;
impl<T, R> Expectation<tokio::sync::watch::Receiver<T>, R> for HasChanged {
    type Success<'a>
        = ()
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    type Rejection<'a>
        = bool
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a tokio::sync::watch::Receiver<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual.has_changed() {
            Ok(true) => Ok(()),
            Ok(_) => Err(false),
            Err(_) => Err(true),
        }
    }
}
impl<T, R> ExpectationDiagnostics<tokio::sync::watch::Receiver<T>, R> for HasChanged {
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a tokio::sync::watch::Receiver<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("has changed"),
            Some((_, closed)) => failure.relation(if closed {
                "is closed"
            } else {
                "has not changed"
            }),
        }
    }
}
/// Checks whether the receiver has not changed, rejecting closed channels.
pub struct HasNotChanged;
impl<T, R> Expectation<tokio::sync::watch::Receiver<T>, R> for HasNotChanged {
    type Success<'a>
        = ()
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    type Rejection<'a>
        = bool
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a tokio::sync::watch::Receiver<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual.has_changed() {
            Ok(false) => Ok(()),
            Ok(_) => Err(false),
            Err(_) => Err(true),
        }
    }
}
impl<T, R> ExpectationDiagnostics<tokio::sync::watch::Receiver<T>, R> for HasNotChanged {
    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a tokio::sync::watch::Receiver<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        _context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure.relation("has not changed"),
            Some((_, closed)) => failure.relation(if closed {
                "is closed"
            } else {
                "has unexpectedly changed"
            }),
        }
    }
}

/// Non-extracting assertions for [`tokio::sync::watch::Receiver`].
///
/// These checks support panic and capture modes without changing the receiver's seen state.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioWatchReceiverAssertions<T, R = crate::DebugRenderer> {
    /// Asserts that the receiver's current value equals `expected` without marking it seen.
    fn has_current_value<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>;

    /// Asserts that the current value has not been seen by this receiver.
    ///
    /// A closed channel fails this assertion. The value is not marked seen.
    fn has_changed(self) -> Self;

    /// Asserts that the current value has already been seen by this receiver.
    ///
    /// A closed channel fails this assertion. The value is not marked seen.
    fn has_not_changed(self) -> Self;
}

impl<T, M: Mode, R> TokioWatchReceiverAssertions<T, R>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, M, R>
{
    #[track_caller]
    fn has_current_value<E>(self, expected: E) -> Self
    where
        T: PartialEq<E::View>,
        E: BorrowFor<T>,
        R: ValueRenderer<T> + ValueRenderer<E::View>,
    {
        self.apply_assertion(HasCurrentValue::new(expected))
    }

    #[track_caller]
    fn has_changed(self) -> Self {
        self.apply_assertion(HasChanged)
    }

    #[track_caller]
    fn has_not_changed(self) -> Self {
        self.apply_assertion(HasNotChanged)
    }
}

#[cfg(test)]
mod tests {
    mod observations {
        use crate::prelude::*;
        use core::{borrow::Borrow, cell::Cell};

        struct Expected<'a>(&'a Cell<usize>);
        impl crate::borrow_for::BorrowFor<i32> for Expected<'_> {
            type View = i32;
        }
        impl Borrow<i32> for Expected<'_> {
            fn borrow(&self) -> &i32 {
                self.0.set(self.0.get() + 1);
                &9
            }
        }

        #[test]
        fn value_rejection_retains_the_expected_borrow_without_marking_the_value_seen() {
            let calls = Cell::new(0);
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_changed();
            let failures =
                assert_that!(receiver).capture(|it| it.has_current_value(Expected(&calls)));
            assert_that!(failures).has_length(1);
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(receiver.has_changed().unwrap()).is_true();
        }
    }

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, tokio::sync::watch::Receiver<()>, Panic, NoRenderer>
                    => TokioWatchReceiverAssertions<(), NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, tokio::sync::watch::Receiver<()>, Capture, NoRenderer>
                    => TokioWatchReceiverAssertions<(), NoRenderer>
            );
        }

        #[test]
        fn change_state_assertions_require_no_renderer() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(());
            receiver.mark_changed();

            assert_that!(receiver)
                .with_renderer(NoRenderer)
                .has_changed();

            let failures = assert_that!(receiver)
                .with_renderer(NoRenderer)
                .capture(|it| it.has_not_changed().has_changed());
            assert_that!(failures).has_length(1);

            receiver.mark_unchanged();
            assert_that!(receiver)
                .with_renderer(NoRenderer)
                .has_not_changed();
            let failures = assert_that!(receiver)
                .with_renderer(NoRenderer)
                .capture(|it| it.has_changed().has_not_changed());
            assert_that!(failures).has_length(1);
        }
    }

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

        #[test]
        fn caller_location_is_as_expected() {
            let (_tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            assert_caller_location!(
                assert_that!(rx),
                has_current_value(Person {
                    name: "alice".into(),
                })
            );
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

        #[test]
        fn caller_location_is_as_expected() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();
            assert_caller_location!(assert_that!(rx), has_changed());
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_supports_capture() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_changed();
            let failures = receiver.verify(TokioWatchReceiverAssertions::have_changed);
            assert_that!(failures).is_empty();
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(true);
        }

        #[test]
        fn capture_passes_without_changing_the_receiver() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_changed();
            let failures = assert_that!(receiver).capture(|it| {
                let it = it.has_changed().has_changed();
                assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                it
            });
            assert_that!(failures).is_empty();
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(true);
            assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
        }

        #[test]
        fn capture_continues_after_rejection_without_changing_the_receiver() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_unchanged();
            let failures = assert_that!(receiver).with_location(false).capture(|it| {
                let it = it.has_changed().has_changed();
                assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                it
            });
            assert_that!(failures).has_length(2);
            assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `receiver`

                has not changed
                -------- assertr --------
            "});
            assert_that!(failures[1].relation.as_deref()).is_equal_to(Some("has not changed"));
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(false);
            assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
        }

        #[test]
        fn capture_rejects_closed_channels_regardless_of_seen_state() {
            for changed in [false, true] {
                let (sender, mut receiver) = tokio::sync::watch::channel(7);
                if changed {
                    receiver.mark_changed();
                }
                drop(sender);
                let failures = assert_that!(receiver)
                    .with_location(false)
                    .capture(|it| it.has_changed().has_changed());
                assert_that!(failures).has_length(2);
                assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `receiver`

                    is closed
                    -------- assertr --------
                "});
                assert_that!(failures[1].relation.as_deref()).is_equal_to(Some("is closed"));
                assert_that!(receiver.has_changed()).is_err();
                assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
            }
        }

        #[tokio::test]
        async fn succeeds_when_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();

            assert_that!(rx).has_changed();
            assert_that!(rx.has_changed().unwrap()).is_equal_to(true);
        }

        #[tokio::test]
        async fn panics_when_not_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rx`

                    has not changed
                    -------- assertr --------
                "});
            assert_that!(rx.has_changed().unwrap()).is_equal_to(false);
        }

        #[tokio::test]
        async fn panics_when_the_channel_is_closed() {
            let (tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();
            drop(tx);

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rx`

                    is closed
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

        #[test]
        fn caller_location_is_as_expected() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();
            assert_caller_location!(assert_that!(rx), has_not_changed());
        }

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_supports_capture() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_unchanged();
            let failures = receiver.verify(TokioWatchReceiverAssertions::not_have_changed);
            assert_that!(failures).is_empty();
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(false);
        }

        #[test]
        fn capture_passes_without_changing_the_receiver() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_unchanged();
            let failures = assert_that!(receiver).capture(|it| {
                let it = it.has_not_changed().has_not_changed();
                assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                it
            });
            assert_that!(failures).is_empty();
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(false);
            assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
        }

        #[test]
        fn capture_continues_after_rejection_without_changing_the_receiver() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(7);
            receiver.mark_changed();
            let failures = assert_that!(receiver).with_location(false).capture(|it| {
                let it = it.has_not_changed().has_not_changed();
                assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                it
            });
            assert_that!(failures).has_length(2);
            assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `receiver`

                has unexpectedly changed
                -------- assertr --------
            "});
            assert_that!(failures[1].relation.as_deref())
                .is_equal_to(Some("has unexpectedly changed"));
            assert_that!(receiver.has_changed().unwrap()).is_equal_to(true);
            assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
        }

        #[test]
        fn capture_rejects_closed_channels_regardless_of_seen_state() {
            for changed in [false, true] {
                let (sender, mut receiver) = tokio::sync::watch::channel(7);
                if changed {
                    receiver.mark_changed();
                }
                drop(sender);
                let failures = assert_that!(receiver)
                    .with_location(false)
                    .capture(|it| it.has_not_changed().has_not_changed());
                assert_that!(failures).has_length(2);
                assert_that!(ToHumanReadableText.render(&failures[0])).is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `receiver`

                    is closed
                    -------- assertr --------
                "});
                assert_that!(failures[1].relation.as_deref()).is_equal_to(Some("is closed"));
                assert_that!(receiver.has_changed()).is_err();
                assert_that!(*tokio::sync::watch::Receiver::borrow(&receiver)).is_equal_to(7);
            }
        }

        #[tokio::test]
        async fn succeeds_when_not_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();

            assert_that!(rx).has_not_changed();
            assert_that!(rx.has_changed().unwrap()).is_equal_to(false);
        }

        #[tokio::test]
        async fn panics_when_changed() {
            let (_tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_changed();

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_not_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rx`

                    has unexpectedly changed
                    -------- assertr --------
                "});
            assert_that!(rx.has_changed().unwrap()).is_equal_to(true);
        }

        #[tokio::test]
        async fn panics_when_the_channel_is_closed() {
            let (tx, mut rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            rx.mark_unchanged();
            drop(tx);

            assert_that_panic_by(|| assert_that!(rx).with_location(false).has_not_changed())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `rx`

                    is closed
                    -------- assertr --------
                "});
        }
    }
}

#[cfg(test)]
mod string_views {
    use crate::{
        prelude::*,
        test_support::{StrOperand, StringRenderer},
    };
    use core::cell::Cell;

    #[test]
    fn literal_and_custom_views_preserve_the_watch_observation() {
        let (_sender, mut receiver) = tokio::sync::watch::channel(String::from("hello"));
        receiver.mark_changed();
        assert_that!(receiver)
            .has_current_value("hello")
            .matches(super::HasCurrentValue::new("hello"));
        let calls = Cell::new(0);
        let failures = assert_that!(receiver)
            .with_renderer(StringRenderer)
            .capture(|root| {
                root.derive(|value| value).has_current_value(StrOperand {
                    value: "world",
                    observe: || {
                        assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                        calls.set(calls.get() + 1);
                    },
                });
                root
            });
        assert_that!(failures).has_length(1);
        assert_that!(calls.get()).is_equal_to(1);
        assert_that!(receiver.has_changed().unwrap()).is_true();
    }
}
