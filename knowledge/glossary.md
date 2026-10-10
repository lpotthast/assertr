---
id: glossary
depends_on: [ ]
sources:
  - knowledge/assertion-lifecycle.md
  - knowledge/expectation-execution.md
  - knowledge/comparison-operands.md
  - knowledge/iterator-execution.md
  - knowledge/failure-processing.md
  - knowledge/diagnostic-rendering.md
  - knowledge/collection-semantics.md
  - knowledge/matcher-composition.md
  - knowledge/observation-boundaries.md
  - knowledge/fluent-entry.md
  - knowledge/reference-identity.md
  - knowledge/extension-contract.md
---

# Glossary

[Architecture overview](README.md)

Use **assertion chain** for `AssertThat` in prose. Use exact Rust names for API items and lowercase for concepts.
Distinguish `AssertionContext` from `RenderingContext`, and diagnostic evidence from the owned type `Evidence`.
Execution adapter means an assertion that performs an operation itself. Private types are marked below. Follow the links
for detailed requirements and exceptions.

## Chain state

| Term | Meaning |
|---|---|
| [AssertThat](assertion-lifecycle.md#chain-representation) | The type representing an assertion chain. |
| [Subject](assertion-lifecycle.md#chain-representation) | The value currently being checked. |
| [Actual](assertion-lifecycle.md#chain-representation) | Borrowed or owned subject storage. |
| [Mode](assertion-lifecycle.md#chain-representation) | Compile-time failure-handling choice. |
| [Panic](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Mode presenting an assertion failure as a panic. |
| [Capture](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Mode that collects failures so checks can continue. |
| [ChainState](assertion-lifecycle.md#chain-representation) | Private state that can move to a chain with a different subject type. |
| [DiagnosticSettings](assertion-lifecycle.md#chain-representation) | Private subject name, expression, location policy, rendering budget, and panic presentation of a chain. |
| [ChainRecords](assertion-lifecycle.md#chain-representation) | Private storage for messages, assertion counts, failures, and an optional link to parent records. |
| [Root chain](assertion-lifecycle.md#chain-representation) | Chain with no parent-record link. |
| [Child chain](assertion-lifecycle.md#chain-representation) | Derived chain linked to ancestor records. |
| [Detached chain](observation-boundaries.md#eventual-observations) | Private `DetachedChain`, produced by `AssertThat::into_parts`: a panic-mode chain's diagnostic settings and collected messages without its subject and records, kept across awaits and attached again as a root. |
| [Unwind safety](assertion-lifecycle.md#unwind-safety) | `UnwindSafe` and `RefUnwindSafe` requirements when catching panics. |

## Entry and continuation

| Term | Meaning |
|---|---|
| [Borrowed entry](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Start a chain that borrows the asserted value. |
| [Owned entry](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Start with ownership of the input. |
| [Reference normalization](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Entry rule selecting the subject type from a value or reference. |
| [Mapping](assertion-lifecycle.md#projections-and-continuation) | Replace the subject and move the existing state and renderer to the new chain. |
| [Derivation](assertion-lifecycle.md#projections-and-continuation) | Create a child while retaining the parent. |
| [Projection](assertion-lifecycle.md#projections-and-continuation) | Select a borrowed part or computed view for a child. |
| [Extraction](assertion-lifecycle.md#continuation-availability) | Check and continue on a selected value. |
| [Fluent alias](fluent-entry.md#borrowing-and-ownership) | Generated spelling preserving method behavior and bounds. |
| [Expression capture](fluent-entry.md#scoped-expression-capture) | Record the asserted expression's source text through macros or fluent rewriting. |

## Evaluation and composition

| Term | Meaning |
|---|---|
| [Assertion family](extension-contract.md#choosing-an-extension) | Assertion methods and reusable definitions grouped by capability or topic. |
| [Leaf assertion](expectation-execution.md#evaluation-and-explanation) | A check that determines its own result and diagnostics. |
| [Expectation](expectation-execution.md#evaluation-and-explanation) | A reusable check describing what a subject should satisfy, with its failure kind and diagnostics. |
| [Expectation::Success](expectation-execution.md#evaluation-and-explanation) | Successful observation available for continuation, such as payload or guard. |
| [Expectation::Rejection](expectation-execution.md#evaluation-and-explanation) | Original failed observation retained for explanation. |
| [Expectation::explain](expectation-execution.md#evaluation-and-explanation) | Explains a rejection or an expectation with no subject. |
| [AssertionContext](expectation-execution.md#child-scopes-and-evidence) | Context supplied by the executor with rendering settings, location policy, paths, and child evidence. |
| [Matcher](matcher-composition.md) | Expectation used in composition. |
| [MatcherList](matcher-composition.md#matcher-lists) | A list whose expectations can be evaluated or described individually. |
| [EntryMatcherList](matcher-composition.md#matcher-lists) | A list of keyed matchers that identifies which stored map entries were visited. |
| [Predicate](matcher-composition.md#evaluation-and-failure-evidence) | An expectation implemented by a function returning `bool`. |
| [Assertion callback](matcher-composition.md#assertion-callbacks) | Closure receiving a chain and performing checks. `satisfying` adapts reusable callbacks. |
| [Evidence](expectation-execution.md#child-scopes-and-evidence) | Diagnostic information. Type `Evidence` holds owned child failures and omissions. |
| [Probe](expectation-execution.md#budgets-and-probes) | Evaluation that does not retain built-in diagnostics. |
| [Expected data](comparison-operands.md#reading-expected-data-again) | A bulk list of operands that must give the same list and comparison values on every access. |
| [BorrowFor](comparison-operands.md#borrowed-views) | Selects the view borrowed from an operand for a given subject, item, key, value, or bound type. |
| [Exact unordered assignment](matcher-composition.md#exact-unordered-assignment) | Maximum one-to-one pairing of actual occurrences and expected slots. |
| [Multiplicity](matcher-composition.md#exact-unordered-assignment) | Occurrence count. |

## Subject capabilities

| Term | Meaning |
|---|---|
| [Capability](collection-semantics.md#capability-model) | A trait providing the behavior needed by a group of assertions. |
| [HasLength](collection-semantics.md#capability-model) | Provides the subject's finite length. |
| [Collection](collection-semantics.md#capability-model) | Allows repeated traversal of references to the same elements in the same order. |
| [StableOrder](collection-semantics.md#capability-model) | Makes element positions part of a collection's meaning, as in a list. |
| [RandomAccess](collection-semantics.md#capability-model) | Stable positions with constant-time indexed access. |
| [SetLookup](collection-semantics.md#capability-model) | Native membership for unique elements, using the same equivalence as the set. |
| [Map](collection-semantics.md#capability-model) | Allows repeated traversal of the same stored keys and values. |
| [MapLookup](collection-semantics.md#keyed-maps) | Native borrowed-key lookup capability. |
| [Map views](collection-semantics.md#key-and-value-views) | `MapKeys` and `MapValues`: order-free collection views of a map's keys or values, created by projection. |
| [Reference identity](reference-identity.md#which-address-is-compared) | Pointer equality of the subject or selected `Borrow` target, including pointer metadata. |

## Diagnostic values

| Term | Meaning |
|---|---|
| [ValueRenderer](diagnostic-rendering.md#capabilities-and-structure) | Formats one diagnostic leaf type. |
| [Diagnostic leaf](diagnostic-rendering.md#capabilities-and-structure) | Value formatted as one unit, possibly an entire opaque subject. |
| [RenderingContext](diagnostic-rendering.md#structural-rendering) | Active renderer and budget, rendering leaves and structures into `Rendered` trees. |
| [Rendered](diagnostic-rendering.md#capabilities-and-structure) | Owned diagnostic tree: text, structure, type metadata, layout, omissions. |
| [RenderingBudget](diagnostic-rendering.md#bounded-retention) | Limits how many items each diagnostic group retains and how many characters each leaf contains. |
| [CollectionPresentation](diagnostic-rendering.md#capabilities-and-structure) | Collection diagnostic syntax, type hints, and ordering. |
| [RenderingOrder](diagnostic-rendering.md#capabilities-and-structure) | Chooses whether diagnostics preserve traversal order or sort by rendered text. |

## Failures

| Term | Meaning |
|---|---|
| [AssertionFailure](failure-processing.md#structured-construction-and-ownership) | Owned diagnostic node: operands, relation, facts, children, paths, metadata. |
| [AssertionFailures](failure-processing.md#structured-construction-and-ownership) | Ordered aggregate returned by capture/verification. |
| [FailureKind](failure-processing.md#structured-construction-and-ownership) | Non-exhaustive failure family, not a specific assertion. |
| [FailureBuilder](failure-processing.md#builder-completion) | Builds a structured failure. The executor or `AssertThat::raise` adds chain metadata and handles it according to the mode. |
| [Relation](failure-processing.md#structured-construction-and-ownership) | Lowercase diagnostic sentence without values or trailing period. |
| [Fact](failure-processing.md#structured-construction-and-ownership) | Optional label plus `Rendered` evidence. |
| [Note](failure-processing.md#structured-construction-and-ownership) | `Fact` without a label, shown as unlabelled detail. |
| [Path](failure-processing.md#paths) | A sequence of `PathSegment` values locating nested evidence relative to its parent subject. |

## Execution and presentation

| Term | Meaning |
|---|---|
| [Execution adapter](observation-boundaries.md) | Performs invocation, polling, traversal, or consumption and passes the observation to assertion execution. |
| [Panic presentation](failure-processing.md#presentation-and-fallback) | Closure producing panic text from a failure. The private `PanicPresentation` type erases it. |
| [Observation](observation-boundaries.md#eventual-observations) | Closure returning a future of a changing value, the subject of an eventual assertion. |
| [Eventual assertion](observation-boundaries.md#eventual-observations) | `eventually` or `consistently`: observes repeatedly until an expectation holds, or while it keeps holding. |
| [Give-up policy](observation-boundaries.md#eventual-observations) | `GiveUp` implementation deciding which failed observations end `eventually_ok`: `KeepRetrying`, `AnyError`, or a closure. |
| [Patience](observation-boundaries.md#eventual-observations) | Timeout, polling interval, and consistency duration of eventual assertions: global, overridden per chain. |
