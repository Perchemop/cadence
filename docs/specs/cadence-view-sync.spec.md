# CADENCE-VIEW-SYNC Feature Specification

## Overview

**Epic**: CADENCE-VIEW-SYNC — Cross-View Settings Synchronization
**User Value**: Ensure consistent settings across all views with predictable conflict resolution — professional planners switch views frequently and expect their preferences to follow them
**Target Users**: Professional planners (primary), site supervisors, portfolio managers
**Status**: Draft for review

---

## Functional Requirements (EARS Format)

### FR-01: Settings Hierarchy
- **FR-01.1**: The system shall maintain a 4-level settings hierarchy: Global → Project → View → Tool.
- **FR-01.2**: Global settings (user-wide): theme, density, shortcuts, assistant config, update channel, high contrast, reduced motion. Persisted to JSON in OS config directory.
- **FR-01.3**: Project settings (per `.cadence` file): currency, default calendar, column visibility defaults, cost rates, auto-capture baselines, EVM display, calendars JSON. Persisted in SQLite `project_settings` table.
- **FR-01.4**: View settings (per view per project): sort, filter, zoom, column widths, split ratios, grouping. In-memory + optional SQLite `view_settings` table (opt-in).
- **FR-01.5**: Tool settings (ephemeral session): inspector width, Gantt drag snap, histogram zoom, column resize, split panes. In-memory only, never persisted.

### FR-02: Settings Resolution
- **FR-02.1**: The system shall resolve effective values through unidirectional fallback: Tool → View → Project → Global → Built-in defaults.
- **FR-01.2**: The system shall provide `SettingsContext::effective_*()` accessors for all UI-consumed values.
- **FR-01.3**: The system shall cache resolved values per view and invalidate on relevant setting changes.

### FR-02: Observer Pattern for Change Notification
- **FR-02.1**: The system shall implement a `SettingsStore` with `SubscriptionRegistry` mapping `SettingsKey` → `Weak<Observer>`.
- **FR-02.2**: The system shall provide `SettingsEvent` enum: GlobalUpdated, ProjectReloaded, ViewSwitched, ToolModified, DefaultsApplied.
- **FR-02.3**: The system shall use Crossbeam channels for thread-safe event dispatch.
- **FR-02.4**: The system shall batch notifications per frame to prevent UI thrashing.
- **FR-02.5**: The system shall debounce rapid setting changes (coalesce within frame).

### FR-03: Conflict Resolution
- **FR-03.1**: The system shall implement higher-level-wins priority: View > Project > Global for effective value resolution.
- **FR-03.2**: The system shall use last-write-wins for same-level concurrent edits.
- **FR-03.3**: The system shall treat tool-level transient state as always winning during active interaction.
- **FR-03.5**: The system shall log all conflicts with timestamps for debugging.

### FR-04: View-Specific Settings Schema
- **FR-04.1**: The system shall define typed settings structs for all 10 view modes:
  - Table: density, sort_key, sort_ascending, filter_text, column_widths, column_visibility, collapsed_wbs, critical_only
  - Gantt: pixels_per_day, split_ratio, show_split, critical_path_highlight, show_slack, time_scale, show_today_line
  - Resources: grouping, histogram_scale, cost_display
  - Baselines: show_variance, show_cost_variance, compare_baseline
  - Calendars: show_holidays, selected_calendar, editing_weekdays
  - Lookahead: window_start, weeks, promised_only
  - Diary: filter_activity, filter_window, sort_descending
  - Production: filter_activity, filter_date_range, show_variance
  - Audit: status_date, min_severity, group_by_category
  - Assistant: last_model, history, max_history
- **FR-04.2**: Each view settings struct shall implement `Default` with sensible defaults.
- **FR-04.3**: View settings shall be serializable via Serde for optional persistence.

### FR-05: Persistence Strategy
- **FR-05.1**: Global settings shall persist to JSON file in OS config directory (`~/.config/cadence/settings.json` / `%APPDATA%/Cadence/settings.json` / `~/Library/Application Support/Cadence/settings.json`).
- **FR-05.2**: Project settings shall persist in SQLite `project_settings` table (JSON blob for flexible fields).
- **FR-05.3**: View settings shall optionally persist in SQLite `view_settings` table with JSON columns per view mode (opt-in via GlobalSettings).
- **FR-05.4**: Tool settings shall never persist (ephemeral in-memory only).
- **FR-05.5**: Schema versioning per level with additive migration functions.

