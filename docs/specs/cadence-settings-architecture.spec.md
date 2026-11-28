# CADENCE-SETTINGS Architecture Specification

**Epic:** CADENCE-SETTINGS — Settings Architecture v1  
**Status:** Draft — Awaiting Review  
**Based on:** `docs/discovery/inspiration-apps-ui-ux-discovery.md`  
**Target:** Cadence v0.3.0 (post v0.1.0 foundation)

---

## 1. Overview

This specification defines the complete settings architecture for Cadence, implementing a **4-level hierarchical settings system** (Global → Project → View → Tool) with:

- **Per-level persistence** (file-based, SQLite, in-memory)
- **Change notification/sync** across levels without circular updates
- **Schema versioning and migration** for each level
- **Default values and fallback chain** (Tool → View → Project → Global → Built-in)
- **API surface** for UI consumption (egui/eframe)

The design follows the discovery hypotheses H1–H7 and addresses technical questions T1–T5.

---

## 2. Settings Hierarchy (4 Levels)

### 2.1 Level Definitions

| Level | Scope | Lifetime | Persistence | Example Settings |
|-------|-------|----------|-------------|------------------|
| **Global** | Application-wide, user-specific | Across all projects | File (JSON/TOML) in config dir | Theme, density, keyboard shortcuts, update channel, assistant config |
| **Project** | Single project file | Per `.cadence` SQLite | Embedded in project DB | Default calendar, currency, cost settings, column visibility, baselines |
| **View** | Single view instance (Table, Gantt, Resources, etc.) | Per view per project | In-memory (optional: project DB) | Column order/sort, filter, zoom, split ratio, visible columns |
| **Tool** | Single tool/context within a view | Ephemeral (session) | In-memory only | Inspector panel width, Gantt bar height, resource histogram scale |

### 2.2 Hierarchy Visualization

```
┌─────────────────────────────────────────────────────────────┐
│ GLOBAL (user config dir)                                    │
│  • theme: dark/light/system                                   │
│  • density: compact/standard/comfortable                      │
│  • shortcuts: { "new_activity": "Ctrl+N", ... }              │
│  • update_channel: stable/beta                                │
│  • assistant: { model, base_url, api_key }                   │
└─────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ PROJECT (embedded in .cadence SQLite)                       │
│  • default_calendar: CalendarId                             │
│  • currency: "USD" / "EUR" / ...                            │
│  • cost_settings: { default_rate, overtime_multiplier }     │
│  • visible_columns: [Name, Duration, Start, ...]            │
│  • baseline_ids: [BaselineId]                               │
│  • lookahead_defaults: { weeks: 2, auto_refresh: true }     │
└─────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ VIEW (in-memory; optional: project DB for persistence)      │
│  TableView:                                                 │
│    • sort_key: Name, sort_ascending: true                   │
│    • filter_text: "", critical_only: false                  │
│    • column_widths: { Name: 300, Duration: 80, ... }       │
│    • visible_columns: [Name, Duration, Start, ...]          │
│  GanttView:                                                 │
│    • pixels_per_day: 14.0                                   │
│    • split_ratio: 0.3                                       │
│    • show_critical_path: true                               │
│    • time_scale: Day/Week/Month                             │
│  ResourcesView:                                             │
│    • histogram_scale: 1.0                                   │
│    • show_overallocated_only: false                         │
│    • group_by: ResourceKind / None                          │
└─────────────────────────────────────────────────────────────┘
                              │ fallback
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ TOOL (ephemeral, in-memory only)                            │
│  InspectorPanel:                                            │
│    • width: 320.0                                           │
│    • active_tab: Details / Resources / Constraints / Progress │
│  GanttBarDrag:                                              │
│    • snap_to_grid: true                                     │
│    • grid_size_days: 1                                      │
│  ResourceHistogram:                                         │
│    • zoom_level: 1.0                                        │
│    • date_range: (project_start, project_finish)            │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Data Structures

### 3.1 Core Settings Types (`crates/core/src/settings.rs` — new module)

```rust
//! Hierarchical settings for Cadence.
//!
//! Four levels: Global → Project → View → Tool.
//! Each level defines its own struct with serde support.
//! Fallback chain: Tool → View → Project → Global → Built-in defaults.

use std::collections::BTreeMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use time::Date;
use cadence_core::model::{ActivityId, BaselineId, CalendarId, Constraint, LinkType, ResourceId, ResourceKind, WbsId};

/// Density level for UI scaling (H1: 3 levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DensityLevel {
    /// Compact: minimal padding, small text, maximum content density.
    Compact,
    /// Standard: balanced padding and readability (default).
    #[default]
    Standard,
    /// Comfortable: generous padding, larger text, relaxed density.
    Comfortable,
}

impl DensityLevel {
    /// Scale factor applied to base spacing/token values.
    pub fn scale_factor(self) -> f32 {
        match self {
            Self::Compact => 0.85,
            Self::Standard => 1.0,
            Self::Comfortable => 1.15,
        }
    }

    /// All density levels in order.
    pub const ALL: [Self; 3] = [Self::Compact, Self::Standard, Self::Comfortable];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Standard => "Standard",
            Self::Comfortable => "Comfortable",
        }
    }

    /// Keyboard shortcut hint.
    pub fn shortcut_hint(self) -> &'static str {
        match self {
            Self::Compact => "Ctrl+Shift+Minus",
            Self::Standard => "Ctrl+Shift+0",
            Self::Comfortable => "Ctrl+Shift+Plus",
        }
    }
}

/// Theme mode (H6: dark/light/system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Follow system preference.
    #[default]
    System,
    /// Force light theme.
    Light,
    /// Force dark theme.
    Dark,
}

/// Time scale for Gantt view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeScale {
    /// Day-level granularity.
    Day,
    /// Week-level granularity.
    #[default]
    Week,
    /// Month-level granularity.
    Month,
    /// Quarter-level granularity.
    Quarter,
}

/// Grouping mode for Resources view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceGroupBy {
    /// No grouping, flat list.
    #[default]
    None,
    /// Group by resource kind (Labour/Equipment/Material).
    Kind,
    /// Group by calendar.
    Calendar,
}

/// Sort key for activity table (re-exported from UI for settings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    #[default]
    Name,
    Duration,
    Start,
    Finish,
    Float,
    Critical,
}

/// Column visibility entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnVisibility {
    /// Column key (matches SortKey).
    pub key: SortKey,
    /// Whether the column is visible.
    pub visible: bool,
    /// Column width in pixels (0 = auto/flex).
    pub width: f32,
}

/// Keyboard shortcut mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutMap {
    /// Action name -> key combination (e.g., "new_activity" -> "Ctrl+N").
    #[serde(default)]
    pub bindings: BTreeMap<String, String>,
}

impl Default for ShortcutMap {
    fn default() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert("new_activity".into(), "Ctrl+N".into());
        bindings.insert("new_relationship".into(), "Ctrl+L".into());
        bindings.insert("new_resource".into(), "Ctrl+R".into());
        bindings.insert("save".into(), "Ctrl+S".into());
        bindings.insert("undo".into(), "Ctrl+Z".into());
        bindings.insert("redo".into(), "Ctrl+Shift+Z".into());
        bindings.insert("find".into(), "Ctrl+F".into());
        bindings.insert("toggle_table".into(), "Ctrl+1".into());
        bindings.insert("toggle_gantt".into(), "Ctrl+2".into());
        bindings.insert("toggle_resources".into(), "Ctrl+3".into());
        bindings.insert("toggle_baselines".into(), "Ctrl+4".into());
        bindings.insert("toggle_calendars".into(), "Ctrl+5".into());
        bindings.insert("toggle_audit".into(), "Ctrl+6".into());
        bindings.insert("toggle_lookahead".into(), "Ctrl+7".into());
        bindings.insert("toggle_diary".into(), "Ctrl+8".into());
        bindings.insert("toggle_production".into(), "Ctrl+9".into());
        bindings.insert("density_compact".into(), "Ctrl+Shift+Minus".into());
        bindings.insert("density_standard".into(), "Ctrl+Shift+0".into());
        bindings.insert("density_comfortable".into(), "Ctrl+Shift+Plus".into());
        bindings.insert("toggle_theme".into(), "Ctrl+Shift+T".into());
        Self { bindings }
    }
}

