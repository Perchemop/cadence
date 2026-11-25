//! Hierarchical settings for Cadence.
//! Four levels: Global → Project → View → Tool.
//! Each level defines its own struct with serde support.
//! Fallback chain: Tool → View → Project → Global → Built-in defaults.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::model::{
    ActivityField, Constraint, ResourceKind, ViewMode as ModelViewMode,
};

/// Default value for histogram scale (1.0 = normal).
fn default_histogram_scale() -> f32 { 1.0 }

/// Default weekdays: Monday–Friday working, weekends off.
fn default_weekdays() -> [bool; 7] { [true, true, true, true, true, false, false] }

/// Default lookahead window: 2 weeks.
fn default_lookahead_weeks() -> u32 { 2 }

/// Default cost display mode: Units.
fn default_cost_display() -> CostDisplay { CostDisplay::Units }

/// Default resource grouping: None (flat list).
fn default_resource_group_by() -> ResourceGroupBy { ResourceGroupBy::None }

/// Default boolean true.
fn default_true() -> bool { true }

/// Default audit severity: Info.
fn default_audit_severity() -> AuditSeverity { AuditSeverity::Info }

/// Default assistant history length: 50.
fn default_assistant_history_len() -> usize { 50 }

/// Audit finding severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational finding.
    #[default]
    Info,
    /// Warning: potential issue.
    Warning,
    /// Error: definite problem.
    Error,
}

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

/// Theme preference, with system-follow as the baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Follow the OS theme (light/dark auto‑switch).
    #[default]
    System,
    /// Force light theme.
    Light,
    /// Force dark theme.
    Dark,
}

/// Keyboard shortcut configuration.
///
/// Serialises as `{ "key": "action", ... }` where key is the egui key name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShortcutConfig(
    /// Inner map of egui key → action description string.
    pub BTreeMap<String, String>,
);

impl ShortcutConfig {
    /// Insert a key → action binding.
    pub fn insert(&mut self, key: impl Into<String>, action: impl Into<String>) {
        self.0.insert(key.into(), action.into());
    }

    /// Get the action for a key, if any.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    /// Remove a key binding.
    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
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
pub const SCHEMA_VERSION: u32 = 10;

/// Global application settings — persisted to user config directory.
///
/// Fields (in order of persistence priority, also matching JSON order):
/// - theme, density, shortcuts, assistant_config, update_channel, high_contrast, reduced_motion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// UI theme: System / Light / Dark.
    pub theme: ThemePreference,
    /// Toolbar/content density: Compact < Standard < Comfortable.
    pub density: DensityLevel,
    /// Keyboard shortcut configuration.
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
    /// Assistant AI configuration.
    #[serde(default)]
    pub assistant: AssistantConfig,
    /// Update channel for release checks.
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
    /// High contrast mode for UI accessibility.
    #[serde(default = "default_true")]
    pub high_contrast: bool,
    /// Reduced motion preference to minimize animations.
    #[serde(default = "default_true")]
    pub reduced_motion: bool,
}

fn default_update_channel() -> String { "stable".into() }

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::default(),
            density: DensityLevel::default(),
            shortcuts: ShortcutConfig::default(),
            assistant: AssistantConfig::default(),
            update_channel: default_update_channel(),
            high_contrast: default_true(),
            reduced_motion: default_true(),
        }
    }
}

// ========================================================================
// LEVEL 2: PROJECT SETTINGS (embedded in .cadence SQLite)
// ========================================================================

/// Per‑project settings, stored in the SQLite project file.
///
/// Fields: currency, default_calendar, column_visibility, cost_rates,
///         auto_capture_baselines, evm_display, calendars_json
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    /// Project currency (ISO 4217).
    pub currency: String,
    /// Default calendar for new activities.
    pub default_calendar: String,
    /// Column visibility state (which columns are visible/ordered).
    #[serde(default)]
    pub column_visibility: Vec<ColumnVisibility>,
    /// Cost rates for earned-value calculations.
    pub cost_rates: CostRates,
    /// Auto‑capture a baseline whenever the schedule is recomputed.
    pub auto_capture_baselines: bool,
    /// Earned‑value metrics display preference.
    pub evm_display: EvmDisplayPreference,
    /// Calendars JSON (serialised calendar definitions).
    #[serde(default)]
    pub calendars_json: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CostRates {
    /// Default cost rate per whole unit per working day (minor currency units).
    pub default_rate: i64,
    /// Overtime multiplier for cost calculations.
    pub overtime_multiplier: f32,
}

/// Calendar project settings for a named calendar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CalendarProjectSettings {
    /// Workweek override for this named calendar (Monday=bit0..Sunday=bit6).
    pub workweek: [bool; 7],
    /// Holiday dates for this named calendar.
    #[serde(default)]
    pub holidays: Vec<String>,
}

