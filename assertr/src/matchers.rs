//! Reusable expectations for direct assertions, collection elements, and structural matching.
//!
//! An **expectation** defines a check and its diagnostics. A **matcher** is an expectation used
//! with [`matches`](crate::assertions::matcher::MatcherAssertions::matches), a `*_matching`
//! method, or composition. Both uses execute the same [`Expectation`] and
//! [`ExpectationDiagnostics`] traits. There is no separate matcher trait or registration step.
//!
//! This module is the catalog of all public built-in expectations. General comparisons, lengths,
//! variants, and composition helpers are available directly. Subject families have namespaces to
//! distinguish names such as [`string::Contains`] and [`collection::Contains`]. Existing short
//! constructors, including [`eq`], [`equal_to`], [`ge`], and [`starts_with`], are available
//! directly.
//!
//! ## Build and reuse a check
//!
//! Use a unit value such as [`IsSome`] for a parameterless check, or `Type::new(...)` to supply
//! expected operands. Convenience functions construct the same types: `ge(18)` constructs
//! [`GreaterOrEqual`]. Pass a reference to reuse a definition without cloning it.
//!
//! ```rust
//! use assertr::{matchers::{all_of, HasLengthOf, IsSome, string}, prelude::*};
//!
//! let three_letters = all_of((string::IsNotBlank, HasLengthOf::new(3)));
//! assert_that!("Ada").matches(&three_letters);
//! assert_that!(["", "Ada", "Grace"]).contains_matching(&three_letters);
//! assert_that!([None, Some(42)]).contains_matching(IsSome);
//! ```
//!
//! [`all_of`] requires every branch to pass. [`any_of`] stops at the first passing branch.
//! Explicit negative definitions, such as [`NotEqualTo`], [`IsNone`], and
//! [`DoesNotMatchPattern`], own their checks and diagnostic evidence. There is no generic `not`.
//! See [`MatcherList`] for tuples, arrays, and the [`matchers!`](crate::matchers!) list macro.
//!
//! A matcher checks the original subject. It does not change the chain's subject type:
//!
//! ```rust
//! use assertr::{matchers::IsSome, prelude::*};
//!
//! assert_that!(Some(42)).matches(IsSome).get_some().is_equal_to(42);
//! ```
//!
//! `matches(IsSome)` checks presence. `get_some()` extracts the payload. Custom projection
//! methods can use [`AssertThat::test_assertion`](crate::AssertThat::test_assertion) to obtain a
//! definition's successful observation. Composition drops successful observations before checking
//! another branch, which also releases acquired guards.
//!
//! ## Find an expectation
//!
//! | Subject or check | Import from this module |
//! |---|---|
//! | Equality and ordering | [`EqualTo`], [`NotEqualTo`], [`LessThan`], [`GreaterThan`], [`LessOrEqual`], [`GreaterOrEqual`] |
//! | Boolean, variant, and type checks | [`IsTrue`], [`IsFalse`], [`IsSome`], [`IsNone`], [`IsOk`], [`IsErr`], [`IsReady`], [`IsPending`], [`IsOfType`] |
//! | Length, formatting, and identity | [`HasLengthOf`], [`IsEmpty`], [`IsNotEmpty`], [`HasDebugString`], [`HasDebugValue`], [`HasDisplayValue`], [`IsSameInstanceAs`], [`IsNotSameInstanceAs`] |
//! | Characters and strings | [`character`], [`string`] |
//! | Elements, maps, and sets | [`collection`], [`map`], [`set`] |
//! | Ranges, cell borrows, and remaining iterator counts | [`range`], [`cell`], [`iterator`] |
//! | Numeric properties and tolerances | `numeric` with `num`. Floating-point classifications also need `std` or `libm`. |
//! | Paths, commands, mutexes, and drop requirements | `path`, `command`, `mutex`, `memory` with `std` |
//! | HTTP headers and responses | `header_value` with `http`, `response` with `reqwest` |
//! | Jiff values | `signed_duration`, `span`, `zoned` with `jiff` |
//! | Executable lookup and reports | `program` with `program`, `report` with `rootcause` |
//! | Tokio locks and channels | `tokio_mutex`, `tokio_rw_lock`, `watch` with `tokio` |
//!
//! Each type documents its constructor, supported subjects, and diagnostic bounds. The catalog
//! re-exports the original types. Their paths under [`assertions`](crate::assertions) continue to
//! work. Start with explicit imports, or use `matchers::*` alongside `prelude::*` to browse the
//! catalog with autocomplete.
//!
//! ## Execution and features
//!
//! Runtime expectations and collection/map macros need no optional feature and support `no_std`
//! with `alloc`. The `matchers` feature enables only the
//! [`partial!`](mod@crate::matchers#structural-syntax) procedural macro. Integration expectations
//! have the same feature requirements as their ordinary assertion methods.
//!
//! Matching borrows the subject. Available checks depend on its capabilities and the active
//! renderer. Renderer bounds apply to diagnostic leaves, and the rendering budget limits evidence
//! without changing whether the check passes. Reusable does not mean side-effect free: predicates,
//! callbacks, and lock observations can affect state. Composition can evaluate an expectation
//! against several subjects or candidate pairs. Explanation uses the original rejection and does
//! not repeat the check. User panics propagate.
//!
//! Ordinary methods that consume a function or iterator, or await a response body, own those
//! execution steps. They also use the expectation traits internally, with private adapters that
//! are not reusable public matchers. `matches(...)` does not invoke an owned `FnOnce` or await I/O.
//! Use the corresponding ordinary assertion for those operations. [`satisfying`] adapts reusable
//! capture-mode assertion callbacks, including custom assertion methods, into composition.
//!
//! ## Structural syntax
//!
//! Use `partial!` to check selected fields of a struct or enum. The production type needs no
//! derives or attributes:
//!
//! ```rust
//! # #[cfg(feature = "matchers")]
//! # {
//! use assertr::{matchers::*, prelude::*};
//!
//! struct Secret;
//! struct User {
//!     name: String,
//!     age: u32,
//!     roles: Vec<&'static str>,
//!     secret: Secret,
//! }
//! let user = User {
//!     name: "Alice".into(),
//!     age: 30,
//!     roles: vec!["reader", "editor"],
//!     secret: Secret,
//! };
//!
//! assert_that!(user).matches(partial!(User {
//!     name: eq("Alice"),
//!     age: ge(18),
//!     roles: HasLengthOf::new(2),
//!     ..
//! }));
//! # }
//! ```
//!
//! [`eq`] is a short alias for [`equal_to`] and uses `PartialEq`, so the `String` name can match
//! a string literal. Fields use explicit matchers, nested `partial!` expectations, or
//! [`satisfying`] to run ordinary and custom assertion methods. A `satisfying` callback runs in
//! capture mode and returns `()`, so end its final assertion with a semicolon. Every assertion must
//! pass. Empty callbacks panic.
//!
//! `..` excludes the remaining fields from comparison and diagnostics. Only selected fields need
//! comparison or rendering support. Without `..`, every field must be listed. Private fields
//! follow ordinary Rust visibility rules.
//!
//! Structs and enum variants support named, tuple, and unit syntax. In tuples, `_` skips one
//! field and a final `..` skips the rest. The `variant` prefix includes the enum variant in
//! diagnostic paths:
//!
//! ```rust
//! # #[cfg(feature = "matchers")]
//! # {
//! use assertr::{matchers::eq, prelude::*};
//!
//! struct Pair(i32, i32);
//! assert_that!(Pair(1, 2)).matches(partial!(Pair(eq(1), _)));
//! assert_that!(Some(3)).matches(partial!(variant Some(eq(3))));
//! # }
//! ```
//!
//! Macro entries require explicit matchers, evaluated once in source order when the matcher is
//! built. Use [`eq(value)`](eq) or [`equal_to(value)`](equal_to) for equality, including inside
//! macros. An expression implementing the expectation traits is always used as a matcher,
//! even if it also supports equality. The same rule applies to `.matches(eq(42))`.
//!
//! ## Collection policies
//!
//! Choose how elements or entries should match:
//!
//! | Matcher | Requirement |
//! |---|---|
//! | [`each(matcher)`](each) | Every element matches. Empty collections pass. |
//! | [`elements_are!`](crate::elements_are) | Exactly these elements, in this order. Requires [`StableOrder`](crate::assertions::collection::StableOrder). |
//! | [`elements_are_in_any_order!`](crate::elements_are_in_any_order) | Exactly these elements in any order, preserving duplicate counts. |
//! | [`entries_are!`](crate::entries_are) | Exactly these keys, each with a matching value. Uses native map lookup. |
//!
//! ```rust
//! use assertr::{matchers::*, prelude::*};
//! use std::collections::BTreeMap;
//!
//! assert_that!([18, 30]).matches(each(ge(18)));
//! assert_that!([18, 30]).matches(elements_are![eq(18), ge(20)]);
//! assert_that!([30, 18]).matches(elements_are_in_any_order![eq(18), ge(20)]);
//! assert_that!(BTreeMap::from([("Ada", 36), ("Grace", 85)]))
//!     .matches(entries_are![("Ada", ge(18)), ("Grace", eq(85))]);
//! ```
//!
//! These matchers also work as fields inside `partial!`, and their entries can contain nested
//! `partial!` expectations. Unordered matching gives each expectation a distinct element, even
//! when constraints overlap. B-tree collections work with `alloc`. Hash collections require `std`.
//! In `entries_are!`, keys remain lookup operands and values require matchers.
//!
//! ## Custom expectations and diagnostics
//!
//! Use [`predicate`] for a boolean check, such as `predicate(|n: &i32| n % 2 == 0)`, or
//! [`condition`] to wrap an [`AssertrCondition`](crate::condition::AssertrCondition) with a typed
//! rejection error. [`pattern!`](crate::pattern) matches Rust patterns. [`satisfying`] combines
//! existing assertion methods and documents callback bounds and type annotations.
//!
//! For a reusable check with custom diagnostics, implement [`Expectation`]
//! and [`ExpectationDiagnostics`]. Ordinary assertions and matcher composition execute the same
//! definitions. See [`expectation`](crate::expectation) for the evaluation and diagnostic
//! contracts.
//!
//! Failures identify selected fields, positions, and keys through
//! [`AssertionFailure::path`](crate::AssertionFailure::path). Use
//! [`capture`](crate::AssertThat::capture) to inspect them and the [rendering
//! guide](crate::renderer) to configure diagnostic values. Reference-valued subjects may need
//! [`dereferenced`].

