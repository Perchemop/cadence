# Requirements Traceability

Every in-scope requirement (from `docs/scope.md`) maps to implementation and
tests. Status mirrors `docs/parity.md`; this file records *where* each
requirement lives.

| ID | Requirement | Implemented in | Verified by | Status |
|---|---|---|---|---|
| R1 | Project model (activities, relationships, settings) | `cadence-core/src/model.rs` | unit tests in `model.rs` | done |
| R2 | CPM engine: forward/backward pass, total/free float, critical path | `cadence-core/src/engine.rs` | unit tests in `engine.rs` (concrete dates) | done |
| R3 | Dependency logic FS/SS/FF/SF with lag/lead | `cadence-core/src/engine.rs` | link-type unit tests | done |
| R4 | Cycle detection with diagnostics | `cadence-core/src/engine.rs` (`SchedError::Cycle`) | cycle unit tests | done |
| R5 | Command layer, single mutation path, undo/redo | `cadence-commands/src/lib.rs` | commands unit tests | done |
| R6 | Working-day calendar arithmetic (Mon–Fri + holidays) | `cadence-core/src/model.rs` (`Calendar`) | calendar unit tests | done |
| R7 | CLI schedule table (headless engine proof) | `cadence-app/src/main.rs` | manual + demo | done |
| R8 | WBS hierarchy | `cadence-core/src/wbs.rs`, `cadence-commands` (WBS commands) | unit tests in `wbs.rs` + commands tests (undo, delete guards) | done |
| R9 | Multiple named calendars, per-activity selection | `cadence-core/src/calendar_set.rs`, engine per-activity resolution | `calendar_set.rs` tests, mixed-calendar engine tests, commands tests | done |
| R10 | SQLite project-file persistence | `cadence-persistence/src/lib.rs` (versioned schema, transactional save/load) | persistence unit tests (full round-trip, overwrite, missing file, newer schema, dangling references, invalid data) | done |
| R11 | Constraints (ASAP/ALAP, dates) | *pending slice P1-5* | — | todo |
| R12 | Property-based engine tests | *pending slice P1-6* | — | todo |
| R13 | Interactive Gantt + activity table | Phase 2 slices | — | todo |
| R14 | Design system, light/dark, keyboard, a11y | Phase 2/3 slices | QC checklists | todo |
| R15 | Resources: dictionary, assignments, histograms, leveling | Phase 3 slices | — | todo |
| R16 | Baselines + variance | Phase 4 slices | — | todo |
| R17 | Progress updating + EVM | Phase 4 slices | — | todo |
| R18 | Open interchange (JSON/CSV/XML), printing/PDF | Phase 5 slices | — | todo |

When a slice lands, this table and `docs/parity.md` are updated in the same
commit.
