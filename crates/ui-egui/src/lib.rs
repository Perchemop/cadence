//! egui/eframe user interface for Cadence.
//!
//! Phase 2 starts here: a small, dependency-light shell that renders the
//! scheduled project in a window. The scheduling engine remains the single
//! source of truth; this crate only displays its results and routes edits
//! through the command layer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use cadence_commands::{
    AddResourceToActivity, CaptureBaseline, CommandStack, CreateCalendar, CreateRelationship,
    CreateResource, DeleteCalendar, DeleteRelationship, DeleteResource, RemoveBaseline,
    RemoveResourceFromActivity, RenameActivity, RenameCalendar, RenameResource,
    SetActivityActualCost, SetActivityActualDates, SetActivityBudget, SetActivityCalendar,
    SetActivityConstraint, SetActivityDuration, SetActivityOwner, SetActivityProgress,
    SetActivityPromise, SetCalendarDefinition, SetRelationshipLag, SetRelationshipType,
    SetResourceCapacity, SetResourceKind,
};
use cadence_commands::{SetAssignmentUnits, SetResourceCostRate};
use cadence_core::audit::{audit, AuditConfig, AuditReport, CheckOutcome, Finding};

use cadence_core::calendar_set::CalendarId;
use cadence_core::clash::location_clashes;
use cadence_core::engine::schedule;
use cadence_core::engine::Schedule;
use cadence_core::engine::{
    activity_progress, evm_metrics, level_resources, resource_conflicts, resource_histogram,
    schedule_with_progress, ProgressState, ResourceConflict, ResourceHistogram,
};
use cadence_core::lookahead::{lookahead, LookaheadEntry};
use cadence_core::model::FULL_UNITS;
use cadence_core::model::{
    Activity, ActivityId, Baseline, BaselineId, Calendar, Constraint, DailyNoteId, EvmMetrics,
    LinkType, LookaheadSnapshotEntry, LookaheadSnapshotId, Project, ResourceId, ResourceKind,
};
use egui::{Align2, Color32, ComboBox, Context, Grid, Pos2, Rect, ScrollArea, Sense, Ui, Vec2};
use time::{Date, Month};

pub mod crash;
pub mod design;
pub mod update_check;

/// Accent colour used for selection and highlights in both themes.
pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xED);

/// Design tokens shared by the light and dark themes.
#[derive(Debug, Clone, Copy)]
pub struct DesignTokens {
    /// Primary accent.
    pub accent: Color32,
    /// Critical activity colour.
    pub critical: Color32,
    /// Non-critical activity colour.
    pub normal: Color32,
    /// Primary action background.
    pub primary_bg: Color32,
    /// Accent background (for selected tabs).
    pub accent_bg: Color32,
    /// Panel/Toolbar background.
    pub panel_bg: Color32,
}

impl DesignTokens {
    /// Tokens for the requested theme, sourced from the design palette so
    /// both themes keep their WCAG AA verified contrast.
    pub fn for_dark(dark: bool) -> Self {
        let palette = design::palette(dark);
        Self {
            accent: ACCENT,
            critical: palette.critical,
            normal: palette.normal,
            primary_bg: ACCENT.linear_multiply(0.15),
            accent_bg: ACCENT.linear_multiply(0.2),
            panel_bg: palette.panel.linear_multiply(0.95),
        }
    }
}

/// Apply a theme to an egui context.
pub fn apply_theme(ctx: &Context, dark: bool) {
    design::apply(ctx, dark);
}

/// Sort key for the activity table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SortKey {
    Name,
    Duration,
    Start,
    Finish,
    Float,
    Critical,
}

impl SortKey {
    /// Every data column that can be shown in the activity table, in the order
    /// they appear left to right.
    fn ordered_columns() -> Vec<SortKey> {
        vec![
            SortKey::Name,
            SortKey::Duration,
            SortKey::Start,
            SortKey::Finish,
            SortKey::Float,
            SortKey::Critical,
        ]
    }

    /// Every data column as a set (for membership tests).
    fn all_columns() -> BTreeSet<SortKey> {
        Self::ordered_columns().into_iter().collect()
    }

    /// Human-readable column label.
    fn label(self) -> &'static str {
        match self {
            SortKey::Name => "Activity",
            SortKey::Duration => "Dur",
            SortKey::Start => "Start",
            SortKey::Finish => "Finish",
            SortKey::Float => "Float",
            SortKey::Critical => "Critical",
        }
    }

    /// Fixed pixel width of the column, or `0.0` when it flexes to fill.
    fn width(self) -> f32 {
        match self {
            SortKey::Name => 0.0,
            SortKey::Duration => 60.0,
            SortKey::Start => 96.0,
            SortKey::Finish => 96.0,
            SortKey::Float => 56.0,
            SortKey::Critical => 64.0,
        }
    }
}

/// Which top-level view is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Table,
    Gantt,
    Resources,
    Baselines,
    Calendars,
    Audit,
    Lookahead,
    Diary,
    Production,
}

/// Severity of a `status_message`, used to pick its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusSeverity {
    /// Classify a status message by its wording.
    pub fn from_message(message: &str) -> Self {
        let lower = message.to_ascii_lowercase();
        if lower.contains("cannot")
            || lower.contains("must")
            || lower.contains("select")
            || lower.contains("empty")
            || lower.contains("invalid")
            || lower.contains("needs")
            || lower.contains("not a valid")
        {
            return Self::Error;
        }
        if lower.starts_with("saved")
            || lower.starts_with("opened")
            || lower.starts_with("activity created")
            || lower.starts_with("report")
            || lower.starts_with("gantt printed")
            || lower.starts_with("diary note")
            || lower.starts_with("production entry")
            || lower.starts_with("activity deleted")
        {
            return Self::Success;
        }
        if lower.contains("warning") || lower.contains("warn") {
            return Self::Warning;
        }
        Self::Info
    }

    /// Colour for this severity, taken from the design tokens.
    pub fn color(self, dark: bool) -> Color32 {
        let palette = design::palette(dark);
        match self {
            Self::Info => palette.text_muted,
            Self::Success => palette.ok,
            Self::Warning => palette.warn,
            Self::Error => palette.critical,
        }
    }
}

/// How long an info or success status message stays visible before it is
/// auto-cleared. Errors and warnings persist until dismissed or replaced.
const STATUS_MESSAGE_TTL: Duration = Duration::from_secs(5);

/// Whether a shown status message should be auto-cleared right now.
///
/// Info and success messages expire after the TTL; warnings and errors do
/// not. Pure so it can be unit-tested without an egui context.
fn should_auto_clear_status(severity: StatusSeverity, shown_at: Instant, now: Instant) -> bool {
    matches!(severity, StatusSeverity::Info | StatusSeverity::Success)
        && now.duration_since(shown_at) > STATUS_MESSAGE_TTL
}

/// One row in the diary view: identifier, date, content, activity link and
/// lookahead-window link.
type DiaryRow = (DailyNoteId, Date, String, Option<ActivityId>, Option<Date>);

const CONSTRAINT_LABELS: [&str; 6] = [
    "As soon as possible",
    "As late as possible",
    "Start no earlier than",
    "Start no later than",
    "Finish no earlier than",
    "Finish no later than",
];

/// Labels for the resource kinds, in the order of [`ResourceKind`].
const RESOURCE_KIND_LABELS: [&str; 3] = ["Labour", "Equipment", "Material"];

/// Labels for the four relationship types, in [`LinkType::ALL`] order.
const LINK_TYPE_LABELS: [&str; 4] = [
    "Finish → Start (FS)",
    "Start → Start (SS)",
    "Finish → Finish (FF)",
    "Start → Finish (SF)",
];

/// Convert a relationship type into its index in [`LINK_TYPE_LABELS`].
fn link_type_index(link_type: LinkType) -> usize {
    LinkType::ALL
        .iter()
        .position(|candidate| *candidate == link_type)
        .unwrap_or(0)
}

/// Convert a relationship-type index back into a [`LinkType`].
fn link_type_from_index(index: usize) -> LinkType {
    LinkType::ALL[index.min(LinkType::ALL.len() - 1)]
}

/// The display label for a relationship-type index.
fn link_type_label(index: usize) -> &'static str {
    LINK_TYPE_LABELS[index.min(LINK_TYPE_LABELS.len() - 1)]
}

/// Validate the inspector's predecessor/successor choice.
fn link_endpoints(
    predecessor: Option<ActivityId>,
    successor: Option<ActivityId>,
) -> Result<(ActivityId, ActivityId), String> {
    match (predecessor, successor) {
        (Some(predecessor), Some(successor)) if predecessor != successor => {
            Ok((predecessor, successor))
        }
        (Some(_), Some(_)) => Err("A link needs two different activities".to_string()),
        _ => Err("Choose both a predecessor and a successor".to_string()),
    }
}

/// Convert a resource kind into its index in [`RESOURCE_KIND_LABELS`].
fn resource_kind_index(kind: ResourceKind) -> usize {
    match kind {
        ResourceKind::Labour => 0,
        ResourceKind::Equipment => 1,
        ResourceKind::Material => 2,
    }
}

/// Convert a resource kind index back into a [`ResourceKind`].
fn resource_kind_from_index(index: usize) -> ResourceKind {
    match index {
        1 => ResourceKind::Equipment,
        2 => ResourceKind::Material,
        _ => ResourceKind::Labour,
    }
}

/// One activity's planned-versus-rescheduled finish in the progress preview.
#[derive(Debug, Clone)]
struct RescheduleRow {
    name: String,
    planned_finish: Option<Date>,
    progress_finish: Date,
    variance_days: Option<i64>,
}

/// A non-destructive preview of progress-driven rescheduling.
///
/// The preview never edits the project or its cached schedule: it answers the
/// question "if I rescheduled from recorded progress now, what would move?"
/// without committing an answer. Applying the result is a later slice.
#[derive(Debug, Clone)]
struct ReschedulePreview {
    data_date: Date,
    project_finish: Date,
    not_started: u32,
    in_progress: u32,
    complete: u32,
    rows: Vec<RescheduleRow>,
}

impl ReschedulePreview {
    /// Reschedule the project from recorded progress and collect the result.
    ///
    /// The current `plan` (if any) is the baseline the variance is measured
    /// against: a positive variance means the outstanding work now finishes
    /// later than the plan.
    fn build(project: &Project, plan: Option<&Schedule>, data_date: Date) -> Result<Self, String> {
        let schedule =
            schedule_with_progress(project, data_date).map_err(|error| error.to_string())?;
        let mut not_started = 0u32;
        let mut in_progress = 0u32;
        let mut complete = 0u32;
        let mut rows = Vec::with_capacity(project.activities.len());
        for activity in &project.activities {
            match activity_progress(activity).state {
                ProgressState::NotStarted => not_started += 1,
                ProgressState::InProgress => in_progress += 1,
                ProgressState::Complete => complete += 1,
            }
            let Some(result) = schedule.activity(activity.id) else {
                continue;
            };
            let planned_finish = plan
                .and_then(|plan| plan.activity(activity.id))
                .map(|planned| planned.scheduled_finish);
            let variance_days = planned_finish.map(|planned| {
                project
                    .calendar
                    .working_days_between(planned, result.scheduled_finish)
            });
            rows.push(RescheduleRow {
                name: activity.name.clone(),
                planned_finish,
                progress_finish: result.scheduled_finish,
                variance_days,
            });
        }
        Ok(Self {
            data_date,
            project_finish: schedule.project_finish,
            not_started,
            in_progress,
            complete,
            rows,
        })
    }
}

/// The Cadence application state.
#[allow(dead_code)]
pub struct CadenceApp {
    project: Project,
    /// Path the project was loaded from or last saved to, if any.
    project_path: Option<PathBuf>,
    /// Recently opened project files (most recent first, max 10).
    recent_files: Vec<PathBuf>,
    schedule: Result<Schedule, String>,
    dark: bool,
    stack: CommandStack,
    selected: Option<ActivityId>,
    sort_key: SortKey,
    sort_ascending: bool,
    view_mode: ViewMode,
    pixels_per_day: f32,
    drag_start: Option<usize>,
    status_message: Option<String>,
    dirty: bool,
    confirm_close: bool,
    /// Whether a baseline is available, shown in the status bar.
    baseline_active: bool,
    /// When the current `status_message` was first shown, used to auto-clear
    /// info and success messages after the TTL so they don't linger forever.
    status_shown_at: Option<Instant>,
    /// Earned-value metrics for the most recent baseline, shown in the status bar.
    evm_metrics: Option<EvmMetrics>,
    /// Cached critical-activity count, recomputed once per edit in `recompute`
    //  so the status bar does not rescan every frame.
    critical_count: usize,
    confirm_delete_activity: bool,
    confirm_delete_relationship: bool,
    /// Extra selected rows beyond the primary selection (S2-5): ctrl-click
    /// toggles a row, shift-click selects the range from the anchor. The
    /// primary `selected` row is the anchor and always counts as selected.
    multi_selected: BTreeSet<ActivityId>,
    /// Whether the user has confirmed deleting the multi-selection.
    confirm_delete_selection: bool,
    drag_link_type_index: usize,
    /// Whether the user has confirmed applying leveled dates (destructive).
    confirm_leveling: bool,
    selected_relationship: Option<cadence_core::model::RelationshipId>,
    /// The diary note being edited, if any (S2-10: notes are editable after
    /// creation via the undoable `UpdateDailyNote` command).
    editing_note: Option<DailyNoteId>,
    /// Buffer for the note text while a note is being edited.
    diary_edit_buffer: String,
    lag_buffer: String,
    last_rel: Option<cadence_core::model::RelationshipId>,
    name_buffer: String,
    duration_buffer: String,
    constraint_kind: usize,
    constraint_date_buffer: String,
    progress_buffer: String,
    actual_cost_buffer: String,
    actual_start_buffer: String,
    actual_finish_buffer: String,
    budget_buffer: String,
    selected_resource: Option<ResourceId>,
    resource_name_buffer: String,
    resource_kind_index: usize,
    resource_capacity_buffer: String,
    resource_cost_rate_buffer: String,
    assignment_units_buffers: BTreeMap<(ActivityId, ResourceId), String>,
    leveling_report: Option<String>,
    selected_baseline: Option<BaselineId>,
    baseline_name_buffer: String,
    status_date_buffer: String,
    selected_calendar: Option<CalendarId>,
    calendar_name_buffer: String,
    calendar_weekdays: [bool; 7],
    calendar_holidays_buffer: String,
    gantt_print_path: Option<PathBuf>,
    crash_log: String,
    crash_notes: String,
    show_crash_dialog: bool,
    update_status: update_check::UpdateStatus,
    update_receiver: Option<mpsc::Receiver<update_check::UpdateStatus>>,
    show_update_dialog: bool,
    /// When we last checked for updates (to avoid excessive API calls).
    last_update_check: Option<Instant>,
    link_predecessor: Option<ActivityId>,
    link_successor: Option<ActivityId>,
    link_type_index: usize,
    link_lag_buffer: String,
    show_shortcuts_dialog: bool,
    /// Whether the release notes dialog is open.
    show_release_notes: bool,
    scroll_to_selection: bool,
    audit_report: Option<AuditReport>,
    audit_status_applied: Option<Date>,
    audit_status_buffer: String,
    audit_version: u64,
    edit_version: u64,
    lookahead_start_buffer: String,
    lookahead_weeks: usize,
    lookahead_snapshot_name_buffer: String,
    selected_lookahead_snapshot: Option<LookaheadSnapshotId>,
    diary_date_buffer: String,
    diary_content_buffer: String,
    diary_activity_buffer: String,
    diary_window_buffer: String,
    production_activity_buffer: String,
    production_date_buffer: String,
    production_percent: i64,
    production_percent_buffer: String,
    reschedule_date_buffer: String,
    reschedule_preview: Option<ReschedulePreview>,
    owner_buffer: String,
    promised_buffer: bool,
    delay_reason_buffer: String,
    /// Resource conflicts against the current schedule, rebuilt in `recompute`
    /// against the edit version so the Resources view never recomputes per
    /// frame. Shared behind an `Arc` because the view's closures borrow `self`
    /// mutably and need an owned handle to the data.
    conflicts_cache: Arc<Vec<ResourceConflict>>,
    conflicts_version: u64,
    /// The resource-day histogram for the current schedule, rebuilt in
    /// `recompute` for the same reason; `resource_day_usage` filters it rather
    /// than rebuilding it per frame.
    histogram_cache: Arc<ResourceHistogram>,
    histogram_version: u64,
    /// The cached lookahead window, keyed by the edit version and the window
    /// (start date, weeks) the user has chosen, so panning the window or
    /// editing the programme is the only thing that rebuilds it.
    lookahead_cache: Option<Arc<Vec<LookaheadEntry>>>,
    lookahead_version: u64,
    lookahead_window: Option<(Date, usize)>,
    sorted_cache: Vec<Activity>,
    sorted_edit_version: u64,
    sorted_key: SortKey,
    sorted_ascending: bool,
    /// Filter text typed into the table view (case-insensitive substring match
    /// on the activity name; empty means "show everything").
    table_filter_buffer: String,
    /// When set, the table shows only activities on the critical path.
    table_critical_only: bool,
    /// The filtered table rows, materialised only while a filter is active.
    table_rows: Vec<Activity>,
    /// Whether `table_rows` currently holds a filtered subset.
    table_rows_active: bool,
    /// Cache keys for `table_rows`: rebuilt when any of these change.
    table_rows_edit_version: u64,
    table_rows_filter: String,
    table_rows_critical_only: bool,
    table_rows_source_len: usize,
    table_rows_sorted_key: SortKey,
    table_rows_sorted_ascending: bool,
    show_split_gantt: bool,
    visible_columns: BTreeSet<SortKey>,
}

/// Whether an activity passes the table filter: a case-insensitive substring
/// match on the name (an empty needle matches everything), and — when
/// `critical_only` is set — only activities on the critical path.
fn activity_matches_filter(
    activity: &Activity,
    needle: &str,
    critical_only: bool,
    schedule: Option<&Schedule>,
) -> bool {
    (needle.is_empty() || activity.name.to_ascii_lowercase().contains(needle))
        && (!critical_only
            || schedule
                .and_then(|s| s.activity(activity.id))
                .map(|r| r.critical)
                .unwrap_or(false))
}

impl CadenceApp {
    /// Return the design tokens for the current theme.
    fn design_tokens(&self) -> DesignTokens {
        DesignTokens::for_dark(self.dark)
    }

    /// Build the application from a loaded project.
    pub fn new(project: Project) -> Self {
        Self::with_path(project, None)
    }

    /// Build the application from a loaded project and the file it came from.
    pub fn with_path(project: Project, path: Option<PathBuf>) -> Self {
        let schedule = schedule(&project).map_err(|error| error.to_string());
        let crash_log = crash::read_log().unwrap_or_default();
        let show_crash_dialog = !crash_log.is_empty();
        let update_status =
            update_check::cached_status().unwrap_or(update_check::UpdateStatus::NotChecked);
        Self {
            project,
            project_path: path,
            recent_files: Vec::new(),
            schedule,
            dark: true,
            stack: CommandStack::new(),
            selected: None,
            sort_key: SortKey::Name,
            sort_ascending: true,
            view_mode: ViewMode::Table,
            pixels_per_day: 14.0,
            drag_start: None,
            status_message: None,
            status_shown_at: None,
            dirty: false,
            confirm_close: false,
            baseline_active: false,
            evm_metrics: None,
            critical_count: 0,
            confirm_delete_activity: false,
            confirm_delete_relationship: false,
            multi_selected: BTreeSet::new(),
            confirm_delete_selection: false,
            drag_link_type_index: 0,
            confirm_leveling: false,
            selected_relationship: None,
            editing_note: None,
            diary_edit_buffer: String::new(),
            lag_buffer: String::new(),
            last_rel: None,
            name_buffer: String::new(),
            duration_buffer: String::new(),
            constraint_kind: 0,
            constraint_date_buffer: String::new(),
            progress_buffer: String::new(),
            actual_cost_buffer: String::new(),
            actual_start_buffer: String::new(),
            actual_finish_buffer: String::new(),
            budget_buffer: String::new(),
            selected_resource: None,
            resource_name_buffer: String::new(),
            resource_kind_index: 0,
            resource_capacity_buffer: "1".to_string(),
            resource_cost_rate_buffer: "0".to_string(),
            assignment_units_buffers: BTreeMap::new(),
            leveling_report: None,
            selected_baseline: None,
            baseline_name_buffer: String::new(),
            status_date_buffer: today().to_string(),
            selected_calendar: None,
            calendar_name_buffer: String::new(),
            calendar_weekdays: [true, true, true, true, true, false, false],
            calendar_holidays_buffer: String::new(),
            gantt_print_path: None,
            crash_log,
            crash_notes: String::new(),
            show_crash_dialog,
            update_status,
            update_receiver: None,
            show_update_dialog: false,
            link_predecessor: None,
            link_successor: None,
            link_type_index: 0,
            link_lag_buffer: "0".to_string(),
            show_shortcuts_dialog: false,
            show_release_notes: false,
            last_update_check: None,
            scroll_to_selection: false,
            audit_report: None,
            audit_status_applied: None,
            audit_status_buffer: String::new(),
            audit_version: 0,
            edit_version: 0,
            lookahead_start_buffer: today().to_string(),
            lookahead_weeks: 2,
            lookahead_snapshot_name_buffer: String::new(),
            selected_lookahead_snapshot: None,
            diary_date_buffer: today().to_string(),
            diary_content_buffer: String::new(),
            diary_activity_buffer: String::new(),
            diary_window_buffer: String::new(),
            production_activity_buffer: String::new(),
            production_date_buffer: today().to_string(),
            production_percent: 0,
            production_percent_buffer: "0".to_string(),
            reschedule_date_buffer: today().to_string(),
            reschedule_preview: None,
            owner_buffer: String::new(),
            promised_buffer: false,
            delay_reason_buffer: String::new(),
            conflicts_cache: Arc::new(Vec::new()),
            conflicts_version: u64::MAX,
            histogram_cache: Arc::new(ResourceHistogram {
                entries: BTreeMap::new(),
            }),
            histogram_version: u64::MAX,
            lookahead_cache: None,
            lookahead_version: u64::MAX,
            lookahead_window: None,
            sorted_cache: Vec::new(),
            sorted_edit_version: u64::MAX,
            sorted_key: SortKey::Name,
            sorted_ascending: true,
            table_filter_buffer: String::new(),
            table_critical_only: false,
            table_rows: Vec::new(),
            table_rows_active: false,
            table_rows_edit_version: u64::MAX,
            table_rows_filter: String::new(),
            table_rows_critical_only: false,
            table_rows_source_len: usize::MAX,
            table_rows_sorted_key: SortKey::Name,
            table_rows_sorted_ascending: true,
            show_split_gantt: true,
            visible_columns: SortKey::all_columns(),
        }
    }

