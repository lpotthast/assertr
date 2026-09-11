//! Core construction and basic operations for assertion chains.

mod capture;
mod diagnostics;
mod projection;
mod rendering;

use alloc::vec::Vec;
use core::{cell::RefCell, marker::PhantomData, panic::AssertUnwindSafe};

use crate::{
    AssertThat, AssertionFailures, ChainRecords, ChainState, Expression,
    actual::Actual,
    mode::{Capture, Mode, Panic},
    renderer::{DebugRenderer, RenderingBudget},
    tracking::NumberOfAssertions,
};

impl<'t> ChainRecords<'t> {
    const fn new(parent: Option<&'t Self>) -> Self {
        Self {
            parent,
            detail_messages: AssertUnwindSafe(RefCell::new(Vec::new())),
            number_of_assertions: AssertUnwindSafe(RefCell::new(NumberOfAssertions::new())),
            failures: AssertUnwindSafe(RefCell::new(AssertionFailures::new())),
        }
    }
}

impl<'t, M: Mode, R> ChainState<'t, M, R> {
    const fn root(renderer: R) -> Self {
        Self {
            records: ChainRecords::new(None),
            subject_name: None,
            expression: Expression::Unset,
            include_location: true,
            rendering_budget: RenderingBudget::DEFAULT,
            panic_presentation: None,
            mode: PhantomData,
            renderer,
        }
    }

    /// Inherit diagnostic settings while starting a new subject and linking only ancestor records.
    fn child<R2>(&self, renderer: R2) -> ChainState<'_, M, R2> {
        ChainState {
            records: ChainRecords::new(Some(&self.records)),
            subject_name: None,
            expression: Expression::Unset,
            include_location: self.include_location,
            rendering_budget: self.rendering_budget,
            panic_presentation: self.panic_presentation.clone(),
            mode: PhantomData,
            renderer,
        }
    }

    fn with_renderer<R2>(self, renderer: R2) -> ChainState<'t, M, R2> {
        ChainState {
            records: self.records,
            subject_name: self.subject_name,
            expression: self.expression,
            include_location: self.include_location,
            rendering_budget: self.rendering_budget,
            panic_presentation: self.panic_presentation,
            mode: self.mode,
            renderer,
        }
    }
}

impl<'t, T> AssertThat<'t, T, Panic> {
    #[track_caller]
    pub(crate) const fn new_panicking(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            state: ChainState::root(DebugRenderer),
        }
    }
}

impl<'t, T> AssertThat<'t, T, Capture> {
    /// Starts a fluent capture root whose receiver expression can be attached after completion.
    #[cfg(feature = "fluent")]
    #[track_caller]
    pub(crate) fn new_fluent_capturing(actual: Actual<'t, T>) -> Self {
        let mut assertion = Self::new_capturing(actual);
        assertion.state.expression = Expression::PendingFluent(core::panic::Location::caller());
        assertion
    }

    #[track_caller]
    pub(crate) const fn new_capturing(actual: Actual<'t, T>) -> Self {
        AssertThat {
            actual,
            state: ChainState::root(DebugRenderer),
        }
    }
}

/* Fluent connect */

impl<T, M: Mode, R> AssertThat<'_, T, M, R> {
    /// Borrows the current assertion subject.
    ///
    /// Custom assertion implementations use this to inspect the value being asserted.
    pub fn actual(&self) -> &T {
        self.actual.borrowed()
    }

    /// Returns the chain unchanged, allowing an optional `and()` between assertions.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// assert_that!(42).is_greater_than(0).and().is_less_than(100);
    /// assert_that!(42).is_greater_than(0).is_less_than(100);
    /// ```
    #[inline]
    #[must_use]
    pub fn and(self) -> Self {
        self
    }
}

/* Unwrapping */

impl<T, R> AssertThat<'_, T, Panic, R> {
    /// Unwraps the owned subject from this assertion.
    ///
    /// # Panics
    ///
    /// Panics if the subject is borrowed. Use `assert_that_owned!(...)` or `.must_owned()` to
    /// create an owned assertion.
    #[track_caller]
    #[must_use]
    pub fn unwrap_inner(self) -> T {
        self.actual.unwrap_owned()
    }
}

