//! Concrete adapter implementations.
//!
//! Each child module conceptually defines one adapter. A module may also define the output, error,
//! and private support types needed to make that adapter work.

mod human_readable;
mod map_err;
mod then;
#[cfg(feature = "std")]
mod writer;

pub use human_readable::{HumanReadableText, ToHumanReadableText};
pub use map_err::MapErr;
pub use then::{Then, ThenError};
#[cfg(feature = "std")]
pub use writer::Writer;
