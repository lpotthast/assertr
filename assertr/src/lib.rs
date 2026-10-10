#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Allow functions named `is_*`, taking self by value instead of taking self by mutable reference or
// reference.
#![allow(clippy::wrong_self_convention)]
//! # assertr
//!
//! [![Crates.io](https://img.shields.io/crates/v/assertr.svg)](https://crates.io/crates/assertr)
//! [![Docs.rs](https://docs.rs/assertr/badge.svg)](https://docs.rs/assertr)
//! [![CI](https://github.com/lpotthast/assertr/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/assertr/actions/workflows/ci.yml)
//! [![MSRV](https://img.shields.io/badge/MSRV-1.89.0-blue.svg)](https://github.com/lpotthast/assertr/blob/main/assertr/Cargo.toml)
//! [![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/assertr.svg)](#license)
//!
//! Fluent assertions for Rust. Pass the value under test to `assert_that!` and chain the checks it
//! should pass. Autocomplete offers only the assertions that fit the value's type, and every
//! failure explains what was found and what was expected.
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! assert_that!("hello, world!")
//!     .starts_with("hello")
//!     .ends_with("!");
//!
//! let numbers = vec![1, 2, 3];
//! assert_that!(numbers).has_length(3).contains(2);
//! ```
//!
//! Had the first chain expected `"?"` instead of `"!"`, the test would fail with:
//!
//! ```text
//! -------- assertr --------
//! Assertion failed at tests/greeting.rs:5:6
//!
//! Expression: `"hello, world!"`
//!
//! Actual: "hello, world!"
//!
//! does not end with
//!
//! Expected: "?"
//! -------- assertr --------
//! ```
//!
//! Compared to `assert!` and `assert_eq!`, the value under test always comes first, one chain
//! replaces a series of separate macro calls, and the report names the relation that did not hold.
//! assertr works in `std` and `no_std` builds.
//!
//! ## Installation
//!
//! ```toml
//! [dev-dependencies]
//! assertr = "0.8.0"
//! ```
//!
//! Everything beyond `std` and `num` is opt-in:
//!
//! | Feature                                         | Adds                                                                                     |
//! |-------------------------------------------------|------------------------------------------------------------------------------------------|
//! | `std` (default)                                 | Assertions for `HashMap`, `HashSet`, `Path`, `Command`, `Mutex`, and panicking closures. |
//! | `num` (default)                                 | Numeric assertions such as `is_zero`, `is_positive`, `is_nan`, and `is_close_to`.        |
//! | `libm`                                          | Floating-point checks like `is_nan` for `num` in `no_std` builds. Does not enable `num`. |
//! | `partial`                                       | The `partial!` macro for matching selected struct fields.                                |
//! | `fluent`                                        | The `value.must()` and `value.verify(..)` entry points with fluent method names.         |
//! | `serde-json`, `serde-toml`                      | `as_json()` and `as_toml()` to assert on a value's serialized form.                      |
//! | `serde`                                         | Both `serde-json` and `serde-toml`.                                                      |
//! | `program`                                       | Checks that a program name or path resolves to an executable, like `which`.              |
//! | `http`, `jiff`, `reqwest`, `rootcause`, `tokio` | Assertions for types of the crate with the same name.                                    |
//! | `thirtyfour`                                    | Async assertions on browser elements, extracting values for ordinary assertions.         |
//! | `thirtyfour-cdp`                                | Chromium accessibility descriptions, in addition to `thirtyfour`.                        |
//! | `full`                                          | All of the above.                                                                        |
//!
//! For `no_std`, disable the default features. `num`, `libm`, `partial`, `fluent`, `rootcause`, and
//! the `serde` features work with `alloc` alone. All other features enable `std`.
//!
//! ## Writing assertions
//!
//! Import the prelude. It brings every assertion trait into scope, so autocomplete can list them.
//! Then start a chain with `assert_that!` and add assertions. The first failing assertion panics.
//!
//! `assert_that!` borrows its argument, so you can keep using a value after asserting on it.
//! Closure literals are the exception: `assert_that!` owns them, so `assert_that!(|| ..).panics()`
//! can run the closure. For other assertions that consume their subject, such as draining an
//! iterator, start with `assert_that_owned!`:
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! let name = String::from("Ada");
//! assert_that!(name).is_equal_to("Ada");
//! assert_that!(name.len()).is_equal_to(3); // `name` is still usable.
//!
//! assert_that_owned!((1..=3).map(|n| n * n)).contains_exactly([1, 4, 9]);
//! ```
//!
//! Some assertions continue with a different subject. Use this to check what is inside an `Option`,
//! a `Result`, or a panic:
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! assert_that!(Some(42)).get_some().is_greater_than(40);
//! assert_that!("42".parse::<u32>()).get_ok().is_equal_to(42);
//! # #[cfg(feature = "std")]
//! assert_that!(|| panic!("boom"))
//!     .panics()
//!     .has_message()
//!     .is_equal_to("boom");
//! ```
//!
//! Expected values can be owned or borrowed, and string literals compare with `String` subjects.
//! Pass a reference to reuse an expected value without cloning it:
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! let expected = String::from("Ada");
//! assert_that!(String::from("Ada")).is_equal_to(&expected);
//! assert_that!(vec![String::from("Ada")]).contains_exactly(["Ada"]);
//! ```
//!
//! Reports show values through their `Debug` implementation. For types without one, or to show a
//! value differently, see [rendering values](crate::renderer).
//!
//! ## Checking parts of a value
//!
//! Use `derive` to assert on a field or a computed value. The parent chain stays usable, so you can
//! check several fields one after another:
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let user = User { name: "Ada".into(), age: 36 };
//! let user = assert_that!(user);
//! user.derive(|u| &u.name).starts_with("A");
//! user.derive(|u| &u.age).is_greater_or_equal_to(18);
//! ```
//!
//! With the `partial` feature, `partial!` describes the fields that matter in one expression and
//! ignores the rest with `..`. The struct needs no derives or annotations:
//!
//! ```rust
//! # #[cfg(feature = "partial")]
//! # {
//! use assertr::{matchers::{eq, ge}, prelude::*};
//!
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let user = User { name: "Ada".into(), age: 36 };
//! assert_that!(user).matches(partial!(User { name: eq("Ada"), age: ge(18), .. }));
//! # }
//! ```
//!
//! For a user named Bob aged 17, the report lists both mismatches:
//!
//! ```text
//! does not match
//!
//! Nested failures:
//!   - At .name:
//!     Expected: "Ada"
//!
//!       Actual: "Bob"
//!
//!   - At .age:
//!     Actual: 17
//!
//!     is not greater than or equal to
//!
//!     Expected: 18
//! ```
//!
//! ## Reusable checks
//!
//! `eq` and `ge` above are matchers. A matcher is a check stored in a value, so you can define it
//! once and apply it to a whole value, to the elements of a collection, or to a field in
//! `partial!`:
//!
//! ```rust
//! use assertr::{matchers::{all_of, string, HasLengthOf}, prelude::*};
//!
//! let short_name = all_of(matchers![string::IsNotBlank, HasLengthOf::new(3)]);
//! assert_that!("Ada").matches(&short_name);
//! assert_that!(["", "Ada", "Grace"]).contains_matching(&short_name);
//! ```
//!
//! The [matcher catalog](mod@crate::matchers) lists every built-in matcher. To name your own
//! domain checks, see the [custom assertions
//! guide](https://docs.rs/assertr/latest/assertr/#custom-assertions).
//!
//! ## Collecting failures
//!
//! `capture` runs a chain without panicking and returns all failures. Use it to report several
//! problems at once, or to test your own assertions:
//!
//! ```rust
//! use assertr::prelude::*;
//!
//! let failures = assert_that!(42).capture(|it| it.is_less_than(10).is_equal_to(43));
//! assert_that!(failures).has_length(2);
//! assert_that!(failures[0].to_string()).contains("is not less than");
//! ```
//!
//! Each failure is structured data. Its fields hold the rendered actual and expected values, the
//! relation between them, and nested failures. Its `Display` implementation produces the report.
//!
//! Assertions that switch to a different subject, such as `get_some()`, are not available in
//! capture mode, because a failure leaves no value to continue with. Use their `*_satisfying`
//! variants instead, for example `is_some_satisfying(|value| ..)`.
//!
//! ## Fluent entry points
//!
//! The `fluent` feature lets you start a chain from the value itself. `must()` panics on the first
//! failure. `verify(..)` collects failures like `capture`. Assertion names read as requirements:
//! `is_x` becomes `be_x`, `has_x` becomes `have_x`, `contains` becomes `contain`, and `is_not_x`
//! becomes `not_be_x`.
//!
//! ```rust
//! # #[cfg(feature = "fluent")]
//! # {
//! use assertr::prelude::*;
//!
//! "hello, world!".must().start_with("hello").end_with("!");
//!
//! let failures = 3.verify(|it| it.be_equal_to(4));
//! assert_that!(failures).has_length(1);
//! # }
//! ```
//!
//! Both borrow the value. `must_owned()` and `verify_owned()` take ownership. See
//! [`FluentEntry`](https://docs.rs/assertr/latest/assertr/trait.FluentEntry.html) for
//! all naming rules and for recording the source expression in reports.
//!
//! ## Finding assertions
//!
//! Autocomplete on a chain is the quickest way to discover assertions. To browse, start at the
//! [assertion families](crate::assertions). Each assertion trait lists its methods and what they
//! require from the subject.
//!
//! Many assertions apply to any type with the right capabilities. Your own types get `is_equal_to`
//! from `PartialEq`, `is_greater_than` from `PartialOrd`, and collection assertions by implementing
//! [`Collection`](https://docs.rs/assertr/latest/assertr/assertions/trait.Collection.html).
//!
//! ## Guides
//!
//! The crate documentation on docs.rs goes deeper:
//!
//! - [Core concepts](https://docs.rs/assertr/latest/assertr/#core-concepts): subjects, ownership,
//!   child chains, and the two assertion modes.
//! - [Partial matching](https://docs.rs/assertr/latest/assertr/matchers/index.html#structural-syntax):
//!   nested fields, enum variants, and collection policies such as `each` and `elements_are!`.
//! - [Custom assertions](https://docs.rs/assertr/latest/assertr/#custom-assertions): reusable
//!   checks, your own chainable methods, and diagnostics in the standard report format.
//! - [Rendering values](crate::renderer): types without `Debug`, custom formatting, and limits for
//!   large values.
//! - [Failure handling](crate::failure): inspect failures as data and customize panic messages.
//! - [Async code](https://docs.rs/assertr/latest/assertr/#async-code): async projections, futures
//!   that panic, and `Send` limitations.
//! - [Type properties](https://docs.rs/assertr/latest/assertr/fn.assert_that_type.html): size, type
//!   name, and drop behavior of a type.
//!
//! ## API stability
//!
//! assertr follows Semantic Versioning for its public items unless their documentation says
//! otherwise. The `*Assertions` traits are public so that their methods are available, not for you
//! to implement. Adding methods to them is not a breaking change. `assertr::__private` is internal
//! macro support and must not be used directly.
//!
//! ## MSRV
//!
//! Current MSRV:
//!
//! - `assertr`: `1.89.0`
//! - `assertr-macros`: `1.89.0`
//!
//! Previous MSRV values:
//!
//! - As of `0.1.0`, the MSRV was `1.76.0`
//! - As of `0.2.0`, the MSRV was `1.85.0`
//! - As of `0.4.0`, the MSRV was `1.89.0`
//!
//! ## Contributing
//!
//! Run `just install-tools` once, then `just verify` before opening a pull request. Record notable
//! changes in `CHANGELOG.md` under `## [Unreleased]`.
//!
//! This README is generated from the crate documentation in `assertr/src/lib.rs`. Edit it there and
//! run `just readme`.
//!
//! ## License
//!
//! Licensed under either of:
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/lpotthast/assertr/blob/main/LICENSE-APACHE))
//! - MIT License ([LICENSE-MIT](https://github.com/lpotthast/assertr/blob/main/LICENSE-MIT))
// cargo-rdme extracts the literal rustdoc above but does not expand this inclusion.
// Keep these detailed guides on the crate documentation page, after the landing page.
#![doc = include_str!("crate_docs.md")]