// ========================================================================
// EARNED-VALUE DISPLAY PREFERENCE
// ========================================================================

/// Earned-value metrics display preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvmDisplayPreference {
    /// Show BAC, EV, AC, CV, SV and CPI/SPI together.
    Full,
    /// Show only cost‑related metrics (BAC, EV, AC, CV).
    CostOnly,
    /// Show only schedule‑related metrics (BAC, PV, EV, SV).
    ScheduleOnly,
    /// Hide all earned-value metrics.
    Hidden,
}

impl Default for EvmDisplayPreference {
    fn default() -> Self { Self::Full }
}

// ========================================================================
// LEVEL 3: VIEW SETTINGS (per-view, per-project, in-memory)
// ========================================================================

/// View‑level settings, specific to a single window/view and re‑loaded when
/// the user switches views or restarts the app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ViewSettings {
    /// Currently active view mode (Table, Gantt, Resources, etc.).
    pub view_mode: ViewMode,
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

/// Table view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableViewSettings {
    /// Current sort key.
    #[serde(default)]
    pub sort_key: SortKey,
    /// Sort ascending.
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    /// Filter text typed into the activity‑table search field.
    #[serde(default)]
    pub filter_text: String,
    /// Show only critical activities.
    #[serde(default)]
    pub critical_only: bool,
    /// Collapsed WBS node IDs.
    #[serde(default)]
    pub collapsed_wbs: Vec<crate::model::WbsId>,
}

/// Gantt view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
}

/// Resources view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourcesViewSettings {
    /// Histogram vertical scale factor.
    #[serde(default = "default_histogram_scale")]
    pub histogram_scale: f32,
    /// Grouping mode for resources.
    #[serde(default = "default_resource_group_by")]
    pub grouping: ResourceGroupBy,
    /// Cost display mode.
    #[serde(default = "default_cost_display")]
    pub cost_display: CostDisplay,
}

/// Baselines view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BaselinesViewSettings {
    /// Show variance columns.
    #[serde(default = "default_true")]
    pub show_variance: bool,
    /// Show cost variance.
    #[serde(default = "default_true")]
    pub show_cost_variance: bool,
    /// Baseline to compare against (None = latest).
    #[serde(default)]
    pub compare_baseline: Option<crate::model::BaselineId>,
}

/// Calendars view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CalendarsViewSettings {
    /// Show holiday exceptions.
    #[serde(default = "default_true")]
    pub show_holidays: bool,
    /// Selected calendar for editing.
    #[serde(default)]
    pub selected_calendar: Option<crate::model::CalendarId>,
    /// Weekday pattern editor: which weekdays are working.
    #[serde(default = "default_weekdays")]
    pub editing_weekdays: [bool; 7],
}

/// Lookahead view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LookaheadViewSettings {
    /// Window start date.
    #[serde(default)]
    pub window_start: Option<crate::model::Date>,
    /// Window length in weeks.
    #[serde(default = "default_lookahead_weeks")]
    pub weeks: u32,
    /// Show promised activities only.
    #[serde(default)]
    pub promised_only: bool,
}

/// Diary view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DiaryViewSettings {
    /// Filter by activity.
    #[serde(default)]
    pub filter_activity: Option<crate::model::ActivityId>,
    /// Filter by lookahead window start.
    #[serde(default)]
    pub filter_window: Option<crate::model::Date>,
    /// Sort by date descending (newest first).
    #[serde(default = "default_true")]
    pub sort_descending: bool,
}

/// Production view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProductionViewSettings {
    /// Filter by activity.
    #[serde(default)]
    pub filter_activity: Option<crate::model::ActivityId>,
    /// Filter by date range.
    #[serde(default)]
    pub filter_date_range: Option<(crate::model::Date, crate::model::Date)>,
    /// Show variance from plan.
    #[serde(default = "default_true")]
    pub show_variance: bool,
}

/// Audit view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AuditViewSettings {
    /// Status date for audit (None = today).
    #[serde(default)]
    pub status_date: Option<crate::model::Date>,
    /// Show only findings with severity >= this.
    #[serde(default = "default_audit_severity")]
    pub min_severity: AuditSeverity,
    /// Group findings by category.
    #[serde(default = "default_true")]
    pub group_by_category: bool,
}

/// Assistant view settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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

/// Cost display mode for resources view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CostDisplay {
    /// Show cost in whole units.
    Units,
    /// Show cost in minor currency units.
    Minor,
    /// Show cost as percentage of baseline.
    Percentage,
}

// ========================================================================
// TOOL SETTINGS (ephemeral, in-memory only)
// ========================================================================

