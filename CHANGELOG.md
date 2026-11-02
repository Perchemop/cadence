# Changelog

All notable changes to Cadence are documented here. The format loosely
follows Keep a Changelog; releases use git tags.

## [Unreleased]

### Added

## [0.9.0] — 2026-11-02

### Added
- Gantt dependency arrowheads and lag labels: dependency lines now render with
  arrowheads at the successor bar and an optional lag label (e.g. `+2d`, `-1d`),
  making link types and lags visible at a glance without hovering.
- Milestone diamonds for zero-duration activities: activities with duration zero
  render as diamond-shaped bars in the Gantt, matching the MS Project / P6
  convention for milestones.
- WBS summary bars: WBS parents show a summary bar spanning their children,
  rendered in a lighter shade so the hierarchy is visible in the Gantt.
- Two-tier timescale: the Gantt header now shows a coarse week/month scale above
  the day grid, and a today line marks the current date with a labelled marker.
- Baseline ghosts: baseline dates are overlaid as translucent bars behind the
  current schedule bars, so variance is visible without switching views.
- Authoring blockers and production view: a new **Production** view (`Ctrl+9`)
  with a dirty-guard prompt (unsaved changes warning on exit), and create/delete
  activity blockers that prevent invalid edits during a production session.
- Dirty guard: the application now warns when closing with unsaved changes and
  offers Save/Discard/Cancel.

### Changed
- Status colour semantics: status messages use more distinct colours for success/
  warning/error and the shortcuts dialog groups entries by category for easier
  scanning.
- Toolbar grouping: toolbar buttons are now grouped logically with separators
  for better visual hierarchy.

### Fixed
- Various UI polish issues found during end-to-end GUI audit.

## [0.3.0] — 2026-10-28

A programme audit tool, lookahead planning, and field planning — the first
three owner-approved expansions, plus the reference-product research that
grounded them.

### Added

- Programme audit tool (all five stages): 14 schedule-quality and health
  checks in `cadence-core::audit` (open ends with milestone exemption, leads,
  lags, relationship types, hard constraints, high/negative float, high
  duration, invalid dates, missing resources, missed tasks, baseline execution
  index, critical-path test, critical path length index); the GUI **Audit**
  view (`Ctrl+6`); `cadence audit <file> [--json] [--status-date]`; and the
  forensic comparison `cadence audit-compare <before> <after> [--json]`
  (two programmes or a baseline vs current) with a `kind: "comparison"`
  JSON document
- Lookahead planning: `cadence-core::lookahead` extracts the rolling
  short-interval window over the schedule, and the GUI **Lookahead** view
  (`Ctrl+7`) shows it with a configurable start date and 2/4/6/12-week length
- Field planning: activities carry an **owner**, a **promised** flag and a
  **delay reason** (undoable commands, inspector section, Lookahead columns)
- Capability inventories for Microsoft Project and Primavera P6
  (`docs/research/`), matching the Aphex research note

### Changed

- SQLite project files are schema version 5; every older file migrates in
  place (v1/v2/v3/v4 tested)
- The workspace version moved 0.1.1 -> 0.2.0 -> 0.3.0 across the two
  public-API-extending slices, per the semver gate
- Accessibility: keyboard-shortcuts window (`F1`), `Up`/`Down` table
  selection, keyboard Gantt zoom, pointer-free **Add relationship** form,
  scrollable inspector, `Status:`-prefixed status text, AccessKit summary for
  the Gantt canvas, and `!` markers on critical bars

### Fixed

- Update checks are opt-in and cached as designed; `cadence version` no
  longer claims updates are unwired (it reports the last cached check)


- Workspace version bumped to 0.3.0: the field-planning slice adds public
  fields to `Activity` and `LookaheadEntry`, which the semver gate treats as
  breaking for exhaustive construction; the bump matches the v0.3.x roadmap
  section
- Field planning (third lookahead slice): activities carry an **owner**, a
  **promised** flag and a **delay reason**; two undoable commands set them,
  the schema migrates to version 5 (v4 files migrate in place), Cadence JSON
  stays lossless, the inspector edits them and the Lookahead grid shows them
- Lookahead planning (first slices, Aphex-inspired): `cadence-core::lookahead`
  extracts the rolling short-interval window over the scheduled programme, and
  the GUI **Lookahead** view (`Ctrl+7`) shows it with a configurable start date
  and 2/4/6/12-week length; entries open the activity in the table. A pure
  view over the same schedule, never a second plan