extern crate alloc;
extern crate self as assertr;
#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[doc(hidden)]
pub mod __private;
pub mod actual;
mod assert_that;
pub mod assertions;
#[cfg(any(feature = "serde-json", feature = "serde-toml"))]
mod conversion;
mod details;
mod entry;
pub mod expectation;
pub mod failure;
pub mod matchers;
pub mod mode;
/// One glob import brings every assertion into scope.
///
/// ```
/// use assertr::prelude::*;
/// ```
pub mod prelude;
pub mod renderer;
#[cfg(test)]
mod test_support;
mod tracking;
mod util;

use alloc::{string::String, vec::Vec};
use core::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    panic::AssertUnwindSafe,
};

/// Borrowed view selection for assertion operands.
pub use ::borrow_for;
use actual::Actual;
#[cfg(feature = "fluent")]
pub use assertr_macros::{fluent_aliases, fluent_expressions};
#[cfg(feature = "fluent")]
pub use entry::{FluentEntry, OwnedFluentEntry};
pub use entry::{PanicValue, Type, assert_that_type};
use failure::AssertionFailures;
use mode::Mode;
use renderer::{DebugRenderer, RenderingBudget};

/// Constructs a partial matcher without annotating the production type.
///
/// Requires the `partial` feature. Pass the result to
/// [`.matches(...)`](crate::AssertThat::matches) or a collection
/// assertion such as `.contains_matching(...)`.
///
/// ```
/// use assertr::{matchers::eq, prelude::*};
///
/// struct User {
///     name: &'static str,
///     age: u32,
/// }
///
/// let user = User { name: "Alice", age: 30 };
/// assert_that!(user).matches(partial!(User { name: eq("Alice"), .. }));
/// ```
///
/// List the fields that matter to the test and use `..` to ignore the rest. Every selected field
/// requires a matcher. Use [`matchers::eq(value)`](crate::matchers::eq) for
/// `PartialEq` equality, another `partial!` for nested fields, or
/// [`matchers::satisfying`](crate::matchers::satisfying) to check a field with existing assertion
/// methods.
///
/// Without `..`, every field must be listed. Write `field: _` to list a field without checking
/// it, which keeps the pattern exhaustive, so adding a field to the type breaks the test at compile
/// time. Omitted fields need no comparison or rendering support. Neither the whole type nor ignored
/// fields need `PartialEq` or `Debug`, and private fields follow ordinary Rust visibility rules.
/// Field expectation expressions are evaluated once when the matcher is constructed. Pass
/// `&matcher` to reuse it.
///
/// Named structs, tuple structs, enum variants, and unit constructors are supported. Tuple `_`
/// positions are wildcards like named `_` fields, and a tuple `..` must be final. Nested collection
/// and map expectations use [`elements_are!`], [`elements_are_in_any_order!`],
/// [`each`](crate::matchers::each), and [`entries_are!`].
///
/// Prefix a constructor with `variant` to include its variant in diagnostic paths. Qualified
/// paths, including `variant crate::Message::Ready` and `variant ::core::option::Option::None`,
/// are supported. Use `r#variant::Type` for an unmarked path through a module named `variant`.
///
/// The macro also works through a re-export of `assertr` in another crate. See the
/// [partial matching guide](mod@crate::matchers) for field constraints, nested examples, collection
/// policies, and diagnostics.
#[cfg(feature = "partial")]
#[macro_export]
macro_rules! partial {
    ($($input:tt)*) => {
        $crate::__private::partial!($crate; $($input)*)
    };
}

