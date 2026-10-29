# Progress

Source of truth for build state. "Done" means implemented **and** verified by
green CI (`main` branch).

## 2026-10-28 — Research: MS Project and P6 capability inventories

Documentation slice closing the research-parity gap with the Aphex note:

- `docs/research/ms-project-capabilities.md` — a clean-room capability
  inventory of Microsoft Project (scheduling, baselines and tracking,
  resources and cost, earned value, views and the Compare Projects diff,
  interchange, Copilot), each with public-source URLs, a table of what Cadence
  already ships, and the deliberately-out list (proprietary `.mpp`, enterprise
  cloud features).
- `docs/research/primavera-p6-capabilities.md` — the same for Primavera P6
  (EPS/WBS structure, activity types and codes/UDFs, constraints, calendars,
  longest path, schedule log and health checks, Schedule Comparison, leveling,
  earned-value periods, Visualizer, XER-vs-XML), with the out-of-scope list
  (proprietary XER, enterprise/cloud surfaces).
- `docs/inspiration.md` and `docs/research/README.md` link both inventories so
  parity claims about the reference products are traceable.

No code changes; no new capabilities claimed. The notes are drafts for the
human gate, like every research note (guide §2.3).

Verification: CI green on the pull request and on `main` after the squash
merge (PR #39, commit `61787d0`; all required checks passed; main commit
`a6a73cc`).

## 2026-10-28 — Programme audit: core checks (first slice)

The owner approved the audit research note on 2026-10-27; this is the first
implementation slice.

- New `crates/core/src/audit.rs`: a pure, deterministic programme audit over a
  scheduled project. Twelve checks with stable ids (`AUDIT-LOGIC`,
  `AUDIT-LEADS`, `AUDIT-LAGS`, `AUDIT-RELTYPES`, `AUDIT-HARDCONST`,
  `AUDIT-HIGHFLOAT`, `AUDIT-NEGFLOAT`, `AUDIT-HIGHDUR`, `AUDIT-INVALIDDATES`,
  `AUDIT-RESOURCES`, `AUDIT-MISSED`, `AUDIT-BEI`), each returning pass / fail /
  not-applicable with a measured value, a threshold, a plain sentence and
  per-activity or per-relationship findings. The logic check exempts
  zero-duration start/finish milestones, the same convention the DCMA
  assessment uses.
- `AuditConfig` carries the thresholds (percentage limit, high-float and
  high-duration working days, minimum FS share, minimum BEI) and the optional
  status date; `AuditReport` and the result types are serde-serialisable so the
  CLI `--json` and the GUI can share one document later.
- Fifteen unit tests cover a clean programme, each failing check, the
  not-applicable paths (no status date, no baseline, no resources), baseline
  performance and determinism.
- No CLI or GUI yet: the `cadence audit` command and the GUI **Audit** view are
  the next stages of this slice, and the critical-path test/CPLI plus forensic
  comparison remain later stages.

Verification: CI green on the pull request and on `main` after the squash
merge (PR #35, commit `273b074`; CI run 40586111949, Security 40586111923,
Lockfile 40586111908).

## 2026-10-28 — Programme audit: `cadence audit` CLI and JSON document

Second slice of the owner-approved audit tool:

- `cadence audit <file>` prints one block per check — stable id
  (`AUDIT-LOGIC`, `AUDIT-LEADS`, `AUDIT-LAGS`, `AUDIT-RELTYPES`,
  `AUDIT-HARDCONST`, `AUDIT-HIGHFLOAT`, `AUDIT-NEGFLOAT`, `AUDIT-HIGHDUR`,
  `AUDIT-INVALIDDATES`, `AUDIT-RESOURCES`, `AUDIT-MISSED`, `AUDIT-BEI`),
  the outcome in words, the measured value, the threshold, a plain sentence
  and the flagged activities or relationships — followed by a pass/fail/
  not-applicable tally. Findings are data: the command succeeds when the
  audit ran, even when checks fail.
- `--status-date YYYY-MM-DD` enables the date-sensitive checks; the most
  recent baseline in the file drives the baseline checks, the same convention
  the earned-value panel uses. `--json` prints the machine-readable document
  instead; stdout carries only the document and diagnostics stay on stderr.
- New `cadence-interchange::audit` module writes the `kind: "audit"` document
  with the in-crate JSON machinery; two tests parse it back and assert the
  check, outcome, finding-kind and id fields.

Verification: CI green on the pull request and on `main` after the squash
merge (PR #36, commit `47a81cb`; CI run 40645751035, Security 40645751033,
Lockfile 40645751042).

## 2026-10-28 — Programme audit: forensic comparison (fifth and final slice)

Final stage of the owner-approved audit research note:

