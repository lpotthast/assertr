

---

# Simplify unordered matcher evidence ownership

## Summary

Give each expected slot sole ownership of its bounded candidate evidence. Missing slots display those rejections. Unexpected elements display their total count and      
applicable surplus-constraint descriptions.

This contract applies at every budget, including unlimited. For [1, 1, 99] against eq(1), the report retains the unexpected count of two and the surplus description     
for 1, but removes the rejection explaining 99.

Public signatures, trait bounds, maximum matching, and once-per-pair evaluation remain unchanged.

## Implementation

- Replace shared sampling with a small private slot collector beside ElementsAreInAnyOrder. Each slot owns a Smallest collection of failures and a total direct-failure  
  count. Remove expectation/assignment.rs, its module declaration, occurrence samples, Rc ownership, union recovery, and destination routing.

- Use the same collectors throughout search and diagnostic completion. Offer each rejected pair exactly once. Keep pair evaluation isolated from retained evidence so    
  sample admission never changes pair budgets or explanation eligibility.

- Move failure totals into the slot collectors. Simplify the sparse pair cache to boolean outcomes, retaining a single cached evaluator for search and completion.
- Preserve search and completion order, immediate explanation of eligible rejections, and guard release before the next pair. Exact matches, probes, and zero            
  allowances continue to skip completion.

- After completion, consume missing slots’ collectors into ordinary Evidence and discard occupied slots’ samples. Preserve complete missing-constraint descriptions,     
  scoped paths, and nested evidence.

- Preserve selection order: cached child-report text in sorted scopes, with original occurrence and child ordinal breaking ties. Iteration-preserving scopes use those   
  identities directly.

- Compute missing-slot omissions as total direct failures minus retained children. Nested omissions remain attached to their failure. Later assembly truncation          
  contributes omissions once.

- Simplify unexpected-element assembly to the existing count and surplus descriptions obtained from cached matches. Rejections excluded by the new contract do not       
  count as omitted children.

- Remove selector helpers used solely by shared ownership and union recovery. Keep Smallest’s existing buffering, bounded selection, and ordered extraction.

Candidate retention becomes at most k × expected_slots direct failures, plus the current pair’s evidence. Assignment can no longer disqualify retained missing-slot      
candidates. Nested payloads and the potentially quadratic scalar cache still prevent an absolute memory bound.

## Validation

- Update exact report tests for count-only unexpected elements, mixed rejected/surplus occurrences, missing slots, empty inputs, and unlimited budgets.
- Replace shared-ownership and starvation regressions with slot independence, bounded retention, deterministic ties, and exact omission accounting. Exercise both        
  rendering orders and budgets 0, 1, 2, and unlimited.

- Preserve overlapping-constraint, duplicate-multiplicity, exhaustive pair-trace, sparse-success, probe, renderer-count, nested-path, and guarded-observation            
  regressions.

- Check collection and iterator adapters, including iterator length shortcuts, and retain coverage in existing downstream and no_std fixtures.
- Write assertions using assertr in focused test submodules. Run focused tests followed by just verify.

The current baseline passes all 26 unordered-matcher tests, three assignment-evidence tests, and the downstream guarded-observation regression.

## Documentation and compatibility

- Update matcher rustdoc and the architecture, matcher-composition, and diagnostic-rendering knowledge pages. Remove shared-routing and assignment-induced-underfill     
  claims.

- Consolidate the existing Unreleased changelog entry around the final behavior relative to v0.7.1. This revises an unreleased matcher contract and requires no          
  additional public API breaking change or version bump.

- Preserve existing staged and unrelated unstaged changes. Add no compatibility mode, dependencies, feature gates, or renderer requirements.                             
                                                                                                                                                                           
---

# Share finite positional checks and existing window storage

## Summary

The prefix/suffix duplication and both full-collection contiguous buffers remain. Refine the implementation around the newer Tail, deque-backed previews, lazy mismatch  
recording, and shared evidence allowance.

Retain the selected behavior: finite contiguous checks stop traversing at the first successful window. Empty patterns visit no elements.

## Implementation

- Share finite positional comparison. In assertr/src/assertions/collection/value.rs, extract a private helper accepting the collection, expected slice, observed         
  length, and offset. Prefix supplies zero. Suffix supplies length.saturating_sub(expected.len()). Preserve PositionalRejection, lazy operand borrowing, first-mismatch  
  short-circuiting, and comparisons of available elements even when the collection is too short.

