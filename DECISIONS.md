# Decisions

Format: **Decision** → context → alternatives considered → consequences.
Decisions are never silently reversed; changes require a new entry.

## D1 — Product name: Cadence (2026-10-20)

Context: guide required an original, professional name before scaffolding.
Alternatives: Keystone, Trestle, Datum. Chosen for rhythm/timing connotation.
Consequences: crate/binary names `cadence-*`.

## D2 — Dual licence MIT OR Apache-2.0 (2026-10-20)

Standard for Rust open source; maximises adoption.

## D3 — Workspace crate split: core / commands / persistence / app (2026-10-20)

Strict layering: `core` is pure domain + engine with no I/O or UI deps;
`commands` is the single mutation path; `persistence` owns storage formats;
`app` wires layers. UI crate (egui) joins in Phase 2.

## D4 — Heavy verification runs only on GitHub Actions (2026-10-20)

Context: the local machine is resource-constrained. Local work is editing +
`cargo fmt`; CI (fmt, clippy `-D warnings`, full tests) is the verification
signal. The repo is public for unrestricted Actions.

## D5 — Linux-only CI for now (2026-10-21)

Discovery answer: multi-OS (Windows/macOS) runners deferred to reduce CI
time/cost until the GUI lands. WASM build check deferred likewise. Revisit in
Phase 2 (see `docs/scope.md` "explicitly deferred").

## D6 — Calendar crossing convention (2026-10-21)

When linked activities use different calendars, forward-pass constraint of a
successor is computed on the successor's calendar; backward-pass constraint
on the predecessor's calendar. Documented in `docs/architecture.md`.

## D7 — WBS deletion semantics (2026-10-21)

Deleting a WBS node with children is rejected (explicit error) rather than
cascading, to guarantee no accidental mass mutation; the UI can offer an
explicit cascade command later.

## D8 — Original, accessibility-aware design system (2026-10-21)

Discovery answer: no existing brand. Light and dark themes designed from
scratch; contrast, keyboard support, and screen-reader basics are
requirements.

## D9 — SQLite schema versioning and save strategy (2026-10-21)

The project file is a relational SQLite database. Schema version lives in
`PRAGMA user_version` (currently 4). Opening a file with a newer version is
an explicit error rather than a best-effort read, so an old build can never
silently misinterpret or overwrite newer data. A version 0 database gets the
current schema created; each older version runs an ordered, in-place
migration step (v1 → v2 adds activity constraint columns).

Saving is a single transaction that clears and rewrites all tables. Failed
saves roll back, leaving the previous project intact. Dangling WBS, calendar,
and relationship references are rejected before writing. Cached schedule
fields are deliberately not persisted; the schedule is derived data and is
recomputed on load.

## D10 — Constraint semantics (2026-10-21; exact pins implemented 2026-11-09/10 — see D29)

Activities support ASAP (default), ALAP, SNET, SNLT, FNET, and FNLT.
Constraint dates are inclusive bounds interpreted on the activity's own
calendar: lower bounds round forward to the next working day, upper bounds
round backward to the previous working day. Exact "must start/finish on"
constraints are deferred until the UI can resolve conflicts (implemented
2026-11-09/10; see D29).

A lower bound raises the forward-pass dates. An upper bound caps the
backward-pass dates and tightens float. If the earliest feasible schedule
already misses an upper bound, scheduling fails with
`SchedError::ConstraintViolated`; we do not silently emit negative float for
an impossible hard constraint.

ALAP is a reporting preference, not a logic change: CPM early/late bounds are
always calculated normally, and `scheduled_start`/`scheduled_finish` choose
late dates for ALAP activities. Scheduled dates are finalised in reverse
topological order so a delayed ALAP activity never pushes an already-scheduled
successor past its own dates; precedence always wins over the ALAP
preference.

## D11 — Resource capacity and assignment model (2026-10-22)

A `Resource` carries a `kind` (Labour, Equipment, Material) and a `capacity`
in **units per working day**, floored at 1 and defaulting to 1. An activity
references resources by `ResourceId` in `Activity::resources`; each
assignment consumes exactly one unit per working day for the activity's whole
duration. There is no per-assignment units factor and no cost rate table yet,
so a "half a crane" is not expressible. This is deliberately the smallest
model that supports over-allocation detection and leveling, and it keeps
assignment lists as plain vectors that round-trip through SQLite and the
interchange formats without joins.