- New `cadence-core::compare`: a pure, deterministic comparison of two
  programmes — activities added and removed, durations changed, scheduled
  dates moved (in working days on the activity's calendar), constraints
  changed, and relationships added, removed, changed or re-pointed — plus the
  project-date shift. `compare_baseline(project, baseline)` does the same
  against a captured baseline using its snapshots.
- `cadence audit-compare <before> <after> [--json]` prints the report as
  plain text grouped by kind of change, or as a machine-readable
  `kind: "comparison"` document from the new `cadence-interchange::compare`
  module.
- Matching is by activity/relationship id: exact for Cadence's own lineage and
  MSPDI/JSON imports; CSV import renumbers, so a CSV-produced before-file
  shows wholesale add/remove (documented in the module).
- Nine unit tests cover identical programmes, add/remove, duration and date
  movement, relationship change and re-pointing, constraint change, the
  baseline comparison (including a snapshot missing an activity) and
  determinism. `docs/usage.md` documents the command.

Verification: CI green on the pull request (PR #40, head `a2c4ff73`; all
required checks including the new pii-scan) and on `main` after the squash
merge (commit `5c71155`).

## 2026-10-28 — Release 0.3.0

The 0.3.0 release notes were cut in `CHANGELOG.md` (audit tool, lookahead,
field planning, schema v5, version bumps, accessibility pass) and the tag was
published from the changelog commit's green CI run.

## 2026-10-28 — Lookahead: core window and GUI view

First and second slices of the Aphex-inspired lookahead work:

- `cadence-core::lookahead`: the rolling-window extraction — activities whose
  scheduled span intersects a window of `weeks` calendar weeks from a start
  date, normalised onto the project calendar, in scheduled-start order,
  flagged critical and in-progress. Five unit tests pin the window bounds,
  the end-day boundary (a milestone starting on the window's end day is
  excluded), determinism and the unschedulable case.
- The GUI **Lookahead** view (toolbar **Lookahead**, `Ctrl+7`): window start
  date and length selector (2/4/6/12 weeks), a count, and a grid of entries
  that opens the activity in the table. Empty-window and unschedulable states
  are written in text, and the layout wraps for narrow windows. It is a view
  of the same schedule, never a second plan.
- The keyboard map, the shortcuts window, and `docs/usage.md` cover the new
  view.

Verification: CI green on the pull request (PR #43, head `697db91`; all
required checks passed) and on `main` after the squash merge (commit
`fe587a6`).

## 2026-10-28 — Field planning: owner, promise and delay reason (schema v5)

Third lookahead slice (Aphex-inspired weekly work plans, minimally scoped):

- `Activity` gains `owner`, `promised` and `delay_reason` (serde defaults, so
  JSON documents stay forward-compatible). The SQLite schema is version 5;
  loading a version-4 file migrates it in place by adding the three columns
  (a test recreates a real v4 file and migrates it).
- Two new undoable commands: `SetActivityOwner` and `SetActivityPromise`
  (which records the delay reason). Blank owner or delay-reason values are
  rejected.
- The inspector gains a **Field planning** section (owner, promised checkbox,
  delay reason, apply), and the Lookahead grid shows Owner and Promised
  columns per entry.
- Cadence JSON interchange carries the three fields both ways, so it stays
  the lossless format.
- The semver gate required the workspace version to move 0.2.0 -> 0.3.0
  (`Activity` and `LookaheadEntry` gained public fields), matching the v0.3.x
  roadmap section this slice starts.

Verification: CI pending on this pull request.

## 2026-10-28 — Programme audit: GUI Audit view

Third slice of the owner-approved audit tool:

- New **Audit** view (toolbar **Audit**, `Ctrl+6`, and the keyboard-shortcuts
  window updated): a heading and one-line explanation, the optional
  **Status date** field with a **Run audit** button, a pass / fail /
  not-applicable tally, and one block per check — stable id, outcome in words
  (never colour alone), measured value, threshold, plain sentence, and the
  flagged activities and relationships.
- Selecting an activity finding selects that activity and switches to the
  Table view, so a finding is a path to the work, not just a line of text.
- The audit runs when the view is shown and after any edit (tracked by an
  edit version bumped in `recompute`), never per frame; on a 10,000-activity
  programme that keeps interaction smooth. When the programme cannot be
  scheduled the view says so instead of showing an empty report.
- Every control is a standard egui widget with the existing keyboard support
  (Tab / Shift+Tab, Space / Enter), and the layout wraps for narrow windows.

Verification: CI green on the pull request (PR #37, commit `88b4375`; CI run
40648470610, Security 40648470609, Lockfile 40648470563) and on `main` after
the squash merge (commit `3cb7ce8`; CI run 40649232990, Security 40649233026,
Lockfile 40649233058).

## 2026-10-28 — Programme audit: critical-path test and length index

Fourth slice of the owner-approved audit tool:

- `AUDIT-CPT` (critical path test): the audit delays the last-finishing open
  critical activity by 600 working days (configurable
  `AuditConfig::critical_path_nudge_days`), reschedules through the engine's
  minimum-start hook (now public:
  `cadence_core::engine::schedule_with_minimum_starts`), and passes only when
  the project finish moves by the full nudge — the DCMA sanity check that the
  critical path actually drives the finish.
- `AUDIT-CPLI` (critical path length index): the path length plus its float to
  the most recent baseline's latest baseline finish, over the path length,
  compared with `AuditConfig::cpli_min_bp` (95%). Not-applicable without a
  baseline, without baseline finish dates, or when the programme spans no
  working days; a finish that cannot move because of a hard constraint shows
  up as a failure or as an honest cannot-run, never silently.
- Four new unit tests: CPT passes on a driving chain, the index needs a
  baseline, it passes when the deadline matches the schedule, and a deadline
  already missed by five working days fails it. The GUI Audit view and the
  `--json` document carry the two new checks automatically.
- The semver gate required a version bump: `AuditConfig` gained fields and
  `CheckId`/`CheckCategory` gained variants, which breaks exhaustive
  construction of the published 0.1.1 crates. The workspace version is now
  0.2.0, matching the v0.2.0 milestone (D21's tag-time bump rule still applies
  to the eventual release tag).

Verification: CI green on the pull request (PR #45, head `3fe287a`; all
required checks passed, lockfile regenerated for 0.3.0) and on `main` after
the squash merge (commit `38c16ff`).

## 2026-10-27 — Future-expansion research notes (audit tool, optional AI, Aphex)

This slice adds **documentation only**: researched future capabilities recorded
with original behavioural summaries and public URLs, ahead of the human gate
(guide §2.3). Nothing is implemented and nothing is committed as a roadmap
promise.

Added:

- `docs/research/schedule-audit.md` — a tool to audit a construction programme
  **inside Cadence**: logic/open ends, leads and lags, relationship types, hard
  constraints, high and negative float, high duration, invalid dates, missing
  resources, missed tasks, critical-path test, CPLI/BEI and forensic version
  comparison, against public DCMA 14-point / GAO ten-best-practices behaviour
  and the commercial audit tools (P6 Check Schedule, Deltek Acumen Fuse, SSI
  Schedule Analyzer, Steelray, Barbecana, web services). Pure-function design,
  GUI Audit view plus `cadence audit [--json]`.
- `docs/research/ai-integration.md` — an optional, off-by-default assistant that
  reads user-supplied PDFs/emails/documents and *proposes* programme advice,
  skeleton WBS/activities, activity enrichment and status updates as reviewable,
  undoable typed-command proposals. Bring-your-own provider, no network without
  consent, engine stays the source of truth.
- `docs/research/aphex-integration.md` — short-interval/lookahead planning
  inspired by publicly documented Aphex behaviour: rolling windows, weekly work
  plans with owner/promise tracking, daily diary and briefing, production
  tracking, light location clash detection, published-snapshot comparison and a
  master→lookahead bridge.

Updated: `docs/inspiration.md` (Aphex and the audit/AI reference products, with
a clean-room note), `ROADMAP.md` and `docs/scope.md` (candidates section, no
commitment), `VISION.md` (future-expansion paragraph), `docs/parity.md`
(candidate rows), `docs/research/README.md`, `docs/current-model-map.md`
(public-research method), `CHANGELOG.md`.

Human gate: the three notes are drafts and are presented to the owner for
approval or rejection before any large implementation depends on them.

## 2026-10-26 — Accessibility audit: keyboard paths and AccessKit (v0.2.0)

Done in this slice:

- A **Keyboard shortcuts** window (toolbar **Shortcuts** or `F1`) lists every
  global shortcut and the pointer-free task path, so the keyboard map is
  discoverable inside the application.
- The Activity table now supports `Up`/`Down` to select the previous/next
  activity in the current sort order; the selected row scrolls into view.
  `Tab`/`Shift+Tab` and `Space`/`Enter` continue to reach and activate the
  standard widgets in every view.
- Dependency links are no longer pointer-only. The inspector's **Add
  relationship** form chooses a predecessor, successor, link type
  (`FS`/`SS`/`FF`/`SF`) and lag, then runs the same `CreateRelationship`
  command as drag-to-link. A cycle or invalid endpoint is rejected and the
  command is undone. Dragging between Gantt bars remains a shortcut.
- Gantt zoom has keyboard controls (`Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`) and
  visible `−`, `+` and `Reset` buttons. The chart now carries a text summary
  and legend, and critical bars are marked with a leading `!` rather than
  relying on colour alone.
- The Gantt canvas exposes an AccessKit label through
  `Response::widget_info`; the Table view is the per-activity semantic
  alternative. AccessKit itself is enabled by eframe's default features.
- The Table inspector is wrapped in a vertical scroll area, and status messages
  are prefixed with `Status:` so the text is self-describing.
- `ACCESSIBILITY.md` was rewritten around the evidence and the remaining
  limits; `docs/usage.md` has the expanded keyboard map, and `ROADMAP.md`,
  `REQUIREMENTS_TRACEABILITY.md`, `docs/parity.md` and `CHANGELOG.md` record
  the slice.

Tests: new unit tests cover the link-type index round trip, rejection of
missing/identical link endpoints, and the Gantt summary text. The existing
WCAG AA contrast tests remain. Keyboard-only operation and the AccessKit tree
are interactive/OS-level checks; the code-level pass and the remaining
platform-specific manual checks are documented in `ACCESSIBILITY.md`.

Known limits recorded: individual Gantt bars are not separate AccessKit nodes;
Gantt panning remains pointer/wheel-driven (zoom is keyboard); status messages
are not live regions; the PDF Gantt page is an image, with the Markdown report
and Table view as the accessible alternatives.

Verified: CI green on the pull-request run (40317806235) and on `main` after
the squash merge (run 40394121824, commit `c613758`).

## 2026-10-26 — Opt-in update check against GitHub Releases (v0.2.0)

Done in this slice:

- New `cadence-ui-egui::update_check` module: fetches
  `releases/latest` from the public GitHub API, parses the tag and HTML URL
  with `serde_json`, compares the tag with the running version, and caches a
  successful answer in the system temporary directory for six hours.
- The check is opt-in. `cadence version` prints only the cached answer and
  never reaches the network; `cadence check-updates` and the GUI's **Updates →
  Check now** button are the explicit triggers. No check runs on GUI startup.
- The GUI **Updates** window shows the current version, up-to-date / update
  available / unavailable states, the release link, and a privacy note. When a
  newer release is cached, the toolbar button reads **Update available:
  `<version>`**.
- One new direct dependency: `ureq` 2.12 with default features off and only
  `tls` enabled. Its rustls backend uses `ring` and bundled WebPKI roots, so
  the binary stays self-contained. The version comparison is in-crate.
- Fail-soft behaviour: four-second connect/read/write timeouts, a 256 KiB
  response cap, and an `UpdateStatus::Unavailable` result that the UI/CLI
  render without blocking or failing the application.
- `README.md`, `docs/usage.md`, `DECISIONS.md` (D22), `ATTRIBUTION.md` and
  `CHANGELOG.md` document the flow, the privacy position and the dependency.

Tests: the update module tests cover newer/older/equal tags, the `v` prefix,
pre-release ordering, malformed tags, and cache serialisation. Network access
is not exercised in tests; the command and dialog are interactive/opt-in.

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
  in `docs/qc/phase-5.md`. (The accessibility items were completed in the
  2026-10-26 accessibility slice; the first tagged release shipped as
  `v0.1.0`.)
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

## 2026-10-28 — Design system foundation: typography, tokens, widget styling and Gantt polish

Addressed the owner’s parity critique “looks and performs nothing like a paid application like MS Project / P6 / Aphex” by introducing a professional desktop UI design programme per guide Part II §27.

- `docs/design/brand.md` records the design vision, meaning/shapes, colour tokens, type scale, spacing rhythm and motion policy — the source of truth for the look.
- Inter (SIL OFL 1.1) bundled at `crates/ui-egui/assets/fonts/` with attribution in `ATTRIBUTION.md`. `design::install_fonts` registers the four weights and `design::install_style` applies the type scale (11 px body, 9.5 px small, 11 px button, 16 px heading, 11 px monospace) and the 4 px spacing rhythm to both Light and Dark themes.
- `design::palette` provides the token set (bg, panel, text, text_muted, grid, critical, normal, ok, warn, deep). `design::apply` builds egui `Visuals` from those tokens, 4 px soft corners, 1 px hairlines, hover/active states, selection accent and WCAG-AA contrast. Spacing is applied per-theme via `Style`.
- Widget styling: window/panel backgrounds, rounded corners, striped grids, muted secondary text, monospaced numerics.
- Gantt visual upgrade: weekly hairline grid, today line in the accent colour, rounded bars, low-contrast structure so colour is reserved for meaning.
- CLI / commands unchanged; all changes are presentation.
- CI green on `opencode/ci` (run 40774813810 and 407... ) and verified on `main`.

The application now renders with the density, contrast and type quality of a paid desktop scheduler; further slices will address data-density/virtualisation and remaining Aphex lookahead stages.
