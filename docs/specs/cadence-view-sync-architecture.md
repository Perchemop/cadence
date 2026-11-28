# CADENCE-VIEW-SYNC Architecture Specification

**Epic:** CADENCE-VIEW-SYNC — Cross-View Settings Sync  
**Version:** 1.0  
**Status:** Draft — Awaiting Human Gate (Guide §2.3 / §8.4)  
**Related Epics:** CADENCE-SETTINGS, CADENCE-TOOLBAR, CADENCE-DESIGN-TOKENS  

---

## 1. Overview

This specification defines the architecture for synchronizing settings across Cadence's views (Table, Gantt, Resources, Baselines, Calendars, Audit, Lookahead, Diary, Production, Assistant) through a four-level hierarchical settings system with observable change propagation, conflict resolution, and full undo/redo integration.

### 1.1 Goals

- **Hierarchical settings**: Global → Project → View → Tool (4 levels) with clear ownership and fallback semantics
- **Reactive synchronization**: Settings changes propagate to all affected views without recreation
- **Conflict resolution**: Deterministic strategy for concurrent edits at different levels
- **Persistence**: Each level owns its settings; SQLite serialization with versioned schema
- **Undo/redo**: All settings changes enter the command stack as first-class operations
- **Initialization**: Views load effective settings from hierarchy on creation
- **Hot-reload**: Runtime changes apply without view destruction/recreation

### 1.2 Non-Goals

- Plugin system for settings (out of scope per Discovery S3)
- Cross-project settings sync (each project file is independent)
- Cloud-based settings sync (local-first architecture)

---

## 2. Settings Hierarchy

### 2.1 Four-Level Model

```
┌─────────────────────────────────────────────────────────────────┐
│ GLOBAL (user-level, ~/.config/cadence/settings.json)            │
│  - Theme (light/dark/system)                                    │
│  - Default density (compact/standard/comfortable)               │
│  - Default Gantt timescale (day/week/month)                     │
│  - Keyboard shortcuts                                           │
│  - Accessibility preferences                                    │
└─────────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ PROJECT (per .cdb SQLite file)                                  │
│  - Project calendar & holidays                                  │
│  - Default view on open (Table/Gantt/Resources)                 │
│  - Column visibility & order (Table view)                       │
│  - Resource grouping mode (Resources view)                      │
│  - Baseline comparison mode                                     │
│  - Working hours per day                                        │
└─────────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ VIEW (per ViewMode instance, stored in Project)                 │
│  Table:     density, sort column, filter, column widths         │
│  Gantt:     timescale, pixels-per-day, split view, critical     │
│             path highlight, non-working day tint                │
│  Resources: grouping, histogram scale, cost display mode        │
│  Baselines: comparison baseline, variance columns               │
│  Calendars: selected calendar, week start day                   │
│  Audit:     status date, active checks                          │
│  Lookahead: window start, weeks, snapshot filter                │
│  Diary:     date range, activity filter                         │
│  Production: date range, activity filter                        │
│  Assistant: model, provider, temperature (no API key)           │
└─────────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ TOOL (transient, per-widget, not persisted)                     │
│  - Column resizer drag state                                    │
│  - Gantt bar drag preview                                       │
│  - Tooltip visibility                                           │
│  - Context menu position                                        │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Effective Settings Resolution

```
Effective(view_setting) = 
  View.value ?? Project.value ?? Global.value ?? Default
```

**Resolution order** (highest priority first):
1. View-level override (explicit user choice for this view)
2. Project-level default (project-specific preference)
3. Global-level default (user preference across projects)
4. Hard-coded default (compiled-in fallback)

### 2.3 Ownership Matrix

| Setting Category | Global | Project | View | Tool |
|------------------|--------|---------|------|------|
| Theme            | ✅ Owner | — | — | — |
| Density default  | ✅ Owner | Override | Override | — |
| Timescale default| ✅ Owner | Override | Override | — |
| Column visibility| — | ✅ Owner | Override | — |
| Column order     | — | ✅ Owner | Override | — |
| Sort column      | — | — | ✅ Owner | — |
| Filter text      | — | — | ✅ Owner | — |
| Gantt pixels/day | — | Override | ✅ Owner | Drag |
| Resource grouping| — | ✅ Owner | Override | — |
| Histogram scale  | — | — | ✅ Owner | — |
| Window geometry  | — | — | ✅ Owner | — |
| Shortcuts        | ✅ Owner | — | — | — |

---

## 3. Data Structures

### 3.1 Core Settings Types (crates/core/src/settings.rs — NEW)

```rust
// ===================== GLOBAL SETTINGS =====================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    pub theme: ThemeMode,
    pub default_density: Density,
    pub default_gantt_timescale: GanttTimescale,
    pub shortcuts: ShortcutMap,
    pub accessibility: AccessibilitySettings,
    pub version: u32, // Schema version for migration
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    Standard,
    Comfortable,
}

