# Changelog

All notable changes to Cadence are documented here. The format loosely
follows Keep a Changelog; releases use git tags.

**Token usage metrics (cumulative):** tracked in `docs/tokens.json` (the
CI-managed ledger, guide §12.1). Current seed values: ~1.2M total since start,
~45K in the 0.12.0 cycle. The ledger's CI update on each tag is queued
(`ROADMAP.md`).

## [0.12.1] — 2026-11-07

### Changed
- Shell overhaul: the top bar is now three visually distinct zones — a
  **File / Edit / View / Window / Help** menu bar, the view-tab strip, and a
  tool cluster — and the status message lives only in the bottom status bar.
- Menu taxonomy: a new **Window** menu owns the theme toggle and the
  split-Gantt switch; the **View** menu is pure view switching (radio items
  mirroring `Ctrl+1`–`Ctrl+9`); the **File** menu gains **Close (Ctrl+W)**,
  **Exit**, and a **Recent files** list of the ten most recent projects.
- Toolbar polish: focus rings on the tabs and Undo/Redo, an accent-tinted
  primary style for Undo/Redo, a zoom slider in the status bar, and a shared
  8 px horizontal / 4 px vertical spacing grid.
- The activity inspector groups its fields under accent **section headers**
  (Basic info, Calendar, Field planning, Progress, Relationships).
- The row context menu gains **Show in Gantt** and **Add to selection**.
- The F1 shortcuts window documents `Ctrl+W`, the multi-selection gestures and
  `Escape`; the release-notes window carries the current release's notes.

### Fixed
- `Ctrl+W` is now bound (it was advertised in the menu but did nothing).
- The release-notes dialog no longer prints the placeholder text with its
  spaces collapsed into long runs.

### Docs
- Reconciled the records with `guide.md` §12.1 and §II.20.4: the addendum is
  archived (guide is the single source of truth), the land-on-`main`-via-PR
  rule is recorded in the agent instructions, and the suite standards are
  adopted in `DECISIONS.md` D27 with the open deviations tracked in
  `ROADMAP.md`. Added the `docs/tokens.json` ledger seed.

## [Unreleased]

### Added
- Iconography (V1/V16): Phosphor Icons (MIT, via `egui-phosphor`) on the menu
  bar, view tabs, tool cluster and inspector section headers, each beside its
  text label so meaning is never icon-only. Font and licence in
  `ATTRIBUTION.md`.
- Calendar date picker (V12): `egui_extras::DatePickerButton` beside every
  editable date field, which stays typeable (`YYYY-MM-DD`) with the inline
  invalid-date hint. The 0.35 button works on `jiff::civil::Date`; a small
  `jiff` dependency is added and dates convert to/from `time::Date` at the field
  boundary (D28).
- WBS expand/collapse (V7): `+`/`-` toggles in the outline's first column; a
  collapsed node hides its descendants in both the table and the Gantt. UI-only
  state, no project mutation.
- Drag-to-move / resize Gantt bars (V8): a hit-tested drag issues the undoable
  `RescheduleActivity` command; bars pinned by a mandatory constraint are
  refused with a status message.
- Progress and status-date lines (V8) on the Gantt chart.
- Mandatory constraints: `Constraint::MustStartOn` / `MustFinishOn` in the model
  and engine (pinning early and late dates, so total float is zero), MSPDI
  import/export codes 8/9, the undoable `SetMandatoryConstraint` command with
  tests, and an eight-kind constraint combo in the inspector.

### Changed
- Two-tier Gantt timescale (V6): the header is drawn from the shared
  `cadence_core::engine::timescale_bands` helper — a coarse
  (month/quarter/year) band over a fine (day/week) band — replacing the earlier
  inline tick loop.
- Reconciled with `guide.md` 2026-11-09: adopted **II.30** (Rust GUI/UX — stack
  matching, tokens, a11y-as-code, evidence) in D28 and recorded the GUI stack in
  `docs/design/gui-stack.md`.

### Fixed
- Opening a project file no longer silently discards unsaved changes
  (2026-11-07 audit F1): File ▸ Open…, Ctrl+O and the recent-files list route
  through the Save / Discard / Cancel dialog when the project is dirty, and the
  window cannot be closed behind that dialog while edits remain.
