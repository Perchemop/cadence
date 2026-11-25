# CADENCE-VIEW-SYNC Specification

This document defines the architecture for cross-view settings synchronization in Cadence, addressing the CADENCE-106 epic. It ensures that settings changes propagate correctly across the four-level hierarchy (Global → Project → View → Tool) without circular updates, while preserving the single-source-of-truth principle.

## 1. Overview

The settings hierarchy defines four nested levels of configuration, each with its own scope, persistence strategy, and lifecycle. Settings flow downward (global defaults → project overrides → view customizations → tool preferences), and changes at any level propagate upward through a sync mechanism.

The four levels are:

| Level | Scope | Primary Store | Persistence | Typical Granularity |
|-------|-------|---------------|-------------|---------------------|
| **Global** | App-wide preferences, independent of any project | Global config file (`$XDG_CONFIG_HOME/cadence/settings.json`) | File-based, app-startup load | Theme, update checks, global shortcuts, default density |
| **Project** | Per-project configuration, stored with the project file | SQLite `user_version` pragma + project metadata | SQLite, transactional with project save | Workweek calendars, resource defaults, baseline preferences |
| **View** | Per-window / per-view configuration | JSON sidecar or embedded in app state | JSON file or app-state, reload on view switch | Active view mode, visible columns, filter state, window position |
| **Tool** | Per-toolbar / per-tool configuration | JSON per-tool or embedded in view state | JSON, scoped to active tool | Density per view, toolbar customization, inspector defaults |

## 2. Data Structures

### 2.1 Global Settings (`GlobalSettings`)
- **theme**: `ThemePreference` (`System`, `Light`, `Dark`)
- **update_checks_enabled**: `bool`
- **default_density**: `DensityLevel` (`Compact`, `Standard`, `Comfortable`)
- **shortcuts**: `ShortcutConfig` (key → action bindings)
- **last_update_check**: `Option<Instant>` (transient, not persisted)

### 2.2 Project Settings (`ProjectSettings`)
- **workweek**: `[bool; 7]` (Monday=bit0..Sunday=bit6)
- **calendars**: `BTreeMap<String, CalendarProjectSettings>`
- **resource_default_capacity**: `u32`
- **resource_default_cost_per_unit**: `i64`
- **auto_capture_baselines**: `bool`
- **evm_display**: `EvmDisplayPreference`

### 2.3 View Settings (`ViewSettings`)
- **view_mode**: `ViewMode` (`Table`, `Gantt`, `Resources`, etc.)
- **visible_columns**: `BTreeSet<SortKey>` (columns visible in activity table)
- **table_filter_buffer**: `String` (filter text for activity table)
- **table_critical_only**: `bool`
- **show_split_gantt**: `bool`
- **window_rect**: `Option<egui::Rect>`
- **sort_key**: `SortKey`
- **sort_ascending**: `bool`

### 2.4 Tool Settings (`ToolSettings`)
- **density_override**: `Option<DensityLevel>`
- **toolbar_visibility**: `BTreeMap<ViewMode, BTreeSet<usize>>`
- **inspector_expanded**: `BTreeMap<ActivityField, bool>`
- **assignment_default_units**: `u32`
- **last_focused_activity**: `Option<String>`

## 3. Sync Mechanism

A pub/sub mechanism (`SettingsSync`) ensures changes propagate correctly:

- **Events**:
  - `GlobalUpdated`: Global settings changed → cascades downward.
  - `ProjectReloaded`: Project file loaded/saved → cascades downward.
  - `ViewSwitched`: View mode changed → tool settings adjust.
  - `ToolModified`: Tool preferences changed → view re-renders.
  - `DefaultsApplied`: Initial startup → fill in missing values.

- **Payload**: JSON-serialized `SettingsEvent` payload (e.g., new `GlobalSettings` value).