Resource names are unique case-insensitively because the SQLite column is
declared `UNIQUE COLLATE NOCASE`; the command layer enforces the same rule so
that an in-memory project cannot reach a state the database would reject.
Empty and whitespace-only names are rejected for the same reason.

## D12 — Resource leveling algorithm (2026-10-22)

Leveling is a greedy, iterative re-scheduler rather than an optimiser. Each
iteration recomputes the schedule, finds every working day on which a resource
is consumed beyond its capacity, and resolves the **earliest** conflicting day
by delaying one activity: the conflicting activity with the **latest**
scheduled start, tie-broken by the **largest** `ActivityId`, so the
later-listed activity in project order moves. Delays are expressed as minimum
start dates fed into the normal forward and backward passes, which means a
leveled schedule obeys exactly the same dependency and constraint rules as any
other schedule and cannot be "impossible" in a way the engine would not
report.

The iteration budget is `64 + activities * 32`. When it is exhausted, or when
re-scheduling errors, leveling stops and returns the conflicts that remain
instead of propagating an error, so the UI can always show something truthful
about why the schedule is not fully leveled. There is no attempt to find the
shortest possible delay: optimal resource-constrained project scheduling is
NP-hard, and a predictable, explicable heuristic is worth more here than a
slow exact solver.

## D13 — Money in minor units, percentages in basis points (2026-10-22)

Cost fields (`budget_at_completion`, `actual_cost`, baseline and EVM
metrics) are `i64` **minor units** — cents — never floats, so arithmetic is
exact and round-trips through SQLite integers without drift. Percentages are
`i64` **basis points** in `0..=10000` (50% is 5000). `EvmMetrics` derives only
`PartialEq` on purpose: SPI and CPI are ratios whose floating-point results
must not be compared for exact equality, and omitting `Eq` makes that
impossible to do by accident.

Interchange formats that express money as a decimal string (MSPDI
`FixedCost`, `ActualCost`) convert at the boundary with rounding to the
nearest minor unit, so the conversion is lossy only in the direction of
formats that cannot represent it exactly.

## D14 — Interchange formats and their documented subsets (2026-10-22)

Cadence talks to the outside world through four open, text-based formats in a
new `cadence-interchange` crate: **Microsoft Project XML** (MSPDI),
**MPX** (Microsoft Project Exchange), **CSV** and **JSON**. The first two are
implemented as documented subsets of someone else's format, and the module
documentation states the mapping field by field. Fields Cadence does not
model — cost rate tables, custom calendars, outline codes, levelling results —
are ignored on import and omitted on export instead of being guessed at.

CSV trades completeness for spreadsheet-friendliness: one activity per row,
with links, resources and progress folded into the row, and no baselines.
JSON is the opposite — Cadence's own schema, versioned in the document
(`"cadence": 1`), lossless for everything Cadence models and with no unit
conversions at the boundary. It exists so that a project can leave the
SQLite file and re-enter it, or be diffed and reviewed as text, without
touching a proprietary tool.

The XML and JSON readers are written in-crate (~200 lines each) instead of
pulling in parsing libraries. The dialects both formats need are narrow and
regular, and the project's offline, dependency-light stance means every added
dependency has to earn its place; each reader is covered by its own unit
tests. For XML that includes mismatched tags, unterminated tags, entity and
character-reference decoding, and angle brackets inside attribute values; for
JSON it includes escapes, surrogate pairs, trailing garbage, numbers out of
range, and a nesting limit that keeps hostile input from exhausting the
stack.

Two conversions are documented and lossy by design:

* Durations convert at **eight working hours per day**, the Microsoft Project
  default calendar, so `PT24H0M0S` is Cadence's 3 working days. Projects built
  on a different working-day length will see duration drift.
* Link lag is written in MSPDI's **tenths of a minute** and converted back to
  whole working days, and MPX lag is written in whole working days. Fractional
  lag does not survive.
* Percent complete is quantised to **whole percentages** in MSPDI, because its
  `PercentComplete` is an integer 0–100; anything finer is rounded to the
  nearest percent rather than truncated. CSV, JSON and the SQLite file keep
  full basis-point precision.

