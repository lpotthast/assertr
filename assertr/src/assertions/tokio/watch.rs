use crate::failure::FailureKind;
use crate::mode::Panic;
use crate::prelude::*;
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use core::borrow::Borrow;

/// Compares the current watch value without marking it seen.
pub struct HasCurrentValue<E>(E);
impl<T, E, R> Expectation<tokio::sync::watch::Receiver<T>, R> for HasCurrentValue<E>
where
    T: PartialEq,
    E: Borrow<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    type Rejection<'a>
        = (tokio::sync::watch::Ref<'a, T>, &'a T)
    where
        Self: 'a,
        tokio::sync::watch::Receiver<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a tokio::sync::watch::Receiver<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let actual = tokio::sync::watch::Receiver::borrow(actual);
        let expected = self.0.borrow();
        if *actual == *expected {
            Ok(())
        } else {
            Err((actual, expected))
        }
    }
}
impl<T, E, R> ExpectationDiagnostics<tokio::sync::watch::Receiver<T>, R> for HasCurrentValue<E>
where
    T: PartialEq,
    E: Borrow<T>,
    R: ValueRenderer<T>,
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
                .expected(render.value(self.0.borrow())),
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
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
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
        self.apply_assertion(HasCurrentValue::new(expected))
    }
}

/// Panic-mode assertions over a watch receiver's change state.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait TokioWatchReceiverExtractAssertions<T, R = crate::DebugRenderer> {
    /// Asserts that the current value has not been seen by this receiver.
    ///
    /// A closed channel fails this assertion.
    fn has_changed(self) -> Self;

    /// Asserts that the current value has already been seen by this receiver.
    ///
    /// A closed channel fails this assertion.
    fn has_not_changed(self) -> Self;
}

impl<T, R> TokioWatchReceiverExtractAssertions<T, R>
    for AssertThat<'_, tokio::sync::watch::Receiver<T>, Panic, R>
{
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
                AssertThat<'static, tokio::sync::watch::Receiver<()>, Panic, NoRenderer>
                    => TokioWatchReceiverExtractAssertions<(), NoRenderer>
            );
        }

        #[test]
        fn change_state_assertions_require_no_renderer() {
            let (_sender, mut receiver) = tokio::sync::watch::channel(());
            receiver.mark_changed();

            assert_that!(receiver)
                .with_renderer(NoRenderer)
                .has_changed();
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
                    Expression: `rx`

                    has not changed
                    -------- assertr --------
                "});
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
                    Expression: `rx`

                    has unexpectedly changed
                    -------- assertr --------
                "});
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
