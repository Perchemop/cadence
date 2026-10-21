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
`PRAGMA user_version` (currently 2). Opening a file with a newer version is
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