CSV additionally treats resource **names** as the identity: it has no resource
table, so importing renumbers the dictionary in first-seen order and matches
assignments by name. Activity identifiers are carried explicitly in the `ID`
column and in predecessor links, so the activity graph is stable.

JSON is the one format that is genuinely lossless: its schema version 2 carries
the WBS tree (in preorder, so parents precede children) and named calendars
with their working weeks and holidays, plus per-activity membership, which
MSPDI, MPX and CSV have no way to express. The version is part of the document
and a reader rejects a newer one, so an older build refuses a richer file
instead of quietly dropping the tree; a version 1 document still loads, with an
empty WBS and no calendars, because that is all it could have contained.

Two MSPDI constraints have no exact Cadence equivalent: *must start on* is
read as *start no earlier than* and *must finish on* as *finish no earlier
than*. Cadence's `Constraint` set has no "must" variant, and inventing one at
the interchange boundary would put a value in the model that the engine cannot
schedule exactly.

## D15 — Recorded progress is a fact, not a constraint (2026-10-22)

Progress is stored in three places: `percent_complete` in basis points
(0–10000, edited in the UI as a percentage), `actual_cost` in minor units, and
`actual_start`/`actual_finish` dates. These are **recorded facts about what
happened**, kept next to — never instead of — the plan. The engine still
derives schedules from logic, calendars and constraints; actual dates do not
feed the forward or backward pass, and they are cleared by no command because
they are not cached schedule state.

This keeps the model honest: the variance views and the earned-value panel
compare the plan against reality, and mixing reality into the scheduling
inputs would make that comparison circular. If progress-driven rescheduling is
wanted later, it should be an explicit engine mode rather than a silent
rewrite of constraints.

Percent complete is exposed as basis points in the model and as a percentage
in the UI and interchange formats, because every format that surrounds Cadence
speaks percentages. The conversion happens at the boundary in both directions
and rounds to whole basis points.

## D17 — CI verifies every platform; Linux-only CI is retired (2026-10-23)

D5 kept CI on Linux only "for now". The guide requires multi-OS runners when
practical, and Cadence targets Windows, macOS and Linux, so the verification
job now runs a matrix over all three and the release binary is packaged on
Linux and uploaded as an artefact.

Formatting and lints stay on Linux alone: they are operating-system
independent, and running them three times would only spend runner minutes. The
test suite — which is where platform differences actually surface: path
handling, SQLite's bundled build, line endings, integer and float formatting —
runs on every platform. `fail-fast` is off so one red platform cannot hide
another's result.

## D16 — Reports are Markdown; PDF printing is in (reversed 2026-10-23)

Printing started as a Markdown-only path: render a report, open it in a
browser, print from there. The reasoning was that a print pipeline means a
dependency. That was later reversed by maintainer decision after the report
existed and was in daily use: what was missing was not a print pipeline but a
*file format*, and a PDF can be produced dependency-free.

The adopted design (reversing the "printing stays out of the binary" half of
this decision, keeping the rest):

* `cadence-core/src/pdf.rs` writes PDF 1.4 in-tree — text pages in Helvetica
  (regular and bold), raw RGB image pages for screenshots, ASCII-hex streams
  so no payload byte can collide with the `endstream` marker, A4 pagination
  with wrapping. No compression, no embedded fonts, no interactive features.
* `report::project_report_pdf` renders the report through it; the CLI exposes
  `cadence print <file> <out.pdf>`.
* The GUI's **Print report…** button uses `rfd` for a native save dialog —
  the one dependency this decision now accepts, chosen because egui has no
  print API and a save dialog is the platform-correct way to pick a
  destination. Dialog behaviour itself is interactive-only (see the Phase-5
  QC gate).

The original reasoning about Markdown remaining the reviewable, diffable
report still holds: PDF is the *print* artifact, Markdown is the *working*
one.

*Original decision (2026-10-23):* Printing is provided by rendering a Markdown
report rather than by wiring a print pipeline into the GUI. A PDF would need
either a print-dialog dependency (platform-native, unverifiable in CI) or a
PDF writer (a large, fiddly format for little gain), while a Markdown report
is plain text that every editor and browser can print or export to PDF
already. The report generator lives in `cadence-core::report`: it is pure text
generation from the model and schedule, so it is unit-testable in CI, reusable
from the CLI today and from the GUI later, and performs no I/O itself.

