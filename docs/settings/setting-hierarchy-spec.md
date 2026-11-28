# CADENCE-SETTINGS: 4-Level Hierarchy Specification

**Status**: Design draft — awaiting implementation phase  
**Related epics**: CADENCE-SETTINGS, CADENCE-TOOLBAR, CADENCE-VIEW-SYNC  
**Reference**: Discovery doc Hypothesis H2, Decision framework "Settings hierarchy depth", Activity 5 Cross-Feature Integration Mapping

## 1. Overview

The settings hierarchy defines four nested levels of configuration, each with its own scope, persistence strategy, and lifecycle. Settings flow downward (global defaults → project overrides → view customizations → tool preferences), and changes at any level propagate upward through a sync mechanism.

The four levels are:

| Level | Scope | Primary Store | Persistence | Typical Granularity |
|-------|-------|--------------|-------------|---------------------|
| **Global** | App-wide preferences, independent of any project | Global config file (`$XDG_CONFIG_HOME/cadence/settings.json`) | File-based, app-startup load | Theme, update checks, global shortcuts, default density |
| **Project** | Per-project configuration, stored with the project file | SQLite `user_version` pragma + project metadata | SQLite, transactional with project save | Workweek calendars, resource defaults, baseline preferences |
| **View** | Per-window / per-view configuration | JSON sidecar or embedded in app state | JSON file or app-state, reload on view switch | Active view mode, visible columns, filter state, window position |
| **Tool** | Per-toolbar / per-tool configuration | JSON per-tool or embedded in view state | JSON, scoped to active tool | Density per view, toolbar customization, inspector defaults |

## 2. Data Structures

### 2.1 Global Settings (`cadence_core::settings::GlobalSettings`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// UI theme (light/dark system follow)
    pub theme: ThemePreference,
    /// Whether update checking is enabled
    pub update_checks_enabled: bool,
    /// Default activity density: compact / standard / comfortable
    pub default_density: DensityLevel,
    /// Global keyboard shortcut configuration
    pub shortcuts: ShortcutConfig,
    /// Last checked for updates timestamp
    #[serde(skip)]
    pub last_update_check: Option<std::time::Instant>,
}
```

**`ThemePreference`**: `System`, `Light`, `Dark`  
**`DensityLevel`**: `Compact`, `Standard`, `Comfortable`  
**`ShortcutConfig`**: Named collection of key→action bindings

### 2.2 Project Settings (already in SQLite, typed wrapper)

The project file already stores workweek, calendars, resources, etc. We add a typed wrapper:

```rust
pub struct ProjectSettings {
    /// Workweek pattern (Monday=bit0..Sunday=bit6)
    pub workweek: [bool; 7],
    /// Named calendars keyed by CalendarId
    pub calendars: BTreeMap<CalendarId, NamedCalendarSettings>,
    /// Resource default capacity (units per working day)
    pub resource_default_capacity: u32,
    /// Resource default cost per unit (minor currency units / working day)
    pub resource_default_cost_per_unit: i64,
    /// Baseline auto-capture preference
    pub auto_capture_baselines: bool,
    /// Earned-value metrics display preference
    pub evm_display: EvmDisplayPreference,
}
```

### 2.3 View Settings (`cadence_core::settings::ViewSettings`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ViewSettings {
    /// Currently active view mode
    pub view_mode: ViewMode,
    /// Which columns are visible in table view
    pub visible_columns: BTreeSet<SortKey>,
    /// Filter text for activity table
    pub table_filter_buffer: String,
    /// Whether critical-only filter is active
    pub table_critical_only: bool,
    /// Window position and size (last settled size)
    pub window_rect: Option<egui::Rect>,
    /// Split Gantt visibility
    pub show_split_gantt: bool,
    /// Sort key and direction
    pub sort_key: SortKey,
    pub sort_ascending: bool,
}
```

