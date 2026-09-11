//! Partial matching for selected fields and nested values.
//!
//! Use `partial!` to describe the parts of a struct or enum that matter to a test. Each selected
//! field can have an expected value, a constraint, or another partial match. A matcher is the value
//! that packages such an expectation. Pass it to
//! [`matches`](crate::assertions::matcher::MatcherAssertions::matches) to check a subject.
//!
//! Matchers primarily support these field expectations. The built-in matchers are a
//! selective set of useful constraints, not a counterpart for every assertion method. When a
//! field needs another check, use [`satisfying`] to bring existing assertion methods into the
//! expectation. Ordinary [assertion methods](crate::assertions) remain the starting point for
//! direct checks on a value, and equality assertions continue to use `PartialEq`.
//!
//! Enable the **`matchers` feature** for `partial!`. Runtime matchers and the collection and map
//! macros work without optional features, including in `no_std` builds with `alloc`.
//!
//! ## Structural syntax
//!
//! Add `features = ["matchers"]` to your `assertr` dependency to run the `partial!` examples.
//! Production types need no derives or attributes, and their equality implementation is unchanged.
//! Without `..`, the compiler checks that every field is listed. With `..`, only selected fields
//! need comparison or rendering support. Private fields follow ordinary Rust visibility rules.
//!
//! ```rust
//! # #[cfg(feature = "matchers")]
//! # {
//! use assertr::prelude::*;
//! use assertr::matchers::{ge, satisfying};
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
//! let adult_alice = partial!(User {
//!     name: "Alice",
//!     age: ge(18),
//!     roles: satisfying(|roles| {
//!         roles.has_length(2);
//!     }),
//!     ..
//! });
//! assert_that!(user).matches(&adult_alice);
//! assert_that!([user]).contains_matching(&adult_alice);
//! # }
//! ```
//!
//! This expectation uses three forms together:
//!
//! - `name: "Alice"` compares the field with a plain expected value using `PartialEq`.
//! - `age: ge(18)` applies a built-in lower-bound matcher.
//! - `roles: satisfying(...)` checks the field with an ordinary length assertion.
//!
//! `..` leaves `secret` out of the comparison and diagnostics. Neither `User` nor `Secret` needs
//! `PartialEq` or `Debug`. Passing `&adult_alice` reuses the same expectation for a subject or for
//! an element in a collection.
//!
//! ### Choosing a field expectation
//!
//! Start with a plain value for equality. Use a built-in matcher such as [`ge`] or [`starts_with`]
//! when it expresses the constraint you need. Use [`satisfying`] for a chain of existing assertion
//! methods, including custom assertion methods. It captures their structured failures and the
//! enclosing partial match associates them with the selected field.
//!
//! A `satisfying` closure receives an assertion context for the field and returns `()`, so end
//! its final assertion with a semicolon. It runs in capture mode and matches only if all its
//! assertions pass. It must perform at least one assertion. Empty closures panic as misuse, and
//! user panics propagate. The callback must be reusable (`Fn`), and its renderer must be `Clone`.
//! See [`satisfying`] for a complete example.
//!
//! For a boolean check, use [`predicate`]. For nested structs or enums, use another `partial!`.
//! For collection or map fields, choose a [collection policy](#collection-policies) below.
//!
//! Field expectations, tuple expectations, and list entries accept arbitrary Rust expressions. Each
//! expected expression is evaluated exactly once in source order when the matcher is built. Bare
//! values use ordinary heterogeneous equality, so a `String` field can match a string literal. An
//! expression implementing the matcher protocol is used as a matcher. If both interpretations
//! apply, inference reports an ambiguity. Write [`equal_to(value)`](equal_to) or
//! [`as_matcher(matcher)`](as_matcher) explicitly. Generic helpers should use these explicit
//! adapters when their bounds leave interpretation open. Outside the macros, `.matches(42)` is
//! rejected. Write `.matches(equal_to(42))`.
//!
//! Named structs, tuple structs, named and tuple enum variants, unit structs, and unit variants are
//! supported. Tuple `_` positions are wildcards. A tuple `..` must be final, so every selected
//! field has a known tuple index. An optional `variant` prefix records a typed variant path
//! segment. This explicit annotation avoids guessing whether a qualified constructor names an enum
//! variant, a struct, or a type alias. Without it, the constructor remains in the constraint
//! description and selected fields retain their ordinary relative paths.
//!
//! ```rust
//! # #[cfg(feature = "matchers")]
//! # {
//! use assertr::prelude::*;
//! struct Pair(i32, i32);
//! assert_that!(Pair(1, 2)).matches(partial!(Pair(1, _)));
//! assert_that!(Some(3)).matches(partial!(variant Some(3)));
//! assert_that!(Ok::<_, ()>(3)).matches(partial!(Ok(3)));
//! # }
//! ```
//!
//! ## Collection policies
//!
//! Collection and map matchers let `partial!` describe a field's contents. Choose whether
//! positions, duplicate counts, or keys define the expectation:
//!
//! | Matcher | What it checks | Required capability |
//! |---|---|---|
//! | [`each(matcher)`](each) | Every element matches. Empty collections pass. | [`Collection`](crate::assertions::collection::Collection) |
//! | [`elements_are!`](crate::elements_are) | Exactly these elements at these positions. | [`StableOrder`](crate::assertions::collection::StableOrder) |
//! | [`elements_are_in_any_order!`](crate::elements_are_in_any_order) | Exactly these elements, pairing each expectation with a distinct element. | [`Collection`](crate::assertions::collection::Collection) |
//! | [`entries_are!`](crate::entries_are) | Exactly these keys, each with a matching value. | [`MapLookup`](crate::assertions::map::MapLookup) through [`MapKeyQuery`](crate::assertions::map::MapKeyQuery) |
//!
//! Nest these matchers inside `partial!` to describe only the fields that matter:
//!
//! ```rust
//! # #[cfg(feature = "matchers")]
//! # {
//! use assertr::prelude::*;
//! struct Secret;
//! struct Child {
//!     id: u32,
//!     secret: Secret,
//! }
//! struct Parent {
//!     children: Vec<Child>,
//!     secret: Secret,
//! }
//! let parent = Parent {
//!     children: vec![
//!         Child { id: 1, secret: Secret },
//!         Child { id: 2, secret: Secret },
//!     ],
//!     secret: Secret,
//! };
//! assert_that!(parent).matches(partial!(Parent {
//!     children: elements_are![partial!(Child { id: 1, .. }), partial!(Child { id: 2, .. })],
//!     ..
//! }));
//! # }
//! ```
//!
//! Unordered matching preserves duplicate counts and finds a one-to-one assignment even when
//! constraints overlap. Duplicate expected map keys cannot substitute for a missing distinct
//! entry. Only keys and selected values need rendering. B-tree collections work with `alloc`.
//! Hash collections require `std`.
//!
//! These policies also work directly on collection and map subjects:
//!
//! ```rust
//! use assertr::prelude::*;
//! use assertr::matchers::{each, ge};
//! use std::collections::BTreeMap;
//!
//! assert_that!([18, 30]).matches(each(ge(18)));
//! assert_that!([18, 30]).matches(elements_are![18, ge(20)]);
//! assert_that!([30, 18]).matches(elements_are_in_any_order![18, ge(20)]);
//! assert_that!(BTreeMap::from([("Ada", 36), ("Grace", 85)]))
//!     .matches(entries_are![("Ada", ge(18)), ("Grace", 85)]);
//! ```
//!
//! ## Reuse and compose constraints
//!
//! The same matchers can also be applied outside `partial!`. This is useful when an expectation
//! is shared by several assertions or combined with other expectations. Matching borrows the
//! actual subject and never clones it. Pass `&matcher` to reuse an expectation. A matcher owns
//! its expectations unless you provide references. Neither the protocol nor structural matching
//! requires `Clone`, `Send`, `Sync`, or `'static`.
//!
//! [`all_of`], [`any_of`], and [`not`] combine constraints, including field expectations inside
//! `partial!`. [`equal_to`] makes equality explicit when composing matchers outside macro syntax:
//!
//! ```rust
//! use assertr::prelude::*;
//! use assertr::matchers::{all_of, equal_to, ge, not};
//!
//! let allowed = all_of((ge(18), not(equal_to(100))));
//! assert_that!(42).matches(&allowed);
//! assert_that!([10, 42]).contains_matching(&allowed);
//! assert_that!(100).does_not_match(&allowed);
//! ```
//!
//! [`all_of`] evaluates every branch and an empty conjunction succeeds. [`any_of`] stops at the
//! first success and an empty disjunction fails. [`not`] reverses both truth and diagnostic
//! polarity. Descriptions exist independently of failures, including when explaining an unexpected
//! success. Predicates run once per evaluation.
//!
//! [`matchers!`](crate::matchers!) constructs a heterogeneous list without boxes or trait objects.
//! Homogeneous arrays, slices, and vectors of matchers also work. Tuple composition supports up to
//! twelve members. Use `matchers!` for larger combinations. [`predicate_list`] adapts homogeneous
//! predicate iterables, and [`entry_matchers`] adapts homogeneous key/matcher pairs. The
//! `*_matching` assertions in the [collection](crate::assertions::collection) and
//! [map](crate::assertions::map) families use these matchers and lists. Their `*_satisfying`
//! counterparts accept assertion closures directly.
//!
//! [Direct iterator assertions](crate::assertions::core::iter::IteratorAssertions) are terminal and
//! single-pass. Membership, prefixes, and contiguous searches stop as soon as they succeed. Exact
//! checks consume at most `expected.len() + 1` items.
//! Suffix checks require exhaustion. Retained diagnostic evidence is bounded to recent elements and
//! rendering limits. Borrowed `into_iter_*` membership remains order-free and retains its subject.
//!
//! ## Projections and renderers
//!
//! Use [`AssertThat::derive`](crate::AssertThat::derive) to project a field and chain assertions on
//! it. The parent remains usable for other field checks.
//! [`AssertThat::satisfies`](crate::AssertThat::satisfies) checks a projection in a closure and
//! returns the original chain. Use `partial!` to describe several selected fields together or
//! package them as an expectation that can be reused or nested in other matchers. The free function
//! [`satisfying`] adapts assertion methods into a matcher for use within such an expectation.
//!
//! A matcher generated by `partial!` stays generic over renderers supported by its selected
//! matchers. A [`satisfying`] closure can select a concrete renderer through its
//! [`AssertThat`](crate::AssertThat) parameter type. Construct that adapter inside a generic helper
//! when the helper needs to support different renderers. Reference-valued fields and owned
//! reference subjects may require [`dereferenced(matcher)`](dereferenced). Unsized leaves are
//! supported by the matcher protocol. Assertion closures follow the sized-subject contract of
//! `AssertThat`. See the [rendering guide](crate::renderer) for configuring diagnostic values.
//!
//! ## Downstream matchers and diagnostics
//!
//! Implement [`AssertrMatcher<Actual, R>`](AssertrMatcher) on an expected-side type.
//! [`MatchResult`] carries truth explicitly. Record failures through a detached
//! [`FailureBuilder`](crate::failure::FailureBuilder) and [`MatchContext::record`], or use
//! [`MatchContext::outcome`] for constraints needing only a description. Evaluation never tracks
//! or raises a root assertion. The enclosing [matcher assertion](crate::assertions::matcher)
//! handles tracking and raising in panic or capture mode.
//!
//! ```rust
//! use assertr::prelude::*;
//! use assertr::matchers::{ConstraintDescription, MatchContext, MatchResult};
//! struct Even;
//! impl<R> AssertrMatcher<i32, R> for Even {
//!     fn describe(&self, _: &MatchContext<'_, R>) -> ConstraintDescription {
//!         ConstraintDescription::new("satisfies the even-number constraint")
//!     }
//!     fn evaluate(&self, value: &i32, context: &mut MatchContext<'_, R>) -> MatchResult {
//!         context.outcome(value % 2 == 0, |context| self.describe(context))
//!     }
//! }
//! assert_that!(2).matches(Even);
//! ```
//!
//! Custom renderers render leaves. Assertr renders structural syntax. Build values with
//! `context.render().value(...)`, and put `ValueRenderer` bounds only on leaf implementations that
//! need them. A structural assertion omits the whole actual object, including under negation.
//! [`AssertionFailure::path`](crate::AssertionFailure::path) carries relative fields, tuple
//! positions, variants, stable collection indexes, and rendered map keys.
//! [`omitted_children`](crate::AssertionFailure::omitted_children) retains truncation counts as
//! data. [Failure adapters](crate::failure::adapter) inspect the same tree that panic presentation
//! uses. Start with [`AssertThat::capture`](crate::AssertThat::capture) to inspect it in a test.
//!
//! [`MatchContext::fork`] borrows its parent exclusively. Dropping the fork discards evidence, and
//! consuming [`MatchFork::commit`] attaches it once to that parent. [`MatchContext::scoped`]
//! restores its parent's path even after a failed child. [`MatchContext::probe`] evaluates truth
//! in isolation and suppresses built-in leaf rendering. It cannot undo side effects or suppress
//! arbitrary downstream rendering or assertion-closure
//! rendering. Unordered matching evaluates each actual/expected pair at most once and caches its
//! evidence. When positive mismatch diagnostics can retain evidence, it completes previously
//! unvisited comparisons involving unmatched elements or expectations. Surplus occurrences that
//! satisfy occupied expectations are explained through their constraint descriptions, without
//! replaying evaluations or requiring an element renderer. Candidate evidence can require quadratic
//! space. Rendering limits bound retained output, not comparison work. Probes and zero-item budgets
//! skip diagnostic completion without changing truth.
//!
//! [Conditions](crate::condition) retain their error-typed authoring trait. Their assertions
//! require rendering support for the condition error, but no subject renderer. Wrap them in
//! [`condition(c)`](condition) to compose them.
//! [`pattern!`](crate::pattern) constructs reusable matchers with `Fn` guards. Direct
//! `is_matching(pattern! (...))` methods continue to support consuming `FnOnce` guards.
//!
//! ## Migration
//!
//! | Previous usage                                             | Replacement                                                                                            |
//! |------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
//! | `#[derive(AssertrEq)]` and companion structs               | Remove the derive and construct `partial!(Type { ... })`.                                              |
//! | `TypeAssertrEq { field: eq(value), ..Default::default() }` | `partial!(Type { field: value, .. })`.                                                                 |
//! | `Eq::Any`, `any()`                                         | Omit named fields with `..`, or use `anything()`.                                                      |
//! | Partial expectations passed to `is_equal_to`               | Use `matches` or `does_not_match`. Equality now uses ordinary `PartialEq`.                             |
//! | `map_type`, `compare_with`, `compare_bounds`               | Select field and container matchers at the expectation site. Put generic bounds on reusable functions. |
//! | Slice or map comparison helpers                            | Use `elements_are!`, `elements_are_in_any_order!`, `each`, or `entries_are!`.                          |
//! | `AssertrPartialEq`, `EqContext`, `Differences`             | Implement expected-side `AssertrMatcher` and record detached structured failures.                      |
//! | `contains_matching(closure)`                               | `contains_matching(predicate(closure))`.                                                               |
//! | Homogeneous predicate arrays                               | Arrays of `predicate(...)`, or `predicate_list(array)`.                                                |
//! | Key/predicate iterables                                    | `entry_matchers(iter.map((k, p) \| (k, predicate(p))))`, or `entries_are!`.                            |
//! | `features = ["derive"]`                                    | `features = ["matchers"]`. Runtime matchers require no feature.                                        |
//!
//! `AssertThat::derive` and its async projection counterparts are unchanged. They are unrelated to
//! the removed equality derive macro.

