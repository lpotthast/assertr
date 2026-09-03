//! Rendering values and diagnostic structures in assertion failure messages.
//!
//! Custom assertion implementations use [`AssertThat::render`](crate::AssertThat::render) to
//! apply the assertion chain's [`ValueRenderer`] and [`RenderingBudget`] to their own diagnostics.

mod budget;
mod context;
mod presentation;
mod type_info;
mod value;

pub use budget::{RenderingBudget, RenderingBudgetBuilder};
pub use context::{RenderedValue, RenderedValues, RenderingContext};
pub use presentation::{CollectionPresentation, GroupStyle, RenderingOrder};
pub use type_info::{TypeHint, Typed};
pub use value::{CustomRenderer, DebugRenderer, ValueRenderer};

pub(crate) use context::omission;