### 2.4 Tool Settings (`cadence_core::settings::ToolSettings`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ToolSettings {
    /// Per-view density override (None = use global default)
    pub density_override: Option<DensityLevel>,
    /// Toolbar customization: which buttons are shown/hidden
    pub toolbar_visibility: BTreeMap<ViewMode, BTreeSet<usize>>,
    /// Inspector default expanded state per activity field
    pub inspector_expanded: BTreeMap<ActivityField, bool>,
    /// Resource assignment default units (basis points, 10_000 = 1 unit)
    pub assignment_default_units: u32,
    /// Last focused activity across view switches
    pub last_focused_activity: Option<ActivityId>,
}
```

### 2.5 Shared Enum Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Table,
    Gantt,
    Resources,
    Baselines,
    Calendars,
    Audit,
    Lookahead,
    Diary,
    Production,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityField {
    Name,
    Duration,
    Constraint,
    Start,
    Finish,
    Float,
    Critical,
    Owner,
    Location,
    Promised,
    DelayReason,
    Budget,
    ActualCost,
    ActualStart,
    ActualFinish,
    ResourceAssignments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DensityLevel {
    Compact,
    Standard,
    Comfortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutKey {
    pub modifier: egui::Modifiers,
    pub key: egui::Key,
}
```

## 3. Persistence Strategy

Each level has a distinct persistence strategy, enabling independent evolution and platform compatibility.

### 3.1 Global (File-Based)

- **Location**: `$XDG_CONFIG_HOME/cadence/settings.json` (or `%APPDATA%\cadence\settings.json` on Windows)
- **Format**: JSON with serde
- **Load**: Once at app startup; cached in `CadenceApp` global settings field
- **Save**: On `set_*` changes, debounced (250ms) or on app exit
- **Migration**: Schema version field `version`; backward-compatible field additions only
- **Concurrency**: Read-only during app run; write on exit or explicit "Save Settings"

### 3.2 Project (SQLite - already established)

- **Schema**: Already versioned via `PRAGMA user_version` (currently v10)
- **Extension**: Add `settings` table or extend existing metadata
- **Transaction**: Atomic with project save; failed save never corrupts project
- **Migration**: Existing v0→v10 migration path continues; new settings columns added with `ALTER TABLE`
- **Concurrency**: SQLite WAL mode; read during project load, write on project save

### 3.3 View (JSON sidecar or embedded)

- **Location**: Per-project `.view-settings.json` alongside the `.cadence` project file, OR embedded in the project SQLite file as a metadata blob
- **Format**: JSON, scoped to (project_id, view_mode)
- **Load**: On view switch; persisted across app restarts
- **Save**: On view configuration changes (column visibility, filter, density)
- **Migration**: New fields added with version pragma; old fields defaulted

### 3.4 Tool (JSON in view state)

- **Location**: Embedded in `ViewSettings` or separate `.tool-settings.json`
- **Format**: JSON, scoped to (project_id, view_mode, tool_id)
- **Load**: On tool activation (toolbar customization, density)
- **Save**: On tool preference changes
- **Migration**: Same pattern as view settings

## 4. Change Notification / Sync Mechanism

A pub/sub sync mechanism ensures changes at one level propagate appropriately to dependent levels and the UI.

### 4.1 Architecture

```
Global Settings  ──► Project Settings  ──► View Settings  ──► Tool Settings
     │                  │                  │                  │
     ▼                  ▼                  ▼                  ▼
 Defaults        Project overrides   View customizations  Tool preferences
```

### 4.2 Sync Events

| Event | Source | Target | Description |
|-------|--------|--------|-------------|
| `GlobalUpdated` | Global settings changed | Project → Reset to defaults where no project override exists | e.g., theme change affects palette |
| `ProjectReloaded` | Project file loaded/saved | View → Re-apply view settings with new project context | e.g., new calendars available |
| `ViewSwitched` | View mode change | Tool → Adjust tool settings for new view | e.g., density override per view |
| `ToolModified` | Tool settings changed | View → Re-render with updated tool preferences | e.g., toolbar visibility change |
| `DefaultsApplied` | Initial app startup | All levels → Fill in missing values from higher level | e.g., view density from global default |

### 4.3 Implementation: `SettingsSync` struct

```rust
pub struct SettingsSync {
    /// Observers registered for each event type
    observers: HashMap<SettingsEvent, Vec<Box<dyn Fn(SettingsEvent)>>>,
}

impl SettingsSync {
    pub fn subscribe(&mut self, event: SettingsEvent, handler: Box<dyn Fn(SettingsEvent)>) {
        self.observers.entry(event).or_default().push(handler);
    }
    
    pub fn notify(&self, event: SettingsEvent) {
        if let Some(handlers) = self.observers.get(&event) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}
```

### 4.4 Flow Examples

**Theme change (Global → UI)**:
1. User toggles dark/light in app menu
2. `GlobalSettings.theme` updated
3. `SettingsSync.notify(GlobalUpdated)`
4. UI reads `global_theme` from `DesignTokens::for_dark(dark)`
5. All widgets repaint with new visuals