### FR-06: Undo/Redo Integration
- **FR-06.1**: The system shall provide `SetSetting` and `BatchSettings` commands in `crates/commands`.
- **FR-06.2**: Commands shall capture previous value for inverse operation.
- **FR-06.3**: Commands shall integrate with existing `CommandStack` (execute/undo/redo).
- **FR-06.3**: View/Tool settings changes shall NOT create commands (mutate directly, no undo).

### FR-07: Initialization & Hot-Reload
- **FR-07.1**: The system shall initialize in sequence: Load Global → Load Project → Construct Store → Create Views → Subscribe → Render.
- **FR-07.2**: Views shall be created lazily with effective settings resolved on first access.
- **FR-07.3**: Settings changes shall propagate without view recreation (hot-reload).
- **FR-07.4**: Pending changes shall be batched and flushed once per frame.
- **FR-07.5**: Observers shall compute minimal diff and only request repaint on actual changes.

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Settings resolution time | < 1ms per accessor |
| NFR-02 | Event propagation latency | < 1ms |
| NFR-03 | Batched notification flush | Once per frame (16ms) |
| NFR-03 | View creation latency | < 50ms |
| NFR-04 | Settings hot-reload latency | < 16ms |
| NFR-05 | Memory overhead | < 2MB for SettingsStore |
| NFR-06 | Schema migration time | < 100ms per level |

---

## Acceptance Criteria (Given/When/Then)

### AC-01: Settings Hierarchy Resolution
**Given** Global density = Standard, Project density = unset, View density = Compact
**When** querying effective density for the view
**Then** the result is Compact (View overrides Project overrides Global)

### AC-02: Settings Change Propagation
**Given** two views open on the same project
**When** the user changes global density from Standard to Compact
**Then** both views receive notification within 1ms
**And** both views update effective density within 16ms
**And** no view is recreated

### AC-03: Conflict Resolution
**Given** View density = Compact, Project density = Standard
**When** querying effective density
**Then** the result is Compact (View wins over Project)
**And** the conflict is logged with timestamp

### AC-04: View-Specific Settings Persistence
**Given** user configures Table view with custom column widths and sort
**When** the user saves and reloads the project (with opt-in enabled)
**Then** the Table view settings are restored exactly

### AC-05: Hot-Reload Without View Recreation
**Given** a view is open with custom settings
**When** the user changes a project-level setting that affects the view
**Then** the view updates in-place without recreation
**And** the view's scroll position and selection are preserved

### AC-06: Undo/Redo for Settings
**Given** the user changes project currency from USD to EUR
**When** the user presses Ctrl+Z
**Then** the currency reverts to USD
**And** all affected views update immediately

### AC-07: Tool Settings Ephemeral
**Given** the user resizes the inspector panel
**When** the user switches to a different view and back
**Then** the inspector width resets to default (tool settings never persist)

### AC-08: Event Bus Performance
**Given** 10 rapid setting changes in 100ms
**When** the event bus processes them
**Then** only one batched notification is delivered per frame
**And** no UI thrashing occurs

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Settings resolution time | < 1ms per accessor |
| NFR-02 | Event propagation latency | < 1ms |
| NFR-03 | Batched notification flush | Once per frame (16ms) |
| NFR-03 | View creation latency | < 50ms |
| NFR-04 | Settings hot-reload latency | < 16ms |
| NFR-05 | Memory overhead | < 2MB for SettingsStore |
| NFR-06 | Schema migration time | < 100ms per level |

---

## Acceptance Criteria (Given/When/Then)

### AC-01: Settings Hierarchy Resolution
**Given** Global density = Standard, Project density = unset, View density = Compact
**When** querying effective density for the view
**Then** the result is Compact (View overrides Project overrides Global)

### AC-02: Settings Change Propagation
**Given** two views open on the same project
**When** the user changes global density from Standard to Compact
**Then** both views receive notification within 1ms
**And** both views update effective density within 16ms
**And** no view is recreated

### AC-03: Conflict Resolution
**Given** View density = Compact, Project density = Standard
**When** querying effective density
**Then** the result is Compact (View wins over Project)
**And** the conflict is logged with timestamp