pub use crate::{
    assertions::{
        collection::{
            Each, ElementsAre, ElementsAreInAnyOrder, contains_contiguous_elements,
            contains_matching, contains_no_matching, each, elements_are, elements_are_in_any_order,
            ends_with_elements, starts_with_elements,
        },
        condition::{Condition, condition},
        core::{
            partial_eq::{EqualTo, NotEqualTo, eq, equal_to},
            partial_ord::{GreaterOrEqual, GreaterThan, LessOrEqual, LessThan, ge, gt, le, lt},
            string::starts_with,
        },
        map::{EntriesAre, Entry, EntryMatcherList, entries_are, entry, entry_matchers},
    },
    expectation::{
        AllOf, AnyOf, Anything, Dereferenced, Expectation, ExpectationDiagnostics, MatcherList,
        Predicate, Satisfying, all_of, any_of, anything, dereferenced, predicate, predicate_list,
        satisfying,
    },
};

/// Primitive, variant, length, formatting, and identity expectations are available directly.
#[doc(inline)]
pub use crate::assertions::{
    alloc::boxed::IsOfType,
    core::{
        bool::{IsFalse, IsTrue},
        debug::{HasDebugString, HasDebugValue},
        display::HasDisplayValue,
        identity::{IsNotSameInstanceAs, IsSameInstanceAs},
        length::{HasLengthOf, IsEmpty, IsNotEmpty},
        option::{IsNone, IsSome},
        pattern::{DoesNotMatchPattern, Pattern},
        poll::{IsPending, IsReady},
        result::{IsErr, IsOk},
    },
};

