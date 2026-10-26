# Progress

Source of truth for build state. "Done" means implemented **and** verified by
green CI (`main` branch).

## 2026-10-26 — Print the Gantt view (v0.2.0 suite infrastructure)

Decided as the next vertical slice (ROADMAP.md).

Done in this slice:
- New **Print Gantt…** toolbar button, enabled only in the Gantt view. It opens
  the same native save dialog as **Print report…** and writes a PDF named after
  the project (`<project>-gantt.pdf`).
- The Gantt is captured with eframe/egui's viewport screenshot command
  (`ViewportCommand::Screenshot`); the returned RGBA frame is converted to RGB
  and written as a single A4 landscape image page through the existing
  `cadence-core::pdf` writer. No new dependency was added.
- The capture is asynchronous: the button records the chosen path and requests
  the screenshot, and the next frame consumes the `Event::Screenshot` payload
  and writes the file. The status bar reports the path or the write error.
- `docs/usage.md` documents the button and the one-page output.
- Tests: a smoke test exercises the `Pdf::image_page` path used by the print
  action. The full screenshot path is interactive-only (needs a running
  viewport) and is recorded as such in the Phase-5 QC gate.
- CI green on `opencode/ci` (run 40289149574 and the follow-up fix run).

Known limit of this first implementation: the screenshot is the whole window,
not a crop of the Gantt timeline, and virtualised rows outside the viewport are
not included. A later slice can crop to the timeline and scroll-and-stitch the
full plan.

## 2026-10-26 — Crash reporting: preview and pre-filled GitHub issue (v0.2.0)

Done in this slice:

- New `cadence-ui-egui::crash` module: reads the same local crash log the panic
  hook writes, builds an issue title/body preview, percent-encodes a pre-filled
  GitHub new-issue URL, and clears the log after the user acts.
- The GUI detects a crash log at startup and shows a **Crash report** window
  with the log preview, an optional notes box, **Open GitHub issue**,
  **Copy details** and **Dismiss**. Nothing is sent automatically; the only
  network-facing step is the browser opening the pre-filled issue URL after the
  user presses the button, and no token is embedded in the binary.
- A **Report a bug…** toolbar button opens the same window without a crash log,
  so ordinary bugs use the same consent-first path.
- The application's panic hook now uses the shared log-path function, keeping
  the writer and reader on the same file.
- `README.md` privacy section and `docs/usage.md` document the flow.

Tests: the crash module tests cover title extraction, empty-log fallback,
notes/log composition, URL encoding and the unreserved-character set. The
dialog itself is interactive-only.

## 2026-10-25 — machine-readable schedule output (`--json`)

Done in this slice:

- `cadence --json` and `cadence load <file> --json` print the schedule as one
  JSON document. `cadence-interchange` gained a `schedule` module that reuses
  the crate's private JSON reader and writer to carry the derived schedule —
  project start/finish, per-activity scheduled/early/late dates, total and
  free float, criticality and the critical path — with a schema version and a
  `kind` field that distinguishes a schedule from an error document.
- This is deliberately the one document where derived dates are written; the
  interchange document still never carries them (D9). The point is
  accessibility of tooling (guide §21.5): scripts and assistive technology
  should not have to parse the aligned plain-text table.
- Usage errors exit with code 2 (real errors stay 1), the usage screen points
  at `cadence --help`, and every failure now prints an
  `ERROR <UTC timestamp> cadence: <message>` diagnostic on stderr; in JSON
  mode the machine reading stdout gets the error document and the person
  reading the terminal gets the error line.
- Tests in `schedule.rs` parse every written document with the crate's own
  JSON reader and assert on the parsed values: the full field set of a
  two-activity FS chain, a milestone's start == finish, quoting and newlines
  surviving a round trip, the empty project, a cyclic project's error
  document, and a single-activity critical path.
- `docs/usage.md` documents the command forms, the document shapes and the
  exit codes; ROADMAP, the traceability table (R24) and the parity tracker
  are updated in the same change.
- Verification runs in GitHub Actions on the verification branch; nothing was
  built or tested locally, per the no-local-builds rule.

