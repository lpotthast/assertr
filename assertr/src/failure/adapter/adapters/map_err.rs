//! Error conversion for typed adapters.

use super::super::Adapter;

/// An adapter that maps errors while preserving successful output.
///
/// Construct it with [`AdapterExt::map_err`](crate::failure::adapter::AdapterExt::map_err).
/// The mapper runs on each error from the wrapped adapter, on the calling thread. It does not
/// run during construction or on success. Both the adapter and mapper may borrow local data.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct MapErr<A, F> {
    adapter: A,
    mapper: F,
}

impl<A, F> MapErr<A, F> {
    /// Creates an adapter with the given error mapper.
    pub const fn new(adapter: A, mapper: F) -> Self {
        Self { adapter, mapper }
    }
}

impl<Input: ?Sized, A, F, Error> Adapter<Input> for MapErr<A, F>
where
    A: Adapter<Input>,
    F: Fn(A::Error) -> Error,
{
    type Output = A::Output;
    type Error = Error;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        self.adapter.adapt(input).map_err(&self.mapper)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use core::cell::Cell;

    use super::*;
    use crate::failure::adapter::{AdapterExt, ThenError};

    // Neither error type needs Display, Error, or Clone.
    #[derive(Debug, PartialEq)]
    struct Empty;

    #[derive(Debug, PartialEq)]
    struct Rejected;

    struct Length;

    impl Adapter<str> for Length {
        type Output = usize;
        type Error = Empty;

        fn adapt(&self, input: &str) -> Result<usize, Empty> {
            if input.is_empty() {
                Err(Empty)
            } else {
                Ok(input.len())
            }
        }
    }

    #[test]
    fn the_mapper_runs_only_on_errors_and_preserves_successful_output() {
        let calls = Cell::new(0);
        let adapter = Length.map_err(|error| {
            assert_eq!(error, Empty);
            calls.set(calls.get() + 1);
            Rejected
        });

        assert_eq!(calls.get(), 0);
        assert_eq!(adapter.adapt("hello"), Ok(5));
        assert_eq!(calls.get(), 0);
        assert_eq!(adapter.adapt(""), Err(Rejected));
        assert_eq!(adapter.adapt(""), Err(Rejected));
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn a_borrowed_trait_object_can_map_errors_to_borrowed_data() {
        let message = String::from("input is empty");
        let original: &dyn Adapter<str, Output = usize, Error = Empty> = &Length;
        let mapped = original.map_err(|_| message.as_str());
        let adapter: &dyn Adapter<str, Output = usize, Error = &str> = &mapped;

        assert_eq!(adapter.adapt("hello"), Ok(5));
        assert_eq!(adapter.adapt(""), Err(message.as_str()));
        assert_eq!(original.adapt(""), Err(Empty));
    }

    #[test]
    fn an_error_mapper_can_follow_a_chain_or_precede_its_next_stage() {
        struct RejectLength;

        impl Adapter<usize> for RejectLength {
            type Output = ();
            type Error = Rejected;

            fn adapt(&self, _: &usize) -> Result<(), Rejected> {
                Err(Rejected)
            }
        }

        let mapped_first = Length.map_err(|_| Rejected).then(RejectLength);
        assert_eq!(mapped_first.adapt(""), Err(ThenError::First(Rejected)));
        assert_eq!(mapped_first.adapt("hello"), Err(ThenError::Next(Rejected)));

        let mapped_chain = Length.then(RejectLength).map_err(|error| match error {
            ThenError::First(Empty) => "empty input",
            ThenError::Next(Rejected) => "length rejected",
        });
        assert_eq!(mapped_chain.adapt(""), Err("empty input"));
        assert_eq!(mapped_chain.adapt("hello"), Err("length rejected"));
    }
}
