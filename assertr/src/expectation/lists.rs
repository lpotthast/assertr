use super::{Expectation, Predicate, predicate};
use crate::{
    __private::{Cons, Nil},
    AssertionContext,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// A supported heterogeneous or homogeneous list of matchers.
///
/// Implementations are sealed. Use [`matchers!`](crate::matchers!) to mix matcher types, or arrays,
/// slices, and vectors of one matcher type. Lists store expectations without boxes or renderer type
/// parameters.
pub trait MatcherList<A: ?Sized, R = crate::DebugRenderer>: sealed::Sealed {
    /// Number of constraints.
    fn len(&self) -> usize;

    /// Whether this list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Describes one expectation slot. The slot must be less than `len()`.
    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure;

    /// Evaluates one expectation slot. The slot must be less than `len()`.
    fn evaluate_at(&self, index: usize, actual: &A, context: &mut AssertionContext<'_, R>) -> bool;
}

/// A borrowed callback list that adapts only the slot being evaluated or described.
pub(crate) struct SatisfyingList<L, F>(L, PhantomData<fn() -> F>);

impl<L: AsRef<[F]>, F> SatisfyingList<L, F> {
    /// Stores the callbacks without accessing them, so tracking precedes the first borrow.
    pub(crate) fn new(callbacks: L) -> Self {
        Self(callbacks, PhantomData)
    }
}

impl<L, F> sealed::Sealed for SatisfyingList<L, F> {}

impl<A, R: Clone, L: AsRef<[F]>, F> MatcherList<A, R> for SatisfyingList<L, F>
where
    F: for<'a> Fn(crate::AssertThat<'a, A, crate::mode::Capture, R>),
{
    fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure {
        context.describe(&super::satisfying(&self.0.as_ref()[index]))
    }

    fn evaluate_at(&self, index: usize, actual: &A, context: &mut AssertionContext<'_, R>) -> bool {
        context.evaluate(actual, &super::satisfying(&self.0.as_ref()[index]))
    }
}

impl sealed::Sealed for Nil {}

impl<H, T> sealed::Sealed for Cons<H, T> {}

impl<A: ?Sized, R> MatcherList<A, R> for Nil {
    fn len(&self) -> usize {
        0
    }

    fn describe_at(&self, _: usize, _: &AssertionContext<'_, R>) -> crate::AssertionFailure {
        panic!("empty matcher list")
    }

    fn evaluate_at(&self, _: usize, _: &A, _: &mut AssertionContext<'_, R>) -> bool {
        panic!("empty matcher list")
    }
}

impl<A: ?Sized, R, H, T> MatcherList<A, R> for Cons<H, T>
where
    H: Expectation<A, R>,
    T: MatcherList<A, R>,
{
    fn len(&self) -> usize {
        1 + self.1.len()
    }

    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure {
        if index == 0 {
            context.describe(&self.0)
        } else {
            self.1.describe_at(index - 1, context)
        }
    }

    fn evaluate_at(&self, index: usize, actual: &A, context: &mut AssertionContext<'_, R>) -> bool {
        if index == 0 {
            context.evaluate(actual, &self.0)
        } else {
            self.1.evaluate_at(index - 1, actual, context)
        }
    }
}

macro_rules! homogeneous {
    ($type:ty $(, $size:ident)?) => {
        impl<M $(, const $size: usize)?> sealed::Sealed for $type {}

        impl<A: ?Sized, R, M $(, const $size: usize)?> MatcherList<A, R> for $type
        where
            M: Expectation<A, R>,
        {
            fn len(&self) -> usize {
                <[M]>::len(self)
            }

            fn describe_at(&self, index: usize, context: &AssertionContext<'_, R>) -> crate::AssertionFailure {
                context.describe(&self[index])
            }

            fn evaluate_at(
                &self,
                index: usize,
                actual: &A,
                context: &mut AssertionContext<'_, R>,
            ) -> bool {
                context.evaluate(actual, &self[index])
            }
        }
    };
}

homogeneous!([M]);
homogeneous!(Vec<M>);
homogeneous!([M; N], N);

impl<L: ?Sized> sealed::Sealed for &L {}

impl<A: ?Sized, R, L> MatcherList<A, R> for &L
where
    L: MatcherList<A, R> + ?Sized,
{
    fn len(&self) -> usize {
        (**self).len()
    }

    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure {
        (**self).describe_at(index, context)
    }

    fn evaluate_at(&self, index: usize, actual: &A, context: &mut AssertionContext<'_, R>) -> bool {
        (**self).evaluate_at(index, actual, context)
    }
}

/// Converts a homogeneous iterable of boolean predicates to a reusable matcher list. For
/// heterogeneous closures, use `matchers![predicate(...), predicate(...)]`.
pub fn predicate_list<A, F, I>(predicates: I) -> Vec<Predicate<F>>
where
    F: Fn(&A) -> bool,
    I: IntoIterator<Item = F>,
{
    predicates.into_iter().map(predicate).collect()
}

/// Constructs a reusable heterogeneous matcher list from explicit expectations.
///
/// Use [`eq`](crate::matchers::eq) or [`equal_to`](crate::matchers::equal_to) for equality. The
/// list's type is an unsupported implementation detail. Let inference pick it, or name the
/// enclosing check as `impl Expectation<T>` or the list as `impl MatcherList<T>`:
///
/// ```
/// use assertr::matchers::{Expectation, all_of, eq, gt};
/// use assertr::prelude::*;
///
/// fn positive_two() -> impl Expectation<i32> + Clone {
///     all_of(matchers![gt(0), eq(2)])
/// }
///
/// assert_that!(2).matches(positive_two());
/// ```
#[macro_export]
macro_rules! matchers {
    (@list) => {
        $crate::__private::Nil
    };
    (@list $head:expr $(, $tail:expr)* $(,)?) => {
        $crate::__private::Cons(
            $head,
            $crate::matchers!(@list $($tail),*)
        )
    };
    ($($value:expr),* $(,)?) => {
        $crate::matchers!(@list $($value),*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use core::cell::Cell;

    #[test]
    fn borrowed_callbacks_need_no_clone_or_item_renderer_and_descriptions_do_not_invoke_them() {
        struct Opaque;
        struct NotClone;
        #[derive(Clone)]
        struct NoRenderer;
        let not_clone = NotClone;
        let calls = Cell::new(0);
        let calls_ref = &calls;
        let callbacks = [move |it: AssertThat<'_, Opaque, Capture, NoRenderer>| {
            core::hint::black_box(&not_clone);
            calls_ref.set(calls_ref.get() + 1);
            it.derive_owned(|_| true)
                .with_renderer(DebugRenderer)
                .is_true();
        }];
        let list = SatisfyingList::new(&callbacks);
        let mut context = AssertionContext::new(&NoRenderer, RenderingBudget::default());
        assert_that!(list.len()).is_equal_to(1);
        let description = list.describe_at(0, &context);
        assert_that!(description.relation.as_deref()).is_equal_to(Some("satisfies the assertions"));
        assert_that!(calls.get()).is_equal_to(0);
        assert_that!(list.evaluate_at(0, &Opaque, &mut context)).is_true();
        assert_that!(calls.get()).is_equal_to(1);
    }
}
