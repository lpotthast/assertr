//! Every assertion trait, plus the capability traits that make them available for your types.
//!
//! Import [`crate::prelude`] and let autocomplete show what fits the current subject. This module
//! is the reference: each trait page lists its methods with their exact bounds. Rustdoc search also
//! finds a method by name.
//!
//! | Subject | Assertion traits |
//! |---|---|
//! | Any value | [`PartialEqAssertions`], [`PartialOrdAssertions`], [`DebugAssertions`], [`DisplayAssertions`], [`IdentityAssertions`], [`PatternAssertions`] |
//! | `bool`, `char`, strings | [`BoolAssertions`], [`CharAssertions`], [`StrAssertions`] |
//! | `Option`, `Result`, `Poll` | [`OptionAssertions`], [`ResultAssertions`], [`PollAssertions`] and their `*ExtractAssertions` |
//! | Ranges, `RefCell`, lengths, types | [`RangeAssertions`], [`RangeBoundAssertions`], [`RefCellAssertions`], [`LengthAssertions`], [`MemAssertions`] |
//! | Collections | [`CollectionAssertions`], [`StableOrderAssertions`], [`StableOrderExtractAssertions`], [`RandomAccessExtractAssertions`] |
//! | Iterators | [`IteratorAssertions`], [`IntoIteratorAssertions`], [`ExactSizeIteratorAssertions`] |
//! | Sets and maps | [`SetAssertions`], [`MapAssertions`] |
//! | Boxed `Any` and panic payloads | [`BoxAssertions`], [`BoxExtractAssertions`] |
//! | Closures (`std`) | `FnOnceAssertions`, `AsyncFnOnceAssertions` |
//! | Numbers (`num`) | `NumAssertions` |
//! | `Path`, `Command`, `Mutex` (`std`) | `PathAssertions`, `CommandAssertions`, `MutexAssertions` |
//! | Observations, closures returning a future of a changing value (`std`) | `EventualAssertions`, with builders and retry policies in `eventually` |
//! | Browser elements (`thirtyfour`) | `ThirtyfourWebElementAssertions`, plus shared reads in `thirtyfour::read`; optional `thirtyfour-cdp` descriptions |
//! | Integrations | `HttpHeaderValueAssertions` (`http`), `SignedDurationAssertions`, `SpanAssertions`, `ZonedAssertions` (`jiff`), `ProgramAssertions` (`program`), `ReqwestResponseAssertions` (`reqwest`), `RootcauseReportAssertions` (`rootcause`), `TokioMutexAssertions`, `TokioRwLockAssertions`, `TokioWatchReceiverAssertions` (`tokio`) |
//!
//! Feature-gated traits appear in the list below when their feature is enabled.
//!
//! The methods run [`Expectation`](crate::expectation::Expectation) values. To use one of these
//! checks with `.matches(..)`, on collection elements, or inside `partial!`, find it in the
//! [`matchers`](mod@crate::matchers) catalog.
//!
//! # Using the families with your own types
//!
//! The families work through capability traits. Implement the one that fits your type and its
//! assertions become available:
//!
//! | Implement | To get |
//! |---|---|
//! | [`HasLength`] | Length assertions such as `has_length` and `is_empty` |
//! | [`Collection`] | Element assertions that ignore order, such as `contains` |
//! | [`StableOrder`] | Assertions on order and positions, such as `contains_exactly` |
//! | [`RandomAccess`] | Extraction by position with `get_at` |
//! | [`SetLookup`] | Set relations such as `is_subset_of` |
//! | [`Map`] and [`MapLookup`] | Map assertions such as `contains_key` |
//! | [`NumericDistance`] | Tolerance checks with `is_close_to` |
//!
//! # Assertion traits
//!
//! The `*Assertions` traits are public so their methods are in scope, not for you to implement.
//! Adding a method to one of them is not a breaking change. For your own types, define a separate
//! trait (see the [custom assertions guide](crate#custom-assertions)).
//!
//! Every assertion trait is implemented for every renderer. Each method requires only the
//! [`ValueRenderer`](crate::renderer::ValueRenderer) implementations its own failure report needs.
//! A renderer that cannot show one type therefore does not hide a whole family. See
//! [`ValueRenderer`](crate::renderer::ValueRenderer#capability-bounds-belong-to-methods) for the
//! reasoning.

