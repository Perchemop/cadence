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

**Extension (2026-11-13):** the progress-driven mode
(`schedule_with_progress`) now applies the same pins to **not-started**
activities — the same D10 rounding in both passes, zero total float, and an
explicit pin overriding both network logic and the data-date floor. Recorded
actual dates are never moved by a pin: a pin cannot turn a fact into a plan.

## D30 — The assistant's HTTP backends: four wires, keyless Ollama, no test sockets (2026-11-17)

The second `cadence-assistant` slice (owner-approved research
`docs/research/ai-integration.md`, 2026-10-27) adds the network half behind the
D-seam `AssistantProvider` trait. Decisions:

- **Four wire protocols**, one per configured provider name: OpenAI
  chat-completions (also the protocol spoken by any other non-empty,
  non-`ollama`/`anthropic`/`gemini` name — the de-facto compatibility
  standard), Anthropic Messages (`x-api-key` + a pinned `anthropic-version` +
  a required `max_tokens`), Ollama's native `/api/chat` (no auth, stream
  disabled), and Gemini `generateContent` (`x-goog-api-key`, model in the
  URL). Shapes were written from the providers' published API references
  (October 2026), not from memory.
- **A model is now required** (`AssistantConfig.model`, serde default so old
  settings files still parse), and an OpenAI-compatible server requires an
  explicit `base_url`. `is_configured` enforces both, so `from_config` fails
  closed with `NotConfigured` — the UI keeps gating on the one function.
- **A local Ollama needs no API key**: `is_configured` exempts it, and the
  request carries no `Authorization` header at all.

**Extension (2026-11-18, from the second-opinion review):** a keyless
**OpenAI-compatible** server — any custom provider name with its own
`base_url` — is also valid without a key. Local inference servers (LM
Studio, vLLM) commonly run without one, and the wire layer already omits
every credential header when no key is set; `is_configured` now matches
that, so the advertised keyless use actually configures. Credential headers
are likewise omitted entirely (never emitted empty) when keyless, and
transport-level failures carry a fixed plain-language phrase rather than
the transport error's own text, which can echo request context.
- **`ureq` is reused** (same workspace dependency, same rustls feature set as
  the update check) — no new dependency families. Connect timeout 5 s (fail
  fast like the update check), read timeout 120 s (generation is genuinely
  slow), write 30 s.
- **Error mapping, plain language:** 401/403 and other 4xx are `Refused`
  (a configuration problem); 408/429/5xx and transport faults are `Network`,
  with the provider's own short JSON error message as detail (truncated to
  200 chars) — never the raw body, and never request material, so no error
  can echo the key. Key handling stays with `SecretKey`; a test asserts the
  key never reaches a request body or URL on any wire.
- **No test opens a socket** (flaky/network policy, M.5): request building and
  response parsing are pure functions asserted directly; the only untested
  step is the thin `ureq` glue in `complete`, documented as such.

The workspace version moves 0.14.0 → 0.15.0 (public `AssistantConfig` gains
two fields — `constructible_struct_adds_field`), and `Cargo.lock` is
regenerated from the Lockfile workflow's artefact (D26 path), because the
crate now consumes `serde_json` and `ureq` at runtime.

## D31 — Development-time agent tooling adopted: caveman, pstack, rtk (2026-11-18)

Owner-directed adoption of the guide §19 token-efficiency skills, the §22
pstack rigour pack, and the rtk output proxy (guide §19 table). All three are
**maintainer-machine tooling only** — nothing here ships in the Cadence
binary or the release assets.

- **caveman skill pack** — `JuliusBrussee/caveman`, licence **Apache-2.0**
  (verified from the repository's LICENSE file; the §19 table's "MIT" is
  superseded by the source). 22 skills installed with
  `npx skills add JuliusBrussee/caveman -a opencode` into
  `~/.config/opencode/.agents/skills/`. Every skill in the pack sets
  `disable-model-invocation: true`, so the terse style applies only when
  explicitly invoked — installing changes nothing by itself.
- **pstack skill pack** — `cursor/plugins/pstack`, licence **MIT**
  (© Lauren Tan, verified from `pstack/LICENSE`). 58 skills including the
  `poteto-mode` entry point, installed the same way; equally
  invocation-gated.
