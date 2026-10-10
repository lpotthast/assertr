use core::ops::Deref;

use crate::{
    expectation::{AssertionContext, Expectation},
    failure::{FailureBuilder, FailureKind},
};

/// Explicitly dereferences an actual value before matching. No actual values are moved.
#[derive(Debug, Clone)]
pub struct Dereferenced<M>(M);

/// Adapts a matcher for subjects implementing [`Deref`], such as references, boxes, reference
/// counted pointers, or `String`, including reference-valued fields and iterator items.
#[must_use]
pub const fn dereferenced<M>(matcher: M) -> Dereferenced<M> {
    Dereferenced(matcher)
}

impl<T: Deref + ?Sized, R, M> Expectation<T, R> for Dereferenced<M>
where
    M: Expectation<T::Target, R>,
{
    type Success<'a>
        = <M as Expectation<T::Target, R>>::Success<'a>
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = <M as Expectation<T::Target, R>>::Rejection<'a>
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        self.0.evaluate(&**actual, context)
    }

    const KIND: FailureKind = <M as Expectation<T::Target, R>>::KIND;
    const FLATTEN: bool = <M as Expectation<T::Target, R>>::FLATTEN;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        self.0.explain(
            rejected.map(|(actual, rejection)| (&**actual, rejection)),
            failure,
            context,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, rc::Rc, string::String};

    use super::dereferenced;
    use crate::{assertions::core::partial_eq::eq, prelude::*};

    #[test]
    fn accepts_owned_references() {
        assert_that_owned!(&42).matches(dereferenced(eq(42)));
    }

    #[test]
    fn accepts_smart_pointers() {
        assert_that!(Box::new(42)).matches(dereferenced(eq(42)));
        assert_that!(Rc::new(42)).matches(dereferenced(eq(42)));
        let mut value = 42;
        assert_that_owned!(&mut value).matches(dereferenced(eq(42)));
    }

    #[test]
    fn accepts_strings_as_str() {
        assert_that!(String::from("hello")).matches(dereferenced(matchers::string::IsNotBlank));
    }

    #[test]
    fn reports_the_inner_failure() {
        let failures = assert_that!(Box::new(1))
            .with_location(false)
            .capture(|it| it.matches(dereferenced(eq(2))));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].kind).is_equal_to(crate::failure::FailureKind::Equality);
        assert_that!(failures[0].subject_type_name).is_equal_to("alloc::boxed::Box<i32>");
    }
}