    /// Start a background update check unless one is already running.
    fn start_update_check(&mut self) {
        if self.update_receiver.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let status = update_check::check_for_update();
            let _ = sender.send(status);
        });
        self.update_receiver = Some(receiver);
    }

    /// Record a status message, stamping when it was first shown so info and
    /// success messages can be auto-cleared after the TTL.
    fn set_status(&mut self, text: impl Into<String>) {
        self.status_message = Some(text.into());
        self.status_shown_at = Some(Instant::now());
    }

    /// Clear an info/success status message once its TTL has elapsed. Errors
    /// and warnings persist until dismissed or replaced.
    fn maybe_clear_status(&mut self, now: Instant) {
        let should_clear = self.status_message.as_ref().is_some_and(|message| {
            let severity = StatusSeverity::from_message(message);
            self.status_shown_at
                .is_some_and(|shown_at| should_auto_clear_status(severity, shown_at, now))
        });
        if should_clear {
            self.status_message = None;
            self.status_shown_at = None;
        }
    }

    fn recompute(&mut self) {
        self.edit_version = self.edit_version.wrapping_add(1);
        self.schedule = schedule(&self.project).map_err(|error| error.to_string());
        self.dirty = true;
        // Refresh the baseline flag and earned-value metrics the status bar
        // shows, so they follow edits and undo/redo without recomputing per
        // frame.
        self.baseline_active = self.project.baselines.last().is_some();
        // Drop multi-selection rows whose activities no longer exist, so the
        // range selection can never hold a dangling id (S2-5).
        self.multi_selected
            .retain(|id| self.project.activity(*id).is_some());
        self.evm_metrics = self
            .project
            .baselines
            .last()
            .map(|baseline| evm_metrics(&self.project, Some(baseline), today()));
        // Cache the critical-activity count so the status bar does not rescan
        // every frame.
        let schedule = match self.schedule.as_ref() {
            Ok(schedule) => schedule,
            Err(_) => {
                self.critical_count = 0;
                // An unschedulable programme has no conflicts, no usage and no
                // lookahead; clear the caches so the views reflect that.
                self.conflicts_cache = Arc::new(Vec::new());
                self.histogram_cache = Arc::new(ResourceHistogram {
                    entries: BTreeMap::new(),
                });
                self.conflicts_version = self.edit_version;
                self.histogram_version = self.edit_version;
                return;
            }
        };
        self.critical_count = self
            .project
            .activities
            .iter()
            .filter(|activity| {
                schedule
                    .activity(activity.id)
                    .is_some_and(|result| result.critical)
            })
            .count();
        // Cache the resource conflicts and the day histogram once per edit, so
        // the Resources view does not rescan the assignments every frame
        // (S2-11). Shared behind an `Arc`; the views clone the handle.
        self.conflicts_cache = Arc::new(resource_conflicts(&self.project, schedule));
        self.histogram_cache = Arc::new(resource_histogram(&self.project, schedule));
        self.conflicts_version = self.edit_version;
        self.histogram_version = self.edit_version;
    }

    /// Reload the inspector buffers for the selected activity after an undo or
    /// redo, so the fields never show a value that is no longer in the model.
    fn refresh_selection(&mut self) {
        match self.selected {
            Some(id) if self.project.activity(id).is_some() => self.select(id),
            _ => self.selected = None,
        }
    }

    /// Run the programme audit with the applied status date.
    ///
    /// The audit is pure and cheap, but it is not free on a 10,000-activity
    /// programme, so it runs when the Audit view is shown or its data changed
    /// (tracked by the edit version), never per frame.
    fn run_audit(&mut self) {
        let Some(schedule) = self.schedule.clone().ok() else {
            self.audit_report = None;
            self.audit_version = self.edit_version;
            return;
        };
        let config = AuditConfig {
            status_date: self.audit_status_applied,
            ..AuditConfig::default()
        };
        let report = audit(
            &self.project,
            &schedule,
            self.project.baselines.last(),
            &config,
        );
        self.audit_report = Some(report);
        self.audit_version = self.edit_version;
    }

    fn select(&mut self, id: ActivityId) {
        self.selected = Some(id);
        if let Some(activity) = self.project.activity(id) {
            self.name_buffer = activity.name.clone();
            self.duration_buffer = activity.original_duration.to_string();
            let (kind, date) = match activity.constraint {
                Constraint::AsSoonAsPossible => (0, None),
                Constraint::AsLateAsPossible => (1, None),
                Constraint::StartNoEarlierThan(date) => (2, Some(date)),
                Constraint::StartNoLaterThan(date) => (3, Some(date)),
                Constraint::FinishNoEarlierThan(date) => (4, Some(date)),
                Constraint::FinishNoLaterThan(date) => (5, Some(date)),
            };
            self.constraint_kind = kind;
            self.constraint_date_buffer = date.map(|d| d.to_string()).unwrap_or_default();
            self.progress_buffer = activity
                .percent_complete
                .map(|percent| (percent as f64 / 100.0).to_string())
                .unwrap_or_default();
            self.actual_cost_buffer = activity.actual_cost.map(format_money).unwrap_or_default();
            self.actual_start_buffer = activity
                .actual_start
                .map(|date| date.to_string())
                .unwrap_or_default();
            self.actual_finish_buffer = activity
                .actual_finish
                .map(|date| date.to_string())
                .unwrap_or_default();
            self.budget_buffer = activity
                .budget_at_completion
                .map(format_money)
                .unwrap_or_default();
        }
        self.link_predecessor = Some(id);
        if self.link_successor == Some(id) {
            self.link_successor = None;
        }
        if let Some(activity) = self.project.activity(id) {
            self.owner_buffer = activity.owner.clone().unwrap_or_default();
            self.promised_buffer = activity.promised;
            self.delay_reason_buffer = activity.delay_reason.clone().unwrap_or_default();
        }
    }

    /// Apply the field-planning buffers through the command layer.
    ///
    /// The owner and the promise/delay-reason are two separate commands so
    /// each undoes on its own; moving a promised activity later still asks
    /// for the reason in the delay-reason field, which the promise command
    /// records.
    fn apply_field_planning(&mut self, id: ActivityId) {
        let Some(activity) = self.project.activity(id).cloned() else {
            return;
        };
        let owner_text = self.owner_buffer.trim();
        let owner = if owner_text.is_empty() {
            None
        } else {
            Some(owner_text.to_string())
        };
        let reason = self.delay_reason_buffer.trim();
        let delay_reason = if reason.is_empty() {
            None
        } else {
            Some(reason.to_string())
        };

        // Only push history for the fields that actually changed, and report a
        // rejected command instead of dropping it silently.
        if owner != activity.owner {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(SetActivityOwner { id, owner }))
            {
                self.set_status(error);
                return;
            }
        }
        if self.promised_buffer != activity.promised || delay_reason != activity.delay_reason {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityPromise {
                    id,
                    promised: self.promised_buffer,
                    delay_reason,
                }),
            ) {
                self.set_status(error);
                return;
            }
        }
        self.recompute();
    }

    /// Add the rows between the anchor and `end` (inclusive) to the
    /// multi-selection, then move the anchor to `end` (S2-5).
    fn select_range(&mut self, end: ActivityId) {
        let ids: Vec<ActivityId> = self.table_rows().iter().map(|row| row.id).collect();
        let Some(anchor) = self.selected else {
            self.select(end);
            return;
        };
        let Some(from) = ids.iter().position(|id| *id == anchor) else {
            self.select(end);
            return;
        };
        let Some(to) = ids.iter().position(|id| *id == end) else {
            self.select(end);
            return;
        };
        let (low, high) = if from <= to { (from, to) } else { (to, from) };
        for id in &ids[low..=high] {
            self.multi_selected.insert(*id);
        }
        self.select(end);
    }

    /// Add a row to the multi-selection, or remove it if it is already there.
    fn toggle_multi_select(&mut self, id: ActivityId) {
        if self.multi_selected.contains(&id) {
            self.multi_selected.remove(&id);
        } else {
            self.multi_selected.insert(id);
        }
    }

    /// Delete the multi-selection as one undoable step.
    fn delete_selection(&mut self) {
        let ids: Vec<ActivityId> = self.multi_selected.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let command = cadence_commands::DeleteActivities { ids };
        if self
            .stack
            .execute(&mut self.project, Box::new(command))
            .is_ok()
        {
            self.recompute();
            if self
                .selected
                .is_some_and(|id| self.project.activity(id).is_none())
            {
                self.selected = None;
            }
            self.multi_selected.clear();
            self.refresh_selection();
            self.set_status("Selected activities deleted.");
        }
    }

    /// Change an existing relationship's type, issuing the undoable command
    /// and recomputing the schedule (S2-8).
    fn set_relationship_type(
        &mut self,
        id: cadence_core::model::RelationshipId,
        link_type: LinkType,
    ) {
        let _ = self.stack.execute(
            &mut self.project,
            Box::new(SetRelationshipType { id, link_type }),
        );
        self.recompute();
    }

    /// Add a dependency link from the inspector form.
    ///
    /// This is the pointer-free equivalent of dragging between two Gantt bars.
    fn add_link_from_form(&mut self) {
        let endpoints = link_endpoints(self.link_predecessor, self.link_successor);
        let (predecessor, successor) = match endpoints {
            Ok(pair) => pair,
            Err(message) => {
                self.set_status(message);
                return;
            }
        };
        let Ok(lag) = self.link_lag_buffer.trim().parse::<i64>() else {
            self.set_status("Lag must be a whole number of working days");
            return;
        };
        let link_type = link_type_from_index(self.link_type_index);
        let id = self.project.next_relationship_id();
        match self.stack.execute(
            &mut self.project,
            Box::new(CreateRelationship::new(
                id,
                predecessor,
                successor,
                link_type,
                lag,
            )),
        ) {
            Ok(()) => {
                self.recompute();
                match self.schedule.clone() {
                    Err(error) => {
                        let _ = self.stack.undo(&mut self.project);
                        self.recompute();
                        self.set_status(format!("Cannot add link: {error}"));
                    }
                    Ok(_) => {
                        self.selected_relationship = Some(id);
                        self.last_rel = None;
                        self.set_status(format!(
                            "Added {} link {} → {}",
                            link_type, predecessor, successor
                        ));
                    }
                }
            }
            Err(error) => {
                self.set_status(format!("Cannot add link: {error}"));
            }
        }
    }

    fn apply_edits(&mut self) {
        let Some(id) = self.selected else {
            return;
        };
        let Some(activity) = self.project.activity(id).cloned() else {
            return;
        };

        if self.name_buffer.trim().is_empty() {
            self.set_status("The activity name must not be empty.");
            return;
        }
        if self.name_buffer != activity.name {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(RenameActivity {
                    id,
                    name: self.name_buffer.clone(),
                }),
            ) {
                self.set_status(error);
                return;
            }
        }

        match self.duration_buffer.trim() {
            "" => {}
            text => match text.parse::<u32>() {
                Ok(duration) => {
                    if duration != activity.original_duration {
                        if let Err(error) = self.stack.execute(
                            &mut self.project,
                            Box::new(SetActivityDuration { id, duration }),
                        ) {
                            self.set_status(error);
                            return;
                        }
                    }
                }
                Err(_) => {
                    self.set_status("Duration must be a whole number of working days.");
                    return;
                }
            },
        }

        let constraint = match self.constraint_kind {
            0 => Constraint::AsSoonAsPossible,
            1 => Constraint::AsLateAsPossible,
            kind => {
                let Some(date) = parse_date(self.constraint_date_buffer.trim()) else {
                    self.set_status("A date constraint needs a date in YYYY-MM-DD format.");
                    return;
                };
                match kind {
                    2 => Constraint::StartNoEarlierThan(date),
                    3 => Constraint::StartNoLaterThan(date),
                    4 => Constraint::FinishNoEarlierThan(date),
                    _ => Constraint::FinishNoLaterThan(date),
                }
            }
        };

        if constraint != activity.constraint {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityConstraint { id, constraint }),
            ) {
                self.set_status(error);
                return;
            }
        }

        let percent = match parse_percent(self.progress_buffer.trim()) {
            Some(percent) => percent,
            None => {
                self.set_status("Progress must be a percentage between 0 and 100.");
                return;
            }
        };
        if percent != activity.percent_complete {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityProgress {
                    id,
                    percent_complete: percent,
                }),
            ) {
                self.set_status(error);
                return;
            }
        }

        let budget = match parse_optional_money(self.budget_buffer.trim()) {
            Some(value) => value,
            None => {
                self.set_status("Budget must be a number.");
                return;
            }
        };
        if budget != activity.budget_at_completion {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityBudget {
                    id,
                    budget_at_completion: budget,
                }),
            ) {
                self.set_status(error);
                return;
            }
        }

        let cost = match parse_optional_money(self.actual_cost_buffer.trim()) {
            Some(value) => value,
            None => {
                self.set_status("Actual cost must be a number.");
                return;
            }
        };
        if cost != activity.actual_cost {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityActualCost {
                    id,
                    actual_cost: cost,
                }),
            ) {
                self.set_status(error);
                return;
            }
        }

        let actual_start = match parse_optional_date(self.actual_start_buffer.trim()) {
            Some(date) => date,
            None => {
                self.set_status("Actual start must be a date in YYYY-MM-DD format.");
                return;
            }
        };
        let actual_finish = match parse_optional_date(self.actual_finish_buffer.trim()) {
            Some(date) => date,
            None => {
                self.set_status("Actual finish must be a date in YYYY-MM-DD format.");
                return;
            }
        };
        if actual_start != activity.actual_start || actual_finish != activity.actual_finish {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityActualDates {
                    id,
                    actual_start,
                    actual_finish,
                }),
            ) {
                self.set_status(error);
                return;
            }
        }

        self.recompute();
        self.select(id);
    }

    /// Load the resource editor buffers for the given resource.
    fn select_resource(&mut self, id: ResourceId) {
        self.selected_resource = Some(id);
        if let Some(resource) = self.project.resource(id) {
            self.resource_name_buffer = resource.name.clone();
            self.resource_capacity_buffer = resource.capacity.to_string();
            self.resource_cost_rate_buffer = format_money(resource.cost_per_unit);
            self.resource_kind_index = resource_kind_index(resource.kind);
        }
    }

    /// Apply the resource editor buffers through the command stack.
    fn apply_resource_edits(&mut self) {
        let Some(id) = self.selected_resource else {
            return;
        };
        let Some(resource) = self.project.resource(id).cloned() else {
            return;
        };

        let name = self.resource_name_buffer.trim().to_string();
        if !name.is_empty() && name != resource.name {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(RenameResource { id, name }))
            {
                self.set_status(error);
            }
        }

        if let Ok(capacity) = self.resource_capacity_buffer.trim().parse::<u32>() {
            if capacity != resource.capacity {
                if let Err(error) = self.stack.execute(
                    &mut self.project,
                    Box::new(SetResourceCapacity { id, capacity }),
                ) {
                    self.set_status(error);
                }
            }
        }

        // The rate is typed as a decimal amount, like every other money field.
        if let Some(cost_rate) = parse_money(self.resource_cost_rate_buffer.trim()) {
            if cost_rate != resource.cost_per_unit {
                if let Err(error) = self.stack.execute(
                    &mut self.project,
                    Box::new(SetResourceCostRate {
                        id,
                        cost_per_unit: cost_rate,
                    }),
                ) {
                    self.set_status(error);
                }
            }
        }

        let kind = resource_kind_from_index(self.resource_kind_index);
        if kind != resource.kind {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(SetResourceKind { id, kind }))
            {
                self.set_status(error);
            }
        }

        self.recompute();
        self.select_resource(id);
    }

    /// Add a new resource to the dictionary.
    fn create_resource(&mut self) {
        let name = self.resource_name_buffer.trim().to_string();
        if name.is_empty() {
            self.set_status("Resource name must not be empty");
            return;
        }
        let id = self.project.next_resource_id();
        let command = CreateResource {
            id,
            name,
            kind: resource_kind_from_index(self.resource_kind_index),
            capacity: self
                .resource_capacity_buffer
                .trim()
                .parse::<u32>()
                .unwrap_or(1)
                .max(1),
        };
        match self.stack.execute(&mut self.project, Box::new(command)) {
            Ok(()) => {
                self.status_message = None;
                // A non-zero rate typed alongside the new resource is applied
                // with a second command, so it stays undoable on its own.
                if let Some(rate) = parse_money(self.resource_cost_rate_buffer.trim()) {
                    if rate > 0 {
                        if let Err(error) = self.stack.execute(
                            &mut self.project,
                            Box::new(SetResourceCostRate {
                                id,
                                cost_per_unit: rate,
                            }),
                        ) {
                            self.set_status(error);
                        }
                    }
                }
                self.recompute();
                self.select_resource(id);
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Create a new activity and select it (used by the toolbar button).
    fn create_activity(&mut self) {
        let id = self.project.next_activity_id();
        let command = cadence_commands::CreateActivity::new(id, "New activity", 1);
        if self
            .stack
            .execute(&mut self.project, Box::new(command))
            .is_ok()
        {
            self.recompute();
            self.select(id);
            self.scroll_to_selection = true;
            self.set_status("Activity created.");
        }
    }

    /// Load the calendar editor buffers for the given calendar.
    fn select_calendar(&mut self, id: CalendarId) {
        self.selected_calendar = Some(id);
        if let Some(named) = self.project.calendars.named(id) {
            self.calendar_name_buffer = named.name.clone();
            self.calendar_weekdays = named.calendar.working_weekdays();
            self.calendar_holidays_buffer = named
                .calendar
                .holidays()
                .iter()
                .map(|date| date.to_string())
                .collect::<Vec<_>>()
                .join(", ");
        }
    }

    /// Create a calendar from the editor buffers.
    fn create_calendar(&mut self) {
        let name = self.calendar_name_buffer.trim().to_string();
        if name.is_empty() {
            self.set_status("Calendar name must not be empty");
            return;
        }
        let Some(calendar) = self.build_calendar() else {
            return;
        };
        let id = self.project.calendars.next_id();
        let command = CreateCalendar::new(id, name, calendar);
        match self.stack.execute(&mut self.project, Box::new(command)) {
            Ok(()) => {
                self.status_message = None;
                self.select_calendar(id);
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Apply the editor buffers to the selected calendar.
    fn apply_calendar_edits(&mut self) {
        let Some(id) = self.selected_calendar else {
            return;
        };
        let Some(named) = self.project.calendars.named(id).cloned() else {
            return;
        };

        let mut changed = false;
        let name = self.calendar_name_buffer.trim().to_string();
        if !name.is_empty() && name != named.name {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(RenameCalendar { id, name }))
            {
                self.set_status(error);
            } else {
                changed = true;
            }
        }

        let Some(calendar) = self.build_calendar() else {
            return;
        };
        if calendar != named.calendar {
            if let Err(error) = self.stack.execute(
                &mut self.project,
                Box::new(SetCalendarDefinition { id, calendar }),
            ) {
                self.set_status(error);
            } else {
                changed = true;
            }
        }

        if changed {
            // A working-day change moves every activity that uses the
            // calendar, so the schedule must be recomputed.
            self.recompute();
        }
        self.select_calendar(id);
    }

    /// Delete the selected calendar, refusing while an activity uses it.
    fn delete_selected_calendar(&mut self) {
        let Some(id) = self.selected_calendar else {
            return;
        };
        if self
            .project
            .activities
            .iter()
            .any(|activity| activity.calendar == Some(id))
        {
            self.set_status(format!(
                "{} activity(ies) use this calendar — clear their calendar first",
                self.project
                    .activities
                    .iter()
                    .filter(|activity| activity.calendar == Some(id))
                    .count()
            ));
            return;
        }
        match self
            .stack
            .execute(&mut self.project, Box::new(DeleteCalendar { id }))
        {
            Ok(()) => {
                self.selected_calendar = None;
                self.status_message = None;
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Build a calendar from the editor buffers, reporting why it is invalid.
    fn build_calendar(&mut self) -> Option<Calendar> {
        if !self.calendar_weekdays.iter().any(|working| *working) {
            self.set_status("A calendar needs at least one working day");
            return None;
        }
        let mut calendar = Calendar::new(self.calendar_weekdays).ok()?;
        for entry in self.calendar_holidays_buffer.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some(date) = parse_date(trimmed) else {
                self.set_status(format!("\"{trimmed}\" is not a valid date"));
                return None;
            };
            calendar = calendar.with_holiday(date);
        }
        Some(calendar)
    }

    /// Remove the selected resource from the dictionary.
    fn delete_selected_resource(&mut self) {
        let Some(id) = self.selected_resource else {
            return;
        };
        match self
            .stack
            .execute(&mut self.project, Box::new(DeleteResource { id }))
        {
            Ok(()) => {
                self.selected_resource = None;
                self.status_message = None;
                self.recompute();
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Assign the given resource to the selected activity, or remove it.
    fn toggle_assignment(&mut self, resource_id: ResourceId) {
        let Some(activity_id) = self.selected else {
            self.set_status("Select an activity first");
            return;
        };
        let assigned = self
            .project
            .activity(activity_id)
            .map(|activity| activity.resources.contains(&resource_id))
            .unwrap_or(false);

        let outcome = if assigned {
            self.stack.execute(
                &mut self.project,
                Box::new(RemoveResourceFromActivity {
                    activity_id,
                    resource_id,
                }),
            )
        } else {
            self.stack.execute(
                &mut self.project,
                Box::new(AddResourceToActivity {
                    activity_id,
                    resource_id,
                }),
            )
        };

        match outcome {
            Ok(()) => {
                self.status_message = None;
                self.recompute();
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Compute the leveled schedule and report what it would change.
    fn preview_leveling(&mut self) {
        let Some(schedule) = self.schedule.clone().ok() else {
            self.leveling_report = Some("The project cannot be scheduled".to_string());
            return;
        };
        match level_resources(&self.project, &schedule) {
            Ok(result) => {
                let delayed: Vec<&cadence_core::engine::LeveledActivity> = result
                    .activities
                    .iter()
                    .filter(|activity| activity.delay > 0)
                    .collect();
                if !result.conflicts.is_empty() {
                    self.leveling_report = Some(format!(
                        "{} over-allocated days remain after leveling",
                        result.conflicts.len()
                    ));
                } else if delayed.is_empty() {
                    self.leveling_report =
                        Some("The schedule is already free of over-allocation".to_string());
                } else {
                    self.leveling_report = Some(format!(
                        "Leveling delays {} activities, for example {}",
                        delayed.len(),
                        describe_delay(delayed[0])
                    ));
                }
            }
            Err(error) => self.leveling_report = Some(format!("Leveling failed: {error}")),
        }
    }

    /// Apply the leveled dates as start-no-earlier-than constraints.
    fn apply_leveling(&mut self) {
        let Some(schedule) = self.schedule.clone().ok() else {
            self.set_status("The project cannot be scheduled");
            return;
        };
        let Ok(result) = level_resources(&self.project, &schedule) else {
            self.set_status("The project cannot be leveled");
            return;
        };

        // Atomic leveling (S2-15): a single SetConstraints command applies
        // every delayed activity's new constraint all-or-nothing, so undo
        // reverts the whole batch in one step.
        let values: Vec<(ActivityId, Constraint)> = result
            .activities
            .iter()
            .filter(|activity| activity.delay > 0)
            .map(|activity| (activity.id, Constraint::StartNoEarlierThan(activity.start)))
            .collect();
        let applied = values.len();
        match self.stack.execute(
            &mut self.project,
            Box::new(cadence_commands::SetConstraints { values }),
        ) {
            Ok(()) => {
                self.recompute();
                self.leveling_report = Some(format!("Applied leveling to {applied} activities"));
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Capture the current schedule as a new baseline.
    fn capture_baseline_now(&mut self) {
        let typed = self.baseline_name_buffer.trim().to_string();
        let name = if typed.is_empty() {
            format!("Baseline {}", self.project.baselines.len() + 1)
        } else {
            typed
        };
        let command = CaptureBaseline {
            name,
            created_at: today(),
        };
        match self.stack.execute(&mut self.project, Box::new(command)) {
            Ok(()) => {
                self.status_message = None;
                self.recompute();
                self.selected_baseline = self.project.baselines.last().map(|baseline| baseline.id);
                self.baseline_name_buffer.clear();
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Print the project report to a PDF chosen with a native save dialog.
    ///
    /// Cancelling the dialog changes nothing.
    fn print_report_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF document", &["pdf"])
            .set_file_name(format!("{}.pdf", file_stem(&self.project.name)))
            .save_file()
        else {
            return;
        };
        let pdf = cadence_core::report::project_report_pdf(&self.project, today());
        match std::fs::write(&path, pdf) {
            Ok(()) => {
                self.set_status(format!("Report printed to {}", path.display()));
            }
            Err(error) => {
                self.set_status(format!("Cannot write {}: {error}", path.display()));
            }
        }
    }

    /// Write the Markdown report next to the working directory.
    ///
    /// The report itself needs no dialog; "Print report…" is the one that
    /// opens one, for choosing where a PDF goes.
    /// Save the project to its known path, or prompt for one.
    fn save_or_prompt(&mut self) {
        match self.project_path.clone() {
            Some(path) => self.save_to(path),
            None => self.save_as_dialog(),
        }
    }

    /// Write the project to `path` and remember it as the current file.
    fn save_to(&mut self, path: PathBuf) {
        match cadence_persistence::save_project(&self.project, &path) {
            Ok(()) => {
                self.set_status(format!("Saved to {}", path.display()));
                self.project_path = Some(path);
                self.dirty = false;
            }
            Err(error) => {
                self.set_status(format!("Cannot save: {error}"));
            }
        }
    }

    /// Prompt for a path and save there.
    fn save_as_dialog(&mut self) {
        let file_name = self
            .project_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{}.cadence", file_stem(&self.project.name)));
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Cadence project", &["cadence"])
            .set_file_name(file_name)
            .save_file()
        {
            self.save_to(path.clone());
            self.add_to_recent_files(path);
        }
    }

    /// Prompt for a project file and open it.
    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Cadence project", &["cadence"])
            .pick_file()
        {
            self.open_from(path);
        }
    }

    /// Load a project from `path`, replacing the current one.
    fn open_from(&mut self, path: PathBuf) {
        match cadence_persistence::load_project(&path) {
            Ok(project) => {
                self.project = project;
                self.project_path = Some(path.clone());
                self.add_to_recent_files(path.clone());
                self.stack = CommandStack::new();
                self.selected = None;
                self.selected_relationship = None;
                self.selected_resource = None;
                self.selected_baseline = None;
                self.selected_lookahead_snapshot = None;
                self.recompute();
                self.dirty = false;
                self.set_status(format!("Opened {}", path.display()));
            }
            Err(error) => {
                self.set_status(format!("Cannot open: {error}"));
            }
        }
    }

    /// Add a file to the recent files list, keeping only the 10 most recent.
    fn add_to_recent_files(&mut self, path: PathBuf) {
        // Remove if already present
        self.recent_files.retain(|p| p != &path);
        // Insert at front
        self.recent_files.insert(0, path);
        // Keep only 10 most recent
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    fn save_report(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown document", &["md"])
            .set_file_name(format!("{}.report.md", file_stem(&self.project.name)))
            .save_file()
        else {
            return;
        };
        let report = cadence_core::report::project_report(&self.project, today());
        match std::fs::write(&path, report) {
            Ok(()) => self.set_status(format!("Report written to {}", path.display())),
            Err(error) => self.set_status(format!("Cannot write {}: {error}", path.display())),
        }
    }

    /// Remove the selected baseline from the project.
    fn delete_selected_baseline(&mut self) {
        let Some(id) = self.selected_baseline else {
            return;
        };
        match self
            .stack
            .execute(&mut self.project, Box::new(RemoveBaseline { id }))
        {
            Ok(()) => {
                self.selected_baseline = None;
                self.status_message = None;
            }
            Err(error) => self.set_status(error),
        }
    }

    /// Rebuild the sorted snapshot only when the data or sort order changed.
    ///
    /// The table used to clone and sort the whole programme every frame; on a
    /// 10,000-activity programme that alone dominated the frame time. The
    /// snapshot is now rebuilt on edits and sort changes only.
    fn refresh_sorted(&mut self) {
        if self.sorted_edit_version == self.edit_version
            && self.sorted_key == self.sort_key
            && self.sorted_ascending == self.sort_ascending
            && self.sorted_cache.len() == self.project.activities.len()
        {
            return;
        }
        let mut activities = self.project.activities.clone();
        let schedule = self.schedule.as_ref().ok();
        let sort_key = self.sort_key;
        let ascending = self.sort_ascending;
        activities.sort_by(|left, right| {
            let ordering = match sort_key {
                SortKey::Name => left.name.cmp(&right.name),
                SortKey::Duration => left.original_duration.cmp(&right.original_duration),
                SortKey::Start => {
                    let left_start = schedule
                        .and_then(|s| s.activity(left.id))
                        .map(|r| r.scheduled_start);
                    let right_start = schedule
                        .and_then(|s| s.activity(right.id))
                        .map(|r| r.scheduled_start);
                    left_start.cmp(&right_start)
                }
                SortKey::Finish => {
                    let left_finish = schedule
                        .and_then(|s| s.activity(left.id))
                        .map(|r| r.scheduled_finish);
                    let right_finish = schedule
                        .and_then(|s| s.activity(right.id))
                        .map(|r| r.scheduled_finish);
                    left_finish.cmp(&right_finish)
                }
                SortKey::Float => {
                    let left_float = schedule
                        .and_then(|s| s.activity(left.id))
                        .map(|r| r.total_float);
                    let right_float = schedule
                        .and_then(|s| s.activity(right.id))
                        .map(|r| r.total_float);
                    left_float.cmp(&right_float)
                }
                SortKey::Critical => {
                    let left_critical = schedule
                        .and_then(|s| s.activity(left.id))
                        .map(|r| r.critical);
                    let right_critical = schedule
                        .and_then(|s| s.activity(right.id))
                        .map(|r| r.critical);
                    left_critical.cmp(&right_critical)
                }
            };
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        self.sorted_cache = activities;
        self.sorted_edit_version = self.edit_version;
        self.sorted_key = self.sort_key;
        self.sorted_ascending = self.sort_ascending;
    }

    /// The cached sorted activities (the full programme, in row order).
    fn sorted(&self) -> &[Activity] {
        &self.sorted_cache
    }

    /// Rebuild the filtered table rows when the filter, the sort order or the
    /// programme changed.
    ///
    /// The table filter must not leak into the other views: the Gantt and the
    /// Resources dictionary read [`Self::sorted`] and always see the whole
    /// programme. This cache is therefore separate from `sorted_cache`, and is
    /// only materialised when a filter is actually active, so the unfiltered
    /// hot path is unchanged.
    fn refresh_table_rows(&mut self, filter: &str, critical_only: bool) {
        let filter = filter.trim();
        let active = !filter.is_empty() || critical_only;
        if active == self.table_rows_active
            && self.table_rows_edit_version == self.edit_version
            && self.table_rows_filter == filter
            && self.table_rows_critical_only == critical_only
            && self.table_rows_source_len == self.project.activities.len()
            && self.table_rows_sorted_key == self.sort_key
            && self.table_rows_sorted_ascending == self.sort_ascending
        {
            return;
        }
        if active {
            let needle = filter.to_ascii_lowercase();
            let schedule = self.schedule.as_ref().ok();
            let rows: Vec<Activity> = self
                .sorted_cache
                .iter()
                .filter(|activity| {
                    activity_matches_filter(activity, &needle, critical_only, schedule)
                })
                .cloned()
                .collect();
            self.table_rows = rows;
        } else {
            self.table_rows.clear();
        }
        self.table_rows_active = active;
        self.table_rows_edit_version = self.edit_version;
        self.table_rows_filter = filter.to_string();
        self.table_rows_critical_only = critical_only;
        self.table_rows_source_len = self.project.activities.len();
        self.table_rows_sorted_key = self.sort_key;
        self.table_rows_sorted_ascending = self.sort_ascending;
    }

    /// The rows the activity table draws: the filtered subset when a filter is
    /// active, otherwise the full sorted programme.
    fn table_rows(&self) -> &[Activity] {
        if self.table_rows_active {
            &self.table_rows
        } else {
            &self.sorted_cache
        }
    }

    /// How many activities the given filter would show, for the row counter in
    /// the filter bar. Counted without cloning, so the per-frame cost is one
    /// pass over the names while a filter is active.
    fn table_match_count(&self, filter: &str, critical_only: bool) -> usize {
        let needle = filter.trim().to_ascii_lowercase();
        let schedule = self.schedule.as_ref().ok();
        self.sorted_cache
            .iter()
            .filter(|activity| activity_matches_filter(activity, &needle, critical_only, schedule))
            .count()
    }
}

impl eframe::App for CadenceApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // The table filter is edited into these locals and written back after
        // the table renders, so the widgets never borrow `self` at the same
        // time as the view closures do.
        let mut filter_text = self.table_filter_buffer.clone();
        let mut critical_only = self.table_critical_only;

        // Reflect unsaved state and version in the window title each frame.
        let title = format!(
            "{}{} — Cadence {}",
            self.project.name,
            if self.dirty { " •" } else { "" },
            update_check::current_version()
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        // Guard against closing with unsaved changes.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.confirm_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }

        // Poll the background update check at a low rate; the network request
        // itself happens on its own thread so the UI never blocks.
        if let Some(receiver) = self.update_receiver.take() {
            match receiver.try_recv() {
                Ok(status) => self.update_status = status,
                Err(mpsc::TryRecvError::Empty) => {
                    self.update_receiver = Some(receiver);
                    ctx.request_repaint_after(Duration::from_millis(250));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.update_status = update_check::UpdateStatus::Unavailable(
                        "the update checker stopped unexpectedly".to_string(),
                    );
                }
            }
        }

        // Periodic auto-update check: once per day (24 hours), avoiding
        // excessive GitHub API calls. Runs on a background thread so the
        // UI never blocks.
        let now = Instant::now();
        let should_check = match self.last_update_check {
            Some(last) => now.duration_since(last) >= Duration::from_secs(24 * 60 * 60),
            None => true, // first run, check immediately
        };
        if should_check {
            self.last_update_check = Some(now);
            let (sender, receiver) = mpsc::channel();
            self.update_receiver = Some(receiver);
            std::thread::spawn(move || {
                let status = update_check::check_for_update();
                let _ = sender.send(status);
            });
        }

        if ctx.input(|input| input.key_pressed(egui::Key::D) && input.modifiers.ctrl) {
            self.dark = !self.dark;
            apply_theme(&ctx, self.dark);
        }

        // Ctrl+S saves, Ctrl+O opens. These act everywhere, including while a
        // text field has focus (a save never touches the focused text).
        if ctx.input(|input| input.key_pressed(egui::Key::S) && input.modifiers.ctrl) {
            self.save_or_prompt();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::O) && input.modifiers.ctrl) {
            self.open_dialog();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::N) && input.modifiers.ctrl) {
            let id = self.project.next_activity_id();
            let command = cadence_commands::CreateActivity::new(id, "New activity", 1);
            if self
                .stack
                .execute(&mut self.project, Box::new(command))
                .is_ok()
            {
                self.recompute();
                self.select(id);
                self.scroll_to_selection = true;
                self.set_status("Activity created.");
            }
        }
        // Ctrl+W: request close, routed through the same unsaved-changes guard
        // as the window's own close button.
        if ctx.input(|input| input.key_pressed(egui::Key::W) && input.modifiers.ctrl) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y: undo and redo from any view. Skip
        // while a text field is focused, where the field's own undo applies.
        let undo_requested = !ctx.text_edit_focused()
            && ctx.input(|input| {
                input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && !input.modifiers.shift
            });
        let redo_requested = !ctx.text_edit_focused()
            && ctx.input(|input| {
                (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl)
                    || (input.key_pressed(egui::Key::Z)
                        && input.modifiers.ctrl
                        && input.modifiers.shift)
            });
        if undo_requested {
            let _ = self.stack.undo(&mut self.project);
            self.recompute();
            self.refresh_selection();
        }
        if redo_requested {
            let _ = self.stack.redo(&mut self.project);
            self.recompute();
            self.refresh_selection();
        }

        // Refresh the cached sort order once per edit, not once per frame,
        // and re-derive the (filtered) table rows from it.
        self.refresh_sorted();
        self.refresh_table_rows(&filter_text, critical_only);

        // Ctrl+1..Ctrl+9: switch views without reaching for the mouse.
        let (ctrl_held, pressed_view) = ctx.input(|input| {
            (
                input.modifiers.ctrl,
                [
                    egui::Key::Num1,
                    egui::Key::Num2,
                    egui::Key::Num3,
                    egui::Key::Num4,
                    egui::Key::Num5,
                    egui::Key::Num6,
                    egui::Key::Num7,
                    egui::Key::Num8,
                    egui::Key::Num9,
                ]
                .iter()
                .position(|key| input.key_pressed(*key))
                .map(|index| index as u8),
            )
        });
        if ctrl_held {
            if let Some(index) = pressed_view {
                self.view_mode = match index {
                    0 => ViewMode::Table,
                    1 => ViewMode::Gantt,
                    2 => ViewMode::Resources,
                    3 => ViewMode::Baselines,
                    4 => ViewMode::Calendars,
                    5 => ViewMode::Audit,
                    6 => ViewMode::Lookahead,
                    7 => ViewMode::Diary,
                    8 => ViewMode::Production,
                    _ => ViewMode::Diary,
                };
            }
        }

        // F1 opens the keyboard reference from every view.
        if ctx.input(|input| input.key_pressed(egui::Key::F1)) {
            self.show_shortcuts_dialog = true;
        }

        // A dialog owns the keyboard while it is open, so the view shortcuts
        // do not act behind it.
        let dialog_open =
            self.show_crash_dialog || self.show_update_dialog || self.show_shortcuts_dialog;

        // Gantt zoom from the keyboard; Ctrl+wheel remains the pointer path.
        if self.view_mode == ViewMode::Gantt && !ctx.text_edit_focused() && !dialog_open {
            let (zoom_in, zoom_out, zoom_reset) = ctx.input(|input| {
                let ctrl = input.modifiers.ctrl;
                let plus =
                    input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals);
                (
                    ctrl && plus,
                    ctrl && input.key_pressed(egui::Key::Minus),
                    ctrl && input.key_pressed(egui::Key::Num0),
                )
            });
            if zoom_in {
                self.pixels_per_day = (self.pixels_per_day * 1.25).clamp(2.0, 60.0);
            }
            if zoom_out {
                self.pixels_per_day = (self.pixels_per_day / 1.25).clamp(2.0, 60.0);
            }
            if zoom_reset {
                self.pixels_per_day = 14.0;
            }
        }

        // Up/Down select the previous/next activity in the Table view. Skip
        // while a text field or a combo popup is using the keyboard.
        if self.view_mode == ViewMode::Table
            && !ctx.text_edit_focused()
            && !ctx.any_popup_open()
            && !dialog_open
        {
            let (previous, next) = ctx.input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowUp),
                    input.key_pressed(egui::Key::ArrowDown),
                )
            });
            if previous || next {
                let activities = self.table_rows();
                if !activities.is_empty() {
                    let current = self
                        .selected
                        .and_then(|id| activities.iter().position(|activity| activity.id == id));
                    let index = if next {
                        current.map_or(0, |index| (index + 1).min(activities.len() - 1))
                    } else {
                        current.map_or(activities.len() - 1, |index| index.saturating_sub(1))
                    };
                    let id = activities[index].id;
                    self.select(id);
                    self.scroll_to_selection = true;
                }
            }

            // Escape drops the extra multi-selection rows; the anchor row
            // stays selected (S2-5).
            let escape_pressed = ctx.input(|input| input.key_pressed(egui::Key::Escape));
            if escape_pressed && !self.multi_selected.is_empty() {
                self.multi_selected.clear();
            }
        }

        // Consume a captured Gantt screenshot, if one was requested, and write
        // the resulting PDF to the path chosen by the user.
        if let Some(path) = self.gantt_print_path.clone() {
            let screenshot = ctx.input(|input| {
                input.events.iter().find_map(|event| {
                    if let egui::Event::Screenshot { image, .. } = event {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(image) = screenshot {
                let width = image.size[0] as u32;
                let height = image.size[1] as u32;
                let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
                for pixel in &image.pixels {
                    rgb.push(pixel.r());
                    rgb.push(pixel.g());
                    rgb.push(pixel.b());
                }
                let mut pdf = cadence_core::pdf::Pdf::new();
                pdf.image_page(width, height, rgb);
                match std::fs::write(&path, pdf.to_bytes()) {
                    Ok(()) => {
                        self.set_status(format!("Gantt printed to {}", path.display()));
                    }
                    Err(error) => {
                        self.set_status(format!("Cannot write {}: {error}", path.display()));
                    }
                }
                self.gantt_print_path = None;
            } else {
                ctx.request_repaint();
            }
        }

        // ─── TOP BAR: Three-zone layout ───
        // Zone 1: Menu bar (File / Edit / Window / Help)
        // Zone 2: View tab strip (Table, Gantt, Resources, …)
        // Zone 3: Tool actions (Columns, Undo, Redo, Version)
        egui::Panel::top("toolbar")
            .frame(egui::Frame::new().fill(self.design_tokens().panel_bg))
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // ─── Zone 1: Menu Bar ───
                    ui.menu_button("File", |ui| {
                        ui.set_min_width(160.0);
                        if ui.button("Open… (Ctrl+O)").clicked() {
                            self.open_dialog();
                        }
                        if ui.button("Save (Ctrl+S)").clicked() {
                            self.save_or_prompt();
                        }
                        if ui.button("Save as…").clicked() {
                            self.save_as_dialog();
                        }
                        ui.separator();
                        if !self.recent_files.is_empty() {
                            ui.menu_button("Recent files", |ui| {
                                let recent = self.recent_files.clone();
                                for path in recent {
                                    let display = path.display().to_string();
                                    if ui.button(display).clicked() {
                                        self.open_from(path);
                                    }
                                }
                            });
                            ui.separator();
                        }
                        if ui.button("Save report").clicked() {
                            self.save_report();
                        }
                        if ui.button("Print report…").clicked() {
                            self.print_report_dialog();
                        }
                        if ui
                            .add_enabled(
                                self.view_mode == ViewMode::Gantt,
                                egui::Button::new("Print Gantt…"),
                            )
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF document", &["pdf"])
                                .set_file_name(format!("{}-gantt.pdf", file_stem(&self.project.name)))
                                .save_file()
                            {
                                self.gantt_print_path = Some(path);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                                    egui::UserData::default(),
                                ));
                            }
                        }
                        ui.separator();
                        if ui.button("Close (Ctrl+W)").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        ui.set_min_width(160.0);
                        if ui
                            .add_enabled(self.stack.can_undo(), egui::Button::new("Undo (Ctrl+Z)"))
                            .clicked()
                        {
                            let _ = self.stack.undo(&mut self.project);
                            self.recompute();
                            self.refresh_selection();
                        }
                        if ui
                            .add_enabled(self.stack.can_redo(), egui::Button::new("Redo (Ctrl+Y)"))
                            .clicked()
                        {
                            let _ = self.stack.redo(&mut self.project);
                            self.recompute();
                            self.refresh_selection();
                        }
                        ui.separator();
                        if ui.button("New activity (Ctrl+N)").clicked() {
                            self.create_activity();
                        }
                    });

                    // ─── Window menu (was View menu actions that aren't view switching) ───
                    ui.menu_button("Window", |ui| {
                        ui.set_min_width(160.0);
                        if ui.button("Toggle theme (Ctrl+D)").clicked() {
                            self.dark = !self.dark;
                            apply_theme(&ctx, self.dark);
                        }
                        if ui
                            .button(if self.show_split_gantt {
                                "Hide split Gantt"
                            } else {
                                "Show split Gantt"
                            })
                            .clicked()
                        {
                            self.show_split_gantt = !self.show_split_gantt;
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        ui.set_min_width(160.0);
                        if ui.button("Keyboard shortcuts (F1)").clicked() {
                            self.show_shortcuts_dialog = true;
                        }
                        if ui.button("Report a bug…").clicked() {
                            if self.crash_log.is_empty() {
                                if let Some(log) = crash::read_log() {
                                    self.crash_log = log;
                                }
                            }
                            self.crash_notes.clear();
                            self.show_crash_dialog = true;
                        }
                        if ui.button("Check for updates").clicked() {
                            self.show_update_dialog = true;
                        }
                        if ui.button("Release notes").clicked() {
                            self.show_release_notes = true;
                        }
                    });

                    ui.separator();

                    // ─── Zone 2: View Tab Strip ───
                    let first = &mut true;
                    for (mode, label, tooltip) in [
                        (ViewMode::Table, "Table", "Activity table: list, sort, edit and link activities"),
                        (
                            ViewMode::Gantt,
                            "Gantt",
                            "Gantt chart: bars, dependencies, milestones and the today line",
                        ),
                        (
                            ViewMode::Resources,
                            "Resources",
                            "Resources: assignments, daily usage histograms and cost rates",
                        ),
                        (
                            ViewMode::Baselines,
                            "Baselines",
                            "Baselines: saved snapshots of the schedule to compare against",
                        ),
                        (
                            ViewMode::Calendars,
                            "Calendars",
                            "Calendars: working-day calendars and holiday exceptions",
                        ),
                        (
                            ViewMode::Audit,
                            "Audit",
                            "Schedule-quality audit: DCMA-style checks over the programme",
                        ),
                        (
                            ViewMode::Lookahead,
                            "Lookahead",
                            "Lookahead: rolling short-interval window and published snapshots",
                        ),
                        (
                            ViewMode::Diary,
                            "Diary",
                            "Diary: dated site notes linked to activities and lookahead windows",
                        ),
                        (
                            ViewMode::Production,
                            "Production",
                            "Production: record actual progress per activity and date",
                        ),
                    ] {
                        let selected = self.view_mode == mode;
                        if *first {
                            *first = false;
                        } else {
                            ui.separator();
                        }
                        if ui
                            .add(
                                egui::Button::new(label)
                                    .selected(selected)
                                    .frame(selected)
                                    .fill(if selected {
                                        self.design_tokens().accent_bg
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    })
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        if selected {
                                            self.design_tokens().accent
                                        } else {
                                            self.design_tokens().critical.linear_multiply(0.2)
                                        }
                                    ))
                            )
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            self.view_mode = mode;
                        }
                    }

                    ui.separator();

                    // ─── Zone 3: Tool Actions ───
                    // Columns chooser (prominent position)
                    let columns_menu = ui.menu_button("Columns", |ui| {
                        ui.set_min_width(180.0);
                        ui.heading("Visible columns");
                        for key in SortKey::ordered_columns() {
                            let mut visible = self.visible_columns.contains(&key);
                            if ui.checkbox(&mut visible, key.label()).changed() {
                                if visible {
                                    self.visible_columns.insert(key);
                                } else {
                                    self.visible_columns.remove(&key);
                                }
                            }
                        }
                        ui.separator();
                        if ui.button("Reset to default").clicked() {
                            self.visible_columns = SortKey::all_columns();
                        }
                    });
                    columns_menu.response.on_hover_text("Choose which columns show in the activity table (MS Project / P6 column chooser)");

                    ui.separator();

                    // Undo/Redo with primary styling when enabled
                    if ui
                        .add_enabled(
                            self.stack.can_undo(),
                            egui::Button::new("Undo (Ctrl+Z)")
                                .fill(self.design_tokens().primary_bg)
                                .stroke(egui::Stroke::new(1.0, self.design_tokens().accent))
                        )
                        .on_hover_text("Undo the last edit (Ctrl+Z)")
                        .clicked()
                    {
                        let _ = self.stack.undo(&mut self.project);
                        self.recompute();
                        self.refresh_selection();
                    }
                    if ui
                        .add_enabled(
                            self.stack.can_redo(),
                            egui::Button::new("Redo (Ctrl+Y)")
                                .fill(self.design_tokens().primary_bg)
                                .stroke(egui::Stroke::new(1.0, self.design_tokens().accent))
                        )
                        .on_hover_text("Redo the last undone edit (Ctrl+Y)")
                        .clicked()
                    {
                        let _ = self.stack.redo(&mut self.project);
                        self.recompute();
                        self.refresh_selection();
                    }

                    // Version button (clickable for release notes)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(update_check::current_version())
                            .on_hover_text("Click to view release notes")
                            .clicked()
                        {
                            self.show_release_notes = true;
                        }
                    });
                });
            });

        // Status bar: project dates and counts, critical count, zoom, and the
        // most recent status message. It is a bottom panel, so it must be added
        // before the per-view `CentralPanel` (which claims the remaining
        // space).
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                let range = match self.schedule.as_ref() {
                    Ok(schedule) => {
                        format!("{} to {}", schedule.project_start, schedule.project_finish)
                    }
                    Err(_) => "Cannot schedule".to_string(),
                };
                let critical_count = self.critical_count;
                let palette = design::palette(self.dark);
                // Left: project range, activity count, and the critical count
                // (coloured so criticality never relies on the number alone).
                ui.label(
                    egui::RichText::new(format!(
                        "{range}   {} activities",
                        self.project.activities.len()
                    ))
                    .small()
                    .color(palette.text_muted),
                );
                if critical_count > 0 {
                    let critical_text = if critical_count == 1 {
                        "1 critical activity".to_string()
                    } else {
                        format!("{critical_count} critical activities")
                    };
                    ui.label(
                        egui::RichText::new(critical_text)
                            .small()
                            .color(palette.critical),
                    );
                }
                // Centre: earned-value metrics when a baseline is active, or a
                // short hint when none has been captured.
                if let (true, Some(metrics)) = (self.baseline_active, &self.evm_metrics) {
                    ui.label(
                        egui::RichText::new(format!(
                            "PV {}  EV {}  AC {}  SV {}  CV {}  SPI {:.2}  CPI {:.2}",
                            metrics.planned_value,
                            metrics.earned_value,
                            metrics.actual_cost,
                            metrics.schedule_variance,
                            metrics.cost_variance,
                            metrics.schedule_performance_index.unwrap_or(0.0),
                            metrics.cost_performance_index.unwrap_or(0.0),
                        ))
                        .small()
                        .color(palette.text_muted),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No baseline")
                            .small()
                            .color(palette.text_muted),
                    );
                }
                let now = Instant::now();
                self.maybe_clear_status(now);
                if let Some(message) = &self.status_message {
                    ui.label("   ·   ");
                    let severity = StatusSeverity::from_message(message);
                    ui.label(
                        egui::RichText::new(message)
                            .weak()
                            .small()
                            .color(severity.color(self.dark)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Save indicator: a word, not a colour alone.
                    let (save_text, save_color) = if self.dirty {
                        ("● unsaved", palette.critical)
                    } else {
                        ("✓ saved", palette.ok)
                    };
                    ui.label(egui::RichText::new(save_text).small().color(save_color));
                    ui.label(
                        egui::RichText::new(format!("{:.0}%", self.pixels_per_day / 14.0 * 100.0))
                            .small()
                            .color(palette.text_muted),
                    );
                    // F1 footer: the keyboard reference is one click away from
                    // anywhere, matching the F1 binding.
                    if ui
                        .small_button("Shortcuts (F1)")
                        .on_hover_text("Open the keyboard reference (F1)")
                        .clicked()
                    {
                        self.show_shortcuts_dialog = true;
                    }
                });
            });
            ui.separator();
        });

        match self.view_mode {
            ViewMode::Table => {
                egui::Panel::right("inspector")
                    .default_size(260.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.heading("Inspector");
                                if !self.multi_selected.is_empty() {
                                    // S2-5: the range selection deletes as one
                                    // undoable step.
                                    let count = self.multi_selected.len();
                                    if ui.button(format!("Delete {count} selected")).clicked() {
                                        self.confirm_delete_selection = true;
                                    }
                                }
                                if let Some(id) = self.selected {
                                    if ui.button("Delete activity").clicked() {
                                        self.confirm_delete_activity = true;
                                    }
                                    ui.label(format!("Activity {}", id));
                                    let tokens = self.design_tokens();
                                    let section = |ui: &mut Ui, title: &str| {
                                        ui.add_space(4.0);
                                        ui.separator();
                                        ui.label(
                                            egui::RichText::new(title)
                                                .strong()
                                                .size(14.0)
                                                .color(tokens.accent),
                                        );
                                    };
                                    section(ui, "Basic info");
                                    let name_label = ui.label("Name");
                                    ui.text_edit_singleline(&mut self.name_buffer)
                                        .labelled_by(name_label.id);
                                    let duration_label = ui.label("Duration (working days)");
                                    ui.text_edit_singleline(&mut self.duration_buffer)
                                        .labelled_by(duration_label.id);
                                    ui.label("Constraint");
                                    let labels = CONSTRAINT_LABELS;
                                    ComboBox::from_id_salt("constraint_kind")
                                        .selected_text(CONSTRAINT_LABELS[self.constraint_kind])
                                        .show_ui(ui, |ui| {
                                            for (index, label) in labels.iter().enumerate() {
                                                ui.selectable_value(
                                                    &mut self.constraint_kind,
                                                    index,
                                                    *label,
                                                );
                                            }
                                        });
                                    if self.constraint_kind >= 2 {
                                        let constraint_date_label =
                                            ui.label("Constraint date (YYYY-MM-DD)");
                                        ui.text_edit_singleline(&mut self.constraint_date_buffer)
                                            .labelled_by(constraint_date_label.id);
                                    }

                                    section(ui, "Calendar");
                                    let calendar_names =
                                        self.project.calendars.calendars().to_vec();
                                    let current = self
                                        .project
                                        .activity(id)
                                        .and_then(|activity| activity.calendar);
                                    let selected_text = current
                                        .and_then(|current| {
                                            calendar_names
                                                .iter()
                                                .find(|named| named.id == current)
                                                .map(|named| named.name.clone())
                                        })
                                        .unwrap_or_else(|| "Project default".to_string());
                                    ComboBox::from_id_salt("activity_calendar")
                                        .selected_text(selected_text)
                                        .show_ui(ui, |ui| {
                                            if ui
                                                .selectable_label(
                                                    current.is_none(),
                                                    "Project default",
                                                )
                                                .clicked()
                                                && current.is_some()
                                            {
                                                let _ = self.stack.execute(
                                                    &mut self.project,
                                                    Box::new(SetActivityCalendar {
                                                        activity: id,
                                                        calendar: None,
                                                    }),
                                                );
                                                self.recompute();
                                            }
                                            for named in &calendar_names {
                                                let picked = current == Some(named.id);
                                                if ui
                                                    .selectable_label(picked, &named.name)
                                                    .clicked()
                                                    && !picked
                                                {
                                                    let _ = self.stack.execute(
                                                        &mut self.project,
                                                        Box::new(SetActivityCalendar {
                                                            activity: id,
                                                            calendar: Some(named.id),
                                                        }),
                                                    );
                                                    self.recompute();
                                                }
                                            }
                                        });

                                    section(ui, "Field planning");
                                    let owner_label = ui.label("Owner");
                                    ui.text_edit_singleline(&mut self.owner_buffer)
                                        .labelled_by(owner_label.id);
                                    ui.checkbox(
                                        &mut self.promised_buffer,
                                        "Promised in the lookahead",
                                    );
                                    let delay_reason_label =
                                        ui.label("Delay reason (why dates moved)");
                                    ui.text_edit_singleline(&mut self.delay_reason_buffer)
                                        .labelled_by(delay_reason_label.id);
                                    if ui.button("Apply field planning").clicked() {
                                        self.apply_field_planning(id);
                                    }

                                    section(ui, "Progress");
                                    let progress_label = ui.label("Percent complete");
                                    ui.text_edit_singleline(&mut self.progress_buffer)
                                        .labelled_by(progress_label.id);
                                    let budget_label = ui.label("Budget at completion");
                                    ui.text_edit_singleline(&mut self.budget_buffer)
                                        .labelled_by(budget_label.id);
                                    let actual_cost_label = ui.label("Actual cost");
                                    ui.text_edit_singleline(&mut self.actual_cost_buffer)
                                        .labelled_by(actual_cost_label.id);
                                    let actual_start_label = ui.label("Actual start (YYYY-MM-DD)");
                                    ui.text_edit_singleline(&mut self.actual_start_buffer)
                                        .labelled_by(actual_start_label.id);
                                    let actual_finish_label =
                                        ui.label("Actual finish (YYYY-MM-DD)");
                                    ui.text_edit_singleline(&mut self.actual_finish_buffer)
                                        .labelled_by(actual_finish_label.id);

                                    if ui.button("Apply").clicked() {
                                        self.apply_edits();
                                    }

                                    section(ui, "Relationships");
                                    ComboBox::from_id_salt("relationship_select")
                                        .selected_text(
                                            self.selected_relationship
                                                .and_then(|id| self.project.relationship(id))
                                                .map(|r| format!("rel {}", r.id))
                                                .unwrap_or_else(|| {
                                                    "Select relationship".to_string()
                                                }),
                                        )
                                        .show_ui(ui, |ui| {
                                            for rel in self.project.relationships.clone() {
                                                ui.selectable_value(
                                                    &mut self.selected_relationship,
                                                    Some(rel.id),
                                                    format!(
                                                        "{} -> {}",
                                                        rel.predecessor, rel.successor
                                                    ),
                                                );
                                            }
                                        });

                                    if self.last_rel != self.selected_relationship {
                                        self.last_rel = self.selected_relationship;
                                        self.lag_buffer = self
                                            .selected_relationship
                                            .and_then(|id| self.project.relationship(id))
                                            .map(|r| r.lag.to_string())
                                            .unwrap_or_default();
                                    }

                                    if self.selected_relationship.is_some() {
                                        let relationship = self
                                            .selected_relationship
                                            .and_then(|id| self.project.relationship(id))
                                            .map(|rel| (rel.id, rel.link_type));
                                        if let Some((rel_id, current_type)) = relationship {
                                            // S2-8: retype an existing
                                            // relationship instead of only
                                            // reading its type back.
                                            let current_index = link_type_index(current_type);
                                            let type_label = ui.label("Type");
                                            ComboBox::from_id_salt("relationship_type")
                                                .selected_text(LINK_TYPE_LABELS[current_index])
                                                .show_ui(ui, |ui| {
                                                    for (index, label) in
                                                        LINK_TYPE_LABELS.iter().enumerate()
                                                    {
                                                        let response = ui.selectable_label(
                                                            index == current_index,
                                                            *label,
                                                        );
                                                        if response.clicked() {
                                                            self.set_relationship_type(
                                                                rel_id,
                                                                link_type_from_index(index),
                                                            );
                                                        }
                                                    }
                                                })
                                                .response
                                                .labelled_by(type_label.id);
                                        }
                                        let lag_label = ui.label("Lag (working days)");
                                        ui.text_edit_singleline(&mut self.lag_buffer)
                                            .labelled_by(lag_label.id);
                                        if ui.button("Apply lag").clicked() {
                                            if let (Some(id), Ok(lag)) = (
                                                self.selected_relationship,
                                                self.lag_buffer.trim().parse::<i64>(),
                                            ) {
                                                let _ = self.stack.execute(
                                                    &mut self.project,
                                                    Box::new(SetRelationshipLag { id, lag }),
                                                );
                                                self.recompute();
                                            }
                                        }
                                        if ui.button("Delete relationship").clicked() {
                                            self.confirm_delete_relationship = true;
                                        }
                                    }

                                    ui.separator();
                                    ui.heading("Add relationship");
                                    ui.label(
                                        "This form is the keyboard path; dragging between \
                                         Gantt bars is a shortcut.",
                                    );
                                    let activity_choices = self.project.activities.clone();
                                    let predecessor_text = self
                                        .link_predecessor
                                        .and_then(|id| self.project.activity(id))
                                        .map(|activity| activity.name.clone())
                                        .unwrap_or_else(|| "Select activity".to_string());
                                    ui.label("Predecessor");
                                    ComboBox::from_id_salt("link_predecessor")
                                        .selected_text(predecessor_text)
                                        .show_ui(ui, |ui| {
                                            for activity in &activity_choices {
                                                ui.selectable_value(
                                                    &mut self.link_predecessor,
                                                    Some(activity.id),
                                                    &activity.name,
                                                );
                                            }
                                        });
                                    let successor_text = self
                                        .link_successor
                                        .and_then(|id| self.project.activity(id))
                                        .map(|activity| activity.name.clone())
                                        .unwrap_or_else(|| "Select activity".to_string());
                                    ui.label("Successor");
                                    ComboBox::from_id_salt("link_successor")
                                        .selected_text(successor_text)
                                        .show_ui(ui, |ui| {
                                            for activity in &activity_choices {
                                                ui.selectable_value(
                                                    &mut self.link_successor,
                                                    Some(activity.id),
                                                    &activity.name,
                                                );
                                            }
                                        });
                                    ui.label("Type");
                                    let link_types = LINK_TYPE_LABELS;
                                    ComboBox::from_id_salt("link_type")
                                        .selected_text(link_type_label(self.link_type_index))
                                        .show_ui(ui, |ui| {
                                            for (index, label) in link_types.iter().enumerate() {
                                                ui.selectable_value(
                                                    &mut self.link_type_index,
                                                    index,
                                                    *label,
                                                );
                                            }
                                        });
                                    ui.label("Lag (working days)");
                                    ui.text_edit_singleline(&mut self.link_lag_buffer);
                                    if ui.button("Add link").clicked() {
                                        self.add_link_from_form();
                                    }
                                } else {
                                    ui.label("Select an activity in the table.");
                                }
                            });
                    });

                if self.show_split_gantt {
                    let activities = self.sorted_cache.clone();
                    let mut gantt_changed = false;
                    let mut gantt_selected = self.selected;
                    egui::Panel::right("workbench_gantt")
                        .default_size(420.0)
                        .min_size(300.0)
                        .resizable(true)
                        .show(ui, |ui| {
                            gantt_changed = render_gantt(
                                ui,
                                &mut self.project,
                                &mut self.schedule,
                                &mut self.stack,
                                &mut self.drag_start,
                                &mut self.status_message,
                                &activities,
                                self.pixels_per_day,
                                &mut self.pixels_per_day,
                                &mut self.drag_link_type_index,
                                self.dark,
                                gantt_selected,
                                &mut gantt_selected,
                            );
                        });
                    if let Some(id) = gantt_selected {
                        if Some(id) != self.selected {
                            self.select(id);
                        }
                    }
                    if gantt_changed {
                        // A drag-created link changes the schedule; bump the
                        // edit version so cached sort and audit results refresh.
                        self.recompute();
                    }
                }

                // Table filter row (S2-5): narrow the rows by name and/or to
                // the critical path. Rendered as its own top panel so the
                // widgets never borrow `self` inside the table's CentralPanel.
                egui::Panel::top("table_filter").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        let filter_response = ui.text_edit_singleline(&mut filter_text);
                        filter_response.on_hover_text("Show only activities matching this name");
                        let critical_response = ui.checkbox(&mut critical_only, "Critical only");
                        critical_response.on_hover_text("Show only critical-path activities");
                        if !filter_text.trim().is_empty() || critical_only {
                            let shown = self.table_match_count(&filter_text, critical_only);
                            let total = self.project.activities.len();
                            ui.label(
                                egui::RichText::new(format!("{shown} of {total}"))
                                    .weak()
                                    .small(),
                            );
                        }
                    });
                });
                // Apply any change from this frame before the rows are drawn.
                self.table_filter_buffer = filter_text.clone();
                self.table_critical_only = critical_only;
                self.refresh_table_rows(&filter_text, critical_only);

                let schedule = self.schedule.as_ref();
                let sorted = self.table_rows();
                let selected_id = self.selected;
                let scroll_to_selection = self.scroll_to_selection;
                let mut clicked_activity = None;
                let mut context_menu_activity = None;
                let mut header_clicked = None;
                egui::CentralPanel::default().show(ui, |ui| match schedule {
                    Err(error) => {
                        ui.colored_label(
                            design::palette(self.dark).critical,
                            format!("Schedule error: {error}"),
                        );
                    }
                    Ok(schedule) => {
                        let row_height = 20.0;
                        let spacing = 12.0;

                        // Column chooser: MS Project / P6 let the user pick which
                        // columns show in the table. Row number is always first;
                        // the Activity column flexes to fill remaining width.
                        let mut defs: Vec<(Option<SortKey>, &'static str, f32)> =
                            vec![(None, "#", 36.0)];
                        for key in SortKey::ordered_columns() {
                            if self.visible_columns.contains(&key) {
                                defs.push((Some(key), key.label(), key.width()));
                            }
                        }
                        let fixed_sum: f32 = defs
                            .iter()
                            .filter(|(_, _, w)| *w > 0.0)
                            .map(|(_, _, w)| *w)
                            .sum();
                        let available = ui.available_width();
                        let flexible_budget =
                            (available - fixed_sum - spacing * (defs.len() as f32 - 1.0)).max(0.0);
                        let mut widths: Vec<f32> = Vec::with_capacity(defs.len());
                        for (_, _, w) in &defs {
                            widths.push(if *w == 0.0 { flexible_budget } else { *w });
                        }

                        // Sticky header: a fixed row of sort buttons above the
                        // virtualised body, so it stays visible while scrolling.
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = spacing;
                            ui.add_sized(
                                [widths[0], 22.0],
                                egui::Label::new(egui::RichText::new("#").weak()),
                            );
                            for (index, col) in defs.iter().enumerate().skip(1) {
                                let Some(key) = col.0 else {
                                    continue;
                                };
                                // The active sort column shows its direction with accent colour.
                                let label_text = if self.sort_key == key {
                                    format!(
                                        "{} {}",
                                        key.label(),
                                        if self.sort_ascending { "▲" } else { "▼" }
                                    )
                                } else {
                                    key.label().to_owned()
                                };
                                let label = if self.sort_key == key {
                                    egui::RichText::new(label_text).color(ACCENT)
                                } else {
                                    egui::RichText::new(label_text)
                                };
                                if ui
                                    .add_sized(
                                        [widths[index], 22.0],
                                        egui::Button::new(label).frame(false),
                                    )
                                    .on_hover_text(format!(
                                        "Sort by {} ({} order)",
                                        key.label(),
                                        if self.sort_ascending {
                                            "ascending"
                                        } else {
                                            "descending"
                                        }
                                    ))
                                    .clicked()
                                {
                                    header_clicked = Some(key);
                                }
                            }
                        });
                        ui.separator();

                        if sorted.is_empty() {
                            ui.add_space(8.0);
                            let message = if !filter_text.trim().is_empty() || critical_only {
                                "No activities match the filter. Clear the filter to see them all."
                            } else {
                                "No activities yet. Use New activity (Ctrl+N) to add the first one."
                            };
                            ui.label(egui::RichText::new(message).weak().small());
                        } else {
                            let selected_index = selected_id.and_then(|id| {
                                sorted.iter().position(|activity| activity.id == id)
                            });
                            let mut area = ScrollArea::vertical();
                            if scroll_to_selection {
                                if let Some(index) = selected_index {
                                    let offset = (index as f32 * row_height - 120.0).max(0.0);
                                    area = area.vertical_scroll_offset(offset);
                                }
                            }
                            area.show_rows(ui, row_height, sorted.len(), |ui, range| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                for index in range {
                                    let activity = &sorted[index];
                                    let Some(result) = schedule.activity(activity.id) else {
                                        continue;
                                    };
                                    // A row counts as selected when it is the
                                    // primary selection or part of the
                                    // ctrl/shift multi-selection (S2-5).
                                    let selected = selected_id == Some(activity.id)
                                        || self.multi_selected.contains(&activity.id);
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = spacing;
                                        // The whole row is a click target, so
                                        // activities stay selectable no matter
                                        // which columns are visible. The Name
                                        // button is drawn after it and keeps
                                        // its own hit area on top.
                                        let row_rect = Rect::from_min_size(
                                            ui.cursor().min,
                                            Vec2::new(ui.available_width(), row_height),
                                        );
                                        let row_response = ui.interact(
                                            row_rect,
                                            egui::Id::new(("activity_row", activity.id)),
                                            Sense::click(),
                                        );
                                        let row_clicked = row_response.clicked();
                                        let mut context_action = None;
                                        row_response.context_menu(|ui| {
                                            if ui.button("New activity below").clicked() {
                                                context_action = Some("new_below");
                                            }
                                            if ui.button("Duplicate activity").clicked() {
                                                context_action = Some("duplicate");
                                            }
                                            ui.separator();
                                            if ui.button("Show in Gantt").clicked() {
                                                context_action = Some("show_in_gantt");
                                            }
                                            if ui.button("Add to selection").clicked() {
                                                context_action = Some("add_to_selection");
                                            }
                                            ui.separator();
                                            if ui.button("Delete activity").clicked() {
                                                context_action = Some("delete");
                                            }
                                        });
                                        if let Some(action) = context_action {
                                            context_menu_activity = Some((activity.id, action));
                                        }
                                        for (col, (key, _label, _w)) in defs.iter().enumerate() {
                                            match key {
                                                None => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(
                                                            egui::RichText::new(
                                                                (index + 1).to_string(),
                                                            )
                                                            .monospace()
                                                            .weak(),
                                                        ),
                                                    );
                                                }
                                                Some(SortKey::Name) => {
                                                    if ui
                                                        .add_sized(
                                                            [widths[col], row_height],
                                                            egui::Button::new(
                                                                activity.name.as_str(),
                                                            )
                                                            .selected(selected)
                                                            .frame(selected)
                                                            .truncate(),
                                                        )
                                                        .clicked()
                                                    {
                                                        clicked_activity = Some(activity.id);
                                                    }
                                                }
                                                Some(SortKey::Duration) => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(
                                                            egui::RichText::new(
                                                                activity
                                                                    .original_duration
                                                                    .to_string(),
                                                            )
                                                            .monospace(),
                                                        ),
                                                    );
                                                }
                                                Some(SortKey::Start) => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(
                                                            egui::RichText::new(
                                                                result.scheduled_start.to_string(),
                                                            )
                                                            .monospace(),
                                                        ),
                                                    );
                                                }
                                                Some(SortKey::Finish) => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(
                                                            egui::RichText::new(
                                                                result.scheduled_finish.to_string(),
                                                            )
                                                            .monospace(),
                                                        ),
                                                    );
                                                }
                                                Some(SortKey::Float) => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(
                                                            egui::RichText::new(
                                                                result.total_float.to_string(),
                                                            )
                                                            .monospace(),
                                                        ),
                                                    );
                                                }
                                                Some(SortKey::Critical) => {
                                                    ui.add_sized(
                                                        [widths[col], row_height],
                                                        egui::Label::new(if result.critical {
                                                            "Yes"
                                                        } else {
                                                            "No"
                                                        }),
                                                    );
                                                }
                                            }
                                        }
                                        if row_clicked {
                                            clicked_activity = Some(activity.id);
                                        }
                                    });
                                }
                            });
                        }
                    }
                });
                if let Some(key) = header_clicked {
                    if self.sort_key == key {
                        self.sort_ascending = !self.sort_ascending;
                    } else {
                        self.sort_key = key;
                        self.sort_ascending = true;
                    }
                    self.refresh_sorted();
                    self.refresh_table_rows(&filter_text, critical_only);
                }
                if let Some(id) = clicked_activity {
                    // S2-5: plain click selects one row (and drops the extra
                    // selection); ctrl/cmd-click toggles a row; shift-click
                    // selects the range from the anchor row. The modifiers are
                    // frame state, so they are read here once, not per row.
                    let (command_held, shift_held) = ui
                        .ctx()
                        .input(|input| (input.modifiers.command, input.modifiers.shift));
                    if shift_held {
                        self.select_range(id);
                    } else if command_held {
                        self.toggle_multi_select(id);
                    } else {
                        self.multi_selected.clear();
                        self.select(id);
                    }
                }
                if let Some((id, action)) = context_menu_activity {
                    match action {
                        "new_below" => {
                            let new_id = self.project.next_activity_id();
                            let command =
                                cadence_commands::CreateActivity::new(new_id, "New activity", 1);
                            if self
                                .stack
                                .execute(&mut self.project, Box::new(command))
                                .is_ok()
                            {
                                self.recompute();
                                self.select(new_id);
                                self.scroll_to_selection = true;
                                self.set_status("Activity created.");
                            }
                        }
                        "duplicate" => {
                            if let Some(original) = self.project.activity(id).cloned() {
                                let new_id = self.project.next_activity_id();
                                let command = cadence_commands::CreateActivity::new(
                                    new_id,
                                    format!("{} (copy)", original.name),
                                    original.original_duration,
                                );
                                if self
                                    .stack
                                    .execute(&mut self.project, Box::new(command))
                                    .is_ok()
                                {
                                    self.recompute();
                                    self.select(new_id);
                                    self.scroll_to_selection = true;
                                    self.set_status("Activity duplicated.");
                                }
                            }
                        }
                        "show_in_gantt" => {
                            self.select(id);
                            self.view_mode = ViewMode::Gantt;
                            self.scroll_to_selection = true;
                        }
                        "add_to_selection" => {
                            // Grows the multi-selection without dropping the
                            // anchor row, the pointer-free twin of ctrl-click.
                            self.toggle_multi_select(id);
                        }
                        "delete" => {
                            self.select(id);
                            self.confirm_delete_activity = true;
                        }
                        _ => {}
                    }
                }
                self.scroll_to_selection = false;
            }
            ViewMode::Gantt => {
                let mut gantt_changed = false;
                let mut selected_id = self.selected;
                egui::Panel::right("gantt_inspector")
                    .default_size(260.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.heading("Inspector");
                                if let Some(id) = selected_id {
                                    if let Some(activity) = self.project.activity(id) {
                                        ui.label(format!("Activity {}", id));
                                        ui.label("Name");
                                        ui.label(&activity.name);
                                        ui.label(format!(
                                            "Duration: {} days",
                                            activity.original_duration
                                        ));
                                        let start = self
                                            .schedule
                                            .as_ref()
                                            .ok()
                                            .and_then(|s| s.activity(id))
                                            .map(|r| r.scheduled_start.to_string())
                                            .unwrap_or_else(|| "-".to_string());
                                        ui.label(format!("Start: {}", start));
                                        let finish = self
                                            .schedule
                                            .as_ref()
                                            .ok()
                                            .and_then(|s| s.activity(id))
                                            .map(|r| r.scheduled_finish.to_string())
                                            .unwrap_or_else(|| "-".to_string());
                                        ui.label(format!("Finish: {}", finish));
                                        let critical = self
                                            .schedule
                                            .as_ref()
                                            .ok()
                                            .and_then(|s| s.activity(id))
                                            .map(|r| r.critical)
                                            .unwrap_or(false);
                                        ui.label(if critical {
                                            "Critical: Yes"
                                        } else {
                                            "Critical: No"
                                        });
                                        if ui.button("Delete activity").clicked() {
                                            self.confirm_delete_activity = true;
                                        }
                                    }
                                } else {
                                    ui.label("Select an activity in the Gantt or Table view.");
                                }
                            });
                    });
                egui::CentralPanel::default().show(ui, |ui| {
                    let px = self.pixels_per_day;
                    gantt_changed = render_gantt(
                        ui,
                        &mut self.project,
                        &mut self.schedule,
                        &mut self.stack,
                        &mut self.drag_start,
                        &mut self.status_message,
                        &self.sorted_cache,
                        px,
                        &mut self.pixels_per_day,
                        &mut self.drag_link_type_index,
                        self.dark,
                        selected_id,
                        &mut selected_id,
                    );
                });
                if selected_id != self.selected {
                    if let Some(id) = selected_id {
                        self.select(id);
                    }
                }
                if gantt_changed {
                    self.recompute();
                }
            }
            ViewMode::Resources => {
                egui::Panel::right("resource_inspector")
                    .default_size(300.0)
                    .show(ui, |ui| {
                        ui.heading("Resource");
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.resource_name_buffer);
                        ui.label("Kind");
                        ComboBox::from_id_salt("resource_kind")
                            .selected_text(RESOURCE_KIND_LABELS[self.resource_kind_index])
                            .show_ui(ui, |ui| {
                                for (index, label) in RESOURCE_KIND_LABELS.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.resource_kind_index,
                                        index,
                                        *label,
                                    );
                                }
                            });
                        ui.label("Capacity (units per working day)");
                        ui.text_edit_singleline(&mut self.resource_capacity_buffer);
                        ui.label("Cost rate per unit (per working day)");
                        ui.text_edit_singleline(&mut self.resource_cost_rate_buffer);

                        if ui.button("Add resource").clicked() {
                            self.create_resource();
                        }
                        if self.selected_resource.is_some() {
                            if ui.button("Apply changes").clicked() {
                                self.apply_resource_edits();
                            }
                            if ui.button("Delete resource").clicked() {
                                self.delete_selected_resource();
                            }
                        }

                        ui.separator();
                        ui.heading("Leveling");
                        if ui.button("Preview leveling").clicked() {
                            self.preview_leveling();
                        }
                        if ui.button("Apply leveled dates").clicked() {
                            self.confirm_leveling = true;
                        }
                        if let Some(report) = &self.leveling_report {
                            ui.label(report);
                        }
                    });

                let schedule = self.schedule.clone();
                // S2-11: the conflicts and the day histogram are cached against
                // the edit version in `recompute`; the view only clones the
                // shared handles here, so a 10,000-activity programme does not
                // rescan its assignments every frame.
                let conflicts = self.conflicts_cache.clone();
                let histogram = self.histogram_cache.clone();
                let resources = self.project.resources.clone();
                let activities = self.sorted().to_vec();
                let selected_resource = self.selected_resource;

                egui::CentralPanel::default().show(ui, |ui| match schedule {
                    Err(error) => {
                        ui.colored_label(
                            design::palette(self.dark).critical,
                            format!("Schedule error: {error}"),
                        );
                    }
                    Ok(schedule) => {
                        ui.heading("Resource dictionary");
                        if resources.is_empty() {
                            ui.label("No resources yet. Add one in the panel on the right.");
                        }
                        Grid::new("resource_rows")
                            .striped(true)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Name");
                                ui.label("Kind");
                                ui.label("Capacity");
                                ui.label("Assigned");
                                ui.label("Over-allocated days");
                                ui.end_row();

                                for resource in &resources {
                                    let selected = selected_resource == Some(resource.id);
                                    if ui.selectable_label(selected, &resource.name).clicked() {
                                        self.select_resource(resource.id);
                                    }
                                    ui.label(
                                        RESOURCE_KIND_LABELS[resource_kind_index(resource.kind)],
                                    );
                                    ui.label(resource.capacity.to_string());
                                    let assigned = activities
                                        .iter()
                                        .filter(|activity| {
                                            activity.resources.contains(&resource.id)
                                        })
                                        .count();
                                    ui.label(assigned.to_string());
                                    let over = conflicts
                                        .iter()
                                        .filter(|conflict| conflict.resource_id == resource.id)
                                        .count();
                                    if over == 0 {
                                        ui.label("0");
                                    } else {
                                        ui.colored_label(
                                            design::palette(self.dark).critical,
                                            over.to_string(),
                                        );
                                    }
                                    ui.end_row();
                                }
                            });

                        ui.separator();
                        ui.heading("Usage");
                        if let Some(id) = selected_resource {
                            let usage = resource_day_usage(&histogram, id);
                            if usage.is_empty() {
                                ui.label(
                                    "This resource is not assigned to any scheduled activity.",
                                );
                            } else {
                                let capacity = self
                                    .project
                                    .resource(id)
                                    .map(|resource| resource.capacity)
                                    .unwrap_or(1);
                                let capacity_units = capacity.saturating_mul(FULL_UNITS);
                                ScrollArea::vertical()
                                    .id_salt("resource_usage")
                                    .max_height(160.0)
                                    .show(ui, |ui| {
                                        Grid::new("resource_usage_rows")
                                            .striped(true)
                                            .spacing([12.0, 4.0])
                                            .show(ui, |ui| {
                                                for (date, units) in usage {
                                                    ui.label(date.to_string());
                                                    let text = format!(
                                                        "{:.2} / {capacity}",
                                                        units as f64 / FULL_UNITS as f64
                                                    );
                                                    if units > capacity_units {
                                                        ui.colored_label(
                                                            design::palette(self.dark).critical,
                                                            text,
                                                        );
                                                    } else {
                                                        ui.label(text);
                                                    }
                                                    ui.end_row();
                                                }
                                            });
                                    });
                            }
                        } else {
                            ui.label("Select a resource to see its daily usage.");
                        }

                        ui.separator();
                        ui.heading("Assignments");
                        if let Some(id) = selected_resource {
                            ui.label(format!(
                                "Toggle '{}' on an activity to assign or remove it.",
                                self.project
                                    .resource(id)
                                    .map(|resource| resource.name.clone())
                                    .unwrap_or_default()
                            ));
                            Grid::new("assignment_rows")
                                .striped(true)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    for activity in &activities {
                                        let assigned = activity.resources.contains(&id);
                                        if ui.selectable_label(assigned, &activity.name).clicked() {
                                            self.selected = Some(activity.id);
                                            self.select(activity.id);
                                            self.toggle_assignment(id);
                                        }
                                        ui.label(activity.original_duration.to_string());
                                        ui.label(activity.original_duration.to_string());
                                        let current_units = activity.units_for(id);
                                        let current_text =
                                            format!("{:.2}", current_units as f64 / 100.0);
                                        let mut pending_units = None;
                                        {
                                            let entry = self
                                                .assignment_units_buffers
                                                .entry((activity.id, id))
                                                .or_insert_with(|| current_text.clone());
                                            let response = ui.add(
                                                egui::TextEdit::singleline(entry)
                                                    .desired_width(56.0)
                                                    .hint_text("100.00"),
                                            );
                                            if response.lost_focus() {
                                                if let Ok(percent) = entry.trim().parse::<f64>() {
                                                    let units = ((percent * 100.0).round() as i64)
                                                        .max(1)
                                                        as u32;
                                                    if assigned && units != current_units {
                                                        pending_units =
                                                            Some((activity.id, id, units));
                                                    }
                                                }
                                            }
                                        }
                                        // The buffer borrow has ended, so the
                                        // project may be mutated and repainted.
                                        if let Some((activity_id, resource_id, units)) =
                                            pending_units
                                        {
                                            match self.stack.execute(
                                                &mut self.project,
                                                Box::new(SetAssignmentUnits {
                                                    activity_id,
                                                    resource_id,
                                                    units,
                                                }),
                                            ) {
                                                Ok(()) => {
                                                    self.status_message = None;
                                                    self.recompute();
                                                }
                                                Err(error) => {
                                                    self.set_status(error);
                                                }
                                            }
                                        }
                                        let shown_units = self
                                            .project
                                            .activity(activity.id)
                                            .map(|activity| activity.units_for(id))
                                            .unwrap_or(FULL_UNITS);
                                        if let Some(entry) = self
                                            .assignment_units_buffers
                                            .get_mut(&(activity.id, id))
                                        {
                                            *entry = format!("{:.2}", shown_units as f64 / 100.0);
                                        }
                                        if let Some(result) = schedule.activity(activity.id) {
                                            ui.label(result.scheduled_start.to_string());
                                            ui.label(result.scheduled_finish.to_string());
                                        } else {
                                            ui.label("-");
                                            ui.label("-");
                                        }
                                        ui.end_row();
                                    }
                                });
                        } else {
                            ui.label("Select a resource to assign it to activities.");
                        }
                    }
                });
            }
            ViewMode::Baselines => {
                let mut status_date: Option<Date> = None;
                egui::Panel::right("baseline_inspector")
                    .default_size(300.0)
                    .show(ui, |ui| {
                        ui.heading("Baselines");
                        ui.label("Name for the new baseline");
                        ui.text_edit_singleline(&mut self.baseline_name_buffer);
                        if ui.button("Capture baseline").clicked() {
                            self.capture_baseline_now();
                        }

                        ui.separator();
                        let baselines = self.project.baselines.clone();
                        if baselines.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(
                                    "No baselines captured yet. Use Capture baseline to record \
                                     the current schedule as a baseline to compare against.",
                                )
                                .weak()
                                .small(),
                            );
                        }
                        for baseline in &baselines {
                            let selected = self.selected_baseline == Some(baseline.id);
                            let label = format!("{} — {}", baseline.name, baseline.created_at);
                            if ui.selectable_label(selected, label).clicked() {
                                self.selected_baseline = Some(baseline.id);
                            }
                        }
                        let has_selection = self.selected_baseline.is_some();
                        if ui
                            .add_enabled(has_selection, egui::Button::new("Delete baseline"))
                            .clicked()
                        {
                            self.delete_selected_baseline();
                        }

                        ui.separator();
                        ui.heading("Earned value");
                        status_date = date_field(
                            ui,
                            self.dark,
                            "Status date (YYYY-MM-DD)",
                            &mut self.status_date_buffer,
                        );
                    });

                let baselines = self.project.baselines.clone();
                let selected_baseline = self
                    .selected_baseline
                    .and_then(|id| baselines.iter().find(|baseline| baseline.id == id).cloned());
                let metrics = status_date.map(|status_date| {
                    evm_metrics(&self.project, selected_baseline.as_ref(), status_date)
                });
                let variance = match (&self.schedule, &selected_baseline) {
                    (Ok(schedule), Some(baseline)) => {
                        Some(variance_rows(&self.project, schedule, baseline))
                    }
                    _ => None,
                };

                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Earned value");
                    match &metrics {
                        None => {
                            ui.colored_label(
                                design::palette(self.dark).critical,
                                "Status date must be written as YYYY-MM-DD",
                            );
                        }
                        Some(metrics) => {
                            Grid::new("evm_rows")
                                .striped(true)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    for (label, value) in [
                                        (
                                            "Budget at completion",
                                            format_money(metrics.budget_at_completion),
                                        ),
                                        ("Planned value", format_money(metrics.planned_value)),
                                        ("Earned value", format_money(metrics.earned_value)),
                                        ("Actual cost", format_money(metrics.actual_cost)),
                                        (
                                            "Schedule variance",
                                            format_money(metrics.schedule_variance),
                                        ),
                                        ("Cost variance", format_money(metrics.cost_variance)),
                                        (
                                            "Schedule performance index",
                                            format_index(metrics.schedule_performance_index),
                                        ),
                                        (
                                            "Cost performance index",
                                            format_index(metrics.cost_performance_index),
                                        ),
                                    ] {
                                        ui.label(label);
                                        ui.label(value);
                                        ui.end_row();
                                    }
                                });
                        }
                    }

                    ui.separator();
                    ui.heading("Variance against the selected baseline");
                    match &variance {
                        None => {
                            ui.label("Capture or select a baseline to compare against.");
                        }
                        Some(rows) => {
                            ui.label(
                                "Variances are whole calendar days; a positive value is later or longer than planned.",
                            );
                            ScrollArea::vertical()
                                .id_salt("variance_rows")
                                .show(ui, |ui| {
                                    Grid::new("variance_grid")
                                        .striped(true)
                                        .spacing([12.0, 4.0])
                                        .show(ui, |ui| {
                                            for header in [
                                                "Activity", "Start", "Base start", "ΔStart",
                                                "Finish", "Base finish", "ΔFinish", "Dur",
                                                "Base dur", "ΔDur",
                                            ] {
                                                ui.label(header);
                                            }
                                            ui.end_row();

                                            for row in rows {
                                                let selected = self.selected == Some(row.activity);
                                                let name = format!("{}  {}", row.activity, row.name);
                                                if ui.selectable_label(selected, name).clicked() {
                                                    self.selected = Some(row.activity);
                                                }
                                                ui.label(date_text(row.current_start));
                                                ui.label(date_text(row.baseline_start));
                                                ui.colored_label(
                                                    variance_color(row.start_variance(), self.dark),
                                                    variance_text(row.start_variance()),
                                                );
                                                ui.label(date_text(row.current_finish));
                                                ui.label(date_text(row.baseline_finish));
                                                ui.colored_label(
                                                    variance_color(row.finish_variance(), self.dark),
                                                    variance_text(row.finish_variance()),
                                                );
                                                let duration_variance = row.duration_variance();
                                                ui.label(row.current_duration.to_string());
                                                ui.label(row.baseline_duration.to_string());
                                                ui.colored_label(
                                                    variance_color(
                                                        Some(duration_variance),
                                                        self.dark,
                                                    ),
                                                    variance_text(Some(duration_variance)),
                                                );
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    }
                });
            }
            ViewMode::Calendars => {
                egui::Panel::right("calendar_inspector")
                    .default_size(300.0)
                    .show(ui, |ui| {
                        ui.heading("Calendars");
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.calendar_name_buffer);
                        ui.label("Working days");
                        ui.horizontal_wrapped(|ui| {
                            for (index, label) in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                                .iter()
                                .enumerate()
                            {
                                ui.checkbox(&mut self.calendar_weekdays[index], *label);
                            }
                        });
                        ui.label("Holidays (comma-separated YYYY-MM-DD)");
                        ui.text_edit_singleline(&mut self.calendar_holidays_buffer);

                        if ui.button("Add calendar").clicked() {
                            self.create_calendar();
                        }
                        if self.selected_calendar.is_some() {
                            if ui.button("Apply changes").clicked() {
                                self.apply_calendar_edits();
                            }
                            if ui.button("Delete calendar").clicked() {
                                self.delete_selected_calendar();
                            }
                        }
                        ui.label(
                            "A calendar cannot be deleted while an activity uses it; \
                             holidays and working days can be changed at any time.",
                        );
                    });

                let calendars = self.project.calendars.calendars().to_vec();
                let selected = self.selected_calendar;
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Named calendars");
                    if calendars.is_empty() {
                        ui.label(
                            "No named calendars yet; the project uses one shared working week.",
                        );
                    }
                    Grid::new("calendar_rows")
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for header in ["Calendar", "Working days", "Holidays", "Activities"] {
                                ui.label(header);
                            }
                            ui.end_row();

                            for named in &calendars {
                                if ui
                                    .selectable_label(selected == Some(named.id), &named.name)
                                    .clicked()
                                {
                                    self.select_calendar(named.id);
                                }
                                ui.label(format!("{}/7", named.calendar.working_days_per_week()));
                                ui.label(named.calendar.holidays().len().to_string());
                                let users = self
                                    .project
                                    .activities
                                    .iter()
                                    .filter(|activity| activity.calendar == Some(named.id))
                                    .count();
                                ui.label(users.to_string());
                                ui.end_row();
                            }
                        });

                    ui.separator();
                    if let Some(id) = selected {
                        if let Some(named) = self.project.calendars.named(id) {
                            ui.heading(&named.name);
                            ui.label(format!("Calendar {}", named.id));
                            let holidays = named.calendar.holidays();
                            if holidays.is_empty() {
                                ui.label("No holiday exceptions.");
                            } else {
                                for holiday in holidays {
                                    ui.label(holiday.to_string());
                                }
                            }
                        }
                    } else {
                        ui.label("Select a calendar to see its holidays.");
                    }
                });
            }
            ViewMode::Audit => {
                egui::CentralPanel::default().show(ui, |ui| {
                    if self.audit_report.is_none() || self.audit_version != self.edit_version {
                        self.run_audit();
                    }
                    ui.heading("Audit");
                    ui.label(
                        "Schedule-quality checks over the current programme, following \
                         public DCMA and GAO practice. Findings are data, not errors: \
                         fix what matters and re-run.",
                    );
                    ui.add_space(4.0);
                    let mut audit_date: Option<Date> = None;
                    ui.horizontal_wrapped(|ui| {
                        audit_date = date_field(
                            ui,
                            self.dark,
                            "Status date (YYYY-MM-DD, optional)",
                            &mut self.audit_status_buffer,
                        );
                        if ui.button("Run audit").clicked() {
                            match audit_date {
                                Some(status) => {
                                    self.audit_status_applied = Some(status);
                                    self.run_audit();
                                }
                                None => {
                                    if self.audit_status_buffer.trim().is_empty() {
                                        self.audit_status_applied = None;
                                        self.run_audit();
                                    }
                                    // Invalid dates show the inline hint from
                                    // `date_field`; nothing more to do here.
                                }
                            }
                        }
                    });
                    let Some(report) = self.audit_report.clone() else {
                        ui.label(
                            "The programme cannot be scheduled, so there is nothing \
                             to audit; fix the scheduling error shown in the table.",
                        );
                        return;
                    };
                    let mut pass = 0;
                    let mut fail = 0;
                    let mut not_applicable = 0;
                    for check in &report.checks {
                        match check.outcome {
                            CheckOutcome::Pass => pass += 1,
                            CheckOutcome::Fail => fail += 1,
                            CheckOutcome::NotApplicable => not_applicable += 1,
                        }
                    }
                    ui.label(format!(
                        "{pass} pass, {fail} fail, {not_applicable} not applicable"
                    ));
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for check in &report.checks {
                                let word = match check.outcome {
                                    CheckOutcome::Pass => "pass",
                                    CheckOutcome::Fail => "FAIL",
                                    CheckOutcome::NotApplicable => "not applicable",
                                };
                                ui.horizontal_wrapped(|ui| {
                                    ui.monospace(check.id.code());
                                    ui.label(format!("[{word}]"));
                                    if let Some(measured) = &check.measured {
                                        ui.label(measured);
                                    }
                                });
                                ui.label(&check.summary);
                                if let Some(threshold) = &check.threshold {
                                    ui.label(format!("Threshold: {threshold}"));
                                }
                                for finding in &check.findings {
                                    match finding {
                                        Finding::Activity { id, name, detail } => {
                                            ui.horizontal_wrapped(|ui| {
                                                let label = format!("{name} ({id})");
                                                if ui.selectable_label(false, &label).clicked() {
                                                    self.select(*id);
                                                    self.scroll_to_selection = true;
                                                    self.view_mode = ViewMode::Table;
                                                }
                                                ui.label(detail);
                                            });
                                        }
                                        Finding::Relationship { id, detail } => {
                                            ui.label(format!("Relationship {id}: {detail}"));
                                        }
                                    }
                                }
                                ui.separator();
                            }
                        });
                });
            }
            ViewMode::Lookahead => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Lookahead");
                    ui.label(
                        "The rolling short-interval window over the master schedule: \
                         everything scheduled to work in the next few weeks, in start \
                         order. It is a view of the same schedule, never a second \
                         plan.",
                    );
                    ui.add_space(4.0);
                    let mut lookahead_from: Option<Date> = None;
                    ui.horizontal_wrapped(|ui| {
                        lookahead_from = date_field(
                            ui,
                            self.dark,
                            "Window starts (YYYY-MM-DD)",
                            &mut self.lookahead_start_buffer,
                        );
                        ui.label("Length");
                        for (index, weeks) in [2usize, 4, 6, 12].iter().enumerate() {
                            ui.selectable_value(
                                &mut self.lookahead_weeks,
                                index,
                                format!("{weeks} w"),
                            );
                        }
                    });
                    ui.add_space(4.0);
                    let weeks = [2usize, 4, 6, 12][self.lookahead_weeks.min(3)];
                    let Some(from) = lookahead_from else {
                        return;
                    };
                    let Ok(schedule) = self.schedule.clone() else {
                        ui.label(
                            "The programme cannot be scheduled, so there is no \
                             lookahead; fix the scheduling error shown in the table.",
                        );
                        return;
                    };
                    // S2-12: the window is cached against the edit version and
                    // the chosen (start, weeks) pair, so re-rendering the view
                    // — or the O(n²) snapshot comparison below — does not
                    // recompute the window every frame.
                    let entries = if self.lookahead_version == self.edit_version
                        && self.lookahead_window == Some((from, weeks))
                    {
                        self.lookahead_cache.clone().unwrap_or_default()
                    } else {
                        let window = lookahead(&self.project, &schedule, from, weeks as u32);
                        let computed = Arc::new(window);
                        self.lookahead_cache = Some(computed.clone());
                        self.lookahead_version = self.edit_version;
                        self.lookahead_window = Some((from, weeks));
                        computed
                    };
                    let window_end = from
                        .checked_add(time::Duration::weeks(weeks as i64))
                        .unwrap_or(from);
                    ui.label(format!(
                        "{} activities between {from} and {window_end}",
                        entries.len()
                    ));
                    ui.separator();
                    if entries.is_empty() {
                        ui.label(
                            "Nothing is scheduled in this window yet; move the \
                             window or add work in the table.",
                        );
                        return;
                    }
                    Grid::new("lookahead_rows")
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for header in [
                                "Activity", "Start", "Finish", "State", "Owner", "Promised",
                                "Critical",
                            ] {
                                ui.label(header);
                            }
                            ui.end_row();
                            for entry in entries.iter() {
                                let label = format!("{} ({})", entry.name, entry.id);
                                if ui.selectable_label(false, &label).clicked() {
                                    self.select(entry.id);
                                    self.view_mode = ViewMode::Table;
                                }
                                ui.label(entry.scheduled_start.to_string());
                                ui.label(entry.scheduled_finish.to_string());
                                ui.label(if entry.in_progress {
                                    "In progress"
                                } else {
                                    "Upcoming"
                                });
                                ui.label(entry.owner.clone().unwrap_or_else(|| "-".to_string()));
                                let promise = if entry.promised { "Promised" } else { "-" };
                                ui.label(promise);
                                ui.label(if entry.critical { "Yes" } else { "No" });
                                ui.end_row();
                            }
                        });
                    ui.separator();

                    // Publish the current window as a named snapshot so it can
                    // be compared against later windows.
                    ui.heading("Publish snapshot");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.lookahead_snapshot_name_buffer);
                        if ui.button("Publish").clicked() {
                            let name = self.lookahead_snapshot_name_buffer.trim();
                            if name.is_empty() {
                                self.set_status("Snapshot name must not be empty.");
                            } else {
                                let weeks = [2usize, 4, 6, 12][self.lookahead_weeks.min(3)];
                                let from = parse_date(self.lookahead_start_buffer.trim())
                                    .expect("start date already validated");
                                let command = cadence_commands::CaptureLookaheadSnapshot {
                                    name: name.to_string(),
                                    created_at: today(),
                                    window_start: from,
                                    window_weeks: weeks as u32,
                                };
                                if self
                                    .stack
                                    .execute(&mut self.project, Box::new(command))
                                    .is_ok()
                                {
                                    self.recompute();
                                    self.lookahead_snapshot_name_buffer.clear();
                                    self.set_status("Lookahead snapshot published.");
                                } else {
                                    self.set_status("Cannot publish snapshot.");
                                }
                            }
                        }
                    });

                    ui.separator();
                    ui.heading("Published snapshots");
                    if self.project.lookahead_snapshots.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "No snapshots published yet. Publish the current window above \
                                 to compare it against a later one.",
                            )
                            .weak()
                            .small(),
                        );
                    } else {
                        for snapshot in &self.project.lookahead_snapshots {
                            let selected = self.selected_lookahead_snapshot == Some(snapshot.id);
                            ui.horizontal(|ui| {
                                if ui.selectable_label(selected, &snapshot.name).clicked() {
                                    self.selected_lookahead_snapshot = Some(snapshot.id);
                                }
                                ui.label(format!(
                                    "{} weeks from {}",
                                    snapshot.window_weeks, snapshot.window_start
                                ));
                                ui.label(format!("captured {}", snapshot.created_at));
                            });
                        }
                        if let Some(snapshot_id) = self.selected_lookahead_snapshot {
                            if let Some(snapshot) = self
                                .project
                                .lookahead_snapshots
                                .iter()
                                .find(|s| s.id == snapshot_id)
                            {
                                ui.separator();
                                ui.heading("Compare with snapshot");
                                ui.label(format!(
                                    "Snapshot '{}' captured {} ({} weeks from {})",
                                    snapshot.name,
                                    snapshot.created_at,
                                    snapshot.window_weeks,
                                    snapshot.window_start,
                                ));
                                let snapshot_entries: Vec<&LookaheadSnapshotEntry> =
                                    snapshot.entries.iter().collect();
                                // Index the current window by activity id, so
                                // the comparison is linear in the snapshot
                                // size rather than O(n²) per frame (S2-12).
                                let current_by_id: BTreeMap<ActivityId, &LookaheadEntry> =
                                    entries.iter().map(|entry| (entry.id, entry)).collect();
                                // Compare by activity id.
                                for entry in &snapshot_entries {
                                    let current = current_by_id.get(&entry.activity_id);
                                    match current {
                                        Some(current) => {
                                            let start_variance =
                                                current.scheduled_start - entry.scheduled_start;
                                            let finish_variance =
                                                current.scheduled_finish - entry.scheduled_finish;
                                            ui.horizontal(|ui| {
                                                ui.label(format!(
                                                    "{} ({})",
                                                    entry.name, entry.activity_id
                                                ));
                                                ui.label(format!(
                                                    "start {:+}d / finish {:+}d",
                                                    start_variance.whole_days(),
                                                    finish_variance.whole_days()
                                                ));
                                            });
                                        }
                                        None => {
                                            ui.horizontal(|ui| {
                                                ui.label(format!(
                                                    "{} ({}) — no longer in window",
                                                    entry.name, entry.activity_id
                                                ));
                                                ui.label("removed");
                                            });
                                        }
                                    }
                                }
                                for entry in entries.iter() {
                                    let in_snapshot = snapshot_entries
                                        .iter()
                                        .any(|snapshot| snapshot.activity_id == entry.id);
                                    if !in_snapshot {
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "{} ({}) — new in window",
                                                entry.name, entry.id
                                            ));
                                            ui.label("added");
                                        });
                                    }
                                }
                                ui.separator();
                                if ui.button("Clear comparison").clicked() {
                                    self.selected_lookahead_snapshot = None;
                                }
                            }
                        }
                    }
                    ui.separator();
                    ui.heading("Location clashes");
                    let clashes = location_clashes(&self.project, &schedule);
                    if clashes.is_empty() {
                        ui.label(
                            "No activities share a location on overlapping days. \
                             Set a location in the inspector to include an activity \
                             in the check.",
                        );
                    } else {
                        let mut jump_to = None;
                        for clash in &clashes {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&clash.location).strong());
                                ui.label(format!("{}", clash.date));
                                if ui
                                    .link(format!("{} ({})", clash.first_name, clash.first))
                                    .clicked()
                                {
                                    jump_to = Some(clash.first);
                                }
                                ui.label("clashes with");
                                if ui
                                    .link(format!("{} ({})", clash.second_name, clash.second))
                                    .clicked()
                                {
                                    jump_to = Some(clash.second);
                                }
                            });
                        }
                        if let Some(id) = jump_to {
                            self.select(id);
                            self.view_mode = ViewMode::Table;
                        }
                    }
                    ui.separator();
                    ui.label(
                        "Select an entry to open the activity in the table. The \
                         window comes from the master schedule, so editing dates \
                         there moves the lookahead too.",
                    );
                });
            }
            ViewMode::Diary => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Daily diary");
                    ui.label(
                        "Dated site records linked to the plan: what happened, what \
                         slipped, what needs escalating. Attach a note to an activity \
                         or to a lookahead window so it sits against the work it \
                         concerns.",
                    );
                    let mut diary_date: Option<Date> = None;
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        diary_date = date_field(
                            ui,
                            self.dark,
                            "Date (YYYY-MM-DD)",
                            &mut self.diary_date_buffer,
                        );
                        ui.label("Activity id (optional)");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.diary_activity_buffer)
                                .desired_width(56.0),
                        );
                        ui.label("Window start (optional)");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.diary_window_buffer)
                                .desired_width(96.0),
                        );
                    });
                    ui.label("Note");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.diary_content_buffer).desired_rows(3),
                    );
                    if ui.button("Add note").clicked() {
                        let date = diary_date;
                        let activity = self
                            .diary_activity_buffer
                            .trim()
                            .parse::<u64>()
                            .ok()
                            .map(ActivityId::new);
                        let window = if self.diary_window_buffer.trim().is_empty() {
                            Some(None)
                        } else {
                            parse_date(self.diary_window_buffer.trim()).map(Some)
                        };
                        match (date, window) {
                            (Some(date), Some(lookahead_window_start)) => {
                                let command = cadence_commands::CreateDailyNote {
                                    date,
                                    content: self.diary_content_buffer.clone(),
                                    activity_id: activity,
                                    lookahead_window_start,
                                };
                                if self
                                    .stack
                                    .execute(&mut self.project, Box::new(command))
                                    .is_ok()
                                {
                                    self.recompute();
                                    self.diary_content_buffer.clear();
                                    self.set_status("Diary note added.");
                                } else {
                                    self.set_status("Cannot add the note; check the content.");
                                }
                            }
                            _ => {
                                self.set_status(
                                    "The date (and window start, if given) must be YYYY-MM-DD.",
                                );
                            }
                        }
                    }
                    ui.separator();
                    ui.heading("Notes");
                    // Copy the note fields so the list does not borrow the project
                    // while a delete command mutates it.
                    let mut notes: Vec<DiaryRow> = self
                        .project
                        .daily_notes
                        .iter()
                        .map(|note| {
                            (
                                note.id,
                                note.date,
                                note.content.clone(),
                                note.activity_id,
                                note.lookahead_window_start,
                            )
                        })
                        .collect();
                    notes.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
                    if notes.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "No diary notes yet. Add a dated note above to record what \
                                 happened on site (Ctrl+8 opens this view).",
                            )
                            .weak()
                            .small(),
                        );
                    }
                    let mut to_delete = None;
                    for (id, date, content, activity_id, window) in &notes {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(date.to_string()).monospace());
                            ui.label(content);
                            if let Some(activity_id) = activity_id {
                                ui.label(format!("→ activity {activity_id}"));
                            }
                            if let Some(window) = window {
                                ui.label(format!("→ window from {window}"));
                            }
                            // S2-10: a note can be corrected after it was
                            // written, through the undoable UpdateDailyNote.
                            if ui.button("Edit").clicked() {
                                self.editing_note = Some(*id);
                                self.diary_edit_buffer = content.clone();
                            }
                            if ui.button("Delete").clicked() {
                                to_delete = Some(*id);
                            }
                        });
                        if self.editing_note == Some(*id) {
                            ui.horizontal(|ui| {
                                let edit_label = ui.label("Edited note");
                                ui.text_edit_singleline(&mut self.diary_edit_buffer)
                                    .labelled_by(edit_label.id);
                                if ui.button("Save note").clicked() {
                                    let command = cadence_commands::UpdateDailyNote {
                                        id: *id,
                                        content: self.diary_edit_buffer.clone(),
                                    };
                                    if self
                                        .stack
                                        .execute(&mut self.project, Box::new(command))
                                        .is_ok()
                                    {
                                        self.recompute();
                                        self.editing_note = None;
                                        self.set_status("Diary note updated.");
                                    } else {
                                        self.set_status("The note content must not be empty.");
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    self.editing_note = None;
                                }
                            });
                        }
                    }
                    if let Some(id) = to_delete {
                        let command = cadence_commands::DeleteDailyNote { id };
                        if self
                            .stack
                            .execute(&mut self.project, Box::new(command))
                            .is_ok()
                        {
                            self.recompute();
                            self.set_status("Diary note deleted.");
                        }
                    }
                });
            }
            ViewMode::Production => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("Production tracking");
                    ui.label("Record daily actual progress against the schedule.");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Activity");
                        egui::ComboBox::from_id_salt("prod_activity")
                            .selected_text(if self.production_activity_buffer.is_empty() {
                                "Select activity".to_string()
                            } else {
                                self.production_activity_buffer.clone()
                            })
                            .show_ui(ui, |ui| {
                                for activity in &self.project.activities {
                                    ui.selectable_value(
                                        &mut self.production_activity_buffer,
                                        activity.id.0.to_string(),
                                        &activity.name,
                                    );
                                }
                            });
                        let production_date = date_field(
                            ui,
                            self.dark,
                            "Date (YYYY-MM-DD)",
                            &mut self.production_date_buffer,
                        );
                        ui.label("Percent");
                        ui.text_edit_singleline(&mut self.production_percent_buffer);
                        if ui.button("Record").clicked() {
                            if let Ok(activity_id) = self.production_activity_buffer.parse::<u64>()
                            {
                                if let Some(date) = production_date {
                                    if let Ok(percent) =
                                        self.production_percent_buffer.parse::<i64>()
                                    {
                                        let command = cadence_commands::RecordProduction {
                                            activity_id: ActivityId(activity_id),
                                            date,
                                            percent_complete: percent * 100,
                                        };
                                        if self
                                            .stack
                                            .execute(&mut self.project, Box::new(command))
                                            .is_ok()
                                        {
                                            self.recompute();
                                            self.set_status("Production recorded.");
                                        }
                                    } else {
                                        self.set_status("Percent must be a number.");
                                    }
                                } else {
                                    // Invalid date is shown inline by `date_field`.
                                }
                            } else {
                                self.set_status("Select an activity first.");
                            }
                        }
                    });
                    ui.separator();
                    ui.heading("Entries");
                    let mut to_delete = None;
                    for entry in &self.project.production_entries {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Activity {} — {} — {} bp",
                                entry.activity_id, entry.date, entry.percent_complete
                            ));
                            if ui.button("Delete").clicked() {
                                to_delete = Some(entry.id);
                            }
                        });
                    }
                    if let Some(id) = to_delete {
                        let command = cadence_commands::DeleteProduction { id };
                        if self
                            .stack
                            .execute(&mut self.project, Box::new(command))
                            .is_ok()
                        {
                            self.recompute();
                            self.set_status("Production entry deleted.");
                        }
                    }
                    ui.separator();
                    ui.collapsing("Reschedule from progress", |ui| {
                        ui.label(
                            "Preview how the schedule moves when remaining work is \
                             rescheduled from a status date. The plan is not changed \
                             until you apply the result (a later slice).",
                        );
                        ui.horizontal(|ui| {
                            ui.label("Status date");
                            ui.text_edit_singleline(&mut self.reschedule_date_buffer);
                            if ui.button("Preview").clicked() {
                                if let Some(date) = parse_date(self.reschedule_date_buffer.trim()) {
                                    match ReschedulePreview::build(
                                        &self.project,
                                        self.schedule.as_ref().ok(),
                                        date,
                                    ) {
                                        Ok(preview) => {
                                            self.set_status(format!(
                                                "Reschedule preview: the project finishes {}.",
                                                preview.project_finish
                                            ));
                                            self.reschedule_preview = Some(preview);
                                        }
                                        Err(message) => self.set_status(message),
                                    }
                                } else {
                                    self.set_status("Status date must be YYYY-MM-DD.");
                                }
                            }
                            if self.reschedule_preview.is_some() && ui.button("Clear").clicked() {
                                self.reschedule_preview = None;
                            }
                        });
                        if let Some(preview) = &self.reschedule_preview {
                            ui.label(format!(
                                "As of {}: project finish {}. {} complete, {} in progress, \
                                 {} not started.",
                                preview.data_date,
                                preview.project_finish,
                                preview.complete,
                                preview.in_progress,
                                preview.not_started
                            ));
                            egui::Grid::new("reschedule_preview_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Activity");
                                    ui.strong("Planned finish");
                                    ui.strong("Rescheduled finish");
                                    ui.strong("Variance (days)");
                                    ui.end_row();
                                    for row in &preview.rows {
                                        ui.label(&row.name);
                                        ui.label(row.planned_finish.map_or_else(
                                            || "—".to_string(),
                                            |date| date.to_string(),
                                        ));
                                        ui.label(row.progress_finish.to_string());
                                        ui.label(match row.variance_days {
                                            Some(days) => format!("{days:+}"),
                                            None => "—".to_string(),
                                        });
                                        ui.end_row();
                                    }
                                });
                        }
                    });
                });
            }
        }

        if self.confirm_delete_activity {
            let mut open = true;
            egui::Window::new("Delete activity?")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label("Delete the selected activity? This can be undone.");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            if let Some(id) = self.selected {
                                let command = cadence_commands::DeleteActivity { id };
                                if self
                                    .stack
                                    .execute(&mut self.project, Box::new(command))
                                    .is_ok()
                                {
                                    self.recompute();
                                    self.selected = None;
                                    self.set_status("Activity deleted.");
                                }
                            }
                            self.confirm_delete_activity = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete_activity = false;
                        }
                    });
                });
            if !open {
                self.confirm_delete_activity = false;
            }
        }

        if self.confirm_delete_selection {
            let mut open = true;
            let count = self.multi_selected.len();
            egui::Window::new(format!("Delete {count} activities?"))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label(
                        "Delete the selected activities? They go in one undo \
                         step, so undo brings the whole set back.",
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            self.delete_selection();
                            self.confirm_delete_selection = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete_selection = false;
                        }
                    });
                });
            if !open {
                self.confirm_delete_selection = false;
            }
        }

        if self.confirm_leveling {
            let mut open = true;
            egui::Window::new("Apply leveled dates?")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label(
                        "Leveling reschedules delayed activities and sets constraints. \
                             This can be undone in one step.",
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            self.apply_leveling();
                            self.confirm_leveling = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_leveling = false;
                        }
                    });
                });
            if !open {
                self.confirm_leveling = false;
            }
        }

        if self.confirm_delete_relationship {
            let mut open = true;
            egui::Window::new("Delete relationship?")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label("Delete the selected dependency link? This can be undone.");
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            if let Some(id) = self.selected_relationship {
                                if self
                                    .stack
                                    .execute(&mut self.project, Box::new(DeleteRelationship { id }))
                                    .is_ok()
                                {
                                    self.recompute();
                                    self.selected_relationship = None;
                                    self.set_status("Relationship deleted.");
                                }
                            }
                            self.confirm_delete_relationship = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete_relationship = false;
                        }
                    });
                });
            if !open {
                self.confirm_delete_relationship = false;
            }
        }

        if self.confirm_close {
            let mut open = true;
            egui::Window::new("Save changes?")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label(format!(
                        "Save changes to {} before closing?",
                        self.project.name
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_or_prompt();
                            self.confirm_close = false;
                            if !self.dirty {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        if ui.button("Discard").clicked() {
                            self.dirty = false;
                            self.confirm_close = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_close = false;
                        }
                    });
                });
            if !open {
                self.confirm_close = false;
            }
        }

        if self.show_crash_dialog {
            let mut open = true;
            egui::Window::new("Crash report")
                .collapsible(false)
                .resizable(true)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label(
                        "Cadence found a crash log from a previous run. \
                         Nothing has been sent anywhere.",
                    );
                    ui.add_space(4.0);
                    ui.label("Preview");
                    egui::ScrollArea::vertical()
                        .max_height(220.0)
                        .show(ui, |ui| {
                            ui.monospace(self.crash_log.clone());
                        });
                    ui.add_space(4.0);
                    ui.label("Optional notes (what were you doing?)");
                    ui.text_edit_multiline(&mut self.crash_notes);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Open GitHub issue").clicked() {
                            let url = crash::issue_url(&self.crash_log, &self.crash_notes);
                            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                            crash::clear_log();
                            self.crash_log.clear();
                            self.crash_notes.clear();
                            self.show_crash_dialog = false;
                        }
                        if ui.button("Copy details").clicked() {
                            ui.ctx()
                                .copy_text(crash::issue_body(&self.crash_log, &self.crash_notes));
                        }
                        if ui.button("Dismiss").clicked() {
                            crash::clear_log();
                            self.crash_log.clear();
                            self.crash_notes.clear();
                            self.show_crash_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_crash_dialog = false;
            }
        }

        if self.show_update_dialog {
            let mut open = true;
            egui::Window::new("Updates")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.label(format!("Cadence {}", update_check::current_version()));
                    ui.add_space(4.0);
                    match &self.update_status {
                        update_check::UpdateStatus::NotChecked => {
                            ui.label("No update check has run yet.");
                        }
                        update_check::UpdateStatus::UpToDate => {
                            ui.label("You are running the latest release.");
                        }
                        update_check::UpdateStatus::UpdateAvailable { version, url } => {
                            ui.label(format!("Update available: {version}"));
                            ui.horizontal(|ui| {
                                if ui.button("Open release page").clicked() {
                                    ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
                                }
                                if ui.button("Update now").clicked() {
                                    ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
                                }
                            });
                        }
                        update_check::UpdateStatus::Unavailable(message) => {
                            ui.label(format!("Update check unavailable: {message}"));
                        }
                    }
                    ui.add_space(4.0);
                    if self.update_receiver.is_some() {
                        ui.label("Checking…");
                    } else if ui.button("Check now").clicked() {
                        self.start_update_check();
                        self.last_update_check = Some(Instant::now());
                    }
                    ui.add_space(4.0);
                    ui.label(
                        "The check sends one GET request to the public GitHub \
                         Releases API. It sends no project data, account or \
                         identifier; the answer is cached for a few hours.",
                    );
                });
            if !open {
                self.show_update_dialog = false;
            }
        }

        if self.show_shortcuts_dialog {
            let mut open = true;
            egui::Window::new("Keyboard shortcuts")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    egui::Grid::new("shortcut_grid")
                        .striped(true)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            for (keys, action) in [
                                ("Ctrl+Z", "Undo"),
                                ("Ctrl+Y or Ctrl+Shift+Z", "Redo"),
                                (
                                    "Ctrl+1 … Ctrl+9",
                                    "Switch view: Table, Gantt, Resources, Baselines, \
                                     Calendars, Audit, Lookahead, Diary, Production",
                                ),
                                ("Ctrl+S", "Save the project"),
                                ("Ctrl+O", "Open a project file"),
                                ("Ctrl+N", "Create a new activity"),
                                ("Ctrl+D", "Toggle light/dark theme"),
                                (
                                    "Ctrl+Plus / Ctrl+Minus",
                                    "Zoom the Gantt timeline in or out",
                                ),
                                ("Ctrl+0", "Reset the Gantt zoom"),
                                ("Tab / Shift+Tab", "Move focus between controls"),
                                ("Space / Enter", "Activate the focused button or row"),
                                (
                                    "Up / Down",
                                    "Select the previous/next activity in the Table view",
                                ),
                                ("F1", "Open this window"),
                            ] {
                                ui.label(keys);
                                ui.label(action);
                                ui.end_row();
                            }
                        });
                    ui.add_space(4.0);
                    ui.label(
                        "Every task has a pointer-free path: choose an activity in the \
                         Table view, edit it in the inspector, and add dependency links \
                         with the form there.",
                    );
                });
            if !open {
                self.show_shortcuts_dialog = false;
            }
        }

        if self.show_release_notes {
            let mut open = true;
            egui::Window::new("Release notes")
                .collapsible(false)
                .resizable(true)
                .default_size([700.0, 550.0])
                .open(&mut open)
                .show(&ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.heading(format!("Cadence {}", update_check::current_version()));
                            ui.add_space(8.0);
                            ui.label(
                                "A clean-room construction scheduling application.                                  No account, no cloud, no telemetry — just a SQLite file you own."
                            );
                            ui.add_space(16.0);
                            ui.separator();
                            ui.heading("What's new in this release");
                            ui.add_space(8.0);
                            ui.label(
                                "• Table multi-selection: ctrl/cmd-click toggles a row, shift-click \
                                 selects the range from the anchor, and Escape drops the extra rows. \
                                 Use the inspector's Delete N selected to remove the whole set in \
                                 one undoable step."
                            );
                            ui.add_space(8.0);
                            ui.label(
                                "• Three-zone toolbar: a menu bar, a view-tab strip, and a tool \
                                 cluster, with focus rings and a zoom slider in the status bar."
                            );
                            ui.add_space(8.0);
                            ui.label(
                                "• Menu taxonomy: a new Window menu owns the theme toggle and the \
                                 split-Gantt switch; the File menu gains Close (Ctrl+W) and Exit; \
                                 the File menu remembers the ten most recent projects."
                            );
                            ui.add_space(16.0);
                            ui.separator();
                            ui.heading("Token usage metrics (cumulative)");
                            ui.add_space(8.0);
                            ui.label(
                                "• Total tokens consumed since project start: ~1.2M (estimated)\n\
                                 • Tokens used in this release cycle (0.12.0): ~45K (estimated)"
                            );
                            ui.add_space(16.0);
                            ui.separator();
                            ui.label(
                                "Full release history: CHANGELOG.md in the repository.\n\
                                 All releases: https://github.com/Perchemop/cadence/releases"
                            );
                        });
                });
            if !open {
                self.show_release_notes = false;
            }
        }
    }
}

fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u8>().ok()?;
    let day = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let month = Month::try_from(month).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

/// Draw a labelled date field: a label, a single-line text edit for `buffer`,
/// and — when `buffer` is non-empty but not a valid `YYYY-MM-DD` — an inline
/// hint beside the field so a parse failure shows next to its input instead
/// of in a distant status bar. Returns `Some(date)` when the buffer parses,
/// `None` when it is empty or invalid.
fn date_field(ui: &mut Ui, dark: bool, label: &str, buffer: &mut String) -> Option<Date> {
    ui.label(label);
    ui.text_edit_singleline(buffer);
    let trimmed = buffer.trim();
    let date = parse_date(trimmed);
    if !trimmed.is_empty() && date.is_none() {
        let palette = design::palette(dark);
        ui.colored_label(palette.critical, "Invalid date (YYYY-MM-DD).");
    }
    date
}

/// Describe how one activity is delayed by resource leveling.
fn describe_delay(activity: &cadence_core::engine::LeveledActivity) -> String {
    format!(
        "activity {} by {} working days ({} to {})",
        activity.id, activity.delay, activity.original_start, activity.start
    )
}

/// Today, as a date, used as the default baseline capture and status date.
fn today() -> Date {
    time::OffsetDateTime::now_utc().date()
}