- Share the current budget-aware explanation. Extract the conditional actual-length fact and mismatch construction. Preserve context.isolated(), lazy record_with,       
  relative PathSegment::Index, and into_evidence().explain(failure). At zero budget, a mismatch contributes one omitted child without rendering its leaves. A length-    
  only rejection contributes no child omission. Keep each public expectation’s explicit relations and subject/expected rendering.

- Extract storage from the existing Tail. Add a crate-private BoundedWindow<T> under the private utility module. Move Tail’s retention limit, deque, and eviction logic  
  into it. Provide new, push, len, ordered deque iteration, and ownership transfer through into_deque. Allocate incrementally, evict before inserting, and retain        
  nothing at limit zero. Keep consumption counting and preview finalization in the iterator-specific Tail wrapper.

- Preserve the newer iterator implementation. Existing equality and matcher scans continue using Tail, backed by the extracted storage. Keep equality’s                  
  max(pattern_length, 16) retention and matcher windows’ pattern-length retention. Preview finalization trims and moves the existing deque. Introduce neither            
  make_contiguous nor a second item buffer. Preserve current rejection types, evidence policies, stopping points, first-exhaustion behavior for non-fused iterators,     
  and resource lifetimes.

- Bound finite contiguous storage. Use BoundedWindow<&C::Item> in equality contiguous evaluation and the contiguous policy in assertr/src/assertions/collection/         
  elements_are.rs. Fill the first candidate, then advance one element after each rejected complete candidate. Retain at most min(actual_length, expected_length)         
  references. Equality compares only complete windows and short-circuits each at its first mismatch.

- Preserve finite matcher scheduling and budgets. Evaluate every available slot within a candidate. Create candidate contexts from alternatives.isolated() so completed  
  candidates reduce subsequent diagnostic allowances. Continue required evaluations after the allowance is exhausted, and discard speculative failures on eventual       
  success. Too-short collections still evaluate their available positions once, describe missing positions, and retain length evidence. Terminal incomplete matcher      
  windows continue evaluating no matchers and retain their separate description behavior.

## Validation

- Preserve existing prefix/suffix budget, nested-path, comparison-count, custom-renderer, borrowed-operand, and NoRenderer regressions. Add combined short-length/       
  first-mismatch coverage for the extracted helper.

- Add finite traversal and comparison traces covering empty patterns, short input, overlaps, wraparound, early and late success, and complete failure. Verify matcher/   
  callback slot order and successful candidates after evidence exhaustion.

- Extend finite matcher evidence tests across zero, small, and unlimited budgets. Pin retained paths, omission counts, missing-slot descriptions, and no repeated        
  evaluation during explanation.

- Test shared-buffer retention and ownership transfer with non-Clone items. Extend the existing allocation fixture to show that finite contiguous working storage stays  
  bounded as collection length grows with a fixed pattern. Measure probes to separate window storage from diagnostic allocations.

- Retain existing assertion_tests! fluent/caller pins and iterator window, allocation, preview, and resource-lifetime tests.
- Run focused collection, iterator, and allocation tests, then just verify. Rechecked baseline passes: 217 collection, 35 iterator, 9 allocation, and 797 no-default-    
  feature assertion tests.

## Compatibility and documentation

Public types, signatures, bounds, dependencies, and features remain unchanged. Preserve the allocation-free successful finite prefix/suffix paths and the newer          
borrowed SatisfyingList adapters.

Update collection semantics documentation for finite traversal and storage. Consolidate release-notable wording into the existing Unreleased positional-assertion entry  
against v0.7.1. No separate refactor entry or version bump is needed.

Preserve all existing staged, unstaged, and untracked work. Diagnostic evidence retains its current budget behavior independently of the reduced element buffer.         
                                                                                                                                                                           
---

• I rechecked the current worktree. The other agent’s five items are now: two implemented, one partially implemented, and two open. Several findings from my review have
also been addressed.

These are implementation statuses from source inspection. I did not rerun tests or benchmarks.

Already implemented:

- Iterator equality uses bounded AssertionContext/Evidence. UnsatisfiedElements, indexed_children, and individual failure vectors are gone. The unused contiguous-search
  evidence channel is also removed. Budget regressions were added in assertr/src/assertions/iterator/positional.rs:386. This addresses other #1 and my #1.

- Child locations consistently use typed paths. The special location-fact APIs and renderer precedence logic are gone. See child rendering (assertr/src/failure/adapter/
  adapters/human_readable.rs:347). This addresses my #2.

- Callback slices adapt directly to MatcherList. SatisfyingList (assertr/src/expectation/lists.rs:37) replaces the eleven intermediate callback-vector conversions. This
  addresses other #4.

The consolidated remaining work is:

