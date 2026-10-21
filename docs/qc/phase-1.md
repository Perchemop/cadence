# Phase 1 Quality Gate

Checklist for the Phase-1 scheduling engine slices. Status values: `pass`,
`fail`, `not applicable`.

## Correctness

- [x] CPM forward/backward pass and total/free float match concrete
      reference cases (FS/SS/FF/SF with lag/lead) — `engine.rs` unit tests.
- [x] Critical set equals activities with total float <= 0 and is ordered by
      early start — `engine.rs` tests.
- [x] Cycle detection reports a concrete path instead of hanging —
      `engine.rs` tests.
- [x] Duplicate identifiers and unknown references are rejected —
      `engine.rs` tests.
- [x] Working-day calendar arithmetic respects weekends and holidays —
      `model.rs` / `engine.rs` tests.
- [x] Named calendars and cross-calendar scheduling follow D6 —
      `calendar_set.rs` and mixed-calendar engine tests.
- [x] WBS invariants (single root, no cycles, delete guards) — `wbs.rs`
      and command tests.
- [x] Constraints (ASAP/ALAP, SNET/SNLT/FNET/FNLT) behave per D10 —
      engine and command tests.
- [x] Property-based invariants hold for arbitrary DAGs and calendar maths —
      `crates/core/tests/properties.rs` (proptest).
- [x] All mutations flow through the command layer with undo/redo —
      `commands` tests.

## Persistence

- [x] Versioned SQLite schema with in-place migration (v1 → v2) —
      `persistence` tests.
- [x] Save/load round-trips WBS, calendars, activities, relationships,
      constraints, and holidays — `persistence` tests.
- [x] Failed saves do not corrupt existing files (transactional) — by
      construction + migration test.

## Engineering

- [x] `cargo fmt --check --all` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes (89 tests + proptest cases).
- [x] CI green on `main` (latest run recorded in `PROGRESS.md`).
- [x] No committed `guide.md`; repository remains public and offline-capable.

## Open items carried into Phase 2

- Interactive Gantt + activity table (P2-1..P2-4).
- Performance baseline at 10k activities (P2-5).
- Resource dictionary, baselines, EVM, open interchange (Phases 3-5).
