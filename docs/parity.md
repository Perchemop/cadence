# Capability Parity Tracker

Living list of capabilities. Status values: `todo`, `in progress`, `done`.

## Phase 0 — Foundation

| Capability | Status | Notes |
|---|---|---|
| Cargo workspace + crate layout | done | core / commands / persistence / app |
| GitHub Actions CI (fmt, clippy, test) | done | `.github/workflows/ci.yml` |
| Typed command layer + undo/redo | done | `cadence-commands` |
| Product name locked | done | `docs/product-name.md` |
| SQLite project file persistence | done | relational schema, `PRAGMA user_version`, transactional save/load, CLI `save`/`load` |
| Minimal project model | done | Project, Activity, Relationship |

## Vertical slice (guide §12.4)

| Capability | Status | Notes |
|---|---|---|
| Create project | done | `Project::new`, command layer |
| Two activities | done | demo builds three |
| FS link with lag | done | plus SS/FF/SF and leads |
| Run CPM | done | forward/backward pass, float, critical set |
| Show dates in a table | done | aligned CLI table via `cadence` binary |

## Phase 1 — Scheduling engine

| Capability | Status | Notes |
|---|---|---|
| WBS hierarchy | done | tree with cycle-safe reparenting, undoable commands |
| SS / FF / SF relationships | done | signed lag/lead supported |
| Working-day calendar arithmetic | done | Mon–Fri pattern + holiday exceptions |
| Multiple calendars (working time, exceptions) | done | named calendars, per-activity selection; cross-calendar convention in `architecture.md` |
| Constraints (ASAP/ALAP, dates) | done | SNET/SNLT/FNET/FNLT date bounds, scheduled vs early/late dates, undoable command |
| Total float / free float / critical path | done | critical activity set ordered by early start; longest-path extraction later |
| Cycle detection & diagnostics | done | Kahn + DFS path, reported as `SchedError::Cycle` |
| Property-based engine tests | done | `crates/core/tests/properties.rs` (proptest) |
| Phase-1 QC gate | done | `docs/qc/phase-1.md` |
| Serde round-trip for model/schedule | done | project, link types, schedule |

## Phase 2 — Gantt & tables

| Capability | Status | Notes |
|---|---|---|
| Interactive Gantt | done | timeline, bars, arrows, zoom/pan, virtualised rows |
| Activity table | done | sortable columns, selection, inspector edits via commands |
| Dependency editing | done | drag-to-create links, lag editing, cycle prevention |
| Zoom / pan | done | Ctrl+wheel zoom in the Gantt view, scroll areas for panning |
| Phase-2 QC gate | done | `docs/qc/phase-2.md` |

## Phase 3 — Resources

| Capability | Status | Notes |
|---|---|---|
| Resource dictionary | done | `Resource` with `kind` and `capacity` (units per working day), unique case-insensitive names |
| Resource assignments | done | `Activity::resources`, undoable assign/unassign commands |
| Usage histograms | done | `engine::resource_histogram`: committed units per working day |
| Over-allocation detection | done | `engine::resource_conflicts` reports every day over capacity |
| Resource leveling | done | `engine::level_resources`, greedy earliest-conflict delay (D12) |
| Resource panel in the UI | done | dictionary, editing, assignments, usage, leveling preview/apply |

## Phase 4 — Baselines, progress & EVM

| Capability | Status | Notes |
|---|---|---|
| Multiple baselines | done | project-wide baselines with per-activity snapshots, undoable capture/remove/restore |
| Variance views | done | Baselines view: per-activity duration/start/finish variance against the selected baseline, coloured by sign |
| EVM metrics | done | `engine::evm_metrics`: PV, EV, AC, SV, CV, SPI, CPI shown against an editable status date (D13) |
| Progress updating (percent complete, actual dates) | done | inspector edits via undoable commands; Gantt bars show a progress overlay; schema v4 records actual dates |

## Phase 5 — Polish & interop

| Capability | Status | Notes |
|---|---|---|
| Microsoft Project XML (MSPDI) import/export | done | `cadence-interchange::mspdi` (D14) |
| MPX import/export | done | `cadence-interchange::mpx` (D14) |
| CSV activity table import/export | done | `cadence-interchange::csv`: one row per activity with constraints, links, assignments and progress; CLI `export-csv`/`import-csv` |
| JSON interchange | done | `cadence-interchange::json`: Cadence's own versioned schema and the only format that is lossless — it carries the WBS tree, named calendars and activity membership as well as everything the other formats do; CLI `export-json`/`import-json` |
| Machine-readable CLI schedule output (`--json`) | done | `cadence-interchange::schedule`, guide §21.5 tooling accessibility |
| WBS and named calendars in interchange | done for JSON | MSPDI, MPX and CSV cannot express them and import them empty (documented in `interchange::json`) |
| Printing / PDF and reports | done | Markdown reports (`cadence report`, **Save report**) and PDF printing (`cadence print`, **Print report…** via a native save dialog); D16 reversed |
| Calendar management in the UI | done | Calendars view: list, create, rename, delete, working-day and holiday editing, per-activity assignment |
| Keyboard map | in progress | Ctrl+Z/Y undo-redo and Ctrl+1..4 view switching added; full keyboard-only operation audit pending |
| Accessibility audit | in progress | WCAG AA contrast for theme text verified by tests (`accessibility_tests`); screen-reader (AccessKit) pass pending |
| Performance pass | in progress | 10k-activity schedule, 10k-day resource analysis and 200-activity leveling all time-budgeted in CI; smooth-interaction profiling pending |
| Documentation, example projects, first tagged release | in progress | `docs/usage.md` and the `bridge_report` example added; tagging is a release decision |
