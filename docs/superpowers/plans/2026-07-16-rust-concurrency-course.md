# Rust Concurrency Course Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `rust-concurrency` as the complete Semestre 3 concurrency course for Jeresoft Academy, aligned with RFC-0001.

**Architecture:** One Rust crate, one public module per concurrency primitive or technique, one `docs/NN-*.md` chapter per module, and parallel `examples/`, `tests/`, `benches/`, `diagrams/`, and `assets/` material. The course moves from safe shared-state primitives to atomics, memory ordering, deadlock reasoning, message passing, and advanced memory reclamation.

**Tech Stack:** Rust 2021 or newer, Cargo, `cargo fmt`, `cargo clippy`, unit tests, integration tests, doctests, Cargo benches, Markdown compatible with mdBook, Mermaid diagrams.

---

## Source Decisions

- [x] Re-read RFC-0001 sections before execution: `§1`, `§2`, `§10`, `§13`, `§14`, `§15`, `§16`, `§17`, `§20`.
- [x] Treat `rust-concurrency` as a course repository, not a generic crate.
- [x] Keep the course sequence fixed unless a future RFC changes it: mutex, rwlock, atomics, arc, memory ordering, deadlocks, channels, lock-free structures, hazard pointers, epoch GC.
- [x] Keep concurrency primitives, invariants, failure modes, and memory reclamation canonical here.
- [x] Keep async runtime details in `rust-async` unless they are necessary for comparison.
- [x] Avoid creating content for future courses inside this repo.

## Repository Foundation

### Task 1: Establish Repository Identity

**Files:**
- Create: `README.md`
- Create: `AGENTS.md`
- Create: `ROADMAP.md`
- Create: `LICENSE-MIT`
- Create: `LICENSE-APACHE`
- Create: `LICENSE-CC-BY-SA-4.0.md`

- [x] Create `README.md` with the course purpose, its place in Semestre 3, how to navigate `docs/`, `src/`, `examples/`, `tests/`, `benches/`, and `diagrams/`.
- [x] State in `README.md` that this repo teaches concurrency in Rust as part of Jeresoft Academy RFC-0001.
- [x] Add the planned chapter table in `README.md` with status columns: `planned`, `draft`, `implemented`, `tested`, `benchmarked`, `reviewed`, `published`.
- [x] Create `AGENTS.md` from RFC-0001 §20, instantiated for `colección = camino troncal / Semestre 3` and `tema = concurrencia en Rust`.
- [x] Create `ROADMAP.md` with the ten chapters and the no-deadlines project philosophy from RFC-0001 §1.
- [x] Add dual licensing: MIT OR Apache-2.0 for code, CC BY-SA 4.0 for educational content.
- [x] Run `git status --short` and confirm only intentional foundation files are pending.
- [x] Commit: `chore: establish concurrency course foundation`.

### Task 2: Create Cargo Crate Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/mutex.rs`
- Create: `src/rwlock.rs`
- Create: `src/atomics.rs`
- Create: `src/arc.rs`
- Create: `src/memory_ordering.rs`
- Create: `src/deadlocks.rs`
- Create: `src/channels.rs`
- Create: `src/lock_free.rs`
- Create: `src/hazard_pointers.rs`
- Create: `src/epoch_gc.rs`