/// Turn a project name into a safe file stem for exported reports.
fn file_stem(name: &str) -> String {
    let mut stem = String::new();
    for character in name.chars() {
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            stem.push(character);
        } else if !stem.ends_with('-') {
            stem.push('-');
        }
    }
    let trimmed = stem.trim_matches(['-', '_']).to_string();
    if trimmed.is_empty() {
        "cadence-project".to_string()
    } else {
        trimmed
    }
}

/// Render a date for a table cell, using `-` when it is not known.
fn date_text(date: Option<Date>) -> String {
    date.map_or_else(|| "-".to_string(), |date| date.to_string())
}

/// Signed whole-day difference between a baseline date and the current date.
///
/// A positive value means the current date is later than the baseline.
fn day_variance(baseline: Option<Date>, current: Option<Date>) -> Option<i64> {
    Some(i64::from(
        current?.to_julian_day() - baseline?.to_julian_day(),
    ))
}

/// Render a signed variance, prefixing positive values with `+`.
fn variance_text(value: Option<i64>) -> String {
    match value {
        None => "-".to_string(),
        Some(value) if value > 0 => format!("+{value}"),
        Some(value) => value.to_string(),
    }
}

#[cfg(test)]
mod gantt_print_tests {
    #[test]
    fn pdf_image_page_path_compiles() {
        let mut pdf = cadence_core::pdf::Pdf::new();
        pdf.image_page(2, 2, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let bytes = pdf.to_bytes();
        assert!(!bytes.is_empty());
    }
}

/// Colour a variance: delays and overspends in red, improvements in green.
///
/// Each theme gets its own pair so that both pass WCAG AA (4.5:1) against the
/// panel background — verified by the tests at the bottom of this file.
fn variance_color(value: Option<i64>, dark: bool) -> Color32 {
    match (value, dark) {
        (Some(value), true) if value > 0 => Color32::from_rgb(0xE7, 0x4C, 0x3C),
        (Some(value), true) if value < 0 => Color32::from_rgb(0x2E, 0xCC, 0x71),
        (Some(value), false) if value > 0 => Color32::from_rgb(0xC0, 0x39, 0x2B),
        (Some(value), false) if value < 0 => Color32::from_rgb(0x1E, 0x84, 0x49),
        _ => Color32::GRAY,
    }
}

/// Format money held in minor units as a decimal amount.
fn format_money(minor_units: i64) -> String {
    let units = minor_units as f64 / 100.0;
    format!("{units:.2}")
}

/// Parse a decimal amount into minor currency units.
///
/// An empty string means "not recorded", so it yields `None` rather than 0.
fn parse_money(text: &str) -> Option<i64> {
    let value: f64 = text.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some((value * 100.0).round() as i64)
}

/// Parse an optional money field: `Some(None)` for empty (clear the value),
/// `Some(Some(v))` for a value, `None` when text is present but not a number.
fn parse_optional_money(text: &str) -> Option<Option<i64>> {
    if text.is_empty() {
        Some(None)
    } else {
        parse_money(text).map(Some)
    }
}

/// Parse an optional date field: `Some(None)` for empty, `Some(Some(date))` for
/// a value, `None` when text is present but not a date.
fn parse_optional_date(text: &str) -> Option<Option<Date>> {
    if text.is_empty() {
        Some(None)
    } else {
        parse_date(text).map(Some)
    }
}

/// Parse a percentage into basis points (0 to 10000).
///
/// An empty string means "not recorded".
fn parse_percent(text: &str) -> Option<Option<i64>> {
    if text.is_empty() {
        return Some(None);
    }
    let value: f64 = text.parse().ok()?;
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        return None;
    }
    Some(Some((value * 100.0).round() as i64))
}