/// An assertion chain over a subject of type `T`.
///
/// Ownership is stored in [`Actual<T>`](Actual), while assertion methods are selected by `T`.
/// Borrowing a sized reference therefore produces `AssertThat<Value>`. Taking ownership of the same
/// reference produces `AssertThat<&Value>`. Unsized targets such as `str` and `[T]` use
/// shared-reference subjects.
///
/// `'t` is the lifetime of a borrowed subject. `M` is [`mode::Panic`] or [`mode::Capture`]. `R` is
/// the active renderer. Renderer capabilities are required by individual methods rather than by the
/// chain, so projections can preserve `R` even when it cannot render every intermediate subject.
///
/// Derived chains keep their parent's mode and inherit its detail messages. Their failures and
/// assertion counts propagate to every ancestor, and failures are stored at the root. A failure on
/// a child therefore behaves as a failure on the root.
///
/// ## Unwind safety
///
/// An assertion chain preserves the unwind-safety requirements of its subject and renderer:
///
/// | Chain trait | Subject bound | Renderer bound |
/// |---|---|---|
/// | [`UnwindSafe`](core::panic::UnwindSafe) | `T: UnwindSafe + RefUnwindSafe` | `R: UnwindSafe` |
/// | [`RefUnwindSafe`](core::panic::RefUnwindSafe) | `T: RefUnwindSafe` | `R: RefUnwindSafe` |
///
/// Both modes have these guarantees. The subject needs both bounds for `UnwindSafe` because
/// [`Actual<T>`](crate::actual::Actual) can hold either `T` or `&T`, even when this particular
/// chain was created with `assert_that_owned!`. Internal counters, messages, and captured
/// failures do not impose additional unwind bounds. A child's ancestor links reference only
/// assertion records. Its unwind safety depends on its own subject and renderer, even if an
/// ancestor's values do not implement the corresponding traits.
///
/// Ordinary construction, projections, renderers, and assertion callbacks do not require these
/// traits. Panic assertions such as `panics()` still accept closures capturing mutable state. They
/// catch the closure's panic explicitly and do not restore captured state. The bounds above matter
/// when passing an existing chain to an API such as `std::panic::catch_unwind`.
///
/// For example, sharing a chain containing a `Cell` across a catch boundary is rejected:
///
/// ```compile_fail,E0277
/// use std::{cell::Cell, panic::catch_unwind};
/// use assertr::assert_that;
///
/// let value = Cell::new((0, 0));
/// let chain = assert_that!(value);
/// let _ = catch_unwind(|| {
///     chain.actual().set((1, 0));
///     panic!("interrupted update");
/// });
/// ```
///
/// The same applies to an owned `RefCell` accessed through a shared chain:
///
/// ```compile_fail,E0277
/// use std::{cell::RefCell, panic::catch_unwind};
/// use assertr::assert_that_owned;
///
/// let chain = assert_that_owned!(RefCell::new((0, 0)));
/// let _ = catch_unwind(|| {
///     chain.actual().borrow_mut().0 = 1;
///     panic!("interrupted update");
/// });
/// ```
///
/// Moving an owned `Cell` chain also requires `RefUnwindSafe`, due to the shared subject
/// representation:
///
/// ```compile_fail,E0277
/// use std::{cell::Cell, panic::catch_unwind};
/// use assertr::assert_that_owned;
///
/// let chain = assert_that_owned!(Cell::new(0));
/// let _ = catch_unwind(move || drop(chain));
/// ```
///
/// Mutable-reference subjects additionally prevent moving a chain across the boundary:
///
/// ```compile_fail,E0277
/// use std::panic::catch_unwind;
/// use assertr::assert_that_owned;
///
/// let mut value = 0;
/// let chain = assert_that_owned!(&mut value);
/// let _ = catch_unwind(move || drop(chain));
/// ```
///
/// Renderer state participates even when the operation does not render a value:
///
/// ```compile_fail,E0277
/// use std::{cell::Cell, panic::catch_unwind};
/// use assertr::assert_that;
///
/// let calls = Cell::new(0);
/// let chain = assert_that!(1).with_debug_format(move |value, f| {
///     calls.set(calls.get() + 1);
///     write!(f, "{value}")
/// });
/// let _ = catch_unwind(|| chain.actual());
/// ```
///
/// An owned renderer can implement `RefUnwindSafe` without implementing `UnwindSafe`. For example,
/// installing a mutable reference is permitted, but moving that chain into `catch_unwind` is
/// rejected:
///
/// ```compile_fail,E0277
/// use std::panic::catch_unwind;
/// use assertr::{assert_that, renderer::DebugRenderer};
///
/// let mut renderer = DebugRenderer;
/// let chain = assert_that!(1).with_renderer(&mut renderer);
/// let _ = catch_unwind(move || drop(chain));
/// ```
///
/// After reviewing the captured state, callers can explicitly assume responsibility with
/// [`AssertUnwindSafe`]:
///
/// ```
/// use std::{cell::Cell, panic::{AssertUnwindSafe, catch_unwind}};
/// use assertr::prelude::*;
///
/// let value = Cell::new(0);
/// let chain = assert_that!(value);
/// let result = catch_unwind(AssertUnwindSafe(|| {
///     chain.actual().set(1);
///     panic!("after the update");
/// }));
/// assert_that!(result).is_err();
/// assert_that!(value.get()).is_equal_to(1);
/// ```
pub struct AssertThat<'t, T, M: Mode, R = DebugRenderer> {
    actual: Actual<'t, T>,
    state: ChainState<'t, M, R>,
}