**Column visibility change (View → UI)**:
1. User clicks "Columns" menu → toggles "Duration"
2. `ViewSettings.visible_columns` updated
3. `SettingsSync.notify(ViewModified)`
4. Table view re-renders with updated column set
5. Persist to view sidecar file

**Density override (Tool → View)**:
1. User selects "Comfortable" density for Gantt view
2. `ToolSettings.density_override` set to `Comfortable`
3. `SettingsSync.notify(ToolModified)`
4. Gantt view applies `DensityLevel::Comfortable` scaling
5. Saved to tool settings for future Gantt sessions

## 5. Schema Versioning and Migration

Each level carries a schema version; migrations are additive (new fields only, never breaking existing ones).

### 5.1 Global Schema

```toml
# In settings.json
{
  "version": 3,
  "theme": "system",
  "update_checks_enabled": true,
  "default_density": "standard",
  "shortcuts": { ... }
}
```

- **v1**: Initial release — theme, update_checks_enabled, default_density
- **v2**: Add shortcuts configuration
- **v3**: Add per-view density overrides (backward-compatible; v2 reads as `Standard`)

Migration is a simple read: if `version` missing, assume v1 defaults; apply upgrades sequentially.

### 5.2 Project Schema (extension of existing)

The existing `PRAGMA user_version` (v10) governs the project-data schema. We add settings migration within the same transaction:

- **Within save_project()**: After `ensure_schema()`, run `migrate_project_settings(&connection)`
- **Migrations**: `ALTER TABLE` additions for new settings fields; data preserved where possible
- **Backward compatibility**: Old project files (v0-v9) migrate to v10+ with new settings fields defaulted

### 5.3 View Schema

```json
{
  "version": 2,
  "visible_columns": ["Name", "Duration", "Start", "Finish"],
  "table_filter": "",
  "sort_key": "Name",
  "sort_ascending": true
}
```

- **v1**: Initial — visible_columns, table_filter, sort_key+direction
- **v2**: Add `show_split_gantt`, `window_rect`

### 5.4 Tool Schema

```json
{
  "version": 1,
  "density_override": "standard",
  "toolbar_visibility": { ... },
  "inspector_expanded": { ... },
  "assignment_default_units": 10000
}
```

- **v1**: Initial release with all fields

### 5.5 Migration Enforcement

- `cargo deny` checks that no serialized field removal occurs without a migration path
- Every new field has a `#[serde(default)]` so old serializations default gracefully
- Migration scripts in `crates/persistence/migrations/` (one per version bump)

## 6. Default Values and Fallback Chain

Each level provides defaults; the fallback chain resolves a setting by checking levels from top to bottom.

### 6.1 Fallback Resolution Order

```
1. Tool settings (most specific)
2. View settings
3. Project settings
4. Global settings (most general)
5. Hard-coded code defaults (last resort)
```

### 6.2 Accessor Example

```rust
impl GlobalSettings {
    pub fn theme(&self) -> ThemePreference {
        self.theme
    }
    
    pub fn effective_theme(&self, project: &ProjectSettings, view: &ViewSettings, tool: &ToolSettings) -> ThemePreference {
        // Check tool first, then view, then project, then global, then default
        if let Some(overridden) = tool.theme_override {
            return overridden;
        }
        // ... cascading checks
        self.theme  // fallback
    }
}
```

### 6.3 Defaults Table

| Setting | Global Default | Project Default | View Default | Tool Default | Code Default |
|---|---|---|---|---|---|
| `theme` | `System` | inherit | inherit | none | `System` |
| `default_density` | `Standard` | inherit | inherit | `Standard` | `Standard` |
| `visible_columns` | all columns | inherit | `Name, Duration, Start, Finish` | multiselect | all columns |
| `toolbar_visibility` | all buttons | inherit | inherit | per-view selection | all buttons |
| `resource_default_capacity` | 1 | 1 | inherit | inherit | 1 |

"inherit" means the value flows down from the level above; if the level above doesn't specify, the next level down is checked.

## 7. API Surface for UI Consumption

All settings are exposed through a clean, egui-integrated API.

### 7.1 Global Settings API

