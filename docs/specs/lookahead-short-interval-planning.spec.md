# Feature: Lookahead / Short-Interval Planning

## Overview
Cadence gains a **configurable rolling lookahead window** (1–6 weeks) for short-interval construction planning. Site superintendents and project controls engineers use it to run weekly work plans: assign activities to owners, track daily promises, record per-activity production, and detect location clashes — all while the lookahead **always live-derives from the master schedule** (no separate stored state). A lightweight weekly work package (activity list + owner per week) is the MVP; per-activity daily production reuses the existing `ProductionEntry` model; location clash detection flags same-location/overlapping-date conflicts within the window. The feature is desktop-only, single-project, cost-free, and read-only against the master schedule.

## Functional Requirements

### FR-LAP-001: Configurable Rolling Window
**While** a project is open, **when** the user opens the Lookahead view, **the system shall** display a rolling window of 1–6 weeks (configurable per project, default 2 weeks) starting from the current date or a user-set anchor date, showing all activities scheduled within that window.

### FR-LAP-002: Live Master Schedule Derivation
**Where** the Lookahead view is active, **the system shall** derive all lookahead entries **live from the current master schedule** — no separate stored lookahead state. Changes to activity dates, logic, or calendars in the master schedule **shall immediately reflect** in the lookahead window on next render or refresh.

### FR-LAP-003: Weekly Work Package (MVP)
**While** the Lookahead view is open, **when** the user expands a week within the window, **the system shall** display a work package containing the list of activities scheduled that week with their assigned **owner** (text field, free-form). The user **shall be able to** set or edit the owner per activity per week. No daily breakdown, resource loading, or cost data in MVP.

### FR-LAP-004: Owner / Promise Tracking
**Where** an activity appears in the lookahead window, **the system shall** display and allow editing of the activity's **owner** (free text) and **promised** flag (boolean). The `promised` field **shall** indicate the field team's commitment to complete the activity as scheduled. Changes **shall** write back to the master `Activity` record immediately.

### FR-LAP-005: Per-Activity Daily Production Recording
**Where** the user records progress in the Lookahead view, **the system shall** create or update a `ProductionEntry` for the given **activity + date** with a **percent complete** value (0–100 basis points). This **shall reuse** the existing `RecordProduction` / `UndoRecordProduction` command and `ProductionEntry` model — no new production model.

### FR-LAP-006: Location Clash Detection
**While** the lookahead window is displayed, **the system shall** evaluate all activities within the window and **shall flag** any pair that share the same **non-empty location** string and have **overlapping scheduled date ranges** (start–finish). Flagged clashes **shall** be displayed in the UI with activity names, location, and overlapping date range. No resource-type or severity analysis in MVP.

### FR-LAP-007: Snapshot Capture for Audit
**When** the user invokes **Capture Lookahead Snapshot**, **the system shall** create a read-only `LookaheadSnapshot` containing the current window's activity states (scheduled dates, owner, promised, percent complete) **for audit/drift comparison**. This reuses the existing `LookaheadSnapshot` model and `CaptureLookaheadSnapshot` command.

### FR-LAP-008: Diary Integration
**Where** the user creates a daily note from the Lookahead view, **the system shall** allow linking the note to the **lookahead window start date** (existing `DailyNote.lookahead_window_start` field) in addition to or instead of a specific activity.

### FR-LAP-009: Window Configuration Persistence
**When** the user changes the lookahead window length (1–6 weeks) or anchor date, **the system shall** persist the setting in the project's `ViewSettings` so it restores on next open.

### FR-LAP-010: Export Inclusion
**Where** the user exports the project to JSON, **the system shall** include the current lookahead window configuration and any captured snapshots in the export (reuses existing JSON export logic). MSPDI/MPX export **shall not** include lookahead-specific data (format limitation).

## Non-Functional Requirements