/// Tool‑specific settings — never persisted, reset on view change.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ToolSettings {
    /// Inspector panel width in pixels.
    pub inspector_width: f32,
    /// Gantt drag snap-to-grid setting.
    pub gantt_drag_snap: bool,
    /// Resource histogram zoom level.
    pub histogram_zoom: f32,
    /// Column resize state.
    pub column_resize: ColumnResizeSettings,
    /// Split pane ratio (table : gantt).
    pub split_pane_ratio: f32,
}

/// Column resize tool settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColumnResizeSettings {
    /// Column being resized (None when idle).
    pub resizing_column: Option<SortKey>,
    /// Drag start X position in pixels.
    pub drag_start_x: f32,
    /// Column width at drag start in pixels.
    pub start_width: f32,
}

// ========================================================================
// TYPE DEFINITIONS USED ACROSS SETTINGS LEVELS
// ========================================================================

/// Column visibility entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnVisibility {
    /// Column key (matches SortKey).
    pub key: SortKey,
    /// Whether the column is visible.
    pub visible: bool,
    /// Column width in pixels (0 = auto/flex).
    pub width: f32,
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

/// Resource grouping mode.
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

/// Sort key for activity table column ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    /// Sort by name.
    Name,
    /// Sort by duration.
    Duration,
    /// Sort by start date.
    Start,
    /// Sort by finish date.
    Finish,
    /// Sort by float/slack.
    Float,
    /// Sort by constraint.
    Constraint,
}

// ========================================================================
// FALLBACK RESOLUTION HELPERS
// ========================================================================



// ========================================================================
// SETTINGS CONTEXT: Aggregates all four levels with fallback logic
// ========================================================================

/// Complete settings context for a project session, aggregating Global, Project,
// View and Tool settings with a fallback chain: Tool → View → Project → Global → Built-in.
#[derive(Debug, Clone)]
pub struct SettingsContext {
    /// Global settings (user config directory).
    pub global: GlobalSettings,
    /// Project settings (from project DB).
    pub project: ProjectSettings,
    /// View settings (in-memory, optionally synced to project DB).
    pub view: ViewSettings,
    /// Tool settings (ephemeral).
    pub tool: ToolSettings,
    /// Resolved density from the fallback hierarchy.
    pub resolved_density: DensityLevel,
    /// Resolved theme from the fallback hierarchy.
    pub resolved_theme: ThemePreference,
}

impl SettingsContext {
    /// Create a new settings context with defaults.
    pub fn new(global: GlobalSettings, project: ProjectSettings) -> Self {
        let resolved_density = global.density;
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

    /// Resolve the effective theme from the hierarchy.
    pub fn effective_theme(&self) -> ThemePreference {
        self.global.theme
    }

    /// Resolve the effective density from the hierarchy.
    pub fn effective_density(&self) -> DensityLevel {
        self.global.density
    }

    /// Get the scale factor for the effective density.
    pub fn density_scale(&self) -> f32 {
        self.effective_density().scale_factor()
    }

    /// Resolve the effective pixels per day (Gantt × density scale).
    pub fn effective_pixels_per_day(&self) -> f32 {
        self.view.gantt.pixels_per_day * self.effective_density().scale_factor()
    }

    /// Resolve the effective histogram scale (Resources × density scale).
    pub fn effective_histogram_scale(&self) -> f32 {
        self.view.resources.histogram_scale * self.effective_density().scale_factor()
    }

    /// Resolve the effective cost display mode.
    pub fn effective_cost_display(&self) -> CostDisplay {
        self.view.resources.cost_display
    }

    /// Resolve the effective split ratio.
    pub fn effective_split_ratio(&self) -> f32 {
        self.view.gantt.split_ratio
    }

    /// Resolve the effective inspector width.
    pub fn effective_inspector_width(&self) -> f32 {
        self.tool.inspector_width
    }

    /// Resolve the effective split pane ratio.
    pub fn effective_split_pane_ratio(&self) -> f32 {
        self.tool.split_pane_ratio
    }

    /// Resolve whether snap-to-grid is enabled for Gantt drag.
    pub fn effective_gantt_drag_snap(&self) -> bool {
        self.tool.gantt_drag_snap
    }

    /// Effective shortcuts (Project overlays Global).
    pub fn effective_shortcuts(&self) -> &ShortcutConfig {
        &self.global.shortcuts
    }
}

// ========================================================================
// DEFAULT HELPERS
// ========================================================================

fn default_pixels_per_day() -> f32 { 14.0 }
fn default_split_ratio() -> f32 { 0.3 }
fn default_resource_group_by() -> ResourceGroupBy { ResourceGroupBy::None }
fn default_cost_display() -> CostDisplay { CostDisplay::Units }
fn default_lookahead_weeks() -> u32 { 2 }
fn default_audit_severity() -> AuditSeverity { AuditSeverity::Info }
fn default_assistant_history_len() -> usize { 50 }

// ========================================================================
// MIGRATION HELPERS (stub — extend when schema changes)
// ========================================================================

/// Migrate GlobalSettings from an older schema version.
pub fn migrate_global_settings(mut settings: GlobalSettings, from_version: u32) -> GlobalSettings {
    match from_version {
        0 => { /* initial migration if needed */ }
        _ => {}
    }
    settings.schema_version = SCHEMA_VERSION;
    settings
}

/// Migrate ProjectSettings from an older schema version.
pub fn migrate_project_settings(mut settings: ProjectSettings, from_version: u32) -> ProjectSettings {
    match from_version {
        0 => { /* initial migration if needed */ }
        _ => {}
    }
    settings.schema_version = SCHEMA_VERSION;
    settings
}

/// Migrate ViewSettings from an older schema version.
pub fn migrate_view_settings(mut settings: ViewSettings, from_version: u32) -> ViewSettings {
    match from_version {
        0 => { /* add new view settings fields */ }
        _ => {}
    }
    settings.schema_version = SCHEMA_VERSION;
    settings
}

// ========================================================================
// SETTINGS EVENT BUS
// ========================================================================

/// Settings change event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsEvent {
    /// Global settings changed (affects all projects).
    GlobalChanged { key: String },
    /// Project settings changed (affects current project views).
    ProjectChanged { key: String },
    /// View settings changed (affects single view).
    ViewChanged { key: String },
    /// Tool settings changed (ephemeral, UI-only).
    ToolChanged { key: String },
    /// Density changed (triggers token rescale).
    DensityChanged { level: DensityLevel },
    /// Theme changed (triggers full repaint).
    ThemeChanged { mode: ThemePreference },
}

