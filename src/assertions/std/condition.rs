use crate::{
    condition::{Condition, ConditionAssertions},
    failure::GenericFailure,
    tracking::AssertionTracking,
    AssertThat, Mode,
};

impl<'t, T, M: Mode> ConditionAssertions<T> for AssertThat<'t, T, M> {
    fn is<C: Condition<T>>(self, condition: C) -> Self {
        self.track_assertion();
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