- [x] Create `Cargo.toml` with package name `rust-concurrency`, edition, license expression `MIT OR Apache-2.0`, repository URL, and no unnecessary dependencies.
- [x] Create `src/lib.rs` with crate-level documentation explaining the course and public `pub mod` declarations for all ten topics.
- [x] Create one empty module file per topic with a module-level doc-comment describing the learning goal.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`.
- [x] Run `cargo test`.
- [x] Commit as part of initial foundation: `chore: establish concurrency course foundation`.

### Task 3: Create Course Directory Layout

**Files and directories:**
- Create: `docs/`
- Create: `examples/`
- Create: `examples/soluciones/`
- Create: `tests/`
- Create: `benches/`
- Create: `diagrams/`
- Create: `assets/`

- [x] Create the standard RFC-0001 §15 directories.
- [x] Add `docs/SUMMARY.md` listing the ten chapters in order.
- [x] Add `diagrams/README.md` explaining Mermaid-first diagram policy.
- [x] Add `examples/README.md` explaining basic, intermediate, advanced, and real-case examples.
- [x] Add `tests/README.md` explaining integration-test naming by topic and deterministic concurrency testing.
- [x] Add `benches/README.md` explaining that benchmarks validate concurrency claims, not vanity performance.
- [x] Commit as part of initial foundation: `chore: establish concurrency course foundation`.

### Task 4: Add CI and Quality Gates

**Files:**
- Create: `.github/workflows/ci.yml`

- [x] Add CI jobs for `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo test --doc`.
- [x] Keep the initial workflow dependency-free unless a tool is justified in `README.md`.
- [x] Commit as part of initial foundation: `chore: establish concurrency course foundation`.

## Chapter Production Pipeline

For each chapter, complete the following checklist before moving to the next topic.

- [ ] Create `docs/NN-title.md` using RFC-0001 §16 exactly.
- [ ] Fill metadata: course, chapter number, prerequisites, code path, video status, site lesson status.
- [ ] Write `Introducción`: what the reader learns and what they already need.
- [ ] Write `Motivación`: real problem first, formalism second.
- [ ] Write `Teoría / Historia`: origin, historical problem, and why the primitive or technique exists.
- [ ] Write `Teoría / Fundamentos`: invariants, guarantees, failure modes, memory model assumptions, and mental model.
- [ ] Write `Teoría / Casos de uso`: real systems where the topic is useful.
- [ ] Write `Teoría / Ventajas y limitaciones`: honest tradeoffs.
- [ ] Write `Teoría / Comparación con alternativas`: when not to use it.
- [ ] Add at least one Mermaid diagram in `diagrams/`.
- [ ] Write complexity, contention, progress, or synchronization-cost table for every public operation.
- [ ] State whether interactive visualization applies; if not, justify in one line.
- [ ] Implement the public API or educational model in `src/<module>.rs`.
- [ ] Add doc-comments with examples and complexity/progress notes for all public items.
- [ ] Add unit tests inside the module for invariants and edge cases.
- [ ] Add integration tests in `tests/<module>_test.rs`.
- [ ] Add doctests through examples in public documentation.
- [ ] Add benchmarks in `benches/<module>_bench.rs` when the topic has meaningful performance or contention claims to measure.
- [ ] Add examples in `examples/<module>_basic.rs`, `examples/<module>_intermediate.rs`, `examples/<module>_advanced.rs`, and `examples/<module>_real_case.rs` when applicable.
- [ ] Add four to eight exercises in the chapter, spanning levels 1 to 4.
- [ ] Add solutions for levels 1 to 3 in `examples/soluciones/`.
- [ ] Add a level 4 discussion of approaches and tradeoffs.
- [ ] Add references: primary source where possible, canonical book, Rust documentation when relevant.
- [ ] Run `cargo fmt`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test --all-targets`.
- [ ] Run `cargo test --doc`.
- [ ] Update `README.md` and `ROADMAP.md` status for the chapter.
- [ ] Commit the chapter with `feat: add <topic> chapter`.

## Course Chapters

### Task 5: Mutex

**Files:**
- Create: `docs/01-mutex.md`
- Modify: `src/mutex.rs`
- Create: `tests/mutex_test.rs`
- Create: `benches/mutex_bench.rs`
- Create: `diagrams/01-mutex.mmd`
- Create: `examples/mutex_basic.rs`
- Create: `examples/mutex_intermediate.rs`
- Create: `examples/mutex_advanced.rs`
- Create: `examples/mutex_real_case.rs`

- [x] Teach mutual exclusion, critical sections, shared mutable state, lock guards, poisoning, contention, and scope-based unlock.
- [x] Explain why mutex comes first: it anchors invariants, ownership across threads, and the cost of serialization.
- [x] Compare against atomics, channels, RwLock, single-thread ownership, and actor-style designs.
- [x] Include tests for protected increments, guard-scoped unlock, poisoning recovery, non-reentrant behavior where observable, and contention-safe updates.
- [x] Include benchmarks for uncontended lock/unlock, contended increments, and a single-thread baseline.

### Task 6: RwLock

**Files:**
- Create: `docs/02-rwlock.md`
- Modify: `src/rwlock.rs`
- Create: `tests/rwlock_test.rs`
- Create: `benches/rwlock_bench.rs`
- Create: `diagrams/02-rwlock.mmd`
- Create: `examples/rwlock_basic.rs`
- Create: `examples/rwlock_intermediate.rs`
- Create: `examples/rwlock_advanced.rs`
- Create: `examples/rwlock_real_case.rs`

- [x] Teach shared readers, exclusive writers, read-heavy workloads, fairness, starvation, poisoning, and upgrade pitfalls.
- [x] Compare against Mutex, atomics, copy-on-write, and immutable snapshots.
- [x] Include tests for multiple readers, writer exclusion, write visibility, poisoning, and read-heavy correctness.
- [x] Include benchmarks contrasting read-heavy, write-heavy, and balanced workloads.

### Task 7: Atomics

