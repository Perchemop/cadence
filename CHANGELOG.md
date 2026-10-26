# Changelog

All notable changes to Cadence are documented here. The format loosely
follows Keep a Changelog; releases use git tags.

## [Unreleased]

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
