---
id: extension-contract
refines:
  - assertr
depends_on:
  - assertion-lifecycle
  - diagnostic-rendering
  - failure-processing
related_to:
  - collection-semantics
  - matcher-composition
  - integration-boundaries
  - reference-identity
sources:
  - assertr/src/crate_docs.md
  - assertr/src/tracking.rs
  - assertr/tests/custom_assertions.rs
  - assertr/src/assertions/mod.rs
  - assertr-macros/src/fluent_aliases/naming.rs
  - AGENTS.md
---

# Assertion families and downstream extensions

[Architecture overview](README.md)

Import `assertr::prelude::*` to bring the enabled assertion traits into scope. Autocomplete lists methods for the
current subject. The [assertion-family rustdoc](../assertr/src/assertions/mod.rs) groups them by capability or value
type and gives their signatures and bounds.

## Choosing an extension

For a custom collection, implement the relevant [behavioral capabilities](collection-semantics.md#capability-model).
This makes an existing family available without duplicating its assertions. Define a separate assertion trait for
domain-specific behavior that those capabilities do not express.

Built-in `*Assertions` traits are public for method discovery. They are not downstream implementation interfaces. The
project treats adding methods to these traits as compatible. Removing or incompatibly changing a method remains
breaking. Other public exports follow normal SemVer rules. Generated macros use the explicitly unsupported `__private`
module.

Use a [typed condition](matcher-composition.md#typed-reusable-conditions) when a reusable predicate with a domain error
is enough. Use a matcher to compose expectations. Use an assertion trait to add methods to the chain.

## Implementing an assertion

Implement the new trait for `AssertThat<'_, Subject, M, R>` with `M: Mode`, so a leaf works in panic and capture mode.
Leave `R` unconstrained on the impl. Put the required `ValueRenderer` and `Clone` bounds on each method in both the
trait and impl. The [custom assertion guide](../assertr/src/crate_docs.md#custom-assertions) has complete examples.

A checking method takes and returns `Self`. Mark it `#[track_caller]`. A leaf calls `self.track_assertion()` first,
whether it passes or fails. A method composed entirely of tracked assertions delegates counting to those assertions.
Missing tracking makes a passing capture look empty. Tracking both a wrapper and its leaves overcounts assertions.

On failure, call `self.failure(FailureKind::...)`, populate
the [structured fields](failure-processing.md#structured-construction-and-ownership), and call `.raise()`. Relations are
lowercase sentences without a trailing period or embedded values. Pass diagnostic values through `self.render()` so the
renderer and budget apply. For intentional comparisons of formatted text,
see [formatted-value comparison](diagnostic-rendering.md#formatted-value-comparison).

Projections and extractions preserve the active renderer.
Choose [mapping or derivation](assertion-lifecycle.md#projections-and-continuation) according to whether the method
continues the existing chain or creates a child. An extraction that cannot produce a subject after failure requires
panic mode.

## Adding a built-in assertion

Place behavior, diagnostic, and adapter tests beside the generic family. Use existing downstream and `no_std` fixtures
to check implementor contracts. A `NoRenderer` regression verifies that an unavailable rendering capability does not
hide an entire trait.

Each method gets its own test module. Pin its fluent alias when enabled and its caller location with
`assert_caller_location!`. New traits use `#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]`. Follow
the [alias naming rules](../assertr-macros/src/fluent_aliases/naming.rs) before adding an explicit
override. [AGENTS.md](../AGENTS.md#adding-assertions) gives the full contribution rules.

## Sources

The [custom assertion guide](../assertr/src/crate_docs.md#custom-assertions), [downstream tests](../assertr/tests/custom_assertions.rs),
and [tracking implementation](../assertr/src/tracking.rs) show how extensions participate in the chain.
The [assertion modules](../assertr/src/assertions/mod.rs) define family boundaries.
