use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use core::future::Future;

use crate::{
    AssertThat,
    actual::Actual,
    mode::{Capture, Mode},
};

impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R> {
    pub(crate) fn replace_actual_with<'u, U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        new_actual: Actual<'u, U>,
    ) -> (Actual<'t, T>, AssertThat<'u, U, M, R>)
    where
        't: 'u,
    {
        let AssertThat { actual, state } = self;
        let mapped = AssertThat {
            actual: new_actual,
            state,
        };
        (actual, mapped)
    }

    /// Maps the assertion subject while preserving the chain state.
    #[must_use]
    pub fn map<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Actual<U>,
    ) -> AssertThat<'t, U, M, R> {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual: mapper(actual),
            state,
        }
    }

    /// Creates an owned copy of the subject with [`ToOwned`], maps it, and preserves the chain
    /// state.
    #[must_use]
    pub fn map_owned<U>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(<T as ToOwned>::Owned) -> U,
    ) -> AssertThat<'t, U, M, R>
    where
        T: ToOwned,
    {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual: Actual::Owned(mapper(actual.borrowed().to_owned())),
            state,
        }
    }

    /// Asynchronously maps the assertion subject to a new owned subject while preserving the
    /// chain state.
    #[must_use]
    pub async fn map_async<U: 't, Fut>(
        self,
        // Note: Not using an explicit generic typename allows calls like `.map<String>(...)`,
        // requiring only one type, which is the type we want to map to.
        mapper: impl FnOnce(Actual<T>) -> Fut,
    ) -> AssertThat<'t, U, M, R>
    where
        Fut: Future<Output = U>,
    {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual: mapper(actual).await.into(),
            state,
        }
    }

    /// Derives a child assertion over an owned projection of the subject.
    ///
    /// The mapper borrows the subject and returns an owned value. Use [`AssertThat::derive`] when
    /// the projection borrows from the subject. A returned reference to an unsized target becomes
    /// the child subject itself, as with [`AssertThat::satisfies_ref`].
    ///
    /// Failures and assertion counts propagate to the root. Use the `derive_*` methods when the
    /// child must be stored or chained. Use `satisfies_*` to return to the original subject after
    /// asserting on the projection.
    #[must_use]
    pub fn derive_owned<'u, U: 'u>(
        &'t self,
        mapper: impl FnOnce(&'t T) -> U,
    ) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        AssertThat {
            actual: Actual::Owned(mapper(self.actual())),
            state: self.state.child(self, self.state.renderer.clone()),
        }
    }

    /// Derives a child assertion over a borrowed projection of the subject.
    ///
    /// The child is `AssertThat<U>`, not `AssertThat<&U>`, so assertions implemented for `U`
    /// remain available. Use [`AssertThat::derive_owned`] for a computed or cloned projection.
    /// Failures and assertion counts propagate to the root.
    #[must_use]
    pub fn derive<'u, U>(&'t self, mapper: impl FnOnce(&'t T) -> &'u U) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        AssertThat {
            actual: Actual::Borrowed(mapper(self.actual())),
            state: self.state.child(self, self.state.renderer.clone()),
        }
    }

    /// The async variant of [`AssertThat::derive_owned`]. The mapper's future produces the owned
    /// projection. Async mappers cannot return a projection borrowing from their input.
    #[must_use]
    pub async fn derive_async<'u, U: 'u, Fut: Future<Output = U>>(
        &'t self,
        mapper: impl FnOnce(&'t T) -> Fut,
    ) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        AssertThat {
            actual: Actual::Owned(mapper(self.actual()).await),
            state: self.state.child(self, self.state.renderer.clone()),
        }
    }

    // It would be nice to optimize this, so that:
    // - we do not need separate satisfies, satisfies_owned and satisfies_ref methods
    // - we use a `for<'a: 'b, 'b>` (see https://users.rust-lang.org/t/why-cant-i-use-lifetime-bounds-in-hrtbs/97277/2) bound for F and A,
    //   telling the compiler that the returned values live shorter than the input.
    // - we can replace () with some type R (return), letting the user write more succinct closures.

    /// Runs the given assertions against a borrowed projection of the subject.
    ///
    /// The `satisfies_*` family creates a child assertion, passes it to `assertions`, and returns
    /// the current chain. Child failures propagate to the root. The closure returns `()`.
    ///
    /// The variants differ only in how the projection is obtained and typed:
    ///
    /// | Method | Mapper returns | Closure receives | Use when |
    /// |---|---|---|---|
    /// | [`satisfies`](AssertThat::satisfies) | `&U` | `AssertThat<U>` | The projection borrows from the subject and `U` is sized (the common case). |
    /// | [`satisfies_owned`](AssertThat::satisfies_owned) | owned `U` | `AssertThat<U>` | The projection is computed (or cloned), not borrowed from the subject. |
    /// | [`satisfies_ref`](AssertThat::satisfies_ref) | `&U` | `AssertThat<&U>` | `U` is unsized (`str`, `[T]`, ...). |
    ///
    /// `satisfies` produces `AssertThat<U>` while borrowing `U`, as `assert_that!(&value)` does.
    /// Use `satisfies_ref` only for unsized `U`, where the child must be `AssertThat<&U>`.
    ///
    /// # Example
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that!(("foo".to_owned(), 42))
    ///     .satisfies(|it| &it.0, |name| {
    ///         name.contains("oo");
    ///     })
    ///     .satisfies_owned(|it| it.0.len(), |len| {
    ///         len.is_equal_to(3);
    ///     })
    ///     .satisfies_ref(|it| it.0.as_str(), |name| {
    ///         name.starts_with("f");
    ///     });
    /// ```
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(self.derive(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        self.satisfies(mapper, assertions)
    }

    /// Runs the given assertions against an owned projection of the subject.
    ///
    /// The closure receives an `AssertThat<U>` owning the projection. Use this for a computed or
    /// cloned projection. Use [`AssertThat::satisfies`] for a borrowed projection.
    ///
    /// See [`AssertThat::satisfies`] for a comparison of the whole family and an example.
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies_owned<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(self.derive_owned(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_owned`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy_owned<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        self.satisfies_owned(mapper, assertions)
    }

    /// Runs the given assertions against a borrowed, unsized projection of the subject.
    ///
    /// The closure receives `AssertThat<&U>`. Use this for unsized projections such as `str` or
    /// `[T]`. Use [`AssertThat::satisfies`] for sized projections.
    ///
    /// See [`AssertThat::satisfies`] for a comparison of the whole family and an example.
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfies_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        assertions(self.derive_owned(mapper));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_ref`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    pub fn satisfy_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        self.satisfies_ref(mapper, assertions)
    }

    /// Runs `assertions` against `element` on a capture-mode assertion, returning every failure
    /// raised. An empty result means that the element satisfies the assertions.
    ///
    /// Used by the collection assertions treating per-element assertions as a matching criterion.
    pub(crate) fn collect_element_failures<'e, U, A>(
        &self,
        element: &'e U,
        assertions: A,
    ) -> Vec<String>
    where
        A: for<'a> FnOnce(AssertThat<'a, U, Capture, R>),
        R: Clone,
    {
        // The closure consumes the assertion handed to it, so failures are collected in a
        // capture-mode sink the closure never owns: `satisfies` derives the handed-out
        // assertion from the sink, letting its failures propagate there.
        let sink = AssertThat::new_capturing(Actual::Borrowed(element))
            .with_renderer(self.state.renderer.clone())
            .with_location(self.state.print_location)
            .satisfies(|it| it, assertions);
        let failures = sink.state.failures.take();
        failures.iter().map(ToString::to_string).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn nested_derived_assertions_propagate_no_failures_when_they_pass() {
        let failures = assert_that!(1).capture(|root| {
            {
                let doubled = root.derive_owned(|it| *it * 2);
                {
                    let incremented = doubled.derive_owned(|it| *it + 1);
                    incremented.is_equal_to(3);
                }
            }
            root
        });

        assert_that!(failures).is_empty();
    }

    #[test]
    fn nested_derived_assertions_propagate_failures_to_the_root() {
        let failures = assert_that!(1).with_location(false).capture(|root| {
            {
                let doubled = root.derive_owned(|it| *it * 2);
                let incremented = doubled.derive_owned(|it| *it + 1);
                incremented.is_equal_to(4);
            }
            root
        });

        assert_that!(failures.as_slice())
            .contains_exactly_matching([|it: &AssertionFailure| {
                it.description.contains("Expected: 4")
            }])
            .contains_exactly_satisfying([|it: AssertThat<AssertionFailure, Capture>| {
                it.satisfies(
                    |failure| &failure.description,
                    |description| {
                        description.contains("Expected: 4");
                    },
                );
            }]);
    }

    #[test]
    fn satisfies_hands_out_a_value_typed_assertion_over_the_borrowed_projection() {
        assert_that!(("foo".to_owned(), 42))
            .satisfies(
                |it| &it.0,
                |name| {
                    name.contains("oo");
                },
            )
            .satisfies(
                |it| &it.1,
                |number| {
                    number.is_equal_to(42);
                },
            );
    }

    #[test]
    fn satisfies_propagates_failures_to_the_root() {
        let failures = assert_that!(("foo".to_owned(), 42))
            .with_location(false)
            .capture(|it| {
                it.satisfies(
                    |v| &v.0,
                    |name| {
                        name.contains("xyz");
                    },
                )
            });

        assert_that!(failures.as_slice())
            .contains_exactly_matching([|it: &AssertionFailure| it.description.contains("xyz")]);
    }
}
