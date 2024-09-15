use crate::{
    condition::Condition, failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode,
};

pub trait ConditionAssertions<T> {
    fn is<C: Condition<T>>(self, condition: C) -> Self;
    fn has<C: Condition<T>>(self, condition: C) -> Self;
}

impl<'t, T, M: Mode> ConditionAssertions<T> for AssertThat<'t, T, M> {
    fn is<C: Condition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        match condition.test(self.actual()) {
            Ok(()) => {}
            Err(err) => self.fail(GenericFailure {
                arguments: format_args!("Condition did not match:\n\n{err}"),
            }),
        }
        self
    }

    fn has<C: Condition<T>>(self, condition: C) -> Self {
        self.is(condition)
    }
}

pub trait IterableConditionAssertions<T, I>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    fn are<C: Condition<T>>(self, condition: C) -> Self;
    fn have<C: Condition<T>>(self, condition: C) -> Self;
}

impl<'t, I, T, M: Mode> IterableConditionAssertions<T, I> for AssertThat<'t, I, M>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    fn are<C: Condition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        let iter = self.actual().into_iter();
        for actual in iter {
            match condition.test(actual) {
                Ok(()) => {}
                Err(err) => self.fail(GenericFailure {
                    arguments: format_args!("Condition did not match:\n\n{err}"),
                }),
            }
        }
        self
    }

    fn have<C: Condition<T>>(self, condition: C) -> Self {
        self.are(condition)
    }
}