- Capability inventories for the two reference products —
  `docs/research/ms-project-capabilities.md` and
  `docs/research/primavera-p6-capabilities.md` (original summaries, public
  URLs, and a shipped-vs-out-of-scope table for each) — so parity claims are
  traceable
- Workspace version bumped to 0.2.0: the audit's critical-path slice extends
  the public `cadence-core` API in breaking ways for exhaustive construction
  (new `AuditConfig` fields, new `CheckId` and `CheckCategory` variants), so
  the semver gate requires the pre-1.0 minor bump; the bump also matches the
  v0.2.0 milestone
- New `cadence-core` programme-audit module (`audit.rs`, first slice): twelve
  schedule-quality and health checks with configurable thresholds and
  per-activity/per-relationship findings — open ends, leads, lags, relationship
  types, hard constraints, high float, negative float, high duration, invalid
  dates, missing resources, missed tasks and the baseline execution index. Pure
  and deterministic; not yet wired to the CLI or GUI
- `cadence audit <file> [--json] [--status-date YYYY-MM-DD]` (second slice):
  prints the audit as plain text (stable check id, outcome in words, measured
  value, threshold, summary, findings, tally) or as a machine-readable
  `kind: "audit"` JSON document written by the in-crate JSON machinery
  (`cadence-interchange/src/audit.rs`); findings are data, so the command
  succeeds when the audit ran even when checks fail
- GUI **Audit** view (third slice): toolbar **Audit** button and `Ctrl+6`, a
  pass/fail/not-applicable tally, one block per check with findings, an
  optional status date with a **Run audit** button, and activity findings that
  jump to the activity in the table; the audit re-runs after edits and never
  per frame
- Forensic comparison (fifth slice): a pure `cadence-core::compare` module
  comparing two programmes (activities added/removed, duration, scheduled-date
  and constraint changes, relationships added/removed/changed/retyped,
  project-date shift) and a baseline against the current programme;
  `cadence audit-compare <before> <after> [--json]` prints the report as text
  or a machine-readable `kind: "comparison"` document
  (`cadence-interchange/src/compare.rs`). Matching is by id, so CSV-imported
  before-files (which renumber) show wholesale add/remove — documented
- Critical-path checks (fourth slice): `AUDIT-CPT` delays the last-finishing
  critical activity by 600 working days and expects the project finish to move
  by the same amount (`cadence-core` gained a public
  `schedule_with_minimum_starts` for the test), and `AUDIT-CPLI` computes the
  critical path length index against the most recent baseline's deadline; both
  report not-applicable when they cannot run honestly
- Research notes (now owner-approved 2026-10-27) for three future-expansion
  capabilities, with original behavioural summaries and public URLs:
  `docs/research/schedule-audit.md` (an in-app tool that audits a construction
  programme against public DCMA/GAO-style checks), `docs/research/ai-integration.md`
  (optional, off-by-default AI assistance that proposes reviewable changes) and
  `docs/research/aphex-integration.md` (short-interval/lookahead and field
  planning inspired by publicly documented Aphex behaviour)
- `docs/inspiration.md` records Aphex and the audit/AI reference products;
  `ROADMAP.md`, `docs/scope.md`, `VISION.md` and `docs/parity.md` list the
  candidates without committing to them
- Accessibility pass: a **Keyboard shortcuts** window (`F1` or the toolbar
  **Shortcuts** button), `Up`/`Down` activity selection in the Table view,
  keyboard Gantt zoom (`Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`) with visible zoom
  buttons, a pointer-free **Add relationship** form in the inspector (the
  drag-to-link gesture remains a shortcut), a scrollable inspector, and
  `Status:`-prefixed status text
- AccessKit summary for the custom-painted Gantt canvas, a text summary and
  legend above it, and a leading `!` on critical bars; the Table view is the
  documented per-activity screen-reader alternative
- `ACCESSIBILITY.md` now records the keyboard pass, the AccessKit behaviour and
  the remaining toolkit limits; the keyboard map in `docs/usage.md` covers the
  new shortcuts
- Machine-readable schedule output: `cadence --json` and
  `cadence load <file> --json` print the schedule as a JSON document
  (`cadence-interchange/src/schedule.rs`), written by the existing in-crate
  JSON machinery; a `kind` field distinguishes a schedule from an error
  document, stdout carries only the document, diagnostics for humans go to
  stderr as `ERROR <UTC timestamp> cadence: <message>`, and usage errors now
  exit with code 2 (the usage screen also documents the exit codes)
