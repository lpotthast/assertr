use crate::{failure::GenericFailure, AssertThat};
use std::any::{Any, TypeId};

/// Assertions for `Any` types.
impl<'t, T: Any> AssertThat<'t, T> {
    #[track_caller]
    pub fn has_type<E>(&self) -> AssertThat<E>
    where
        E: 'static,
    {
        let any = self.actual.borrowed() as &dyn Any;
        match any.downcast_ref::<E>() {
            Some(casted) => AssertThat {
                // TODO: use mapping?
                actual: casted.into(),
                print_location: self.print_location,
                additional_messages: Vec::new(),
            },
            None => {
                self.fail_with(GenericFailure {
                    arguments: format_args!(
                        "actual: {actual_type_name} ({actual_type_id:?}) \n\nis not of expected type: {expected_type_name} ({expected_type_id:?})",
                        actual_type_name = std::any::type_name::<T>(),
                        actual_type_id = TypeId::of::<T>(),
                        expected_type_name = std::any::type_name::<E>(),
                        expected_type_id = TypeId::of::<E>(),
                    ),
                });
            }
        }
    }
}

/// Assertions for `Box<dyn Any + Send>`.
impl<'t> AssertThat<'t, Box<dyn Any + Send>> {
    #[track_caller]
    pub fn has_box_type<E>(&'t self) -> AssertThat<'t, Box<&'t E>>
    where
        E: 'static,
    {
        let any = self.actual.borrowed();
        match any.downcast_ref::<E>() {
            Some(casted) => AssertThat {
                // TODO: use mapping?
                actual: Box::new(casted).into(),
                print_location: self.print_location,
                additional_messages: Vec::new(),
            },
            None => {
                self.fail_with(GenericFailure {
                    arguments: format_args!(
                        "is not of expected type: {expected_type_name} ({expected_type_id:?})",
                        expected_type_name = std::any::type_name::<E>(),
                        expected_type_id = TypeId::of::<E>(),
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use crate::{assert_that, AssertThat};

    #[test]
    fn test_any_has_type() {
        assert_that(String::from("ok")).has_type::<String>();
        let any = &42i32 as &dyn Any;

        assert!(any.downcast_ref::<i32>().is_some());

        eprintln!("i32 has type id: {:?}", TypeId::of::<i32>());
        eprintln!(
            "AssertThat has type id: {:?}",
            (AssertThat {
                actual: 42.into(),
                print_location: true,
                additional_messages: Vec::new(),
            }
            .actual
            .borrowed() as &dyn Any)
                .type_id()
        );
        eprintln!(
            "AssertThat has type id: {:?}",
            (AssertThat {
                actual: any.into(),
                print_location: true,
                additional_messages: Vec::new(),
            }
            .actual
            .borrowed() as &dyn Any)
                .type_id()
        );

        assert_eq!(
            (AssertThat {
                actual: 42.into(),
                print_location: true,
                additional_messages: Vec::new(),
            }
            .actual
            .borrowed() as &dyn Any)
                .type_id(),
            TypeId::of::<i32>()
        );

        assert_that(42).has_type::<i32>();
    }
}