The report is honest by construction: it renders whatever the engine can
schedule and states plainly when it cannot. It never invents dates, and
derived schedule data is recomputed at generation time rather than read from
cached fields (D9).

## D18 — Project-health practices adopted from opensource.guide (2026-10-24)

The build was reviewed against GitHub's opensource.guide collection, which
covers what a *project* needs (not only what a *program* needs). The result was
a body of work with one property: none of it changes the scheduling engine, and
all of it is required before the project can honestly call itself open source.

Adopted, and implemented in this change:

* **Required repository documents.** `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `ACCESSIBILITY.md`, `VISION.md`, `GOVERNANCE.md` and `CONTRIBUTORS.md`, each
  written for this project (nothing is copied from a third-party template), and
  `README.md` and `CONTRIBUTING.md` rewritten for a reader who has never heard
  of Cadence. `ATTRIBUTION.md` now also covers non-code assets, the licence
  classes accepted, and the tooling/model terms behind the build's provenance.
* **Contribution surface.** Issue forms for a bug, a feature and — separately,
  because the context and urgency differ — an accessibility report; a pull
  request template with an accessibility section for UI changes and a licence
  and secret-scanning line for dependencies; contribution labels.
* **Security in CI.** A `Security` workflow: secret scanning over the full
  history, dependency licence/advisory/ban/source checks driven by `deny.toml`,
  dependency review on pull requests, CodeQL static analysis of the Rust
  sources and of the workflow files themselves, and a CycloneDX SBOM per run —
  the release workflow attaches an SBOM to each release as well. Dependabot
  covers cargo and the pinned actions, and every action is pinned to a commit
  SHA (except `dtolnay/rust-toolchain`, which reads its own ref name to choose
  the toolchain).
* **Governance in writing.** `GOVERNANCE.md` names the model (BDFL with a
  written trail), the decision rule (consensus seeking, maintainer as
  tiebreaker), the public-by-default rule with its two private exceptions, the
  path to becoming a maintainer, the metrics that are and are not tracked, and
  the response-time and effort boundaries.
* **Repository controls.** The default branch requires the multi-platform build
  checks and an up-to-date branch, and refuses force pushes and deletion;
  required *review* is deliberately not enabled while there is a single
  maintainer, because GitHub forbids self-approval and an unsatisfiable rule is
  not a control — it is recorded to be switched on with the second maintainer.
  Private vulnerability reporting, secret scanning with push protection, and
  Dependabot alerts/security updates are enabled in the repository settings
  (build-side equivalents live in the security workflow, so the guarantees do
  not rest on a setting alone).

Recorded but **not decided** — these are for the maintainer, and they are listed
in `VISION.md` so they stay visible: the name-conflict and trademark check, the
canonical home URL and handles, the employer/IP question, and the second-admin
and MFA requirements. The guide's own conflict list (local-test policy versus
contributor expectations, public-from-day-one versus security gates, direct
pushes versus protected branches) is handled as follows: `CONTRIBUTING.md`
documents the exact local build and test commands and states plainly that the
no-local-builds rule is a constraint of the maintainer's machine rather than a
rule for contributors; and the default branch is protected with required status
checks, which matches the pull-request flow already in use.

## D19 — Cargo.lock is regenerated in CI, and its freshness is checked (2026-10-24)

Adding the PDF print path pulled in `rfd`, and the committed `Cargo.lock` was
never regenerated — cargo is not run on the maintainer's machine, so a `cargo`
step that would have rewritten the lockfile never ran. CI stayed green because
cargo updates the lockfile in the runner's workspace, which means the committed
lockfile was silently lagging the manifests: builds were not reproducible from
the repository, and the licence check could not see the new dependency at all.

Two changes follow:

* `.github/workflows/lockfile.yml` regenerates `Cargo.lock` on a runner and
  uploads it as an artefact, for the maintainer to commit like any other
  change. The file is deliberately **not** committed from inside the workflow,
  so a lockfile change is always a reviewed, CI-verified change.
* A lockfile-freshness check (`cargo metadata --locked` in the same workflow)
  fails the build when the committed lockfile no longer matches the manifests,
  so this class of drift cannot recur silently. The check is switched on in the
  change that commits the refreshed lockfile, because enabling it first would
  fail every build until the lockfile it describes has been written.

The alternative — running cargo locally — remains excluded by the build rules.

## D20 — The dependency checks are policy, not decoration (2026-10-24)

The security workflow's first runs found five real things, and each one was
answered by fixing the repository rather than relaxing the check:

* **Unmaintained crates.** Four "unmaintained crate" advisories fire on the
  GUI stack: `ttf-parser` (RUSTSEC-2026-0192) and the merged-into-`zbus`-6
  trio `zvariant`, `zbus_names`, `zvariant_derive` (RUSTSEC-2026-0370, -0371,
  -0372). None is a vulnerability, none is a direct dependency, and none has a
  safe upgrade: the parents (`winit`/`eframe`, `accesskit`, and `ashpd` via
  `rfd`) have not migrated yet. Each advisory ID is an explicit, dated ignore
  entry in `deny.toml` with the reason and the re-check trigger. The
  alternative — downgrading the whole unmaintained class to a warning — would
  have made every future case invisible instead of three current ones
  deliberate.
* **A yanked pin.** `clipboard-win 5.4.1` was yanked upstream. The remedy is
  the D19 regeneration, not an ignore: the lockfile workflow re-resolves that
  package explicitly, and the refreshed lock commits the result.
* **Font licences.** `epaint_default_fonts` embeds Ubuntu and Noto Emoji
  glyphs under the Ubuntu Font Licence 1.0 and OFL-1.1 — permissive,
  OSI-approved font licences covering an asset, not code. They are
  allow-listed for exactly that purpose, and the fonts are now recorded in
  `ATTRIBUTION.md`, which had wrongly said the project had no third-party
  non-code assets.
* **Wildcard requirements.** The wildcard ban flagged the internal workspace
  dependencies, which were path-only. The fix is an explicit `version` on each
  `[workspace.dependencies]` entry — good hygiene, and it keeps the crates
  publishable as-is — rather than `wildcards = "allow"`, which would have
  re-opened the door for every future dependency.

## D21 — Release discipline: the version string, the checksums and the tag must agree (2026-10-24)

v0.1.1 was published twice, and the first publishing taught three lessons the
hard way. The tag was cut before the crate version was bumped, so the released
binary answered `cadence version` with "0.1.0" from inside the v0.1.1 release —
exactly the string a bug report would quote. The checksum step listed artefact
*directory* paths and, because the shell creates the output file before the
listing runs, checksummed the empty sums file itself; the hash values were
right but `sha256sum -c` could not be run against the downloaded assets.

Both were fixed, the release was deleted, and v0.1.1 was re-published from the
corrected commit. The rules that follow are recorded so they are decisions, not
memories:

1. **The crate version is bumped in the same change as the tag that publishes
   it.** A release whose `cadence version` disagrees with its tag is a defect,
   because every bug report, checksum comparison and support answer starts
   from the string the binary prints.
2. **`SHA256SUMS.txt` lists plain asset names** — the names as released — so a
   downloader can run `sha256sum -c SHA256SUMS.txt` directly, and it is written
   where the listing cannot see it, so it is never its own input.
3. **Linux ARM64 is a first-class target.** The maintainer's machine is
   aarch64, so every change builds and tests on ARM Linux in CI, and every
   release carries `cadence-linux-aarch64`. "It works on the machines that
   build it" is not a release claim this project can make, because none of its
   verification happens on the maintainer's laptop.

The six `cadence-*` entries in `Cargo.lock` were updated by hand for the
version bump — path dependencies have no resolution to redo — and the
lockfile-freshness check in CI validated the result.

## D22 — Update checks are opt-in and use one small HTTP dependency (2026-10-26)

The update check asks `https://api.github.com/repos/Perchemop/cadence/releases/latest`
for the latest release and compares its tag with the running version. It is
**opt-in**: nothing is requested on startup. `cadence version` reports only the
last cached answer; `cadence check-updates` and the GUI's **Updates → Check
now** button are the two explicit triggers. This follows the recorded
assumption that crash submission and update checks are opt-in, fail-soft
network features.