- Opening a project clears the previous project's selection and transient edit
  state (F5): a stale multi-selection can no longer pre-arm "Delete N selected"
  for rows the user never picked, and leftover diary, lag, link-form, leveling
  and reschedule-preview state no longer survives the switch.
- The per-resource assignment-units field is editable again (F2): the buffer is
  no longer overwritten from the model while the field is focused, so
  `SetAssignmentUnits` can actually fire; invalid input is reported instead of
  being silently ignored.
- Production ▸ Record reports rejected commands (F3), and Diary ▸ Add note
  distinguishes a blank activity id from a non-numeric one (F4), surfacing the
  command's own error text instead of blaming the note content.
- Dialog keyboard: global shortcuts are gated behind modals, Escape dismisses
  the front dialog, Enter confirms primary actions (F13).
- Release-notes dialog now reads token metrics from `docs/tokens.json`, corrects
  zoom-slider claim, spaces, and recent-files wording (F18).
- Update dialog no longer shows duplicate "Update now" button (F15).
- Crash dialog shows honest empty state, caps issue-URL size, and clears log
  only after dismissal (F16).
- Split-Gantt toggle shows resize hint (F21).
- Confirm deletes for resources, baselines, diary notes and production entries
  (F22): each destructive delete now routes through a confirm dialog with
  Escape/Cancel/Enter support and frozen selection ids.
- Non-working day shading on the Gantt canvas (V5): weekends and holidays are
  shaded from the assigned calendar, giving the chart its professional texture.
- Token colours on canvas and grids (V9/F12): all `Color32::YELLOW/GRAY/WHITE`
  literals replaced with `design::palette` tokens; `variance_color` fallback
  uses the grid token.
- Monospace numerics in every grid (V10/F20): dates, durations, counts, costs
  and performance indices render in Inter tabular figures across Resources,
  Baselines, Calendars, Diary and Production views.
- Status bar wraps instead of clipping (V2/F14): `horizontal_wrapped` layout
  with a zoom tooltip showing the Gantt percentage and Ctrl+wheel hint.
- Mandatory-constraint rounding (D29): `MustFinishOn` on a non-working day
  now rounds backward in both the early and the late pass (previously the
  early pass rounded forward, which could leave phantom float); mandatory
  constraints are now property-tested — `MustStartOn`/`MustFinishOn` added
  to the constraint property strategy — with a weekend-bound regression
  test.

## [0.12.0] — 2026-11-07

### Added
- Table multi-selection (S2-5): ctrl/cmd-click toggles a row and shift-click
  selects the range from the anchor row; Escape drops the extra rows. A
  **Delete N selected** action in the inspector removes the whole set through
  the new atomic `DeleteActivities` command — one undo step restores the lot,
  relationships included.

## [0.11.0] — 2026-11-06

### Added
- Workbench UI polish: the status bar now shows the critical-activity count,
  the most recent baseline's earned-value metrics (`PV EV AC SV CV SPI CPI`) or
  `No baseline`, a save indicator that reads as a word (`✓ saved` /
  `● unsaved`), and the zoom level; every primary toolbar action carries a hover
  tooltip naming its shortcut; and the active sort column renders in the accent
  colour with its order in the tooltip. Metrics are recomputed on edit and
  undo/redo, not per frame, so the table and Gantt hot paths are unchanged.
- Gantt legibility: a 2 px today line with a shape marker at the top, and signed
  dependency lag labels (`+2d` / `-1d`) drawn on a small badge so they stay
  readable over the grid.
- Empty states: the Table, Baselines, Published snapshots and Diary views each
  state what the empty surface is for and the one action that fills it, in the
  product's own voice.
- Discoverability: an **F1: shortcuts** control in the status bar opens the
  keyboard reference from anywhere (`docs/design/ui-polish-audit.md`).
- Menu bar (S2-1): File/Edit/View/Help menus carry the primary actions, the
  view tabs move into a compact strip, and the status message lives only in
  the bottom bar.