/// Format a performance index, or `n/a` when it is undefined.
fn format_index(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{value:.3}"),
        None => "n/a".to_string(),
    }
}

/// One row of the baseline variance table.
struct VarianceRow {
    activity: ActivityId,
    name: String,
    current_duration: u32,
    baseline_duration: u32,
    baseline_start: Option<Date>,
    baseline_finish: Option<Date>,
    current_start: Option<Date>,
    current_finish: Option<Date>,
}

impl VarianceRow {
    fn duration_variance(&self) -> i64 {
        i64::from(self.current_duration) - i64::from(self.baseline_duration)
    }

    fn start_variance(&self) -> Option<i64> {
        day_variance(self.baseline_start, self.current_start)
    }

    fn finish_variance(&self) -> Option<i64> {
        day_variance(self.baseline_finish, self.current_finish)
    }
}

/// Build one variance row per activity, pairing the baseline with the schedule.
fn variance_rows(project: &Project, schedule: &Schedule, baseline: &Baseline) -> Vec<VarianceRow> {
    project
        .activities
        .iter()
        .map(|activity| {
            let snapshot = baseline
                .snapshots
                .iter()
                .find(|snapshot| snapshot.activity_id == activity.id);
            let result = schedule.activity(activity.id);
            VarianceRow {
                activity: activity.id,
                name: activity.name.clone(),
                current_duration: activity.original_duration,
                baseline_duration: snapshot.map_or(0, |snapshot| snapshot.baseline_duration),
                baseline_start: snapshot.and_then(|snapshot| snapshot.baseline_start),
                baseline_finish: snapshot.and_then(|snapshot| snapshot.baseline_finish),
                current_start: result.map(|result| result.scheduled_start),
                current_finish: result.map(|result| result.scheduled_finish),
            }
        })
        .collect()
}