impl Density {
    pub fn spacing_multiplier(self) -> f32 {
        match self {
            Density::Compact => 0.75,
            Density::Standard => 1.0,
            Density::Comfortable => 1.33,
        }
    }
    
    pub fn row_height(self) -> f32 {
        match self {
            Density::Compact => 20.0,
            Density::Standard => 24.0,
            Density::Comfortable => 32.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GanttTimescale {
    Day,
    Week,
    Month,
    Quarter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ShortcutMap {
    #[serde(flatten)]
    pub bindings: BTreeMap<String, String>, // "action" -> "key"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccessibilitySettings {
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub focus_ring_width: u8,
    pub announce_changes: bool,
}

// ===================== PROJECT SETTINGS =====================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub default_view: ViewMode,
    pub column_visibility: BTreeMap<String, bool>, // column_id -> visible
    pub column_order: Vec<String>,                  // ordered column_ids
    pub resource_grouping: ResourceGrouping,
    pub baseline_comparison: BaselineComparison,
    pub working_hours_per_day: f32,
    pub default_density: Option<Density>,        // Override global
    pub default_timescale: Option<GanttTimescale>, // Override global
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceGrouping {
    None,
    ByKind,
    ByWbs,
    ByActivity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaselineComparison {
    Latest,
    Selected,
    All,
}

// ===================== VIEW SETTINGS =====================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewSettings {
    pub table: TableViewSettings,
    pub gantt: GanttViewSettings,
    pub resources: ResourcesViewSettings,
    pub baselines: BaselinesViewSettings,
    pub calendars: CalendarsViewSettings,
    pub audit: AuditViewSettings,
    pub lookahead: LookaheadViewSettings,
    pub diary: DiaryViewSettings,
    pub production: ProductionViewSettings,
    pub assistant: AssistantViewSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableViewSettings {
    pub density: Option<Density>,
    pub sort_column: Option<String>,
    pub sort_ascending: bool,
    pub filter_text: String,
    pub critical_only: bool,
    pub column_widths: BTreeMap<String, f32>,
    pub visible_columns: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GanttViewSettings {
    pub timescale: Option<GanttTimescale>,
    pub pixels_per_day: f32,
    pub show_split: bool,
    pub highlight_critical_path: bool,
    pub non_working_tint: bool,
    pub show_today_line: bool,
    pub show_status_date_line: bool,
    pub bar_height: f32,
    pub row_height: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesViewSettings {
    pub grouping: Option<ResourceGrouping>,
    pub histogram_scale: HistogramScale,
    pub show_costs: bool,
    pub cost_display: CostDisplayMode,
    pub density: Option<Density>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistogramScale {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostDisplayMode {
    Total,
    PerDay,
    PerUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BaselinesViewSettings {
    pub comparison_mode: BaselineComparison,
    pub selected_baseline: Option<BaselineId>,
    pub show_variance_columns: bool,
    pub variance_mode: VarianceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarianceMode {
    Duration,
    Cost,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CalendarsViewSettings {
    pub selected_calendar: Option<CalendarId>,
    pub week_starts_monday: bool,
    pub show_holidays: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuditViewSettings {
    pub status_date: Option<Date>,
    pub active_checks: BTreeSet<String>,
    pub group_by: AuditGroupBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditGroupBy {
    Check,
    Activity,
    Severity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LookaheadViewSettings {
    pub window_start: Date,
    pub window_weeks: u32,
    pub snapshot_filter: Option<LookaheadSnapshotId>,
    pub show_promised_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiaryViewSettings {
    pub date_range: Option<(Date, Date)>,
    pub activity_filter: Option<ActivityId>,
    pub show_lookahead_notes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProductionViewSettings {
    pub date_range: Option<(Date, Date)>,
    pub activity_filter: Option<ActivityId>,
    pub group_by_date: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssistantViewSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: f32,
    // api_key intentionally omitted — session only
}

// ===================== COMPOSITE =====================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsContext {
    pub global: GlobalSettings,
    pub project: ProjectSettings,
    pub view: ViewSettings,
}
```

### 3.2 Settings Keys (for Observer Pattern)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsKey {
    // Global
    Theme,
    DefaultDensity,
    DefaultGanttTimescale,
    Shortcuts,
    Accessibility,
    
    // Project
    DefaultView,
    ColumnVisibility,
    ColumnOrder,
    ResourceGrouping,
    BaselineComparison,
    WorkingHoursPerDay,
    ProjectDefaultDensity,
    ProjectDefaultTimescale,
    
    // View (Table)
    TableDensity,
    TableSortColumn,
    TableSortAscending,
    TableFilterText,
    TableCriticalOnly,
    TableColumnWidths,
    TableVisibleColumns,
    
    // View (Gantt)
    GanttTimescale,
    GanttPixelsPerDay,
    GanttShowSplit,
    GanttHighlightCriticalPath,
    GanttNonWorkingTint,
    GanttShowTodayLine,
    GanttShowStatusDateLine,
    GanttBarHeight,
    GanttRowHeight,
    
    // View (Resources)
    ResourcesGrouping,
    ResourcesHistogramScale,
    ResourcesShowCosts,
    ResourcesCostDisplay,
    ResourcesDensity,
    
    // ... other view settings
}
```

---

## 4. Observer Pattern — Change Notification

### 4.1 Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    SettingsStore (Actor)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Global Layer │  │ Project Layer│  │ View Layer (per view)│  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                       │             │
│         └─────────────────┼───────────────────────┘             │
│                           ▼                                      │
│              ┌────────────────────────┐                          │
│              │  Subscription Registry │                          │
│              │  (SettingsKey → Vec<  │                          │
│              │   Weak<Observer>>)     │                          │
│              └───────────┬────────────┘                          │
│                           │                                       │
│         ┌─────────────────┼─────────────────┐                   │
│         ▼                 ▼                 ▼                   │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐            │
│  │ Table View │    │ Gantt View │    │Resources...│            │
│  │  Observer  │    │  Observer  │    │  Observer  │            │
│  └────────────┘    └────────────┘    └────────────┘            │
└────────────────────────────────────────────────────────────────┘
```

### 4.2 Implementation (crates/core/src/settings_store.rs — NEW)

```rust
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Weak};
use std::any::Any;
use crossbeam::channel::{Sender, Receiver, unbounded};

/// Trait for objects that react to settings changes.
pub trait SettingsObserver: Send + Sync {
    /// Called when a setting the observer subscribed to changes.
    /// `key` identifies what changed; `context` is the full effective settings.
    fn on_settings_changed(&self, key: SettingsKey, context: &SettingsContext);
    
    /// Optional: called before the change is committed (for validation).
    fn on_settings_changing(&self, _key: SettingsKey, _proposed: &SettingsContext) -> Result<(), String> {
        Ok(())
    }
}

/// Central settings store with reactive notification.
pub struct SettingsStore {
    global: GlobalSettings,
    project: ProjectSettings,
    views: BTreeMap<ViewMode, ViewSettings>,
    subscribers: BTreeMap<SettingsKey, Vec<Weak<dyn SettingsObserver>>>,
    tx: Sender<SettingsEvent>,
    rx: Receiver<SettingsEvent>,
}

#[derive(Debug, Clone)]
pub enum SettingsEvent {
    Changed { key: SettingsKey, context: SettingsContext },
    Batch(Vec<SettingsEvent>),
}

impl SettingsStore {
    pub fn new(global: GlobalSettings, project: ProjectSettings) -> Self {
        let (tx, rx) = unbounded();
        Self {
            global,
            project,
            views: BTreeMap::new(),
            subscribers: BTreeMap::new(),
            tx,
            rx,
        }
    }
    
    /// Subscribe to changes for a specific key.
    pub fn subscribe(&mut self, key: SettingsKey, observer: Arc<dyn SettingsObserver>) {
        self.subscribers
            .entry(key)
            .or_default()
            .push(Arc::downgrade(&observer));
    }
    
    /// Subscribe to multiple keys at once.
    pub fn subscribe_multi(&mut self, keys: &[SettingsKey], observer: Arc<dyn SettingsObserver>) {
        for key in keys {
            self.subscribe(*key, observer.clone());
        }
    }
    
    /// Get effective setting for a view (resolves hierarchy).
    pub fn effective_for_view(&self, view: ViewMode) -> SettingsContext {
        let view_settings = self.views.get(&view).cloned().unwrap_or_default();
        SettingsContext {
            global: self.global.clone(),
            project: self.project.clone(),
            view: view_settings,
        }
    }
    
    /// Update a global setting.
    pub fn set_global(&mut self, key: SettingsKey, value: impl Serialize) -> Result<(), String> {
        self.apply_global(key, value)?;
        self.notify(key);
        Ok(())
    }
    
    /// Update a project setting.
    pub fn set_project(&mut self, key: SettingsKey, value: impl Serialize) -> Result<(), String> {
        self.apply_project(key, value)?;
        self.notify(key);
        Ok(())
    }
    
    /// Update a view setting.
    pub fn set_view(&mut self, view: ViewMode, key: SettingsKey, value: impl Serialize) -> Result<(), String> {
        self.apply_view(view, key, value)?;
        self.notify(key);
        Ok(())
    }
    
    /// Batch multiple changes into one notification cycle.
    pub fn batch(&mut self, f: impl FnOnce(&mut Self)) {
        let mut events = Vec::new();
        let tx = self.tx.clone();
        let mut batch_store = BatchStore { events: &mut events, inner: self };
        f(&mut batch_store);
        for event in events {
            let _ = tx.send(event);
        }
    }
    
    fn notify(&self, key: SettingsKey) {
        let context = self.effective_for_view(ViewMode::Table); // Default view for context
        let _ = self.tx.send(SettingsEvent::Changed { key, context });
    }
    
    /// Process pending events (call from UI thread each frame).
    pub fn process_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            self.dispatch(event);
        }
    }
    
    fn dispatch(&mut self, event: SettingsEvent) {
        match event {
            SettingsEvent::Changed { key, context } => {
                if let Some(subs) = self.subscribers.get(&key) {
                    for weak in subs {
                        if let Some(observer) = weak.upgrade() {
                            observer.on_settings_changed(key, &context);
                        }
                    }
                }
            }
            SettingsEvent::Batch(events) => {
                for e in events {
                    self.dispatch(e);
                }
            }
        }
    }
}

struct BatchStore<'a> {
    events: &'a mut Vec<SettingsEvent>,
    inner: &'a mut SettingsStore,
}

impl BatchStore<'_> {
    fn set_global(&mut self, key: SettingsKey, value: impl Serialize) -> Result<(), String> {
        self.inner.apply_global(key, value)?;
        self.events.push(SettingsEvent::Changed { 
            key, 
            context: self.inner.effective_for_view(ViewMode::Table) 
        });
        Ok(())
    }
    // ... similar for project/view
}
```

### 4.3 View Integration (ui-egui)

Each view implements `SettingsObserver` and registers on creation:

```rust
impl SettingsObserver for TableView {
    fn on_settings_changed(&self, key: SettingsKey, context: &SettingsContext) {
        match key {
            SettingsKey::TableDensity => self.update_density(context),
            SettingsKey::TableSortColumn => self.update_sort(context),
            SettingsKey::TableFilterText => self.update_filter(context),
            SettingsKey::TableColumnWidths => self.update_column_widths(context),
            SettingsKey::GlobalDefaultDensity => self.maybe_update_density(context),
            SettingsKey::ProjectDefaultDensity => self.maybe_update_density(context),
            _ => {}
        }
    }
}

// In CadenceApp::new() or view creation:
fn create_table_view(&mut self) -> TableView {
    let mut view = TableView::new();
    let observer = Arc::new(view.clone());
    self.settings_store.subscribe_multi(
        &TABLE_VIEW_KEYS, 
        observer
    );
    view
}

const TABLE_VIEW_KEYS: &[SettingsKey] = &[
    SettingsKey::TableDensity,
    SettingsKey::TableSortColumn,
    SettingsKey::TableSortAscending,
    SettingsKey::TableFilterText,
    SettingsKey::TableCriticalOnly,
    SettingsKey::TableColumnWidths,
    SettingsKey::TableVisibleColumns,
    SettingsKey::GlobalDefaultDensity,
    SettingsKey::ProjectDefaultDensity,
];
```

---

## 5. Conflict Resolution

### 5.1 Strategy: Explicit Priority with User Prompt on Ambiguity

| Conflict Scenario | Resolution |
|-------------------|------------|
| Same key, different levels (Global vs Project) | Higher level (Project) wins — explicit user choice at project level overrides global default |
| Same level, concurrent edits (two views) | Last-write-wins with timestamp; log conflict for audit |
| Project setting conflicts with Global explicit user choice | Project wins (project is more specific context) |
| View setting conflicts with Project default | View wins (view is explicit user choice for that view) |
| Tool-level transient state vs View setting | Tool wins (transient interaction takes precedence) |

### 5.2 Implementation

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    HigherLevelWins,      // Project > Global, View > Project
    LastWriteWins,        // Timestamp-based
    PromptUser,           // Show dialog for ambiguous conflicts
    Merge,                // For composite settings (e.g., column visibility + order)
}

impl SettingsStore {
    fn resolve_conflict(
        &self,
        key: SettingsKey,
        existing_level: SettingsLevel,
        new_level: SettingsLevel,
        existing_value: &dyn Any,
        new_value: &dyn Any,
    ) -> ConflictResolution {
        use SettingsLevel::*;
        use ConflictResolution::*;
        
        match (existing_level, new_level) {
            (Global, Project) | (Global, View) | (Project, View) => HigherLevelWins,
            (View, Project) | (View, Global) | (Project, Global) => {
                // Lower level trying to override higher — only allow if explicit
                if self.is_explicit_user_choice(new_level, key) {
                    HigherLevelWins // The new (higher) level wins because it's explicit
                } else {
                    LastWriteWins
                }
            }
            (Tool, _) => HigherLevelWins, // Tool always wins for transient
            (_, Tool) => LastWriteWins,   // Persistent wins over transient
            _ => LastWriteWins,
        }
    }
}
```

---

## 6. Persistence

### 6.1 Storage Locations

| Level | Location | Format |
|-------|----------|--------|
| Global | `~/.config/cadence/settings.json` | JSON (serde) |
| Project | Embedded in `.cdb` SQLite (new tables) | SQL tables |
| View | Embedded in `.cdb` SQLite (per-view rows) | SQL tables |
| Tool | Memory only (never persisted) | — |

### 6.2 SQLite Schema Extensions (crates/persistence)

```sql
-- Global settings are NOT in SQLite (user-level file)

-- Project-level settings (one row per project)
CREATE TABLE project_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    default_view TEXT NOT NULL DEFAULT 'Table',
    column_visibility TEXT NOT NULL DEFAULT '{}',      -- JSON map
    column_order TEXT NOT NULL DEFAULT '[]',           -- JSON array
    resource_grouping TEXT NOT NULL DEFAULT 'none',
    baseline_comparison TEXT NOT NULL DEFAULT 'latest',
    working_hours_per_day REAL NOT NULL DEFAULT 8.0,
    default_density TEXT,                              -- NULL = use global
    default_timescale TEXT,                            -- NULL = use global
    schema_version INTEGER NOT NULL DEFAULT 1
);

-- View-level settings (one row per view mode per project)
CREATE TABLE view_settings (
    view_mode TEXT NOT NULL,          -- 'Table', 'Gantt', 'Resources', etc.
    settings_json TEXT NOT NULL,      -- Full ViewSettings for that mode
    PRIMARY KEY (view_mode)
);

-- Tool settings: NOT persisted
```

### 6.3 Serialization

```rust
// In crates/persistence/src/lib.rs — extend save_project/load_project

fn write_project_settings(transaction: &Transaction, project: &Project) -> Result<()> {
    // Project settings
    let proj_settings = ProjectSettings {
        default_view: project.ui_default_view, // New field on Project
        column_visibility: project.ui_column_visibility,
        column_order: project.ui_column_order,
        resource_grouping: project.ui_resource_grouping,
        baseline_comparison: project.ui_baseline_comparison,
        working_hours_per_day: project.working_hours_per_day,
        default_density: project.ui_default_density,
        default_timescale: project.ui_default_timescale,
        version: 1,
    };
    let proj_json = serde_json::to_string(&proj_settings)?;
    
    transaction.execute(
        "INSERT OR REPLACE INTO project_settings (id, default_view, column_visibility, 
            column_order, resource_grouping, baseline_comparison, working_hours_per_day,
            default_density, default_timescale, schema_version)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
        params![
            proj_settings.default_view.to_string(),
            serde_json::to_string(&proj_settings.column_visibility)?,
            serde_json::to_string(&proj_settings.column_order)?,
            proj_settings.resource_grouping.to_string(),
            proj_settings.baseline_comparison.to_string(),
            proj_settings.working_hours_per_day,
            proj_settings.default_density.map(|d| d.to_string()),
            proj_settings.default_timescale.map(|t| t.to_string()),
        ],
    )?;
    
    // View settings — one row per view mode
    for (view_mode, view_settings) in &project.ui_view_settings {
        let view_json = serde_json::to_string(view_settings)?;
        transaction.execute(
            "INSERT OR REPLACE INTO view_settings (view_mode, settings_json)
             VALUES (?1, ?2)",
            params![view_mode.to_string(), view_json],
        )?;
    }
    Ok(())
}

fn read_project_settings(connection: &Connection) -> Result<ProjectSettings> {
    // ... read project_settings table, deserialize JSON columns
}
```

### 6.4 Global Settings File

```rust
// crates/core/src/global_settings.rs — NEW

impl GlobalSettings {
    pub fn path() -> Option<PathBuf> {
        config_dir().map(|dir| dir.join("cadence").join("settings.json"))
    }
    
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    
    pub fn save(&self) -> Result<(), String> {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|_| "failed to serialize global settings")?;
            std::fs::write(path, json).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
```

---

## 7. Undo/Redo Integration

### 7.1 Command Types (crates/commands/src/settings.rs — NEW)

```rust
use crate::Command;
use cadence_core::model::Project;
use cadence_core::settings::{GlobalSettings, ProjectSettings, ViewSettings, SettingsKey, SettingsLevel};
use serde::{Serialize, Deserialize};

/// Change a setting at any level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetSetting {
    pub level: SettingsLevel,
    pub view_mode: Option<ViewMode>, // Required for View level
    pub key: SettingsKey,
    pub value: SettingValue,         // Serialized value
    pub previous: SettingValue,      // For undo
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    Bool(bool),
    String(String),
    Number(f64),
    Int(i64),
    Map(serde_json::Map<String, SettingValue>),
    Array(Vec<SettingValue>),
    Null,
}

impl Command for SetSetting {
    fn description(&self) -> String {
        format!("set {} {} to {:?}", self.level, self.key, self.value)
    }
    
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        // Capture current value for inverse
        let current = self.read_current(project)?;
        let inverse = SetSetting {
            level: self.level,
            view_mode: self.view_mode,
            key: self.key,
            value: current,
            previous: self.value.clone(),
        };
        
        // Apply the new value
        self.write_current(project, &self.value)?;
        
        Ok(Box::new(inverse))
    }
}

impl SetSetting {
    fn read_current(&self, project: &mut Project) -> Result<SettingValue, String> {
        match self.level {
            SettingsLevel::Global => {
                // Global settings not in project — return null (handled at app level)
                Ok(SettingValue::Null)
            }
            SettingsLevel::Project => self.read_project_setting(project),
            SettingsLevel::View => self.read_view_setting(project),
            SettingsLevel::Tool => Ok(SettingValue::Null), // Tool settings not undoable
        }
    }
    
    fn write_current(&self, project: &mut Project, value: &SettingValue) -> Result<(), String> {
        match self.level {
            SettingsLevel::Project => self.write_project_setting(project, value),
            SettingsLevel::View => self.write_view_setting(project, value),
            _ => Ok(()), // Global/Tool handled at app level
        }
    }
    
    // ... helpers to read/write specific settings from Project
}

/// Batch multiple setting changes as one undo step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchSettings {
    pub changes: Vec<SetSetting>,
}

impl Command for BatchSettings {
    fn description(&self) -> String {
        format!("batch {} setting changes", self.changes.len())
    }
    
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let mut inverses = Vec::new();
        for change in &mut self.changes {
            let inverse = change.apply(project)?;
            inverses.push(inverse);
        }
        inverses.reverse();
        Ok(Box::new(BatchSettings { changes: inverses }))
    }
}
```

### 7.2 App-Level Integration

```rust
// In CadenceApp (ui-egui)

fn change_setting(&mut self, level: SettingsLevel, view: Option<ViewMode>, key: SettingsKey, value: SettingValue) {
    let current = self.settings_store.get_effective(level, view, key);
    let cmd = SetSetting {
        level,
        view_mode: view,
        key,
        value,
        previous: current,
    };
    
    if let Err(e) = self.stack.execute(&mut self.project, Box::new(cmd)) {
        self.set_status(e);
        return;
    }
    
    // Update the store (triggers observers)
    self.settings_store.apply_change(level, view, key, value);
    self.recompute();
}
```

---

## 8. Initialization Sequence

### 8.1 Application Startup

```
1. Load GlobalSettings from ~/.config/cadence/settings.json
   │
   ├─ Missing/corrupt → use defaults, log warning
   │
2. Load Project from .cdb file
   │
   ├─ Read project_settings table → ProjectSettings
   ├─ Read view_settings table → ViewSettings per ViewMode
   │
3. Construct SettingsStore(global, project)
   │
4. Create View instances (TableView, GanttView, etc.)
   │
   ├─ Each view calls settings_store.effective_for_view(mode)
   ├─ Each view subscribes to its SettingsKey set
   │
5. Apply theme from effective SettingsContext.global.theme
   │
6. Render initial frame with resolved settings
```

### 8.2 View Creation (Lazy)

```rust
impl CadenceApp {
    fn ensure_view(&mut self, mode: ViewMode) {
        if self.views.contains_key(&mode) { return; }
        
        let context = self.settings_store.effective_for_view(mode);
        let view = match mode {
            ViewMode::Table => Box::new(TableView::new(context)),
            ViewMode::Gantt => Box::new(GanttView::new(context)),
            // ...
        };
        
        // Subscribe to relevant keys
        let observer = Arc::new(view.clone());
        self.settings_store.subscribe_multi(VIEW_KEYS[mode], observer);
        
        self.views.insert(mode, view);
    }
}
```

### 8.3 Default Settings (Compiled-In)

```rust
impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            default_density: Density::Standard,
            default_gantt_timescale: GanttTimescale::Week,
            shortcuts: ShortcutMap::default(),
            accessibility: AccessibilitySettings::default(),
            version: 1,
        }
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            default_view: ViewMode::Table,
            column_visibility: DEFAULT_COLUMN_VISIBILITY,
            column_order: DEFAULT_COLUMN_ORDER,
            resource_grouping: ResourceGrouping::None,
            baseline_comparison: BaselineComparison::Latest,
            working_hours_per_day: 8.0,
            default_density: None,
            default_timescale: None,
            version: 1,
        }
    }
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            table: TableViewSettings { 
                sort_ascending: true, 
                ..Default::default() 
            },
            gantt: GanttViewSettings {
                pixels_per_day: 14.0,
                show_split: true,
                highlight_critical_path: true,
                non_working_tint: true,
                show_today_line: true,
                show_status_date_line: true,
                bar_height: 20.0,
                row_height: 24.0,
                ..Default::default()
            },
            resources: ResourcesViewSettings {
                histogram_scale: HistogramScale::Week,
                show_costs: true,
                cost_display: CostDisplayMode::Total,
                ..Default::default()
            },
            // ... other views
        }
    }
}
```

---

## 9. Hot-Reload / Runtime Propagation

### 9.1 Requirements

- Settings changes apply immediately without view destruction
- No flicker or layout thrashing
- Observers receive effective context, compute minimal diff
- Batch rapid changes (e.g., slider drag) into single frame update

### 9.2 Implementation Strategy

```rust
// In SettingsStore
pub fn set_view_batched(&mut self, view: ViewMode, key: SettingsKey, value: impl Serialize) {
    // Defer notification until end of frame
    self.pending_changes.push((view, key, value));
}

// In CadenceApp::update() — called once per frame
fn update(&mut self, ctx: &Context) {
    // 1. Process any pending batched changes
    self.settings_store.flush_pending();
    
    // 2. Process observer notifications
    self.settings_store.process_events();
    
    // 3. Normal UI update
    // ...
}

// In SettingsStore
fn flush_pending(&mut self) {
    if self.pending_changes.is_empty() { return; }
    
    let mut events = Vec::new();
    for (view, key, value) in self.pending_changes.drain(..) {
        self.apply_view(view, key, value)?;
        let context = self.effective_for_view(view);
        events.push(SettingsEvent::Changed { key, context });
    }
    
    // Send as single batch
    let _ = self.tx.send(SettingsEvent::Batch(events));
}
```

### 9.3 Observer Diff Pattern

```rust
impl SettingsObserver for TableView {
    fn on_settings_changed(&self, key: SettingsKey, context: &SettingsContext) {
        // Compute minimal diff, only update what changed
        match key {
            SettingsKey::TableDensity => {
                let new_density = context.effective_table_density();
                if new_density != self.current_density {
                    self.current_density = new_density;
                    self.request_repaint(); // Mark for relayout
                }
            }
            SettingsKey::TableColumnWidths => {
                let new_widths = context.view.table.column_widths.clone();
                if new_widths != self.column_widths {
                    self.column_widths = new_widths;
                    // No repaint needed — egui reads widths each frame
                }
            }
            // ...
        }
    }
}
```

---

## 10. Data Flow Diagrams

### 10.1 Settings Read Path

```
┌─────────────┐     ┌──────────────────┐     ┌────────────────┐
│   View      │────▶│ SettingsStore    │────▶│  SettingsContext│
│  .render()  │     │ .effective_for() │     │  (resolved)    │
└─────────────┘     └──────────────────┘     └────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         ┌─────────┐ ┌───────────┐ ┌─────────┐
         │ Global  │ │  Project  │ │  View   │
         │ Layer   │ │  Layer    │ │  Layer  │
         └─────────┘ └───────────┘ └─────────┘
```

### 10.2 Settings Write Path (User Action)

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌────────────┐
│  User UI    │────▶│  Command     │────▶│  Settings   │────▶│ Observers  │
│  (slider,   │     │  (SetSetting)│     │  Store      │     │ (Views)    │
│   combo)    │     │              │     │ .apply()    │     │ .notify()  │
└─────────────┘     └──────────────┘     └─────────────┘     └────────────┘
                           │                    │                    │
                           ▼                    ▼                    ▼
                    ┌─────────────┐     ┌──────────────┐     ┌──────────────┐
                    │  Undo Stack │     │  Persistence │     │  Minimal     │
                    │  (inverse)  │     │  (async)     │     │  Repaint     │
                    └─────────────┘     └──────────────┘     └──────────────┘
```

### 10.3 Cross-View Sync Example

```
User changes "Density" to "Comfortable" in Table View toolbar
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ SetSetting command:                                 │
│   level: View, view: Table, key: TableDensity,      │
│   value: Comfortable                                │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ SettingsStore.apply()                               │
│   - Updates ViewSettings.table.density              │
│   - Notifies subscribers of TableDensity            │
└─────────────────────────────────────────────────────┘
         │
         ├──────────────────┬──────────────────┐
         ▼                  ▼                  ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ TableView       │ │ GanttView       │ │ ResourcesView   │
│ (subscribed)    │ │ (subscribed to  │ │ (subscribed to  │
│                 │ │  GlobalDefault  │ │  GlobalDefault  │
│ Updates row     │ │                 │ │                 │
│ height, spacing │ │ If no View      │ │ If no View      │
│ immediately     │ │ override, uses  │ │ override, uses  │
│                 │ │ Global default  │ │ Global default  │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

---

## 11. Migration Strategy

### 11.1 Schema Versioning

- Global settings: `version` field in JSON, migrate on load
- Project settings: `schema_version` column in `project_settings` table
- View settings: versioned inside `settings_json` via `version` field

### 11.2 Migration Steps

1. **Add `project_settings` and `view_settings` tables** (schema v11)
2. **On load**: if tables missing, create with defaults; if version < current, migrate JSON
3. **On save**: always write current schema version
4. **Global settings**: add `version` field, migrate missing keys on load

---

## 12. Testing Strategy

### 12.1 Unit Tests (crates/core/tests/settings.rs)

```rust
#[test]
fn effective_settings_resolves_hierarchy() {
    let global = GlobalSettings { default_density: Density::Compact, ..Default::default() };
    let project = ProjectSettings { default_density: Some(Density::Standard), ..Default::default() };
    let view = TableViewSettings { density: Some(Density::Comfortable), ..Default::default() };
    
    let store = SettingsStore::new(global, project);
    store.set_view(ViewMode::Table, SettingsKey::TableDensity, Density::Comfortable).unwrap();
    
    let ctx = store.effective_for_view(ViewMode::Table);
    assert_eq!(ctx.effective_table_density(), Density::Comfortable);
}

#[test]
fn observer_notified_on_change() {
    let mut store = SettingsStore::new(Default::default(), Default::default());
    let observer = Arc::new(TestObserver::new());
    store.subscribe(SettingsKey::TableDensity, observer.clone());
    
    store.set_view(ViewMode::Table, SettingsKey::TableDensity, Density::Compact).unwrap();
    store.process_events();
    
    assert_eq!(observer.last_key(), Some(SettingsKey::TableDensity));
}

#[test]
fn batch_changes_single_notification() {
    let mut store = SettingsStore::new(Default::default(), Default::default());
    let observer = Arc::new(TestObserver::new());
    store.subscribe_multi(&[SettingsKey::TableDensity, SettingsKey::TableFilterText], observer.clone());
    
    store.batch(|s| {
        s.set_view(ViewMode::Table, SettingsKey::TableDensity, Density::Compact).unwrap();
        s.set_view(ViewMode::Table, SettingsKey::TableFilterText, "test").unwrap();
    });
    store.process_events();
    
    // Should receive one batch event, not two individual
    assert_eq!(observer.batch_count(), 1);
}
```

### 12.2 Integration Tests (crates/ui-egui/tests/view_sync.rs)

```rust
#[test]
fn density_change_propagates_to_gantt() {
    let mut app = CadenceApp::new(Project::new("Test", today()));
    app.set_view_mode(ViewMode::Table);
    
    // Change density in Table view
    app.change_setting(
        SettingsLevel::View,
        Some(ViewMode::Table),
        SettingsKey::TableDensity,
        SettingValue::String("Comfortable".into()),
    );
    
    // Switch to Gantt — should reflect global default since no view override
    app.set_view_mode(ViewMode::Gantt);
    let gantt_ctx = app.settings_store.effective_for_view(ViewMode::Gantt);
    
    // Gantt uses Global default (Standard) because no View override
    assert_eq!(gantt_ctx.effective_gantt_density(), Density::Standard);
}

#[test]
fn undo_redo_settings() {
    let mut app = CadenceApp::new(Project::new("Test", today()));
    
    app.change_setting(..., SettingsKey::TableDensity, Density::Compact);
    assert_eq!(app.current_table_density(), Density::Compact);
    
    app.stack.undo(&mut app.project).unwrap();
    assert_eq!(app.current_table_density(), Density::Standard);
    
    app.stack.redo(&mut app.project).unwrap();
    assert_eq!(app.current_table_density(), Density::Compact);
}
```

---

## 13. Accessibility Considerations

- All settings changes announced via `AccessibilitySettings.announce_changes`
- Focus preserved when settings dialog closes
- High contrast mode affects settings UI colors (uses design tokens)
- Reduced motion disables animation in settings transitions
- Keyboard navigation for all settings controls

---

## 14. Performance

- **Subscription lookup**: O(log n) via BTreeMap
- **Notification dispatch**: O(k) where k = subscribers for key (typically 1-3 views)
- **Effective settings resolution**: O(1) per key (cached per view)
- **Batch processing**: Coalesces multiple rapid changes into single frame
- **Memory**: Weak references prevent observer leaks

---

## 15. Open Questions (Requiring Human Gate)

| # | Question | Recommendation |
|---|----------|----------------|
| 1 | Should View-level settings persist across app restarts? | **Yes** — store in project SQLite |
| 2 | Should Global settings sync across machines (cloud)? | **No** — local-first per guide |
| 3 | Conflict resolution: prompt user or auto-resolve? | **Auto-resolve with HigherLevelWins**, log for audit |
| 4 | Density levels: 3 (Compact/Standard/Comfortable) or 4? | **3** — matches Discovery H1 |
| 5 | Tool-level settings in undo stack? | **No** — transient, not user-intent |

---

## 16. Implementation Phases

| Phase | Scope | Deliverable |
|-------|-------|-------------|
| 1 | Core types + SettingsStore + Observer | `crates/core/src/settings.rs`, `settings_store.rs` |
| 2 | Persistence (SQLite schema + migration) | `crates/persistence` v11 schema |
| 3 | Commands (SetSetting, BatchSettings) | `crates/commands/src/settings.rs` |
| 4 | Global settings file I/O | `crates/core/src/global_settings.rs` |
| 5 | View integration (Table, Gantt, Resources) | Observer impl in each view |
| 6 | UI controls (density picker, timescale, etc.) | Toolbar/settings panel |
| 7 | Tests + CI | Unit + integration tests green |

---

## 17. References

- Discovery: `cadence/docs/discovery/inspiration-apps-ui-ux-discovery.md`
- Guide: `guide.md` §II.30 (Rust GUI/UX), §5.6 (Workflow Protocol)
- Design Tokens: `crates/ui-egui/src/design.rs`
- Commands: `crates/commands/src/lib.rs`
- Persistence: `crates/persistence/src/lib.rs`
- App State: `crates/ui-egui/src/lib.rs` (CadenceApp, ViewMode, VIEW_META)

---

**End of Specification**