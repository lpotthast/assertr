use std::any::{Any, TypeId};

use crate::{failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode};

/// Assertions for boxed values.
pub trait BoxAssertions<'t, M: Mode> {
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M>;
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M>;
}

impl<'t, M: Mode> BoxAssertions<'t, M> for AssertThat<'t, Box<dyn Any>, M> {
    /// If this fails in capturing mode, a panic is raised!
    #[track_caller]
    fn has_type_ref<E: 'static>(&'t self) -> AssertThat<'t, &'t E, M> {
        self.track_assertion();
        match self.actual().downcast_ref::<E>() {
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
    fn has_type<E: 'static>(self) -> AssertThat<'t, E, M> {
        self.track_assertion();

        enum CastResult<'c, C> {
            Owned(Box<C>),
            Ref(&'c C),
            Err(String),
        }

        let cast = match self.actual {
            crate::actual::Actual::Borrowed(b) => b
                .downcast_ref::<E>()
                .map(|it| CastResult::Ref(it))
                .unwrap_or_else(|| CastResult::Err(String::from("asd"))),
            crate::actual::Actual::Owned(o) => o
                .downcast::<E>()
                .map(|it| CastResult::Owned(it))
                .unwrap_or_else(|err| CastResult::Err(format!("{err:#?}"))),
        };

        match cast {
            CastResult::Owned(casted) => {
                AssertThat {
                    parent: self.parent,
                    actual: (*casted).into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    num_assertions: self.num_assertions,
                    failures: self.failures,
                    mode: self.mode,
                }
            }
            CastResult::Ref(casted) => {
                AssertThat {
                    parent: self.parent,
                    actual: casted.into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    num_assertions: self.num_assertions,
                    failures: self.failures,
                    mode: self.mode,
                }
            }
            CastResult::Err(err) => {
                AssertThat {
                    parent: self.parent,
                    actual: err.into(),
                    subject_name: self.subject_name, // We cannot clone self.subject_name, as the mapper produces what has to be considered a "new" subject!
                    detail_messages: self.detail_messages,
                    print_location: self.print_location,
                    num_assertions: self.num_assertions,
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
