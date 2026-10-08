//! Control how values appear in failure reports.
//!
//! By default, values are shown through their `Debug` implementation by [`DebugRenderer`]. To show
//! a type that has no `Debug` implementation, or to show it differently, give the chain a
//! formatting closure with [`AssertThat::with_debug_format`](crate::AssertThat::with_debug_format):
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
//! The closure handles the subject's type only. When a report also shows other types, such as
//! collection elements or map keys, write a renderer type. Implement
//! [`ValueRenderer<T>`](ValueRenderer) on it once for every type it should show, and install it
//! with [`AssertThat::with_renderer`](crate::AssertThat::with_renderer).
//!
//! Assertions that start a child chain, such as `derive`, `satisfies`, or `is_some_satisfying`,
//! give the child its own copy of the renderer, so they need `R: Clone`. Installing a reference, as
//! in `with_renderer(&renderer)`, works for any renderer.
//!
//! ## What needs a renderer
//!
//! You only render leaves. assertr writes the brackets, commas, and field names around them. An
//! assertion on a `Vec<T>` element therefore needs `ValueRenderer<T>`, and a map assertion needs
//! renderers for the key and value types. Assertions that look at the subject as a whole, such as
//! `is_equal_to` and `has_length`, need a renderer for the whole type. The bounds on each method
//! say exactly what it needs.
//!
//! Counts and lengths are values too. Assertions that report them need `ValueRenderer<usize>`.
//! Errors keep their own type, so a custom renderer can show errors that implement neither `Debug`
//! nor `Display`. There is no fallback. If a method needs a renderer that is missing, it does not
//! compile.
//!
//! A check that never shows a value needs no renderer for it. Here, neither `Secret` nor the array
//! can be rendered, and the assertion still compiles:
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
//! Add only the renderers whose evidence you want. This renderer shows counts and nothing else:
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
//! ## Limit the size of reports
//!
//! A [`RenderingBudget`] caps how many items of a collection are shown and how long each value may
//! be. It shortens the report but never changes whether an assertion passes. Set it with
//! [`AssertThat::with_rendering_budget`](crate::AssertThat::with_rendering_budget). Child chains
//! inherit it. By default, a report shows up to 256 items per group and 4,096 characters per value.
//! Use [`RenderingBudget::unlimited`] when you need everything.
//!
//! Values left out by the budget are gone. Neither captured failures nor a custom panic
//! presentation can recover them.
//!
//! ## Rendered values
//!
//! Rendering produces a [`Rendered`] tree that keeps the text of each leaf, the structure around
//! it, type information, and how many items were left out.
//! [`AssertionFailure`](crate::failure::AssertionFailure) stores these trees, so code that
//! processes failures can inspect them without parsing text. To change the layout of panic
//! messages, use [`with_panic_presentation`](crate::AssertThat::with_panic_presentation).
//!
//! assertr writes some text itself: field names and positions in failure paths, omission notes,
//! type names, labels such as `2xx`, and prose passed in by the assertion author. Map keys in
//! paths, counts, lengths, and `any_of` branch numbers are values and go through the renderer.
//! Converting a string, format arguments, or a primitive into [`Rendered`] produces such verbatim
//! text, which bypasses the renderer and the budget. Use these conversions only for that kind of
//! text. Render values through a [`RenderingContext`].
//!
//! ## Render values in custom assertions
//!
//! A custom assertion gets a [`RenderingContext`] from
//! [`AssertThat::render`](crate::AssertThat::render), or from
//! [`AssertionContext::render`](crate::expectation::AssertionContext::render) inside an
//! expectation. It applies the chain's renderer and budget. Pass its results to the failure builder
//! or to [`Fact`](crate::failure::Fact) constructors:
//!
//! | To show | Use |
//! | --- | --- |
//! | One value | [`value`](RenderingContext::value) |
//! | A collection, presented as usual | [`collection`](RenderingContext::collection) |
//! | A collection where positions matter | [`stable_collection`](RenderingContext::stable_collection) |
//! | A map | [`map`](RenderingContext::map) |
//! | A list built for the report | [`borrowed_values`](RenderingContext::borrowed_values) |
//! | Key and value pairs built for the report | [`entry_list`](RenderingContext::entry_list) |
//! | A value inside an enum variant or struct | [`variant`](RenderingContext::variant), [`struct_field`](RenderingContext::struct_field) |
//! | A struct field that cannot be accessed | [`unavailable_struct_field`](RenderingContext::unavailable_struct_field) |
//!
//! Lists built for the report have no Rust type of their own, so you choose their
//! [`RenderingOrder`]. Collections follow their [`CollectionPresentation`]. `stable_collection`
//! requires [`StableOrder`](crate::assertions::StableOrder) and keeps iteration order.
//! [`RenderingContext::compact`] renders values in their single-line form.
//!
//! Every method needs renderers only for the leaves it shows and applies the budget. Sorted lists
//! are sorted by their rendered text before the budget drops items. The budget limits the output,
//! not the work of walking the value.
//!
//! If you collect evidence yourself, read the limits from [`RenderingContext::budget`]. Decide the
//! result of the check before applying them, and record what you left out on the failure builder.
//! The [structural evidence guide](crate#structural-evidence) shows a complete example.

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
