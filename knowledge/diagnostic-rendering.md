---
id: diagnostic-rendering
depends_on: [ ]
sources:
  - assertr/src/renderer/value.rs
  - assertr/src/renderer/context.rs
  - assertr/src/renderer/rendered.rs
  - assertr/src/renderer/budget.rs
  - assertr/src/renderer/mod.rs
  - assertr/src/util/selection.rs
  - assertr/src/assertions/map/entry.rs
  - assertr/src/assertions/reqwest/response.rs
  - assertr/src/crate_docs.md
  - assertr/tests/custom_assertions.rs
  - assertr-no-std-tests/src/lib.rs
---

# Diagnostic rendering

[Architecture overview](README.md)

`ValueRenderer<T>` formats one diagnostic value, called a leaf. Assertr combines leaves into collections, maps, and
other structures, then [lays out the failure report](failure-processing.md#report-grammar). `DebugRenderer` is the
default. Custom renderers can handle subjects that do not implement `Debug`. A shared reference `&R` renders like `R`,
so `with_renderer(&renderer)` satisfies the `Clone` bound of derived assertions when the renderer itself is not `Clone`.

## Capabilities and structure

Keep assertion-trait implementations independent of renderers. Put `ValueRenderer` and `Clone` bounds on individual
methods in both trait and impl. Put leaf renderer bounds on `ExpectationDiagnostics`. Evaluation also needs those bounds
when it constructs child diagnostics or paths. Disabling diagnostics at runtime does not remove compile-time bounds.

| Operation | Rendering capabilities |
|---|---|
| Opaque assertions, including direct equality | May render the entire subject as one leaf. |
| Structural collections and maps | Items, or keys and values separately. |
| Value comparison | Actual type, selected `BorrowFor::View`, required structural leaves. |
| Bulk map queries and keyed matcher paths | Selected query view. Actual maps and unexpected keys use stored key types. Exact-entry equality also renders expected-value views and counts. |
| `Entry` | Query view and nested matcher capabilities. Exact keyed matching also needs stored keys. Paths are built during evaluation, so those bounds remain. |
| Callback wrappers | Delegated checks' capabilities. Positive collection membership needs no element renderer. Exact/positional collection checks and iterator scans also render counts. |
| Negative membership | Element renderers to identify unexpected matches. |

[Operand selection](comparison-operands.md#borrowed-views) determines which view needs a renderer. The operand wrapper
itself needs none. Map entry callbacks use the same key bounds as keyed matchers. These tests check that only the required
renderers are needed:
`collection_callbacks_need_only_the_renderers_their_failures_use` and
`stable_order_callbacks_need_only_a_count_renderer` in [custom_assertions.rs](../assertr/tests/custom_assertions.rs).

Obtain `RenderingContext` through `context.render()` in expectations or `self.render()` in execution adapters. Its
adapters build owned `Rendered` trees containing leaf text, children, type metadata, layout, and omissions. Failure adapters inspect
these trees without rerendering original values. See [structural evidence examples](../assertr/src/crate_docs.md#structural-evidence).

### Rendering adapters

| Adapter | Structure and ordering |
|---|---|
| `collection`, `borrowed_collection` | Collection subject with `CollectionPresentation` metadata. |
| `stable_collection`, `stable_borrowed_collection` | Require `StableOrder`, always preserve iteration. |
| `map` | Map subject with key/value leaves. |
| `values`, `borrowed_values` | Synthetic groups, optional `with_order(RenderingOrder)`. Child types only, no outer Rust type. |
| `entry_list` | Synthetic key/value tuples with explicit order. |
| `variant`, `struct_field`, `unavailable_struct_field` | One-field wrappers preserving the owner's canonical type separately from field type. Unavailable fields have structural placeholders and no inferred type. |

`CollectionPresentation` selects syntax, type-hint visibility through `show_type_hint(bool)`, and `RenderingOrder`. It
does not enable assertion methods. `PreserveIteration` follows traversal. `SortByRenderedText` sorts formatted evidence
and marks it "(sorted for rendering)" when at least two items, counting omitted ones, were considered. Built-in
sequences and tree collections preserve iteration. Hash collections and heaps sort.

Adapters have public names and private fields. Constructing one does not traverse values or require a renderer.
Formatting it, or converting it into an owned tree with `IntoRendered::into_rendered` or the compact
`into_rendered_compact`, requires renderers for the displayed leaves. Each formatting traverses the borrowed source
again. An owned tree can be reused without another traversal. Wrapper names and placeholders are structural text outside
the leaf budget. A renderer returning `fmt::Error` does not cause a panic. The leaf keeps the text written before the
error, followed by a `<renderer error>` marker. Metadata construction, internal rendering traits, and sorting remain
private.

## Bounded retention

| `RenderingBudget` setting | Scope |
|---|---|
| Default 256 items | Each repeated group independently, including nested groups. |
| Default 4,096 characters | Each leaf independently. An opaque subject is one leaf. |
| `unlimited()` | Remove both limits. |

Omission counts remain structured data. Limits do not bound total report size, traversal work, or process memory.
`RenderingContext::budget()` returns a copy. Custom collectors must account for omitted values. See
[expectation execution](expectation-execution.md#budgets-and-probes) for how budgets and probes affect child failures.

### Sorted retention

For finite limit `k`, sorted value, map, and entry-list adapters retain at most `k` entries plus the incoming candidate:

- Produce the same retained entries as a stable full sort using the truncated leaf text and chosen compact or pretty
  format. Encounter order breaks ties.
- Rank maps by key text, then value text. Rank synthetic entries by tuple text.
- Render each inspected leaf once per conversion, including values of losing map entries.
- A nonzero limit still requires inspecting every candidate. A zero limit neither advances the iterator nor renders
  values. An unlimited budget collects and sorts everything.
- Grow storage incrementally without reserving `k` upfront. Complexity and selector mechanics are documented
  [beside the implementation](../assertr/src/util/selection.rs).

These rules apply to rendered values. [Expectation contexts](expectation-execution.md#budgets-and-probes) select child
failures, and unordered assignment also [samples candidate failures](matcher-composition.md#bounded-candidate-evidence).

## Sensitive HTTP header evidence

Reqwest `has_header_value` and `does_not_have_header` consult `ValueRenderer::sensitive_value_policy` only for failing
header evidence:

| Policy | Renderer input |
|---|---|
| `Preserve` (custom-renderer default) | Original value and sensitivity flag. Does not itself redact. |
| `Reveal` (`DebugRenderer`) | Diagnostic clone with sensitivity cleared. Original unchanged. |

Comparisons use raw bytes, including non-UTF-8 values. Passing checks do not query the policy. Generic rendering,
including direct `HeaderValue` equality, receives the original value without consulting the policy. The budget applies
after rendering.