/// The source expression of one diagnostic subject. Fluent capture roots defer attachment until
/// `#[fluent_expressions]` sees their completed failures, leaving callback arguments untouched.
#[derive(Clone, Copy)]
enum Expression {
    Unset,
    #[cfg(feature = "fluent")]
    PendingFluent(&'static core::panic::Location<'static>),
    Explicit(&'static str),
}

impl Expression {
    /// The expression recorded by an entry macro, fluent rewriting, or an explicit override.
    fn explicit(self) -> Option<&'static str> {
        if let Self::Explicit(expression) = self {
            Some(expression)
        } else {
            None
        }
    }

    /// The location of a fluent root whose receiver expression is attached after completion.
    #[cfg(feature = "fluent")]
    fn pending_fluent(self) -> Option<&'static core::panic::Location<'static>> {
        if let Self::PendingFluent(location) = self {
            Some(location)
        } else {
            None
        }
    }
}

/// Everything a projection preserves while replacing its subject.
///
/// Keeping this separate from `AssertThat` lets `map`, async mappings, and extractions move the
/// entire state without reconstructing its fields. It has no subject type parameter, so changing
/// `T` preserves the records, diagnostic settings, mode, and renderer automatically.
///
/// User-provided rendering and presentation state stays here, outside the unwind exemptions on
/// `ChainRecords`. Auto traits therefore continue to check it as part of the assertion chain.
struct ChainState<'t, M: Mode, R> {
    /// This node's records and its link to ancestor records.
    records: ChainRecords<'t>,

    /// How failures raised on this chain are described and presented.
    settings: DiagnosticSettings,

    /// Compile-time marker selecting immediate panics or failure collection. Derived chains retain
    /// the same mode.
    mode: PhantomData<M>,

    /// Active renderer for diagnostic leaf values, preserved by mappings and cloned for derived
    /// chains.
    ///
    /// `R` is intentionally not constrained by `ValueRenderer<T>` here. A chain must be able to
    /// install a renderer after construction (including for a non-`Debug` `T`), and projections
    /// preserve `R` while changing `T` even when the next assertion does not render the new
    /// subject. Each assertion method must instead declare the exact `ValueRenderer<U>`
    /// capabilities used by its failure path. Keep blanket assertion-trait impls
    /// renderer-unconstrained so one unavailable rendering capability does not hide the entire
    /// trait.
    renderer: R,
}

