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

## D10 — Constraint semantics (2026-10-21)

Activities support ASAP (default), ALAP, SNET, SNLT, FNET, and FNLT.
Constraint dates are inclusive bounds interpreted on the activity's own
calendar: lower bounds round forward to the next working day, upper bounds
round backward to the previous working day. Exact "must start/finish on"
constraints are deferred until the UI can resolve conflicts.

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
