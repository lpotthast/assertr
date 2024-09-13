use alloc::boxed::Box;
use core::any::{type_name, Any, TypeId};

use crate::{
    actual::Actual, failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode,
    PanicValue,
};

use super::boxed::BoxAssertions;

/// Assertions for PanicValue's, the output of a panic occurred within a `assert_that_panic_by`.
pub trait PanicValueAssertions<'t, M: Mode> {
    /// NOTE: If this fails in capturing mode, a panic is raised!
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M>;
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M>;
}

impl<'t, M: Mode> PanicValueAssertions<'t, M> for AssertThat<'t, PanicValue, M> {
    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M> {
        self.track_assertion();

        let any = &self.actual().0;
        match any.downcast_ref::<E>() {
            Some(casted) => self.derive(|_actual| casted),
            None => {
                self.fail(GenericFailure {
                    arguments: format_args!(
                        "is not of expected type: {expected_type_name} ({expected_type_id:?})",
                        expected_type_name = type_name::<E>(),
                        expected_type_id = TypeId::of::<E>(),
                    ),
                });
                panic!("Cannot continue in capturing mode!"); // Consider typestates!
            }
        }
    }

    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M> {
        self.map::<Box<dyn Any>>(|it| match it {
            Actual::Borrowed(b) => Actual::Borrowed(&b.0),
            Actual::Owned(o) => Actual::Owned(o.0),
        })
        .has_type::<E>()
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, PanicValue};

    #[test]
    fn has_type_ref_succeeds_when_type_matches() {
        let actual = PanicValue(Box::new(String::from("foo")));

        assert_that(actual)
            .has_type_ref::<String>()
            .is_equal_to(&String::from("foo"));
    }

    #[test]
    fn has_type_succeeds_when_type_matches() {
        let actual: PanicValue = PanicValue(Box::new(String::from("foo")));

        assert_that::<'_, PanicValue>(&actual)
            .has_type::<String>()
            .is_equal_to(String::from("foo"));

        assert_that(actual)
            .has_type::<String>()
            .is_equal_to(String::from("foo"));
    }
}
