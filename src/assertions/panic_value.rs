use std::any::{Any, TypeId};

use crate::{actual::Actual, failure::GenericFailure, AssertThat, PanicValue};

/// Assertions for PanicValue's, the output of a panic occurred within a `assert_that_panic_by`.
pub trait PanicValueAssertions<'t> {
    fn has_type_derived<E: 'static>(&'t self) -> AssertThat<'t, Box<&'t E>>;
    fn has_type<E: 'static>(self) -> AssertThat<'t, E>;
}

impl<'t> PanicValueAssertions<'t> for AssertThat<'t, PanicValue> {
    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    fn has_type_derived<E: 'static>(&'t self) -> AssertThat<'t, Box<&'t E>> {
        let any = &self.actual.borrowed().0;
        match any.downcast_ref::<E>() {
            Some(casted) => self.derive(|_actual| Box::new(casted)),
            None => {
                self.fail(GenericFailure {
                    arguments: format_args!(
                        "is not of expected type: {expected_type_name} ({expected_type_id:?})",
                        expected_type_name = std::any::type_name::<E>(),
                        expected_type_id = TypeId::of::<E>(),
                    ),
                });
                panic!("Cannot continue in capturing mode!");
            }
        }
    }

    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    fn has_type<E: 'static>(self) -> AssertThat<'t, E> {
        self.map::<Box<dyn Any>>(|it| Actual::Owned(it.unwrap_owned().0))
            .has_type::<E>()
    }
}