/// Assistant AI configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantConfig {
    /// Model identifier (e.g., "gpt-4", "claude-3-sonnet").
    pub model: String,
    /// Base URL for API (empty = default provider).
    pub base_url: String,
    /// API key (stored in OS keyring, not in settings file).
    #[serde(skip)]
    pub api_key: Option<String>,
    /// Temperature for generation.
    pub temperature: f32,
    /// Maximum tokens per response.
    pub max_tokens: u32,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-mini".into(),
            base_url: String::new(),
            api_key: None,
            temperature: 0.2,
            max_tokens: 2000,
        }
    }
}

// ========================================================================
// LEVEL 1: GLOBAL SETTINGS (user config directory)
// ========================================================================

/// Schema version for global settings (increment on breaking changes).
pub const GLOBAL_SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Global application settings — persisted to user config directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Schema version for migration.
    #[serde(default = "default_global_schema_version")]
    pub schema_version: u32,
    /// UI theme.
    #[serde(default)]
    pub theme: ThemeMode,
    /// Toolbar/content density.
    #[serde(default)]
    pub density: DensityLevel,
    /// Keyboard shortcuts.
    #[serde(default)]
    pub shortcuts: ShortcutMap,
    /// Update check channel.
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
    /// Assistant AI configuration.
    #[serde(default)]
    pub assistant: AssistantConfig,
    /// Recently opened projects (max 10, most recent first).
    #[serde(default)]
    pub recent_projects: Vec<PathBuf>,
    /// Whether to show the splash screen on startup.
    #[serde(default = "default_true")]
    pub show_splash: bool,
    /// Whether to restore the last session (open last project).
    #[serde(default = "default_true")]
    pub restore_session: bool,
    /// Auto-save interval in seconds (0 = disabled).
    #[serde(default = "default_autosave_interval")]
    pub autosave_interval_secs: u32,
    /// Language/locale (ISO 639-1, empty = system).
    #[serde(default)]
    pub locale: String,
}

fn default_global_schema_version() -> u32 { GLOBAL_SETTINGS_SCHEMA_VERSION }
fn default_update_channel() -> String { "stable".into() }
fn default_true() -> bool { true }
fn default_autosave_interval() -> u32 { 300 } // 5 minutes

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            schema_version: GLOBAL_SETTINGS_SCHEMA_VERSION,
            theme: ThemeMode::default(),
            density: DensityLevel::default(),
            shortcuts: ShortcutMap::default(),
            update_channel: default_update_channel(),
            assistant: AssistantConfig::default(),
            recent_projects: Vec::new(),
            show_splash: true,
            restore_session: true,
            autosave_interval_secs: 300,
            locale: String::new(),
        }
    }
}

// ========================================================================
// LEVEL 2: PROJECT SETTINGS (embedded in .cadence SQLite)
// ========================================================================

/// Schema version for project settings (stored in project DB user_version or separate table).
pub const PROJECT_SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Project-level settings — persisted inside the project SQLite file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Schema version for migration.
    #[serde(default = "default_project_schema_version")]
    pub schema_version: u32,
    /// Default calendar for new activities.
    #[serde(default)]
    pub default_calendar: Option<CalendarId>,
    /// Project currency (ISO 4217).
    #[serde(default = "default_currency")]
    pub currency: String,
    /// Default cost rate for new resources (minor units per whole unit per day).
    #[serde(default)]
    pub default_cost_rate: i64,
    /// Overtime multiplier for cost calculations.
    #[serde(default = "default_overtime_multiplier")]
    pub overtime_multiplier: f32,
    /// Visible columns in table view (project-wide default).
    #[serde(default = "default_column_visibility")]
    pub visible_columns: Vec<ColumnVisibility>,
    /// Saved baseline IDs for earned value.
    #[serde(default)]
    pub baseline_ids: Vec<BaselineId>,
    /// Default lookahead window settings.
    #[serde(default)]
    pub lookahead_defaults: LookaheadDefaults,
    /// Project-specific keyboard shortcuts (overlay on global).
    #[serde(default)]
    pub shortcut_overrides: ShortcutMap,
    /// Whether to show WBS column by default.
    #[serde(default = "default_true")]
    pub show_wbs_column: bool,
    /// Default constraint for new activities.
    #[serde(default)]
    pub default_constraint: Constraint,
}

fn default_project_schema_version() -> u32 { PROJECT_SETTINGS_SCHEMA_VERSION }
fn default_currency() -> String { "USD".into() }
fn default_overtime_multiplier() -> f32 { 1.5 }
fn default_column_visibility() -> Vec<ColumnVisibility> {
    vec![
        ColumnVisibility { key: SortKey::Name, visible: true, width: 0.0 },
        ColumnVisibility { key: SortKey::Duration, visible: true, width: 80.0 },
        ColumnVisibility { key: SortKey::Start, visible: true, width: 100.0 },
        ColumnVisibility { key: SortKey::Finish, visible: true, width: 100.0 },
        ColumnVisibility { key: SortKey::Float, visible: true, width: 70.0 },
        ColumnVisibility { key: SortKey::Critical, visible: true, width: 70.0 },
    ]
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            schema_version: PROJECT_SETTINGS_SCHEMA_VERSION,
            default_calendar: None,
            currency: default_currency(),
            default_cost_rate: 0,
            overtime_multiplier: 1.5,
            visible_columns: default_column_visibility(),
            baseline_ids: Vec::new(),
            lookahead_defaults: LookaheadDefaults::default(),
            shortcut_overrides: ShortcutMap::default(),
            show_wbs_column: true,
            default_constraint: Constraint::AsSoonAsPossible,
        }
    }
}

/// Lookahead window defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookaheadDefaults {
    /// Window length in weeks.
    #[serde(default = "default_lookahead_weeks")]
    pub weeks: u32,
    /// Auto-refresh on project changes.
    #[serde(default = "default_true")]
    pub auto_refresh: bool,
    /// Show promised activities only.
    #[serde(default)]
    pub promised_only: bool,
}

fn default_lookahead_weeks() -> u32 { 2 }

impl Default for LookaheadDefaults {
    fn default() -> Self {
        Self {
            weeks: 2,
            auto_refresh: true,
            promised_only: false,
        }
    }
}

// ========================================================================
// LEVEL 3: VIEW SETTINGS (per-view, per-project, in-memory + optional persist)
// ========================================================================

/// Schema version for view settings.
pub const VIEW_SETTINGS_SCHEMA_VERSION: u32 = 1;

/// View-specific settings — primarily in-memory, optionally persisted to project DB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewSettings {
    /// Schema version for migration.
    #[serde(default = "default_view_schema_version")]
    pub schema_version: u32,
    /// Table view settings.
    #[serde(default)]
    pub table: TableViewSettings,
    /// Gantt view settings.
    #[serde(default)]
    pub gantt: GanttViewSettings,
    /// Resources view settings.
    #[serde(default)]
    pub resources: ResourcesViewSettings,
    /// Baselines view settings.
    #[serde(default)]
    pub baselines: BaselinesViewSettings,
    /// Calendars view settings.
    #[serde(default)]
    pub calendars: CalendarsViewSettings,
    /// Lookahead view settings.
    #[serde(default)]
    pub lookahead: LookaheadViewSettings,
    /// Diary view settings.
    #[serde(default)]
    pub diary: DiaryViewSettings,
    /// Production view settings.
    #[serde(default)]
    pub production: ProductionViewSettings,
    /// Audit view settings.
    #[serde(default)]
    pub audit: AuditViewSettings,
    /// Assistant view settings.
    #[serde(default)]
    pub assistant: AssistantViewSettings,
}

