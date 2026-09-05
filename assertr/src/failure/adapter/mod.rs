//! Typed adapters for structured assertion failures and other representations.
//!
//! [`Adapter`] transforms a borrowed input into an owned output, and [`AdapterExt::then`] chains
//! transformations. Adapters retain their typed outputs and errors and run on the calling thread.
//! [`AdapterExt::map_err`] changes an adapter's error type without changing its successful output.
//! Adapters do not choose between capture and panic mode.
//!
//! [`ToHumanReadableText`] is the default panic presentation. An assertion context can select
//! another text-producing adapter through
//! [`with_panic_presentation`](crate::AssertThat::with_panic_presentation), which owns a `'static`
//! adapter and converts its errors to strings internally. Derived assertions share that adapter
//! without requiring it to implement `Clone`. Adapters used explicitly may still borrow local data.
//! Capture mode retains structured failures without invoking presentation. Captured failures can
//! be passed explicitly to any adapter, including chains with non-text outputs or side effects.
//!
//! ```
//! use core::convert::Infallible;
//! use assertr::failure::adapter::{
//!     Adapter, AdapterExt, HumanReadableText, ToHumanReadableText,
//! };
//! use assertr::prelude::*;
//!
//! struct TextLength;
//!
//! impl Adapter<HumanReadableText> for TextLength {
//!     type Output = usize;
//!     type Error = Infallible;
//!
//!     fn adapt(&self, text: &HumanReadableText) -> Result<usize, Self::Error> {
//!         Ok(text.as_str().len())
//!     }
//! }
//!
//! let failures = assert_that!(1)
//!     .with_location(false)
//!     .capture(|it| it.is_equal_to(2));
//! let chain = ToHumanReadableText.then(TextLength);
//! let length = chain.adapt(&failures[0]).unwrap();
//! assert!(length > 0);
//! ```

mod adapters;

#[cfg(feature = "std")]
pub use adapters::StdOutLogger;
pub use adapters::{HumanReadableText, MapErr, Then, ThenError, ToHumanReadableText};

/// Transforms a borrowed input into an owned output.
///
/// An adapter can change representation or perform a side effect. Side-effect-only adapters use
/// `()` as their output. The input is generic so the output of one adapter can be the input of the
/// next one.
///
/// This trait supports dynamic dispatch when both associated types are specified, for example
/// `dyn Adapter<str, Output = usize, Error = String>`. Use [`AdapterExt::map_err`] when adapters
/// with different error types need to share the same trait-object type.
pub trait Adapter<Input: ?Sized> {
    /// The owned value produced by this adapter.
    type Output;

    /// The error produced by this adapter.
    type Error;

    /// Adapts one borrowed input.
    ///
    /// # Errors
    ///
    /// Returns the adapter's typed error when it cannot produce its output.
    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error>;
}

impl<Input: ?Sized, A: Adapter<Input> + ?Sized> Adapter<Input> for &A {
    type Output = A::Output;
    type Error = A::Error;

    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error> {
        (**self).adapt(input)
    }
}

/// Fluent composition methods for adapters.
///
/// This is separate from [`Adapter`] because that trait's generic input cannot always be inferred
/// at the point where a chain is assembled. The resulting composition implements [`Adapter`]
/// only when its adjacent stages have compatible types.
pub trait AdapterExt: Sized {
    /// Passes this adapter's successful output to `next`.
    fn then<Next>(self, next: Next) -> Then<Self, Next> {
        Then::new(self, next)
    }

    /// Maps this adapter's errors while preserving its successful output.
    ///
    /// The mapper runs only when [`Adapter::adapt`] returns an error. It can produce any error
    /// type and may borrow local data. Use `(&adapter).map_err(...)` to keep the original adapter.
    ///
    /// ```
    /// use core::num::ParseIntError;
    /// use assertr::failure::adapter::{Adapter, AdapterExt};
    ///
    /// struct ParseNumber;
    ///
    /// impl Adapter<str> for ParseNumber {
    ///     type Output = usize;
    ///     type Error = ParseIntError;
    ///
    ///     fn adapt(&self, input: &str) -> Result<usize, ParseIntError> {
    ///         input.parse()
    ///     }
    /// }
    ///
    /// let adapter = ParseNumber.map_err(|error| error.to_string());
    /// let adapter: &dyn Adapter<str, Output = usize, Error = String> = &adapter;
    /// assert_eq!(adapter.adapt("42"), Ok(42));
    /// assert!(adapter.adapt("not a number").is_err());
    /// ```
    fn map_err<Input: ?Sized, F, Error>(self, mapper: F) -> MapErr<Self, F>
    where
        Self: Adapter<Input>,
        F: Fn(Self::Error) -> Error,
    {
        MapErr::new(self, mapper)
    }
}

impl<T> AdapterExt for T {}
