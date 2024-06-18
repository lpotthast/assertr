use std::{
    any::{Any, TypeId},
    mem::ManuallyDrop,
};

use crate::{failure::GenericFailure, AssertThat};

/// Assertions for boxed values.
impl<'t> AssertThat<'t, Box<dyn Any>> {
    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    pub fn has_type_ref<E>(&'t self) -> AssertThat<'t, Box<&'t E>>
    where
        E: 'static,
    {
        let any = self.actual.borrowed();
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
    pub fn has_type<E: 'static>(mut self) -> AssertThat<'t, E> {
        let any: Box<dyn Any> = unsafe {
            // Safety: AssertThat's Drop impl does not use this field.
            ManuallyDrop::take(&mut self.actual).unwrap_owned()
        };
        match any.downcast::<E>() {
            Ok(casted) => self.map_with_actual_already_taken(move || (*casted).into()),
            Err(err) => {
                self.map_with_actual_already_taken(move || err.into())
                    .with_detail_message(format!(
                        "Panic value was not of type '{expected_type_name}'",
                        expected_type_name = std::any::type_name::<E>()
                    ))
                    .fail(GenericFailure {
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
}