fn default_view_schema_version() -> u32 { VIEW_SETTINGS_SCHEMA_VERSION }

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            schema_version: VIEW_SETTINGS_SCHEMA_VERSION,
            table: TableViewSettings::default(),
            gantt: GanttViewSettings::default(),
            resources: ResourcesViewSettings::default(),
            baselines: BaselinesViewSettings::default(),
            calendars: CalendarsViewSettings::default(),
            lookahead: LookaheadViewSettings::default(),
            diary: DiaryViewSettings::default(),
            production: ProductionViewSettings::default(),
            audit: AuditViewSettings::default(),
            assistant: AssistantViewSettings::default(),
        }
    }
}

/// Table view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableViewSettings {
    /// Current sort key.
    #[serde(default)]
    pub sort_key: SortKey,
    /// Sort ascending.
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    /// Filter text (case-insensitive substring on name).
    #[serde(default)]
    pub filter_text: String,
    /// Show only critical activities.
    #[serde(default)]
    pub critical_only: bool,
    /// Column widths (key -> pixels, 0 = auto).
    #[serde(default)]
    pub column_widths: BTreeMap<SortKey, f32>,
    /// Column visibility (overrides project defaults).
    #[serde(default)]
    pub column_visibility: Vec<ColumnVisibility>,
    /// Whether WBS outline is shown.
    #[serde(default = "default_true")]
    pub show_wbs_outline: bool,
    /// Collapsed WBS node IDs.
    #[serde(default)]
    pub collapsed_wbs: Vec<WbsId>,
}

impl Default for TableViewSettings {
    fn default() -> Self {
        Self {
            sort_key: SortKey::Name,
            sort_ascending: true,
            filter_text: String::new(),
            critical_only: false,
            column_widths: BTreeMap::new(),
            column_visibility: Vec::new(), // empty = use project defaults
            show_wbs_outline: true,
            collapsed_wbs: Vec::new(),
        }
    }
}

/// Gantt view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GanttViewSettings {
    /// Horizontal zoom: pixels per working day.
    #[serde(default = "default_pixels_per_day")]
    pub pixels_per_day: f32,
    /// Table/Gantt split ratio (0.0–1.0, table fraction).
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f32,
    /// Show critical path highlight.
    #[serde(default = "default_true")]
    pub show_critical_path: bool,
    /// Show slack/float bars.
    #[serde(default = "default_true")]
    pub show_slack: bool,
    /// Time scale (Day/Week/Month/Quarter).
    #[serde(default)]
    pub time_scale: TimeScale,
    /// Show today line.
    #[serde(default = "default_true")]
    pub show_today_line: bool,
    /// Show baseline bars.
    #[serde(default)]
    pub show_baselines: bool,
    /// Selected baseline ID for comparison.
    #[serde(default)]
    pub selected_baseline: Option<BaselineId>,
    /// Row height in pixels.
    #[serde(default = "default_gantt_row_height")]
    pub row_height: f32,
    /// Show dependency links.
    #[serde(default = "default_true")]
    pub show_links: bool,
    /// Link style: straight / curved / orthogonal.
    #[serde(default = "default_link_style")]
    pub link_style: LinkStyle,
}

fn default_pixels_per_day() -> f32 { 14.0 }
fn default_split_ratio() -> f32 { 0.3 }
fn default_gantt_row_height() -> f32 { 28.0 }
fn default_link_style() -> LinkStyle { LinkStyle::Orthogonal }

impl Default for GanttViewSettings {
    fn default() -> Self {
        Self {
            pixels_per_day: 14.0,
            split_ratio: 0.3,
            show_critical_path: true,
            show_slack: true,
            time_scale: TimeScale::Week,
            show_today_line: true,
            show_baselines: false,
            selected_baseline: None,
            row_height: 28.0,
            show_links: true,
            link_style: LinkStyle::Orthogonal,
        }
    }
}

/// Dependency link visual style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LinkStyle {
    /// Straight line.
    Straight,
    /// Curved bezier.
    Curved,
    /// Orthogonal (L-shaped).
    #[default]
    Orthogonal,
}

/// Resources view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesViewSettings {
    /// Histogram vertical scale factor.
    #[serde(default = "default_histogram_scale")]
    pub histogram_scale: f32,
    /// Show only overallocated resources.
    #[serde(default)]
    pub overallocated_only: bool,
    /// Grouping mode.
    #[serde(default)]
    pub group_by: ResourceGroupBy,
    /// Show cost histogram (vs hours).
    #[serde(default)]
    pub show_cost: bool,
    /// Date range for histogram (None = project span).
    #[serde(default)]
    pub date_range: Option<(Date, Date)>,
    /// Row height.
    #[serde(default = "default_resource_row_height")]
    pub row_height: f32,
}

fn default_histogram_scale() -> f32 { 1.0 }
fn default_resource_row_height() -> f32 { 28.0 }

impl Default for ResourcesViewSettings {
    fn default() -> Self {
        Self {
            histogram_scale: 1.0,
            overallocated_only: false,
            group_by: ResourceGroupBy::None,
            show_cost: false,
            date_range: None,
            row_height: 28.0,
        }
    }
}

/// Baselines view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselinesViewSettings {
    /// Show variance columns.
    #[serde(default = "default_true")]
    pub show_variance: bool,
    /// Show cost variance.
    #[serde(default = "default_true")]
    pub show_cost_variance: bool,
    /// Baseline to compare against (None = latest).
    #[serde(default)]
    pub compare_baseline: Option<BaselineId>,
}

impl Default for BaselinesViewSettings {
    fn default() -> Self {
        Self {
            show_variance: true,
            show_cost_variance: true,
            compare_baseline: None,
        }
    }
}

/// Calendars view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarsViewSettings {
    /// Show holiday exceptions.
    #[serde(default = "default_true")]
    pub show_holidays: bool,
    /// Selected calendar for editing.
    #[serde(default)]
    pub selected_calendar: Option<CalendarId>,
    /// Weekday pattern editor: which weekdays are working.
    #[serde(default = "default_weekdays")]
    pub editing_weekdays: [bool; 7],
}

fn default_weekdays() -> [bool; 7] { [true, true, true, true, true, false, false] }

impl Default for CalendarsViewSettings {
    fn default() -> Self {
        Self {
            show_holidays: true,
            selected_calendar: None,
            editing_weekdays: default_weekdays(),
        }
    }
}

/// Lookahead view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookaheadViewSettings {
    /// Window start date.
    #[serde(default)]
    pub window_start: Option<Date>,
    /// Window length in weeks.
    #[serde(default = "default_lookahead_weeks")]
    pub weeks: u32,
    /// Show promised activities only.
    #[serde(default)]
    pub promised_only: bool,
    /// Selected snapshot for comparison.
    #[serde(default)]
    pub selected_snapshot: Option<cadence_core::model::LookaheadSnapshotId>,
}

impl Default for LookaheadViewSettings {
    fn default() -> Self {
        Self {
            window_start: None,
            weeks: 2,
            promised_only: false,
            selected_snapshot: None,
        }
    }
}

/// Diary view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiaryViewSettings {
    /// Filter by activity.
    #[serde(default)]
    pub filter_activity: Option<ActivityId>,
    /// Filter by lookahead window start.
    #[serde(default)]
    pub filter_window: Option<Date>,
    /// Sort by date descending (newest first).
    #[serde(default = "default_true")]
    pub sort_descending: bool,
}