/// The diagnostic settings of a chain, independent of its subject, records, and renderer.
///
/// A chain detached across an await keeps these settings without its records, so this type holds
/// no cells and no borrows. It is `Send` and `Sync`.
struct DiagnosticSettings {
    /// Optional user-provided subject name shown in failure diagnostics. Derived chains start
    /// without a name because they describe a new subject.
    subject_name: Option<String>,

    /// Source expression shown in failure diagnostics, usually recorded by an entry macro or
    /// fluent-expression rewriting. Derived chains start without an expression.
    expression: Expression,

    /// Whether failures record the assertion caller's file, line, and column. Derived chains
    /// inherit this setting. Tests can disable it when comparing exact failure reports.
    include_location: bool,

    /// Limits items per repeated diagnostic group and characters per rendered leaf. Derived chains
    /// inherit these limits, which the rendering context applies in both panic and capture mode.
    rendering_budget: RenderingBudget,

    /// An inherited override for panic text. `None` uses the failure's `Display` report. Capture
    /// mode never invokes presentation. `Arc` shares the closure with derived chains without
    /// requiring it to be `Clone`, and keeps it `Send` for assertions that await.
    panic_presentation: Option<alloc::sync::Arc<failure::panic_presentation::PanicPresentation>>,
}

/// Messages, assertion counts, and captured failures for one node in an assertion chain.
///
/// Each child starts with its own records. Assertion counts propagate through every ancestor,
/// failures are stored at the root, and diagnostics collect local messages before ancestor
/// messages. Starting `capture` detaches the parent link and retains the ancestor messages as
/// inherited messages, which still follow every local message.
///
/// Parent links expose only these records, never the parent's subject, renderer, or panic
/// presentation. This lets a child retain its ancestry without requiring the parent's user values
/// to be unwind safe. The child's own subject and renderer still determine its auto traits.
struct ChainRecords<'t> {
    /// The ancestor records used for propagation. `None` marks a root, including one created by
    /// `capture`. This deliberately does not point to an entire assertion chain or `ChainState`.
    parent: Option<&'t ChainRecords<'t>>,

    // These exemptions cover only library-owned data. User conversions and rendering finish
    // before a mutable borrow is taken. If a borrow, allocation, or counter increment panics,
    // guards are released and each cell retains a valid value. Completed messages, failures, and
    // attempted assertion counts need no rollback, and no completion contract runs on drop.
    // Keep the exemptions on these fields so future fields must establish their own unwind safety.
    /// Local messages, collected before ancestor messages.
    detail_messages: AssertUnwindSafe<RefCell<Vec<String>>>,
    /// Includes assertions attempted on derived chains, even when an assertion panics.
    ///
    /// [`AssertThat::capture`] uses the count to reject capture closures that perform no
    /// assertions. In panic mode, unused assertion chains are flagged instead, by
    /// `unused_must_use` warnings from the `#[must_use]` annotations on the entry points.
    number_of_assertions: AssertUnwindSafe<Cell<usize>>,
    /// Captured failures owned by this chain. Children forward failures through `parent`.
    failures: AssertUnwindSafe<RefCell<AssertionFailures>>,
    /// Ancestor messages retained when `capture` detached this chain from its parent. They are
    /// collected after local messages, exactly where the detached parent's messages would appear.
    /// This field is never mutated after construction and needs no unwind exemption.
    inherited_messages: Vec<String>,
}
