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

The runtime supports `no_std` and requires `alloc`. The [runtime manifest](../assertr/Cargo.toml) defines which dependencies
each feature enables. [CI](../.github/workflows/ci.yml) and [Justfile](../Justfile) define the checks run for each configuration.

## Feature support

| Feature | What it enables or requires |
|---|---|
| Default | `std`, `num`. |
| `partial` | The `partial!` macro, a `macro_rules!` wrapper that forwards `$crate` to a hidden procedural macro, usable through facade re-exports. Runtime matchers and declarative matcher macros need no optional feature. |
| `fluent` | Entry, aliases, expression capture. Independently enables the macro dependency, as does `partial`. |
| `std` | Hash collections, unwind-catching APIs, `borrow-for/std`, and the optional `num-traits/std`, `serde_json/std`, and `toml/std`. |
| `num` | Numeric assertions. |
| `libm` | Floating-point classification checks when `num` is enabled without `std`. Neither `libm` nor `std` implicitly enables `num`. |
| `jiff`, `tokio`, `program`, `reqwest`, `http` | Enable `std` for wrapped std-only dependencies. |
| `rootcause`, `serde-json`, `serde-toml`, `serde` | Support embedded `no_std` with `alloc` without enabling runtime `std`. `std` enables the `std` features of `serde_json` and `toml` through weak edges. `serde` combines JSON and TOML. |
| `thirtyfour` | Enables `std` and typed WebDriver assertions, without selecting an HTTP client, TLS backend, or browser manager. |
| `thirtyfour-cdp` | Adds Chromium description reads using typed CDP commands and serde derives. |
| `full` | Every optional API and integration. |

Core assertions, capture, structured failures, rendering, tree collections, and iterator scans remain available without `std`.
Memory assertions (`assertions::MemAssertions`, `matchers::memory::NeedsDrop`) need no optional feature.

The published `borrow-for` dependency disables defaults and always enables `alloc` for wrappers, strings, and vectors.
[Unwind bounds](assertion-lifecycle.md#unwind-safety) use `core` traits. See [failure
processing](failure-processing.md#presentation-and-fallback) for how panic presentations behave with and without `std`.

## Runtime and macro compatibility

The runtime pins `assertr-macros` exactly: generated code uses unsupported `assertr::__private`. Keep released versions
synchronized. Both crates declare a minimum supported Rust version (MSRV) of 1.89.0. Follow
[AGENTS.md](../AGENTS.md) when releasing or updating the MSRV.

## Validation coverage

| Coverage | Configurations |
|---|---|
| Runtime | No defaults, isolated `std`, isolated `num`, `partial` with and without `fluent`, defaults, all features. |
| Feature independence | Each optional feature alone, to catch dependencies otherwise supplied by Cargo feature unification. |
| Macro crate | Independent build. Tests run in the workspace only, because the package ships only `src` and licenses. |
| Hosted no-std fixture | Public APIs used from another crate without runtime `std`. The test harness can catch panics. |
| Embedded | `thumbv8m.main-none-eabihf`: base runtime, `num,libm`, `fluent,rootcause,partial`, `serde-json,serde-toml`, fixture with `num` and `partial`. |
| Rustdoc | All features, warnings denied. |
| README | Freshness against literal crate-level rustdoc. |
| MSRV | All-feature runtime, macro crate, hosted no-std fixture. |

A check on a hosted target does not establish embedded compatibility. See the [downstream fixture](../assertr-no-std-tests/).
