//! Reusable checks for whole values, collection elements, and struct fields.
//!
//! A matcher is a check stored in a value. Pass it to
//! [`matches`](crate::AssertThat::matches), to a `*_matching` assertion
//! such as `contains_matching`, or to another matcher. Every matcher implements
//! [`Expectation`](crate::expectation::Expectation), the same trait behind the ordinary assertion
//! methods. So "expectation" and "matcher" name the same thing, and there is no separate trait to
//! implement or register.
//!
//! This module lists every built-in matcher. The most common ones, such as [`eq`], [`ge`], and
//! [`IsSome`], are available directly. Subject-specific matchers live in namespaces such as
//! [`string`] and [`collection`], which keeps names like [`string::Contains`] and
//! [`collection::Contains`] apart.
//!
//! ## Build and reuse a check
//!
//! Unit values such as [`IsSome`] need no arguments. Others take their expected values in
//! `Type::new(..)`, and short functions build the same types: `ge(18)` is
//! [`GreaterOrEqual::new(18)`](GreaterOrEqual::new). Pass `&matcher` to use one definition several
//! times:
//!
//! ```rust
//! use assertr::{matchers::{all_of, HasLengthOf, IsSome, string}, prelude::*};
//!
//! let three_letters = all_of(matchers![string::IsNotBlank, HasLengthOf::new(3)]);
//! assert_that!("Ada").matches(&three_letters);
//! assert_that!(["", "Ada", "Grace"]).contains_matching(&three_letters);
//! assert_that!([None, Some(42)]).contains_matching(IsSome);
//! ```
//!
//! [`all_of`] requires every matcher to pass and reports each one that fails. [`any_of`] passes as
//! soon as one matcher passes. If none does, it reports all of them, numbered by a zero-based
//! `Branch` fact. Use [`matchers!`](crate::matchers!) to list matchers of different types, or an
//! array for matchers of one type (see [`MatcherList`](crate::expectation::MatcherList)).
//!
//! There is no generic `not`. Negated checks such as [`NotEqualTo`], [`IsNone`], and
//! [`DoesNotMatchPattern`] are separate matchers with their own failure reports.
//!
//! `matches` keeps the chain's subject. Use an extracting assertion to continue with the content:
//!
//! ```rust
//! use assertr::{matchers::IsSome, prelude::*};
//!
//! assert_that!(Some(42)).matches(IsSome).some().is_equal_to(42);
//! ```
//!
//! Matchers implement `Debug` and `Clone` when their contents do. Matchers without generic
//! contents, such as [`IsSome`] and [`HasLengthOf`], and identity matchers such as
//! [`IsSameInstanceAs`] are also `Copy`. Matchers holding a closure, such as [`Predicate`] and
//! [`Pattern`], leave it out of their `Debug` output, and identity matchers show addresses.
//!
//! ## Find a matcher
//!
//! | Subject or check | Matchers |
//! |---|---|
//! | Equality and ordering | [`eq`], [`EqualTo`], [`NotEqualTo`], [`one_of`], [`lt`], [`le`], [`gt`], [`ge`], and their types |
//! | Tolerance | [`close_to`], [`IsCloseTo`] for numbers and durations implementing [`NumericDistance`](crate::assertions::NumericDistance) |
//! | Booleans, variants, and types | [`IsTrue`], [`IsFalse`], [`IsSome`], [`IsNone`], [`IsOk`], [`IsErr`], [`IsReady`], [`IsPending`], [`IsOfType`], [`HasPanicMessage`] |
//! | Length, formatting, and identity | [`HasLengthOf`], [`IsEmpty`], [`IsNotEmpty`], [`HasDebugString`], [`HasDebugValue`], [`HasDisplayValue`], [`IsSameInstanceAs`], [`IsNotSameInstanceAs`] |
//! | Characters and strings | [`character`], [`string`] |
//! | Element search | [`contains_matching`], [`does_not_contain_matching`], [`starts_with_elements`], [`ends_with_elements`], [`contains_contiguous_elements`] |
//! | Elements, maps, and sets | [`collection`], [`map`], [`set`], [`entry`] |
//! | Ranges, `RefCell` borrows, and iterators | [`range`], [`cell`], [`iterator`] |
//! | Drop behavior | [`memory`] |
//! | Numbers (`num`) | `numeric`. Floating-point checks also need `std` or `libm`. |
//! | Paths, commands, and mutexes (`std`) | `path`, `command`, `mutex` |
//! | HTTP (`http`, `reqwest`) | `header_value`, `response` |
//! | Jiff (`jiff`) | `signed_duration`, `span`, `zoned` |
//! | Programs and reports (`program`, `rootcause`) | `program`, `report` |
//! | Tokio (`tokio`) | `tokio_mutex`, `tokio_rw_lock`, `watch` |
//! | Combining checks | [`all_of`], [`any_of`], [`each`], [`field`], [`predicate`], [`satisfying`], [`dereferenced`], [`anything`] |
//!
//! Each matcher's page lists the subjects it supports and what it needs to render a failure. This
//! module is the public home of these types. Import `matchers::*` next to `prelude::*` if you want
//! to browse them with autocomplete.
//!
//! ## Structural syntax
//!
//! With the `partial` feature, `partial!` checks selected fields of a struct or enum. The type
//! needs no derives or attributes:
//!
//! ```rust
//! # #[cfg(feature = "partial")]
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
//! Each listed field takes a matcher. Plain values are not accepted, so write `eq(value)` for
//! equality. `eq` uses `PartialEq`, which lets the `String` field match a string literal. A field
//! can also hold a nested `partial!`, or [`satisfying`] to run ordinary assertion methods:
//!
//! ```rust
//! # #[cfg(feature = "partial")]
//! # {
//! use assertr::{matchers::*, prelude::*};
//!
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//! let user = User { name: "Alice".into(), age: 30 };
//!
//! assert_that!(user).matches(partial!(User {
//!     age: satisfying(|age| {
//!         age.is_greater_or_equal_to(18).is_less_than(65);
//!     }),
//!     ..
//! }));
//! # }
//! ```
//!
//! The `satisfying` closure runs in capture mode and returns `()`, so end its last assertion with a
//! semicolon. All of its assertions must pass, and it must run at least one.
//!
//! `..` skips the remaining fields. Skipped fields need no `PartialEq` or `Debug`, which is why
//! `secret` above needs neither. Without `..`, every field must be listed. `field: _` lists a field
//! without checking it, keeping the pattern exhaustive so a newly added field fails to compile.
//! Private fields follow the usual visibility rules.
//!
//! Tuple structs and enum variants work too. In tuples, `_` skips one field and a final `..` skips
//! the rest. Prefix an enum constructor with `variant` to include the variant name in failure
//! paths. Constructor paths can start with `crate`, `self`, `super`, `Self`, or `::`. To use an
//! unmarked path that starts with a module named `variant`, write `r#variant::Type`:
//!
//! ```rust
//! # #[cfg(feature = "partial")]
//! # {
//! use assertr::{matchers::eq, prelude::*};
//!
//! struct Pair(i32, i32);
//! assert_that!(Pair(1, 2)).matches(partial!(Pair(eq(1), _)));
//! assert_that!(Some(3)).matches(partial!(variant Some(eq(3))));
//! assert_that!(Some(3)).matches(partial!(variant ::core::option::Option::Some(eq(3))));
//! # }
//! ```
//!
//! Field matchers are built once, in source order, when the `partial!` expression is evaluated.
//!
//! ## Collection policies
//!
//! Choose how the elements or entries must match:
//!
//! | Matcher | Passes when |
//! |---|---|
//! | [`each(matcher)`](each) | Every element matches. An empty collection passes. |
//! | [`elements_are!`](crate::elements_are) | The elements match these matchers one to one, in order. Requires [`StableOrder`](crate::assertions::StableOrder). |
//! | [`elements_are_in_any_order!`](crate::elements_are_in_any_order) | The elements match these matchers one to one, in any order. |
//! | [`entries_are!`](crate::entries_are) | The map has exactly these keys, and each value matches its matcher. |
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
//! In unordered matching, each matcher claims a different element, even when several matchers would
//! accept the same one. These policies also work as fields in `partial!`, and their entries can
//! contain `partial!`. B-tree collections work with `alloc`. Hash collections need `std`.
//!
//! Map keys are looked up, not matched, so they are plain values. Like other expected values, they
//! can be borrowed forms of the stored key. A `Vec<u8>` key can be looked up with a slice:
//!
//! ```
//! use assertr::{matchers::{entry, eq}, prelude::*};
//! use std::collections::BTreeMap;
//!
//! let query = &[1_u8, 2][..];
//! assert_that!(BTreeMap::from([(vec![1_u8, 2], 3)]))
//!     .matches(entries_are![(query, eq(3))])
//!     .matches(entry(query, eq(3)));
//! ```
//!
//! ## Behavior
//!
//! Matchers need no optional feature and work in `no_std` with `alloc`, except `partial!` and the
//! integration matchers, which need the same features as their assertion methods.
//!
//! Equality and ordering accept the expected value owned or borrowed, so `eq("hello")` matches both
//! a `String` and a `&str`. A subject that is a reference keeps its type. Use [`dereferenced`] to
//! match what a reference, `Box`, `String`, or other `Deref` type points to.
//!
//! Matching borrows the subject. A rendering budget shortens the report but never changes the
//! result. Matchers can have side effects, since predicates, callbacks, and lock checks run user
//! code or touch state. A composite matcher may run a child matcher against several elements. A
//! failure report is built from what the check observed, without running it again. Panics in user
//! code are not caught.
//!
//! Assertions that call a closure, drain an iterator, or await a response body consume the subject.
//! `matches` only borrows, so it cannot do that. Use the corresponding assertion method instead.
//!
//! ## Custom matchers
//!
//! [`predicate`] turns a boolean closure into a matcher, as in `predicate(|n: &i32| n % 2 == 0)`.
//! [`pattern!`](crate::pattern) matches a Rust pattern. [`satisfying`] runs assertion methods,
//! including your own, and explains the type annotations its closure needs. [`field`] applies a
//! matcher to one field.
//!
//! For a check with its own failure report, implement
//! [`Expectation`](crate::expectation::Expectation). The [custom assertions
//! guide](crate#custom-assertions) walks through all options, and the
//! [`expectation`](crate::expectation) module describes the contract.
//!
//! Failures name the field, position, or key that failed in
//! [`AssertionFailure::path`](crate::failure::AssertionFailure::path). Use
//! [`capture`](crate::AssertThat::capture) to inspect them, and see the [rendering
//! guide](crate::renderer) to change how values are shown.

