use crate::{
    condition::{Condition, ConditionAssertions},
    failure::GenericFailure,
    AssertThat,
};

impl<'t, T> ConditionAssertions<T> for AssertThat<'t, T> {
    fn is<C: Condition<T>>(self, condition: C) -> Self {
        match condition.test(self.actual()) {
            Ok(()) => {}
            Err(arguments) => self.fail(GenericFailure {
                arguments: format_args!("Condition did not match:\n\n{arguments}"),
            }),
        }
        self
    }

    fn has<C: Condition<T>>(self, condition: C) -> Self {
        self.is(condition)
    }
}