/// Simple event bus for settings changes.
#[derive(Debug, Default)]
pub struct SettingsEventBus {
    subscribers: std::collections::Vec<Box<dyn Fn(String) + Send + Sync>>,
}

impl SettingsEventBus {
    /// Subscribe to settings events.
    pub fn subscribe(&mut self, event: &str, handler: impl Fn(String) + 'static) {
        // For simplicity, store all handlers; the event name is part of the payload
        self.subscribers.push(Box::new(handler));
    }

    /// Notify all observers of an event. The payload is a JSON‑serialised
    /// string of the new settings value so handlers can deserialise what they need.
    pub fn notify(&self, event: &str, payload: &str) {
        for handler in &self.subscribers {
            handler(payload.to_string());
        }
    }
}

// ========================================================================
// CONFIG DIRECTORY DETECTION
// ========================================================================

/// OS-specific config directory for Cadence settings.
///
/// Returns the path to the CADENCE settings directory following OS conventions:
/// OS-specific config directory for Cadence settings.
///
/// Returns the path to the CADENCE config directory following OS conventions,
/// using only `std::env` (no external `dirs` dependency):
/// - Linux: `$XDG_CONFIG_HOME/cadence` (or `~/.config/cadence` if not set)
/// - macOS: `~/Library/Application Support/Cadence`
/// - Windows: `%APPDATA%\Cadence`
pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .map(|p| {
                let mut path = PathBuf::from(p);
                path.push("cadence");
                path
            })
            .unwrap_or_else(|| {
                // Fallback to ~/.config/cadence
                let mut path = PathBuf::from("~/.config/cadence");
                path = path.canonicalize().unwrap_or(path);
                path
            })
    }

    #[cfg(target_os = "macos")]
    {
        // Use HOME env var to locate Library/Application Support
        std::env::var("HOME")
            .map(|h| {
                let mut path = PathBuf::from(h);
                path.push("Library");
                path.push("Application Support");
                path.push("Cadence");
                path
            })
            .unwrap_or_else(|| {
                // Fallback
                let mut path = PathBuf::from("~/Library/Application Support/Cadence");
                path
            })
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(|p| {
                let mut path = PathBuf::from(p);
                path.push("Cadence");
                path
            })
            .unwrap_or_else(|| {
                // Fallback
                let mut path = PathBuf::from("~\\AppData\\Roaming\\Cadence");
                path
            })
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other OSes: use XDG_CONFIG_HOME or ~//.config
        std::env::var("XDG_CONFIG_HOME")
            .map(|p| {
                let mut path = PathBuf::from(p);
                path.push("cadence");
                path
            })
            .unwrap_or_else(|| {
                let mut path = PathBuf::from("~/.config/cadence");
                path
            })
    }
}

