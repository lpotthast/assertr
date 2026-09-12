---
id: extension-contract
depends_on: [ expectation-execution, collection-semantics ]
sources:
  - assertr/src/crate_docs.md
  - assertr/src/assertions/mod.rs
  - assertr/src/tracking.rs
  - assertr/tests/custom_assertions.rs
  - assertr-macros/src/fluent_aliases/naming.rs
  - AGENTS.md
---

# Assertion extensions

[Architecture overview](README.md)

An extension participates in the chain's execution, failure, and rendering contracts. The
[custom assertion guide](../assertr/src/crate_docs.md#custom-assertions) contains complete examples.

## Choosing an extension

| Need                                                  | Extension point                                                                                |
|-------------------------------------------------------|------------------------------------------------------------------------------------------------|
| Existing operations on a custom subject               | Implement the appropriate [behavioral capabilities](collection-semantics.md#capability-model). |
| Reusable domain property with a typed error           | Implement `AssertrCondition`. Use it directly or through `condition`.                          |
| Reusable check in ordinary assertions and composition | Implement [Expectation and ExpectationDiagnostics](expectation-execution.md).                  |
| New chain methods                                     | Define a domain assertion trait for `AssertThat`, delegating reusable checks to expectations.  |
| Different diagnostics                                 | Implement `ValueRenderer` for leaves or a failure `Adapter` for completed failures.            |

Extend a capability-based family before adding a type-specific trait. `assertr::prelude::*` imports enabled assertion
traits for method discovery. Built-in `*Assertions` traits are not downstream implementation interfaces, so adding a
method is compatible by project policy. Removing or incompatibly changing one is breaking. Other public exports follow
normal SemVer rules. Macro-only plumbing belongs in the unsupported `__private` module.

## Implementing an assertion

A retaining check normally takes and returns `Self`, implemented for `AssertThat<'_, Subject, M, R>` with `M: Mode`.
Keep the impl independent of renderer capabilities. Put `ValueRenderer` and `Clone` bounds on individual methods in both
trait and impl.

The implementation must preserve caller tracking and record its attempt before the work it invokes, whether the check
passes or fails. Argument expressions have already been evaluated before the method starts. Delegation to
`apply_assertion`, `test_assertion`, or other tracked assertions must not track again. Missing tracking makes a passing
capture look empty. Double tracking inflates the count.

Implement a leaf's decision in `Expectation::evaluate` and its diagnostics in `ExpectationDiagnostics::explain`.
Populate the supplied builder's [structured fields](failure-processing.md#structured-construction-and-ownership),
rendering values through `context.render()` and its rendering adapters. The chain executor raises through the attached
failure builder. Neither hook tracks or raises, and explanation never repeats the observation. Composing methods
delegate to existing tracked assertions. Do not assemble a failure body manually.

Preserve the active renderer when changing subjects.
Choose [mapping or derivation](assertion-lifecycle.md#projections-and-continuation)
according to whether the method continues the existing chain or creates a child. Require panic mode when failed
extraction cannot return the promised subject. Explicit negative assertions own their evidence and relation.

## Verification and contribution rules

Keep behavior and diagnostics beside the owning family to test its generic contract across built-in subjects. Use
existing downstream and no-std fixtures to verify that public capabilities are sufficient outside the crate. The
regression [`trait_is_implemented_without_renderer_support`](../assertr/tests/custom_assertions.rs) demonstrates that an
unavailable leaf renderer must not hide the assertion trait.
Trait-implementation checks alone do not establish method availability. The `callback_renderer_bounds` tests in that
fixture call methods on opaque subjects with renderers limited to the leaves their diagnostics use. The no-std fixture
also compiles and runs these callback boundaries.

The `structural_rendering` tests in these same fixtures exercise the
[supported rendering adapters](diagnostic-rendering.md#capabilities-and-structure) from outside the runtime crate.
They cover collection presentation, positional and borrowed views, maps and synthetic entries, one-field wrappers,
and budget access with leaf-only renderers. Custom collectors use the copied budget for retention while preserving
truth and accounting for omitted evidence. Detailed adapter behavior remains covered beside the renderer implementation.

[AGENTS.md](../AGENTS.md#adding-assertions) owns exact test placement, caller-location and fluent-alias pins, naming,
attributes, documentation, and release-note rules.
The [alias naming implementation](../assertr-macros/src/fluent_aliases/naming.rs)
owns generated spellings. Change those rules at their source, rather than maintaining a second checklist here.
