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

## Fixed in the second pass (parallel fix agents)

| Location | Defect | Fix |
|---|---|---|
| `persistence` `ensure_schema` | Migration arms ran outside a transaction with non-idempotent `ALTER TABLE ADD COLUMN`; an interrupted migration left a half-applied file that never loaded again. | Each migration arm now runs inside one transaction (`unchecked_transaction` + commit), so DDL and the version stamp commit or roll back together. |
| `commands` `SetCalendarDefinition` | Removed then re-added the calendar, appending it to the end; undo did not restore the original position, so calendar order drifted on every apply/undo/redo. | The captured position now drives the forward apply via `CalendarSet::restore`, and undo/redo go through the same path. |
| `commands` `SetActivityCalendar` / `SetCalendarDefinition` | Changed working days without clearing cached scheduled dates/floats, leaving stale schedule values on affected activities. | Both now call `clear_schedule()` on the affected activities (all activities using a redefined calendar). |
| `commands` `CreateActivity` / `RenameActivity` | Accepted empty/whitespace activity names, unlike every other entity command. | Rejected with `"activity name must not be empty"`, validated before mutation. |
| `compare` `compare_baseline` | Reported **every relationship as Added** (the rebuilt before-side was relationship-free), emitted false constraint changes (rebuilt activities defaulted to ASAP), and never reported baseline-only activities as removed. | The before-side now retains current relationships and constraints so both sections stay empty as documented, and is seeded from the baseline snapshots so a snapshot with no surviving activity is reported as removed. |
| `report` / `pdf` | Generated report text embedded an em dash and other non-ASCII punctuation; the base-14 PDF font has no such glyphs, so the PDF rendered mojibake. | A `transliterate` step maps typographic punctuation to ASCII before both outputs; ASCII passes through untouched. |
| `audit` invalid-dates | Flagged every in-progress activity as "forecast start before the status date" (the engine ignores actual dates), and never tested the forecast finish. | Started activities are excluded from the start comparison; an open activity whose forecast finish precedes the status date is now flagged. |
| `audit` negative float | Iterated all activities, so a completed activity with negative float failed the check. | Restricted to open activities, matching the high-float check. |
| `audit` lags | Numerator counted lagged relationships with an open successor while the denominator counted all relationships. | Both populations are now every relationship (the DCMA convention), stated in the check description. |
| `model.rs` / `engine.rs` EVM | `bac * fraction` and the project totals overflowed `i64` (panic in debug, wrapped cost in release); `percent_complete` was never range-checked. | Computed in `i128` with `percent_complete` clamped to 0–10 000 and results saturated to `i64`; covered by tests at activity and project level. |
| `engine.rs` resource analysis | `checked_add(...).expect("…date overflow")` was a reachable panic for extreme dates, and a duplicated resource id on one activity was counted twice, reporting a false over-allocation. | Date advance now stops the loop instead of panicking, and `distinct_resources` deduplicates before counting. |
| CLI `export_interchange` / `import_interchange` / `print_report*` | Wrote to the output path with no guard that it differs from the input, so `cadence export-xml proj.sqlite proj.sqlite` destroyed the project file. | `ensure_distinct_files` rejects identical paths (textual or canonicalised) before any read or write. |
| CLI `--json` handling | Only accepted `--json` as the last argument, so the documented `audit <file> --json --status-date <date>` failed; a stray `--json` was treated as a filename; failures left stdout empty. | `--json` is accepted in any position, a misplaced one is rejected, and `audit`/`audit-compare` now emit a machine-readable `kind: "error"` document on failure. |
| CLI `parse_status_date` | Duplicated date parsing and reported a generic usage error. | Reuses the interchange `parse_date` and names the offending value. |

## Open findings

### Core domain & engine

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

- **HIGH** `commands` `SetCalendarDefinition` — removes then re-adds the calendar,
  appending it to the end; undo does not restore the original position.
- **MEDIUM** `persistence` — resource-assignment order, and the order of resources,
  baselines, lookahead snapshots and daily notes, are not round-tripped (no
  `position` column).
- **MEDIUM** `persistence` — WBS/calendar `next` counters are not persisted, so a
  `Project` can differ from the same project after a save/load round-trip.