Opportunity                                                Status                   What remains                                                                        
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Simplify ordered exact equality diagnostics                Open                     ContainsExactly (assertr/src/assertions/collection/value.rs:610) still runs
bipartite matching after positional rejection. Switching to positional mismatch
evidence remains a diagnostic-contract decision.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Share finite prefix/suffix comparison and explanation      Open                     Prefix (assertr/src/assertions/collection/value.rs:306) and suffix still duplicate
comparison and mismatch-report construction. This can proceed independently of
changing ordered-exact diagnostics.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Share rolling-window traversal                             Partial                  Iterator equality and matcher paths now share Tail storage. Their traversal loops
remain separate, and collection contiguous checks (assertr/src/assertions/
collection/value.rs:536) still buffer the entire subject before checking an empty
expectation.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Centralize lazy evidence recording                         Partial                  record_with (assertr/src/expectation/context.rs:186) exists and is used by equality
paths. Manual diagnostic/omission branches remain in collection matching, exact map
matching, and unordered report assembly. outcome also still duplicates part of that
policy.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Reduce simple production expectation boilerplate           Open                     Character, numeric, and status-class expectations still have repeated
implementations. The new test macro does not address production code.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Deduplicate diagnostic tests across modes and renderers    Partial                  assertion_tests! (assertr/src/test_support/mod.rs:36) now shares fluent/caller
pins. Repeated complete reports across panic, capture, custom rendering, and
redaction remain, for example in reqwest tests (assertr/src/assertions/reqwest/
response.rs:1215).
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Narrow integration and no_std test ownership               Open                     The specific cross-layer duplication identified remains. Unused-chain and panic-
propagation behavior still appears in unit tests, runtime integration tests, and
the no_std fixture. The substantial manual-leaf authoring suite also remains.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Move header emptiness into HasLength                       Open                     HttpHeaderValueAssertions (assertr/src/assertions/http/header_value.rs:54) still
owns both emptiness methods and their projection/Clone requirements. Requires
deliberate API compatibility treatment.
─────────────────────────────────────────────────────────  ───────────────────────  ─────────────────────────────────────────────────────────────────────────────────────
Simplify unordered matcher evidence ownership              Open, design decision    CandidateSamples/AssignedEvidence (assertr/src/expectation/assignment.rs:71) still
use occurrence and slot samples, shared ownership, union recovery, and routing.
Both proposed alternatives remain unimplemented.

The smaller findings are also still open:

- Replace RefCell<NumberOfAssertions> with an appropriately unwind-wrapped Cell<usize>.
- Share the duplicated runtime-crate resolver between the procedural macros.
- Unify local workflow and CI configurations. The numeric no_std fixture configurations still differ.

I would next prioritize shared window traversal, finite prefix/suffix sharing, and completing lazy-recording adoption. Ordered-exact diagnostics and unordered evidence
ownership are the larger decisions because they intentionally change what failures explain.

---

# Maintainable Send support for ordinary async chains

## 1. Recommendation and tradeoffs

Make ordinary chains conditionally Send and Sync under std. Keep synchronization private, retain concrete presentation types, and remove unused panic-presentation       
state from capture chains.

This improves the previous plan in four ways:

- Passing assertions use an atomic counter instead of acquiring a mutex.
- Capture keeps its existing callback signatures and avoids restrictions from unused local adapters.
- Borrowed async mapping gets a necessary lifetime fix.
- Consuming async function assertions accept Send-only callbacks without requiring Sync.

Approach                              Advantages                            Costs and implications                Decision                                              
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  
Typed presentation parameter          Automatic Send inference,             Different adapters produce            Use                                                   
preserves local adapters, follows     different chain types and                                                                   
the renderer model                    additional monomorphizations                                                                
────────────────────────────────────  ────────────────────────────────────  ────────────────────────────────────  ─────────────────────────────────────────────────────  
Erased local/shared presentation      Easier runtime replacement and        Separate local/thread-safe APIs       Reject following your preference                      
types                                 type unification                      and less automatic inference                                                                
────────────────────────────────────  ────────────────────────────────────  ────────────────────────────────────  ─────────────────────────────────────────────────────  
Public local/shared record            Users could avoid synchronization     Another public type dimension,        Keep record backends private                          
policies                              for local execution under std         more bounds, and more combinations                                                          
to maintain                                                                                 
────────────────────────────────────  ────────────────────────────────────  ────────────────────────────────────  ─────────────────────────────────────────────────────  
One mutex for all records             Simple synchronization model          Every passing assertion acquires a    Use an atomic counter and protected diagnostic data   
mutex                                                                                       
────────────────────────────────────  ────────────────────────────────────  ────────────────────────────────────  ─────────────────────────────────────────────────────  
Presentation embedded in renderer     Fewer visible generic parameters      Couples independent                   Reject                                                
or mode                                                                     responsibilities and complicates                                                            
replacement and projection                                                                  
────────────────────────────────────  ────────────────────────────────────  ────────────────────────────────────  ─────────────────────────────────────────────────────  
Ownership-specific chain types        Would support owned subjects that     Broadly changes entry, extraction,    Defer                                                 
are Send but not Sync                 callbacks, and subject storage

