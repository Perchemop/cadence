# CADENCE-SETTINGS Feature Specification

## Overview

**Epic**: CADENCE-SETTINGS — 4-Level Settings Hierarchy with Sync
**User Value**: Reduce configuration friction for professional planners to < 30s project setup, < 10s view setup, < 5s tool density toggle
**Target Users**: Professional planners (primary), site supervisors, portfolio managers, power users
**Status**: Draft for review

---

## Functional Requirements (EARS Format)

### FR-01: Global Settings
- **FR-01.1**: The system shall provide a `GlobalSettings` structure containing theme preference, density level, keyboard shortcuts, assistant configuration, update channel, high contrast mode, and reduced motion preference.
- **FR-01.2**: The system shall persist global settings to a JSON file in the OS configuration directory (`$XDG_CONFIG_HOME/cadence/settings.json` on Linux, `~/Library/Application Support/Cadence/settings.json` on macOS, `%APPDATA%/Cadence/settings.json` on Windows).
- **FR-01.3**: The system shall load global settings at application startup and apply theme/density immediately before UI initialization.

### FR-02: Project Settings
- **FR-02.1**: The system shall provide a `ProjectSettings` structure containing currency, default calendar ID, column visibility defaults, cost rates, auto-capture baselines flag, EVM display preference, and calendars JSON.
- **FR-02.2**: The system shall persist project settings in the project SQLite database (`project_settings` table) with schema version tracking.
- **FR-02.3**: The system shall save project settings atomically with the project file in a single transaction.
- **FR-02.4**: The system shall migrate project settings schema additively (new columns only, `IF NOT EXISTS` tables).

### FR-03: View Settings
- **FR-03.1**: The system shall provide a `ViewSettings` structure containing per-view-mode settings for all 10 view modes (Table, Gantt, Resources, Baselines, Calendars, Lookahead, Diary, Production, Audit, Assistant).
- **FR-03.2**: The system shall provide `TableViewSettings` with sort key, sort direction, filter text, column visibility/widths, collapsed WBS nodes, and critical-only flag.
- **FR-03.4**: The system shall provide `GanttViewSettings` with pixels per day, split ratio, split visibility, critical path highlight, slack visibility, time scale, and today line visibility.
- **FR-03.5**: The system shall provide `ResourcesViewSettings` with histogram scale, grouping mode, and cost display mode.
- **FR-03.6**: The system shall provide specialized settings for Baselines, Calendars, Lookahead, Diary, Production, Audit, and Assistant views.
- **FR-03.7**: The system shall persist view settings optionally in SQLite `view_settings` table (opt-in via global setting), otherwise keep in-memory only.

### FR-04: Tool Settings
- **FR-04.1**: The system shall provide a `ToolSettings` structure containing inspector width, Gantt drag snap, histogram zoom, column resize state, and split pane ratio.
- **FR-04.2**: The system shall keep tool settings ephemeral (in-memory only, never persisted).
- **FR-04.3**: The system shall reset tool settings to defaults on view switch.

### FR-05: Settings Context & Fallback
- **FR-05.1**: The system shall provide a `SettingsContext` aggregating all four levels (Global, Project, View, Tool).
- **FR-05.2**: The system shall resolve effective values through unidirectional fallback: Tool → View → Project → Global → Built-in defaults.
- **FR-05.3**: The system shall provide accessor methods for all effective values: `effective_theme()`, `effective_density()`, `effective_pixels_per_day()`, `effective_histogram_scale()`, `effective_cost_display()`, `effective_split_ratio()`, `effective_inspector_width()`, `effective_split_pane_ratio()`, `effective_gantt_drag_snap()`.

### FR-06: Settings Synchronization
- **FR-06.1**: The system shall provide a `SettingsEventBus` with publish/subscribe pattern for change notification.
- **FR-06.2**: The system shall emit events for: GlobalChanged, ProjectChanged, ViewChanged, ToolChanged, DensityChanged, ThemeChanged.
- **FR-06.3**: The system shall propagate changes unidirectionally: Global → Project → View → Tool (no circular updates).
- **FR-06.4**: The system shall batch notifications per frame to prevent UI thrashing.

### FR-07: Schema Versioning & Migration
- **FR-07.1**: The system shall define independent schema versions per level (`GLOBAL_SCHEMA_VERSION`, `PROJECT_SCHEMA_VERSION`, `VIEW_SCHEMA_VERSION`).
- **FR-07.2**: The system shall provide migration functions for each level (additive only).
- **FR-07.4**: The system shall apply migrations at load time with automatic backup.

### FR-08: Commands & Undo/Redo
- **FR-08.1**: The system shall provide `SetGlobalSettings` command with undo support (stores previous value).
- **FR-08.2**: The system shall provide `SetViewSettings` command with undo support.
- **FR-08.3**: The system shall provide `SetToolSettings` command with undo support.
- **FR-08.4**: The system shall NOT create commands for tool settings (ephemeral).