- Table filter (S2-5): a filter field narrows the rows by activity name
  (case-insensitive) and a **Critical only** toggle shows just the critical
  path, with a `shown of total` counter while either is active. The filter
  never leaks into the Gantt or the Resources dictionary.
- Relationship type editing (S2-8): the inspector's relationship section now
  offers a type combo (FS/SS/FF/SF) issuing the new undoable
  `SetRelationshipType` command, alongside the existing lag editor and the
  confirmed delete.
- Diary note editing (S2-10): every note gains an **Edit** action that loads
  the text into the editor and saves through the undoable `UpdateDailyNote`
  command, so a note can be corrected after it was written.
- Row context menu (S1-1c): right-clicking a table row offers New below,
  Duplicate and Delete.
- Date fields (S2-4): every `YYYY-MM-DD` entry in the GUI goes through one
  helper that validates inline, so parse errors appear next to the field.
- Inspector completeness (S2-9): the activity inspector now carries the
  Location and WBS fields, which the model and commands already supported.
- Update awareness: the window title carries the app version, and the app
  checks for a newer release daily; the README explains the download-and-run
  install for people without Rust.

### Changed
- Leveling is undoable as one step (S2-15): applying levelled dates issues a
  single composite `SetConstraints` command, so a mid-batch failure can no
  longer leave a partially levelled state behind one undo entry.
- Destructive actions confirm consistently (S2-18): deleting an activity, a
  relationship, or applying levelled dates all use the same modal pattern.
- Every label+field pair in the inspectors is linked for screen readers via
  AccessKit `labelled_by` (S2-20).
- The theme toggle lives in one place (S2-22): the toolbar keeps it, the status
  bar no longer duplicates it.
- Saving a report opens a file dialog like printing one (S2-14), and
  Ctrl+wheel over the Gantt zooms without also scrolling the canvas (S2-16).

### Performance
- Resource conflicts and the day-usage histogram are computed once per edit
  and shared behind an `Arc`, instead of rescanning the assignments every
  frame (S2-11).
- The lookahead window is cached against the edit version and the chosen
  window, and the snapshot comparison is indexed by activity id, so it is
  linear rather than O(n²) per frame (S2-12).
- The critical-activity count is computed in `recompute`, not per frame
  (S2-13).

### Fixed
- Status bar: info and success messages now auto-clear after five seconds so
  they don't linger indefinitely; errors and warnings persist until dismissed
  or replaced (S2-3b).
- Move off the yanked `async-executor 1.14.0` pin (superseded by `1.14.1`) so
  the required dependency-advisory check stays green; the lockfile workflow now
  re-resolves this pin explicitly (`DECISIONS.md` D26).
- Error colours route through the design tokens (S3-1): schedule errors in the
  Table and Resources views, the earned-value status-date warning and the
  over-allocation counts now use the palette's critical token instead of raw
  reds.
- The nine view tabs carry tooltips (S3-7), the window asserts a minimum size
  so the toolbar cannot wrap into nonsense (S3-18), and `ACCESSIBILITY.md`
  documents the full `Ctrl+1`–`Ctrl+9` view range (S3-6).

## [0.10.0] — 2026-11-02

### Added
- Progress-driven rescheduling mode: `schedule_with_progress(project,
  data_date)` in `cadence-core`, the CLI `cadence reschedule <file>
  [--status-date YYYY-MM-DD] [--json]`, and a non-destructive preview in the
  GUI **Production** view. Completed activities stay at their recorded dates,
  work in progress resumes from the status date, and not-started work is
  scheduled from logic but never before it. The default schedule is unchanged,
  so variance and earned-value views stay honest (`DECISIONS.md` D24).
- Per-assignment resource units and a single resource cost rate: an assignment
  can consume a fraction or a multiple of one resource unit (stored sparsely in
  `Activity::assignment_units`), and a resource can carry a cost per unit per
  day (`Resource::cost_per_unit`). Over-allocation now compares committed units
  against capacity, so two half-unit assignments share a capacity-one resource;
  the report shows a planned resource-cost total. Persistence schema v10;
  `SetAssignmentUnits` and `SetResourceCostRate` commands; GUI editing in the
  **Resources** view (`DECISIONS.md` D11, D25).

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