A local primitive benchmark supported separating the counter from the mutex. This is directional evidence, not a prediction of whole-library performance. Compiler       
probes on Rust 1.89 also confirmed the proposed auto-trait structure and the borrowed-mapper correction.

## 2. Public API, presentation, and capture

### Presentation remains independent

Extend the chain with one defaulted parameter:

AssertThat<'t, T, M, R = DebugRenderer, P = ToHumanReadableText>

Reuse the existing ToHumanReadableText as the default. Do not introduce another default-presentation marker.

- Export a documented, sealed PanicPresentation handle contract and opaque CustomPanicPresentation<A>.
- Adapter authors continue implementing the existing Adapter trait. The handle contract is not another downstream extension protocol.
- Store custom adapters through typed Arc sharing under std and typed Rc sharing without std. Preserve the existing 'static, RefUnwindSafe, output, and error-           
  formatting requirements.

- Neither adapters nor their errors gain blanket Send, Sync, or Clone requirements. Implement handle cloning without requiring A: Clone.
- Put presentation Clone bounds on operations that actually derive children. Mappings and replacements move presentation state.
- Erase the adapter only through a temporary borrow during synchronous presentation. Keep the stored type concrete so Rust can infer its auto traits.
- Preserve the existing error and panic fallback, including catching error-formatting panics.

Make these setters available on panic-mode chains:

- with_panic_presentation(adapter) replaces the presentation type completely. It does not retain or nest the previous presentation.
- with_default_panic_presentation() restores the default type while preserving the subject, renderer, records, and diagnostic settings.

The reset method provides a way to return to an ordinary chain type without extracting the subject or losing history.

### Capture does not carry unused presentation

Normalize presentation at the capture boundary:

- Capture callbacks receive ordinary default-presentation capture chains.
- Retain the incoming panic-presentation handle in a named local guard outside the capture chain until capture completes or unwinds.
- Do not invoke that handle during capture.
- Restrict presentation setters to panic mode. Capture callers customize reporting by adapting the returned failures.
- Preserve the existing generic parameter counts and callback shapes of capture, verify, and verify_owned.

This preserves adapter lifetime through the callback while allowing capture children to cross thread boundaries independently of a local panic adapter.

Propagate P through actual child derivation, extraction, and child callbacks. Keep isolated matcher callbacks and collect_assertions on default-presentation capture     
chains. Expectation, ExpectationDiagnostics, and AssertionContext remain presentation-independent.

Generalize assertion implementations over P, but add presentation parameters to assertion traits only where their signatures expose inherited chain types.

## 3. Private records and async operation boundaries

### Synchronization

Centralize record storage and operations in a private module. Other subsystems use operations such as tracking, appending messages, recording failures, collecting       
messages, and completing capture. They do not receive guards or access cells directly.

- Keep parent links as borrowed references to ancestor records. Do not introduce Arc ownership for the chain graph.
- Under std, use an AtomicUsize assertion counter and a mutex protecting messages and captured failures.
- Without std, use local cell storage and RefCell diagnostic storage. Add no atomic-target requirement or synchronization dependency.
- Use relaxed atomic ordering because the counter records attempts and does not publish diagnostic data or signal completion. Its role must remain separate from         
  synchronization. Rust’s ordering documentation (https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)

- Saturate the private count at usize::MAX in both backends. Preserve exact counts below that limit and prevent overflow from making a nonempty capture appear empty.
- Release each record guard before visiting ancestors. Never hold multiple record locks together.
- Run callbacks, conversions, renderer cloning, rendering, presentation, and user-value destruction outside internal locks.
- Recover poisoned diagnostic locks only with documented valid-state invariants. Reserve capacity before updating paired failure/expression-index bookkeeping so         
  unwinding cannot leave invalid indexes.

Keep synchronization confined to library-owned records. Do not add unsafe auto-trait implementations or broaden unwind exemptions to subjects and renderers.

Sequential diagnostic behavior remains unchanged. Concurrent failures are ordered by insertion at the capture root. Message collection takes per-node snapshots,         
preserving local-before-ancestor order. Do not promise a transaction covering the entire ancestor graph.

