use core::fmt::Display;

pub trait Condition<T> {
    type Error: Display;

    /// Test that the actual `value` conforms to / matches this condition (`self`).
    ///
    /// # Errors
    ///
    /// Returns an error describing why the value does not match the condition.
    fn test(&self, value: &T) -> Result<(), Self::Error>;
}