**Files:**
- Create: `docs/03-atomics.md`
- Modify: `src/atomics.rs`
- Create: `tests/atomics_test.rs`
- Create: `benches/atomics_bench.rs`
- Create: `diagrams/03-atomics.mmd`
- Create: `examples/atomics_basic.rs`
- Create: `examples/atomics_intermediate.rs`
- Create: `examples/atomics_advanced.rs`
- Create: `examples/atomics_real_case.rs`

- [x] Teach atomic load/store, fetch-add, compare-exchange, counters, flags, CAS loops, and overflow policy.
- [x] Compare against Mutex, channels, thread-local aggregation, and sequential code.
- [x] Include tests for counters, flags, compare-exchange success/failure, and concurrent aggregation.
- [x] Include benchmarks for atomic increments, mutex-protected increments, and per-thread aggregation.

### Task 8: Arc

**Files:**
- Create: `docs/04-arc.md`
- Modify: `src/arc.rs`
- Create: `tests/arc_test.rs`
- Create: `benches/arc_bench.rs`
- Create: `diagrams/04-arc.mmd`
- Create: `examples/arc_basic.rs`
- Create: `examples/arc_intermediate.rs`
- Create: `examples/arc_advanced.rs`
- Create: `examples/arc_real_case.rs`

- [ ] Teach atomic reference counting, shared ownership, `Send`/`Sync` boundaries, `Arc<Mutex<T>>`, weak references, and cycle risks.
- [ ] Compare against `Rc`, borrowing, cloning data, channels, and scoped threads.
- [ ] Include tests for clone counts, weak upgrade/drop behavior, shared immutable data, and shared mutable data through a lock.
- [ ] Include benchmarks for clone/drop overhead and shared read access.

### Task 9: Memory Ordering

**Files:**
- Create: `docs/05-memory-ordering.md`
- Modify: `src/memory_ordering.rs`
- Create: `tests/memory_ordering_test.rs`
- Create: `benches/memory_ordering_bench.rs`
- Create: `diagrams/05-memory-ordering.mmd`
- Create: `examples/memory_ordering_basic.rs`
- Create: `examples/memory_ordering_intermediate.rs`
- Create: `examples/memory_ordering_advanced.rs`
- Create: `examples/memory_ordering_real_case.rs`

- [ ] Teach happens-before, compiler/CPU reordering, `Relaxed`, `Acquire`, `Release`, `AcqRel`, and `SeqCst`.
- [ ] Compare orderings by guarantees, cost, and failure modes.
- [ ] Include tests for publication with release/acquire, relaxed counters, and compare-exchange ordering.
- [ ] Include benchmarks comparing ordering choices only where the measurement is meaningful and documented.

### Task 10: Deadlocks

**Files:**
- Create: `docs/06-deadlocks.md`
- Modify: `src/deadlocks.rs`
- Create: `tests/deadlocks_test.rs`
- Create: `benches/deadlocks_bench.rs`
- Create: `diagrams/06-deadlocks.mmd`
- Create: `examples/deadlocks_basic.rs`
- Create: `examples/deadlocks_intermediate.rs`
- Create: `examples/deadlocks_advanced.rs`
- Create: `examples/deadlocks_real_case.rs`

- [ ] Teach Coffman conditions, wait-for graphs, lock ordering, timeouts, try-lock strategies, and API design that prevents circular waits.
- [ ] Compare prevention, avoidance, detection, and recovery.
- [ ] Include tests for lock-order validation, wait-for graph cycle detection, and safe transfer examples.
- [ ] Include benchmarks only if a prevention strategy has measurable overhead; otherwise explain why it does not apply.

### Task 11: Channels

**Files:**
- Create: `docs/07-channels.md`
- Modify: `src/channels.rs`
- Create: `tests/channels_test.rs`
- Create: `benches/channels_bench.rs`
- Create: `diagrams/07-channels.mmd`
- Create: `examples/channels_basic.rs`
- Create: `examples/channels_intermediate.rs`
- Create: `examples/channels_advanced.rs`
- Create: `examples/channels_real_case.rs`

- [ ] Teach message passing, ownership transfer, producers, consumers, bounded vs unbounded channels, backpressure, close semantics, and fan-out/fan-in.
- [ ] Compare against shared memory, Mutex, Arc, async channels, and actor-style designs.
- [ ] Include tests for send/receive order, channel close, multiple producers, worker pools, and bounded backpressure if implemented.
- [ ] Include benchmarks for throughput, bounded capacity effects, and shared-memory alternatives.

### Task 12: Lock-Free Structures

