use std::fmt::{Debug, Display};

pub trait Failure<'a>: Display {}

pub struct GenericFailure<'a> {
    pub arguments: std::fmt::Arguments<'a>,
}

impl<'a> Failure<'a> for GenericFailure<'a> {}

impl<'a> Display for GenericFailure<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(self.arguments)
    }
}

pub struct ExpectedActualFailure<'e, 'a, E: Debug, A: Debug> {
    pub expected: &'e E,
    pub actual: &'a A,
}

impl<'f, 'e, 'a, E: Debug, A: Debug> Failure<'f> for ExpectedActualFailure<'e, 'a, E, A> {}

impl<'e, 'a, E: Debug, A: Debug> Display for ExpectedActualFailure<'e, 'a, E, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Expected: {:#?}", self.expected))?;
        f.write_str("\n\n")?;
        f.write_fmt(format_args!("Actual: {:#?}", self.actual))
    }
}