### FR-09: Persistence
- **FR-09.1**: Global settings shall persist to JSON file in OS config directory.
- **FR-09.2**: Project settings shall persist in SQLite `project_settings` table (JSON blob for flexible fields).
- **FR-09.3**: View settings shall optionally persist in SQLite `view_settings` table (opt-in).
- **FR-09.4**: Tool settings shall never persist (ephemeral in-memory only).

### FR-10: API Surface for UI
- **FR-10.1**: The system shall embed `SettingsContext` in `CadenceApp` with accessor methods.
- **FR-10.2**: The system shall provide `SettingsSync` pub/sub for UI hot-reload.
- **FR-10.3**: View/Tool settings shall mutate directly in UI layer (no command).
- **FR-10.4**: Global/Project settings shall change via commands (for undo/redo).

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Project configuration time | < 30 seconds from creation to first schedule |
| NFR-02 | View configuration time | < 10 seconds per view |
| NFR-03 | Tool density toggle | < 5 seconds |
| NFR-04 | Settings load time | < 50ms at startup |
| NFR-05 | Settings change propagation | < 16ms (one frame) |
| NFR-06 | Schema migration time | < 100ms per level |
| NFR-07 | Memory overhead | < 1MB for settings context |
| NFR-08 | Atomic persistence | No partial writes on crash |
| NFR-09 | Backward compatibility | Load v0 settings, migrate to v10 |

---

## Acceptance Criteria (Given/When/Then)

### AC-01: Global Settings Persistence
**Given** a fresh Cadence installation
**When** the user sets theme to "Dark" and density to "Comfortable"
**Then** the settings persist to `$XDG_CONFIG_HOME/cadence/settings.json`
**And** restarting the app restores Dark theme and Comfortable density

### AC-02: Project Settings Persistence
**Given** a project with custom currency "EUR" and default calendar "6-day"
**When** the user saves the project
**Then** the project settings persist in the SQLite `project_settings` table
**And** loading the project restores EUR currency and 6-day calendar

### AC-03: View Settings Fallback
**Given** a project with no view settings saved
**When** the user opens the Gantt view
**Then** the view uses project defaults for pixels-per-day and split ratio
**And** changing pixels-per-day in the view does not affect other views

### AC-04: Settings Fallback Chain
**Given** global density = Compact, project density = unset, view density = Comfortable
**When** querying effective density for a view
**Then** the result is Comfortable (view overrides project overrides global)

### AC-05: Settings Change Propagation
**Given** two views open on the same project
**When** the user changes global density from Standard to Compact
**Then** both views update their effective density within 16ms
**And** no view is recreated

### AC-06: Settings Persistence Round-trip
**Given** a project with custom view settings (column widths, sort, filter)
**When** the user saves and reloads the project
**Then** all view settings are restored exactly

### AC-07: Undo/Redo Global Settings
**Given** the user changes theme from Light to Dark
**When** the user presses Ctrl+Z
**Then** the theme reverts to Light
**And** the change is reflected in the UI immediately

### AC-08: Settings Migration
**Given** a project file with schema version 9
**When** the user opens the project
**Then** the system migrates settings to schema v10 automatically
**And** no data is lost

---

## Error Handling

| Error Condition | Detection | Response |
|----------------|-----------|----------|
| Config file corrupted | JSON parse error on load | Log error, use built-in defaults, notify user |
| SQLite table missing | `sqlite3_prepare_v2` returns error | Auto-create table via `CREATE TABLE IF NOT EXISTS` |
| Schema version mismatch | Version < current | Run migration functions sequentially |
| JSON parse error in view settings | Serde deserialization error | Log warning, use defaults, continue |
| Config directory inaccessible | File I/O error | Fall back to in-memory defaults, show warning |

---

## Implementation TODO Checklist

### Phase 1: Core Data Structures (Week 1)
- [ ] Create `crates/core/src/settings.rs` with all data structures
- [ ] Implement `DensityLevel`, `ThemePreference`, `ShortcutConfig`
- [ ] Implement `GlobalSettings`, `ProjectSettings`, `ViewSettings`, `ToolSettings`
- [ ] Implement `SettingsContext` with fallback resolution
- [ ] Implement `SettingsEvent` / `SettingsEventBus`
- [ ] Add comprehensive unit tests (serde roundtrips, defaults, fallback chain)
- [ ] Add `pub mod settings;` to `crates/core/src/lib.rs`

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

### Phase 8: Polish & Accessibility (Week 6-7)
- [ ] Keyboard navigation for all settings dialogs
- [ ] AccessKit labels for all settings controls
- [ ] High contrast mode support
- [ ] Reduced motion support
- [ ] CI integration tests

---

## Validation

Run CI verification:
```bash
ci.sh run "CADENCE-SETTINGS Phase 1: Core data structures"
```

Expected: All tests pass, clippy clean, formatting clean.