impl Default for DiaryViewSettings {
    fn default() -> Self {
        Self {
            filter_activity: None,
            filter_window: None,
            sort_descending: true,
        }
    }
}

/// Production view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionViewSettings {
    /// Filter by activity.
    #[serde(default)]
    pub filter_activity: Option<ActivityId>,
    /// Filter by date range.
    #[serde(default)]
    pub filter_date_range: Option<(Date, Date)>,
    /// Show variance from plan.
    #[serde(default = "default_true")]
    pub show_variance: bool,
}

impl Default for ProductionViewSettings {
    fn default() -> Self {
        Self {
            filter_activity: None,
            filter_date_range: None,
            show_variance: true,
        }
    }
}

/// Audit view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditViewSettings {
    /// Status date for audit (None = today).
    #[serde(default)]
    pub status_date: Option<Date>,
    /// Show only findings with severity >= this.
    #[serde(default = "default_audit_severity")]
    pub min_severity: AuditSeverity,
    /// Group findings by category.
    #[serde(default = "default_true")]
    pub group_by_category: bool,
}

fn default_audit_severity() -> AuditSeverity { AuditSeverity::Info }

impl Default for AuditViewSettings {
    fn default() -> Self {
        Self {
            status_date: None,
            min_severity: AuditSeverity::Info,
            group_by_category: true,
        }
    }
}

/// Audit finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

/// Assistant view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantViewSettings {
    /// Last used model (overrides global).
    #[serde(default)]
    pub last_model: Option<String>,
    /// Conversation history (truncated to last N).
    #[serde(default)]
    pub history: Vec<AssistantMessage>,
    /// Max history length.
    #[serde(default = "default_assistant_history_len")]
    pub max_history: usize,
}

fn default_assistant_history_len() -> usize { 50 }

impl Default for AssistantViewSettings {
    fn default() -> Self {
        Self {
            last_model: None,
            history: Vec::new(),
            max_history: 50,
        }
    }
}

/// Assistant conversation message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantMessage {
    /// Role: "user" or "assistant".
    pub role: String,
    /// Message content.
    pub content: String,
    /// Timestamp.
    pub timestamp: String, // ISO 8601
}

// ========================================================================
// LEVEL 4: TOOL SETTINGS (ephemeral, in-memory only)
// ========================================================================

/// Tool-specific settings — never persisted, reset on view change.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ToolSettings {
    /// Inspector panel settings.
    pub inspector: InspectorToolSettings,
    /// Gantt bar drag settings.
    pub gantt_drag: GanttDragToolSettings,
    /// Resource histogram settings.
    pub histogram: HistogramToolSettings,
    /// Column resize settings.
    pub column_resize: ColumnResizeToolSettings,
    /// Split pane settings.
    pub split_pane: SplitPaneToolSettings,
}

/// Inspector panel tool settings.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorToolSettings {
    /// Panel width in pixels.
    pub width: f32,
    /// Active tab index.
    pub active_tab: usize,
    /// Whether panel is pinned (stays open).
    pub pinned: bool,
}

impl Default for InspectorToolSettings {
    fn default() -> Self {
        Self { width: 320.0, active_tab: 0, pinned: true }
    }
}

/// Gantt bar drag tool settings.
#[derive(Debug, Clone, PartialEq)]
pub struct GanttDragToolSettings {
    /// Snap to grid during drag.
    pub snap_to_grid: bool,
    /// Grid size in working days.
    pub grid_size_days: u32,
    /// Show ghost bar during drag.
    pub show_ghost: bool,
}

impl Default for GanttDragToolSettings {
    fn default() -> Self {
        Self { snap_to_grid: true, grid_size_days: 1, show_ghost: true }
    }
}

/// Resource histogram tool settings.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramToolSettings {
    /// Current zoom level.
    pub zoom: f32,
    /// Horizontal scroll offset (days from project start).
    pub scroll_offset_days: i64,
    /// Tooltip hover activity.
    pub hover_activity: Option<ActivityId>,
    /// Tooltip hover date.
    pub hover_date: Option<Date>,
}

impl Default for HistogramToolSettings {
    fn default() -> Self {
        Self { zoom: 1.0, scroll_offset_days: 0, hover_activity: None, hover_date: None }
    }
}

/// Column resize tool settings.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnResizeToolSettings {
    /// Column being resized.
    pub resizing_column: Option<SortKey>,
    /// Drag start X position.
    pub drag_start_x: f32,
    /// Column width at drag start.
    pub start_width: f32,
}

impl Default for ColumnResizeToolSettings {
    fn default() -> Self {
        Self { resizing_column: None, drag_start_x: 0.0, start_width: 0.0 }
    }
}

/// Split pane tool settings.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitPaneToolSettings {
    /// Which split is being dragged (table/gantt, gantt/detail, etc.).
    pub dragging_split: Option<SplitId>,
    /// Drag start position.
    pub drag_start: f32,
    /// Ratio at drag start.
    pub start_ratio: f32,
}

impl Default for SplitPaneToolSettings {
    fn default() -> Self {
        Self { dragging_split: None, drag_start: 0.0, start_ratio: 0.0 }
    }
}

/// Split pane identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitId {
    TableGantt,
    GanttDetail,
    ResourcesHistogram,
    LookaheadDetail,
}

// ========================================================================
// SETTINGS CONTAINER: Aggregates all four levels with fallback logic
// ========================================================================

/// Complete settings context for a project session.
#[derive(Debug, Clone)]
pub struct SettingsContext {
    /// Global settings (user config dir).
    pub global: GlobalSettings,
    /// Project settings (from project DB).
    pub project: ProjectSettings,
    /// View settings (in-memory, optionally synced to project DB).
    pub view: ViewSettings,
    /// Tool settings (ephemeral).
    pub tool: ToolSettings,
    /// Density level resolved from hierarchy (Global → Project override? → View override?).
    pub resolved_density: DensityLevel,
    /// Theme resolved from hierarchy.
    pub resolved_theme: ThemeMode,
}

impl SettingsContext {
    /// Create a new settings context with defaults.
    pub fn new(global: GlobalSettings, project: ProjectSettings) -> Self {
        let resolved_density = global.density; // Project/View don't override density yet
        let resolved_theme = global.theme;
        Self {
            global,
            project,
            view: ViewSettings::default(),
            tool: ToolSettings::default(),
            resolved_density,
            resolved_theme,
        }
    }

    /// Resolve the effective density (Global only for now; extend if Project/View override).
    pub fn effective_density(&self) -> DensityLevel {
        self.resolved_density
    }

    /// Resolve the effective theme (Global only for now).
    pub fn effective_theme(&self) -> ThemeMode {
        self.resolved_theme
    }

    /// Get the scale factor for the effective density.
    pub fn density_scale(&self) -> f32 {
        self.effective_density().scale_factor()
    }

    /// Get table column visibility, falling back to project defaults.
    pub fn table_column_visibility(&self) -> Vec<ColumnVisibility> {
        if self.view.table.column_visibility.is_empty() {
            self.project.visible_columns.clone()
        } else {
            // Merge: project defaults as base, view overrides on top
            let mut merged = self.project.visible_columns.clone();
            for view_col in &self.view.table.column_visibility {
                if let Some(proj_col) = merged.iter_mut().find(|c| c.key == view_col.key) {
                    proj_col.visible = view_col.visible;
                    if view_col.width > 0.0 {
                        proj_col.width = view_col.width;
                    }
                } else {
                    merged.push(view_col.clone());
                }
            }
            merged
        }
    }

