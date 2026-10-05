---
id: extension-contract
depends_on: [ ]
sources:
  - assertr/src/crate_docs.md
  - assertr/src/assertions/mod.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/tracking.rs
  - assertr/tests/custom_assertions.rs
  - assertr-macros/src/fluent_aliases/naming.rs
  - AGENTS.md
---

# Assertion extensions

[Architecture overview](README.md)

Choose an extension point based on what you need to change. The
[custom assertion guide](../assertr/src/crate_docs.md#custom-assertions) provides worked implementations.

## Choosing an extension

| Need | Extension point |
|---|---|
| Existing operations on a custom subject | Implement [behavioral capabilities](collection-semantics.md#capability-model). |
| A check usable on its own or in a matcher | Implement [evaluation and explanation](expectation-execution.md#evaluation-and-explanation). |
| New chain method | Extend a capability family first. Use a type-specific trait only for type-specific behavior. Select a [public executor entry](expectation-execution.md#chain-execution). |
| Invocation, polling, or consumption | Use an [execution adapter](observation-boundaries.md#execution-adapters). |
| New subject or extraction | Choose [mapping, derivation, and continuation mode](assertion-lifecycle.md#projections-and-continuation). |
| A wrapper for expected values | Implement [borrowed-view selection](comparison-operands.md#borrowed-views). |
| Different formatting for diagnostic values | Implement a [leaf renderer](diagnostic-rendering.md#capabilities-and-structure). |
| Different report or output target | Implement a [failure adapter](failure-processing.md#presentation-and-fallback). |

## Testing extensions

Test behavior beside the generic implementation. Use downstream fixtures to verify that an extension can be written
using only public APIs.

| What to check | Existing coverage |
|---|---|
| Trait availability independent of renderer | `trait_is_implemented_without_renderer_support` in [custom_assertions.rs](../assertr/tests/custom_assertions.rs). |
| Method bounds | `callback_renderer_bounds` calls methods on opaque subjects with only required leaf renderers. Trait-implementation checks alone cannot establish method availability. |
| Custom rendering | `structural_rendering` uses public structural adapters and reads a copy of the budget from outside the crate. |
| Platform support | The existing [no-std fixture](../assertr-no-std-tests/src/lib.rs) exercises downstream callbacks and rendering. See [validation coverage](platform-compatibility.md#validation-coverage) for the configurations tested. |

[AGENTS.md](../AGENTS.md) specifies naming, attributes, caller-location and fluent-alias tests, test placement,
documentation, SemVer, and release procedures. Generated aliases follow the
[naming rules](../assertr-macros/src/fluent_aliases/naming.rs).
