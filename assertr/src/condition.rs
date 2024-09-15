use core::fmt::Display;

pub trait Condition<T> {
    type Error: Display;

    /// Test that the actual `value` conforms to / matches this condition (`self`).
    fn test<'a>(&self, value: &T) -> Result<(), Self::Error>;
}