#[cfg(test)]
mod tests {
    mod unwind_safety {
        use alloc::{rc::Rc, string::String};
        use core::{
            cell::{Cell, RefCell},
            panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe},
        };
        use std::panic::catch_unwind;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn auto_traits_require_only_the_corresponding_component_bounds() {
            fn moved<T: UnwindSafe + RefUnwindSafe, M: Mode, R: UnwindSafe>() {
                assert_trait_impl!(AssertThat<'_, T, M, R> => UnwindSafe);
            }
            fn shared<T: RefUnwindSafe, M: Mode, R: RefUnwindSafe>() {
                assert_trait_impl!(AssertThat<'_, T, M, R> => RefUnwindSafe);
            }

            moved::<i32, Panic, Cell<i32>>();
            moved::<i32, Capture, Cell<i32>>();
            shared::<&mut i32, Panic, &mut NoRenderer>();
            shared::<&mut i32, Capture, &mut NoRenderer>();
        }

        #[test]
        fn scalar_contexts_can_be_shared_and_moved_across_catch_boundaries() {
            let root = assert_that_owned!(1).with_renderer(NoRenderer);
            catch_unwind(|| assert_eq!(root.actual(), &1)).unwrap();
            catch_unwind(move || assert_eq!(root.unwrap_inner(), 1)).unwrap();

            let failures = assert_that!(1).capture(|root| {
                catch_unwind(|| root.track_assertion()).unwrap();
                catch_unwind(move || root).unwrap()
            });
            assert!(failures.is_empty());
        }

        #[test]
        fn an_owned_cell_renderer_can_be_moved_across_a_catch_boundary() {
            let context = assert_that_owned!(1).with_renderer(Cell::new(0));
            catch_unwind(move || assert_eq!(context.unwrap_inner(), 1)).unwrap();
        }

        #[test]
        fn explicitly_overridden_subject_state_remains_accessible_after_a_panic() {
            let value = Cell::new((0, 0));
            let context = assert_that!(value);
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    context.actual().set((1, 0));
                    panic!("interrupted update");
                }))
                .is_err()
            );
            assert_eq!(context.actual().get(), (1, 0));

            let context = assert_that_owned!(RefCell::new((0, 0)));
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    context.actual().borrow_mut().0 = 1;
                    panic!("interrupted update");
                }))
                .is_err()
            );
            assert_eq!(*context.actual().borrow(), (1, 0));
        }

        #[test]
        fn stateful_renderers_and_callbacks_remain_usable_without_unwind_bounds() {
            let calls = Rc::new(Cell::new(0));
            let renderer_calls = Rc::clone(&calls);
            let failures = assert_that!(RefCell::new(1))
                .with_debug_format(move |value, f| {
                    renderer_calls.set(renderer_calls.get() + 1);
                    write!(f, "{}", value.borrow())
                })
                .capture(|root| {
                    root.derive(|value| value).is_equal_to(RefCell::new(2));
                    root
                });
            assert_eq!(failures.len(), 1);
            assert_eq!(calls.get(), 2);

            let context = assert_that!(1).with_debug_format(|_, _| {
                calls.set(calls.get() + 1);
                panic!("renderer panic");
            });
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    context.derive_owned(|value| *value).is_equal_to(2);
                }))
                .is_err()
            );
            assert_eq!(calls.get(), 3);
        }

        #[test]
        fn a_safe_child_does_not_inherit_its_parents_subject_or_renderer_state() {
            let failures = assert_that_owned!(RefCell::new((1, 2)))
                .with_renderer(Cell::new(0))
                .with_detail_message("parent detail")
                .capture(|root| {
                    let child = root
                        .derive_owned(|value| value.borrow().0)
                        .with_renderer(NoRenderer);
                    catch_unwind(|| {
                        assert_eq!(child.actual(), &1);
                        child.track_assertion();
                    })
                    .unwrap();
                    catch_unwind(move || {
                        child.with_renderer(DebugRenderer).is_equal_to(3);
                    })
                    .unwrap();
                    assert_eq!(root.state.records.assertion_count(), 2);
                    root
                });
            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].messages, ["parent detail"]);
        }

        #[test]
        fn capture_bookkeeping_remains_usable_after_a_caught_panic() {
            let failures = assert_that!(1).with_detail_message("root").capture(|root| {
                let child = root.derive(|value| value).with_detail_message("child");
                assert!(
                    catch_unwind(|| {
                        child.derive(|value| value).is_equal_to(2);
                        panic!("after recording a failure");
                    })
                    .is_err()
                );
                child.is_equal_to(3);
                assert_eq!(root.state.records.assertion_count(), 2);
                root.is_equal_to(4)
            });
            assert_eq!(failures.len(), 3);
            assert_eq!(failures[0].messages, ["child", "root"]);
            assert_eq!(failures[1].messages, ["child", "root"]);
            assert_eq!(failures[2].messages, ["root"]);
        }

        #[test]
        fn message_conversion_runs_outside_the_bookkeeping_borrow() {
            struct Message<'a>(&'a dyn Fn());
            impl From<Message<'_>> for String {
                fn from(message: Message<'_>) -> Self {
                    (message.0)();
                    panic!("message conversion panic");
                }
            }

            let failures = assert_that!(1).capture(|root| {
                assert!(
                    catch_unwind(|| {
                        root.add_detail_message(Message(&|| root.add_detail_message("nested")));
                    })
                    .is_err()
                );
                root.add_detail_message("after panic");
                root.is_equal_to(2)
            });
            assert_eq!(failures[0].messages, ["nested", "after panic"]);
        }

        #[test]
        #[cfg(feature = "std")]
        fn panic_assertions_still_accept_mutably_captured_state() {
            let mut value = 0;
            assert_that_owned!(|| {
                value = 1;
                panic!("closure panic");
            })
            .panics()
            .has_type::<&str>()
            .is_equal_to("closure panic");
            assert_eq!(value, 1);

            assert_that_owned!(|| {
                value = 2;
                value
            })
            .does_not_panic()
            .is_equal_to(2);
            assert_eq!(value, 2);
        }

        #[tokio::test]
        #[cfg(feature = "std")]
        async fn async_panic_assertions_still_accept_mutably_captured_state() {
            let mut value = 0;
            assert_that_panic_by_async(|| async {
                value = 1;
                panic!("async closure panic");
            })
            .await
            .has_type::<&str>()
            .is_equal_to("async closure panic");
            assert_eq!(value, 1);
        }
    }

    mod unwrap_inner {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn panics_on_borrowed_value_in_panic_mode() {
            let value = String::from("foo");
            let assert = assert_that!(&value).with_location(false).is_equal_to("foo");

            assert_that_panic_by(move || assert.unwrap_inner())
                .has_type::<&str>()
                .is_equal_to(formatdoc! {r"Cannot unwrap a borrowed value. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."});
        }

        #[test]
        fn succeeds_on_owned_value_in_panic_mode() {
            let assert = assert_that_owned!(42).with_location(false).is_equal_to(42);
            let actual = assert.unwrap_inner();
            assert_that!(actual).is_equal_to(42);
        }
    }
}
