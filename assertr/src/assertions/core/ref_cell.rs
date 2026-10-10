use core::cell::{Ref, RefCell, RefMut};

use crate::{
    AssertThat, Mode,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Defines an expectation that a cell is borrowed in some way. Evaluation tries the conflicting
/// `probe` borrow. Acquiring it proves that the required borrow is absent, so a rejection retains
/// the acquired guard and renders the value through it.
macro_rules! borrowed_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        probe $probe:ident -> $guard:ident;
        relations $relation:literal, $negated:literal;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<T, R> $crate::expectation::Expectation<RefCell<T>, R> for $name
        where
            R: ValueRenderer<T>,
        {
            type Success<'a>
                = ()
            where
                Self: 'a,
                RefCell<T>: 'a;
            type Rejection<'a>
                = $guard<'a, T>
            where
                Self: 'a,
                RefCell<T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a RefCell<T>,
                _: &$crate::expectation::AssertionContext<'_, R>,
            ) -> Result<(), $guard<'a, T>> {
                actual.$probe().map_or(Ok(()), Err)
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::Other;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a RefCell<T>, $guard<'a, T>)>,
                failure: $crate::failure::FailureBuilder,
                context: &$crate::expectation::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                let render = context.render();
                failure.relations(
                    rejected.map(|(actual, guard)| {
                        render.struct_field(actual, "RefCell", "value", &*guard)
                    }),
                    $relation,
                    $negated,
                )
            }
        }
    };
}

/// Defines an expectation that a cell lacks a conflicting borrow. Evaluation acquires the `probe`
/// borrow and returns its guard as the successful observation. A rejection cannot read the value,
/// so diagnostics show it as borrowed and need no renderer.
macro_rules! not_borrowed_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        probe $probe:ident -> $guard:ident;
        relations $relation:literal, $negated:literal;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<T, R> $crate::expectation::Expectation<RefCell<T>, R> for $name {
            type Success<'a>
                = $guard<'a, T>
            where
                Self: 'a,
                RefCell<T>: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                RefCell<T>: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a RefCell<T>,
                _: &$crate::expectation::AssertionContext<'_, R>,
            ) -> Result<$guard<'a, T>, ()> {
                actual.$probe().map_err(|_| ())
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::Other;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a RefCell<T>, ())>,
                failure: $crate::failure::FailureBuilder,
                context: &$crate::expectation::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                let render = context.render();
                failure.relations(
                    rejected.map(|(actual, ())| {
                        render.unavailable_struct_field(actual, "RefCell", "value", "<borrowed>")
                    }),
                    $relation,
                    $negated,
                )
            }
        }
    };
}

borrowed_expectation! {
    /// Observes whether a cell is borrowed, retaining an acquired borrow on rejection.
    pub struct IsBorrowed;
    probe try_borrow_mut -> RefMut;
    relations "is borrowed", "is not borrowed";
}

borrowed_expectation! {
    /// Observes whether a cell is mutably borrowed, retaining an acquired borrow on rejection.
    pub struct IsMutablyBorrowed;
    probe try_borrow -> Ref;
    relations "is mutably borrowed", "is not mutably borrowed";
}

not_borrowed_expectation! {
    /// Acquires an exclusive borrow if the cell has no active borrow.
    pub struct IsNotBorrowed;
    probe try_borrow_mut -> RefMut;
    relations "is not borrowed", "is unexpectedly borrowed";
}

not_borrowed_expectation! {
    /// Acquires a shared borrow if the cell has no active mutable borrow.
    pub struct IsNotMutablyBorrowed;
    probe try_borrow -> Ref;
    relations "is not mutably borrowed", "is unexpectedly mutably borrowed";
}

