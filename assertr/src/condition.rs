//! Reusable, named predicates usable with `is` / `has` / `are` / `have`.

use core::fmt::Display;

/// A reusable, named predicate.
///
/// Implement this trait on a type describing a domain property. Pass an instance to
/// [`is`](crate::assertions::condition::ConditionAssertions::is) or
/// [`has`](crate::assertions::condition::ConditionAssertions::has). Use
/// [`are`](crate::assertions::condition::IterableConditionAssertions::are) or
/// [`have`](crate::assertions::condition::IterableConditionAssertions::have) to apply it to every
/// element of an iterable subject.
///
/// Use the [`satisfies`](crate::AssertThat::satisfies) family for inline nested assertions and a
/// condition for a reusable domain predicate with a domain-specific error.
///
/// The `Assertr` prefix avoids collisions with other prelude traits named `Condition`.
///
/// Conditions are also implemented for references, so one instance can be reused by passing
/// `&condition`.
///
/// ```
/// use assertr::prelude::*;
///
/// struct IsEven;
///
/// impl AssertrCondition<i32> for IsEven {
///     type Error = String;
///
///     fn test(&self, value: &i32) -> Result<(), Self::Error> {
///         if value % 2 == 0 {
///             Ok(())
///         } else {
///             Err(format!("{value} is odd!"))
///         }
///     }
/// }
///
/// let even = IsEven;
/// assert_that!(2).is(&even);
/// assert_that!([2, 4, 6]).are(even);
/// ```
pub trait AssertrCondition<T> {
    /// Describes why a value did not match. On failure, exposed verbatim as an
    /// [`AssertionFailure::details`](crate::AssertionFailure::details) entry.
    type Error: Display;

    /// Tests whether `value` matches this condition.
    ///
    /// # Errors
    ///
    /// Returns an error describing why the value does not match the condition.
    fn test(&self, value: &T) -> Result<(), Self::Error>;
}

impl<T, C: AssertrCondition<T>> AssertrCondition<T> for &C {
    type Error = C::Error;

    fn test(&self, value: &T) -> Result<(), Self::Error> {
        C::test(self, value)
    }
}
