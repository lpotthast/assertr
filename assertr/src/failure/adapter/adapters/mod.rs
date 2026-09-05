//! Concrete adapter implementations.
//!
//! Each child module conceptually defines one adapter. A module may also define the output,
//! error, and private support types needed to make that adapter work.

mod human_readable;
#[cfg(feature = "std")]
mod logging;
mod map_err;
mod then;

pub use human_readable::{HumanReadableText, ToHumanReadableText};
#[cfg(feature = "std")]
pub use logging::StdOutLogger;
pub use map_err::MapErr;
pub use then::{Then, ThenError};