## 2026-10-24 — project health: open-source readiness (D18, D19)

The program was complete; the *project* was not. Reviewed against GitHub's
opensource.guide collection, which covers running a project other people can
use and join, rather than producing software. The gaps were: no code of
conduct, security policy, accessibility statement, vision or governance; a
README and contributing guide written for someone who already knew the project;
no issue templates; no static analysis, dependency-licence check or SBOM; and no
protected branch.

Done in this slice:

- Six required documents written for this project (nothing copied from a
  third-party template): `CODE_OF_CONDUCT.md`, `SECURITY.md` (private
  reporting + threat model + incident response), `ACCESSIBILITY.md`,
  `VISION.md`, `GOVERNANCE.md`, `CONTRIBUTORS.md`.
- `README.md` rewritten stranger-first with a runnable example and a
  documentation index; `CONTRIBUTING.md` extended with environment setup, the
  exact local test commands, review expectations and the contribution paths.
  It states explicitly that the no-local-builds rule is a constraint of the
  maintainer's machine, not a rule for contributors.
- Issue forms (bug, feature, accessibility) and a contact-links page; pull
  request template extended with an accessibility checklist and a
  dependency/secret line.
- `Security` workflow: secret scanning over the full history, `cargo deny`
  licence/advisory/ban/source checks from `deny.toml`, dependency review on
  pull requests, CodeQL static analysis of the Rust sources and the workflow
  files, and a CycloneDX SBOM per run (the release workflow attaches one to
  each release).
- Dependabot for cargo and GitHub Actions; all workflow actions pinned to
  commit SHAs; default branch protected with required status checks.
- `ATTRIBUTION.md` extended to licence classes, non-code assets, build-tooling
  and model terms, and the SBOM location.
- Keyboard completeness: `Ctrl+5` for the Calendars view, so all five views
  have a shortcut (merged as `96bc03a`, run `40051603252`).

Found while doing this: the committed `Cargo.lock` never picked up `rfd`, the
dependency the PDF print path added, because cargo is not run on the
maintainer's machine. CI stayed green only because cargo rewrites the lock in
the runner's workspace — so builds were not reproducible from the repository
and the licence check could not see the new dependency. Recorded as D19;
`.github/workflows/lockfile.yml` regenerates the lockfile in CI for the
maintainer to commit, and a freshness check (`cargo metadata --locked`) is
switched on with the refreshed lockfile.

Resolved in the same pull request. The security workflow's first runs found
five more real things, each answered by fixing the repository rather than
relaxing the check (D20):

- Four unmaintained-crate advisories fire on the GUI stack (`ttf-parser`, and
  `zvariant`, `zbus_names`, `zvariant_derive`, merged into `zbus` 6 upstream).
  None is a vulnerability, none is a direct dependency, and none has a safe
  upgrade until `winit`/`eframe`, `accesskit` and `ashpd` migrate. Each ID is
  an explicit, dated ignore in `deny.toml` with the reason and the re-check
  trigger.
- A yanked `clipboard-win 5.4.1` pin: the remedy is the regeneration, not an
  ignore. The lockfile workflow now runs on every pull request and re-resolves
  that package explicitly.
- `epaint_default_fonts` embeds Ubuntu and Noto Emoji glyphs under the Ubuntu
  Font Licence 1.0 and OFL-1.1 — permissive, OSI-approved font licences. Both
  are allow-listed in `deny.toml` for that purpose, and the fonts are now
  recorded in `ATTRIBUTION.md`, which had wrongly said the project carried no
  third-party non-code assets.
- The wildcard-dependency ban flagged the internal workspace dependencies,
  which were path-only. The fix is an explicit `version` on each
  `[workspace.dependencies]` entry — good hygiene, and it keeps the crates
  publishable — rather than relaxing the ban.

The refreshed `Cargo.lock` is committed in the same change, with the freshness
check on.

Not decided here (maintainer-side, listed in `VISION.md` and `ROADMAP.md`):
name/trademark check, canonical home URL and handles, employer/IP policy, a
second admin with MFA, real-machine testing of the Windows and macOS binaries,
and the human-gated research notes.

## 2026-10-23 — merged to `main`; guide conformance verified from a clean checkout

