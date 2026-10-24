# Changelog

All notable changes to Cadence are documented here. The format loosely
follows Keep a Changelog; releases use git tags.

## [Unreleased]

### Added

- `cadence-interchange` crate: Microsoft Project XML (MSPDI) and MPX import
  and export for activities, durations, constraints, relationships with lag,
  resources with capacities, assignments and baselines, with an in-crate XML
  reader (no new dependency)
- CLI `export-xml`, `import-xml`, `export-mpx` and `import-mpx` commands
- Resource dictionary with kinds and capacities (units per working day) and
  resource assignments on activities
- Resource usage histograms and over-allocation detection per working day
- Greedy resource leveling that preserves dependencies and constraint bounds
- Resources view in the UI: dictionary table, editing, assignment toggles,
  per-day usage against capacity, leveling preview and undoable apply
- Multiple project baselines with per-activity snapshots and variance data
- Baselines view in the UI: capture/delete baselines, a per-activity variance
  table against the selected baseline, and an earned-value panel driven by an
  editable status date
- Earned value metrics (PV, EV, AC, SV, CV, SPI, CPI) in minor units and basis
  points
- SQLite schema v3: resource capacities, resources, activity assignments,
  baselines, baseline snapshots and activity cost columns, with v1→v3 and
  v2→v3 migrations
- CSV activity table interchange: one row per activity with constraints,
  predecessor links, resource assignments, progress and actual dates; CLI
  `export-csv` / `import-csv`
- Cadence JSON interchange: a lossless, documented, versioned JSON schema for
  the whole project (WBS tree, named calendars, activities with membership,
  constraints, relationships, resources, assignments, baselines and progress),
  written and read by an in-crate JSON parser; CLI `export-json` /
  `import-json`
- Markdown project reports: schedule table, resource usage, earned-value
  metrics and baseline list; `cadence report <file> [out]`
- Keyboard map in the GUI: Ctrl+Z/Ctrl+Y undo-redo, Ctrl+1..4 view switching
- WCAG AA contrast for theme text, verified by tests on both themes
- Performance gates: 10k committed resource days and 100-activity leveling
  time-budgeted in CI
- `docs/usage.md` user guide and the `bridge_report` example
- PDF printing: a dependency-free PDF writer (`cadence-core/src/pdf.rs`),
  `cadence print <file> <out.pdf>`, and a GUI **Print report…** button backed
  by a native save dialog (`rfd`)
- v0.1.0 release: tag, GitHub Release with binaries for Linux/Windows/macOS
  and SHA-256 checksums; crash-report and update-check stubs; ROADMAP.md,
  pull-request template, release workflow, brand and research documents,
  CONTRIBUTING.md
- Property tests for resource leveling (delays only, delays match the
  calendar, conflict-free results are reachable through constraints) and for
  all four interchange formats (arbitrary projects round-trip within each
  format's documented scope)
- GUI **Save report** button writing the Markdown report next to the working
  directory

### Fixed

- MSPDI round-tripped sub-percent progress as zero because `PercentComplete`
  was truncated; it is now rounded to the nearest whole percent
- Progress updating: percent complete (basis points), budget at completion,
  actual cost and recorded actual start/finish dates, editable through
  undoable commands and persisted in schema v4 (with v1, v2 and v3
  migrations); Gantt bars draw a progress overlay
- Phase-3 QC gate recorded in `docs/qc/phase-3.md`
- Cargo workspace: `cadence-core` (domain + CPM engine), `cadence-commands`
  (typed commands, undo/redo), `cadence-persistence`, `cadence-app` (CLI)
- CPM engine: forward/backward pass, FS/SS/FF/SF with lag/lead, total and
  free float, critical set, cycle detection with path diagnostics
- egui/eframe app shell with design tokens and light/dark theme switching
- CLI `gui [file]` command opening a window over the demo or a project file
- Phase-1 QC gate recorded in `docs/qc/phase-1.md`
- SQLite project files: relational schema versioned with `PRAGMA user_version`,
  transactional save/load, validation of dangling references, in-place
  migrations, and CLI `save <file>` / `load <file>` commands
- Working-day calendar (Mon–Fri + holiday exceptions)
- Multiple named calendars with per-activity selection and a documented
  cross-calendar scheduling convention
- WBS hierarchy: single-rooted tree, cycle-safe reparenting, delete guards,
  undoable create/rename/move/delete/assign commands
- Command layer with concrete create/delete/rename/duration commands and
  undo/redo
- `cadence` binary: demo project rendered as an aligned schedule table
- GitHub Actions CI (fmt, clippy `-D warnings`, full tests)
- Project documentation: brief, scope, inspiration record, traceability,
  decisions, progress, parity tracker
