---
id: diagnostic-rendering
refines:
  - assertr
depends_on: [ ]
related_to:
  - failure-processing
  - matcher-composition
  - collection-semantics
  - integration-boundaries
sources:
  - assertr/src/renderer/**
  - assertr/src/assertions/core/debug.rs
  - assertr/src/assertions/core/display.rs
  - assertr/src/assertions/rootcause/report.rs
  - assertr/src/assertions/reqwest/response.rs
---

# Diagnostic rendering

[Architecture overview](README.md)

`ValueRenderer<T>` formats a leaf of type `T`. Assertr builds collection, map, tuple, wrapper, and failure structure
around those leaves.

## Capabilities and structure

Each assertion method requires only the renderer capabilities its diagnostics use. A renderer can support one method
while another remains unavailable because it needs a different leaf type.
The [extension contract](extension-contract.md#implementing-an-assertion) explains where to place these bounds.

`RenderingContext` provides adapters for values, sequences, maps, and structural fields. They produce `Rendered` trees
with type metadata and omission information. Custom assertions use these adapters to preserve the active renderer and
budget. [Mapping and derivation](assertion-lifecycle.md#projections-and-continuation) carry the rendering configuration
to new subjects.

Stable collections retain semantic iteration order in diagnostics. Sets and maps may sort by rendered text for
deterministic output. This presentation choice does not
grant [ordering or lookup capabilities](collection-semantics.md#capability-model).

## Bounded retention

`RenderingBudget` defaults to 256 items per repeated group and 4,096 characters per leaf. Each limit applies separately
to each group or leaf. Neither is a total failure-size limit. `unlimited()` removes both limits. Truncated values record
how many items or characters were omitted.

The budget limits retained evidence. Assertions may inspect more items to decide truth, and sorted diagnostics may
render more candidates before selecting which ones to keep. Matcher probes can suppress evidence work. Derived chains
and nested matching contexts inherit the
budget. [Streaming scans](collection-semantics.md#borrowed-traversal-versus-terminal-streams) also have their own
preview retention limits.

## Formatted-value comparison

Formatting assertions compare complete generated text before applying diagnostic limits. `has_debug_string` compares the
subject's `Debug` output with preformatted expected text verbatim. `has_debug_value` formats both operands with `Debug`.
`has_display_value` formats both with `Display`. Quotes, escapes, and every other character affect the comparison.

After a mismatch, `ValueRenderer<str>` and the budget control the retained actual and expected text. The diagnostic
renderer does not determine whether the formatted values match.

Rootcause current-context assertions follow the same distinction through the report's formatter hook. The display form
formats both sides. The debug-string form accepts already-formatted expected text. These assertions compare the
formatter's view of the current context.

## Sensitive HTTP header evidence

Failing reqwest header assertions consult `ValueRenderer::sensitive_value_policy` before rendering a sensitive
`HeaderValue`:

- `Preserve`, the default for custom renderers, passes the original value and sensitivity flag. The renderer decides how
  to display it.
- `Reveal`, used by `DebugRenderer`, passes a diagnostic clone with the flag cleared. The original header is unchanged.

The budget applies after rendering. Header comparisons use raw bytes, including non-UTF-8 bytes. Passing assertions do
not query the policy. Generic rendering, such as equality on a `HeaderValue`, receives the original value without
consulting it.

## Sources

The [renderer module](../assertr/src/renderer/mod.rs) documents the
design. [Budgets](../assertr/src/renderer/budget.rs), [structural adapters](../assertr/src/renderer/context.rs),
and [Rendered](../assertr/src/renderer/rendered.rs) implement
retention. [ValueRenderer](../assertr/src/renderer/value.rs) defines leaf formatting and sensitivity
policy. [Debug](../assertr/src/assertions/core/debug.rs), [Display](../assertr/src/assertions/core/display.rs),
and [rootcause](../assertr/src/assertions/rootcause/report.rs) assertions compare formatted
text. [Reqwest response assertions](../assertr/src/assertions/reqwest/response.rs) apply the header policy.