mod all_of;
mod any_of;
mod anything;
mod as_matcher;
mod condition;
mod contains_matching;
mod context;
mod dereferenced;
mod each;
mod elements_are;
mod elements_are_in_any_order;
mod entries_are;
mod entry;
mod entry_matcher_list;
mod equal_to;
pub(crate) mod field;
mod greater_or_equal;
pub(crate) mod lists;
pub(crate) mod normalize;
mod not;
pub(crate) mod partial_match;
mod predicate;
mod satisfying;
mod starts_with;

#[cfg(test)]
mod test_support;

use crate::DebugRenderer;
use core::any::type_name;

pub use all_of::{AllOf, all_of};
pub use any_of::{AnyOf, any_of};
pub use anything::{Anything, anything};
pub use as_matcher::{AsMatcher, as_matcher};
pub use condition::{Condition, condition};
pub use contains_matching::{ContainsMatching, contains_matching};
pub use context::{ConstraintDescription, MatchContext, MatchFork, MatchResult, Mismatch};
pub use dereferenced::{Dereferenced, dereferenced};
pub use each::{Each, each};
pub use elements_are::{
    ElementsAre, contains_contiguous_elements, elements_are, ends_with_elements,
    starts_with_elements,
};
pub use elements_are_in_any_order::{ElementsAreInAnyOrder, elements_are_in_any_order};
pub use entries_are::{EntriesAre, entries_are};
pub use entry::{Entry, entry};
pub use entry_matcher_list::{EntryMatcherList, entry_matchers};
pub use equal_to::{EqualTo, equal_to};
pub use greater_or_equal::{GreaterOrEqual, ge};
pub use lists::{MatcherList, MatcherSequence, predicate_list};
pub use not::{Not, not};
pub use predicate::{Predicate, predicate};
pub use satisfying::{Satisfying, satisfying};
pub use starts_with::{StartsWith, starts_with};

pub(crate) use equal_to::equals;
#[cfg(feature = "tokio")]
pub(crate) use satisfying::collect_assertions;

/// A constraint implemented on the expected side. Evaluation records detached evidence and never
/// raises.
#[diagnostic::on_unimplemented(
    message = "this expectation is not a matcher for the actual subject",
    note = "use `equal_to(value)`, `predicate(closure)`, or `satisfying(assertions)` on the expected side"
)]
pub trait AssertrMatcher<Actual: ?Sized, R = DebugRenderer> {
    /// Describes the constraint independently of evaluation.
    fn describe(&self, _context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new(type_name::<Self>())
    }

    /// Evaluates once, returning truth independently of retained evidence.
    fn evaluate(&self, actual: &Actual, context: &mut MatchContext<'_, R>) -> MatchResult;
}

impl<A: ?Sized, R, M> AssertrMatcher<A, R> for &M
where
    M: AssertrMatcher<A, R> + ?Sized,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        (**self).describe(context)
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        (**self).evaluate(actual, context)
    }
}