    /// Get table column width for a key, with fallback chain.
    pub fn table_column_width(&self, key: SortKey) -> f32 {
        // 1. View explicit width
        if let Some(&width) = self.view.table.column_widths.get(&key) {
            if width > 0.0 { return width; }
        }
        // 2. View column visibility width
        if let Some(col) = self.view.table.column_visibility.iter().find(|c| c.key == key) {
            if col.width > 0.0 { return col.width; }
        }
        // 3. Project column visibility width
        if let Some(col) = self.project.visible_columns.iter().find(|c| c.key == key) {
            if col.width > 0.0 { return col.width; }
        }
        // 4. Built-in default
        match key {
            SortKey::Name => 0.0,       // flex
            SortKey::Duration => 80.0,
            SortKey::Start => 100.0,
            SortKey::Finish => 100.0,
            SortKey::Float => 70.0,
            SortKey::Critical => 70.0,
        }
    }
}
```

---

## 4. Persistence Strategy

### 4.1 Per-Level Persistence

| Level | Storage | Format | Location | Migration |
|-------|---------|--------|----------|-----------|
| **Global** | File | JSON (pretty) | `$CONFIG_DIR/cadence/settings.json` | Schema version in file; migrate on load |
| **Project** | SQLite | Table `project_settings` (JSON blob) | Inside `.cadence` project file | Schema version in blob; migrate on load |
| **View** | Optional SQLite | Table `view_settings` (JSON blob per view) | Inside `.cadence` project file | Schema version in blob; migrate on load |
| **Tool** | None | In-memory only | — | N/A |

### 4.2 Global Settings File

**Path resolution:**
- Linux: `~/.config/cadence/settings.json` (or `$XDG_CONFIG_HOME/cadence/settings.json`)
- macOS: `~/Library/Application Support/Cadence/settings.json`
- Windows: `%APPDATA%\Cadence\settings.json`

**File format:**
```json
{
  "schema_version": 1,
  "theme": "system",
  "density": "standard",
  "shortcuts": { "bindings": { "new_activity": "Ctrl+N", ... } },
  "update_channel": "stable",
  "assistant": { "model": "gpt-4o-mini", "base_url": "", "temperature": 0.2, "max_tokens": 2000 },
  "recent_projects": ["/path/to/project1.cadence", "..."],
  "show_splash": true,
  "restore_session": true,
  "autosave_interval_secs": 300,
  "locale": ""
}
```

### 4.3 Project Settings in SQLite

**New table:**
```sql
CREATE TABLE IF NOT EXISTS project_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL DEFAULT 1,
    data TEXT NOT NULL  -- JSON blob of ProjectSettings
);
```

**Migration:** On `load_project`, after `ensure_schema`, read `project_settings`. If missing or `schema_version < PROJECT_SETTINGS_SCHEMA_VERSION`, migrate and write back.

### 4.4 View Settings in SQLite (Optional)

**New table:**
```sql
CREATE TABLE IF NOT EXISTS view_settings (
    view_name TEXT PRIMARY KEY,  -- "table", "gantt", "resources", etc.
    schema_version INTEGER NOT NULL DEFAULT 1,
    data TEXT NOT NULL  -- JSON blob of the specific view settings struct
);
```

**Rationale:** View settings are primarily in-memory for performance. Persistence is opt-in:
- Save on project close / explicit "Save View Layout" action
- Load on project open
- If table missing or empty, use defaults

### 4.5 Migration Functions

```rust
// In crates/core/src/settings.rs

/// Migrate GlobalSettings from an older schema version.
pub fn migrate_global_settings(mut settings: GlobalSettings, from_version: u32) -> GlobalSettings {
    // Version 1 is current; add cases here when schema changes.
    match from_version {
        0 => { /* initial migration if needed */ }
        _ => {}
    }
    settings.schema_version = GLOBAL_SETTINGS_SCHEMA_VERSION;
    settings
}

/// Migrate ProjectSettings from an older schema version.
pub fn migrate_project_settings(mut settings: ProjectSettings, from_version: u32) -> ProjectSettings {
    match from_version {
        0 => { /* add default_cost_rate, overtime_multiplier, etc. */ }
        _ => {}
    }
    settings.schema_version = PROJECT_SETTINGS_SCHEMA_VERSION;
    settings
}

/// Migrate ViewSettings from an older schema version.
pub fn migrate_view_settings(mut settings: ViewSettings, from_version: u32) -> ViewSettings {
    match from_version {
        0 => { /* add new view settings fields */ }
        _ => {}
    }
    settings.schema_version = VIEW_SETTINGS_SCHEMA_VERSION;
    settings
}
```

---

## 5. Change Notification & Sync Mechanism

### 5.1 Design Principles (Addressing T5: Circular Updates)

1. **Single Source of Truth per Level** — Each level owns its data; no duplicate state.
2. **Unidirectional Fallback** — Resolution flows Tool → View → Project → Global → Built-in. Never reverse.
3. **Explicit Sync Points** — Changes propagate only at well-defined boundaries:
   - Global changes → notify all open projects (rare)
   - Project changes → notify affected views (on next frame)
   - View changes → local only (no auto-propagate up)
   - Tool changes → never persist, never propagate
4. **Immutable Snapshots for UI** — UI reads from `SettingsContext` snapshots; mutations go through commands.

### 5.2 Event System

```rust
// In crates/core/src/settings.rs

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Settings change event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsEvent {
    /// Global settings changed (affects all projects).
    GlobalChanged { key: String },
    /// Project settings changed (affects current project views).
    ProjectChanged { key: String },
    /// View settings changed (affects single view).
    ViewChanged { view: ViewName, key: String },
    /// Tool settings changed (ephemeral, UI-only).
    ToolChanged { tool: ToolName, key: String },
    /// Density changed (triggers token rescale).
    DensityChanged { level: DensityLevel },
    /// Theme changed (triggers full repaint).
    ThemeChanged { mode: ThemeMode },
}

/// View name for events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewName {
    Table, Gantt, Resources, Baselines, Calendars,
    Lookahead, Diary, Production, Audit, Assistant,
}

/// Tool name for events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    Inspector, GanttDrag, Histogram, ColumnResize, SplitPane,
}

/// Simple event bus for settings changes.
#[derive(Debug, Default)]
pub struct SettingsEventBus {
    subscribers: Mutex<Vec<Arc<dyn Fn(SettingsEvent) + Send + Sync>>>,
}

impl SettingsEventBus {
    /// Subscribe to settings events.
    pub fn subscribe(&self, callback: Arc<dyn Fn(SettingsEvent) + Send + Sync>) {
        self.subscribers.lock().unwrap().push(callback);
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: SettingsEvent) {
        for cb in self.subscribers.lock().unwrap().iter() {
            cb(event.clone());
        }
    }
}

/// Global event bus instance (lazy static or passed through App).
static SETTINGS_EVENT_BUS: once_cell::sync::Lazy<SettingsEventBus> =
    once_cell::sync::Lazy::new(SettingsEventBus::default);

/// Emit a settings event (convenience).
pub fn emit_settings_event(event: SettingsEvent) {
    SETTINGS_EVENT_BUS.emit(event);
}
```

### 5.3 UI Integration (egui)

```rust
// In crates/ui-egui/src/lib.rs — add to CadenceApp

impl CadenceApp {
    /// Handle settings events from the event bus.
    fn handle_settings_events(&mut self) {
        // Drain events from a channel or poll the bus
        // For egui, we typically handle this in the frame loop
        // by checking a receiver or polling a flag.
        
        // Example: if density changed, rescale tokens
        // if theme changed, reapply theme
    }

    /// Apply density scale to egui context.
    fn apply_density(&self, ctx: &egui::Context) {
        let scale = self.settings.density_scale();
        ctx.set_pixels_per_point(ctx.pixels_per_point() * scale);
        // Also adjust spacing tokens if using design tokens
    }

