# Whole-app end-to-end audit

Date: 2026-10-30. Scope: every crate — `cadence-core` (model, engine, calendar,
wbs, audit, compare, lookahead, clash, report, pdf), `cadence-commands`,
`cadence-persistence`, `cadence-interchange` (MSPDI, MPX, CSV, JSON, XML,
schedule, audit, compare), and the `cadence-app` CLI. Method: exhaustive read of
the source, each finding verified against the code before ranking. Read-only;
fixes are listed with their commit.

Severity: **CRITICAL** = data loss / corrupt output / reachable panic;
**HIGH** = wrong result or broken round-trip; **MEDIUM** = edge case;
**LOW/NIT** = polish.

## Fixed in this audit (CRITICAL)

| Location | Defect | Fix |
|---|---|---|
| `persistence/src/lib.rs` `clear_project` | It never deleted `lookahead_snapshots`, `lookahead_snapshot_entries` or `daily_notes`. A project containing a published lookahead snapshot or a diary note could not be **re-saved**: the second save re-inserted the same primary keys and the whole transaction failed (UNIQUE violation). Verified: a save→load→save round-trip was impossible. | Delete the three tables (children first) before the rest; added a regression test that saves twice with a snapshot and a note. |
| `commands/src/lib.rs` `DeleteActivity` | Deleting an activity left it referenced in baselines, lookahead snapshots and daily notes; on the next save those rows violated the foreign keys and the project became **unsavable**. | It now removes those references and `RestoreActivity` puts them back on undo. |
| `commands/src/lib.rs` `DeleteDailyNote` | The inverse re-created the note via `CreateDailyNote`, which allocates a **fresh id and appends**, so undo renumbered and reordered notes. | Added `RestoreDailyNote` to restore the original identifier and position. |
| `core/src/pdf.rs` `image_page` | The image XObject declared **no `/Filter /ASCIIHexDecode`** while its payload was ASCII-hex encoded, and closed **without `endstream`**. Every Gantt-print PDF was corrupt. | Mirror the text `stream()` encoding (filter + `endstream`); added a test asserting both markers. |

## Open findings

### Core domain & engine

- **CRITICAL** `model.rs:1247-1248` — EVM `bac * fraction` / `bac * pct` overflow in
  `i64`; a project file with a large `budget_at_completion` panics in debug or wraps
  in release. Use `i128`/`checked_mul`.
- **CRITICAL** `engine.rs:577-580` — project EVM totals accumulate unbounded user
  values in `i64`. Accumulate in `i128`.
- **CRITICAL** `engine.rs:264,354` — `checked_add(...).expect("…date overflow")` is a
  reachable panic for extreme dates (e.g. a milestone at 9999-12-31 with a resource).
  Return a typed error instead.
- **HIGH** `model.rs:1245` — `percent_complete` is never range-checked before use.
- **HIGH** `engine.rs:249-251` — duplicate resource ids on one activity double-count
  and report a false over-allocation.
- **HIGH** `engine.rs:249,342` — per-calendar-day loops scale with duration; a huge
  duration hangs resource analysis.
- **HIGH** `engine.rs:832` — cross-calendar forward/backward and free-float
  asymmetry (see D6); late/float can be inconsistent for mixed calendars.
- **HIGH** `model.rs:826` — `next_activity_id` is `max+1` with no persisted
  watermark, so deleting the highest-id activity re-issues its id, silently
  retargeting baseline/lookahead/diary rows keyed on it.
- **MEDIUM** silent calendar fallback (`engine.rs:244`), `Calendar` serde bypasses
  validation (`model.rs:218`), free-float clamp inconsistency (`engine.rs:970`),
  leveling baseline uses ALAP `scheduled_*` (`engine.rs:431`).

### Commands & persistence

- **HIGH** `persistence` `ensure_schema` — migration arms run outside a transaction
  with non-idempotent `ALTER TABLE ADD COLUMN`; an interrupted migration can leave a
  file that never loads again. Wrap in a transaction / guard each `ADD COLUMN`.
- **HIGH** `commands` `SetCalendarDefinition` — removes then re-adds the calendar,
  appending it to the end; undo does not restore the original position.
- **MEDIUM** `persistence` — resource-assignment order, and the order of resources,
  baselines, lookahead snapshots and daily notes, are not round-tripped (no
  `position` column).
- **MEDIUM** `persistence` — WBS/calendar `next` counters are not persisted, so a
  `Project` can differ from the same project after a save/load round-trip.
- **MEDIUM** `persistence` `validate` — does not reject dangling
  `daily_notes.activity_id` / `baseline_snapshots.activity_id` /
  `lookahead_snapshot_entries.activity_id`, nor duplicate ids.
- **MEDIUM** `commands` — several `clear_schedule` inverses do not restore the
  previously computed schedule; undo returns the project to a different (un-scheduled)
  state. Document or snapshot derived fields.
