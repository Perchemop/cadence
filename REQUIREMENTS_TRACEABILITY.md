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
| R11 | Constraints (ASAP/ALAP, dates) | `cadence-core/src/model.rs` (`Constraint`), `engine.rs` (constraint-aware passes), `cadence-commands` (`SetActivityConstraint`), persistence schema v2 | engine constraint tests, command undo test, persistence round-trip and v1→v2 migration test | done |
| R12 | Property-based engine tests | `crates/core/tests/properties.rs` (proptest) | 5 property tests covering calendar invariants, DAG scheduling, ALAP, constraints | done |
| R13 | Interactive Gantt + activity table | `cadence-ui-egui` (Gantt view, sortable table, inspector, dependency drag) | Phase-2 QC gate in `docs/qc/phase-2.md` | done |
| R14 | Design system, light/dark, keyboard, a11y | `cadence-ui-egui` (design tokens, themes) | `docs/qc/phase-2.md`; keyboard and a11y audit still pending | in progress |
| R15 | Resources: dictionary, assignments, histograms, leveling | `cadence-core/src/model.rs` (`Resource`), `engine.rs` (`resource_histogram`, `resource_conflicts`, `level_resources`), `cadence-commands`, `cadence-persistence` (schema v3), `cadence-ui-egui` Resources view | engine tests in `engine.rs`, command tests in `cadence-commands`, persistence round-trip test, `docs/qc/phase-3.md` | done |
| R16 | Baselines + variance | `cadence-core/src/model.rs` (`Baseline`), `engine.rs` (`capture_baseline`), `cadence-commands`, `cadence-persistence`, `cadence-ui-egui` Baselines view | engine, command and persistence tests; variance table and EVM panel in the Baselines view | done |
| R17 | Progress updating + EVM | `cadence-core/src/model.rs` (cost/percent/actual-date fields), `engine.rs` (`evm_metrics`), `cadence-commands` (`SetActivityProgress`, `SetActivityActualCost`, `SetActivityBudget`, `SetActivityActualDates`), `cadence-persistence` (schema v4), `cadence-ui-egui` (inspector progress section, Gantt progress overlay) | command tests (undo/redo, range and date validation), persistence round-trip and v3→v4 migration tests, EVM unit tests | done |
| R18 | Open interchange (JSON/CSV/XML), printing/PDF | `cadence-interchange` (MSPDI, MPX, CSV), CLI `export-xml`/`import-xml`/`export-mpx`/`import-mpx`/`export-csv`/`import-csv` | round-trip and hand-written-document tests in `interchange::mspdi`, `interchange::mpx` and `interchange::csv`, plus a CSV-vs-MSPDI equivalence test; JSON and printing pending | in progress |

When a slice lands, this table and `docs/parity.md` are updated in the same
commit.