**Files:**
- Create: `docs/08-lock-free.md`
- Modify: `src/lock_free.rs`
- Create: `tests/lock_free_test.rs`
- Create: `benches/lock_free_bench.rs`
- Create: `diagrams/08-lock-free.mmd`
- Create: `examples/lock_free_basic.rs`
- Create: `examples/lock_free_intermediate.rs`
- Create: `examples/lock_free_advanced.rs`
- Create: `examples/lock_free_real_case.rs`

- [ ] Teach progress guarantees, CAS loops, ABA, retry behavior, false sharing, and why lock-free does not mean wait-free.
- [ ] Prefer safe Rust models first. Any `unsafe` requires `// SAFETY:` and written justification in the chapter.
- [ ] Compare against Mutex, channels, sharding, and wait-free designs.
- [ ] Include tests for stack/queue model correctness, concurrent push/pop behavior, and ABA demonstration if safely modelable.
- [ ] Include benchmarks contrasting lock-free model and locked baseline under contention.

### Task 13: Hazard Pointers

**Files:**
- Create: `docs/09-hazard-pointers.md`
- Modify: `src/hazard_pointers.rs`
- Create: `tests/hazard_pointers_test.rs`
- Create: `benches/hazard_pointers_bench.rs`
- Create: `diagrams/09-hazard-pointers.mmd`
- Create: `examples/hazard_pointers_basic.rs`
- Create: `examples/hazard_pointers_intermediate.rs`
- Create: `examples/hazard_pointers_advanced.rs`
- Create: `examples/hazard_pointers_real_case.rs`

- [ ] Teach safe memory reclamation, protected pointers, retired lists, scanning, delayed reclamation, and per-thread records.
- [ ] Compare against epoch GC, reference counting, garbage collection, and leaking memory.
- [ ] Include tests for protecting a node, retiring a node, delayed reclamation while protected, and reclamation after unprotect.
- [ ] Include benchmarks for scan cost, retire threshold, and protected read overhead.

### Task 14: Epoch GC

**Files:**
- Create: `docs/10-epoch-gc.md`
- Modify: `src/epoch_gc.rs`
- Create: `tests/epoch_gc_test.rs`
- Create: `benches/epoch_gc_bench.rs`
- Create: `diagrams/10-epoch-gc.mmd`
- Create: `examples/epoch_gc_basic.rs`
- Create: `examples/epoch_gc_intermediate.rs`
- Create: `examples/epoch_gc_advanced.rs`
- Create: `examples/epoch_gc_real_case.rs`

- [ ] Teach pinned epochs, global epoch advancement, retired objects, quiescent states, stalled participants, and memory retention.
- [ ] Compare against hazard pointers, reference counting, lock-based reclamation, and tracing GC.
- [ ] Include tests for pin/unpin, retirement, epoch advancement, blocked advancement, and eventual reclamation.
- [ ] Include benchmarks for pin overhead, retirement throughput, and delayed reclamation behavior.

## Cross-Course Integration

- [ ] Add references from Mutex, RwLock, atomics, Arc, and deadlocks to future `rust-operating-systems` chapters where the OS provides or motivates primitives.
- [ ] Add references from Memory Ordering, lock-free structures, hazard pointers, and epoch GC to future `rust-low-level` and `rust-performance` material.
- [ ] Add references from Channels to future `rust-async`, `rust-distributed-systems`, and `rust-messaging`.
- [ ] Add references from deadlocks, hazard pointers, and epoch GC to future `rust-database-internals`.
- [ ] Keep every cross-course link as a citation or "later in the path" note, not a full re-explanation.

## Final Course Completion

- [ ] Every public item has doc-comments with examples and complexity/progress notes.
- [ ] Every chapter has the eleven required RFC-0001 §14 sections; visualization is either present or explicitly justified as not applicable.
- [ ] Every chapter has four to eight exercises across levels 1 to 4.
- [ ] Every level 1 to 3 exercise has a solution in `examples/soluciones/`.
- [ ] Every level 4 exercise has a tradeoff discussion.
- [ ] Every topic has unit tests, integration tests, doctests, and benchmarks where meaningful.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test --all-targets` passes.
- [ ] `cargo test --doc` passes.
- [ ] Documentation builds or validates without broken links.
- [ ] README status table and ROADMAP match the actual repo state.
- [ ] GitHub topics and description match the course identity.
- [ ] The repo is ready for `academy-web` ingestion once the site content mechanism is decided.
- [ ] Final commit: `docs: mark concurrency course checklist complete`.

## Execution Options

1. **Subagent-Driven:** dispatch a fresh worker per major task or per chapter, then review each result before continuing.
2. **Inline Execution:** execute the checklist in this session with checkpoints after foundation, CI, and each chapter.
