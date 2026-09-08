use alloc::borrow::ToOwned;
use core::future::Future;

use crate::{AssertThat, actual::Actual, mode::Mode};

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

    /// Asynchronously maps the assertion subject to a new owned subject while preserving the chain
    /// state.
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
    /// The mapper borrows the parent and returns the child value. The result is stored as the
    /// child subject, including when it is a reference to an unsized target. Use [`Self::derive`]
    /// to borrow a sized field. Neither the parent nor the child value needs to implement `Clone`.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let name = String::from("Ada");
    /// let name = assert_that!(name);
    /// name.derive_owned(String::len).is_equal_to(3);
    /// name.derive_owned(String::as_str).starts_with("A");
    /// ```
    ///
    /// The child inherits diagnostic settings and propagates failures and assertion counts as
    /// described in [`Self::derive`]. Projection itself does not count as an assertion.
    /// Use [`Self::satisfies_owned`] to check a projection in a closure and return the original
    /// chain.
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
            state: self.state.child(self.state.renderer.clone()),
        }
    }

    /// Derives a child assertion over a borrowed projection of the subject.
    ///
    /// The mapper returns `&U` and the child is `AssertThat<U>`, keeping assertions implemented
    /// for `U` available. The parent remains usable, so check several fields in separate chains:
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let person = (String::from("Ada"), 36);
    /// let person = assert_that!(person);
    /// person.derive(|p| &p.0).starts_with("A").has_length(3);
    /// person.derive(|p| &p.1).is_greater_or_equal_to(18);
    /// ```
    ///
    /// The child clones the active renderer and inherits detail messages, rendering budget,
    /// location settings, and panic presentation. Its subject name and source expression start
    /// empty. Failures and assertion counts propagate to the parent. Projection itself does not
    /// count as an assertion, and neither the parent nor its field needs to implement `Clone`.
    ///
    /// In [`Self::capture`], return the parent after checking its children. A child borrows the
    /// parent and cannot replace it as the closure's returned chain:
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!((String::from("Ada"), 16)).capture(|person| {
    ///     person.derive(|p| &p.0).starts_with("B");
    ///     person.derive(|p| &p.1).is_greater_or_equal_to(18);
    ///     person
    /// });
    /// assert_that!(failures).has_length(2);
    /// ```
    ///
    /// The projection methods differ in what the mapper returns:
    ///
    /// | Method | Mapper returns |
    /// |---|---|
    /// | [`derive`](Self::derive) | `&U`, where `U: Sized`. |
    /// | [`derive_owned`](Self::derive_owned) | A computed `U`, or a reference such as `&str` or `&[T]`. |
    /// | [`derive_async`](Self::derive_async) | A future producing the child value. |
    ///
    /// Use [`Self::derive_owned`] for computed values, or for references to unsized targets such
    /// as `str` and `[T]`. Use [`Self::satisfies`] to check a projection in a closure and return
    /// the original chain. With the `fluent` feature, this method keeps the spelling `derive`, as
    /// in `person.must().derive(|p| &p.1).be_greater_or_equal_to(18)`.
    #[must_use]
    pub fn derive<'u, U>(&'t self, mapper: impl FnOnce(&'t T) -> &'u U) -> AssertThat<'u, U, M, R>
    where
        't: 'u,
        R: Clone,
    {
        AssertThat {
            actual: Actual::Borrowed(mapper(self.actual())),
            state: self.state.child(self.state.renderer.clone()),
        }
    }

    /// The async variant of [`Self::derive_owned`].
    ///
    /// The mapper borrows the parent and returns a future producing the child value. A returned
    /// reference becomes the child subject itself, as with [`Self::derive_owned`]. Await the
    /// projection before chaining assertions:
    ///
    /// ```
    /// # async fn example() {
    /// use assertr::prelude::*;
    ///
    /// let person = (String::from("Ada"), 36);
    /// let person = assert_that!(person);
    /// person.derive_async(|p| async move { p.0.len() }).await.is_equal_to(3);
    /// # }
    /// ```
    ///
    /// The child inherits diagnostic settings and propagates failures and assertion counts as
    /// described in [`Self::derive`]. Projection itself does not count as an assertion.
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
            state: self.state.child(self.state.renderer.clone()),
        }
    }

    // It would be nice to optimize this, so that:
    // - we do not need separate satisfies, satisfies_owned and satisfies_ref methods
    // - we use a `for<'a: 'b, 'b>` (see https://users.rust-lang.org/t/why-cant-i-use-lifetime-bounds-in-hrtbs/97277/2)
    //   bound for F and A, telling the compiler that the returned values live shorter than the
    //   input.
    // - we can replace () with some type R (return), letting the user write more succinct closures.

    /// Runs the given assertions against a borrowed projection of the subject.
    ///
    /// The `satisfies_*` family creates a child assertion, passes it to `assertions`, and returns
    /// the current chain. Use it to check fields or computed properties with the same assertions
    /// you would use on a standalone value. The closure returns `()`, so end its final assertion
    /// with a semicolon.
    ///
    /// Child failures follow the root's mode. They panic immediately in panic mode and join the
    /// collected failures inside [`Self::capture`]. The child preserves the active renderer and
    /// inherits detail messages, rendering budget, location settings, and panic presentation. Its
    /// subject name and source expression start empty and can be set on the child.
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
    ///
    /// Use [`Self::derive`] or [`Self::derive_owned`] to project first and then chain assertions
    /// directly on the child. To express a reusable expectation across several fields or nested
    /// collections, see [structural matching](mod@crate::matchers#structural-syntax). A
    /// domain-specific method can wrap these projections as a [custom
    /// assertion](crate#custom-assertions).
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
    /// Captures assertions on a locked value using the shared matcher capture helper. Leaf values
    /// are rendered when failures are built. Presentation remains deferred.
    #[cfg(feature = "tokio")]
    pub(crate) fn collect_element_failures<'e, U, A>(
        &self,
        element: &'e U,
        assertions: A,
    ) -> crate::AssertionFailures
    where
        A: for<'a> FnOnce(AssertThat<'a, U, crate::mode::Capture, R>),
        R: Clone,
    {
        crate::matchers::collect_assertions(
            element,
            self.render(),
            self.state.include_location,
            assertions,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::test_support::{SENTINEL, SentinelRenderer};

    #[derive(PartialEq)]
    struct Secret(u32);

    #[test]
    fn custom_renderer_is_preserved_by_satisfies() {
        let failures = assert_that!(Secret(1))
            .with_renderer(SentinelRenderer)
            .with_location(false)
            .capture(|it| {
                it.satisfies(
                    |secret| &secret.0,
                    |inner| {
                        inner.is_equal_to(2);
                    },
                )
            });

        assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
    }

    #[test]
    fn debug_format_closure_is_cloneable_for_derived_chains() {
        assert_that!(Secret(7))
            .with_debug_format(|value: &Secret, f| write!(f, "Secret({})", value.0))
            .derive_owned(|secret| Secret(secret.0))
            .is_equal_to(Secret(7));
    }

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
            .contains_exactly_matching(crate::matchers::predicate_list([
                |it: &AssertionFailure| ToHumanReadableText.render(it).contains("Expected: 4"),
            ]))
            .contains_exactly_satisfying([|it: AssertThat<AssertionFailure, Capture>| {
                it.satisfies_owned(
                    |failure| ToHumanReadableText.render(failure),
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

        assert_that!(failures.as_slice()).contains_exactly_matching(
            crate::matchers::predicate_list([|it: &AssertionFailure| {
                ToHumanReadableText.render(it).contains("xyz")
            }]),
        );
    }
}