The request itself is a single bounded GET with a four-second connect/read/write
timeout, a `Cadence/<version>` user agent and a 256 KiB response cap. It sends
no project data, account, identifier or token. A successful answer is cached
in the system temporary directory for six hours; failures are reported in the
UI/CLI but never block or fail the application, and are not cached so a later
attempt can recover.

The one new direct dependency is `ureq` 2.12 with default features disabled and
only `tls` enabled. Hand-rolling TLS is not acceptable, and `ureq` is the
smallest maintained blocking client that fits the existing synchronous code.
Its rustls backend uses `ring` and the bundled WebPKI roots, so the release
binary stays self-contained and has no OpenSSL runtime dependency. The version
comparison is written in-crate (~40 lines) rather than adding `semver`: release
tags are a narrow `vMAJOR.MINOR.PATCH` shape, the parser handles the `v`
prefix, build metadata and pre-release ordering, and unit tests cover malformed
tags so a bad tag cannot claim an update.

The shared `update-check` crate remains deferred to the point where a second
application joins the suite (ROADMAP "Later"); for now the module lives in
`cadence-ui-egui` and the binary's CLI calls it.

## D23 — AccessKit through eframe, with the Table as the Gantt's semantic alternative (2026-10-26)

AccessKit support comes from `eframe` 0.35's default features (`accesskit` is
in the default set), so no extra dependency, feature flag or parallel
accessibility tree was added. Standard egui widgets expose their role, label,
enabled state and selected state through the platform adapter; Cadence does not
reimplement those semantics.