- **LOW** empty activity names accepted (`commands:201,1399`); `SetActivityPromise`
  keeps a `delay_reason` when not promised; `RestoreResource` can no-op silently.

### Core features (audit / compare / lookahead / clash / report / pdf)

- **HIGH** `audit.rs:724` — the invalid-dates check flags **every in-progress**
  activity (the engine never reads actual dates), so a healthy programme fails check 9.
  Exclude started work; also test the forecast finish.
- **HIGH** `compare.rs:201,319` — `compare_baseline` reports **every relationship as
  Added**; the doc promises the section stays empty.
- **MEDIUM** `compare.rs:191,292` — baseline comparison emits false constraint
  changes (rebuilt `before` defaults to ASAP); it also drops baseline-only activities
  so `removed_activities` is always empty.
- **MEDIUM** `audit.rs:462` — lags use a mismatched percentage basis
  (open-successor numerator vs all-relationship denominator).
- **MEDIUM** `audit.rs:270` — `is_open` and `is_finished_by` disagree, so the
  missed-task and BEI checks contradict on the same activity.
- **MEDIUM** `report.rs:42` / `pdf.rs:361` — non-ASCII (the em dash) renders as
  mojibake in the PDF report; base-14 Helvetica has no such glyphs.
- **MEDIUM** `audit.rs:369,581,624,886` — quadratic `activity()`/`schedule.activity()`
  lookups violate the "linear-ish" requirement for large programmes.
- **MEDIUM** `lookahead.rs:74` — finished activities are not filtered, contradicting
  the module doc; `clash.rs:78` does not test working days or exclude milestones.
- **LOW** `audit.rs` — no per-activity finding dedupe; negative-float includes closed
  activities; critical-path test measures movement on the wrong calendar.

### Interchange & CLI

- **HIGH** `json.rs` — JSON, documented as the only complete format, drops
  `Activity.location`, `Project.lookahead_snapshots`, `Project.daily_notes` and the
  default `Project.calendar`. Add them to export/import (and update the "complete"
  claim).
- **HIGH** `app/src/main.rs:208` — `export_interchange` writes to `output` with no
  guard that `input != output`; `cadence export-xml proj.sqlite proj.sqlite` destroys
  the project file. Same class in `import_interchange` and the report/print paths.
- **HIGH** `csv.rs:481` — CSV export does not neutralise leading `=`, `+`, `-`, `@`
  (formula injection when opened in a spreadsheet).
- **MEDIUM** `main.rs:136` — the documented `audit --json --status-date …` order is
  rejected; a misplaced `--json` is treated as a filename; `audit --json` emits no
  machine-readable error document.
- **MEDIUM** `mspdi.rs:175` / `mpx.rs:184` — resource `Type 2` (cost) imports as
  `Equipment`.
- **MEDIUM** `mspdi.rs:561` — unknown duration units silently import as 0 days.
- **MEDIUM** `csv.rs:199` — a CSV with no date defaults the project start to *today*,
  so the same file imports differently on different days.
- **MEDIUM** `json.rs:435` — `percent_complete` out of range is silently dropped;
  MSPDI clamps instead.
- **MEDIUM** `xml.rs:93` — no size/depth cap on the XML reader (JSON caps depth).
- **LOW/NIT** lax JSON number grammar; MPX negative/oversized values saturate;
  duplicate sections only partly read; NIT `MaxUnits` written as a bare number.

## Fields lost on interchange (before fixes)

`✓` carried · `✗` lost · `~` quantised. Derived schedule dates are intentionally
excluded (D9).

| Field | MSPDI | MPX | CSV | JSON |
|---|---|---|---|---|
| owner / promised / delay_reason | ✗ | ✗ | ✗ | ✓ |
| location | ✗ | ✗ | ✗ | ✗ |
| lookahead_snapshots | ✗ | ✗ | ✗ | ✗ |
| daily_notes | ✗ | ✗ | ✗ | ✗ |
| default Project.calendar | ✗ | ✗ | ✗ | ✗ |
| baselines | ✓ | ✗ | ✗ | ✓ |
| wbs / named calendars | ✗ | ✗ | ✗ | ✓ |
| resource kind | ~ | ~ | ✗ | ✓ |

## Verified sound

Kahn cycle detection and path recovery; FS/SS/FF/SF forward and backward formulas
including `+1` FS and negative lags; calendar working-day arithmetic and holidays;
duplicate activity/relationship detection at the engine level; SQL parameter binding
(no injection); `to_sql_id`/`from_sql_id` range checks; every migration arm 1→8
reaches `user_version = 8` with the same columns as `SCHEMA`; and all recently added
fields (`owner`, `promised`, `delay_reason`, `location`) appear in both the writer
and the reader.
