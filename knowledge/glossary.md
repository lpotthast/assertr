---
id: glossary
depends_on: [ ]
sources:
  - knowledge/assertion-lifecycle.md
  - knowledge/expectation-execution.md
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

Use **assertion chain** for `AssertThat` in prose. Distinguish `AssertionContext` from `RenderingContext`. Use exact
Rust names for types and traits, and lowercase terms for concepts. In particular, evidence means diagnostic information,
while `Evidence` names the owned child-failure container. Private implementation types are marked below.

Qualify adapter roles in prose. A **failure adapter** implements `Adapter<Input>` to process completed failures or later
conversion stages. A **rendering adapter** builds a `Rendered` value using the active renderer and budget. An
**execution adapter** owns invocation, polling, or consumption around an expectation. These are distinct
responsibilities.

Definitions are brief. Follow each term to its owning contract for guarantees and exceptions. Reuse an existing name
when it fits, and introduce a new term only for a distinct concept.

| Term                                                                               | Meaning                                                                                                                                                   |
|------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| [Actual](assertion-lifecycle.md#chain-representation)                              | Subject storage: `Borrowed(&T)` or `Owned(T)`.                                                                                                            |
| [Adapter](failure-processing.md#presentation-and-fallback)                         | Trait used by failure adapters and subsequent conversion stages, with borrowed input and a declared output or error.                                      |
| [Assertion callback](matcher-composition.md#assertion-callbacks)                   | A closure receiving an assertion chain and performing checks. `satisfying` adapts a reusable callback to an expectation.                                  |
| [Assertion family](extension-contract.md#choosing-an-extension)                    | Methods grouped by subject capability or domain, with reusable definitions beside their owning implementation.                                            |
| [AssertionContext](expectation-execution.md#child-scopes-and-evidence)             | Executor-supplied rendering settings, location policy, paths, and bounded child evidence for expectation evaluation.                                      |
| [AssertionFailure](failure-processing.md#structured-construction-and-ownership)    | Owned diagnostic node containing rendered evidence, relation, nested failures, paths, and metadata.                                                       |
| [AssertionFailures](failure-processing.md#structured-construction-and-ownership)   | Ordered aggregate returned when capture or fluent verification completes.                                                                                 |
| [AssertrCondition](matcher-composition.md#typed-reusable-conditions)               | A reusable domain property whose test returns `Result<(), Error>`.                                                                                        |
| [AssertThat](assertion-lifecycle.md#chain-representation)                          | The typed assertion chain, combining subject storage, mode, renderer, and chain state.                                                                    |
| [Attached](failure-processing.md#structured-construction-and-ownership)            | Builder target whose `raise()` adds chain metadata and delivers the failure through the mode.                                                             |
| [Borrowed entry](assertion-lifecycle.md#entry-subject-ownership-and-mode)          | Starting a chain without taking ownership of the asserted value.                                                                                          |
| [Capability](collection-semantics.md#capability-model)                             | A trait contract enabling behavior, such as traversal or lookup. Renderer capabilities separately permit diagnostic leaf formatting.                      |
| [Capture](assertion-lifecycle.md#entry-subject-ownership-and-mode)                 | Mode that stores assertion failures and permits continuation. It does not catch user panics.                                                              |
| [ChainRecords](assertion-lifecycle.md#chain-representation)                        | Private per-node messages, assertion count, captured failures, and parent-record link.                                                                    |
| [ChainState](assertion-lifecycle.md#chain-representation)                          | Private state transferred intact when a chain's subject type changes.                                                                                     |
| [Child chain](assertion-lifecycle.md#chain-representation)                         | Derived chain with its own subject and records, forwarding counts and captured failures to ancestors.                                                     |
| [Collection](collection-semantics.md#capability-model)                             | Finite, repeatable element traversal by reference. Positional meaning requires `StableOrder`.                                                             |
| [CollectionPresentation](diagnostic-rendering.md#capabilities-and-structure)       | Diagnostic collection syntax, type-hint visibility, and ordering settings. Grants no behavior.                                                            |
| [Derivation](assertion-lifecycle.md#projections-and-continuation)                  | Creating a child chain while retaining its parent, inheriting settings and cloning the renderer.                                                          |
| [Detached](failure-processing.md#structured-construction-and-ownership)            | Builder target whose `build()` returns failure data without raising or collecting chain metadata.                                                         |
| [Diagnostic leaf](diagnostic-rendering.md#capabilities-and-structure)              | A value formatted as one unit. An opaque assertion may treat its whole subject as a leaf.                                                                 |
| [Evidence](expectation-execution.md#child-scopes-and-evidence)                     | Diagnostic information explaining a result. The type `Evidence` holds owned child failures and omission counts without borrowed observations.             |
| [Exact unordered assignment](matcher-composition.md#exact-unordered-assignment)    | Maximum one-to-one pairing of actual occurrences and expected slots, preserving duplicates.                                                               |
| [Expectation](expectation-execution.md#evaluation-and-explanation)                 | A reusable expected-side definition that evaluates a borrowed subject without tracking or raising.                                                        |
| [Expectation::Rejection](expectation-execution.md#evaluation-and-explanation)      | Original failed observation retained for explanation without repeating the check.                                                                         |
| [Expectation::Success](expectation-execution.md#evaluation-and-explanation)        | Successful observation, such as a borrowed payload or acquired guard, available for continuation.                                                         |
| [ExpectationDiagnostics](expectation-execution.md#evaluation-and-explanation)      | Failure kind and explanation hook for a rejection or an unmet expectation with no subject.                                                                |
| [Execution adapter](observation-boundaries.md)                                     | Code owning invocation, polling, traversal, or consumption around expectation execution. Distinct from `Adapter<Input>`.                                  |
| [Expression capture](fluent-entry.md#scoped-expression-capture)                    | Attaching source spelling to failure metadata, explicitly through macros or by rewriting fluent calls.                                                    |
| [Extraction](observation-boundaries.md#retaining-checks-versus-extraction)         | Checking for a value and continuing on it. Requires panic mode when rejection leaves no continuation.                                                     |
| [Fact](failure-processing.md#structured-construction-and-ownership)                | Additional failure evidence stored as a label and a `Rendered` value.                                                                                     |
| [Failure adapter](failure-processing.md#presentation-and-fallback)                 | An `Adapter<Input>` implementation processing completed failures or a later report representation.                                                        |
| [FailureBuilder](failure-processing.md#structured-construction-and-ownership)      | Structured construction API for one failure. Its target selects raising or returning data.                                                                |
| [FailureKind](failure-processing.md#structured-construction-and-ownership)         | Non-exhaustive failure-family classification. The relation and evidence describe the specific failure.                                                    |
| [Fluent alias](fluent-entry.md#borrowing-and-ownership)                            | Generated method spelling that preserves the original assertion's behavior and bounds.                                                                    |
| [HasLength](collection-semantics.md#capability-model)                              | Native finite length, measured in elements or bytes according to the subject type.                                                                        |
| [HumanReadableText](failure-processing.md#presentation-and-fallback)               | Owned text-report wrapper produced by text adapters.                                                                                                      |
| [Leaf assertion](extension-contract.md#implementing-an-assertion)                  | Check responsible for tracking and raising its failure, directly or through the shared executor.                                                          |
| [Map](collection-semantics.md#capability-model)                                    | Finite, repeatable traversal of stored key/value entries.                                                                                                 |
| [MapKeyQuery](collection-semantics.md#exact-comparisons-and-keyed-maps)            | Expected-key adapter selecting and borrowing the native query type for bulk map assertions.                                                               |
| [MapLookup](collection-semantics.md#exact-comparisons-and-keyed-maps)              | Native borrowed-key lookup returning references to the same stored entries yielded by `Map`.                                                              |
| [Mapping](assertion-lifecycle.md#projections-and-continuation)                     | Replacing the subject while moving existing chain state, including the renderer, into the continuation.                                                   |
| [Matcher](matcher-composition.md)                                                  | A reusable expectation used in composition through `Expectation` and `ExpectationDiagnostics`. `matchers` is the public catalog, not a separate protocol. |
| [Mode](assertion-lifecycle.md#chain-representation)                                | Sealed compile-time failure-handling choice: `Panic` or `Capture`.                                                                                        |
| [Multiplicity](matcher-composition.md#exact-unordered-assignment)                  | Occurrence count. Exact comparisons distinguish `[1, 1, 2]` from `[1, 2, 2]` even without order.                                                          |
| [Note](failure-processing.md#structured-construction-and-ownership)                | A `Fact` with an empty label, displayed as an unlabeled detail.                                                                                           |
| [Owned entry](assertion-lifecycle.md#entry-subject-ownership-and-mode)             | Starting a chain that owns its input. For `&T`, it owns the reference.                                                                                    |
| [Panic](assertion-lifecycle.md#entry-subject-ownership-and-mode)                   | Default mode that presents and panics on the first raised failure.                                                                                        |
| [PanicPresentation](failure-processing.md#presentation-and-fallback)               | Private, shared text-adapter trait object used only when raising assertion panics.                                                                        |
| [Path](failure-processing.md#structured-construction-and-ownership)                | Relative `PathSegment` sequence locating nested evidence within a subject.                                                                                |
| [Predicate](matcher-composition.md#truth-and-evidence)                             | Matcher wrapper around `Fn(&T) -> bool`, with an optional description and no typed error.                                                                 |
| [Probe](expectation-execution.md#budgets-and-probes)                               | Evaluation with diagnostic retention disabled. User effects still occur.                                                                                  |
| [Projection](assertion-lifecycle.md#projections-and-continuation)                  | Selecting a borrowed part or computed view of a subject for a child chain.                                                                                |
| [RandomAccess](collection-semantics.md#capability-model)                           | Stable positions plus constant-time indexed element access.                                                                                               |
| [Reference identity](reference-identity.md#which-address-is-compared)              | Pointer equality of the selected subject or `Borrow` target, including fat-pointer metadata.                                                              |
| [Reference normalization](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Borrowing entry removes one reference layer for sized pointees. Unsized targets remain reference-typed subjects.                                          |
| [Relation](failure-processing.md#structured-construction-and-ownership)            | Lowercase diagnostic sentence, stored separately from operands, without a trailing period.                                                                |
| [Rendered](diagnostic-rendering.md#capabilities-and-structure)                     | Owned diagnostic value tree with leaf text, structure, type metadata, layout, and omissions.                                                              |
| [Rendering adapter](diagnostic-rendering.md#capabilities-and-structure)            | A value wrapper that constructs a diagnostic tree through the active renderer and budget.                                                                 |
| [RenderingBudget](diagnostic-rendering.md#bounded-retention)                       | Independent retention limits per repeated group and per leaf. Implementors must keep truth independent of these limits.                                   |
| [RenderingContext](diagnostic-rendering.md#capabilities-and-structure)             | Active renderer and budget with adapters for building diagnostic value trees.                                                                             |
| [RenderingOrder](diagnostic-rendering.md#capabilities-and-structure)               | Diagnostic choice to preserve iteration or sort rendered text.                                                                                            |
| [Root chain](assertion-lifecycle.md#chain-representation)                          | Chain without a parent-record link, receiving descendant counts and owning captured failures.                                                             |
| [SetLookup](collection-semantics.md#capability-model)                              | Unique elements plus native membership using the equivalence relation enforcing uniqueness.                                                               |
| [StableOrder](collection-semantics.md#capability-model)                            | Collection capability making iteration positions semantically meaningful.                                                                                 |
| [Subject](assertion-lifecycle.md#chain-representation)                             | Current value under assertion, accessed by reference through `actual()`.                                                                                  |
| [ToHumanReadableText](failure-processing.md#presentation-and-fallback)             | Built-in adapter rendering one failure or an aggregate with Assertr's report grammar.                                                                     |
| [Unwind safety](assertion-lifecycle.md#unwind-safety)                              | Auto-trait requirements for moving or sharing chain state across an unwind-catching boundary.                                                             |
| [ValueRenderer](diagnostic-rendering.md#capabilities-and-structure)                | Trait formatting one diagnostic leaf type. Assertr owns surrounding structure.                                                                            |