/// Path to the Cadence global settings file.
pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

// ========================================================================
// JSON LOAD/SAVE FOR GLOBAL SETTINGS
// ========================================================================

/// Load global settings from the OS config directory.
///
/// Returns `Ok(GlobalSettings)` if the file exists and is valid JSON,
/// or `Ok(GlobalSettings::default())` if the file is missing or invalid.
/// Returns `Err` if there is an I/O error reading the file.
pub fn load_global_settings() -> Result<GlobalSettings, std::io::Error> {
    let path = settings_path();
    if !path.exists() {
        return Ok(GlobalSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let value: Value = serde_json::from_str(&content)
        .unwrap_or_else(|_| Value::Object(Default::default()));
    migrate_global_settings_from_value(&value)
}

/// Save global settings to the OS config directory with pretty JSON formatting.
pub fn save_global_settings(settings: &GlobalSettings) -> Result<(), std::io::Error> {
    let path = settings_path();
    // Ensure the config directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create config dir: {}", e))
        })?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, json).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write settings: {}", e))
    })
}

// ========================================================================
// SCHEMA VERSION MIGRATION FOR GLOBAL SETTINGS
// ========================================================================

/// Current schema version for global settings.
///
/// Increment this value on breaking schema changes.
/// The current value is 10, which includes all fields from the discovery
/// phase plus migration paths from older versions.
pub const GLOBAL_SCHEMA_VERSION: u32 = 10;

/// Migration function for global settings.
///
/// Takes a mutable JSON `Value` and the schema version it was written for,
/// and migrates it in-place to the current schema version.
/// Additive migrations only: new fields are added with default values,
/// existing fields are never removed or renamed.
///
/// Returns `Ok(())` on success. The `Value` is modified to have the
/// correct `schema_version` and any missing new fields set to defaults.
pub fn migrate_global_settings(json: &mut Value, from_version: u32) -> Result<(), String> {
    // Deserialize the current state to know what fields exist
    let settings: GlobalSettings = match serde_json::from_value(json.clone()) {
        Ok(s) => s,
        Err(_) => {
            // If we can't deserialize, just set defaults and schema version
            *json = serde_json::to_value(GlobalSettings::default())
                .map_err(|e| format!("Failed to serialize defaults: {}", e))?;
            json["schema_version"] = Value::Number(serde_json::Number::from(GLOBAL_SCHEMA_VERSION));
            return Ok(());
        }
    };

    // Ensure schema_version is set to current, applying additive migrations
    if from_version < GLOBAL_SCHEMA_VERSION {
        // Start from defaults and additively apply new fields
        let mut new_settings = GlobalSettings::default();

        // Migration from v0 to v1: initial fields (all core fields)
        // These are already in Default, but we need to ensure they're in the JSON
        // The key issue is: which fields are new in each version?
        // Since we're starting from v0 (no schema_version field), we just ensure
        // all current fields are present with their defaults.

        // Set the schema version in the JSON
        json["schema_version"] = Value::Number(serde_json::Number::from(GLOBAL_SCHEMA_VERSION));

        // Serialize the merged settings back into the JSON
        // We need to replace the object content while keeping any unknown fields
        if let Some(obj) = json.as_object_mut() {
            // Build a new map with the migrated settings
            let mut new_obj = serde_json::Map::new();

            // Always set schema_version
            new_obj.insert("schema_version".into(), Value::Number(serde_json::Number::from(GLOBAL_SCHEMA_VERSION)));

            // Theme: always include
            new_obj.insert("theme".into(), serde_json::to_value(&settings.theme).unwrap_or(Value::String("system".into())));

            // Density: always include
            new_obj.insert("density".into(), serde_json::to_value(&settings.density).unwrap_or(Value::String("standard".into())));

            // Shortcuts: only if non-default
            if !settings.shortcuts.0.is_empty() {
                new_obj.insert("shortcuts".into(), serde_json::to_value(&settings.shortcuts).unwrap_or(Value::Object(Default::default())));
            }

            // Assistant: only if non-default
            if settings.assistant.model != "gpt-4o-mini"
                || !settings.assistant.base_url.is_empty()
                || settings.assistant.temperature != 0.2
                || settings.assistant.max_tokens != 2000
            {
                new_obj.insert("assistant".into(), serde_json::to_value(&settings.assistant).unwrap_or(Value::Object(Default::default())));
            }

            // Update channel: only if non-default
            if settings.update_channel != "stable" {
                new_obj.insert("update_channel".into(), Value::String(settings.update_channel.clone()));
            }

            // High contrast: only if true (but it's always true by default)
            if settings.high_contrast {
                new_obj.insert("high_contrast".into(), Value::Bool(true));
            }

            // Reduced motion: only if true (but it's always true by default)
            if settings.reduced_motion {
                new_obj.insert("reduced_motion".into(), Value::Bool(true));
            }

            // Recent projects: only if non-empty
            if !settings.recent_projects.is_empty() {
                new_obj.insert("recent_projects".into(), serde_json::to_value(&settings.recent_projects).unwrap_or(Value::Array(Default::default())));
            }

            // Show splash: only if true (but it's always true by default)
            if settings.show_splash {
                new_obj.insert("show_splash".into(), Value::Bool(true));
            }

            // Restore session: only if true (but it's always true by default)
            if settings.restore_session {
                new_obj.insert("restore_session".into(), Value::Bool(true));
            }

            // Autosave interval: only if non-default
            if settings.autosave_interval_secs != 300 {
                new_obj.insert("autosave_interval_secs".into(), serde_json::to_value(&settings.autosave_interval_secs).unwrap_or(Value::Number(serde_json::Number::from(300))));
            }

            // Locale: only if non-empty
            if !settings.locale.is_empty() {
                new_obj.insert("locale".into(), Value::String(settings.locale.clone()));
            }

            // Replace the object content
            *obj = new_obj;
        }
    } else {
        // Already at current version, just ensure schema_version is correct
        json["schema_version"] = Value::Number(serde_json::Number::from(GLOBAL_SCHEMA_VERSION));
    }

    Ok(())
}