    /// Apply theme to egui context.
    fn apply_theme(&self, ctx: &egui::Context) {
        design::apply(ctx, matches!(self.settings.effective_theme(), ThemeMode::Dark));
    }
}
```

### 5.4 Sync Rules Summary

| Change Origin | Propagates To | Mechanism |
|---------------|---------------|-----------|
| Global (theme/density) | All open projects, all views | `SettingsEvent::GlobalChanged` → UI re-reads `SettingsContext` |
| Global (shortcuts) | All projects | `SettingsEvent::GlobalChanged` → command layer rebinds |
| Project (columns, calendar) | Current project views | `SettingsEvent::ProjectChanged` → view recomputes on next frame |
| View (sort, filter, zoom) | Same view only | Local state update; no event emitted upward |
| Tool (panel width, drag) | Same tool only | Local state; never persists |

**No circular updates possible** because:
- View never writes to Project or Global
- Tool never writes to View, Project, or Global
- Project never writes to Global
- Global writes down only via explicit "Apply to Project" user action (future)

---

## 6. Schema Versioning & Migration

### 6.1 Versioning Scheme

Each level has independent schema version:
- **Global**: `GLOBAL_SETTINGS_SCHEMA_VERSION` (u32)
- **Project**: `PROJECT_SETTINGS_SCHEMA_VERSION` (u32)
- **View**: `VIEW_SETTINGS_SCHEMA_VERSION` (u32)

Stored in the persisted data (`schema_version` field).

### 6.2 Migration Strategy

1. **On Load**: Read persisted data, check `schema_version`.
2. **If Older**: Call appropriate `migrate_*_settings()` function.
3. **Write Back**: Save migrated data immediately (atomic for SQLite).
4. **No Downgrades**: Never support reading newer schema versions.

### 6.3 Migration Example: Adding a Field

```rust
// When adding `default_cost_rate` to ProjectSettings in v2:

pub fn migrate_project_settings(mut settings: ProjectSettings, from_version: u32) -> ProjectSettings {
    match from_version {
        1 => {
            // v1 didn't have default_cost_rate or overtime_multiplier
            settings.default_cost_rate = 0;
            settings.overtime_multiplier = 1.5;
            // visible_columns already existed in v1
        }
        0 => { /* initial */ }
        _ => {}
    }
    settings.schema_version = PROJECT_SETTINGS_SCHEMA_VERSION;
    settings
}
```

### 6.4 Testing Migrations

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_global_v0_to_v1() {
        let v0 = r#"{"theme": "dark", "density": "compact"}"#;
        let mut settings: GlobalSettings = serde_json::from_str(v0).unwrap();
        settings = migrate_global_settings(settings, 0);
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.theme, ThemeMode::Dark);
        assert_eq!(settings.density, DensityLevel::Compact);
        // New fields have defaults
        assert_eq!(settings.update_channel, "stable");
    }

    #[test]
    fn migrate_project_v1_adds_cost_fields() {
        let v1 = ProjectSettings {
            schema_version: 1,
            default_calendar: None,
            currency: "EUR".into(),
            default_cost_rate: 0, // will be set by migration
            overtime_multiplier: 1.5, // will be set
            ..Default::default()
        };
        let migrated = migrate_project_settings(v1, 1);
        assert_eq!(migrated.schema_version, PROJECT_SETTINGS_SCHEMA_VERSION);
        assert_eq!(migrated.default_cost_rate, 0);
        assert_eq!(migrated.overtime_multiplier, 1.5);
    }
}
```

---

## 7. Default Values & Fallback Chain

### 7.1 Fallback Resolution

```rust
// In SettingsContext

/// Resolve a setting with the full fallback chain.
/// Returns the first non-None value from: Tool → View → Project → Global → Built-in.
fn resolve_setting<T>(&self, tool: Option<T>, view: Option<T>, project: Option<T>, global: Option<T>, builtin: T) -> T {
    tool.or(view).or(project).or(global).unwrap_or(builtin)
}
```

### 7.2 Concrete Fallback Examples

| Setting | Tool | View | Project | Global | Built-in Default |
|---------|------|------|---------|--------|------------------|
| **Density** | — | — | — | `GlobalSettings.density` | `Standard` |
| **Theme** | — | — | — | `GlobalSettings.theme` | `System` |
| **Table Sort** | — | `ViewSettings.table.sort_key` | — | — | `Name` |
| **Table Filter** | — | `ViewSettings.table.filter_text` | — | — | `""` |
| **Gantt Zoom** | `ToolSettings.gantt_drag.grid_size_days` | `ViewSettings.gantt.pixels_per_day` | — | — | `14.0` |
| **Column Width** | `ToolSettings.column_resize.start_width` | `ViewSettings.table.column_widths[key]` | `ProjectSettings.visible_columns[key].width` | — | Built-in per column |
| **Visible Columns** | — | `ViewSettings.table.column_visibility` | `ProjectSettings.visible_columns` | — | All 6 visible |
| **Calendar** | — | `ViewSettings.calendars.selected_calendar` | `ProjectSettings.default_calendar` | — | Project default calendar |
| **Shortcuts** | — | — | `ProjectSettings.shortcut_overrides` | `GlobalSettings.shortcuts` | Built-in defaults |

### 7.3 Effective Settings API

```rust
impl SettingsContext {
    /// Effective density (Global only for v1).
    pub fn effective_density(&self) -> DensityLevel { self.global.density }

    /// Effective theme (Global only for v1).
    pub fn effective_theme(&self) -> ThemeMode { self.global.theme }

    /// Effective shortcuts (Project overlays Global).
    pub fn effective_shortcuts(&self) -> ShortcutMap {
        let mut merged = self.global.shortcuts.clone();
        for (action, key) in &self.project.shortcut_overrides.bindings {
            merged.bindings.insert(action.clone(), key.clone());
        }
        merged
    }

    /// Effective table sort.
    pub fn effective_table_sort(&self) -> (SortKey, bool) {
        (self.view.table.sort_key, self.view.table.sort_ascending)
    }

    /// Effective table filter.
    pub fn effective_table_filter(&self) -> &str {
        &self.view.table.filter_text
    }

    /// Effective Gantt pixels per day.
    pub fn effective_pixels_per_day(&self) -> f32 {
        self.view.gantt.pixels_per_day * self.density_scale()
    }

    /// Effective Gantt row height.
    pub fn effective_gantt_row_height(&self) -> f32 {
        self.view.gantt.row_height * self.density_scale()
    }

    /// Effective resource histogram scale.
    pub fn effective_histogram_scale(&self) -> f32 {
        self.view.resources.histogram_scale * self.density_scale()
    }

    /// Effective column visibility for table.
    pub fn effective_table_columns(&self) -> Vec<ColumnVisibility> {
        self.table_column_visibility() // defined earlier
    }
}
```

---

## 8. API Surface for UI Consumption

### 8.1 Settings Access in UI (`crates/ui-egui/src/lib.rs`)

```rust
// Add to CadenceApp
pub struct CadenceApp {
    // ... existing fields ...
    /// Complete settings context.
    pub settings: SettingsContext,
    // ... existing fields ...
}

impl CadenceApp {
    /// Access effective density scale for UI.
    pub fn density_scale(&self) -> f32 {
        self.settings.density_scale()
    }

    /// Access effective theme.
    pub fn effective_theme(&self) -> ThemeMode {
        self.settings.effective_theme()
    }

    /// Get table column width for rendering.
    pub fn table_column_width(&self, key: SortKey) -> f32 {
        self.settings.table_column_width(key) * self.density_scale()
    }

    /// Get Gantt pixels per day.
    pub fn gantt_pixels_per_day(&self) -> f32 {
        self.settings.effective_pixels_per_day()
    }

    /// Get Gantt row height.
    pub fn gantt_row_height(&self) -> f32 {
        self.settings.effective_gantt_row_height()
    }

    /// Get resource histogram scale.
    pub fn histogram_scale(&self) -> f32 {
        self.settings.effective_histogram_scale()
    }

    /// Get table column visibility.
    pub fn table_columns(&self) -> Vec<ColumnVisibility> {
        self.settings.effective_table_columns()
    }
}
```

