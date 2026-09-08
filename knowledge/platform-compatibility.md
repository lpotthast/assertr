---
id: platform-compatibility
refines:
  - assertr
depends_on:
  - failure-processing
related_to:
  - fluent-entry
  - integration-boundaries
sources:
  - assertr/src/lib.rs
  - assertr/Cargo.toml
  - assertr-macros/Cargo.toml
  - .github/workflows/ci.yml
  - assertr-no-std-tests/**
  - justfile
  - AGENTS.md
---

# Feature and platform compatibility

[Architecture overview](README.md)

The runtime supports `no_std` with `alloc`. Feature dependencies determine which assertion families and external
integrations are available.

## Feature topology

Defaults enable `std` and `num`. Runtime matchers and declarative matcher macros need no feature. The `matchers` feature
enables the procedural `partial!` macro. The independent `fluent` feature enables fluent entry, aliases, and expression
capture. Both use `assertr-macros`. `full` enables every integration and optional API.

Without `std`, the runtime retains core assertions, capture, structured failures, rendering, tree collections, and
streaming iterators. Hash collections and unwind-catching APIs require
`std`. [Panic presentation](failure-processing.md#presentation-and-fallback) also catches adapter panics only with
`std`.

The chain's [unwind-safety traits](assertion-lifecycle.md#panic-observation-boundaries) and presentation's
`RefUnwindSafe` bound use `core` and apply independently of the runtime's `std` feature.

`matchers`, `fluent`, `num`, `libm`, and `rootcause` support embedded `no_std` targets. `http` leaves the runtime in
`no_std` mode but its dependencies currently need a hosted target. Integrations for jiff, Tokio, serde JSON/TOML,
program lookup, and reqwest enable `std` themselves. `serde` combines the JSON and TOML features.

The `std` and `libm` dependencies on optional `num-traits` features are weak. Enabling either does not silently enable
`num`. Select `num` with `libm` for floating-point classifications without `std`.

## Runtime and macro compatibility

The runtime pins `assertr-macros` exactly because generated code calls its unsupported `__private` protocol. Update and
release the pair together when that protocol changes. Downstream code must not depend on the private protocol.

Both crates declare Rust 1.89.0 as their minimum supported version. CI installs that toolchain and checks the runtime
with all features, the macro crate, and the hosted no-std fixture.

## Validation

CI checks no-default, isolated `std`, isolated `num`, default, all-feature, macro-crate, and hosted no-std
configurations. Every optional feature is also checked alone to catch undeclared dependencies hidden by Cargo feature
unification.

Embedded-target checks verify compilation beyond a hosted test harness. Hosted no-std tests exercise allocation and
panic observation without enabling the runtime's `std` feature. Documentation builds enable all features and deny
warnings. README freshness is checked against the literal crate-level rustdoc.

## Sources

The [runtime manifest](../assertr/Cargo.toml) defines feature dependencies and the
exact [macro crate](../assertr-macros/Cargo.toml) requirement. [CI](../.github/workflows/ci.yml),
the [no-std fixture](../assertr-no-std-tests/), and the [justfile](../justfile) define
validation. [AGENTS.md](../AGENTS.md) records release and MSRV update rules.