Add Send + Sync to the sealed Mode contract, reflecting its existing marker implementations.

### Async operations

Borrowed mapping: Change the mapper input of map_async to Actual<'t, T>. The current elided lifetime rejects a mapper that carries a borrowed Actual into its future.    
This failure was reproduced against the actual crate on Rust 1.89.

Preserve lazy mapper invocation. Support both computing an owned result and returning a reference tied to the original borrowed subject.

Consuming async function assertions: Separate the callable from Actual before constructing the returned future. Store an owned callable or a borrowed-input marker       
alongside continuation state.

Preserve the existing timing:

- Caller location is captured when the method is called.
- Tracking, borrowed-input rejection, invocation, and polling begin on first poll.
- Panicked futures are never repolled.
- Existing output-drop and cancellation boundaries remain unchanged.

This allows the assertion future to be Send when its owned callable, produced future, and retained settings are Send. The callable need not also implement Sync. Apply   
this to both async function assertions and assert_that_panic_by_async.

Carry the presentation type through Reqwest extraction and preserve its existing synchronous ownership checks and caller locations.

Do not add unconditional + Send requirements to async methods or their outputs.

## 4. Validation and explicit limits

Add focused tests beside the owning implementation, using assertr assertions.

Compile-time coverage

- Default roots in both modes, with owned and borrowed subjects.
- Moving a chain with a Send-only renderer, while rejecting shared operations that require that renderer to be Sync.
- Send futures for mapping, derivation, async function assertions, and Reqwest extraction.
- Send-only callable and future captures, including Cell-based examples.
- Thread-safe custom adapters, non-Clone adapters, and adapters whose error type is non-Send.
- Local adapters and renderers remaining usable locally and preventing transfer where appropriate.
- Presentation replacement and reset removing only the replaced component’s restriction.
- NoRenderer availability across the generalized implementations.
- An owned derived child being Send even when an unrelated ancestor subject is not thread-safe.
- Borrowed async mapping succeeding within its subject’s lifetime and failing when a returned reference would escape.

Runtime coverage

- Tokio tasks with at least one genuine suspension, including assertions before and after suspension.
- Scoped concurrent children propagating exact counts and captured failures.
- Capture under a local panic adapter, with thread-safe capture children and the adapter kept alive through completion.
- Reentrant conversions, rendering, and presentation without deadlock.
- Capture isolation, metadata inheritance, adapter sharing and replacement, poison recovery, and counter saturation.
- Existing exact diagnostics, caller-location, lazy-execution, and unwind tests.

Use in-memory HTTP responses and existing fixtures. Add only Tokio’s development-time multithreaded runtime feature where needed.

Boundaries to document

- With the existing Actual<T> representation, moving a chain still requires T: Send + Sync, even for an owned subject. The async callable preparation improves           
  consuming operations without changing that representation.

- derive_async borrows its parent, so sending its future requires a shareable parent. A borrowed child does not become 'static.
- A Send future can produce a non-Send output. In particular, an erased PanicValue must be inspected or extracted into a suitable typed value before being retained      
  across another await or returned from a spawned task. Tokio’s spawning requirements (https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)

- Attached failure builders retain their existing synchronous construction contract. Do not promise that they can cross awaits.
- Allocation-only Send support, async capture APIs, borrowed presentation adapters, and ownership-specific subject storage remain separate changes.

## 5. Delivery, compatibility, and performance

Keep the initial maintenance batch small: document current local execution and fix the borrowed-mapper lifetime. Include compiled examples of assertions created after   
an await and local polling through LocalSet.

Deliver the presentation and record redesign as a separate architectural batch. Update those examples and all relevant knowledge documents to describe the final         
conditional support.

Before implementation, record a baseline for passing roots, repeated assertions, derived chains, and capture. Afterward, compare runtime, allocations attributable to    
Assertr, and representative compile time and binary size. Passing tracking must not acquire diagnostic mutexes. Default presentation must remain allocation-free. Do     
not add a benchmark framework or synchronization dependency for this work.

Run focused tests first, then just verify, Rust 1.89 checks, allocation-only and embedded checks, and the SemVer audit.

Assess compatibility against v0.7.1, not an intermediate checkout. In particular, do not label the changed with_panic_presentation return type as a release break        
because that method was absent from v0.7.1. Preserve published call shapes where possible and consolidate any actual net breaks with existing changelog entries.

Regenerate README from landing-page rustdoc. Preserve unrelated staged and unstaged changes, and do not bump versions or publish anything.                               
                                                                                                                                                                           
