//! Sequential composition of typed adapters.
//!
//! [`Then`] runs the first adapter, borrows its successful output as the next adapter's input, and
//! returns the next adapter's output. [`ThenError`] retains the typed error and identifies the
//! stage that failed. An error from the first stage prevents the next stage from running.
//!
//! [`AdapterExt::then`](crate::failure::adapter::AdapterExt::then) constructs these chains. They
//! run on the calling thread, work without `std`, and propagate panics to their caller. Composition
//! does not choose how assertion failures are captured or presented in a panic.

use core::{error::Error, fmt};

use super::super::Adapter;

/// A linear composition that passes the first adapter's output to the next adapter.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Then<First, Next> {
    first: First,
    next: Next,
}

impl<First, Next> Then<First, Next> {
    /// Creates a linear composition.
    pub const fn new(first: First, next: Next) -> Self {
        Self { first, next }
    }
}

impl<Input: ?Sized, First, Next> Adapter<Input> for Then<First, Next>
where
    First: Adapter<Input>,
    Next: Adapter<First::Output>,
{
    type Output = Next::Output;
    type Error = ThenError<First::Error, Next::Error>;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        let intermediate = self.first.adapt(input).map_err(ThenError::First)?;
        self.next.adapt(&intermediate).map_err(ThenError::Next)
    }
}

/// Identifies which stage of a [`Then`] composition failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThenError<FirstError, NextError> {
    /// The first adapter failed, so the next adapter was not run.
    First(FirstError),
    /// The next adapter failed after the first adapter succeeded.
    Next(NextError),
}

/// Names the failed stage only, such as `first adapter failed`. The stage's own error is available
/// through [`Error::source`], so error reports walking the source chain print it exactly once.
///
/// The alternate form (`{:#}`) appends the stage's error in its own alternate form, such as
/// `first adapter failed: invalid input`, for contexts that only keep one message.
impl<FirstError: fmt::Display, NextError: fmt::Display> fmt::Display
    for ThenError<FirstError, NextError>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternate = f.alternate();
        match self {
            Self::First(error) => {
                f.write_str("first adapter failed")?;
                if alternate {
                    write!(f, ": {error:#}")?;
                }
            }
            Self::Next(error) => {
                f.write_str("next adapter failed")?;
                if alternate {
                    write!(f, ": {error:#}")?;
                }
            }
        }
        Ok(())
    }
}

impl<FirstError: Error + 'static, NextError: Error + 'static> Error
    for ThenError<FirstError, NextError>
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::First(error) => Some(error),
            Self::Next(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use alloc::{format, rc::Rc, string::ToString, vec::Vec};
    use core::{cell::RefCell, error::Error, fmt};

    use super::*;
    use crate::failure::adapter::AdapterExt;

    #[derive(Debug)]
    struct ExampleError;

    impl fmt::Display for ExampleError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("example error")
        }
    }

    impl Error for ExampleError {}

    #[test]
    fn then_error_preserves_the_source_of_either_stage() {
        for error in [
            ThenError::First(ExampleError),
            ThenError::Next(ExampleError),
        ] {
            assert_that!(error.source().unwrap().downcast_ref::<ExampleError>()).is_some();
        }
    }

    #[test]
    fn then_error_display_names_the_stage_without_repeating_its_source() {
        let first: ThenError<ExampleError, ExampleError> = ThenError::First(ExampleError);
        let next: ThenError<ExampleError, ExampleError> = ThenError::Next(ExampleError);

        assert_that!(first.to_string()).is_equal_to("first adapter failed");
        assert_that!(next.to_string()).is_equal_to("next adapter failed");
    }

    #[test]
    fn then_error_alternate_display_appends_the_nested_errors() {
        let nested: ThenError<ThenError<ExampleError, ExampleError>, ExampleError> =
            ThenError::First(ThenError::Next(ExampleError));

        assert_that!(format!("{nested:#}"))
            .is_equal_to("first adapter failed: next adapter failed: example error");
    }

    #[derive(Clone)]
    struct NumberStep {
        events: Rc<RefCell<Vec<&'static str>>>,
        name: &'static str,
        increment: u8,
        error: Option<&'static str>,
    }

    impl Adapter<u8> for NumberStep {
        type Output = u8;
        type Error = &'static str;

        fn adapt(&self, input: &u8) -> Result<Self::Output, Self::Error> {
            self.events.borrow_mut().push(self.name);
            match self.error {
                Some(error) => Err(error),
                None => Ok(*input + self.increment),
            }
        }
    }

    #[test]
    fn then_passes_the_output_forward_and_stops_at_the_first_error() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let first = NumberStep {
            events: Rc::clone(&events),
            name: "first",
            increment: 1,
            error: None,
        };
        let second = NumberStep {
            events: Rc::clone(&events),
            name: "second",
            increment: 2,
            error: None,
        };

        assert_that!(first.clone().then(second).adapt(&3)).is_equal_to(Ok(6));
        assert_that!(*events.borrow()).contains_exactly(["first", "second"]);

        events.borrow_mut().clear();
        let failing = NumberStep {
            error: Some("no value"),
            ..first
        };
        let result = failing
            .then(NumberStep {
                events: Rc::clone(&events),
                name: "not run",
                increment: 0,
                error: None,
            })
            .adapt(&3);

        assert_that!(result).is_equal_to(Err(ThenError::First("no value")));
        assert_that!(*events.borrow()).contains_exactly(["first"]);
    }

    #[test]
    fn then_preserves_the_second_error() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let first = NumberStep {
            events: Rc::clone(&events),
            name: "first",
            increment: 1,
            error: None,
        };
        let second = NumberStep {
            events: Rc::clone(&events),
            name: "second",
            increment: 0,
            error: Some("second failed"),
        };

        assert_that!(first.then(second).adapt(&3))
            .is_equal_to(Err(ThenError::Next("second failed")));
        assert_that!(*events.borrow()).contains_exactly(["first", "second"]);
    }
}
