//! The assertion traits, organized by the value family they apply to.
//!
//! # Finding assertions
//!
//! Import [`crate::prelude`] and use IDE autocomplete to see the assertions available for the
//! current subject. For browsing, start with the family that owns the subject:
//!
//! - [General-purpose values, wrappers, ranges, and iterators](core)
//! - [Reusable conditions](condition)
//! - [Collections and sequences](collection)
//! - [Sets](set)
//! - [Maps](map)
//! - [Heap-backed values and panic payloads](alloc)
//!
//! The optional `num`, `std`, `http`, `jiff`, `program`, `reqwest`, `rootcause`, and `tokio`
//! integration modules appear in the module list below when their corresponding Cargo feature is
//! enabled. Each assertion trait page is the authoritative list of its methods, signatures, and
//! required bounds. Rustdoc search can also find a method directly by name.
//!
//! # Assertion traits
//!
//! The `*Assertions` traits are public so their methods participate in Rust's method resolution,
//! not as downstream implementation interfaces. Adding a method to one of them is a compatible
//! change. For a custom type, define a separate assertion trait instead (see
//! [`AssertThat::track_assertion`](crate::AssertThat::track_assertion)).
//!
//! # Renderer capabilities
//!
//! An assertion trait is implemented independently of the active renderer's capabilities. Each
//! method requires only the [`ValueRenderer`](crate::ValueRenderer) implementations its
//! own failure path uses. Consequently, a renderer that cannot format one value does not hide an
//! entire assertion family, and projections preserve the active renderer until a later method
//! needs a specific rendering capability. See
//! [`ValueRenderer`](crate::ValueRenderer#capability-bounds-belong-to-methods) for the
//! design rationale.
//!
//! To extend an existing family to a custom type, implement [`HasLength`] for length assertions,
//! [`Collection`](collection::Collection) for the element assertions,
//! [`Sequence`](collection::Sequence) for the order-sensitive ones on top of those,
//! [`Set`](set::Set) for the set relations, or [`Map`](map::Map) with
//! [`MapLookup`](map::MapLookup) for the map assertions.

mod has_length;

pub use has_length::HasLength;

pub mod alloc;
pub mod collection;
/// Assertions based on reusable [`crate::condition::AssertrCondition`] values.
pub mod condition;
pub mod core;
#[cfg(feature = "http")]
pub mod http;
pub(crate) mod iterator;
#[cfg(feature = "jiff")]
pub mod jiff;
pub mod map;
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