The custom-painted Gantt is the exception. It gets one summary node through
`Response::widget_info` (activity count, critical count, project dates, and a
legend explaining the critical marker and progress fill). Individual bars are
**not** separate AccessKit nodes: adding them would duplicate the Table view,
risk interfering with drag-to-link, and require a manual assistive-technology
pass that CI cannot perform. The Table view is the documented per-activity
semantic alternative, with one row per activity and text columns for duration,
start, finish, float and criticality.

Keyboard-only operation is treated as a first-class path, not a fallback:
`Up`/`Down` select activities in the Table, the inspector has an **Add
relationship** form that runs the same command as drag-to-link, and the Gantt
has keyboard zoom (`Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`). The remaining limits —
pointer/wheel panning of the Gantt canvas, status text that is not a live
region, an image-only Gantt PDF, and the need for manual screen-reader passes on
each release platform — are recorded in `ACCESSIBILITY.md` instead of being
claimed as complete.

## D24 — Progress-driven rescheduling is a separate engine mode (2026-11-02)

Status and earned value compare the plan against reality, so recorded progress
stays a fact and never feeds the default schedule (D15). Rescheduling a live
programme *from* that progress is a different question from planning it, and it
is answered by a **separate** engine function rather than a change to
`schedule`:

- `cadence_core::engine::schedule_with_progress(project, data_date)` is the
  explicit mode. `schedule` is untouched, so the variance views and the
  earned-value panel keep comparing plan against reality without circularity.
- `engine::activity_progress` classifies each activity from the recorded facts
  alone — complete, in progress, or not started — and returns the outstanding
  duration, rounded **up** so a partial day of work is never dropped.
- A complete activity is pinned to its recorded dates and drives its successors
  from them (the retained-logic convention). An in-progress activity starts at
  its actual start and its outstanding work resumes from the data date. A
  not-started activity is scheduled by logic, never before the data date.
- Complete work reports zero float; in-progress float is measured on the
  outstanding work, not on the past part of the activity.
- Upper-bound constraints are enforced only for work that has not started. A
  missed deadline on work already done or under way is left for the audit tool
  to flag: the recorded facts are never silently overridden, and a delay that
  has already happened is not a scheduling error.
- The CLI gains `cadence reschedule <file> [--status-date YYYY-MM-DD] [--json]`,
  and the GUI's **Production** view gains a non-destructive preview. Applying
  the rescheduled dates to the project is a later slice; the mode itself is the
  deliverable here.

## D25 — Per-assignment units and a single resource cost rate (2026-11-02)

D11 kept the resource model at exactly one unit per assignment and no cost
rates. This slice widens it by the smallest step that makes the common cases
expressible:

- `Resource` gains `cost_per_unit` (minor currency units per whole unit per
  working day; zero means no cost).
