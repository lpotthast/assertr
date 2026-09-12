use super::{ExpectationDiagnostics, Predicate, predicate};
use crate::{
    __private::{Cons, Nil},
    AssertionContext,
};
use alloc::vec::Vec;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// A supported heterogeneous or homogeneous list of matchers.
///
/// Implementations are sealed. Use `matchers!`, tuples of up to twelve matchers, arrays, slices, or
/// vectors. Lists store expectations without boxes or renderer type parameters.
pub trait MatcherList<A: ?Sized, R>: sealed::Sealed {
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
    H: ExpectationDiagnostics<A, R>,
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
            M: ExpectationDiagnostics<A, R>,
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

impl sealed::Sealed for () {}

impl<A: ?Sized, R> MatcherList<A, R> for () {
    fn len(&self) -> usize {
        0
    }

    fn describe_at(
        &self,
        index: usize,
        context: &AssertionContext<'_, R>,
    ) -> crate::AssertionFailure {
        <Nil as MatcherList<A, R>>::describe_at(&Nil, index, context)
    }

    fn evaluate_at(&self, index: usize, actual: &A, context: &mut AssertionContext<'_, R>) -> bool {
        <Nil as MatcherList<A, R>>::evaluate_at(&Nil, index, actual, context)
    }
}

macro_rules! tuple {
    ($length:expr; $($matcher:ident: $slot:tt),+) => {
        impl<$($matcher),+> sealed::Sealed for ($($matcher,)+) {}

        impl<A: ?Sized, R, $($matcher),+> MatcherList<A, R> for ($($matcher,)+)
        where
            $($matcher: ExpectationDiagnostics<A, R>),+
        {
            fn len(&self) -> usize {
                $length
            }

            fn describe_at(&self, index: usize, context: &AssertionContext<'_, R>) -> crate::AssertionFailure {
                match index {
                    $($slot => context.describe(&self.$slot),)+
                    _ => panic!("matcher slot out of bounds"),
                }
            }

            fn evaluate_at(
                &self,
                index: usize,
                actual: &A,
                context: &mut AssertionContext<'_, R>,
            ) -> bool {
                match index {
                    $($slot => context.evaluate(actual, &self.$slot),)+
                    _ => panic!("matcher slot out of bounds"),
                }
            }
        }
    };
}

tuple!(1; M0: 0);
tuple!(2; M0: 0, M1: 1);
tuple!(3; M0: 0, M1: 1, M2: 2);
tuple!(4; M0: 0, M1: 1, M2: 2, M3: 3);
tuple!(5; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4);
tuple!(6; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5);
tuple!(7; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6);
tuple!(8; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6, M7: 7);
tuple!(9; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6, M7: 7, M8: 8);
tuple!(10; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6, M7: 7, M8: 8, M9: 9);
tuple!(11; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6, M7: 7, M8: 8, M9: 9, M10: 10);
tuple!(12; M0: 0, M1: 1, M2: 2, M3: 3, M4: 4, M5: 5, M6: 6, M7: 7, M8: 8, M9: 9, M10: 10, M11: 11);

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
/// Use [`eq`](crate::matchers::eq) or [`equal_to`](crate::matchers::equal_to) for equality.
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
