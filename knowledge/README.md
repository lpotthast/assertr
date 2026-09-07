---
id: assertr
refines: [ ]
depends_on: [ ]
related_to: [ ]
sources:
  - README.md
  - Cargo.toml
  - assertr/Cargo.toml
  - assertr-macros/Cargo.toml
  - assertr/src/lib.rs
  - assertr/src/assertions/mod.rs
  - assertr/src/prelude.rs
  - assertr-no-std-tests/**
  - justfile
  - AGENTS.md
---

# Assertr architecture

Assertr is a Rust assertion library built around `AssertThat`, a typed chain over a borrowed or owned subject.
Assertions either panic on failure or collect structured failures for inspection. The core uses `alloc` and supports
`no_std`. Runtime matchers are always available. Optional features add `partial!`, fluent entry and aliases, and
ecosystem integrations.

These documents explain the design for contributors. Start with assertion lifecycle, then failure processing and
diagnostic rendering. The [assertion-family rustdoc](../assertr/src/assertions/mod.rs) remains the authority for
methods, signatures, and bounds.

Use the [glossary](glossary.md) to find existing concepts and their preferred names before introducing terminology.

## Architecture map

| Document                                                    | Covers                                                                                                   |
|-------------------------------------------------------------|----------------------------------------------------------------------------------------------------------|
| [Glossary](glossary.md) | Existing concepts, preferred terms, and their Rust names. |
| [Assertion lifecycle](assertion-lifecycle.md)               | Chain state, ownership, mapping, derivation, capture completion, and panic observation.                  |
| [Failure processing](failure-processing.md)                 | Structured failures, root storage, adapters, and panic presentation.                                     |
| [Diagnostic rendering](diagnostic-rendering.md)             | Leaf renderers, structural output, ordering, budgets, and formatted-value comparisons.                   |
| [Matcher composition](matcher-composition.md)               | Matcher truth and evidence, typed conditions, assertion callbacks, unordered assignment, and `partial!`. |
| [Collections, maps, and iterators](collection-semantics.md) | Behavioral capabilities, keyed lookup, and stream consumption.                                           |
| [Reference identity](reference-identity.md)                 | Pointer comparisons, reference normalization, multiplicity, and pointer metadata.                        |
| [Fluent entry](fluent-entry.md)                             | Borrowing and owning entry methods, aliases, and expression capture.                                     |
| [Assertion extensions](extension-contract.md)               | Choosing a family and implementing a tracked, mode-generic assertion.                                    |
| [Integration boundaries](integration-boundaries.md)         | State observations, extraction, response consumption, and serialization.                                 |
| [Platform compatibility](platform-compatibility.md)         | Feature dependencies, `std` and `no_std`, macro compatibility, and validation.                           |

## Repository structure

[assertr](../assertr/Cargo.toml) contains the runtime and public API. [assertr-macros](../assertr-macros/Cargo.toml)
generates structural matchers, fluent aliases, and expression capture. [assertr-no-std-tests](../assertr-no-std-tests/)
checks the runtime from a downstream crate without its `std` feature. Runtime integration tests live
under [assertr/tests](../assertr/tests/).

The runtime separates assertion families, matching, failure processing, and rendering. Behavioral capabilities determine
which assertions a subject supports. Renderers format diagnostic leaves. An assertion failure carries the resulting
evidence to capture storage or panic presentation.

## Document metadata

Each page begins with YAML metadata:

- `id` is its unique identifier. The overview uses `assertr`.
- `refines` names broader documents that this page expands.
- `depends_on` names concepts to read first. It describes reading prerequisites, not Rust module dependencies.
- `related_to` names useful companion documents. These references need not be reciprocal.
- `sources` lists supporting paths relative to the repository root. Globs are allowed. Markdown links in the body are
  relative to the page itself.

Update the relevant page and its source references when behavior changes. Keep detailed API examples in the owning
rustdoc and link to them here. [AGENTS.md](../AGENTS.md) records contribution rules. The [justfile](../justfile) defines
maintenance and validation commands.