// Primitive, variant, length, formatting, and identity expectations are available directly.
#[doc(inline)]
pub use crate::assertions::{
    alloc::boxed::{HasPanicMessage, IsOfType},
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
pub use crate::{
    assertions::{
        collection::{
            ContainsMatching, DoesNotContainMatching, Each, ElementsAre, ElementsAreInAnyOrder,
            contains_contiguous_elements, contains_matching, does_not_contain_matching, each,
            elements_are, elements_are_in_any_order, ends_with_elements, starts_with_elements,
        },
        core::{
            partial_eq::{EqualTo, IsOneOf, NotEqualTo, eq, one_of},
            partial_ord::{GreaterOrEqual, GreaterThan, LessOrEqual, LessThan, ge, gt, le, lt},
        },
        distance::{IsCloseTo, close_to},
        map::{EntriesAre, Entry, entries_are, entry},
    },
    expectation::{
        all_of::{AllOf, all_of},
        any_of::{AnyOf, any_of},
        anything::{Anything, anything},
        dereferenced::{Dereferenced, dereferenced},
        field::{Field, field},
        predicate::{Predicate, predicate},
        satisfying::{Satisfying, satisfying},
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
        EqualToIgnoringAsciiCase, IsAsciiBlank, IsBlank, IsNotBlank, StartsWith,
    };
}

/// Element membership, ordering, identity, and projection expectations.
pub mod collection {
    #[doc(inline)]
    pub use crate::assertions::collection::{
        Contains, ContainsAll, ContainsContiguous, ContainsExactly, ContainsExactlyInAnyOrder,
        ContainsExactlySameInstances, ContainsExactlySameInstancesInAnyOrder,
        ContainsSameInstanceAs, DoesNotContain, DoesNotContainSameInstanceAs, EndsWith,
        HasElementAt, HasFirst, HasLast, HasSingle, StartsWith,
    };
}

/// Key, value, entry, and exact map expectations.
pub mod map {
    #[doc(inline)]
    pub use crate::assertions::map::{
        ContainsEntry, ContainsEntryMatching, ContainsExactlyEntries, ContainsKey, ContainsKeys,
        ContainsValue, ContainsValueMatching, DoesNotContainEntry, DoesNotContainKey,
        DoesNotContainValue,
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
        IsBorrowed, IsMutablyBorrowed, IsNotBorrowed, IsNotMutablyBorrowed,
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
    /// Floating-point classifications also require `std` or `libm`.
    #[cfg(any(feature = "std", feature = "libm"))]
    #[doc(inline)]
    pub use crate::assertions::num::{IsFinite, IsInfinite, IsNan, IsNormal, IsSubnormal};
    #[doc(inline)]
    pub use crate::assertions::num::{IsNegative, IsOne, IsPositive, IsZero};
}

/// Command argument expectations. Requires `std`.
#[cfg(feature = "std")]
pub mod command {
    #[doc(inline)]
    pub use crate::assertions::std::command::HasArg;
}

/// Drop-requirement expectations.
pub mod memory {
    #[doc(inline)]
    pub use crate::assertions::core::mem::NeedsDrop;
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
        IsADirectory, IsAFile, IsASymlink, IsAbsolute, IsRelative, StartsWith,
    };
}

/// ASCII and sensitivity header-value expectations. Requires `http`.
#[cfg(feature = "http")]
pub mod header_value {
    #[doc(inline)]
    pub use crate::assertions::http::header_value::{IsAscii, IsNotSensitive, IsSensitive};
}

/// Signed-duration sign and zero expectations. Requires `jiff`.
///
/// For tolerances, use the root [`close_to`] matcher, which accepts
/// `jiff::SignedDuration`.
#[cfg(feature = "jiff")]
pub mod signed_duration {
    #[doc(inline)]
    pub use crate::assertions::jiff::signed_duration::{IsNegative, IsPositive, IsZero};
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

/// Response status and header expectations, with the reasons header expectations reject.
/// Requires `reqwest`.
#[cfg(feature = "reqwest")]
pub mod response {
    #[doc(inline)]
    pub use crate::assertions::reqwest::response::{
        DoesNotHaveHeader, HasHeader, HasHeaderValue, HasStatusCode, HeaderRejection,
        IsClientError, IsInformational, IsRedirection, IsServerError, IsSuccess,
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

#[cfg(test)]
mod tests {
    mod common_traits {
        use alloc::{format, string::String};
        use core::fmt::Debug;

        use crate::{matchers::*, prelude::*};

        fn debug_and_clone<T: Debug + Clone>(value: &T) -> String {
            format!("{:?}", value.clone())
        }

        fn copy<T: Copy>(_: &T) {}

        fn some_pattern() -> Pattern<impl Fn(&Option<i32>) -> bool + Clone> {
            pattern!(Some(_))
        }

        #[test]
        fn data_free_and_copyable_expectations_are_copy() {
            copy(&IsSome);
            copy(&HasLengthOf::new(3));
            copy(&IsTrue);
            copy(&string::IsBlank);
            copy(&IsOfType::<String>::new());
            copy(&IsSameInstanceAs::new("shared"));
            copy(&map::ContainsKey::new("key"));
            copy(&anything());
        }

        #[test]
        fn value_expectations_derive_debug_and_clone() {
            assert_that!(debug_and_clone(&IsSome)).is_equal_to("IsSome");
            assert_that!(debug_and_clone(&IsTrue)).is_equal_to("IsTrue");
            assert_that!(debug_and_clone(&string::IsBlank)).is_equal_to("IsBlank");
            assert_that!(debug_and_clone(&string::Contains::new("needle")))
                .is_equal_to(r#"Contains("needle")"#);
            assert_that!(debug_and_clone(&ge(18))).is_equal_to("GreaterOrEqual { expected: 18 }");
            assert_that!(debug_and_clone(&HasLengthOf::new(3))).is_equal_to("HasLengthOf(3)");
            assert_that!(debug_and_clone(&collection::ContainsExactly::new([1, 2])))
                .is_equal_to("ContainsExactly { expected: [1, 2] }");
            assert_that!(debug_and_clone(&range::ContainsElement::new(2)))
                .is_equal_to("ContainsElement(2)");
            assert_that!(debug_and_clone(&IsOfType::<String>::new()))
                .is_equal_to("IsOfType<alloc::string::String>");
        }

        #[test]
        fn compositions_derive_debug_and_clone_from_their_parts() {
            assert_that!(debug_and_clone(&all_of(matchers![eq(1), ge(0)])))
                .is_equal_to("AllOf([EqualTo(1), GreaterOrEqual { expected: 0 }])");
            assert_that!(debug_and_clone(&any_of([eq(1), eq(2)])))
                .is_equal_to("AnyOf([EqualTo(1), EqualTo(2)])");
            assert_that!(debug_and_clone(&each(IsSome))).is_equal_to("Each(IsSome)");
            assert_that!(debug_and_clone(&dereferenced(eq(1))))
                .is_equal_to("Dereferenced(EqualTo(1))");
            assert_that!(debug_and_clone(&elements_are([eq(1)])))
                .is_equal_to("ElementsAre { list: [EqualTo(1)], placement: Exact }");
            assert_that!(debug_and_clone(&entries_are([entry("a", eq(1))])))
                .is_equal_to(r#"EntriesAre([Entry { key: "a", matcher: EqualTo(1) }])"#);
            assert_that!(debug_and_clone(&field(
                "x",
                |point: &(i32,)| &point.0,
                eq(1)
            )))
            .is_equal_to(r#"Field { path: Field("x"), matcher: EqualTo(1), .. }"#);
        }

        #[test]
        fn callback_expectations_clone_with_their_callbacks_and_omit_them_from_debug() {
            let positive = predicate(|value: &i32| *value > 0)
                .described_as("is positive")
                .rejected_as("is not positive");
            assert_that!(debug_and_clone(&positive)).is_equal_to(
                r#"Predicate { description: "is positive", rejection: Some("is not positive"), .. }"#,
            );
            assert_that!(1).matches(positive.clone());

            let callback = satisfying(|it: AssertThat<'_, i32, Capture>| {
                it.is_equal_to(1);
            });
            assert_that!(debug_and_clone(&callback)).is_equal_to("Satisfying { .. }");

            let some = some_pattern();
            assert_that!(Some(1)).matches(some.clone());
            assert_that!(debug_and_clone(&some))
                .is_equal_to(r#"Pattern { pattern: "Some(_)", .. }"#);
            assert_that!(debug_and_clone(&DoesNotMatchPattern::new(some)))
                .is_equal_to(r#"DoesNotMatchPattern(Pattern { pattern: "Some(_)", .. })"#);
        }

        #[test]
        fn identity_expectations_show_addresses_without_rendering_targets() {
            struct Opaque;
            let target = Opaque;
            let expected = format!("IsSameInstanceAs({:?})", core::ptr::from_ref(&target));
            assert_that!(debug_and_clone(&IsSameInstanceAs::new(&target))).is_equal_to(expected);
        }
    }
}