### AC-04: View-Specific Settings Persistence
**Given** user configures Table view with custom column widths and sort
**When** the user saves and reloads the project (with opt-in enabled)
**Then** the Table view settings are restored exactly

### AC-05: Hot-Reload Without View Recreation
**Given** a view is open with custom settings
**When** the user changes a project-level setting that affects the view
**Then** the view updates in-place without recreation
**And** the view's scroll position and selection are preserved

### AC-06: Undo/Redo for Settings
**Given** the user changes project currency from USD to EUR
**When** the user presses Ctrl+Z
**Then** the currency reverts to USD
**And** all affected views update immediately

### AC-07: Tool Settings Ephemeral
**Given** the user resizes the inspector panel
**When** the user switches to a different view and back
**Then** the inspector width resets to default (tool settings never persist)

### AC-08: Event Bus Performance
**Given** 10 rapid setting changes in 100ms
**When** the event bus processes them
**Then** only one batched notification is delivered per frame
**And** no UI thrashing occurs

---

## Error Handling

| Error Condition | Detection | Response |
|----------------|-----------|----------|
| Settings file corrupted | JSON parse error | Log error, use built-in defaults, notify user |
| SQLite table missing | `sqlite3_prepare_v2` error | Auto-create table via `CREATE TABLE IF NOT EXISTS` |
| Schema version mismatch | Version < current | Run migration functions sequentially |
| Circular update detected | Event loop > 1 iteration | Log warning, break cycle, use higher-level value |
| Observer panic | Observer callback panics | Catch panic, log error, continue other observers |

---

## Implementation TODO Checklist

### Phase 1: Core Data Structures (Week 1)
- [ ] Create `crates/core/src/settings.rs` with all data structures
- [ ] Implement `DensityLevel`, `ThemePreference`, `ShortcutConfig`
- [ ] Implement `GlobalSettings`, `ProjectSettings`, `ViewSettings`, `ToolSettings`
- [ ] Implement `SettingsContext` with fallback resolution
- [ ] Implement `SettingsEvent` / `SettingsEventBus`
- [ ] Add comprehensive unit tests (serde roundtrips, defaults, fallback chain)

### Phase 2: Global Persistence (Week 1-2)
- [ ] Implement config directory detection per OS
- [ ] Implement JSON load/save for `GlobalSettings`
- [ ] Add schema version migration for global settings
- [ ] Integrate with `CadenceApp::new()`

### Phase 3: Project Settings in SQLite (Week 2)
- [ ] Add `project_settings` table to SQL schema (v11)
- [ ] Implement `ensure_project_settings_table()`
- [ ] Integrate with `save_project()` / `load_project()`
- [ ] Add migration from v10 to v11

### Phase 4: View & Tool Settings (Week 3-4)
- [ ] Replace scattered `CadenceApp` fields with `ViewSettings` / `ToolSettings`
- [ ] Implement `SettingsContext::effective_*()` accessors
- [ ] Add optional `view_settings` SQLite table
- [ ] Update UI to use `app.view_settings()` / `app.tool_settings()`

### Phase 5: Settings UI (Week 4-5)
- [ ] Global settings dialog (theme, density, shortcuts, assistant)
- [ ] Per-view settings panels (columns, filters, zoom)
- [ ] Tool settings inline (inspector width, snap, zoom)
- [ ] Keyboard shortcuts for density cycling (`Ctrl+Shift+D`)

### Phase 6: Change Notification (Week 5)
- [ ] Implement `SettingsEventBus` with `SettingsEvent` types
- [ ] Wire `SettingsSync` into `CadenceApp` update loop
- [ ] Verify no circular updates possible
- [ ] Add batched notification flushing

### Phase 7: Migration & Testing (Week 6)
- [ ] Comprehensive schema migration tests
- [ ] Settings round-trip tests (save → load → verify)
- [ ] Fallback chain unit tests
- [ ] Event bus subscription/notification tests

### Phase 7: Polish & Accessibility (Week 6-7)
- [ ] Keyboard navigation for all settings dialogs
- [ ] AccessKit labels for all settings controls
- [ ] High contrast mode support
- [ ] Reduced motion support
- [ ] CI integration tests

---

## Validation

Run CI verification:
```bash
ci.sh run "CADENCE-VIEW-SYNC Phase 1-7"
```

Expected: All tests pass, clippy clean, formatting clean.