- The set of assigned resources stays `Activity::resources`. Units are a
  **sparse override**: `Activity::assignment_units` maps a `ResourceId` to its
  units in basis points (`10 000` = one whole unit), and an absent key means one
  whole unit. Every override names a resource in `resources`; `set_units_for`
  ignores an unassigned resource and never stores the default value, and the
  assignment/delete commands remove the override when the assignment goes away.
  Keeping the map sparse is what lets the assignment list stay a plain vector
  through SQLite and the interchange formats in the common case.
- The engine compares committed units in basis points against
  `capacity × 10 000`, so a capacity-one resource can host two half-unit
  assignments without over-allocation — the "half a crane" case D11 could not
  express. `ResourceConflict::required` and `capacity` are therefore reported in
  basis points.
- `Activity::resource_cost` rolls planned cost up as
  `units × cost_per_unit × duration`, saturating rather than overflowing.

Deliberately deferred (recorded in the roadmap as later work): time-phased rate
tables, cost-per-use, and per-assignment cost overrides. One rate per resource
is enough for a first cut and keeps the model small. The lossless JSON format
carries units and the rate both ways; MSPDI, MPX and CSV carry them only where
the format has a natural field and otherwise document the limitation.

## D26 — A yanked pin is moved by the lockfile workflow, not by hand (2026-11-03)

`async-executor 1.14.0` was yanked upstream on 2026-11-02 and superseded by
`1.14.1` the same day. It reaches the graph through
`accesskit_unix → accesskit_winit → egui-winit → eframe` (screen-reader
support), so no direct dependency of ours controls the pin. The advisories
check in the security workflow is a required status check and `yanked = "deny"`
in `deny.toml`, so the yank turned a green default branch red without any
change of ours.

Following D20, the remedy is the D19 regeneration, not an ignore: the committed
`Cargo.lock` is replaced with the freshly resolved graph, which pins
`async-executor 1.14.1` and takes the one coincidental transitive bump that
came with it (`async-lock 3.4.2 → 3.5.0`). The lockfile workflow's explicit
yank step now re-resolves `async-executor` alongside `clipboard-win`, so the
next yank of this package is moved by the workflow rather than by hand. No
advisory-ignore entry was added: relaxation would have hidden every future yank
instead of resolving this one.

## D27 — Adopt the guide §12.1 suite standards, with recorded deviations (2026-11-07)

Context: `guide.md` §12.1 ("Suite-wide Standards — audit approved 2026-11-04")
sets five standards every suite app must meet, and §II.28.3 requires any
deviation to be recorded here rather than silently aligned. This entry records
adoption, the current state, and the deviations still open.

Adopted as the target:

- **Token metrics ledger.** A CI-managed `docs/tokens.json` carrying
  `total_tokens_since_start`, `release_tokens` and `release_version`; the
  release notes reference the ledger values instead of free-text estimates.
  Seeded now with honest estimates; the CI update step on each semver tag is
  tracked in `ROADMAP.md`.
- **Shared `crates/update-check`.** Consumed as a pinned git submodule shared
  across the suite, updated with `git submodule update --remote` in CI.
- **Signed auto-update.** Releases signed and carrying `SHA256SUMS.txt`; the
  in-app updater verifies signature + checksum before install, and soft-fails
  offline without blocking start.
- **Version UI.** The version sits subtly in the **bottom-right** corner; a
  click opens the standard Release Notes popup (version history, per-release app
  usage, token metrics, Update button) with the same layout suite-wide.
- **README screenshots.** Screenshots refreshed at v1.0 and every major release.

Deviations recorded (not yet aligned, each tracked as roadmap work):

- `update-check` is currently an **inline module**
  (`crates/ui-egui/src/update_check.rs`), not the shared submodule. Moving it is
  a cross-repo change (submodule + CI pin) and is queued.
- Releases are **not yet signed**, and the updater opens the release page rather
  than verifying-and-installing. Signing and the verify-then-install flow are
  queued.
- The version is currently shown in the **status bar's right edge and the tool
  cluster**, not a dedicated bottom-right control; the Release Notes popup exists
  but its body is a hand-written summary, not the ledger-driven, all-versions
  history the standard specifies.

Consequences: the items above are the remaining §12.1 work; nothing here weakens
the existing gates (branch protection, the II.29 scan, CI-before-try). The
`"<App Name> Dev" <dev@example.com>` identity and the land-on-`main`-via-PR rule
(guide II.20.4, II.25) are reaffirmed unchanged.

