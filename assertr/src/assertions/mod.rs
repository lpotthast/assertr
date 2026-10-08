//! The assertion traits, grouped by the kind of value they apply to.
//!
//! # Finding assertions
//!
//! Import [`crate::prelude`] and let autocomplete show what is available for the current subject.
//! To browse, start with the family that fits the subject:
//!
//! - [General values, `Option`, `Result`, ranges, strings, and iterators](core)
//! - [Collections such as slices, arrays, and `Vec`](collection)
//! - [Sets](set)
//! - [Maps](map)
//! - [Boxed `Any` values and panic payloads](alloc)
//!
//! Modules for `num`, `std`, `http`, `jiff`, `program`, `reqwest`, `rootcause`, and `tokio` appear
//! below when their feature is enabled. Each trait page lists its methods with their exact bounds.
//! Rustdoc search also finds a method by name.
//!
//! The assertion methods are built from [`Expectation`](crate::Expectation) values. To use one of
//! these checks with `.matches(..)`, on collection elements, or inside `partial!`, find it in the
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
//! | [`Collection`](collection::Collection) | Element assertions that ignore order, such as `contains` |
//! | [`StableOrder`](collection::StableOrder) | Assertions on order and positions, such as `contains_exactly` |
//! | [`SetLookup`](set::SetLookup) | Set relations such as `is_subset_of` |
//! | [`Map`](map::Map) and [`MapLookup`](map::MapLookup) | Map assertions such as `contains_key` |
//!
//! # Assertion traits
//!
//! The `*Assertions` traits are public so their methods are in scope, not for you to implement.
//! Adding a method to one of them is not a breaking change. For your own types, define a separate
//! trait (see the [custom assertions guide](crate#custom-assertions)).
//!
//! Every assertion trait is implemented for every renderer. Each method requires only the
//! [`ValueRenderer`](crate::ValueRenderer) implementations its own failure report needs. A renderer
//! that cannot show one type therefore does not hide a whole family. See
//! [`ValueRenderer`](crate::ValueRenderer#capability-bounds-belong-to-methods) for the reasoning.

#[macro_use]
mod support;

pub mod alloc;
pub mod collection;
pub mod core;
pub mod distance;
mod has_length;
#[cfg(feature = "http")]
pub mod http;
pub(crate) mod iterator;
#[cfg(feature = "jiff")]
pub mod jiff;
pub mod map;
pub mod matcher;
#[cfg(feature = "num")]
pub mod num;
#[cfg(feature = "program")]
pub mod program;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "rootcause")]
pub mod rootcause;
pub mod set;
#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

pub use has_length::HasLength;
