use std::any::{Any, TypeId};

use crate::{failure::GenericFailure, AssertThat, Mode};

/// Assertions for boxed values.
impl<'t, M: Mode> AssertThat<'t, Box<dyn Any>, M> {
    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    pub fn has_type_ref<E>(&'t self) -> AssertThat<'t, &'t E, M>
    where
        E: 'static,
    {
        let any = self.actual();
        match any.downcast_ref::<E>() {
            Some(casted) => self.derive(|_actual| casted),
            None => {
                self.fail(GenericFailure {
                    arguments: format_args!(
                        "is not of expected type: {expected_type_name} ({expected_type_id:?})",
                        expected_type_name = std::any::type_name::<E>(),
                        expected_type_id = TypeId::of::<E>(),
                    ),
                });
                panic!("Cannot continue in capturing mode!"); // TODO: Consider typestates!
            }
        }
    }

    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    pub fn has_type<E: 'static>(self) -> AssertThat<'t, E, M> {
        // TODO: Remove unsafe!
        let any: Box<dyn Any> = self.actual.unwrap_owned();

        match any.downcast::<E>() {
            Ok(casted) => {
                AssertThat {
                    actual: (*casted).into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    failures: self.failures,
                    mode: self.mode,
                }
            }
            Err(err) => {
                AssertThat {
                    actual: err.into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    failures: self.failures,
                    mode: self.mode,
                }
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
                panic!("Cannot continue in capturing mode!"); // TODO: Consider typestates!
            }
        }
    }
}