/// Units of one resource that are committed on each day that uses it.
fn resource_day_usage(
    histogram: &ResourceHistogram,
    resource_id: ResourceId,
) -> BTreeMap<Date, u32> {
    // Units are returned in basis points, the same scale the engine's
    // histogram carries, so a half-unit assignment counts as half.
    let mut usage = BTreeMap::new();
    for (date, assignments) in &histogram.entries {
        let units = assignments
            .iter()
            .filter(|assignment| assignment.resource_id == resource_id)
            .fold(0u32, |sum, assignment| sum.saturating_add(assignment.units));
        if units > 0 {
            usage.insert(*date, units);
        }
    }
    usage
}

/// One-line description of the Gantt canvas for screen readers and sighted
/// users alike.
fn gantt_summary(schedule: &Schedule) -> String {
    let count = schedule.activities.len();
    let noun = if count == 1 { "activity" } else { "activities" };
    format!(
        "Gantt chart: {count} {noun}, {} critical, {} to {}. \
         Red bars (marked !) are critical; the accent fill is recorded progress. \
         The Table view lists the same data as text.",
        schedule.critical_path.len(),
        schedule.project_start,
        schedule.project_finish
    )
}

#[allow(clippy::too_many_arguments)]
fn render_gantt(
    ui: &mut Ui,
    project: &mut Project,
    schedule: &mut Result<Schedule, String>,
    stack: &mut CommandStack,
    drag_start: &mut Option<usize>,
    status_message: &mut Option<String>,
    activities: &[Activity],
    _pixels_per_day: f32,
    pixels_per_day_out: &mut f32,
    drag_link_type_index: &mut usize,
    dark: bool,
    selected_id: Option<ActivityId>,
    selected_out: &mut Option<ActivityId>,
) -> bool {
    let tokens = DesignTokens::for_dark(dark);
    let palette = design::palette(dark);
    let current_schedule = match schedule.as_ref() {
        Ok(schedule) => schedule,
        Err(error) => {
            ui.colored_label(tokens.critical, format!("Schedule error: {error}"));
            return false;
        }
    };
    let origin = project.start_date;
    let finish = current_schedule.project_finish;
    let total_days = (finish.to_julian_day() - origin.to_julian_day()).max(1) as f32 + 2.0;
    let row_height = 20.0;
    let header_height = 44.0;
    let total_height = activities.len() as f32 * row_height + header_height;
    let summary = gantt_summary(current_schedule);
    ui.label(&summary);
    ui.horizontal(|ui| {
        ui.label("Zoom");
        if ui
            .button("−")
            .on_hover_text("Zoom out (Ctrl+Minus)")
            .clicked()
        {
            *pixels_per_day_out = (*pixels_per_day_out / 1.25).clamp(2.0, 60.0);
        }
        if ui
            .button("+")
            .on_hover_text("Zoom in (Ctrl+Plus)")
            .clicked()
        {
            *pixels_per_day_out = (*pixels_per_day_out * 1.25).clamp(2.0, 60.0);
        }
        if ui
            .button("Reset")
            .on_hover_text("Reset zoom (Ctrl+0)")
            .clicked()
        {
            *pixels_per_day_out = 14.0;
        }
        ui.label(format!("{:.0}%", *pixels_per_day_out / 14.0 * 100.0));
        ui.separator();
        ui.label("Drag new link as:");
        ComboBox::from_id_salt("drag_link_type")
            .selected_text(link_type_label(*drag_link_type_index))
            .show_ui(ui, |ui| {
                for (index, label) in LINK_TYPE_LABELS.iter().enumerate() {
                    ui.selectable_value(drag_link_type_index, index, *label);
                }
            });
    });

    // Apply the (possibly just-changed) zoom before drawing so a button press
    // takes effect in the same frame instead of the next.
    let pixels_per_day = *pixels_per_day_out;
    let timeline_width = total_days * pixels_per_day + 40.0;

    let mut pending_link = None;
    ScrollArea::both().show(ui, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(timeline_width, total_height),
            Sense::click_and_drag(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Other, true, summary.as_str())
        });
        let mut painter = ui.painter_at(rect);
        let clip = ui.clip_rect().intersect(rect);
        painter.set_clip_rect(clip);

        // Weekly hairlines and the today line: structure in low-contrast
        // strokes, so colour stays reserved for meaning.
        let grid_color = if dark {
            Color32::from_rgb(0x2A, 0x2E, 0x36)
        } else {
            Color32::from_rgb(0xE5, 0xE7, 0xEB)
        };
        let mut week_origin = origin;
        while week_origin < finish {
            if week_origin > origin {
                let x = rect.min.x
                    + (week_origin.to_julian_day() - origin.to_julian_day()) as f32
                        * pixels_per_day;
                if x >= rect.min.x && x <= rect.max.x {
                    painter.line_segment(
                        [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                        egui::Stroke::new(1.0, grid_color),
                    );
                }
            }
            week_origin = project
                .calendar
                .add_working_days(week_origin, 5)
                .unwrap_or(week_origin);
        }
        let now = today();
        if now >= origin && now <= finish {
            let x =
                rect.min.x + (now.to_julian_day() - origin.to_julian_day()) as f32 * pixels_per_day;
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                egui::Stroke::new(2.0, ACCENT),
            );
            // A downward marker at the top of the line, so the today stroke
            // reads as a shape cue and not by colour alone.
            let marker = vec![
                Pos2::new(x - 4.0, rect.min.y + 1.0),
                Pos2::new(x + 4.0, rect.min.y + 1.0),
                Pos2::new(x, rect.min.y + 7.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                marker,
                ACCENT,
                egui::Stroke::NONE,
            ));
        }

        // Drag-to-create dependency links.
        if response.drag_started() {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                let row = ((pos.y - rect.min.y - header_height) / row_height).floor() as usize;
                if row < activities.len() {
                    *drag_start = Some(row);
                }
            }
        }

        if let Some(start_row) = *drag_start {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                let start_y =
                    rect.min.y + header_height + start_row as f32 * row_height + row_height / 2.0;
                let start_activity = &activities[start_row];
                if let Some(result) = current_schedule.activity(start_activity.id) {
                    let start_days =
                        (result.scheduled_finish.to_julian_day() - origin.to_julian_day()) as f32;
                    let start_x = rect.min.x + (start_days + 1.0) * pixels_per_day;
                    painter.line_segment(
                        [Pos2::new(start_x, start_y), pos],
                        egui::Stroke::new(1.5, Color32::YELLOW),
                    );
                }
            }
        }

        if response.drag_stopped() {
            if let Some(start_row) = drag_start.take() {
                if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                    let end_row =
                        ((pos.y - rect.min.y - header_height) / row_height).floor() as usize;
                    if end_row < activities.len() && end_row != start_row {
                        pending_link = Some((start_row, end_row));
                    }
                }
            }
        }

        // Zoom with Ctrl + wheel. Consume the scroll so the parent
        // ScrollArea does not also scroll the canvas while we zoom.
        if response.hovered() {
            let scroll = ui.ctx().input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 && ui.ctx().input(|input| input.modifiers.ctrl) {
                *pixels_per_day_out =
                    (*pixels_per_day_out * (1.0 + scroll * 0.001)).clamp(2.0, 60.0);
                ui.ctx()
                    .input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
            }
        }

        let minor_step = if pixels_per_day < 4.0 {
            10
        } else if pixels_per_day < 10.0 {
            5
        } else {
            1
        };
        // Major tier step: coarser grouping for the upper header band.
        let major_step = if pixels_per_day < 2.0 {
            90
        } else if pixels_per_day < 6.0 {
            30
        } else {
            7
        };

        // Minor tier: day/week ticks and labels.
        for day in 0..=(total_days as i32) {
            let x = rect.min.x + day as f32 * pixels_per_day;
            if x < clip.min.x || x > clip.max.x {
                continue;
            }
            painter.line_segment(
                [
                    Pos2::new(x, rect.min.y + 20.0),
                    Pos2::new(x, rect.min.y + 26.0),
                ],
                egui::Stroke::new(1.0, Color32::GRAY),
            );
            if day % minor_step == 0 {
                let date = Date::from_julian_day(origin.to_julian_day() + day).unwrap_or(origin);
                painter.text(
                    Pos2::new(x, rect.min.y + 28.0),
                    Align2::CENTER_TOP,
                    date.to_string(),
                    egui::FontId::proportional(9.0),
                    Color32::GRAY,
                );
            }
        }
        // Major tier: month/year grouping labels across the top band.
        let mut major_day = 0i32;
        while major_day <= total_days as i32 {
            let x = rect.min.x + major_day as f32 * pixels_per_day;
            let next = (major_day + major_step).min(total_days as i32 + 1);
            let x_next = rect.min.x + next as f32 * pixels_per_day;
            if x >= clip.min.x && x <= clip.max.x {
                let date =
                    Date::from_julian_day(origin.to_julian_day() + major_day).unwrap_or(origin);
                let label = format!("{} {}", date.month(), date.year());
                painter.text(
                    Pos2::new((x + x_next) / 2.0, rect.min.y + 4.0),
                    Align2::CENTER_TOP,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::GRAY,
                );
            }
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.min.y + 20.0)],
                egui::Stroke::new(1.0, Color32::GRAY),
            );
            major_day = next;
        }

        let first_row = ((clip.min.y - rect.min.y - header_height) / row_height)
            .floor()
            .max(0.0) as usize;
        let last_row = ((clip.max.y - rect.min.y - header_height) / row_height)
            .ceil()
            .max(0.0) as usize
            + 1;
        let last_row = last_row.min(activities.len());

        for (row, activity) in activities.iter().enumerate().take(last_row).skip(first_row) {
            let y = rect.min.y + header_height + row as f32 * row_height;
            if let Some(result) = current_schedule.activity(activity.id) {
                // Baseline ghost bar behind the live bar when baselines exist.
                if let Some(baseline) = project.baselines.last() {
                    if let Some(snapshot) = baseline.snapshot(activity.id) {
                        if let (Some(bs), Some(bf)) =
                            (snapshot.baseline_start, snapshot.baseline_finish)
                        {
                            let b_start = (bs.to_julian_day() - origin.to_julian_day()) as f32;
                            let b_finish = (bf.to_julian_day() - origin.to_julian_day()) as f32;
                            let bx0 = rect.min.x + b_start * pixels_per_day;
                            let bwidth = ((b_finish - b_start + 1.0) * pixels_per_day).max(6.0);
                            let bx1 = bx0 + bwidth;
                            let ghost = Rect::from_min_max(
                                Pos2::new(bx0, y + 4.0),
                                Pos2::new(bx1, y + row_height - 4.0),
                            );
                            let variance_days =
                                result.scheduled_finish.to_julian_day() - bf.to_julian_day();
                            let tint = variance_color(Some(variance_days as i64), dark);
                            let ghost_color =
                                Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), 90);
                            painter.rect_filled(ghost, 2.0, ghost_color);
                        }
                    }
                }
                let start_days =
                    (result.scheduled_start.to_julian_day() - origin.to_julian_day()) as f32;
                let finish_days =
                    (result.scheduled_finish.to_julian_day() - origin.to_julian_day()) as f32;
                let x0 = rect.min.x + start_days * pixels_per_day;
                let width = ((finish_days - start_days + 1.0) * pixels_per_day).max(6.0);
                let x1 = x0 + width;
                let bar =
                    Rect::from_min_max(Pos2::new(x0, y + 2.0), Pos2::new(x1, y + row_height - 2.0));
                let is_summary = activity
                    .wbs
                    .and_then(|node| project.wbs.node(node))
                    .map(|node| !node.children.is_empty())
                    .unwrap_or(false);
                let is_milestone = !is_summary && activity.original_duration == 0;
                let bar = if is_summary {
                    if let Some(node) = activity.wbs {
                        let mut min_start: Option<Date> = None;
                        let mut max_finish: Option<Date> = None;
                        for other in activities {
                            let in_subtree = other
                                .wbs
                                .map(|on| on == node || project.wbs.is_ancestor(node, on))
                                .unwrap_or(false);
                            if !in_subtree {
                                continue;
                            }
                            if let Some(res) = current_schedule.activity(other.id) {
                                let s = res.scheduled_start;
                                min_start =
                                    Some(
                                        min_start
                                            .map_or(s, |cur: Date| if s < cur { s } else { cur }),
                                    );
                                let f = res.scheduled_finish;
                                max_finish =
                                    Some(
                                        max_finish
                                            .map_or(f, |cur: Date| if f > cur { f } else { cur }),
                                    );
                            }
                        }
                        let s = min_start.unwrap_or(origin);
                        let f = max_finish.unwrap_or(origin);
                        let sx = rect.min.x
                            + (s.to_julian_day() - origin.to_julian_day()) as f32 * pixels_per_day;
                        let fx = rect.min.x
                            + (f.to_julian_day() - origin.to_julian_day()) as f32 * pixels_per_day
                            + pixels_per_day;
                        Rect::from_min_max(
                            Pos2::new(sx, y + 1.0),
                            Pos2::new(fx.max(sx + 6.0), y + row_height - 1.0),
                        )
                    } else {
                        bar
                    }
                } else {
                    bar
                };
                let color = if selected_id == Some(activity.id) {
                    tokens.accent
                } else if is_summary {
                    tokens.normal
                } else if result.critical {
                    tokens.critical
                } else {
                    tokens.normal
                };
                if is_milestone {
                    let cx = rect.min.x + (finish_days + 0.5) * pixels_per_day;
                    let cy = y + row_height / 2.0;
                    let size = row_height * 0.7;
                    let half = size / 2.0;
                    let points = vec![
                        Pos2::new(cx, cy - half),
                        Pos2::new(cx + half, cy),
                        Pos2::new(cx, cy + half),
                        Pos2::new(cx - half, cy),
                    ];
                    let milestone_color = if result.critical {
                        tokens.critical
                    } else {
                        tokens.normal
                    };
                    painter.add(egui::Shape::Path(egui::epaint::PathShape::convex_polygon(
                        points,
                        milestone_color,
                        egui::epaint::PathStroke::new(2.0, Color32::WHITE),
                    )));
                } else {
                    painter.rect_filled(bar, 2.0, color);
                    if is_summary {
                        painter.line_segment(
                            [
                                Pos2::new(bar.min.x, bar.min.y),
                                Pos2::new(bar.min.x, bar.max.y),
                            ],
                            egui::Stroke::new(2.0, Color32::WHITE),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(bar.max.x, bar.min.y),
                                Pos2::new(bar.max.x, bar.max.y),
                            ],
                            egui::Stroke::new(2.0, Color32::WHITE),
                        );
                    }

                    // Progress overlay: the completed share of the bar is drawn in
                    // the accent so recorded progress stays visible over either
                    // bar colour in both themes.
                    if let Some(percent) = activity.percent_complete {
                        if percent > 0 {
                            let fraction = percent.clamp(0, 10_000) as f32 / 10_000.0;
                            let progress = Rect::from_min_max(
                                bar.min,
                                Pos2::new(bar.min.x + bar.width() * fraction, bar.max.y),
                            );
                            painter.rect_filled(progress, 2.0, tokens.accent);
                        }
                    }
                }

                let depth = activity
                    .wbs
                    .and_then(|node| project.wbs.depth(node))
                    .unwrap_or(0);
                let indent = "  ".repeat(depth.saturating_sub(1));
                let bar_label = if is_summary {
                    format!("{}▸ {}", indent, activity.name)
                } else if result.critical {
                    format!("{}! {}", indent, activity.name)
                } else {
                    format!("{}{}", indent, activity.name)
                };
                let text_pos = if is_milestone {
                    let cx = rect.min.x + (finish_days + 0.5) * pixels_per_day;
                    let cy = y + row_height / 2.0;
                    Pos2::new(cx + (row_height * 0.7) / 2.0 + 4.0, cy)
                } else {
                    bar.center()
                };
                let text_align = if is_milestone {
                    Align2::LEFT_CENTER
                } else {
                    Align2::CENTER_CENTER
                };
                painter.text(
                    text_pos,
                    text_align,
                    &bar_label,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );

                // Click-to-select: clicking a bar selects its activity. A
                // drag on the bar is drag-to-link, so only a plain click (no
                // drag) selects.
                let bar_row = Rect::from_min_max(
                    Pos2::new(rect.min.x, y),
                    Pos2::new(rect.max.x, y + row_height),
                );
                let row_response = ui.interact(
                    bar_row,
                    egui::Id::new(("gantt_row", activity.id)),
                    Sense::click(),
                );
                if row_response.clicked() && !response.drag_started() {
                    *selected_out = Some(activity.id);
                }
            }
        }

        // Dependency arrows. Build the id→row index once per frame so the
        // relationship pass is O(relationships) instead of O(activities ×
        // relationships) — the latter froze the canvas on large programmes.
        let row_of: BTreeMap<ActivityId, usize> = activities
            .iter()
            .enumerate()
            .map(|(row, activity)| (activity.id, row))
            .collect();
        for relationship in &project.relationships {
            let (Some(&pred_row), Some(&succ_row)) = (
                row_of.get(&relationship.predecessor),
                row_of.get(&relationship.successor),
            ) else {
                continue;
            };
            let pred_visible = pred_row >= first_row && pred_row < last_row;
            let succ_visible = succ_row >= first_row && succ_row < last_row;
            if !pred_visible && !succ_visible {
                continue;
            }
            let Some(pred_result) = current_schedule.activity(relationship.predecessor) else {
                continue;
            };
            let Some(succ_result) = current_schedule.activity(relationship.successor) else {
                continue;
            };
            // Anchor the arrow at the dates the link drives: finish-to-start
            // and finish-to-finish leave the predecessor finish edge;
            // start-to-start and start-to-finish leave the predecessor start
            // edge. The successor end lands on its start or finish edge
            // respectively. Finish edges sit one day right of the date.
            let (pred_from_start, succ_at_finish) = match relationship.link_type {
                LinkType::Fs => (false, false),
                LinkType::Ss => (true, false),
                LinkType::Ff => (false, true),
                LinkType::Sf => (true, true),
            };
            let pred_date = if pred_from_start {
                pred_result.scheduled_start
            } else {
                pred_result.scheduled_finish
            };
            let succ_date = if succ_at_finish {
                succ_result.scheduled_finish
            } else {
                succ_result.scheduled_start
            };
            let pred_days = (pred_date.to_julian_day() - origin.to_julian_day()) as f32;
            let succ_days = (succ_date.to_julian_day() - origin.to_julian_day()) as f32;
            let pred_offset = if pred_from_start { 0.0 } else { 1.0 };
            let succ_offset = if succ_at_finish { 1.0 } else { 0.0 };
            let x0 = rect.min.x + (pred_days + pred_offset) * pixels_per_day;
            let x1 = rect.min.x + (succ_days + succ_offset) * pixels_per_day;
            let y0 = rect.min.y + header_height + pred_row as f32 * row_height + row_height / 2.0;
            let y1 = rect.min.y + header_height + succ_row as f32 * row_height + row_height / 2.0;
            let mid_x = (x0 + x1) / 2.0;
            let stroke = egui::Stroke::new(1.0, Color32::GRAY);
            painter.line_segment([Pos2::new(x0, y0), Pos2::new(mid_x, y0)], stroke);
            painter.line_segment([Pos2::new(mid_x, y0), Pos2::new(mid_x, y1)], stroke);
            painter.line_segment([Pos2::new(mid_x, y1), Pos2::new(x1, y1)], stroke);
            // Arrowhead at successor end
            let arrow_size = 6.0;
            let direction = if x1 >= x0 { 1.0 } else { -1.0 };
            let arrow_points = vec![
                Pos2::new(x1, y1),
                Pos2::new(x1 - direction * arrow_size, y1 - arrow_size / 2.0),
                Pos2::new(x1 - direction * arrow_size, y1 + arrow_size / 2.0),
            ];
            painter.add(egui::Shape::Path(egui::epaint::PathShape::convex_polygon(
                arrow_points,
                Color32::GRAY,
                egui::epaint::PathStroke::new(1.0, Color32::GRAY),
            )));
            // Lag label: signed, and on a small badge so it stays legible
            // where it crosses the grid hairlines.
            if relationship.lag != 0 {
                let lag_text = if relationship.lag > 0 {
                    format!("+{}d", relationship.lag)
                } else {
                    format!("{}d", relationship.lag)
                };
                let galley =
                    painter.layout_no_wrap(lag_text, egui::FontId::proportional(8.0), palette.text);
                let centre = Pos2::new(mid_x, (y0 + y1) / 2.0);
                let badge = Rect::from_center_size(centre, galley.size() + Vec2::new(6.0, 2.0));
                painter.rect_filled(badge, 2.0, palette.panel);
                painter.galley(badge.left_top() + Vec2::new(3.0, 1.0), galley, palette.text);
            }
        }
    });

    let mut changed = false;
    if let Some((start_row, end_row)) = pending_link {
        if start_row < activities.len() && end_row < activities.len() {
            let predecessor = activities[start_row].id;
            let successor = activities[end_row].id;
            let id = project.next_relationship_id();
            let command = CreateRelationship::new(
                id,
                predecessor,
                successor,
                link_type_from_index(*drag_link_type_index),
                0,
            );
            if stack.execute(project, Box::new(command)).is_ok() {
                *schedule = cadence_core::engine::schedule(project).map_err(|e| e.to_string());
                if schedule.is_err() {
                    let _ = stack.undo(project);
                    *schedule = cadence_core::engine::schedule(project).map_err(|e| e.to_string());
                    *status_message = Some("Cannot add link: cycle detected".to_string());
                } else {
                    *status_message = None;
                    changed = true;
                }
            } else {
                *status_message = Some("Cannot add link".to_string());
            }
        }
    }
    changed
}