### 8.2 Settings Commands (Mutations via Command Layer)

```rust
// In crates/commands/src/lib.rs — new commands

use cadence_core::settings::{GlobalSettings, ProjectSettings, ViewSettings, DensityLevel, ThemeMode, SortKey};

/// Change global density level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetGlobalDensity {
    pub level: DensityLevel,
}

impl Command for SetGlobalDensity {
    fn description(&self) -> String {
        format!("set global density to {:?}", self.level)
    }
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        // Global settings not in Project; this would need a different mechanism
        // For now, emit event and update global settings file
        Err("Global settings changes not yet implemented via commands".into())
    }
}

/// Change global theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetGlobalTheme {
    pub mode: ThemeMode,
}

impl Command for SetGlobalTheme { /* similar */ }

/// Change project default currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetProjectCurrency {
    pub currency: String,
    previous: Option<String>,
}

impl Command for SetProjectCurrency {
    fn description(&self) -> String { format!("set project currency to {}", self.currency) }
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        // ProjectSettings not yet in Project model; would need to add
        // For now, return error
        Err("Project settings not yet in Project model".into())
    }
}

/// Change table sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTableSort {
    pub key: SortKey,
    pub ascending: bool,
    previous_key: SortKey,
    previous_ascending: bool,
}

impl Command for SetTableSort {
    fn description(&self) -> String {
        format!("set table sort to {:?} {}", self.key, if self.ascending { "asc" } else { "desc" })
    }
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        // View settings are in UI state, not Project
        // This command pattern works for Project-level data only
        // View/Tool settings use direct UI mutation + optional persist
        Err("View settings mutated directly in UI".into())
    }
}
```

**Note:** View and Tool level settings are **UI state**, not Project data. They are mutated directly in the UI layer and optionally persisted to the project DB on save. Only Global and Project settings go through the command layer for undo/redo.

### 8.3 Settings UI — Global Settings Dialog

```rust
// In crates/ui-egui/src/settings_ui.rs (new module)

use cadence_core::settings::{GlobalSettings, DensityLevel, ThemeMode, ShortcutMap, AssistantConfig};
use egui::{Ui, ComboBox, Grid, TextEdit};

/// Render the Global Settings dialog.
pub fn show_global_settings_dialog(app: &mut CadenceApp, ui: &mut Ui) {
    let settings = &mut app.settings.global;
    let original_density = settings.density;
    let original_theme = settings.theme;

    ui.heading("Global Settings");

    Grid::new("global_settings_grid").num_columns(2).show(ui, |ui| {
        // Theme
        ui.label("Theme:");
        ComboBox::from_id_salt("theme_combo")
            .selected_text(match settings.theme {
                ThemeMode::System => "System",
                ThemeMode::Light => "Light",
                ThemeMode::Dark => "Dark",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.theme, ThemeMode::System, "System");
                ui.selectable_value(&mut settings.theme, ThemeMode::Light, "Light");
                ui.selectable_value(&mut settings.theme, ThemeMode::Dark, "Dark");
            });
        ui.end_row();

        // Density
        ui.label("Density:");
        ComboBox::from_id_salt("density_combo")
            .selected_text(settings.density.label())
            .show_ui(ui, |ui| {
                for level in DensityLevel::ALL {
                    ui.selectable_value(&mut settings.density, level, level.label());
                }
            });
        ui.end_row();

        // Auto-save interval
        ui.label("Auto-save interval (seconds, 0 = off):");
        let mut autosave_str = settings.autosave_interval_secs.to_string();
        if ui.add(TextEdit::singleline(&mut autosave_str).desired_width(80.0)).changed() {
            if let Ok(val) = autosave_str.parse::<u32>() {
                settings.autosave_interval_secs = val;
            }
        }
        ui.end_row();

        // Update channel
        ui.label("Update channel:");
        ComboBox::from_id_salt("update_channel_combo")
            .selected_text(&settings.update_channel)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.update_channel, "stable".into(), "Stable");
                ui.selectable_value(&mut settings.update_channel, "beta".into(), "Beta");
            });
        ui.end_row();

        // Locale
        ui.label("Language (empty = system):");
        ui.add(TextEdit::singleline(&mut settings.locale).desired_width(120.0));
        ui.end_row();
    });

    ui.separator();
    ui.heading("Keyboard Shortcuts");
    // ... render shortcut editor ...

    ui.separator();
    ui.heading("AI Assistant");
    // ... render assistant config ...

    // Apply immediate changes for density/theme
    if settings.density != original_density {
        cadence_core::settings::emit_settings_event(
            cadence_core::settings::SettingsEvent::DensityChanged { level: settings.density }
        );
    }
    if settings.theme != original_theme {
        cadence_core::settings::emit_settings_event(
            cadence_core::settings::SettingsEvent::ThemeChanged { mode: settings.theme }
        );
    }
}
```

### 8.4 Settings UI — View Settings (Per-View)

```rust
// In crates/ui-egui/src/settings_ui.rs

/// Render view-specific settings (called from each view's toolbar/menu).
pub fn show_view_settings(app: &mut CadenceApp, ui: &mut Ui, view: ViewName) {
    match view {
        ViewName::Table => show_table_view_settings(app, ui),
        ViewName::Gantt => show_gantt_view_settings(app, ui),
        ViewName::Resources => show_resources_view_settings(app, ui),
        ViewName::Baselines => show_baselines_view_settings(app, ui),
        ViewName::Calendars => show_calendars_view_settings(app, ui),
        ViewName::Lookahead => show_lookahead_view_settings(app, ui),
        ViewName::Diary => show_diary_view_settings(app, ui),
        ViewName::Production => show_production_view_settings(app, ui),
        ViewName::Audit => show_audit_view_settings(app, ui),
        ViewName::Assistant => show_assistant_view_settings(app, ui),
    }
}

fn show_table_view_settings(app: &mut CadenceApp, ui: &mut Ui) {
    let view = &mut app.settings.view.table;
    
    ui.label("Table View Settings");
    
    Grid::new("table_view_settings").num_columns(2).show(ui, |ui| {
        ui.label("Default Sort:");
        ComboBox::from_id_salt("table_sort")
            .selected_text(format!("{:?}", view.sort_key))
            .show_ui(ui, |ui| {
                for key in SortKey::ordered_columns() {
                    ui.selectable_value(&mut view.sort_key, key, format!("{:?}", key));
                }
            });
        ui.end_row();

        ui.label("Sort Ascending:");
        ui.checkbox(&mut view.sort_ascending, "");
        ui.end_row();

        ui.label("Show Critical Only:");
        ui.checkbox(&mut view.critical_only, "");
        ui.end_row();

        ui.label("Show WBS Outline:");
        ui.checkbox(&mut view.show_wbs_outline, "");
        ui.end_row();
    });

    ui.separator();
    ui.label("Column Visibility & Widths:");
    // Render column editor with checkboxes and width inputs
    for col in &mut app.settings.effective_table_columns() {
        ui.horizontal(|ui| {
            let mut visible = col.visible;
            if ui.checkbox(&mut visible, &format!("{:?}", col.key)).changed() {
                // Update view override
                if let Some(vcol) = view.column_visibility.iter_mut().find(|c| c.key == col.key) {
                    vcol.visible = visible;
                } else {
                    view.column_visibility.push(ColumnVisibility { key: col.key, visible, width: col.width });
                }
            }
            if visible {
                ui.label("Width:");
                let mut width_str = if col.width > 0.0 { col.width.to_string() } else { "auto".into() };
                if ui.add(TextEdit::singleline(&mut width_str).desired_width(60.0)).changed() {
                    if let Ok(w) = width_str.parse::<f32>() {
                        if let Some(vcol) = view.column_visibility.iter_mut().find(|c| c.key == col.key) {
                            vcol.width = w;
                        } else {
                            view.column_visibility.push(ColumnVisibility { key: col.key, visible: true, width: w });
                        }
                    } else if width_str == "auto" {
                        if let Some(vcol) = view.column_visibility.iter_mut().find(|c| c.key == col.key) {
                            vcol.width = 0.0;
                        }
                    }
                }
            }
        });
    }
}
```

