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
| R14 | Design system, light/dark, keyboard, a11y | `cadence-ui-egui` (design tokens, themes, keyboard shortcuts, pointer-free link form, AccessKit summary) | `docs/qc/phase-2.md`; `ACCESSIBILITY.md` records the keyboard pass, AccessKit path, WCAG AA contrast tests and remaining toolkit limits | done |
| R15 | Resources: dictionary, assignments, histograms, leveling | `cadence-core/src/model.rs` (`Resource`), `engine.rs` (`resource_histogram`, `resource_conflicts`, `level_resources`), `cadence-commands`, `cadence-persistence` (schema v3), `cadence-ui-egui` Resources view | engine tests in `engine.rs`, command tests in `cadence-commands`, persistence round-trip test, `docs/qc/phase-3.md` | done |
| R16 | Baselines + variance | `cadence-core/src/model.rs` (`Baseline`), `engine.rs` (`capture_baseline`), `cadence-commands`, `cadence-persistence`, `cadence-ui-egui` Baselines view | engine, command and persistence tests; variance table and EVM panel in the Baselines view | done |
| R17 | Progress updating + EVM | `cadence-core/src/model.rs` (cost/percent/actual-date fields), `engine.rs` (`evm_metrics`), `cadence-commands` (`SetActivityProgress`, `SetActivityActualCost`, `SetActivityBudget`, `SetActivityActualDates`), `cadence-persistence` (schema v4), `cadence-ui-egui` (inspector progress section, Gantt progress overlay) | command tests (undo/redo, range and date validation), persistence round-trip and v3→v4 migration tests, EVM unit tests | done |
| R18 | Open interchange (JSON/CSV/XML), printing/PDF | `cadence-interchange` (MSPDI, MPX, CSV, JSON), CLI `export-xml`/`import-xml`/`export-mpx`/`import-mpx`/`export-csv`/`import-csv`/`export-json`/`import-json`; Markdown and PDF reports in `cadence-core/src/report.rs` + `cadence-core/src/pdf.rs`, CLI `cadence report`/`cadence print`, GUI **Save report**/**Print report…** | round-trip and hand-written-document tests in all four format modules, CSV-vs-MSPDI and JSON-vs-MSPDI equivalence tests, report and PDF-writer unit tests | done |

| R19 | Open-source project documents (conduct, security, accessibility, vision, governance, contributors) | root: `CODE_OF_CONDUCT.md`, `SECURITY.md`, `ACCESSIBILITY.md`, `VISION.md`, `GOVERNANCE.md`, `CONTRIBUTORS.md` | reviewed in `docs/qc/project-health.md` | done |
| R20 | Contribution surface: issue forms, pull request checklist, labels | `.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md` | template files + workflow runs | done |
| R21 | Security in CI: secret scanning, dependency licence/advisory checks, dependency review, SAST, SBOM | `.github/workflows/security.yml`, `.github/workflows/release.yml`, `deny.toml`, `.github/dependabot.yml` | security workflow runs on `main` and on the pull request | done |
| R22 | Protected default branch with required status checks | repository settings | state recorded in `docs/qc/project-health.md` | done |
| R23 | Reproducible dependency graph: current `Cargo.lock`, regenerated in CI | `Cargo.lock`, `.github/workflows/lockfile.yml` | lockfile workflow run + freshness check | in progress (D19) |
| R24 | Machine-readable schedule output (--json) for scripts and assistive technology | `cadence-interchange/src/schedule.rs`, CLI `cadence [--json]`, `cadence load <file> --json` | unit tests parsing the document in `schedule.rs`; documented in `docs/usage.md` | done |
| R25 | Crash reporting with consent: local crash log, preview, pre-filled GitHub issue | `cadence-app/src/crash_report.rs` (panic hook), `cadence-ui-egui/src/crash.rs` (preview/URL), GUI **Crash report** and **Report a bug…** | crash module unit tests (title, body, URL encoding); dialog interactive-only; documented in `README.md` and `docs/usage.md` | done |
| R26 | Opt-in update check against GitHub Releases, fail-soft offline, cached | `cadence-ui-egui/src/update_check.rs`, CLI `cadence check-updates`, GUI **Updates** window | version-comparison, malformed-tag and cache-serialisation tests; network path opt-in and interactive; documented in `README.md`, `docs/usage.md`, D22 | done |

When a slice lands, this table and `docs/parity.md` are updated in the same
commit.

## Future expansion (candidates — no requirement ID yet)

These are **not** in scope and carry no requirement ID until the owner approves
the research note (guide §2.3). They are listed so the note is traceable.

| Candidate | Research note | Status |
|---|---|---|
| Programme audit tool (audit a construction programme inside Cadence) | `docs/research/schedule-audit.md` | draft — human gate pending |
| Optional AI integration (document understanding, proposals, advice) | `docs/research/ai-integration.md` | draft — human gate pending |
| Lookahead / field planning (Aphex-inspired) | `docs/research/aphex-integration.md` | draft — human gate pending |
