---
id: platform-compatibility
depends_on: [ ]
sources:
  - assertr/Cargo.toml
  - assertr-macros/Cargo.toml
  - assertr/src/lib.rs
  - .github/workflows/ci.yml
  - assertr-no-std-tests/Cargo.toml
  - assertr-no-std-tests/src/lib.rs
  - Justfile
  - AGENTS.md
---

# Feature and platform compatibility

[Architecture overview](README.md)

The runtime requires `alloc` and supports `no_std`. The [runtime manifest](../assertr/Cargo.toml) owns feature
dependencies. The [CI workflow](../.github/workflows/ci.yml) and [Justfile](../Justfile) own the validation matrix.

## Feature topology

| Feature                                                           | Boundary                                                                                                                                          |
|-------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| Defaults                                                          | `std` and `num`.                                                                                                                                  |
| `matchers`                                                        | Enables procedural `partial!`. Runtime matchers and declarative matcher macros need no feature.                                                   |
| `fluent`                                                          | Independently enables fluent entry, aliases, and expression capture. Uses `assertr-macros`, as does `matchers`.                                   |
| `std`                                                             | Enables hash collections and unwind-catching APIs. Also enables optional `num-traits` std support.                                                |
| `num`, `libm`                                                     | `num` enables numeric assertions. Add `libm` for floating-point classifications without `std`. Neither `std` nor `libm` implicitly enables `num`. |
| `jiff`, `tokio`, `program`, `reqwest`, `serde-json`, `serde-toml` | Enable `std` because their wrapped dependencies require it. `serde` combines JSON and TOML.                                                       |
| `rootcause`                                                       | Supports `no_std` without enabling the runtime's `std` feature.                                                                                   |
| `http`                                                            | Leaves the runtime in `no_std` mode, but the current `http` dependency requires `std` and a hosted target.                                        |
| `full`                                                            | Enables every optional API and integration.                                                                                                       |

Without `std`, core assertions, capture, structured failures, rendering, tree collections, and streaming remain
available. Panic presentation falls back on returned adapter errors in both configurations. Catching adapter panics
requires `std`, as described in [panic presentation](failure-processing.md#presentation-and-fallback). Unwind-safety
traits and presentation's `RefUnwindSafe` bound come from `core` and apply independently.

## Runtime and macro compatibility

The runtime pins [assertr-macros](../assertr-macros/Cargo.toml) exactly because generated code calls unsupported
`assertr::__private` plumbing. Keep the released pair synchronized when that protocol changes. Both crates currently
declare Rust 1.89.0 as their MSRV. [AGENTS.md](../AGENTS.md) defines release and MSRV update requirements.

## Validation coverage

CI exercises no-default, isolated `std`, isolated `num`, default, all-feature, macro-crate, and hosted no-std
configurations. Each optional feature is also checked alone to expose dependencies hidden by Cargo feature unification.

The [no-std fixture](../assertr-no-std-tests/) checks downstream use without the runtime's `std` feature. Hosted tests
can catch panics through their test harness. Embedded checks on `thumbv8m.main-none-eabihf` cover the base runtime,
`num,libm`, and the fixture with `matchers`. A hosted feature check is not evidence of embedded compatibility.

Rustdoc builds enable all features and deny warnings.
README freshness is checked against literal crate-level rustdoc. MSRV CI checks the all-feature runtime, macro crate,
and hosted no-std fixture.