## D28 — Adopt guide II.30 (Rust GUI/UX); the date picker uses jiff at the boundary (2026-11-09)

Context: `guide.md` gained **II.30 "Rust GUI/UX — stack matching, tokens and
evidence"** on 2026-11-09, and §12.3 makes guide synchronisation a process gate.
Cadence is a native **egui/eframe 0.35** desktop app, so II.30 applies directly.

Decision and adoption:

- **Stack record.** The toolkit and its **resolved** versions are recorded in
  [`docs/design/gui-stack.md`](docs/design/gui-stack.md) (egui/eframe 0.35.0,
  egui_extras 0.35.0 + `datepicker`, egui-phosphor 0.13.0), together with the
  shell shape and the layout model.
- **One design path (II.30.1, II.30.14).** Cadence uses only the project's own
  token module plus first-party `egui`/`egui_extras` and the `egui-phosphor`
  icon font. No shadcn-egui, spaceui, egui-material3, Fluent, or any web/CSS
  design system is added to this native GUI; those remain inspiration only.
- **Tokens (II.30.6).** `crates/ui-egui/src/design.rs` is the single token
  source (`palette(dark)`, type scale, spacing, accent). **Deviation recorded:**
  the theme module lives at `design.rs`, not a `src/theme/` directory, so an
  II.30.6 raw-colour grep gate must target `crates/ui-egui/src/design.rs` as the
  excluded path — wiring that gate is queued.
- **Iconography.** Adopt `egui-phosphor` (MIT) for command-bar, tab-strip and
  inspector icons, each rendered beside its text label so meaning is never
  icon-only (II.30.7.4); the glyph font and licence are recorded in
  `ATTRIBUTION.md`.
- **Date picker.** Adopt first-party `egui_extras::DatePickerButton` (II.30.4:
  prefer first-party widgets) but **confirm the API from the resolved version**:
  in 0.35 it works on `jiff::civil::Date`, not a string. Hence `jiff`
  (Unlicense OR MIT, allow-listed in `deny.toml`) is added and dates are
  converted to/from `time::Date` at the field boundary. The field stays
  **typeable** (a text edit beside the calendar button), keeping full keyboard
  entry and the inline invalid-date validation.
- **No toolkit bump.** This slice does not change `egui`/`eframe`; per II.30.3 a
  dependency move is its own green slice.

Deviations / consequences:

- `egui_kittest` visual assertions are **not** available yet: the shell renders
  through `eframe::App::update`, not a plain `&mut Ui` (II.27.12 row 1). A seam
  refactor to make that possible is its own slice; until then, evidence is the
  built app reviewed hands-on and offscreen captures as CI artefacts, never
  committed and with no pixel baselines (II.30.8).
- The §12.1 version-UI and token-ledger deviations from D27 are unchanged.

## D29 — Mandatory-constraint rounding (2026-11-10)

D10 deferred exact "must start/finish on" constraints until conflicts could
be resolved; they are now implemented, and this entry records their
semantics. `Constraint::MustStartOn(date)` is a **start-typed** pin: both the
early and the late pass round the bound **forward** to the next working day.
`Constraint::MustFinishOn(date)` is a **finish-typed** pin: both passes round
the bound **backward** to the previous working day. A start bound follows the
lower-bound direction and a finish bound the upper-bound direction — the same
convention D10 set for the four bound kinds (SNET, SNLT, FNET, FNLT), which
keep their recorded semantics unchanged.

Because both passes apply the same rounding to the same bound, they pin the
identical working day and the activity carries zero total float even when
the mandatory date falls on a weekend or holiday. The date is mandatory; the
day it lands on is the nearest working day on the activity's own calendar.
Alternatives considered: rejecting a bound that falls on a non-working day
(turns a data-entry nuisance into a scheduling error), and leaving the exact
date unpinned while reporting the collision (D10 already refuses to silently
emit negative float for an impossible hard constraint).

Defect fixed with this entry: the first implementation rounded the
`MustFinishOn` bound **forward** in the early pass only, while the late pass
rounded it backward, so a weekend bound produced two different pin dates and
phantom total float. The forward pass now rounds backward like the late
pass. The deterministic regression test
`mandatory_constraint_rounding_agrees_between_early_and_late_passes` pins
the weekend case, and the constraint property test now generates and checks
all eight kinds, including the two mandatory pins.