#[macro_use]
mod support;

pub(crate) mod alloc;
pub(crate) mod collection;
pub(crate) mod core;
pub(crate) mod distance;
mod has_length;
#[cfg(feature = "http")]
pub(crate) mod http;
pub(crate) mod iterator;
#[cfg(feature = "jiff")]
pub(crate) mod jiff;
pub(crate) mod map;
#[cfg(feature = "num")]
pub(crate) mod num;
#[cfg(feature = "program")]
pub(crate) mod program;
#[cfg(feature = "reqwest")]
pub(crate) mod reqwest;
#[cfg(feature = "rootcause")]
pub(crate) mod rootcause;
pub(crate) mod set;
#[cfg(feature = "std")]
pub(crate) mod std;
#[cfg(feature = "std")]
pub use self::std::eventually;
#[cfg(feature = "thirtyfour")]
pub mod thirtyfour;
#[cfg(feature = "thirtyfour")]
pub use thirtyfour::ThirtyfourWebElementAssertions;
#[cfg(feature = "tokio")]
pub(crate) mod tokio;

// Capabilities that make the assertion families available for a type.
// Boxed `Any` values and panic payloads.
pub use alloc::boxed::{BoxAssertions, BoxExtractAssertions};

pub use collection::{
    Collection, CollectionAssertions, RandomAccess, RandomAccessExtractAssertions, StableOrder,
    StableOrderAssertions, StableOrderExtractAssertions,
};
pub use distance::NumericDistance;
pub use has_length::HasLength;
#[cfg(feature = "http")]
pub use http::header_value::{HttpHeaderValueAssertions, HttpHeaderValueExtractAssertions};
#[cfg(feature = "jiff")]
pub use jiff::{
    signed_duration::SignedDurationAssertions, span::SpanAssertions, zoned::ZonedAssertions,
};
pub use map::{Map, MapAssertions, MapLookup};
#[cfg(feature = "num")]
pub use num::NumAssertions;
#[cfg(feature = "program")]
pub use program::{Program, ProgramAssertions, ProgramExtractAssertions};
#[cfg(feature = "reqwest")]
pub use reqwest::response::{ReqwestResponseAssertions, ReqwestResponseExtractAssertions};
#[cfg(feature = "rootcause")]
pub use rootcause::report::{
    RootcauseDynamicReportAssertions, RootcauseDynamicReportExtractAssertions,
    RootcauseReportAssertions,
};
pub use set::{SetAssertions, SetLookup};
#[cfg(feature = "tokio")]
pub use tokio::{
    mutex::TokioMutexAssertions, rw_lock::TokioRwLockAssertions,
    watch::TokioWatchReceiverAssertions,
};

// Values, references, and core types.
pub use self::core::bool::BoolAssertions;
#[cfg(any(feature = "std", test))]
pub use self::core::r#fn::{AsyncFnOnceAssertions, FnOnceAssertions};
// Iterators, collections, sets, and maps.
pub use self::core::iter::{
    ExactSizeIteratorAssertions, IntoIteratorAssertions, IteratorAssertions,
};
pub use self::core::{
    char::CharAssertions,
    debug::DebugAssertions,
    display::DisplayAssertions,
    identity::IdentityAssertions,
    length::LengthAssertions,
    mem::MemAssertions,
    option::{OptionAssertions, OptionExtractAssertions},
    partial_eq::PartialEqAssertions,
    partial_ord::PartialOrdAssertions,
    pattern::PatternAssertions,
    poll::{PollAssertions, PollExtractAssertions},
    range::{RangeAssertions, RangeBoundAssertions},
    ref_cell::RefCellAssertions,
    result::{ResultAssertions, ResultExtractAssertions},
    string::StrAssertions,
};
// Feature integrations.
#[cfg(feature = "std")]
pub use self::std::{
    command::CommandAssertions,
    eventually::{EventualAssertions, Patience},
    mutex::MutexAssertions,
    path::PathAssertions,
};