### Performance
- Lookahead window derivation **< 100 ms** for schedules up to 10,000 activities (headless test with generated data).
- Clash detection **< 50 ms** for windows up to 6 weeks / 500 activities.
- UI render **< 16 ms/frame** (60 fps) on reference hardware.

### Security
- All data remains **fully offline** — no network calls introduced.
- Owner/location strings are **display-only free text**; no authentication or personal identifiers linked.
- Export privacy: lookahead snapshots **shall include** owner/location strings in JSON export (same as master activities). No PII scrubbing mode in MVP.
- Crash reports **shall not** include file paths with usernames or raw activity data beyond what core already scrubs.

### Scalability
- Supports schedules up to 10,000 activities, 100 lookahead snapshots, 1,000 production entries per project.
- Window derivation uses the existing `lookahead()` function (O(n) scan) — no new indexes required.

### Accessibility (WCAG 2.2 AA)
- All lookahead controls reachable by keyboard (Tab order, Enter/Space activation).
- Clash flags **never conveyed by colour alone** — include text/icon label.
- Focus ring visible on all interactive elements.
- Target size ≥ 24 pt.

## Acceptance Criteria

### AC-LAP-001: Configurable Window
**Given** a project with activities spanning multiple weeks  
**When** the user sets the lookahead window to 4 weeks and opens the Lookahead view  
**Then** the view shows exactly those activities whose scheduled work falls within the next 4 weeks from the anchor date  
**And** the window length setting persists across app restarts

### AC-LAP-002: Live Derivation
**Given** the Lookahead view is open showing a 2-week window  
**When** the user changes an activity's scheduled start date in the Table view such that it moves into/out of the lookahead window  
**Then** the Lookahead view updates on next render to reflect the change without manual refresh  
**And** no separate "sync" or "refresh" action is required

### AC-LAP-003: Weekly Work Package Owner Assignment
**Given** the Lookahead view shows Week 1 with three activities  
**When** the user clicks the owner field for Activity A and enters "J. Smith"  
**Then** Activity A's `owner` field in the master schedule updates to "J. Smith"  
**And** the change is undoable via the standard command stack

### AC-LAP-004: Promise Flag
**Given** an activity in the lookahead window  
**When** the user toggles the `promised` checkbox  
**Then** the activity's `promised` field in the master schedule updates  
**And** the change is immediately visible in all views

### AC-LAP-005: Production Recording
**Given** the Lookahead view shows an activity with 0% production today  
**When** the user enters 50% in the production field for today  
**Then** a `ProductionEntry` for (activity, today, 50%) is created/updated  
**And** the existing `RecordProduction` command is used  
**And** the entry is visible in the Production view

### AC-LAP-006: Location Clash Detection
**Given** two activities in the lookahead window share location "Zone 3" and have overlapping scheduled dates  
**When** the Lookahead view renders  
**Then** both activities are marked with a clash indicator  
**And** selecting the indicator shows a dialog with both activity names, location "Zone 3", and the overlapping date range  
**And** no clash is reported for activities with different locations or non-overlapping dates

### AC-LAP-007: Snapshot Capture
**Given** the Lookahead view shows a configured window  
**When** the user clicks "Capture Snapshot" and enters a name  
**Then** a `LookaheadSnapshot` is created with all window activities' current state  
**And** the snapshot appears in the snapshots list and is included in JSON export

### AC-LAP-008: Diary Link to Window
**Given** the Lookahead view is open on a 2-week window starting 2026-01-12  
**When** the user creates a daily note from the Lookahead view  
**Then** the note's `lookahead_window_start` is set to 2026-01-12  
**And** the note appears in the Diary view filtered to that window

### AC-LAP-009: Window Config Persistence
**Given** the user sets window to 3 weeks, anchor date to next Monday  
**When** the user closes and reopens the project  
**Then** the Lookahead view opens with 3-week window starting from that Monday

### AC-LAP-010: Export Includes Snapshots
**Given** the project has two captured lookahead snapshots  
**When** the user exports to JSON  
**Then** the JSON includes both snapshots with all activity states  
**And** MSPDI export does not error and does not include lookahead data