/// Character case and equality expectations.
pub mod character {
    #[doc(inline)]
    pub use crate::assertions::core::char::{
        EqualToIgnoringAsciiCase, IsAsciiLowercase, IsAsciiUppercase, IsLowercase, IsUppercase,
    };
}

/// String contents, prefixes, suffixes, and whitespace expectations.
pub mod string {
    #[doc(inline)]
    pub use crate::assertions::core::string::{
        Contains, DoesNotContain, DoesNotEndWith, DoesNotStartWith, EndsWith,
        EqualToIgnoringAsciiCase, IsBlank, IsBlankAscii, IsNotBlank, StartsWith, starts_with,
    };
}

/// Element membership, ordering, identity, and projection expectations.
pub mod collection {
    #[doc(inline)]
    pub use crate::assertions::collection::{
        Contains, ContainsAll, ContainsContiguous, ContainsExactly, ContainsExactlyInAnyOrder,
        ContainsExactlySameInstances, ContainsExactlySameInstancesInAnyOrder, ContainsMatching,
        ContainsNoMatching, ContainsSameInstanceAs, DoesNotContain, DoesNotContainSameInstanceAs,
        Each, ElementsAre, ElementsAreInAnyOrder, EndsWith, HasElementAt, HasFirst, HasLast,
        HasSingle, StartsWith, contains_contiguous_elements, contains_matching,
        contains_no_matching, each, elements_are, elements_are_in_any_order, ends_with_elements,
        starts_with_elements,
    };
}