---

## 9. Conflicts with Existing Code

### 9.1 Identified Conflicts

| Existing Code | Conflict | Resolution |
|---------------|----------|------------|
| `CadenceApp::dark: bool` | Duplicates `GlobalSettings.theme` | Remove `dark`; derive from `settings.effective_theme()` |
| `CadenceApp::pixels_per_day: f32` | Duplicates `ViewSettings.gantt.pixels_per_day` | Remove; use `settings.effective_pixels_per_day()` |
| `CadenceApp::split_ratio` / `show_split_gantt` | Duplicates `ViewSettings.gantt.split_ratio` | Remove; use view settings |
| `CadenceApp::sort_key`, `sort_ascending` | Duplicates `ViewSettings.table.sort_key` | Remove; use view settings |
| `CadenceApp::table_filter_buffer`, `table_critical_only` | Duplicates `ViewSettings.table.filter_text`, `critical_only` | Remove; use view settings |
| `CadenceApp::visible_columns: BTreeSet<SortKey>` | Duplicates `ViewSettings.table.column_visibility` | Remove; use view settings with full `ColumnVisibility` |
| `CadenceApp::collapsed_wbs: HashSet<WbsId>` | Duplicates `ViewSettings.table.collapsed_wbs` | Remove; use view settings |
| `CadenceApp::resource_*_buffer` fields | Scattered UI buffers, not settings | Consolidate into `ToolSettings` / `ViewSettings` |
| `CadenceApp::assistant_config` | Duplicates `GlobalSettings.assistant` + `ViewSettings.assistant` | Remove; use settings context |
| `CadenceApp::update_status`, `update_checks_enabled` | Update check state, not settings | Keep as runtime state; move config to `GlobalSettings` |

### 9.2 Migration Path

1. **Add `SettingsContext` to `CadenceApp`** alongside existing fields.
2. **Initialize from loaded project** + global settings file.
3. **Gradually replace direct field access** with `settings.effective_*()` calls.
4. **Remove deprecated fields** once all views use the settings API.
5. **Add persistence** for Global (config file) and Project (SQLite).

---

## 10. Implementation Phases

### Phase 1: Core Data Structures (Week 1)
- [ ] Create `crates/core/src/settings.rs` with all data structures
- [ ] Implement `SettingsContext` with fallback logic
- [ ] Add serialization (serde) for all settings types
- [ ] Write unit tests for fallback chain and migration
- [ ] Add `cargo test` coverage for all migration paths

### Phase 2: Global Settings Persistence (Week 1-2)
- [ ] Implement config directory detection (cross-platform)
- [ ] Add `load_global_settings()` / `save_global_settings()`
- [ ] Add migration logic for global schema versions
- [ ] Integrate with `CadenceApp::new()` — load on startup
- [ ] Add "Save Global Settings" on change (debounced)

### Phase 3: Project Settings in SQLite (Week 2)
- [ ] Add `project_settings` table to persistence schema (v11)
- [ ] Implement `load_project_settings()` / `save_project_settings()`
- [ ] Extend `save_project` / `load_project` to handle settings
- [ ] Add migration for existing projects (default settings)
- [ ] Update `PersistenceError` for settings-specific errors

### Phase 4: View Settings (In-Memory + Optional Persist) (Week 2-3)
- [ ] Replace `CadenceApp` view-related fields with `ViewSettings`
- [ ] Implement `effective_*()` accessors in `SettingsContext`
- [ ] Update all views (Table, Gantt, Resources, etc.) to use settings API
- [ ] Add optional view settings persistence to SQLite (`view_settings` table)
- [ ] Add "Save View Layout" / "Reset View Layout" actions

### Phase 5: Tool Settings (Ephemeral) (Week 3)
- [ ] Define `ToolSettings` with all tool-specific state
- [ ] Replace scattered UI buffers in `CadenceApp` with `ToolSettings`
- [ ] Ensure tool settings reset on view change
- [ ] No persistence needed

### Phase 6: Settings UI (Week 3-4)
- [ ] Create `crates/ui-egui/src/settings_ui.rs`
- [ ] Implement Global Settings dialog (theme, density, shortcuts, assistant)
- [ ] Implement per-view settings panels (toolbar/menu access)
- [ ] Add keyboard shortcuts for density cycling (Ctrl+Shift+Minus/0/Plus)
- [ ] Add theme toggle shortcut (Ctrl+Shift+T)

### Phase 7: Change Notification & Sync (Week 4)
- [ ] Implement `SettingsEventBus` and `SettingsEvent` types
- [ ] Wire Global changes → UI update (theme, density)
- [ ] Wire Project changes → View recompute
- [ ] Verify no circular updates possible
- [ ] Add integration tests for cross-level sync

### Phase 8: Schema Migration & Testing (Week 4-5)
- [ ] Comprehensive migration tests for all three levels
- [ ] Test loading v0.1.0/v0.2.0 projects into v0.3.0
- [ ] Test global settings migration across versions
- [ ] CI: add settings migration test to pipeline

### Phase 9: Polish & Accessibility (Week 5)
- [ ] Ensure all settings dialogs are keyboard navigable
- [ ] Add AccessKit labels for screen readers
- [ ] Verify contrast in both themes for settings UI
- [ ] Reduced-motion support for settings animations
- [ ] Update `ACCESSIBILITY.md` with settings coverage

---

## 11. Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| Global settings persist to config directory | Manual test: change theme, restart, verify |
| Project settings persist in `.cadence` SQLite | Manual test: change currency, save, reopen, verify |
| View settings persist optionally | Manual test: adjust columns, "Save Layout", reopen, verify |
| Tool settings never persist | Code review: no serialization for `ToolSettings` |
| Density change rescales UI immediately | Manual test: Ctrl+Shift+Plus, verify scaling |
| Theme change repaints immediately | Manual test: Ctrl+Shift+T, verify |
| Fallback chain works correctly | Unit tests: `SettingsContext::effective_*()` |
| No circular updates | Code review: unidirectional event flow |
| Schema migration works | Unit tests: load v1 settings in v2 code |
| Keyboard shortcuts for density/theme work | Manual test: shortcuts cycle correctly |
| Settings dialogs accessible | axe-core / AccessKit test + manual NVDA/VoiceOver |

---

## 12. Future Extensions (Post v1.0)

- **Project-level density/theme override** — Allow project to override global density
- **Settings import/export** — JSON file for team sharing
- **Settings profiles** — Named presets (e.g., "Planning", "Field", "Review")
- **Plugin system for settings** — Extensions can contribute settings pages
- **Cloud sync** — Optional sync of global settings across devices
- **Per-user project settings** — Overlay user preferences on shared project

---

## 13. References

- Discovery Document: `docs/discovery/inspiration-apps-ui-ux-discovery.md`
- Hypotheses: H1 (3 densities), H2 (4-level hierarchy), H3 (resize), H4 (sync), H5 (keyboard), H6 (semantic colors), H7 (customization)
- Technical Questions: T1 (egui density), T2 (persistence), T3 (resize perf), T4 (AccessKit), T5 (sync)
- Target Epics: CADENCE-SETTINGS, CADENCE-TOOLBAR, CADENCE-DESIGN-TOKENS, CADENCE-ACCESSIBILITY, CADENCE-VIEW-SYNC