- **Observer Pattern**:
  - `SettingsSync.observers`: `HashMap<SettingsEvent, Vec<Box<dyn Fn(String)>>>`.
  - Handlers receive the new settings value as a JSON string and deserialize as needed.

## 4. Conflict Resolution Strategy

- **Single Source of Truth**: Each setting value is resolved from the highest applicable level in the hierarchy (Tool → View → Project → Global).
- **LWW (Last Write Wins)**: Within a level, the most recent change wins. However, this is overridden by the hierarchy.
- **Explicit Reset**: To clear a level and fall back to inheritance, use the `reset_to_inherit` operation (not part of v1 but planned for v2).
- **No Prompts**: In v1, conflicts are resolved automatically; no user prompts are required.
- **Immutable Snapshots**: Settings are treated as immutable snapshots; changes are applied as new values rather than modifying existing state.

## 5. View Settings Schema

The `ViewSettings` struct defines the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `view_mode` | `ViewMode` | Current view mode (Table, Gantt, Resources, etc.) |
| `visible_columns` | `BTreeSet<SortKey>` | Columns visible in the activity table, ordered by user preference |
| `table_filter_buffer` | `String` | Text filter for activity table search |
| `table_critical_only` | `bool` | Whether critical-only filter is active |
| `show_split_gantt` | `bool` | Whether split-gantt auxiliary view is visible |
| `window_rect` | `Option<egui::Rect>` | Last settled window position/size |
| `sort_key` | `SortKey` | Current sort key for table (e.g., Name, Duration) |
| `sort_ascending` | `bool` | Sort direction |

## 5. Tool Settings Schema

The `ToolSettings` struct defines:

| Field | Type | Description |
|-------|------|-------------|
| `density_override` | `Option<DensityLevel>` | Per-view density override; `None` means use global/default |
| `toolbar_visibility` | `BTreeMap<ViewMode, BTreeSet<usize>>` | Which buttons are shown/hidden per view mode |
| `inspector_expanded` | `BTreeMap<ActivityField, bool>` | Inspector field expansion state |
| `assignment_default_units` | `u32` | Default assignment units (basis points) for new resource assignments |
| **last_focused_activity** | `Option<String>` | Last focused activity ID across view switches |

## 5. Initialization Sequence

1. **App Startup**:
   - Load `GlobalSettings` from config file (or use defaults).
   - Load `ProjectSettings` from SQLite (or use defaults if no project file).
   - Initialize `CadenceApp` with default values for `view_settings` and `tool_settings`.

2. **Project Load**:
   - Load project file via `load_project()`.
   - Load `ProjectSettings` from SQLite (or use defaults).
   - Apply `ProjectSettings` to `CadenceApp` (e.g., set `global_settings`, `resource_default_capacity`, etc.).
   - Notify `SettingsSync` of `ProjectReloaded` event.

3. **View Switch**:
   - User switches view (e.g., Table → Gantt).
   - `ViewSettings` are updated (e.g., `visible_columns`, `sort_key`).
   - `SettingsSync.notify(ViewSwitched)` is called.
   - `ToolSettings` may be adjusted (e.g., `density_override`) based on new view.

4. **Tool Preference Change**:
   - User changes density or toolbar visibility.
   - `SettingsSync.notify(ToolModified)` is called.
   - View UI updates accordingly.

5. **Global Settings Change**:
   - User changes theme or update_checks_enabled.
   - `SettingsSync.notify(GlobalUpdated)` is called.
   - All levels receive the update; UI re-renders with new theme/color scheme.

## 5. Command Integration

Settings changes are implemented as commands:

- `SetGlobalSettings`: Takes `GlobalSettings`, stores previous value, updates `project.global_settings`, and pushes `SetGlobalSettingsUndo` to `CommandStack`.
- `SetViewSettings`: Takes `ViewSettings`, stores previous value, updates `project.view_settings`, pushes `SetViewSettingsUndo`.
- `SetToolSettings`: Takes `ToolSettings`, stores previous value, updates `project.tool_settings`, pushes `SetToolSettingsUndo`.

