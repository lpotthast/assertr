use crate::{AssertThat, Mode, condition::Condition, tracking::AssertionTracking};

pub trait ConditionAssertions<T> {
    fn is<C: Condition<T>>(self, condition: C) -> Self;
    fn has<C: Condition<T>>(self, condition: C) -> Self;
}

impl<T, M: Mode> ConditionAssertions<T> for AssertThat<'_, T, M> {
    #[track_caller]
    fn is<C: Condition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        match condition.test(self.actual()) {
            Ok(()) => {}
            Err(err) => self.fail(format_args!("Condition did not match:\n\n{err}\n")),
        }
        self
    }

    #[track_caller]
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

impl<I, T, M: Mode> IterableConditionAssertions<T, I> for AssertThat<'_, I, M>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn are<C: Condition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        let iter = self.actual().into_iter();
        for actual in iter {
            match condition.test(actual) {
                Ok(()) => {}
                Err(err) => self.fail(format_args!("Condition did not match:\n\n{err}\n")),
            }
        }
        self
    }

    #[track_caller]
    fn have<C: Condition<T>>(self, condition: C) -> Self {
        self.are(condition)
    }
}
