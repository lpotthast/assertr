use core::{future::Future, panic::Location};

use crate::{AssertThat, actual::Actual, mode::Mode};

impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R> {
    /// Takes the owned subject for an operation that consumes it, continuing on a unit subject
    /// with the same chain state.
    ///
    /// # Panics
    ///
    /// Panics with `misuse` if the subject is borrowed.
    #[track_caller]
    pub(crate) fn take_owned(self, misuse: &'static str) -> (T, AssertThat<'t, (), M, R>) {
        let AssertThat { actual, state } = self;
        let Actual::Owned(actual) = actual else {
            panic!("{misuse}")
        };
        (
            actual,
            AssertThat {
                actual: Actual::Owned(()),
                state,
            },
        )
    }

    /// Maps the assertion subject while preserving the chain state.
    // The mapper is an `impl` argument rather than a named type parameter, so callers can name
    // just the target type, as in `.map::<String>(...)`. The same applies to `map_owned`.
    #[must_use]
    pub fn map<U>(self, mapper: impl FnOnce(Actual<T>) -> Actual<U>) -> AssertThat<'t, U, M, R> {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual: mapper(actual),
            state,
        }
    }

    /// Maps the subject by value and preserves the chain state.
    ///
    /// An owned subject is moved into the mapper. A borrowed subject is cloned first.
    #[must_use]
    pub fn map_owned<U>(self, mapper: impl FnOnce(T) -> U) -> AssertThat<'t, U, M, R>
    where
        T: Clone,
    {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual: Actual::Owned(mapper(actual.into_owned())),
            state,
        }
    }

    /// Asynchronously maps the assertion subject to a new owned subject while preserving the chain
    /// state.
    #[must_use]
    pub async fn map_async<U: 't, Fut>(
        self,
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
            state: self.state.child(),
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
            state: self.state.child(),
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
            state: self.state.child(),
        }
    }

    // Three `satisfies` variants exist because a higher-ranked mapper bound cannot express that
    // an owned result may borrow from its input. `satisfies_owned` requires a result independent
    // of the input lifetime, so `satisfies_ref` covers references to unsized targets.

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
    /// A closure cannot forward `#[track_caller]`, so failures raised inside `assertions` are
    /// located at this `satisfies` call rather than at the failing assertion within the closure.
    /// A `#[track_caller]` custom assertion wrapping `satisfies` therefore reports its own caller.
    /// Nested calls use the outermost `satisfies`.
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
    #[track_caller]
    pub fn satisfies<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(self.derive(mapper).located_at_callback(Location::caller()));
        self
    }

    /// Fluent alias of [`AssertThat::satisfies`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    #[track_caller]
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
    #[track_caller]
    pub fn satisfies_owned<U, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> U,
        for<'a> A: FnOnce(AssertThat<'a, U, M, R>),
        R: Clone,
    {
        assertions(
            self.derive_owned(mapper)
                .located_at_callback(Location::caller()),
        );
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_owned`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    #[track_caller]
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
    #[track_caller]
    pub fn satisfies_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        assertions(
            self.derive_owned(mapper)
                .located_at_callback(Location::caller()),
        );
        self
    }

    /// Fluent alias of [`AssertThat::satisfies_ref`].
    #[cfg(feature = "fluent")]
    #[allow(clippy::return_self_not_must_use)]
    #[track_caller]
    pub fn satisfy_ref<U: ?Sized, F, A>(self, mapper: F, assertions: A) -> Self
    where
        for<'a> F: FnOnce(&'a T) -> &'a U,
        for<'a> A: FnOnce(AssertThat<'a, &'a U, M, R>),
        R: Clone,
    {
        self.satisfies_ref(mapper, assertions)
    }

    /// Locates this child's failures at `caller`, the `satisfies` call handing it to a callback,
    /// unless an enclosing callback already did.
    fn located_at_callback(mut self, caller: &'static Location<'static>) -> Self {
        self.state.settings.callback_caller.get_or_insert(caller);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        test_support::{SENTINEL, SentinelRenderer},
    };

    #[derive(PartialEq)]
    struct Secret(u32);

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            (1, "one")
                .must()
                .satisfy(
                    |pair| &pair.0,
                    |number| {
                        number.be_equal_to(1);
                    },
                )
                .satisfy_owned(
                    |pair| pair.0 + 1,
                    |number| {
                        number.be_equal_to(2);
                    },
                )
                .satisfy_ref(
                    |pair| pair.1,
                    |text| {
                        text.be_equal_to("one");
                    },
                );
        }
    }

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

        assert_that!(failures[0].to_string()).contains(SENTINEL);
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

        assert_that!(&failures[..])
            .contains_exactly_matching(
                [|it: &AssertionFailure| it.to_string().contains("Expected: 4")]
                    .map(matchers::predicate),
            )
            .contains_exactly_satisfying([|it: AssertThat<AssertionFailure, Capture>| {
                it.satisfies_owned(ToString::to_string, |description| {
                    description.contains("Expected: 4");
                });
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

        assert_that!(&failures[..]).contains_exactly_matching(
            [|it: &AssertionFailure| it.to_string().contains("xyz")].map(matchers::predicate),
        );
    }

    mod callback_location {
        use core::panic::Location;

        use crate::prelude::*;

        #[test]
        fn satisfies_locates_callback_failures_at_its_call() {
            assert_caller_location!(
                assert_that!(1),
                satisfies(
                    |it| it,
                    |it| {
                        it.is_equal_to(2);
                    }
                )
            );
            assert_caller_location!(
                assert_that!(1),
                satisfies_owned(
                    |it| *it,
                    |it| {
                        it.is_equal_to(2);
                    }
                )
            );
            assert_caller_location!(
                assert_that!(String::from("a")),
                satisfies_ref(
                    |it| it.as_str(),
                    |it| {
                        it.is_equal_to("b");
                    }
                )
            );
        }

        /// A custom assertion wrapping `satisfies`, as downstream crates write them.
        #[track_caller]
        fn has_first(
            it: AssertThat<'_, (i32, i32), Capture>,
            expected: i32,
        ) -> AssertThat<'_, (i32, i32), Capture> {
            it.satisfies(
                |pair| &pair.0,
                |first| {
                    first.is_equal_to(expected);
                },
            )
        }

        #[test]
        fn a_tracked_custom_assertion_reports_its_caller() {
            let mut caller = None;
            let failures = assert_that!((1, 2)).capture(|it| {
                caller = Some(Location::caller());
                has_first(it, 9)
            });
            let caller = caller.unwrap();
            let location = failures[0].location.unwrap();
            assert_that!((location.file(), location.line()))
                .is_equal_to((caller.file(), caller.line() + 1));
        }

        #[test]
        fn nested_callbacks_use_the_outermost_call() {
            let mut outer = None;
            let failures = assert_that!(((1, 2), 3)).capture(|it| {
                outer = Some(Location::caller());
                it.satisfies(
                    |it| &it.0,
                    |pair| {
                        pair.satisfies(
                            |pair| &pair.1,
                            |second| {
                                second.is_equal_to(9);
                            },
                        );
                    },
                )
            });
            let outer = outer.unwrap();
            assert_that!(failures[0].location.map(Location::line))
                .is_equal_to(Some(outer.line() + 1));
        }

        #[test]
        fn derived_chains_keep_the_location_of_their_own_assertions() {
            let mut caller = None;
            let failures = assert_that!((1, 2)).capture(|it| {
                it.derive(|pair| &pair.0).is_equal_to(9);
                caller = Some(Location::caller());
                it
            });
            assert_that!(failures[0].location.map(Location::line))
                .is_equal_to(Some(caller.unwrap().line() - 1));
        }
    }

    mod map_owned {
        use core::cell::Cell;

        use crate::prelude::*;

        /// Counts its clones in a shared cell.
        struct Counted<'a>(&'a Cell<usize>);

        impl Clone for Counted<'_> {
            fn clone(&self) -> Self {
                self.0.set(self.0.get() + 1);
                Self(self.0)
            }
        }

        #[test]
        fn moves_an_owned_subject() {
            let clones = Cell::new(0);
            assert_that_owned!(Counted(&clones))
                .map_owned(|counted| counted.0.get())
                .is_equal_to(0);
        }

        #[test]
        fn clones_a_borrowed_subject_once() {
            let clones = Cell::new(0);
            let counted = Counted(&clones);
            assert_that!(&counted)
                .map_owned(|counted| counted.0.get())
                .is_equal_to(1);
        }
    }
}