/// Assertions for the dynamic borrow state of a [`RefCell`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RefCellAssertions<T, R = DebugRenderer> {
    /// Asserts that the `RefCell` has an active shared or mutable borrow.
    fn is_borrowed(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the `RefCell` has an active mutable borrow.
    fn is_mutably_borrowed(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the `RefCell` has no active borrow, shared or mutable.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use core::cell::RefCell;
    ///
    /// let cell = RefCell::new(42);
    /// drop(cell.borrow());
    /// assert_that!(cell).is_not_borrowed();
    /// ```
    fn is_not_borrowed(self) -> Self;

    /// Asserts that the `RefCell` has no active mutable borrow.
    ///
    /// Immutable borrows are allowed.
    fn is_not_mutably_borrowed(self) -> Self;
}

impl<T, M: Mode, R> RefCellAssertions<T, R> for AssertThat<'_, RefCell<T>, M, R> {
    #[track_caller]
    fn is_borrowed(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsBorrowed)
    }

    #[track_caller]
    fn is_mutably_borrowed(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsMutablyBorrowed)
    }

    #[track_caller]
    fn is_not_borrowed(self) -> Self {
        self.matches(IsNotBorrowed)
    }

    #[track_caller]
    fn is_not_mutably_borrowed(self) -> Self {
        self.matches(IsNotMutablyBorrowed)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use core::cell::RefCell;

        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            {
                let cell = RefCell::new(42);
                let borrow = cell.borrow();
                cell.must().be_borrowed();
                drop(borrow);
            }
            {
                let cell = RefCell::new(42);
                let borrow = cell.borrow_mut();
                cell.must().be_mutably_borrowed();
                drop(borrow);
            }
            RefCell::new(42).must().not_be_borrowed();
            RefCell::new(42).must().not_be_mutably_borrowed();
        }
    }

    mod observations {
        use core::cell::RefCell;

        use super::super::{IsBorrowed, IsMutablyBorrowed, IsNotBorrowed, IsNotMutablyBorrowed};
        use crate::{matchers::all_of, prelude::*, test_support::NoRenderer};

        #[test]
        fn composed_checks_release_rejected_and_successful_borrows_between_siblings() {
            let cell = RefCell::new(7);
            let failures = assert_that!(cell)
                .capture(|it| it.matches(all_of(matchers![IsBorrowed, IsMutablyBorrowed])));
            assert_that!(failures[0].children).has_length(2);
            assert_that!(cell)
                .with_renderer(NoRenderer)
                .matches(all_of(matchers![
                    IsNotMutablyBorrowed,
                    IsNotBorrowed,
                    crate::test_support::opaque_predicate(|cell: &RefCell<i32>| {
                        cell.try_borrow_mut().is_ok()
                    })
                ]));
        }
    }

    mod renderer_contract {
        use core::cell::RefCell;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

        struct Secret;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, RefCell<i32>, Panic, NoRenderer>
                    => RefCellAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn failures_render_the_inner_value_with_the_active_renderer() {
            let cell = RefCell::new(Secret);
            let failures = assert_that!(&cell)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(RefCellAssertions::is_borrowed);

            assert_that!(failures[0].to_string())
                .contains("RefCell {")
                .contains(SENTINEL);
        }
    }

    mod is_borrowed {
        use core::cell::RefCell;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let cell = RefCell::new(42);
            assert_caller_location!(assert_that!(&cell), is_borrowed());
        }

        #[test]
        fn succeeds_when_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            assert_that!(&cell).is_borrowed();
            drop(borrow);
        }

        #[test]
        fn succeeds_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(&cell).is_borrowed();
            drop(borrow);
        }

        #[test]
        fn panics_when_not_borrowed() {
            let cell = RefCell::new(42);
            assert_that!(|| assert_that!(&cell).with_location(false).is_borrowed())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&cell`

                    Actual: RefCell {{
                        value: 42,
                    }}

                    is not borrowed
                    -------- assertr --------
                "});
        }
    }

    mod is_mutably_borrowed {
        use core::cell::RefCell;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let cell = RefCell::new(42);
            assert_caller_location!(assert_that!(cell), is_mutably_borrowed());
        }

        #[test]
        fn succeeds_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(&cell).is_mutably_borrowed();
            drop(borrow);
        }

        #[test]
        fn rejects_shared_borrows() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            let failures = assert_that!(&cell).capture(RefCellAssertions::is_mutably_borrowed);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is not mutably borrowed"));
            drop(borrow);
        }
    }

    mod is_not_borrowed {
        use core::cell::RefCell;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let cell = RefCell::new(42);
            let _borrow = cell.borrow();
            assert_caller_location!(assert_that!(cell), is_not_borrowed());
        }

        #[test]
        fn succeeds_when_not_borrowed_at_all() {
            let cell = RefCell::new(42);
            assert_that!(&cell).is_not_borrowed();
        }

        #[test]
        fn rejects_shared_and_mutable_borrows() {
            let cell = RefCell::new(42);
            let shared = cell.borrow();
            let failures = assert_that!(&cell).capture(RefCellAssertions::is_not_borrowed);
            drop(shared);
            let mutable = cell.borrow_mut();
            let more = assert_that!(&cell).capture(RefCellAssertions::is_not_borrowed);
            drop(mutable);
            for failure in failures.iter().chain(more.iter()) {
                assert_that!(failure.relation.as_deref())
                    .is_equal_to(Some("is unexpectedly borrowed"));
                assert_that!(failure.to_string()).contains("value: <borrowed>");
            }
        }
    }

    mod is_not_mutably_borrowed {
        use core::cell::RefCell;

        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let cell = RefCell::new(42);
            let _borrow = cell.borrow_mut();
            assert_caller_location!(assert_that!(cell), is_not_mutably_borrowed());
        }

        #[test]
        fn succeeds_when_not_borrowed_at_all() {
            let cell = RefCell::new(42);
            assert_that!(&cell).is_not_mutably_borrowed();
        }

        #[test]
        fn succeeds_when_immutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow();
            assert_that!(&cell).is_not_mutably_borrowed();
            drop(borrow);
        }

        #[test]
        fn panics_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(|| {
                assert_that!(&cell)
                    .with_location(false)
                    .is_not_mutably_borrowed()
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `&cell`

                    Actual: RefCell {{
                        value: <borrowed>,
                    }}

                    is unexpectedly mutably borrowed
                    -------- assertr --------
                "});
            drop(borrow);
        }
    }
}