- `docs/usage.md` documents the `--json` forms, the schedule and error
  document shapes, and the exit codes
- GUI **Print Gantt…** button: captures the Gantt view through eframe's
  viewport screenshot command and writes it as a single A4 landscape image
  page in a PDF via the existing `cadence-core::pdf` writer (no new
  dependency)
- GUI crash reporting: a **Crash report** window previews the local crash log,
  accepts optional notes, and opens a pre-filled GitHub issue in the browser;
  **Report a bug…** uses the same consent-first flow, and nothing is sent
  automatically (no token is embedded in the binary)
- Opt-in update check against the public GitHub Releases API: the new
  `cadence check-updates` command and the GUI **Updates** window compare the
  latest release tag with the running version, link to the release page, cache
  a successful answer for six hours, and fail soft offline (`cadence version`
  uses only the cached answer and never reaches the network)

## [0.1.1] — 2026-10-24

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
- Keyboard map in the GUI: Ctrl+Z/Ctrl+Y undo-redo, Ctrl+1..5 view switching
  (Table, Gantt, Resources, Baselines, Calendars)
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
- Calendar management in the GUI (Calendars view) and a
  `SetCalendarDefinition` command for editing working days and holidays in
  place; per-activity calendar picker in the inspector
- Property tests for resource leveling (delays only, delays match the
  calendar, conflict-free results are reachable through constraints) and for
  all four interchange formats (arbitrary projects round-trip within each
  format's documented scope)
- GUI **Save report** button writing the Markdown report next to the working
  directory

Project health (opensource.guide review — see D18 and
[`docs/qc/project-health.md`](docs/qc/project-health.md)):

- `CODE_OF_CONDUCT.md`, `SECURITY.md` (private reporting, threat model,
  incident response), `ACCESSIBILITY.md` (target standard, limitations,
  dedicated report path), `VISION.md` (goals, non-goals, open questions),
  `GOVERNANCE.md` (roles, decision rule, commit access, metrics, boundaries)
  and `CONTRIBUTORS.md`
- `README.md` and `CONTRIBUTING.md` rewritten for a first-time reader: install
  and run, exact local test commands, review expectations, contribution paths,
  and the private channels for security and conduct reports
- Issue forms for bug, feature and accessibility reports, with a contact-links
  page; pull request template extended with an accessibility checklist for UI
  changes and a dependency-licence/secret-scanning line
- `Security` workflow: secret scanning over the full history, dependency
  licence/advisory/ban/source checks driven by `deny.toml`, dependency review
  on pull requests, CodeQL static analysis of the Rust sources and the workflow
  files, and a CycloneDX SBOM per run; the release workflow attaches an SBOM to
  each release
- Dependabot for cargo and GitHub Actions (grouped, labelled); every workflow
  action pinned to a commit SHA
- Lockfile workflow that regenerates `Cargo.lock` on a runner and uploads it
  (it runs on every pull request), because cargo is never run on the
  maintainer's machine; a freshness check fails any build whose lockfile lags
  the manifests (D19)
- `Cargo.lock` refreshed from that workflow, clearing a yanked `clipboard-win`
  pin that cargo could never have moved locally
- The four unmaintained-crate advisories from the GUI stack recorded as
  explicit, dated ignores in `deny.toml` (D20), and the fonts the GUI stack
  embeds (Ubuntu, Noto Emoji) recorded with their licences in
  `ATTRIBUTION.md` and allow-listed in the licence check
- Workspace-internal dependencies carry explicit versions, so the
  wildcard-dependency ban is satisfied by the manifest rather than relaxed
- Linux is built and tested on ARM64 (aarch64) as well as x86-64, in CI and in
  every release: the first release shipped no ARM Linux binary, and the
  maintainer's machine is aarch64
- Release hygiene (D21): the crate version is bumped in the same change as
  the tag that publishes it, and `SHA256SUMS.txt` lists plain asset names that
  `sha256sum -c` verifies directly — the first publishing of v0.1.1 had
  neither and was withdrawn and re-published
- `ATTRIBUTION.md` extended with the accepted licence classes, non-code asset
  policy, the terms of the tooling and models used to build the project, and
  the SBOM location
- Protected default branch with required status checks

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