PR #1 (`opencode/ci` → `main`, 59 commits, merge `01223602`) is merged. CI on
`main` — run `39927047788` — is green: 203 tests on Linux, Windows and macOS,
`cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D
warnings` on Linux, and the `cadence-linux-x86_64` release artefact. Every
"Done" entry above is therefore verified on `main`, not only on the pull
request branch.

Guide conformance completed in the same pass:

- CI now runs a matrix over ubuntu, windows and macos tests with the release
  binary packaged and uploaded as an artefact (D17 supersedes the Linux-only
  CI recorded in D5); formatting and lints stay on Linux alone because they are
  platform-independent.
- `ATTRIBUTION.md` now distinguishes direct from dev-only dependencies, adds
  `proptest`, and no longer marks `egui`/`eframe` as planned.
- `docs/development.md` documents the `ci.sh` verification workflow, the
  multi-OS CI, and the current crate layout including `cadence-interchange`.
- Tracked files scanned for credentials (private keys, cloud and service
  tokens, inline secrets): no matches; credentials live outside the repository.
- `guide.md` is not tracked by git and never leaves the local machine.
- `docs/implementation-report.md` records the evidence-based completion report
  the guide asks for at the end of the agreed scope: what was implemented, the
  verification evidence with run IDs, the guide quality gates, and the decided
  limitations.

## 2026-10-23 — JSON schema v2: WBS tree and named calendars

- JSON now carries the two things no other interchange format can express: the
  WBS tree (written in preorder, so a parent always precedes its children) and
  named calendars with their Monday-first working weeks and holiday
  exceptions. Activities gained `wbs` and `calendar` membership.
- The schema version is bumped to 2 and the reader accepts 1 as well, so an
  older build refuses a newer document instead of silently dropping the tree,
  while a version 1 document still loads with an empty WBS and no calendars.
- Import validates membership: an activity naming a missing WBS node or
  calendar is rejected with the offending reference, rather than being handed
  to the project file to reject later.
- Tests: round trip of tree and calendars, preorder ordering of the export, a
  version 1 document loading, dangling membership rejected, malformed trees
  (second root, child before parent) and calendars (empty week, short
  workweek) rejected, plus a property test over generated trees and calendar
  weeks. 203 tests in total.
- CI green on `opencode/ci`.

## 2026-10-24 — Calendars view and in-place calendar definition command

- New `SetCalendarDefinition` command: replaces a named calendar's working
  week and holidays in place, so activities keep their reference; the inverse
  command restores the previous definition, and a failed command rolls the
  old one back rather than losing it (tested, including the unknown-calendar
  rejection).
- New Calendars view in the GUI: calendar table (working days/week, holidays,
  activities using each), create with name/working-day checkboxes/holiday
  dates, rename, delete (refused while an activity uses the calendar, with a
  count), holiday list for the selected calendar.
- Inspector gained a per-activity calendar picker routed through the existing
  `SetActivityCalendar` command.
- Command tests cover the round trip (definition changes, reference intact,
  undo restores) and rejection. Clippy caught `assert_eq!` with literal bools
  in the first test draft, corrected to `assert!`/`assert!(!..)`.
- CI green on `opencode/ci`.

## 2026-10-24 — v0.1.0 released; guide rewritten as suite-ready edition

The guide was rewritten as the "Suite-Ready" edition. Its new requirements
were implemented and verified:

- **v0.1.0 published**: tag `v0.1.0` on main; release.yml built release
  binaries on Linux, Windows and macOS and published them with SHA-256SUMS at
  https://github.com/Perchemop/cadence/releases/tag/v0.1.0 (workflow
  checksum-glob bug found on the first run and fixed).
- **Crash stub**: `crash_report::install()` panic hook appends version, OS,
  panic message and a backtrace to a local crash log; nothing is sent
  anywhere. README gained a privacy section. MSRV bumped to 1.81 for
  `std::panic::PanicHookInfo`.
- **Update-check stub**: `update_check` module with the full v0.2.0 API shape
  (variants allowed dead code until wired) surfaced as `cadence version`.
