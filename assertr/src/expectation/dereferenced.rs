use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    failure::{FailureBuilder, FailureKind},
};
use core::ops::Deref;

/// Explicitly dereferences an actual value before matching. No actual values are moved.
pub struct Dereferenced<M>(M);

/// Adapts a matcher for subjects implementing [`Deref`], such as references, boxes, reference
/// counted pointers, or `String`, including reference-valued fields and iterator items.
pub fn dereferenced<M>(matcher: M) -> Dereferenced<M> {
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
}
impl<T: Deref + ?Sized, R, M> ExpectationDiagnostics<T, R> for Dereferenced<M>
where
    M: ExpectationDiagnostics<T::Target, R>,
{
    const KIND: FailureKind = <M as ExpectationDiagnostics<T::Target, R>>::KIND;
    const FLATTEN: bool = <M as ExpectationDiagnostics<T::Target, R>>::FLATTEN;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        self.0.explain(
            rejected.map(|(actual, rejection)| (&**actual, rejection)),
            failure,
            context,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::dereferenced;
    use crate::{assertions::core::partial_eq::equal_to, prelude::*};
    use alloc::{boxed::Box, rc::Rc, string::String};

    #[test]
    fn accepts_owned_references() {
        assert_that_owned!(&42).matches(dereferenced(equal_to(42)));
    }

    #[test]
    fn accepts_smart_pointers() {
        assert_that!(Box::new(42)).matches(dereferenced(equal_to(42)));
        assert_that!(Rc::new(42)).matches(dereferenced(equal_to(42)));
        let mut value = 42;
        assert_that_owned!(&mut value).matches(dereferenced(equal_to(42)));
    }

    #[test]
    fn accepts_strings_as_str() {
        assert_that!(String::from("hello")).matches(dereferenced(matchers::string::IsNotBlank));
    }

    #[test]
    fn reports_the_inner_failure() {
        let failures = assert_that!(Box::new(1))
            .with_location(false)
            .capture(|it| it.matches(dereferenced(equal_to(2))));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0].kind).is_equal_to(crate::FailureKind::Equality);
        assert_that!(failures[0].subject_type_name).is_equal_to("alloc::boxed::Box<i32>");
    }
}