```rust
pub struct GlobalSettingsApi {
    /// Read the current global settings (cached from file)
    pub fn get(&self) -> GlobalSettings;
    /// Write global settings (persists to file)
    pub fn set(&mut self, new: GlobalSettings) -> Result<(), SettingsError>;
    /// Observe changes (register a callback)
    pub fn observe(&mut self, handler: Box<dyn Fn(&GlobalSettings)>) -> Subscription;
}

pub enum SettingsError {
    FileAccess(String),
    Serde(String),
    Validation(String),
}
```

### 7.2 Project Settings API (within persistence)

```rust
impl Project {
    /// Load project with full settings integration
    pub fn load_with_settings(path: &Path) -> Result<Self, PersistenceError>;
    /// Save project and persist settings
    pub fn save_with_settings(&self, path: &Path) -> Result<(), PersistenceError>;
    /// Get project-level settings
    pub fn settings(&self) -> &ProjectSettings;
    /// Mutate project settings
    pub fn mutate_settings(&mut self, f: impl FnOnce(&mut ProjectSettings));
}
```

### 7.3 View Settings API (within UI crate)

```rust
impl CadenceApp {
    /// Get current view settings
    pub fn view_settings(&self) -> &ViewSettings;
    /// Update view settings and sync
    pub fn set_view_settings(&mut self, new: ViewSettings);
    /// Get tool settings for the active view
    pub fn tool_settings(&self) -> &ToolSettings;
    /// Apply a command that may change settings
    pub fn apply_setting_command(&mut self, cmd: Box<dyn Command>) -> Result<(), String>;
}
```

### 7.4 UI Integration Patterns

**Reading a setting in egui:**

```rust
// Somewhere in the UI build function
ui.horizontal(|ui| {
    egui::checkbox(
        ui,
        "Dark mode",
        &mut app.view_settings().theme  // or app.global_settings().theme
    );
});
```

**Applying a setting change through commands:**

```rust
// User changes a setting in the inspector
if self.apply_setting_command(Box::new(SetViewSettings {
    visible_columns: new_set,
})) {
    self.set_status("View settings saved");
}
```

**Sync across levels on project load:**

```rust
// In CadenceApp::load_project()
self.project_settings = project.settings().clone();
self.view_settings = ViewSettings::default(); // resets to defaults
self.tool_settings = ToolSettings::default();
SettingsSync.notify(ViewSwitched); // re-apply with new project context
```

## 8. Implementation Phases

### Phase 1: Foundations (1 sprint)
- [ ] Define data structures in `crates/core/src/settings.rs`
- [ ] Add `GlobalSettings` with serde + default values
- [ ] Define `ViewSettings`, `ToolSettings`, `ProjectSettings` types
- [ ] Add schema version field to `GlobalSettings`
- [ ] Create `SettingsSync` pub/sub mechanism
- [ ] Write unit tests for fallback resolution

### Phase 2: Global Persistence (1 sprint)
- [ ] Create config directory discovery (`~/.config/cadence/`)
- [] Implement `save_global_settings()` and `load_global_settings()`
- [] Integrate with `CadenceApp` construction
- [ ] Add `set_update_checks_enabled()` → persists global setting
- [ ] CI: verify read/write on Linux/macOS/Windows

### Phase 3: Project Settings Integration (2 sprints)
- [ ] Extend `save_project()` / `load_project()` to persist settings
- [ ] Add `ProjectSettings` wrapper with fallbacks
- [ ] Migrate existing project files through v10+ with new settings
- [ ] View settings re-apply on project load
- [ ] CI: round-trip test: load project → modify settings → save → reload

### Phase 4: View + Tool Settings (2 sprints)
- [ ] Implement `ViewSettings` with column visibility, filter, sort
- [ ] Add UI: "Columns..." menu, density selector
- [ ] Implement `ToolSettings` with density overrides, toolbar visibility
- [ ] Add per-view density scaling in Gantt/table rendering
- [ ] Sync mechanism: ViewSwitched, ToolModified events

### Phase 5: Polish and Sync (1 sprint)
- [ ] Default value cascade testing across all levels
- [ ] Edge cases: corrupt settings file, missing config dir
- [ ] Accessibility: keyboard navigation of settings UI
- [ ] Performance: minimal repaint on settings change
- [ ] CHANGELOG entry; PROGRESS.md update

## 9. Conflicts with Existing Code

### 9.1 Confirmed Conflicts