/// Key, value, entry, and exact map expectations.
pub mod map {
    #[doc(inline)]
    pub use crate::assertions::map::{
        ContainsEntry, ContainsEntryMatching, ContainsExactlyEntries, ContainsKey, ContainsKeys,
        ContainsValue, ContainsValueMatching, DoesNotContainEntry, DoesNotContainKey,
        DoesNotContainValue, EntriesAre, Entry, EntryMatcherList, entries_are, entry,
        entry_matchers,
    };
}

/// Subset, superset, and disjointness expectations.
pub mod set {
    #[doc(inline)]
    pub use crate::assertions::set::{IsDisjointFrom, IsSubsetOf, IsSupersetOf};
}

/// Range membership and bound expectations.
pub mod range {
    #[doc(inline)]
    pub use crate::assertions::core::range::{
        ContainsElement, DoesNotContainElement, IsInRange, IsNotInRange,
    };
}

/// Borrow-state expectations for `core::cell::RefCell`.
pub mod cell {
    #[doc(inline)]
    pub use crate::assertions::core::ref_cell::{
        IsBorrowed, IsMutablyBorrowed, IsNotMutablyBorrowed,
    };
}

/// Reusable remaining-count expectations for exact-size iterators.
pub mod iterator {
    #[doc(inline)]
    pub use crate::assertions::core::iter::{
        HasNoRemainingElements, HasRemainingCount, HasRemainingElements,
    };
}

/// Numeric properties and tolerance expectations. Requires `num`.
#[cfg(feature = "num")]
pub mod numeric {
    #[doc(inline)]
    pub use crate::assertions::num::{IsCloseTo, IsNegative, IsOne, IsPositive, IsZero};
    /// Floating-point classifications also require `std` or `libm`.
    #[cfg(any(feature = "std", feature = "libm"))]
    #[doc(inline)]
    pub use crate::assertions::num::{IsFinite, IsInfinite, IsNan, IsNormal, IsSubnormal};
}

