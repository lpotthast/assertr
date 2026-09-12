---
id: diagnostic-rendering
depends_on: [ ]
sources:
  - assertr/src/renderer/value.rs
  - assertr/src/renderer/context.rs
  - assertr/src/renderer/rendered.rs
  - assertr/src/renderer/budget.rs
  - assertr/tests/custom_assertions.rs
  - assertr-no-std-tests/src/lib.rs
  - assertr/src/assertions/core/debug.rs
  - assertr/src/assertions/core/display.rs
  - assertr/src/assertions/rootcause/report.rs
  - assertr/src/assertions/reqwest/response.rs
  - assertr/tests/custom_assertions.rs
  - assertr-no-std-tests/src/lib.rs
---

# Diagnostic rendering

[Architecture overview](README.md)

[`ValueRenderer<T>`](../assertr/src/renderer/value.rs) formats a diagnostic leaf. Assertr builds structural syntax and
failure layout around leaves. `DebugRenderer` is the default, but subjects need no `Debug` implementation when a custom
renderer supplies the required capabilities.

## Capabilities and structure

An assertion needs only the leaf-rendering implementations its diagnostics use. Structural collection checks render
items, and map checks render keys and values separately. An opaque assertion, such as direct equality, can treat the
whole subject as one leaf. Method-level bounds let Rust reject a missing rendering capability without hiding unrelated
assertions. Keeping those bounds off blanket trait implementations is an assertion-author obligation.

Callback wrappers require only the rendering capabilities used by their delegated checks and callback assertions,
plus `Clone` to carry the active renderer into child chains. Positive collection membership callbacks can inspect opaque
elements without any leaf renderer. Exact and positional collection checks and iterator scans additionally render
counts. Map entry callbacks render query keys, and exact keyed callbacks also render unexpected stored keys.
Negative membership checks retain element renderers because their failures identify unexpectedly matching elements.

[`RenderingContext`](../assertr/src/renderer/context.rs), obtained through `AssertThat::render()` or
`AssertionContext::render()`, supplies rendering adapters for values, collections, maps, and fields. These build owned
[`Rendered`](../assertr/src/renderer/rendered.rs) trees containing leaf text, structural children, type metadata, layout
settings, and omission counts. Failure adapters can inspect or print them without rendering original values again.
Mapping moves the renderer and derivation clones it, as described
in [assertion lifecycle](assertion-lifecycle.md#projections-and-continuation).

`CollectionPresentation` chooses collection syntax, type-hint visibility, and `RenderingOrder`.
`PreserveIteration` retains traversal order. `SortByRenderedText` orders formatted evidence for deterministic reports.
Built-in sequences and tree collections preserve iteration. Hash collections and heaps sort diagnostic text. These
settings grant no [behavioral capability](collection-semantics.md#capability-model).

The supported downstream surface uses `collection` and `borrowed_collection` for collection subjects with their
presentation metadata, `stable_collection` and `stable_borrowed_collection` for positional diagnostics, and `map`
for map subjects. The stable adapters require `StableOrder` and always preserve iteration order. `values` and
`borrowed_values` create synthetic groups with an optional `with_order(RenderingOrder)` setting. `entry_list`
creates synthetic key/value tuples with an explicit `RenderingOrder`. Synthetic groups retain child types
without claiming an outer Rust type.

`variant`, `struct_field`, and `unavailable_struct_field` cover one-field wrappers. They retain the owner's
canonical type independently of the field type. An unavailable field has a structural placeholder and no
inferred type. Wrapper names and placeholders are structural text, outside the leaf budget.

Adapters have public names in `renderer` and inaccessible fields. Construction is lazy and renderer-independent.
Formatting and conversion to `Rendered` require only the displayed leaf renderers. Each formatting traverses
the borrowed source again, while an owned tree can be reused without rerendering. Metadata construction,
rendering implementation traits, and sorting algorithms remain private.

## Bounded retention

[`RenderingBudget`](../assertr/src/renderer/budget.rs) defaults to 256 items per repeated group and 4,096 characters per
leaf. Limits apply independently, so they bound neither total report size nor comparison work. `unlimited()` removes
both limits. Omitted item and character counts remain in the rendered data.

The budget controls retained evidence. Expectation implementors must keep pass/fail independent of that budget, and the
executor records truth separately from retained children. An assertion may inspect more items, and sorted diagnostics
may render more candidates before selecting retained entries. Derived chains and nested expectation contexts inherit the
budget.
`RenderingContext::budget()` exposes a copy of both limits for custom evidence collectors. Changing that copy
does not update the chain. Collectors determine truth independently, account for omitted children, and consult
`AssertionContext::is_diagnostic()` when retaining optional expectation evidence. The budget bounds retained
output, not traversal work or peak sorting memory.
[Probes](expectation-execution.md#budgets-and-probes) suppress built-in diagnostics, while
[streaming scans](collection-semantics.md#borrowed-traversal-versus-terminal-streams) also impose a preview limit.

## Formatted-value comparison

Formatting assertions compare complete generated text before diagnostic truncation. `has_debug_string` compares `Debug`
output with preformatted text verbatim. `has_debug_value` formats both operands with `Debug`. `has_display_value` uses
`Display` for both. Their rejections retain the generated text, so explanation does not format operands again.

On mismatch, `ValueRenderer<str>` and the budget control the retained evidence. They do not determine equality. Probes
still format because formatting is the check. Rootcause current-context formatting assertions use the report's formatter
hook with the same separation between comparison and diagnostic rendering.

## Sensitive HTTP header evidence

Reqwest response `has_header_value` and `does_not_have_header` consult `ValueRenderer::sensitive_value_policy` when
rendering header evidence. `Preserve`, the default for custom renderers, passes the original value and sensitivity flag.
It does not itself redact. `Reveal`, selected by `DebugRenderer`, clears the flag on a diagnostic clone of a sensitive
header. The original stays unchanged.

Header comparisons use raw bytes, including non-UTF-8 bytes. Passing checks do not query the policy. Generic rendering,
such as direct equality on `HeaderValue`, receives the original value without consulting this policy. The budget applies
after rendering.
