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
#[cfg(feature = "tokio")]
pub(crate) mod tokio;

// Capabilities that make the assertion families available for a type.
pub use collection::{Collection, RandomAccess, StableOrder};
pub use distance::NumericDistance;
pub use has_length::HasLength;
pub use map::{Map, MapLookup};
pub use set::SetLookup;

// Values, references, and core types.
pub use self::core::bool::BoolAssertions;
pub use self::core::char::CharAssertions;
pub use self::core::debug::DebugAssertions;
pub use self::core::display::DisplayAssertions;
#[cfg(feature = "std")]
pub use self::core::r#fn::{AsyncFnOnceAssertions, FnOnceAssertions};
pub use self::core::identity::IdentityAssertions;
pub use self::core::length::LengthAssertions;
pub use self::core::mem::MemAssertions;
pub use self::core::option::{OptionAssertions, OptionExtractAssertions};
pub use self::core::partial_eq::PartialEqAssertions;
pub use self::core::partial_ord::PartialOrdAssertions;
pub use self::core::pattern::PatternAssertions;
pub use self::core::poll::{PollAssertions, PollExtractAssertions};
pub use self::core::range::{RangeAssertions, RangeBoundAssertions};
pub use self::core::ref_cell::RefCellAssertions;
pub use self::core::result::{ResultAssertions, ResultExtractAssertions};
pub use self::core::string::StrAssertions;

// Iterators, collections, sets, and maps.
pub use self::core::iter::{
    ExactSizeIteratorAssertions, IntoIteratorAssertions, IteratorAssertions,
};
pub use collection::{
    CollectionAssertions, RandomAccessExtractAssertions, StableOrderAssertions,
    StableOrderExtractAssertions,
};
pub use map::MapAssertions;
pub use set::SetAssertions;

// Boxed `Any` values and panic payloads.
pub use alloc::boxed::{BoxAssertions, BoxExtractAssertions};

// Feature integrations.
#[cfg(feature = "std")]
pub use self::std::{command::CommandAssertions, mutex::MutexAssertions, path::PathAssertions};
#[cfg(feature = "http")]
pub use http::header_value::{HttpHeaderValueAssertions, HttpHeaderValueExtractAssertions};
#[cfg(feature = "jiff")]
pub use jiff::{
    signed_duration::SignedDurationAssertions, span::SpanAssertions, zoned::ZonedAssertions,
};
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
#[cfg(feature = "tokio")]
pub use tokio::{
    mutex::TokioMutexAssertions, rw_lock::TokioRwLockAssertions,
    watch::TokioWatchReceiverAssertions,
};
