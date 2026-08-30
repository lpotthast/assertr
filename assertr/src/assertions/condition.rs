use alloc::string::ToString;

use crate::{AssertThat, Mode, condition::AssertrCondition};

/// Assertions that apply a reusable [`AssertrCondition`] to the subject.
///
/// `has` is a readability alias of `is`. For example, `is(alive)` and `has(name("Bob"))`.
///
/// With the `fluent` feature enabled, `be` is the fluent alias of `is`. For example,
/// `person.must().be(alive)`. `has` has no fluent alias because `have`
/// would be ambiguous with [`IterableConditionAssertions::have`] on iterable subjects.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ConditionAssertions<T> {
    /// Asserts that the subject matches the given condition.
    ///
    /// On failure, the condition's error is exposed verbatim as an
    /// [`AssertionFailure::details`](crate::AssertionFailure::details) entry.
    ///
    /// Pass `&condition` to keep the condition usable for further assertions.
    #[cfg_attr(feature = "fluent", fluent_alias("be"))]
    fn is<C: AssertrCondition<T>>(self, condition: C) -> Self;

    /// Readability synonym of [`is`](ConditionAssertions::is).
    fn has<C: AssertrCondition<T>>(self, condition: C) -> Self;
}

impl<T, M: Mode, R> ConditionAssertions<T> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is<C: AssertrCondition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        if let Err(err) = condition.test(self.actual()) {
            self.fail_with_details([err.to_string()], "Condition did not match.\n");
        }
        self
    }

    #[track_caller]
    fn has<C: AssertrCondition<T>>(self, condition: C) -> Self {
        self.is(condition)
    }
}

/// Assertions that apply a reusable condition to every element of an iterable subject.
///
/// Each non-matching element raises its own failure naming the element's zero-based index, so
/// capture mode reports every offending element.
///
/// `have` is a readability alias of `are`. It also serves as the fluent spelling because
/// `people.must().have(condition)` already reads imperatively.
#[allow(clippy::return_self_not_must_use)]
pub trait IterableConditionAssertions<T, I>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    /// Asserts that every element of the subject matches the given condition.
    ///
    /// On failure, each offending element's condition error is exposed verbatim as an
    /// [`AssertionFailure::details`](crate::AssertionFailure::details) entry of its own failure.
    fn are<C: AssertrCondition<T>>(self, condition: C) -> Self;

    /// Readability synonym of [`are`](IterableConditionAssertions::are).
    fn have<C: AssertrCondition<T>>(self, condition: C) -> Self;
}

impl<I, T, M: Mode, R> IterableConditionAssertions<T, I> for AssertThat<'_, I, M, R>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn are<C: AssertrCondition<T>>(self, condition: C) -> Self {
        self.track_assertion();
        for (index, actual) in self.actual().into_iter().enumerate() {
            if let Err(err) = condition.test(actual) {
                self.fail_with_details(
                    [err.to_string()],
                    format_args!(
                        "Condition did not match for the element at zero-based index {index}.\n"
                    ),
                );
            }
        }
        self
    }

    #[track_caller]
    fn have<C: AssertrCondition<T>>(self, condition: C) -> Self {
        self.are(condition)
    }
}