## Error Handling

| Error Condition | Response |
|----------------|----------|
| User enters invalid window length (<1 or >6 weeks) | Clamp to 1–6; show inline validation message |
| Clash detection finds no location data on either activity | Skip pair silently (no false positive) |
| Production entry for (activity, date) already exists | Update existing entry (replace % complete) — existing behaviour |
| Snapshot name empty | Inline validation; disable capture until name entered |
| Master schedule has circular dependency | Existing cycle detection surfaces error; lookahead derivation aborts cleanly |

## Implementation TODO

### Backend (crates/core, crates/commands, crates/persistence)
- [ ] **Data model**: Add `window_weeks: u8` (1–6) and `anchor_date: Option<Date>` to `ViewSettings` or a new `LookaheadConfig` struct in `model.rs`
- [ ] **Core logic**: Extend `lookahead()` in `lookahead.rs` to accept configurable week count and anchor date
- [ ] **Clash detection**: Add `detect_location_clashes(window: &[Activity]) -> Vec<Clash>` in `lookahead.rs` or new `clash.rs`; `Clash` struct with `activity_a`, `activity_b`, `location`, `overlap_start`, `overlap_finish`
- [ ] **Command**: `UpdateLookaheadConfig` (window weeks, anchor date) in `commands/src/lib.rs`
- [ ] **Persistence**: Migrate `ViewSettings` schema (or new table) for lookahead config; bump schema to v11
- [ ] **Tests**: Unit tests for `lookahead()` with various window sizes; clash detection with overlapping/non-overlapping/same-different locations; config round-trip

### Frontend (crates/ui-egui)
- [ ] **Lookahead view**: Add week-grouped table (expandable per week), owner editable cell, promised checkbox, production % cell, clash indicator column
- [ ] **Window config UI**: Dropdown (1–6 weeks) + date picker for anchor in view header
- [ ] **Clash UI**: Inline indicator (icon + tooltip) + clash detail dialog
- [ ] **Owner/Promise cells**: Inline editing with immediate command dispatch
- [ ] **Production cell**: Inline % entry, dispatches `RecordProduction`
- [ ] **Snapshot button**: Dialog for name → `CaptureLookaheadSnapshot`
- [ ] **Diary integration**: "Add Note" button in Lookahead view pre-fills `lookahead_window_start`
- [ ] **Shortcuts**: Document `Ctrl+7` for Lookahead in `docs/qc/shortcuts.md`

### Testing
- [ ] **Headless workflow test**: Create project → add activities → open lookahead → set owner/promise → record production → detect clash → capture snapshot → export JSON → verify all data present
- [ ] **Accessibility test**: `egui_kittest` assertions for labelled owner/promise/production fields, clash indicator has accessible name, keyboard navigation through all interactive elements
- [ ] **Contrast test**: Clash indicator meets 3:1 on both themes
- [ ] **Property test**: Clash detection symmetry, no self-clash, no duplicate pairs

## Out of Scope (MVP)
- Resource leveling / optimization algorithms
- Cost loading / cash flow at lookahead level
- Multi-project lookahead across programmes
- Mobile / field-device app
- Automated clash resolution suggestions
- Daily breakdown within weekly work packages (beyond per-activity production %)
- Resource loading per week/day
- Promise history / change audit trail (beyond snapshot capture)
- Automated weekly window advancement (user controls anchor)

## Open Questions
- [ ] Should the anchor date default to "current week Monday" or "today"? (Current: today)
- [ ] Should clash detection consider only activities with non-empty location, or also flag missing location as a warning? (Current: skip silently)
- [ ] Is the 6-week max window sufficient, or do some projects need longer? (Current: 1–6 weeks)
- [ ] Should the lookahead view show critical-path highlighting? (Current: not in MVP)

---
*Spec generated via feature-forge workshop. Save location: `docs/specs/lookahead-short-interval-planning.spec.md`*