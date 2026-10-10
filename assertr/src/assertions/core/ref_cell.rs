use crate::{
    AssertThat, Mode,
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    renderer::DebugRenderer,
    renderer::ValueRenderer,
};
use core::cell::RefCell;

/// Observes whether a cell is borrowed, retaining an acquired borrow on rejection.
#[derive(Debug, Clone, Copy)]
pub struct IsBorrowed;

impl<T, R> Expectation<RefCell<T>, R> for IsBorrowed
where
    R: ValueRenderer<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        RefCell<T>: 'a;
    type Rejection<'a>
        = core::cell::RefMut<'a, T>
    where
        Self: 'a,
        RefCell<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a RefCell<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual.try_borrow_mut() {
            Ok(guard) => Err(guard),
            Err(_) => Ok(()),
        }
    }

    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a RefCell<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure.relations(
            rejected
                .map(|(actual, guard)| render.struct_field(actual, "RefCell", "value", &*guard)),
            "is borrowed",
            "is not borrowed",
        )
    }
}

/// Observes whether a cell is mutably borrowed, retaining an acquired borrow on rejection.
#[derive(Debug, Clone, Copy)]
pub struct IsMutablyBorrowed;

impl<T, R> Expectation<RefCell<T>, R> for IsMutablyBorrowed
where
    R: ValueRenderer<T>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        RefCell<T>: 'a;
    type Rejection<'a>
        = core::cell::Ref<'a, T>
    where
        Self: 'a,
        RefCell<T>: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a RefCell<T>,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual.try_borrow() {
            Ok(guard) => Err(guard),
            Err(_) => Ok(()),
        }
    }

    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a RefCell<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure.relations(
            rejected
                .map(|(actual, guard)| render.struct_field(actual, "RefCell", "value", &*guard)),
            "is mutably borrowed",
            "is not mutably borrowed",
        )
    }
}

/// Acquires a shared borrow if the cell has no active mutable borrow.
#[derive(Debug, Clone, Copy)]
pub struct IsNotMutablyBorrowed;
impl<T, R> Expectation<RefCell<T>, R> for IsNotMutablyBorrowed {
    type Success<'a>
        = core::cell::Ref<'a, T>
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
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.try_borrow().map_err(|_| ())
    }

    const KIND: FailureKind = FailureKind::Other;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a RefCell<T>, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure.relations(
            rejected.map(|(actual, ())| {
                render.unavailable_struct_field(actual, "RefCell", "value", "<borrowed>")
            }),
            "is not mutably borrowed",
            "is unexpectedly mutably borrowed",
        )
    }
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
    fn is_not_mutably_borrowed(self) -> Self {
        self.matches(IsNotMutablyBorrowed)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;
        use core::cell::RefCell;

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
            RefCell::new(42).must().not_be_mutably_borrowed();
        }
    }

    mod observations {
        use super::super::{IsBorrowed, IsMutablyBorrowed, IsNotMutablyBorrowed};
        use crate::{matchers::all_of, prelude::*, test_support::NoRenderer};
        use core::cell::RefCell;

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
                    crate::test_support::opaque_predicate(|cell: &RefCell<i32>| {
                        cell.try_borrow_mut().is_ok()
                    })
                ]));
        }
    }

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use core::cell::RefCell;

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
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::cell::RefCell;

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
        use crate::prelude::*;
        use std::cell::RefCell;

        #[test]
        fn caller_location_is_as_expected() {
            let cell = RefCell::new(42);
            assert_caller_location!(assert_that!(cell), is_mutably_borrowed());
        }

        #[test]
        fn succeeds_when_mutably_borrowed() {
            let cell = RefCell::new(42);
            let borrow = cell.borrow_mut();
            assert_that!(&cell).is_borrowed();
            assert_that!(&cell).is_mutably_borrowed();
            drop(borrow);
        }
    }

    mod is_not_mutably_borrowed {
        use crate::prelude::*;
        use indoc::formatdoc;
        use std::cell::RefCell;

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
