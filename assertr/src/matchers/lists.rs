use super::{AssertrMatcher, Description, MatchContext, MatchResult, Predicate, predicate};
use alloc::vec::Vec;

pub(super) mod sealed {
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
    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description;

    /// Evaluates one expectation slot. The slot must be less than `len()`.
    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult;
}

/// Wrapper constructed by `matchers!`. The internal list representation is unsupported.
pub struct MatcherSequence<L>(L);

#[doc(hidden)]
pub fn matcher_sequence<L>(list: L) -> MatcherSequence<L> {
    MatcherSequence(list)
}

impl<L> sealed::Sealed for MatcherSequence<L> {}

impl<A: ?Sized, R, L> MatcherList<A, R> for MatcherSequence<L>
where
    L: MatcherList<A, R>,
{
    fn len(&self) -> usize {
        self.0.len()
    }

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        self.0.describe_at(index, context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        self.0.evaluate_at(index, actual, context)
    }
}

#[doc(hidden)]
pub struct Nil;

#[doc(hidden)]
pub struct Cons<H, T>(pub H, pub T);

impl sealed::Sealed for Nil {}

impl<H, T> sealed::Sealed for Cons<H, T> {}

impl<A: ?Sized, R> MatcherList<A, R> for Nil {
    fn len(&self) -> usize {
        0
    }

    fn describe_at(&self, _: usize, _: &MatchContext<'_, R>) -> Description {
        panic!("empty matcher list")
    }

    fn evaluate_at(&self, _: usize, _: &A, _: &mut MatchContext<'_, R>) -> MatchResult {
        panic!("empty matcher list")
    }
}

impl<A: ?Sized, R, H, T> MatcherList<A, R> for Cons<H, T>
where
    H: AssertrMatcher<A, R>,
    T: MatcherList<A, R>,
{
    fn len(&self) -> usize {
        1 + self.1.len()
    }

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        if index == 0 {
            self.0.describe(context)
        } else {
            self.1.describe_at(index - 1, context)
        }
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        if index == 0 {
            self.0.evaluate(actual, context)
        } else {
            self.1.evaluate_at(index - 1, actual, context)
        }
    }
}

impl<M> sealed::Sealed for [M] {}

impl<A: ?Sized, R, M> MatcherList<A, R> for [M]
where
    M: AssertrMatcher<A, R>,
{
    fn len(&self) -> usize {
        <[M]>::len(self)
    }

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        self[index].describe(context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        self[index].evaluate(actual, context)
    }
}

macro_rules! homogeneous {
    ($type:ty $(, $size:ident)?) => {
        impl<M $(, const $size: usize)?> sealed::Sealed for $type {}

        impl<A: ?Sized, R, M $(, const $size: usize)?> MatcherList<A, R> for $type
        where
            M: AssertrMatcher<A, R>,
        {
            fn len(&self) -> usize {
                self.as_slice().len()
            }

            fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
                self[index].describe(context)
            }

            fn evaluate_at(
                &self,
                index: usize,
                actual: &A,
                context: &mut MatchContext<'_, R>,
            ) -> MatchResult {
                self[index].evaluate(actual, context)
            }
        }
    };
}

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

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        (**self).describe_at(index, context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        (**self).evaluate_at(index, actual, context)
    }
}

impl sealed::Sealed for () {}

impl<A: ?Sized, R> MatcherList<A, R> for () {
    fn len(&self) -> usize {
        0
    }

    fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
        <Nil as MatcherList<A, R>>::describe_at(&Nil, index, context)
    }

    fn evaluate_at(
        &self,
        index: usize,
        actual: &A,
        context: &mut MatchContext<'_, R>,
    ) -> MatchResult {
        Nil.evaluate_at(index, actual, context)
    }
}

macro_rules! tuple {
    ($length:expr; $($matcher:ident: $slot:tt),+) => {
        impl<$($matcher),+> sealed::Sealed for ($($matcher,)+) {}

        impl<A: ?Sized, R, $($matcher),+> MatcherList<A, R> for ($($matcher,)+)
        where
            $($matcher: AssertrMatcher<A, R>),+
        {
            fn len(&self) -> usize {
                $length
            }

            fn describe_at(&self, index: usize, context: &MatchContext<'_, R>) -> Description {
                match index {
                    $($slot => self.$slot.describe(context),)+
                    _ => panic!("matcher slot out of bounds"),
                }
            }

            fn evaluate_at(
                &self,
                index: usize,
                actual: &A,
                context: &mut MatchContext<'_, R>,
            ) -> MatchResult {
                match index {
                    $($slot => self.$slot.evaluate(actual, context),)+
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

/// Constructs a reusable heterogeneous matcher list. Bare expressions mean equality.
#[macro_export]
macro_rules! matchers {
    (@list) => {
        $crate::__private::Nil
    };
    (@list $head:expr $(, $tail:expr)* $(,)?) => {
        $crate::__private::Cons(
            $crate::__private::normalize($head),
            $crate::matchers!(@list $($tail),*)
        )
    };
    ($($value:expr),* $(,)?) => {
        $crate::__private::matcher_sequence($crate::matchers!(@list $($value),*))
    };
}