- **rtk 1.2.0** — `rtk-ai/rtk`, licence **Apache-2.0**. No brew on this
  machine and `cargo install` is banned, so the **prebuilt static musl
  binary** was taken from the project's GitHub releases. The x86_64 build
  does not run here — the machine is **aarch64** — and the
  `aarch64-unknown-linux-musl` build was installed to `~/.local/bin/rtk`
  instead (`rtk --version` verified).
- **Measured-only rule (§19 rule 3):** no vendor token-savings figure is
  recorded anywhere in this repository. Any claim about these tools' token
  effects must come from this project's own measured runs and live only in
  `docs/tokens.md`.
- **Clean-room unchanged (§19 rule 5):** these are style, rigour and
  efficiency aids; everything they help produce stays subject to the same
  clean-room test, CI gates and review as any other change.
- **One context tool at a time (§19 rule 6):** rtk wraps command *output*;
  no indexing or compression layer is installed alongside it. If that
  changes, record the reason here first.

## D32 — The §5.6.3 workflow commands are repository files, not private config (2026-11-18)

The seven OpenCode workflow commands — `/grill-me`, `/to-spec`,
`/to-tickets`, `/tdd`, `/implement`, `/implement-spec`, `/code-review` —
are committed to `.opencode/command/` rather than kept only in the
maintainer's private `~/.config/opencode/`. They encode the guide §5.6.3
specifications, and each file carries a pointer to the §5.6.5
reconciliation table, so a command file cannot bypass the guide's rules:
the stricter rule wins, and the guide wins over any command file (guide
Authority preamble).

Consequences:

- Any contributor cloning the repository gets the same disciplined
  workflow; the command files are versioned and reviewed like code (§5.5
  item 12 — instructions readable by everyone, no private-machine notes).
- The files restate rules that also live in the local guide; when the
  guide changes, this directory is synced in the same session (§12.3), and
  the §5.6.5 pointer in every file is the backstop that keeps drift
  harmless.
