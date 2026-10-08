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
//! let report = failures[0].to_string();
//! assert_that!(report).contains("user #7").contains("user #9");
//! ```
//!
//! The closure renders the subject's type only. If assertions also display collection elements,
//! map keys, or projections of other types, implement [`ValueRenderer<T>`](ValueRenderer) for each
//! displayed type on one renderer and install it with
//! [`AssertThat::with_renderer`](crate::AssertThat::with_renderer). That method includes a reusable
//! renderer example. Every `derive*` and `satisfies*` projection, and every assertion composed from
//! them such as `is_some_satisfying`, requires `R: Clone`, because each child chain receives its
//! own renderer. Installing a reference, as in `with_renderer(&renderer)`, satisfies this for any
//! renderer. Consuming mappings (`map` and `map_owned`), Result extraction, and serialization
//! conversions preserve the renderer without cloning it.
//!
//! ## Values and structure
//!
//! Assertr owns collection, map, and wrapper syntax. Structural assertions ask the renderer for
//! the leaf values they display. Generic assertions such as `is_equal_to` and `has_length` treat
//! the subject as a whole and need a renderer for that whole type. Each assertion's method bounds
//! state which implementations it needs. See [`ValueRenderer`] for details and pretty-printing.
//!
//! The resulting [`Rendered`] tree retains leaf text, structure, type information, and omission
//! counts in the [`AssertionFailure`](crate::AssertionFailure). The failure's `Display`
//! implementation prints these trees as the default report, and custom code can inspect them
//! directly. Use [`with_panic_presentation`](crate::AssertThat::with_panic_presentation) to change
//! panic report layout. Value renderers apply before either capture or panic handling.
//!
//! Lengths, counts, user-supplied expected indices, and errors are evidence too. Methods displaying
//! numeric evidence require `ValueRenderer<usize>`. Errors retain their original type, including
//! expectation rejections. The default renderer requires `Debug`, while a custom renderer may
//! support errors implementing neither `Debug` nor `Display`. There is no fallback renderer for
//! evidence.
//!
//! ## Minimal renderer capabilities
//!
//! A callback that only checks an `Option` variant needs no payload renderer. Positive collection
//! membership adds no leaf requirement of its own:
//!
//! ```
//! use assertr::prelude::*;
//!
//! struct Secret;
//! #[derive(Clone)]
//! struct NoRenderer;
//!
//! assert_that!([Some(Secret)])
//!     .with_renderer(NoRenderer)
//!     .contains_satisfying(|it| { it.is_some(); });
//! ```
//!
//! An exact callback comparison also reports counts. Implement just `ValueRenderer<usize>` to
//! retain that evidence, even when neither the collection nor its items can be rendered:
//!
//! ```
//! use assertr::prelude::*;
//!
//! struct Secret;
//! #[derive(Clone)]
//! struct Counts;
//! impl ValueRenderer<usize> for Counts {
//!     fn fmt(&self, value: &usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//!         write!(f, "count({value})")
//!     }
//! }
//! fn is_some(it: AssertThat<'_, Option<Secret>, Capture, Counts>) {
//!     it.is_some();
//! }
//!
//! let failures = assert_that!([Some(Secret), Some(Secret)])
//!     .with_renderer(Counts)
//!     .capture(|it| it.contains_exactly_satisfying([is_some]));
//! assert_that!(failures).has_length(1);
//! assert_that!(failures[0].to_string()).contains("count(2)");
//! ```
//!
//! `Clone` carries the renderer into callback child chains. Passing a check does not remove its
//! method's diagnostic bounds. A direct equality check on `Secret` would still require a
//! renderer for `Secret` as well as its comparison capability.
//!
//! ## Structural metadata
//!
//! Structural positions and diagnostic paths, matcher branch and expected-slot identifiers,
//! omission summaries, type names, status-class labels such as `2xx`, and explicit diagnostic prose
//! are formatted by Assertr. [`Rendered`] conversions from strings, formatting arguments, and
//! primitives are verbatim and bypass the renderer and budget. Use them only for structural text
//! or caller-authored prose. Pass evidence through a [`RenderingContext`], including notes.
//!
//! ## Limit diagnostic output
//!
//! [`RenderingBudget`] bounds retained items and leaf characters without changing whether an
//! assertion passes. Configure it through
//! [`AssertThat::with_rendering_budget`](crate::AssertThat::with_rendering_budget), which includes
//! an example. Derived assertions inherit the budget. Captured failures and panic presentations
//! receive the bounded tree, so they cannot recover omitted values. Use
//! [`RenderingBudget::unlimited`] when complete diagnostics are needed.
//!
//! ## Render values in custom assertions
//!
//! Custom assertion implementations use [`AssertThat::render`](crate::AssertThat::render) to apply
//! the chain's renderer and budget. Expectation definitions obtain the same [`RenderingContext`]
//! through [`AssertionContext::render`](crate::AssertionContext::render). Pass the returned
//! [`Rendered`] trees to the failure builder or [`Fact`](crate::Fact) constructors:
//!
//! | Evidence | Method |
//! | --- | --- |
//! | One leaf | [`value`](RenderingContext::value) |
//! | Collection with its presentation and type | [`collection`](RenderingContext::collection) |
//! | Collection with meaningful positions | [`stable_collection`](RenderingContext::stable_collection) |
//! | Map with its ordering policy and type | [`map`](RenderingContext::map) |
//! | Synthetic list | [`borrowed_values`](RenderingContext::borrowed_values) |
//! | Synthetic key/value tuples | [`entry_list`](RenderingContext::entry_list) |
//! | One-field tuple variant or named struct | [`variant`](RenderingContext::variant), [`struct_field`](RenderingContext::struct_field) |
//! | Inaccessible struct field | [`unavailable_struct_field`](RenderingContext::unavailable_struct_field) |
//!
//! Synthetic groups have no outer Rust type. Select their diagnostic order with an explicit
//! [`RenderingOrder`] argument. Collection methods follow [`CollectionPresentation`]. Positional
//! rendering requires [`StableOrder`](crate::assertions::collection::StableOrder) and always
//! preserves iteration order. Use [`RenderingContext::compact`] to render leaves in their compact
//! form.
//!
//! Each method requires only the displayed leaf renderers, renders every leaf once, and applies
//! the budget. Sorted groups order rendered text before retaining the requested number of items.
//! The budget limits retained output, not traversal work or peak memory.
//!
//! [`RenderingContext::budget`] returns a copy of the active limits for custom evidence collectors.
//! Keep assertion truth independent of retention and record omitted evidence in the failure
//! builder. The [custom assertions guide](crate#structural-evidence) demonstrates collection,
//! map, and wrapper diagnostics without constructing metadata or rendering syntax by hand.

mod budget;
mod context;
mod presentation;
mod rendered;
mod value;

pub use budget::RenderingBudget;
pub use context::RenderingContext;
pub use presentation::{CollectionPresentation, GroupStyle, RenderingOrder};
pub use rendered::{Rendered, RenderedBody};
pub use value::{CustomRenderer, DebugRenderer, ValueRenderer};

pub(crate) use context::omission;