| Area | Existing Code | Conflict | Resolution |
|------|--------------|----------|------------|
| **Theme management** | `CadenceApp.dark: bool` + `design::apply(ctx, dark)` | GlobalSettings introduces `ThemePreference` with `System` option | Unify: `ThemePreference::System` maps to current `dark` state; `apply_theme()` uses the effective theme |
| **Project persistence** | `crates/persistence/src/lib.rs` with `SCHEMA_VERSION = 10` | New settings migration must fit within existing schema versioning | Extend `ensure_schema()` to also run `migrate_project_settings()` — same transaction, no breaking changes |
| **View state** | `CadenceApp` has `view_mode`, `visible_columns`, `table_filter_buffer` etc. scattered | Consolidate into `ViewSettings` struct | Refactor: move fields from `CadenceApp` into `ViewSettings`, update all references; the struct is owned by `CadenceApp` internally |
| **Command layer** | `cadence_commands::Command` trait + `CommandStack` | Setting changes should go through commands for undo/redo | Adopt: create `SetGlobalSettings`, `SetViewSettings`, `SetToolSettings` commands that implement `Command`; UI always goes through the command stack |
| **Undo/redo** | `CommandStack` supports undo/redo for project mutations | Settings changes must also be undoable | Design: `SetGlobalSettings` command stores previous value; `CommandStack.undo()` restores it; same pattern as other command families |
| **SQLite schema** | Migration v0→v10 is complex, hand-written SQL | New settings schema must not disrupt existing migration | Add settings as a new table or JSON blob within existing schema; migration adds column or creates table with `IF NOT EXISTS` |
| **Serialisation** | Some `CadenceApp` fields use `String` buffers, not serde | New settings types must be serde-compatible for persistence | Ensure all new settings types derive `Serialize, Deserialize`; add `#[serde(default)]` for forward compatibility |

### 9.2 Non-Conflicts (Existing Behavior Preserved)

- `dark` theme toggling continues to work as before; `GlobalSettings` is an additional/alternative mechanism
- SQLite project file format unchanged; settings migration is opt-in per project save
- `CommandStack` undo/redo for project commands continues unaffected
- UI state (view mode, column visibility) continues to function; now additionally persisted
- No breaking changes to public API surfaces; all new types are additive

## 10. Migration Path from Current Code

1. **Add `crates/core/src/settings.rs`** with all setting types
2. **Move scattered settings from `CadenceApp`** into the appropriate struct fields
3. **Update `CadenceApp::new()` / `CadenceApp::with_path()`** to initialize settings from defaults or persisted state
4. **Extend `crates/persistence/src/lib.rs`** `save_project()` / `load_project()` to handle settings
5. **Create command types** `SetGlobalSettings`, `SetViewSettings`, `SetToolSettings` in `cadence_commands`
6. **Update UI** to use the new API (`app.view_settings()`, etc.)
7. **Add CI tests** for settings round-trip and fallback resolution
8. **Update CHANGELOG and PROGRESS.md**

## 11. API Signature Summary

```rust
// Core settings types (crates/core/src/settings.rs)
pub struct GlobalSettings { ... }
pub struct ProjectSettings { ... }
pub struct ViewSettings { ... }
pub struct ToolSettings { ... }
pub struct SettingsSync { ... }

// API access (crates/ui-egui/src/lib.rs or cadence_app module)
impl CadenceApp {
    pub fn global_settings(&self) -> &GlobalSettings;
    pub fn set_global_settings(&mut self, settings: GlobalSettings);
    pub fn view_settings(&self) -> &ViewSettings;
    pub fn set_view_settings(&mut self, settings: ViewSettings);
    pub fn tool_settings(&self) -> &ToolSettings;
    pub fn set_tool_settings(&mut self, settings: ToolSettings);
}

// Commands (crates/commands/src/lib.rs)
pub struct SetGlobalSettings { pub settings: GlobalSettings }
pub struct SetViewSettings { pub settings: ViewSettings }
pub struct SetToolSettings { pub settings: ToolSettings }
impl Command for SetGlobalSettings { ... }
impl Command for SetViewSettings { ... }
impl Command for SetToolSettings { ... }

// Persistence (crates/persistence/src/lib.rs)
pub fn save_project_with_settings(project: &Project, path: &Path) -> Result<()>;
pub fn load_project_with_settings(path: &Path) -> Result<Project>;
```

---
*End of specification. This document defines the 4-level settings hierarchy, data structures, persistence strategies, sync mechanism, schema versioning, default/fallback chain, API surface, implementation phases, conflict analysis, and migration path for the CADENCE-SETTINGS epic.*