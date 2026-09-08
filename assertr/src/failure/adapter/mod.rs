//! Process structured failures and customize reports with typed adapters.
//!
//! Start with [`AssertThat::capture`](crate::AssertThat::capture) to collect failures. Read their
//! [`AssertionFailure`](crate::AssertionFailure) fields directly when you need structured evidence,
//! or use [`ToHumanReadableText::render`] for the same report that panic mode uses by default:
//!
//! ```
//! use assertr::prelude::*;
//! use assertr::failure::adapter::HumanReadableText;
//!
//! let failures = assert_that!(42).capture(|it| it.is_less_than(0).is_equal_to(43));
//! let reports: Vec<_> = failures.iter().map(|failure| ToHumanReadableText.render(failure)).collect();
//! assert_that!(reports).contains_exactly_satisfying([
//!     |report: AssertThat<HumanReadableText, Capture>| {
//!         report.contains("is not less than");
//!     },
//!     |report: AssertThat<HumanReadableText, Capture>| {
//!         report.contains("Expected: 43");
//!     },
//! ]);
//! ```
//!
//! ## Chain transformations
//!
//! [`Adapter`] transforms a borrowed input into an owned output. Import [`AdapterExt`] to chain
//! compatible stages with [`then`](AdapterExt::then). The output need not be text. This example
//! measures a rendered report, but an adapter can also consume an `AssertionFailure` directly to
//! build a machine-readable representation from its fields and retained
//! [`Rendered`](crate::renderer::Rendered) value trees.
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
//!
//! Each stage keeps its output and error types. [`ThenError`] identifies which stage failed, and
//! [`AdapterExt::map_err`] changes an error type without changing successful output. Adapters run
//! on the calling thread and may perform side effects. Use `()` as the output for a stage that
//! only logs or records its input. Adapters used explicitly may borrow local data.
//!
//! ## Select panic presentation
//!
//! Pass an adapter producing [`HumanReadableText`] to
//! [`AssertThat::with_panic_presentation`](crate::AssertThat::with_panic_presentation). Its example
//! adds context to the default report. The context owns the adapter, so it must be `'static`.
//! Move or clone any local data into it, or share owned data through `Rc`. Derived assertions share
//! the adapter without requiring `Clone`. Displayable adapter errors become strings internally.
//! Panic presentation also requires [`RefUnwindSafe`](core::panic::RefUnwindSafe), preserving the
//! adapter's unwind-safety guarantee after its type is erased. Explicit adapter calls have no such
//! requirement.
//!
//! Capture mode stores structured failures without running this presentation. Apply adapters
//! explicitly to captured failures as shown above. To change individual diagnostic values before
//! either mode handles a failure, configure a [value renderer](crate::renderer). The
//! [failure model](crate::failure) explains how construction, handling, and presentation fit
//! together.

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
/// This trait supports dynamic dispatch when both associated types are specified, for example `dyn
/// Adapter<str, Output = usize, Error = String>`. Use [`AdapterExt::map_err`] when adapters with
/// different error types need to share the same trait-object type.
///
/// See the [adapter guide](crate::failure::adapter) for capture, composition, and panic
/// presentation examples.
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
/// at the point where a chain is assembled. The resulting composition implements [`Adapter`] only
/// when its adjacent stages have compatible types.
pub trait AdapterExt: Sized {
    /// Passes this adapter's successful output to `next`.
    fn then<Next>(self, next: Next) -> Then<Self, Next> {
        Then::new(self, next)
    }

    /// Maps this adapter's errors while preserving its successful output.
    ///
    /// The mapper runs only when [`Adapter::adapt`] returns an error. It can produce any error type
    /// and may borrow local data. Use `(&adapter).map_err(...)` to keep the original adapter.
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
    fn map_err<Input: ?Sized, F, Error>(self, mapper: F) -> MapErr<Self, F, Input>
    where
        Self: Adapter<Input>,
        F: Fn(Self::Error) -> Error,
    {
        MapErr::new(self, mapper)
    }
}

impl<T> AdapterExt for T {}
