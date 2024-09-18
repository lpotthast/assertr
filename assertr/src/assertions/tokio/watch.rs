use core::borrow::Borrow;
use core::fmt::Debug;
use core::ops::Deref;

use crate::prelude::PartialEqAssertions;
use crate::{AssertThat, Mode};

/// Assertions for the tokio::sync::watch::Receiver type.
pub trait TokioWatchReceiverAssertions<T: Debug> {
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq;
}

impl<'t, T: Debug, M: Mode> TokioWatchReceiverAssertions<T>
    for AssertThat<'t, tokio::sync::watch::Receiver<T>, M>
{
    #[track_caller]
    fn has_current_value(self, expected: impl Borrow<T>) -> Self
    where
        T: PartialEq,
    {
        self.derive(|it| it.borrow())
            .derive(|it| it.deref())
            .is_equal_to(expected.borrow());
        self
    }
}

#[cfg(test)]
mod tests {
    mod has_current_value {
        use crate::assertions::tokio::watch::TokioWatchReceiverAssertions;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[derive(Debug, PartialEq)]
        struct Person {
            name: String,
        }

        #[tokio::test]
        async fn succeeds_when_equal() {
            let (tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });
            tx.send(Person {
                name: "kevin".into(),
            })
            .unwrap();

            assert_that(rx).has_current_value(Person {
                name: "kevin".into(),
            });
        }

        #[tokio::test]
        async fn panics_when_not_equal() {
            let (_tx, rx) = tokio::sync::watch::channel(Person { name: "bob".into() });

            assert_that_panic_by(|| {
                assert_that(rx)
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
}