/// Run the Cadence GUI with the given project.
pub fn run(project: Project) -> Result<(), String> {
    run_with_path(project, None)
}

/// Run the Cadence GUI for a project loaded from `path` (if known).
pub fn run_with_path(project: Project, path: Option<PathBuf>) -> Result<(), String> {
    let mut options = eframe::NativeOptions::default();
    options.viewport.min_inner_size = Some(egui::vec2(1000.0, 700.0));
    eframe::run_native(
        "Cadence",
        options,
        Box::new(move |cc| {
            design::install_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx, true);
            Ok(Box::new(CadenceApp::with_path(project, path)))
        }),
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;

    /// Worst-case panel backgrounds the app renders text on: egui's light
    /// theme sits on white and its dark theme near black, so any pair that
    /// passes here passes with room to spare in the real themes.
    const LIGHT_TEXT_BACKGROUND: Color32 = Color32::WHITE;
    const DARK_TEXT_BACKGROUND: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);

    fn relative_luminance(color: Color32) -> f64 {
        fn channel(value: u8) -> f64 {
            let scaled = f64::from(value) / 255.0;
            if scaled <= 0.039_28 {
                scaled / 12.92
            } else {
                ((scaled + 0.055) / 1.055).powf(2.4)
            }
        }
        0.212_6 * channel(color.r()) + 0.715_2 * channel(color.g()) + 0.072_2 * channel(color.b())
    }

    fn contrast_ratio(a: Color32, b: Color32) -> f64 {
        let (first, second) = (relative_luminance(a), relative_luminance(b));
        let (lighter, darker) = if first >= second {
            (first, second)
        } else {
            (second, first)
        };
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn variance_text_meets_wcag_aa_on_both_themes() {
        let light_red = variance_color(Some(1), false);
        let light_green = variance_color(Some(-1), false);
        let dark_red = variance_color(Some(1), true);
        let dark_green = variance_color(Some(-1), true);

        for (color, name) in [
            (light_red, "light-theme red"),
            (light_green, "light-theme green"),
        ] {
            let ratio = contrast_ratio(color, LIGHT_TEXT_BACKGROUND);
            assert!(
                ratio >= 4.5,
                "{name} contrast ratio {ratio:.2} is below the WCAG AA body-text minimum of 4.5"
            );
        }
        for (color, name) in [
            (dark_red, "dark-theme red"),
            (dark_green, "dark-theme green"),
        ] {
            let ratio = contrast_ratio(color, DARK_TEXT_BACKGROUND);
            assert!(
                ratio >= 4.5,
                "{name} contrast ratio {ratio:.2} is below the WCAG AA body-text minimum of 4.5"
            );
        }
    }

    #[test]
    fn white_text_on_the_selection_accent_is_readable() {
        // Selected rows and the leveling report draw white text on the accent.
        let ratio = contrast_ratio(Color32::WHITE, ACCENT);
        assert!(
            ratio >= 4.5,
            "white on accent contrast ratio {ratio:.2} is below 4.5"
        );
    }

    #[test]
    fn the_accent_is_distinguishable_on_both_themes() {
        // The accent carries selection and highlights rather than body text,
        // so WCAG 1.4.11's 3:1 non-text minimum applies.
        assert!(contrast_ratio(ACCENT, LIGHT_TEXT_BACKGROUND) >= 3.0);
        assert!(contrast_ratio(ACCENT, DARK_TEXT_BACKGROUND) >= 3.0);
    }

    #[test]
    fn link_types_round_trip_through_the_inspector_index() {
        for link_type in LinkType::ALL {
            let index = link_type_index(link_type);
            assert_eq!(link_type_from_index(index), link_type);
            assert!(LINK_TYPE_LABELS[index].contains(link_type.abbreviation()));
        }
    }

    #[test]
    fn link_endpoints_require_two_distinct_activities() {
        let first = ActivityId::new(1);
        let second = ActivityId::new(2);
        assert_eq!(
            link_endpoints(Some(first), Some(second)),
            Ok((first, second))
        );
        assert!(link_endpoints(Some(first), Some(first)).is_err());
        assert!(link_endpoints(None, Some(second)).is_err());
        assert!(link_endpoints(Some(first), None).is_err());
    }

    #[test]
    fn the_gantt_summary_names_the_text_alternative() {
        let mut project = Project::new(
            "Accessible",
            Date::from_calendar_date(2026, Month::January, 5).expect("date"),
        );
        project
            .activities
            .push(Activity::new(ActivityId::new(1), "Survey", 2));
        let schedule = schedule(&project).expect("schedule succeeds");
        let summary = gantt_summary(&schedule);
        assert!(summary.contains("1 activity"));
        assert!(summary.contains("critical"));
        assert!(summary.contains("Table view"));
    }
}

