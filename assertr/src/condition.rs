use std::fmt::Arguments;

pub trait Condition<T> {
    /// `value` can be considered to be the "expected" value.
    fn test<'a>(&self, value: &T) -> Result<(), Arguments<'a>>;
}

pub trait ConditionAssertions<T> {
    fn is<C: Condition<T>>(self, condition: C) -> Self;
    fn has<C: Condition<T>>(self, condition: C) -> Self;
}

// TODO: implement or consider removal
pub trait CollectionConditionAssertions<T> {
    fn are<C: Condition<T>>(self, condition: C) -> Self;
    fn have<C: Condition<T>>(self, condition: C) -> Self;
}
