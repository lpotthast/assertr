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
//! assert_that!(report).contains("user #7").contains("user #9");
//! ```
//!
//! The closure renders the subject's type only. If assertions also display collection elements,
//! map keys, or projections of other types, implement [`ValueRenderer<T>`](ValueRenderer) for each
//! displayed type on one renderer and install it with
//! [`AssertThat::with_renderer`](crate::AssertThat::with_renderer). That method includes a reusable
//! renderer example. Derived borrowed assertions require `Clone`. Owned mappings, Result
//! extraction, and serialization conversions preserve the renderer without cloning it.
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
//! Lengths, counts, user-supplied expected indices, and errors are evidence too. Methods displaying
//! numeric evidence require `ValueRenderer<usize>`. Errors retain their original type, including
//! condition errors. The default renderer requires `Debug`, while a custom renderer may support
//! errors implementing neither `Debug` nor `Display`. There is no fallback renderer for evidence.
//!
//! Structural positions and diagnostic paths, matcher branch and expected-slot identifiers,
//! omission summaries, type names, status-class labels such as `2xx`, and explicit diagnostic prose
//! are formatted by Assertr. [`IntoRendered`] conversions from strings, formatting arguments, and
//! primitives are verbatim and bypass the renderer and budget. Use them only for structural text
//! or caller-authored prose. Pass evidence through rendering adapters, including notes.
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
//! the chain's renderer and budget. Expectation definitions obtain the same [`RenderingContext`]
//! through [`AssertionContext::render`](crate::AssertionContext::render). Pass adapters to the
//! failure builder or [`Fact`](crate::Fact) constructors:
//!
//! | Evidence | Adapter |
//! | --- | --- |
//! | One leaf | [`value`](RenderingContext::value) |
//! | Collection with its presentation and type | [`collection`](RenderingContext::collection), [`borrowed_collection`](RenderingContext::borrowed_collection) |
//! | Collection with meaningful positions | [`stable_collection`](RenderingContext::stable_collection), [`stable_borrowed_collection`](RenderingContext::stable_borrowed_collection) |
//! | Map with its ordering policy and type | [`map`](RenderingContext::map) |
//! | Synthetic list or set | [`values`](RenderingContext::values), [`borrowed_values`](RenderingContext::borrowed_values) |
//! | Synthetic key/value tuples | [`entry_list`](RenderingContext::entry_list) |
//! | One-field tuple variant or named struct | [`variant`](RenderingContext::variant), [`struct_field`](RenderingContext::struct_field) |
//! | Inaccessible struct field | [`unavailable_struct_field`](RenderingContext::unavailable_struct_field) |
//!
//! Synthetic groups have no outer Rust type. Select their diagnostic order with [`RenderingOrder`],
//! using [`RenderedValues::with_order`] or the `entry_list` argument. Collection adapters follow
//! [`CollectionPresentation`]. Positional adapters require
//! [`StableOrder`](crate::assertions::collection::StableOrder) and always preserve iteration order.
//!
//! Construction is lazy and needs no renderer capability. Formatting or converting with
//! [`IntoRendered`] requires only the displayed leaf renderers, traverses the borrowed source,
//! and applies the budget. Sorted groups order rendered text before retaining the requested
//! number of items. The budget limits retained output, not traversal work or peak memory.
//! Reuse the resulting [`Rendered`] tree to avoid rendering again.
//!
//! [`RenderingContext::budget`] returns a copy of the active limits for custom evidence collectors.
//! Keep assertion truth independent of retention and record omitted evidence in the failure
//! builder. The [custom assertions guide](crate#structural-evidence) demonstrates collection,
//! map, and wrapper diagnostics without constructing metadata or rendering syntax by hand.

mod budget;
mod context;
mod presentation;
mod rendered;
mod type_info;
mod value;

pub use budget::RenderingBudget;
pub use context::{
    EntryList, MapEntries, RenderedValue, RenderedValues, RenderingContext, StructField,
    UnavailableStructField, Variant,
};
pub use presentation::{CollectionPresentation, GroupStyle, RenderingOrder};
pub use rendered::{IntoRendered, Rendered, RenderedBody};
pub use type_info::{TypeHint, Typed};
pub use value::{CustomRenderer, DebugRenderer, SensitiveValuePolicy, ValueRenderer};

pub(crate) use context::{Compact, omission};