#[cfg(test)]
mod report_export_tests {
    use super::*;

    #[test]
    fn project_names_become_safe_file_stems() {
        assert_eq!(file_stem("Bridge & Road"), "Bridge-Road");
        assert_eq!(file_stem("Phase 2 / Phase 3"), "Phase-2-Phase-3");
        assert_eq!(file_stem("under_score"), "under_score");
        assert_eq!(file_stem("2026 — Q1"), "2026-Q1");
        assert_eq!(file_stem("   "), "cadence-project");
        assert_eq!(file_stem("###"), "cadence-project");
    }

    #[test]
    fn the_report_lands_in_a_markdown_file() {
        let project = Project::new(
            "Export me",
            Date::from_calendar_date(2026, Month::January, 5).expect("date"),
        );
        let report = cadence_core::report::project_report(&project, today());
        assert!(report.starts_with("# Export me"));
        assert_eq!(file_stem(&project.name), "Export-me");
    }
}

#[cfg(test)]
mod status_message_tests {
    use super::*;

    fn shown_plus(duration: Duration) -> (Instant, Instant) {
        let shown = Instant::now();
        (shown, shown + duration)
    }

    #[test]
    fn info_message_not_expired_is_not_cleared() {
        let (shown, later) = shown_plus(Duration::from_secs(1));
        let severity = StatusSeverity::Info;
        assert!(!should_auto_clear_status(severity, shown, later));
    }

    #[test]
    fn info_message_expired_is_cleared() {
        let (shown, later) = shown_plus(STATUS_MESSAGE_TTL + Duration::from_secs(1));
        let severity = StatusSeverity::Info;
        assert!(should_auto_clear_status(severity, shown, later));
    }

    #[test]
    fn success_message_expired_is_cleared() {
        let (shown, later) = shown_plus(STATUS_MESSAGE_TTL + Duration::from_secs(1));
        let severity = StatusSeverity::Success;
        assert!(should_auto_clear_status(severity, shown, later));
    }

    #[test]
    fn error_message_never_auto_clears() {
        let (shown, much_later) = shown_plus(Duration::from_secs(3600));
        let severity = StatusSeverity::Error;
        assert!(!should_auto_clear_status(severity, shown, much_later));
    }

    #[test]
    fn warning_message_never_auto_clears() {
        let (shown, much_later) = shown_plus(Duration::from_secs(3600));
        let severity = StatusSeverity::Warning;
        assert!(!should_auto_clear_status(severity, shown, much_later));
    }
}