/// Command argument expectations. Requires `std`.
#[cfg(feature = "std")]
pub mod command {
    #[doc(inline)]
    pub use crate::assertions::std::command::HasArg;
}

/// Drop-requirement expectations. Requires `std`.
#[cfg(feature = "std")]
pub mod memory {
    #[doc(inline)]
    pub use crate::assertions::std::mem::NeedsDrop;
}

/// Lock and poison-state expectations for standard mutexes. Requires `std`.
#[cfg(feature = "std")]
pub mod mutex {
    #[doc(inline)]
    pub use crate::assertions::std::mutex::{IsLocked, IsNotLocked, IsNotPoisoned, IsPoisoned};
}

/// Filesystem state and path-component expectations. Requires `std`.
#[cfg(feature = "std")]
pub mod path {
    #[doc(inline)]
    pub use crate::assertions::std::path::{
        DoesNotExist, EndsWith, Exists, HasARoot, HasExtension, HasFileName, HasFileStem,
        IsADirectory, IsAFile, IsASymlink, IsRelative, StartsWith,
    };
}

/// ASCII header-value expectations. Requires `http`.
#[cfg(feature = "http")]
pub mod header_value {
    #[doc(inline)]
    pub use crate::assertions::http::header_value::IsAscii;
}

/// Signed-duration properties and tolerances. Requires `jiff`.
#[cfg(feature = "jiff")]
pub mod signed_duration {
    #[doc(inline)]
    pub use crate::assertions::jiff::signed_duration::{IsCloseTo, IsNegative, IsPositive, IsZero};
}

/// Span sign and zero expectations. Requires `jiff`.
#[cfg(feature = "jiff")]
pub mod span {
    #[doc(inline)]
    pub use crate::assertions::jiff::span::{IsNegative, IsPositive, IsZero};
}

/// Time-zone expectations for zoned values. Requires `jiff`.
#[cfg(feature = "jiff")]
pub mod zoned {
    #[doc(inline)]
    pub use crate::assertions::jiff::zoned::{IsInTimeZone, IsInTimeZoneNamed};
}

/// Executable lookup expectations. Requires `program`.
#[cfg(feature = "program")]
pub mod program {
    #[doc(inline)]
    pub use crate::assertions::program::Exists;
}

/// Response status and header expectations. Requires `reqwest`.
#[cfg(feature = "reqwest")]
pub mod response {
    #[doc(inline)]
    pub use crate::assertions::reqwest::response::{
        DoesNotHaveHeader, HasHeader, HasHeaderValue, HasStatusCode, IsClientError,
        IsInformational, IsRedirection, IsServerError, IsSuccess,
    };
}

/// Report context, attachment, and child expectations. Requires `rootcause`.
#[cfg(feature = "rootcause")]
pub mod report {
    #[doc(inline)]
    pub use crate::assertions::rootcause::report::{
        HasAttachmentCount, HasChildCount, HasCurrentContext, HasCurrentContextDebugString,
        HasCurrentContextDisplayValue, HasCurrentContextType,
    };
}

/// Lock-state and value-callback expectations for Tokio mutexes. Requires `tokio`.
#[cfg(feature = "tokio")]
pub mod tokio_mutex {
    #[doc(inline)]
    pub use crate::assertions::tokio::mutex::{HasValueSatisfying, IsLocked, IsNotLocked};
}

/// Read/write lock-state expectations. Requires `tokio`.
#[cfg(feature = "tokio")]
pub mod tokio_rw_lock {
    #[doc(inline)]
    pub use crate::assertions::tokio::rw_lock::{IsNotLocked, IsReadLocked, IsWriteLocked};
}

/// Watch value and change-state expectations. Requires `tokio`.
#[cfg(feature = "tokio")]
pub mod watch {
    #[doc(inline)]
    pub use crate::assertions::tokio::watch::{HasChanged, HasCurrentValue, HasNotChanged};
}