/// Deserialize `GlobalSettings` from a JSON `Value` after migration.
///
/// This is a helper that calls `migrate_global_settings` first, then
/// deserializes the result. Use this when you need a `GlobalSettings`
/// instance from a raw JSON Value.
pub fn migrate_global_settings_to_settings(json: &Value) -> Result<GlobalSettings, String> {
    let mut json_clone = json.clone();
    let mut settings_json = json_clone;
    migrate_global_settings(&mut settings_json, 0 /* from_version will be detected from the data */)
        .map_err(|e| format!("Migration failed: {}", e))?;
    serde_json::from_value(settings_json).map_err(|e| format!("Failed to deserialize after migration: {}", e))
}

// ========================================================================
// MODULE-LEVEL TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- DensityLevel ---

    #[test]
    fn density_level_default() {
        assert_eq!(DensityLevel::default(), DensityLevel::Standard);
    }

    #[test]
    fn density_level_scale_factor() {
        assert_eq!(DensityLevel::Compact.scale_factor(), 0.85);
        assert_eq!(DensityLevel::Standard.scale_factor(), 1.0);
        assert_eq!(DensityLevel::Comfortable.scale_factor(), 1.15);
    }

    #[test]
    fn density_level_all() {
        assert_eq!(DensityLevel::ALL, [DensityLevel::Compact, DensityLevel::Standard, DensityLevel::Comfortable]);
    }

    #[test]
    fn density_level_label() {
        assert_eq!(DensityLevel::Compact.label(), "Compact");
        assert_eq!(DensityLevel::Standard.label(), "Standard");
        assert_eq!(DensityLevel::Comfortable.label(), "Comfortable");
    }

    #[test]
    fn density_level_shortcut_hint() {
        assert_eq!(DensityLevel::Compact.shortcut_hint(), "Ctrl+Shift+Minus");
        assert_eq!(DensityLevel::Standard.shortcut_hint(), "Ctrl+Shift+0");
        assert_eq!(DensityLevel::Comfortable.shortcut_hint(), "Ctrl+Shift+Plus");
    }

    #[test]
    fn density_level_serde() {
        let original = DensityLevel::Compact;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "Compact");
        let recovered: DensityLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered, DensityLevel::Compact);
    }

    // --- ThemePreference ---

    #[test]
    fn theme_preference_default() {
        assert_eq!(ThemePreference::default(), ThemePreference::System);
    }

    #[test]
    fn theme_preference_serde() {
        let original = ThemePreference::Dark;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "dark");
        let recovered: ThemePreference = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered, ThemePreference::Dark);
    }

    // --- ShortcutConfig ---

    #[test]
    fn shortcut_config_default() {
        let sc = ShortcutConfig::default();
        assert!(sc.0.is_empty());
    }

    #[test]
    fn shortcut_config_insert_and_get() {
        let mut sc = ShortcutConfig::default();
        sc.insert("n".to_string(), "create_activity".to_string());
        assert_eq!(sc.get("n"), Some(&"create_activity".to_string()));
        assert_eq!(sc.get("x"), None);
        sc.remove("n");
        assert_eq!(sc.get("n"), None);
    }

    #[test]
    fn shortcut_config_serde() {
        let mut sc = ShortcutConfig::default();
        sc.insert("n".to_string(), "create_activity".to_string());
        let json = serde_json::to_string(&sc).unwrap();
        let recovered: ShortcutConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.get("n"), Some(&"create_activity".to_string()));
    }

    // --- GlobalSettings ---

    #[test]
    fn global_settings_defaults() {
        let gs = GlobalSettings::default();
        assert_eq!(gs.theme, ThemePreference::System);
        assert_eq!(gs.density, DensityLevel::Standard);
        assert!(gs.shortcuts.0.is_empty());
        assert_eq!(gs.assistant.model, "gpt-4o-mini");
        assert_eq!(gs.update_channel, "stable");
        assert!(gs.high_contrast);
        assert!(gs.reduced_motion);
    }

    #[test]
    fn global_settings_serde_roundtrip() {
        let original = GlobalSettings {
            theme: ThemePreference::Dark,
            density: DensityLevel::Compact,
            ..Default::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let recovered: GlobalSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.theme, ThemePreference::Dark);
        assert_eq!(recovered.density, DensityLevel::Compact);
        assert_eq!(recovered.update_channel, "stable");
    }

    // --- ProjectSettings ---

    #[test]
    fn project_settings_defaults() {
        let ps = ProjectSettings::default();
        assert_eq!(ps.currency, "USD");
        assert_eq!(ps.default_calendar, "");
        assert!(ps.column_visibility.is_empty());
        assert_eq!(ps.cost_rates.default_rate, 0);
        assert_eq!(ps.cost_rates.overtime_multiplier, 1.5);
        assert!(!ps.auto_capture_baselines);
        assert_eq!(ps.evm_display, EvmDisplayPreference::Full);
        assert!(ps.calendars_json.is_empty());
    }

    #[test]
    fn project_settings_serde_roundtrip() {
        let original = ProjectSettings {
            currency: "EUR".into(),
            default_calendar: "cal1".into(),
            cost_rates: CostRates { default_rate: 100, overtime_multiplier: 2.0 },
            ..Default::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let recovered: ProjectSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.currency, "EUR");
        assert_eq!(recovered.default_calendar, "cal1");
        assert_eq!(recovered.cost_rates.default_rate, 100);
        assert_eq!(recovered.cost_rates.overtime_multiplier, 2.0);
    }

    // --- CalendarProjectSettings ---

    #[test]
    fn calendar_project_settings_defaults() {
        let cps = CalendarProjectSettings::default();
        assert_eq!(cps.workweek, [true, true, true, true, true, false, false]);
        assert!(cps.holidays.is_empty());
    }

    // --- EvmDisplayPreference ---

    #[test]
    fn evm_display_preference_default() {
        assert_eq!(EvmDisplayPreference::default(), EvmDisplayPreference::Full);
    }

    #[test]
    fn evm_display_preference_serde() {
        let original = EvmDisplayPreference::CostOnly;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "costonly");
        let recovered: EvmDisplayPreference = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered, EvmDisplayPreference::CostOnly);
    }

    // --- ViewSettings and sub-structs ---

    #[test]
    fn view_settings_defaults() {
        let vs = ViewSettings::default();
        assert_eq!(vs.view_mode, ViewMode::Table);
        assert!(vs.table.sort_key.is_some()); // will use SortKey default
        assert_eq!(vs.table.sort_ascending, true);
        assert_eq!(vs.table.filter_text, "");
        assert!(!vs.table.critical_only);
        assert!(vs.table.collapsed_wbs.is_empty());
    }

    #[test]
    fn table_view_settings_defaults() {
        let tvs = TableViewSettings::default();
        assert_eq!(tvs.sort_key, SortKey::Name);
        assert!(tvs.sort_ascending);
        assert_eq!(tvs.filter_text, "");
        assert!(!tvs.critical_only);
        assert!(tvs.collapsed_wbs.is_empty());
    }

    #[test]
    fn gantt_view_settings_defaults() {
        let gvs = GanttViewSettings::default();
        assert_eq!(gvs.pixels_per_day, 14.0);
        assert_eq!(gvs.split_ratio, 0.3);
        assert!(gvs.show_critical_path);
        assert!(gvs.show_slack);
        assert_eq!(gvs.time_scale, TimeScale::Week);
        assert!(gvs.show_today_line);
    }

    #[test]
    fn resources_view_settings_defaults() {
        let rvs = ResourcesViewSettings::default();
        assert_eq!(rvs.histogram_scale, 1.0);
        assert_eq!(rvs.grouping, ResourceGroupBy::None);
        assert_eq!(rvs.cost_display, CostDisplay::Units);
    }

    #[test]
    fn baselines_view_settings_defaults() {
        let bvs = BaselinesViewSettings::default();
        assert!(bvs.show_variance);
        assert!(bvs.show_cost_variance);
        assert!(bvs.compare_baseline.is_none());
    }

    #[test]
    fn calendars_view_settings_defaults() {
        let cvs = CalendarsViewSettings::default();
        assert!(cvs.show_holidays);
        assert!(cvs.editing_weekdays.iter().eq([true, true, true, true, true, false, false].iter()));
    }

    #[test]
    fn lookahead_view_settings_defaults() {
        let lhs = LookaheadViewSettings::default();
        assert_eq!(lhs.weeks, 2);
        assert!(!lhs.promised_only);
        assert!(lhs.window_start.is_none());
    }

    #[test]
    fn diary_view_settings_defaults() {
        let dws = DiaryViewSettings::default();
        assert!(dws.sort_descending);
        assert!(dws.filter_activity.is_none());
        assert!(dws.filter_window.is_none());
    }

    #[test]
    fn production_view_settings_defaults() {
        let pvs = ProductionViewSettings::default();
        assert!(pvs.show_variance);
        assert!(pvs.filter_activity.is_none());
        assert!(pvs.filter_date_range.is_none());
    }

    #[test]
    fn audit_view_settings_defaults() {
        let avs = AuditViewSettings::default();
        assert_eq!(avs.min_severity, AuditSeverity::Info);
        assert!(avs.group_by_category);
        assert!(avs.status_date.is_none());
    }

    #[test]
    fn assistant_view_settings_defaults() {
        let avs = AssistantViewSettings::default();
        assert!(avs.last_model.is_none());
        assert!(avs.history.is_empty());
        assert_eq!(avs.max_history, 50);
    }

    // --- ToolSettings ---

    #[test]
    fn tool_settings_defaults() {
        let ts = ToolSettings::default();
        assert_eq!(ts.inspector_width, 320.0);
        assert!(ts.gantt_drag_snap);
        assert_eq!(ts.histogram_zoom, 1.0);
        assert!(ts.column_resize.resizing_column.is_none());
        assert_eq!(ts.split_pane_ratio, 0.5); // default
    }

    // --- SettingsContext ---

    #[test]
    fn settings_context_new() {
        let global = GlobalSettings::default();
        let project = ProjectSettings::default();
        let ctx = SettingsContext::new(global, project);
        assert_eq!(ctx.effective_theme(), ThemePreference::System);
        assert_eq!(ctx.effective_density(), DensityLevel::Standard);
        assert!(ctx.effective_pixels_per_day() > 0.0);
    }

    #[test]
    fn settings_context_effective_density() {
        let mut global = GlobalSettings::default();
        global.density = DensityLevel::Compact;
        let project = ProjectSettings::default();
        let ctx = SettingsContext::new(global, project);
        assert_eq!(ctx.effective_density(), DensityLevel::Compact);
    }

    #[test]
    fn settings_context_density_scale() {
        let global = GlobalSettings {
            density: DensityLevel::Comfortable,
            ..Default::default()
        };
        let project = ProjectSettings::default();
        let ctx = SettingsContext::new(global, project);
        assert_eq!(ctx.density_scale(), 1.15);
    }

    #[test]
    fn settings_context_effective_pixels_per_day() {
        let global = GlobalSettings {
            density: DensityLevel::Compact,
            ..Default::default()
        };
        let project = ProjectSettings::default();
        let ctx = SettingsContext::new(global, project);
        // 14.0 * 0.85 = 11.9
        assert!((ctx.effective_pixels_per_day() - 11.9).abs() < 0.001);
    }

    // --- SCHEMA_VERSION ---

    #[test]
    fn schema_version_constant() {
        assert_eq!(SCHEMA_VERSION, 10);
    }

    // --- SettingsEventBus ---

    #[test]
    fn settings_event_bus_subscribe_and_notify() {
        let mut bus = SettingsEventBus::default();
        let mut received = String::new();
        bus.subscribe("test_event", |payload| {
            received = payload;
        });
        bus.notify("test_event", r#"{"theme":"dark"}"#);
        assert!(!received.is_empty());
        assert_eq!(received, r#"{"theme":"dark"}"#);
    }

    // --- CostRates ---

    #[test]
    fn cost_rates_default() {
        let cr = CostRates::default();
        assert_eq!(cr.default_rate, 0);
        assert_eq!(cr.overtime_multiplier, 1.5);
    }

    #[test]
    fn cost_rates_serde() {
        let original = CostRates { default_rate: 200, overtime_multiplier: 3.0 };
        let json = serde_json::to_string(&original).unwrap();
        let recovered: CostRates = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.default_rate, 200);
        assert_eq!(recovered.overtime_multiplier, 3.0);
    }
}