- **MEDIUM** `persistence` `validate` — does not reject duplicate ids; the
  dangling-reference checks added in this audit cover activity links.
- **MEDIUM** `commands` — several `clear_schedule` inverses do not restore the
  previously computed schedule; undo returns the project to a different (un-scheduled)
  state. Document or snapshot derived fields.
- **LOW** empty activity names accepted (`commands:201,1399`); `SetActivityPromise`
  keeps a `delay_reason` when not promised; `RestoreResource` can no-op silently.

### Core features (audit / compare / lookahead / clash / report / pdf)

- **MEDIUM** `audit.rs:369,581,624,886` — quadratic `activity()`/`schedule.activity()`
  lookups violate the "linear-ish" requirement for large programmes.
- **MEDIUM** `pdf.rs:361` — `escape_pdf` passes bytes ≥ 0x80 through and only escapes
  `(`, `)` and `\`; other control characters are written raw.
- **MEDIUM** `lookahead.rs:74` — finished activities are not filtered, contradicting
  the module doc; `clash.rs:78` does not test working days or exclude milestones.
- **MEDIUM** `audit.rs` — no per-activity finding dedupe (two `Finding::Activity`
  rows can be pushed for one activity in a single check).
- **LOW** critical-path test measures movement on the activity's calendar while
  `project_finish` may come from the project calendar.

### Interchange & CLI

- **HIGH** `json.rs` — JSON, documented as the only complete format, drops
  `Activity.location`, `Project.lookahead_snapshots`, `Project.daily_notes` and the
  default `Project.calendar`. Add them to export/import and update the "complete"
  claim. *(fix agent in progress)*
- **HIGH** `csv.rs:481` — CSV export does not neutralise leading `=`, `+`, `-`, `@`
  (formula injection when opened in a spreadsheet). *(fix agent in progress)*
- **MEDIUM** `mspdi.rs:175` / `mpx.rs:184` — resource `Type 2` (cost) imports as
  `Equipment`. *(fix agent in progress)*
- **MEDIUM** `mspdi.rs:561` — unknown duration units silently import as 0 days.
- **MEDIUM** `csv.rs:199` — a CSV with no date defaults the project start to *today*,
  so the same file imports differently on different days.
- **MEDIUM** `json.rs:435` — `percent_complete` out of range is silently dropped;
  MSPDI clamps instead.
- **MEDIUM** `xml.rs:93` — no size/depth cap on the XML reader (JSON caps depth).
  *(fix agent in progress)*
- **MEDIUM** `mspdi.rs` / `mpx.rs` — no duplicate-activity-id validation on import
  (csv.rs does reject duplicates). *(fix agent in progress)*
- **LOW/NIT** lax JSON number grammar; MPX negative/oversized values saturate;
  duplicate sections only partly read; NIT `MaxUnits` written as a bare number;
  `crash_report` log grows without rotation.

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

## Capability audit against the reference products

A separate, exhaustive capability audit compares Cadence against every feature,
capability, button and action named in the reference-product inventories
(Microsoft Project, Primavera P6, Asta Powerproject, ProjectLibre, Aphex, the
DCMA/GAO/AACE audit tools and the optional AI tools). It inventories 273
distinct capabilities and records each as `shipped` / `partial` / `missing` /
`out-of-scope` with concrete evidence. See
[`capability-audit.md`](capability-audit.md). Headline: 130 shipped, 37 partial,
86 missing, 20 deliberately out of scope. The largest gaps are cost/EVM depth,
resource rate/curve modelling, configurable layouts and filters/grouping, and
production tracking.

## Verified sound

Kahn cycle detection and path recovery; FS/SS/FF/SF forward and backward formulas
including `+1` FS and negative lags; calendar working-day arithmetic and holidays;
duplicate activity/relationship detection at the engine level; SQL parameter binding
(no injection); `to_sql_id`/`from_sql_id` range checks; every migration arm 1→8
reaches `user_version = 8` with the same columns as `SCHEMA`; and all recently added
fields (`owner`, `promised`, `delay_reason`, `location`) appear in both the writer
and the reader.
