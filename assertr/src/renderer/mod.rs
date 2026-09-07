//! Customize how values appear in assertion failures.
//!
//! Assertions use [`DebugRenderer`] by default. To display a type that has no `Debug`
//! implementation, or to choose a different representation for it, install a formatter with
//! [`AssertThat::with_debug_format`](crate::AssertThat::with_debug_format):
//!
//! ```
//! use assertr::prelude::*;
//!
//! #[derive(PartialEq)]
//! struct UserId(u32);
//!
//! let failures = assert_that!(UserId(7))
//!     .with_debug_format(|id, f| write!(f, "user #{}", id.0))
//!     .capture(|it| it.is_equal_to(UserId(9)));
//!
//! let report = ToHumanReadableText.render(&failures[0]);
//! assert!(report.contains("user #7"));
//! assert!(report.contains("user #9"));
//! ```
//!
//! The closure renders the subject's type only. If assertions also display collection elements,
//! map keys, or projections of other types, implement [`ValueRenderer<T>`](ValueRenderer) for each
//! displayed type on one renderer and install it with
//! [`AssertThat::with_renderer`](crate::AssertThat::with_renderer). That method includes a reusable
//! renderer example. Projections preserve the renderer and require it to implement `Clone`.
//!
//! ## Values and structure
//!
//! Assertr owns collection, map, and wrapper syntax. Structural assertions ask the renderer for
//! the leaf values they display. Generic assertions such as `is_equal_to` and `has_length` treat
//! the subject as a whole and need a renderer for that whole type. Each assertion's method bounds
//! state which implementations it needs. See [`ValueRenderer`] for details and pretty-printing.
//!
//! The resulting [`Rendered`] tree retains leaf text, structure, type information, and omission
//! counts in the [`AssertionFailure`](crate::AssertionFailure). A failure
//! [adapter](crate::failure::adapter) reads this tree to produce a complete report. Use
//! [`with_panic_presentation`](crate::AssertThat::with_panic_presentation) to change panic report
//! layout. Value renderers apply before either capture or panic handling.
//!
//! ## Sensitive values
//!
//! [`ValueRenderer::sensitive_value_policy`] controls how sensitivity-aware assertions prepare
//! values for the renderer's `fmt` method. Currently, reqwest response `has_header_value` and
//! `does_not_have_header` consult this policy for header values.
//!
//! Custom renderers default to [`SensitiveValuePolicy::Preserve`], receiving the original header
//! and sensitivity flag so their formatter controls redaction. [`DebugRenderer`] chooses
//! [`SensitiveValuePolicy::Reveal`], displaying header contents for test diagnostics by rendering
//! an unmarked copy. Custom renderers can opt into the same policy. The response's header stays
//! unchanged, and rendering budgets still apply. Generic value rendering, including direct
//! equality on a header, passes the original value to `fmt` without consulting this policy.
//!
//! ## Limit diagnostic output
//!
//! [`RenderingBudget`] bounds retained items and leaf characters without changing whether an
//! assertion passes. Configure it through
//! [`AssertThat::with_rendering_budget`](crate::AssertThat::with_rendering_budget), which includes
//! an example. Derived assertions inherit the budget. Adapters receive the bounded tree, so they
//! cannot recover omitted values. Use [`RenderingBudget::unlimited`] when complete diagnostics
//! are needed.
//!
//! ## Render values in custom assertions
//!
//! Custom assertion implementations use [`AssertThat::render`](crate::AssertThat::render) to apply
//! the chain's renderer and budget. Its [`RenderingContext`] supplies adapters for single values,
//! collections, and maps. Pass those adapters to the failure builder instead of formatting values
//! directly. The [custom assertions guide](crate#custom-assertions) shows a complete
//! implementation.

mod budget;
mod context;
mod presentation;
mod rendered;
mod type_info;
mod value;

pub use budget::{RenderingBudget, RenderingBudgetBuilder};
pub use context::{RenderedValue, RenderedValues, RenderingContext};
pub use presentation::{CollectionPresentation, GroupStyle, RenderingOrder};
pub use rendered::{IntoRendered, Rendered, RenderedBody};
pub use type_info::{TypeHint, Typed};
pub use value::{CustomRenderer, DebugRenderer, SensitiveValuePolicy, ValueRenderer};

pub(crate) use context::Compact;
pub(crate) use context::omission;
