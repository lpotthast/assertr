---
id: integration-boundaries
refines:
  - assertr
depends_on:
  - assertion-lifecycle
  - failure-processing
  - diagnostic-rendering
related_to:
  - platform-compatibility
sources:
  - assertr/src/assertions/std/mutex.rs
  - assertr/src/assertions/tokio/**
  - assertr/src/assertions/program.rs
  - assertr/src/assertions/reqwest/response.rs
  - assertr/src/assertions/rootcause/report.rs
  - assertr/src/assertions/alloc/boxed.rs
  - assertr/src/assertions/alloc/panic_value.rs
  - assertr/src/conversion.rs
---

# Integration observation and extraction boundaries

[Architecture overview](README.md)

Integrations use the existing assertion modes and diagnostic system. Their external APIs determine whether a check can retain its subject, return a new one, or consume it.

## Retaining checks versus extraction

Checks that retain the original subject can support both modes. Extractions that cannot produce the promised next subject after failure require panic mode. Successful continuations use the usual [mapping and derivation rules](assertion-lifecycle.md#projections-and-continuation).

`Program::exists` retains the program and supports both modes. `get_resolved_path` requires panic mode and returns an owned `PathBuf`. Lookup failure records the program and resolution reason. Rootcause report type checks can inspect a borrowed current context in either mode. Typed extraction requires panic mode when a mismatch leaves no value to return.

`Box<dyn Any>` and `PanicValue` offer retaining `is_of_type` checks in either mode. `has_type` and `has_type_ref` extract a payload in panic mode. An owned box yields an owned payload. Borrowed boxes and reference extraction yield borrows. Box mismatches use `FailureKind::Variant`. Panic payload mismatches use `FailureKind::Panic`. Diagnostics can name `&str` and `String` payloads. Other erased payloads appear as `dyn Any` with an explanatory note.

JSON and TOML conversions serialize the borrowed subject once and map the chain to an owned `Result<String, Error>` in either mode. Conversion preserves chain metadata and rendering settings and does not count as an assertion. Serialization errors stay in `Err` for ordinary result assertions.

## Lock and channel observations

Lock checks use immediate acquisition attempts. A standard `Mutex` is treated as unlocked when `try_lock` succeeds or returns an acquirable poisoned guard. `WouldBlock` means locked. Poison checks are separate. If a failing `is_locked` check acquires a guard for rendering, it drops the guard before raising so its own panic does not poison the mutex.

Tokio mutex checks also use `try_lock`. Value assertions run nested capture only after acquisition. Contention produces a mutex failure.

Tokio `RwLock` checks combine `try_write` and `try_read`. Write acquisition means unlocked. Failed write acquisition followed by successful read acquisition means read-locked. Both failing means write-locked. Queued waiters and reader limits can affect these results, so they describe acquisition state rather than guard counts.

A failing `RwLock` check that acquires a guard retains it through rendering and raising. This keeps the value consistent with the observation. If acquisition fails, the value is rendered as unavailable.

A watch receiver can inspect its current value without marking it seen. Its change-state checks currently require panic mode even though they retain the receiver. They use `has_changed` and treat channel closure as failure.

## HTTP response consumption

Reqwest header extraction checks presence and continues on a clone of the first header value. Header failures follow the renderer's [sensitivity policy](diagnostic-rendering.md#sensitive-http-header-evidence).

Body text and JSON extraction consume an owned response in panic mode. A borrowed call panics as misuse before returning a future. A body-read failure records the request URL and read error. JSON decoding follows a successful text read. A decoding failure records the received text, URL, expected type, and parser error.

Temporary read-error details do not remain on successful continuations. Cancellation or partial body consumption has no response-recovery guarantee. [Function assertions](assertion-lifecycle.md#panic-observation-boundaries) define the separate execution boundaries that produce `PanicValue`.

## Sources

[Program lookup](../assertr/src/assertions/program.rs), [rootcause reports](../assertr/src/assertions/rootcause/report.rs), [erased boxes](../assertr/src/assertions/alloc/boxed.rs), and [panic payloads](../assertr/src/assertions/alloc/panic_value.rs) implement retaining and extracting checks. [Serialization](../assertr/src/conversion.rs) maps to `Result`. [Standard mutex](../assertr/src/assertions/std/mutex.rs), [Tokio](../assertr/src/assertions/tokio/mod.rs), and [reqwest response](../assertr/src/assertions/reqwest/response.rs) assertions define their observation and consumption behavior.