- `ci.sh`'s default ignore list excludes `.opencode/**` from its
  auto-commit (by design, so agent-local configuration never leaks into a
  commit). Repository-tracked `.opencode/` content is therefore committed
  directly with the asserted `Cadence Dev <dev@example.com>` identity and
  then verified through `ci.sh` like any other change — the recorded
  precedent in this repository (42e7677, "agents: add a dedicated UI/UX
  designer subagent").
- `docs/reference/index.md` (guide §5.7.2) is added as the agent-team
  documentation pre-loading index — one line per `docs/` file, grouped —
  and is regenerated whenever the docs tree changes.

## D33 — Release-artefact signing: signed SHA256SUMS.txt, runner-side key custody (2026-11-18)

Every release's `SHA256SUMS.txt` is signed with minisign and the signature is
attached as `SHA256SUMS.txt.minisig` next to the checksums, satisfying the
guide §7.2 signature requirement and the "signature attached before the
release is published" directive.

- **Key custody — runner-side only.** The signing keypair is generated by the
  new hand-triggered `Signing key rotation` workflow (`.github/workflows/
  signing-key.yml`, workflow_dispatch only) on the GitHub Actions runner and
  stored via `gh secret set` into the repository secrets `RELEASE_SIGNING_KEY`
  and (optional) `RELEASE_SIGNING_PASSWORD`. No key is ever generated on,
  copied to, or printed from a developer machine; the job summary shows only
  the public key, key ID and fingerprint, and the runner's key material is
  removed when the job ends. The human triggers the first rotation.
- **The rotation workflow needs a PAT.** The workflow's own `GITHUB_TOKEN`
  has no secret-management permission (GitHub REST docs: repository-secrets
  writes require the classic `repo` scope or a fine-grained PAT with
  "Secrets: read and write"), so the rotation workflow authenticates with a
  third secret, `SECRET_MANAGEMENT_TOKEN`, set once by the maintainer.
  Recorded 2026-11-18 from docs.github.com (Actions secrets REST API).
- **Signing lives inside the existing release job, before publishing.** The
  release job already generated `SHA256SUMS.txt` (hand-rolled, sorted,
  flattened, never checksumming itself), so signing extends that job rather
  than duplicating checksum generation in a second job. A missing
  `RELEASE_SIGNING_KEY` fails the release with a plain-language guard instead
  of publishing unsigned. After signing, the workflow recreates the public
  key from the stored secret key (`minisign -R`) and verifies the fresh
  signature with it, so a corrupted or wrong-key secret fails the release,
  not the user's verification.
- **Pinned tool.** minisign **0.12**, official asset
  `https://github.com/jedisct1/minisign/releases/download/0.12/minisign-0.12-linux.tar.gz`,
  SHA-256 `9a599b48ba6eb7b1e80f12f36b94ceca7c00b7a5173c95c3efc88d9822957e73`
  (computed from that asset on 2026-11-18; the runner checks it before use).
  URL and asset name verified against the project's public releases page, and
  the non-interactive password behaviour in the minisign 0.12 source
  (`get_line.c`: without a TTY the password is read as a plain stdin line;
  `-W` stores an unencrypted key).
- **App-side verification is a later slice.** Once `crates/ui-egui` frees
  up, the updater will verify the downloaded installer against
  `SHA256SUMS.txt` and the minisign signature, pinning the public key as a
  compile-time constant; until then the signature serves manual verification.
  Because of that compile-time pin, a key rotation must ship the new public
  key in a new app release before further artefacts are signed with the old
  key (§7.2), and `docs/update-signing.md` carries the current-fingerprint
  placeholder to be filled after the first rotation.
- **Doc placement.** `docs/dev.md` does not exist in this repository; the
  release-signing notes and the secret names went into `docs/development.md`,
  which already serves as the repository's development and secrets
  documentation.



## D34 — Token ledger: measured usage from local OpenCode records, aggregated locally, validated in CI (2026-11-18)

The token ledger (guide §24) is now **measured**, not seeded. The
`docs/tokens.json` seed values (~1.2M total / ~45K per release) were
placeholder estimates; the measured-only rule (guide §19 rule 3) requires
every number to come from actual records, so they are replaced by real
aggregates and the ledger gains a per-release table in `docs/tokens.md`.

- **Data source.** The local OpenCode session store on the maintainer's
  machine — a SQLite database, by default
  `~/.local/share/opencode/opencode.db`. Its `session_v2` table carries
  per-session token aggregates (`tokens_input`, `tokens_output`,
  `tokens_reasoning`, `tokens_cache_read`, `tokens_cache_write`) with
  millisecond timestamps; `project.worktree` identifies the repository a
  session belonged to.
- **Design.** CI cannot read a local store, so the work is split:
  `scripts/token-ledger.py` (stdlib-only, run by the maintainer before
  tagging a release) aggregates the records and rewrites
  `docs/tokens.md` + `docs/tokens.json` together; a CI step ("Token ledger
  consistency" in `ci.yml`) validates that the two committed files are
  mutually consistent and that the JSON shape the app's build-time
  release-notes reader depends on is intact. **CI validates only — it
  never computes token numbers.**
- **What counts.** The sum of the store's five per-session token columns
  for sessions attributable to this repository by path (the checkout and
  its `cadence-*` worktrees), bucketed by release-tag date into cycles
  `(previous tag, this tag]`; sessions that started before the first tag
  fall into the first release's cycle.
- **Privacy constraints (binding on the script).** It reads only numeric
  usage columns and path/timestamp metadata; it never reads message or
  part tables, message JSON, session titles, or any content that could
  carry conversation text. It opens the store read-only (falling back to a
  read-only temp copy when the live store is locked) and performs no
  network access.
- **Known limitation — the ledger is a floor, not a ceiling.** Sessions
  launched from the home directory (orchestrator sessions and their
  subagents) are not attributable to any repository by this store and are
  excluded. The measured total is therefore a conservative underestimate
  of the true development cost, and the numbers can only grow when the
  script is re-run after more attributable work. This limitation is stated
  in `docs/tokens.md` next to the numbers.
- **Compatibility.** `total_tokens_since_start`, `release_tokens` and
  `release_version` keep their names and meaning (the app's release-notes
  reader in `crates/ui-egui/src/lib.rs` deserialises them); the guide §24
  `releases` array is additive — serde ignores unknown fields. CI asserts
  the reader keys are present so a future schema change cannot silently
  break the in-app popup.
## D35 — User-directed skills: relocation, invocation policy, and the pstack licence verification (2026-11-19)

The owner-directed skill packs (D31) were installed into
`~/.config/opencode/.agents/skills/`, which this OpenCode build does not load —
invocation failed with "Unable to load skill" for both `poteto-mode` and
`caveman`. The packs were **relocated to `~/.config/opencode/skill/<name>/`**
(the location the working built-in skills occupy), after which the registry
hot-reloaded and `poteto-mode` loaded successfully in-session. Lesson recorded:
the upstream `npx skills add` target directory is not the directory this
runtime loads.

- **Invocation policy (owner directive, codified in the global ruleset
  2026-11-19):** user-directed skills are invoked automatically when a task
  matches their trigger; skipping a trigger match is a process defect. The
  `caveman` family stays manual-only (owner toggles the style), per the
  policy's plain-language guard (II.22).
- **pstack licence verified from source:** `pstack/LICENSE` and
  `plugin.json` both declare **MIT** (© Lauren Tan). The "Apache-2.0" claim in
  guide §22.2.3 (2026-11-18 revision) is superseded by the licence file —
  §25 use-rule 3: the source repo's LICENSE governs. ATTRIBUTION.md already
  records MIT; no change needed there.
- **§19/§22 posture adopted (guide 2026-11-19):** reference-catalog,
  adapt-don't-install. The D31 installation used `npx skills add -a opencode`
  (an OpenCode target, not Claude Code), and the packs now live in the
  §19-sanctioned directory; the relocation completes the adaptation.
- **Repo-level enforcement:** `.opencode/agents/orchestrator.md` carries the
  invocation policy so every future Cadence session auto-invokes per trigger.

No shipped artefact changes: these are maintainer-machine and process files.

## D36 — Token ledger attribution: two-tier parent-chain mapping (2026-11-19)

D34's ledger was honest but coarse: it attributed sessions by path only
(checkout + `cadence-*` worktrees), which measured a floor of 6,790,816
tokens and silently excluded every orchestrator session and its subagent
tree — the bulk of real development usage, launched from the home directory.
The store's `session_v2.parent_id` links make the missing mapping computable,
so the ledger gains a second attribution tier.

- **Tier A — direct (strict floor, unchanged rule plus one widening).** A
  session is directly attributable when its own directory, its project's
  `worktree`, or a directory the store's `worktree` table lists for one of
  the repository's projects, is the checkout or a `cadence-*` sibling
  worktree. The `worktree`-table widening catches opencode-managed worktree
  paths whose names carry no repo prefix (e.g.
  `~/.local/share/opencode/worktree/<id>/shell-polish`). Tier A is reported
  per release as `tokens_direct` (JSON) and the "Direct (Tier A)" column
  (tokens.md).
- **Tier A+B — orchestration-inclusive (headline).** Seed = the Tier-A set;
  add every transitive ancestor (the orchestrators that spawned direct
  sessions) and every transitive descendant (further subagents) over the
  parent links. Siblings — other children of an ancestor — are neither and
  stay excluded. Release rows and `total_tokens_since_start` report Tier A+B.
- **Measured result (store read 2026-11-19):** Tier A 10 sessions /
  6,790,816 tokens (identical to the D34 floor, confirming no regression);
  Tier A+B 12 sessions / 1,711,676,910 tokens — the two ancestor `build`
  orchestrators carry 1,704,886,094 tokens, bucketed by start time into the
  0.3.0 (+1,125,404,657) and 0.11.0 (+579,494,770) cycles; 832 of 844 store
  sessions remain excluded.
- **Accepted caveat — conservative over-attribution.** An orchestrator that
  worked across multiple repositories attributes wholly to Cadence under the
  ancestor rule: the store records no per-repository split inside a session.
  Tier A+B therefore leans high; the direct Tier-A figure is the strict
  lower bound. Both are measured; the ledger stays accuracy-over-precision
  with the floor/ceiling honesty stated next to the numbers.
- **Unreleased-cycle consistency fix (same machinery).** The D34 script added
  after-newest-tag sessions to `total_tokens_since_start` while CI asserted
  `total == last release's tokens_cumulative` — a latent contradiction that
  would turn CI red on the first mid-cycle regeneration. The ledger now
  reports `tokens_unreleased` / `tokens_direct_unreleased` additively and CI
  validates `total == last cumulative + tokens_unreleased`.
- **Privacy wall reaffirmed and extended.** The script still reads only
  numeric usage columns plus path/timestamp/ID metadata — now including
  `parent_id`, an opaque ID used solely to build the attribution graph. It
  never reads message or part tables, message JSON, session titles, or any
  content that could carry conversation text; the store is opened read-only
  and no network access is performed.
- **Compatibility.** The app reader's three keys keep their names and
  meaning (`total_tokens_since_start` is now the Tier A+B headline); all new
  keys and the `tokens_direct` release field are additive (serde ignores
  unknown fields). CI's "Token ledger consistency" step validates the new
  fields and the extended tokens.md table — validation only, CI never
  computes token numbers.

## D37 — Assistant review-panel slice: settings file location and test seams (2026-11-19)

RP-1 and RP-2 of the review-panel spec
(`docs/specs/review-panel.spec.md`) landed in `crates/assistant`:
`prompt.rs` (the `build_proposal_prompt` / `parse_proposal_response` seam)
and `settings.rs` (the `settings_path` / `load_settings` /
`store_settings` storage). TDD evidence: RP-1 red run 44500433308 (commit
4013487, 5 tests failing for the expected reasons on all four Verify
platforms) and green run 44504244460 (commit 85d3437, `=== CI RESULT:
SUCCESS ===`); RP-2 red run 44511538745 (commit adef464, 5 expected
failures on Ubuntu, 4 on Windows and macOS) with its green run recorded in
PROGRESS.md. Decisions the tickets left open:

- **Settings file name and layout.** The tickets fixed the platform
  directory rules but not the file name: the module stores
  `<platform config dir>/cadence/assistant-settings.json`. An app-named
  subdirectory follows the XDG convention (applications do not clutter the
  config root) and gives the file a self-describing name;
  `store_settings` creates the directory chain.
- **No `tempfile` dev-dependency.** The RP-2 ticket requires no new
  dependency (lockfile unchanged), so the tests hand-roll a unique temp
  directory (pid + nanos) under the platform temp dir and remove it
  afterwards. Tradeoff: ~15 lines of test helper instead of a
  battle-tested dev-dep; acceptable at this scale and trivially
  replaceable later.
- **Test seams without environment mutation.** `std::env::set_var` is
  process-global and racy under the test harness's parallel threads, so
  `config_dir` takes the environment lookup as a closure and load/store
  are path-parameterised private helpers the public functions delegate
  to. Tests inject values and write only inside their own temp directory,
  exactly as the ticket's acceptance requires. The platform branch
  conditions use `cfg!`, so every branch compiles on every platform.
- **No `PartialEq` on `AssistantConfig`.** The corrupt-file test asserts
  field-by-field instead of whole-struct equality, avoiding a production
  type change outside the ticket's declared target files.

No new dependencies; the lockfile is unchanged.

## D38 — Signing-key bootstrap deviation and the minisign positional fix (2026-11-19)

The one-time signing setup hit two facts worth recording:

- **Bootstrap deviation (owner-directed "do it for me"):** the rotation
  workflow's self-store step requires a `SECRET_MANAGEMENT_TOKEN` (a PAT the
  owner had not yet created), and PATs cannot be created programmatically.
  The release was otherwise ready, so the keypair was generated locally in a
  single pipeline that uploaded the secret key with `gh secret set` and
  shredded the local material in the same command chain. End state per §7.2:
  the private key's only resting place is the repository secret
  `RELEASE_SIGNING_KEY`; no copy remains on any machine; the key never
  appears in logs, arguments, or the repository. The runner-side rotation
  workflow (D33) remains the standing procedure for every future rotation;
  the `SECRET_MANAGEMENT_TOKEN` one-time setup is still recommended so future
  rotations never touch a developer machine. The public key, key ID
  `5F50A5EC98109664` and fingerprint
  `1eeba006282e225fb95ed5f47f4cb939a3e56603349bb00ce903290941ac47db` are
  recorded in `docs/update-signing.md`, and v0.15.0 is the first release
  shipped with this key.
- **Minisign 0.12 positional argument finding:** `minisign -S/-V` with a bare
  positional file argument exits 0 **silently without signing or verifying**
  (observed on the Debian 0.12-1 arm64 build during the self-test). The
  release workflow's sign and verify steps used that form; both now pass
  `-m <file>` explicitly. The workflow's post-sign self-verify would have
  caught a silent no-op at release time (fail loud, never publish unsigned),
  but the fix removes the wasted release round.