All commands are reversible via `CommandStack.undo()` and `CommandStack.redo()`, ensuring that settings changes are undoable and redoable, and are auditable via the command history.

## 5. Persistence and Migration

- **Global Settings**: Persist to JSON file; schema version field `version` in `GlobalSettings` (added in v1).
- **Project Settings**: Stored in `project_settings` table (v10+). Migration path:
  - v0 → v1: add `project_settings` table.
  - v1 → v2: add `workweek` column (already present), add `resource_default_capacity`, etc.
  - v9 → v10: add `evm_display`, `auto_capture_baselines`, etc.
- **View Settings**: JSON sidecar file (e.g., `.view-settings.json`) or embedded in `CadenceApp` state; versioned via `ViewSettings` struct.
- **Tool Settings**: JSON per-tool or embedded; versioned similarly.

## 5. Implementation Phases

| Phase | Goals | Deliverables |
|-------|-------|--------------|
| **Phase 1: Foundations** | Define setting types, defaults, and `SettingsSync` pub/sub. | `settings.rs` with all structs, `SettingsSync` implementation, unit tests for fallback resolution. |
| **Phase 2: Global Persistence** | Implement file-based persistence for `GlobalSettings`. | `save_global_settings()`, `load_global_settings()`, integration with `CadenceApp`. |
| **Phase 3: Project Settings Integration** | Extend persistence to include `ProjectSettings` in project file. | Extend `save_project()`, `load_project()`, add `ProjectSettings` migration logic. |
| **Phase 4: View & Tool Settings** | Implement view and tool settings UI and persistence. | `ViewSettings` UI controls, `ToolSettings` UI controls, persistence for sidecar files. |
| **Phase 5: Polish and Sync** | Ensure all levels propagate changes correctly, add tests, update docs. | End-to-end tests, updated `PROGRESS.md`, updated `CHANGELOG`. |

## 5. Files Changed

- `crates/core/src/settings.rs` — new setting types and `SettingsSync` implementation.
- `crates/ui-egui/src/lib.rs` — update `CadenceApp` to hold `view_settings` and `tool_settings` fields; update UI to use new API.
- `crates/commands/src/lib.rs` — add `SetGlobalSettings`, `SetViewSettings`, `SetToolSettings` commands implementing `Command`.
- `crates/persistence/src/lib.rs` — extend `save_project()` and `load_project()` to handle settings persistence.

## 5. Public API

- `CadenceApp::global_settings() -> &GlobalSettings`
- `CadenceApp::set_global_settings(settings: GlobalSettings)`
- `CadenceApp::view_settings() -> &ViewSettings`
- `CadenceApp::set_view_settings(settings: ViewSettings)`
- `CadenceApp::tool_settings() -> &ToolSettings`
- `CadenceApp::set_tool_settings(settings: ToolSettings)`

## 5. Tests

- Unit tests for `SettingsSync` event propagation.
- Integration tests for settings round-trip (global → project → view → tool).
- Regression tests for v1 → v2 → v3 schema migrations.

## 5. Final Notes

- The specification ensures that settings are **immutable** at the point of emission (a new value is set, not the old one mutated).
- The **single-source-of-truth** principle is enforced by the hierarchy and the `SettingsSync` pub/sub mechanism.
- The **4-level hierarchy** is strictly enforced; no level may access settings from a lower level directly.
- The **command-based** approach ensures that all settings changes are undoable and redoable, and are auditable via the command history.
- The **4-level hierarchy** is strictly enforced; no level may access settings from a lower level directly.
- The **command-based** approach ensures that all settings changes are undoable and redoable, and are auditable via the command history.

This specification provides a complete, implementable blueprint for CADENCE-VIEW-SYNC that aligns with the guide's quality gates, clean-room rules, and CI requirements.