- **ROADMAP.md**: milestones with evidence-based readiness.
- **.github/PULL_REQUEST_TEMPLATE.md**: the guide §8.3 checklist.
- **.github/workflows/release.yml**: tag-triggered release builds and notes.
- **docs/design/brand.md**: original identity and tested colour tokens.
- **docs/research/**: process README (human gate) plus four draft notes
  (Gantt printing, AccessKit, crash submission, update check) — original
  summaries and public URLs only.
- **CONTRIBUTING.md**: licence terms for PRs.
- Native PDF printing landed earlier in this pass (D16 reversed): PDF writer,
  `cadence print`, GUI **Print report…** with a native save dialog (`rfd`).

Verified on main at each step (runs on 862c268, 25c17b8, bf32a89, d15467a,
f7e3553 and the release run 40042554896).

## 2026-10-24 — P5-2b native PDF printing (D16 reversed)

- `cadence-core/src/pdf.rs`: a dependency-free PDF 1.4 writer — text pages in
  Helvetica (regular/bold) with wrapping and A4 pagination, raw RGB image
  pages, every stream ASCII-hex encoded so no payload byte can collide with
  the `endstream` marker.
- `report::project_report_pdf` renders the report through it; CLI
  `cadence print <file> <out.pdf>`; GUI **Print report…** opens a native save
  dialog (`rfd`, the one dependency this slice adds) and writes the PDF.
- D16's "printing stays out of the binary" half is reversed by maintainer
  decision; the Markdown report remains the reviewable artifact.
- Eight PDF unit tests: structure and escaping, xref offsets, page objects,
  empty-document handling, hex-stream safety, wrapping, image fitting.
- CI green on `opencode/ci`; dialog behaviour itself is interactive-only and
  recorded as such in the Phase-5 QC gate.

## 2026-10-23 — property tests for leveling and interchange, GUI report export

- New property test in `cadence-core`: leveling never moves work earlier, the
  reported delay matches the calendar distance, and a conflict-free result
  must be reproducible through ordinary start-no-earlier-than constraints —
  which is the D12 equivalence claim checked against arbitrary projects.
- New property suite in `cadence-interchange` (`tests/properties.rs`,
  proptest as a dev-dependency): arbitrary projects round-trip through all
  four formats, with each format's documented scope asserted — JSON exactly,
  MSPDI exactly apart from percent quantisation, CSV and MPX for the plan.
- The interchange properties found two real defects on their first run:
  * MSPDI truncated sub-percent progress (149 basis points became 0% instead
    of 1%); it now rounds to the nearest whole percent, because MSPDI stores
    an integer `PercentComplete` and cannot hold anything finer.
  * CSV renumbers the resource dictionary on import, which the properties
    surfaced as an assignment mismatch; it is now documented as a property of
    the format (assignments match by name) rather than assumed away.
- The GUI gained a **Save report** button that writes the Markdown report next
  to the working directory and reports the path in the status area, so the
  printable artifact is reachable without the CLI.
- CI green on `opencode/ci` (run 39898422581, 197 tests passed).

## 2026-10-23 — P5-2a/P5-3/P5-4 reports, keyboard map, contrast and performance gates

- `cadence-core::report` renders a Markdown project report: schedule table
  with dates, float and criticality; per-resource committed and
  over-allocated days; earned-value metrics against the most recent baseline;
  and the baseline list. Unschedulable projects say so instead of emitting an
  empty document. CLI: `cadence report <file> [out]` (stdout when no file).
- Keyboard map in the GUI: Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y undo-redo and
  Ctrl+1..4 view switching, working from every view.
- Accessibility: every colour the app draws text in now passes WCAG AA
  (4.5:1) against its panel background on both themes, verified by new
  contrast tests in `cadence-ui-egui` (variance colours are theme-specific
  now; white-on-accent selections and 3:1 non-text accent contrast are
  covered too).
- Performance gates in CI: 10,000 committed resource days analysed under
  750 ms and 100 parallel activities leveled under 3 s, alongside the
  existing 10,000-activity schedule gate.
- `docs/usage.md` documents commands, formats, the keyboard map and reports;
  `crates/app/examples/bridge_report.rs` builds the sample project, prints
  its report and checks a JSON round trip (linted by `clippy --all-targets`).
- Native print/PDF, the full keyboard-only audit, screen-reader support and
  the first tagged release are recorded as deliberate, user-owned deferrals
  in `docs/qc/phase-5.md`.
- CI green on `opencode/ci` (run 39892304802, 189 tests passed).

## 2026-10-23 — P5-1c Cadence JSON interchange

- `cadence-interchange::json` reads and writes Cadence's own JSON schema,
  which is lossless for everything the other interchange formats carry:
  activities with constraints and progress, relationships with lag, the
  resource dictionary with kinds and capacities, assignments and baselines.
- The schema version lives in the document (`"cadence": 1`) and a newer
  version is rejected with a clear error rather than misread.
- Scheduled dates are deliberately not written (D9): they are derived data and
  the engine recomputes them, so JSON export never needs a successful
  schedule.
- Written and read by an in-crate recursive-descent parser with a 64-level
  nesting limit, full escape and surrogate-pair handling, and no JSON
  dependency in the build (D14).
- Tests: full round trip, minimal hand-written document with defaults,
  schema-version and missing-field rejections, parser edge cases (escapes,
  surrogates, trailing garbage, deep nesting, number forms), string escaping
  round trip, and a JSON-vs-MSPDI equivalence check.
- CLI: `cadence export-json <in> <out>` and `cadence import-json <in> <out>`.
- CI green on `opencode/ci` (run 39859287583, 178 tests passed).

## 2026-10-23 — P5-1b CSV activity table interchange

- `cadence-interchange::csv` exchanges a project as one spreadsheet-friendly
  table: one row per activity carrying duration, constraint code and date,
  predecessor links (`1FS+1;2SS-1`), resource names, progress, actual dates,
  budget and actual cost, and the scheduled dates used to recover the project
  start on import.
- RFC 4180 quoting plus backslash-escaped semicolons inside list cells, so
  names may contain commas, quotes, newlines and semicolons.
- Resource names missing from the dictionary are created as Labour resources
  with capacity 1 (CSV has no kind or capacity columns — documented).
- CLI: `cadence export-csv <in> <out>` and `cadence import-csv <in> <out>`.
- Tests: round trip, hand-written table, predecessor shorthand parsing,
  quoting and escape round trips, rejection cases, and a CSV-vs-MSPDI
  equivalence check on the same project.
- CI green on `opencode/ci` (run 39844511576, 166 tests passed).

## 2026-10-22 — P4-2 progress updating

- Activities now record `actual_start`/`actual_finish` alongside the existing
  `percent_complete` and cost fields, persisted in schema v4 with in-place
  v1, v2 and v3 migrations.
- New undoable commands: `SetActivityProgress` (basis points, range-checked),
  `SetActivityBudget`, `SetActivityActualCost`, and `SetActivityActualDates`
  (finish-before-start rejected).
- The activity inspector edits all five progress fields in one apply, routed
  through the command stack.
- Gantt bars draw a progress overlay over the completed share.
- MSPDI imports and exports `ActualStart`/`ActualFinish`.
- Recorded progress stays out of the scheduling inputs (D15), so variance and
  earned-value comparisons stay honest.
- CI green on `opencode/ci` (runs 39774816170 and 39775089237).

## 2026-10-22 — P4-1/P4-3 baseline variance and earned-value panel

- New Baselines view in the egui shell: capture a baseline from the current
  schedule (undoable), select or delete one, and read a variance table that
  pairs every activity's current dates and duration against the selected
  baseline, colouring delays in red and improvements in green.
- The same view renders the earned-value metrics against an editable status
  date, so SPI and CPI are visible without leaving the application.
- CI green on `opencode/ci` (run 39764500014, 152 tests passed).

## 2026-10-22 — P5-1 open interchange: Microsoft Project XML and MPX

- New `cadence-interchange` crate with `mspdi` (Microsoft Project XML) and
  `mpx` readers and writers covering activities, durations at eight working
  hours per day, constraints, relationships with lag, the resource dictionary
  with capacities, assignments and baselines.
- XML is read by an in-crate reader (`interchange::xml`): declarations,
  comments and processing instructions are skipped, entities are decoded, and
  mismatched or unterminated tags are rejected, so no dependency is added to
  the build.
- CLI: `cadence export-xml|import-xml|export-mpx|import-mpx <in> <out>`.
- Round-trip and hand-written-document tests for both formats.
- CI green on `opencode/ci` (run 39754077818, 152 tests passed).

## 2026-10-22 — P3-3 resource conflicts, leveling and resource panel

- `engine::resource_conflicts` reports every working day on which an activity
  consumes a resource beyond its capacity;
  `engine::level_resources` greedily delays the latest-starting conflicting
  activity until the over-allocation clears, keeping dependencies intact.
- `Schedule` gained a scheduling path parameterised by minimum start dates so
  leveling can probe candidate schedules without mutating the project.
- New Resources view in the egui shell: dictionary table with kind, capacity,
  assignment count and over-allocated days; create/rename/delete; capacity and
  kind editing; per-day usage against capacity; assignment toggles; leveling
  preview and an apply action that writes undoable `SetActivityConstraint`
  commands.
- CI green on `opencode/ci` (runs 39751953234 and 39748409298).

## 2026-10-22 — P3-1/P3-2 resources, baselines and EVM engine

- Resource dictionary on `Project`: `ResourceKind`, name uniqueness
  (case-insensitive, matching the SQLite constraint), and `capacity` in units
  per working day. Undoable commands for create/rename/set-capacity/set-kind/
  delete, plus assignment commands with correct inverse operations.
- `engine::resource_histogram` collects the resource units committed on each
  working day.
- Project baselines: `engine::capture_baseline` snapshots duration, start and
  finish per activity; capture/remove/restore are undoable and preserve
  position.
- Earned value metrics (`engine::evm_metrics`): planned value, earned value,
  actual cost, schedule and cost variance, SPI and CPI in basis points, with
  cost stored in minor units.
- SQLite schema v3 adds `resources.capacity` and the resource, assignment,
  baseline and cost tables and columns, with v1→v3 and v2→v3 migrations.
- Comments in the schema and decisions record the append-only table style.
- CI green on `opencode/ci` (runs 39748409298 and 39751953234).

## 2026-10-21 — P2-5 performance baseline + Phase-2 QC gate

- Added a 10,000-activity scheduling performance test and the Phase-2 QC
  gate in `docs/qc/phase-2.md`.
- CI green on `main`.

## 2026-10-21 — P2-4 dependency editing

- Drag-to-create FS links in the Gantt view with cycle detection and
  feedback; lag editing via the inspector and `SetRelationshipLag`.
- CI green on `main` (run 39586623396).

## 2026-10-21 — P2-3 Gantt rendering

- Added a Gantt view with timeline axis, activity bars, dependency arrows,
  zoom (Ctrl+wheel), scrolling, and row virtualisation.
- CI green on `main` (run 39579147001).

## 2026-10-21 — P2-2 activity table

- Activity table now supports column sorting, row selection, an inspector
  with name/duration/constraint editing routed through commands (with
  undo/redo), and schedule recomputation.
- CI green on `main` (run 39578242160).

## 2026-10-21 — P2-1 egui app shell

- Added `cadence-ui-egui` crate with design tokens, light/dark theme
  switching, and a window rendering the project and scheduled dates.
- CLI gains `cadence gui [file]` (demo if no file).
- CI green on `main` (run 39577082469).

## 2026-10-21 — P1-6 property-based tests + Phase-1 QC gate

- Added `crates/core/tests/properties.rs` (proptest): working-day invariants
  for arbitrary DAGs, relationship logic and non-negative float, ALAP
  preservation of bounds/logic/project finish, constraint acceptance or
  violation, and calendar arithmetic round-trips.
- Fixed a mixed-calendar backward-pass bug found by the property tests:
  late finish is now normalised to the activity's own working calendar even
  for non-terminal activities.
- Recorded the Phase-1 QC gate in `docs/qc/phase-1.md`.
- CI green on `main` (run 39576146773, 89 tests + proptest cases).

## 2026-10-21 — P1-5 constraints

- Activities carry a `Constraint`: ASAP (default), ALAP, and start/finish
  no-earlier/no-later-than date bounds. Lower bounds raise the forward pass;
  upper bounds cap the backward pass and produce
  `SchedError::ConstraintViolated` when even the earliest schedule misses
  them.
- `ActivitySchedule` now reports `scheduled_start`/`scheduled_finish` next to
  the CPM early/late bounds. ALAP activities use their late dates, pulled
  back only where an already-scheduled successor would be violated (D10).
- Added the undoable `SetActivityConstraint` command and CLI columns for
  scheduled dates.
- Project-file schema is now version 2 with an in-place migration from
  version 1 (new `constraint_kind`/`constraint_date` columns).
- CI green on `main` (run 39574713546, 84 tests).

## 2026-10-21 — P1-4 SQLite persistence

- Project files are ordinary SQLite databases with a relational schema
  (`project`, `project_holidays`, `calendars`, `calendar_holidays`,
  `wbs_nodes`, `activities`, `relationships`) versioned with
  `PRAGMA user_version` (D9).
- Saves validate dangling WBS/calendar/relationship references before
  writing, then replace the file contents in one transaction; a failed save
  leaves the previous file intact. Loads reject newer schema versions and
  malformed data with explicit errors.
- CLI now supports `cadence save <file>` and `cadence load <file>` on top of
  the demo; `LinkType::from_abbreviation` round-trips precedence types.
- CI green on `main` (run 39566216222, 58 tests).

## 2026-10-21 — Guide updated → process retro-fit + WBS & calendars

- Adopted the updated generic guide: discovery conversation completed with
  the human; created `docs/project-brief.md`, `docs/scope.md`,
  `docs/inspiration.md`, `docs/assumptions-and-questions.md`, and all root
  tracking files; added `.opencode/agents/` definitions.
- Discovery answers recorded; multi-OS CI explicitly deferred (D5).
- P1-2 done: WBS tree (`wbs.rs`) with cycle-safe reparenting, delete guards,
  and undoable commands.
- P1-3 done: named calendars (`calendar_set.rs`), per-activity calendar
  selection, cross-calendar CPM convention (D6), undoable calendar commands.
  CI green on `main` (run 39559389569) after fixing cross-calendar
  working-day stepping with dedicated `advance`/`retreat` helpers.

## 2026-10-20 — Vertical slice complete

- Repo created (public: https://github.com/Perchemop/cadence), Phase 0
  scaffold + CI.
- P1-1 done: domain model, full CPM engine (FS/SS/FF/SF + lag, float,
  critical set, cycle detection), command layer with undo/redo, CLI demo
  printing the schedule table. 30+ unit tests. CI green after one clippy fix
  (needless lifetime in a test helper).

## Next slice

- Phase 2 (P2-1..): egui/eframe app shell, design system, then Gantt and
  tables. No further Phase-1 work remains.
## 2026-10-24 — v0.1.1: open-source readiness released, Linux ARM64 first-class

PR #25 merged (all fourteen checks green, twelve of them required on `main`):
Linux ARM64 joined the CI matrix and the release matrix, so `cadence-linux-
aarch64` is built on every change and shipped in every release. `v0.1.1` was
tagged on `main` and the release workflow passed all six jobs, including
`Build (ubuntu-24.04-arm)`.

The released ARM64 binary was installed on the maintainer's aarch64 machine
(`~/.local/bin/cadence`), its SHA-256 verified, and exercised: `--help`,
`version`, the demo schedule, a `save`/`load` round-trip, a Markdown report
and a PDF print — all working on the target machine. The GUI needs a display
and was not exercised here.

That run exposed two defects in the first publishing (D21): the binary
reported version 0.1.0 inside the v0.1.1 release (the tag was cut before the
crate version was bumped), and `SHA256SUMS.txt` listed artefact directory paths
plus a checksum of its own empty file. Both were repaired, the workspace
version is now 0.1.1 (the six `Cargo.lock` entries synced to match), and
v0.1.1 was deleted and re-published from the corrected commit so that the tag,
the checksum file and the string `cadence version` prints all agree.
