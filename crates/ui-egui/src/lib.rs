//! egui/eframe user interface for Cadence.
//!
//! Phase 2 starts here: a small, dependency-light shell that renders the
//! scheduled project in a window. The scheduling engine remains the single
//! source of truth; this crate only displays its results and routes edits
//! through the command layer.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use cadence_commands::{
    AddResourceToActivity, CaptureBaseline, CommandStack, CreateCalendar, CreateRelationship,
    CreateResource, DeleteCalendar, DeleteResource, RemoveBaseline, RemoveResourceFromActivity,
    RenameActivity, RenameCalendar, RenameResource, SetActivityActualCost, SetActivityActualDates,
    SetActivityBudget, SetActivityCalendar, SetActivityConstraint, SetActivityDuration,
    SetActivityProgress, SetCalendarDefinition, SetRelationshipLag, SetResourceCapacity,
    SetResourceKind,
};
use cadence_core::calendar_set::CalendarId;
use cadence_core::engine::schedule;
use cadence_core::engine::Schedule;
use cadence_core::engine::{evm_metrics, level_resources, resource_conflicts, resource_histogram};
use cadence_core::model::{
    Activity, ActivityId, Baseline, BaselineId, Calendar, Constraint, LinkType, Project,
    ResourceId, ResourceKind,
};
use egui::{
    Align2, Color32, ComboBox, Context, Grid, Pos2, Rect, ScrollArea, Sense, Ui, Vec2, Visuals,
};
use time::{Date, Month};

pub mod crash;
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
}

impl DesignTokens {
    /// Tokens for the requested theme.
    pub fn for_dark(dark: bool) -> Self {
        if dark {
            Self {
                accent: ACCENT,
                critical: Color32::from_rgb(0xFF, 0x6B, 0x6B),
                normal: Color32::from_rgb(0x9B, 0xC1, 0xFF),
            }
        } else {
            Self {
                accent: ACCENT,
                critical: Color32::from_rgb(0xC0, 0x39, 0x2B),
                normal: Color32::from_rgb(0x1F, 0x3B, 0x73),
            }
        }
    }
}

/// Apply a theme to an egui context.
pub fn apply_theme(ctx: &Context, dark: bool) {
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.selection.bg_fill = ACCENT;
    visuals.hyperlink_color = ACCENT;
    ctx.set_visuals(visuals);
}

/// Sort key for the activity table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Name,
    Duration,
    Start,
    Finish,
    Float,
    Critical,
}

/// Which top-level view is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Table,
    Gantt,
    Resources,
    Baselines,
    Calendars,
}

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

/// The Cadence application state.
pub struct CadenceApp {
    project: Project,
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
    selected_relationship: Option<cadence_core::model::RelationshipId>,
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
    link_predecessor: Option<ActivityId>,
    link_successor: Option<ActivityId>,
    link_type_index: usize,
    link_lag_buffer: String,
    show_shortcuts_dialog: bool,
    scroll_to_selection: bool,
}

impl CadenceApp {
    /// Build the application from a loaded project.
    pub fn new(project: Project) -> Self {
        let schedule = schedule(&project).map_err(|error| error.to_string());
        let crash_log = crash::read_log().unwrap_or_default();
        let show_crash_dialog = !crash_log.is_empty();
        let update_status =
            update_check::cached_status().unwrap_or(update_check::UpdateStatus::NotChecked);
        Self {
            project,
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
            selected_relationship: None,
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
            scroll_to_selection: false,
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

    fn recompute(&mut self) {
        self.schedule = schedule(&self.project).map_err(|error| error.to_string());
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
    }

    /// Add a dependency link from the inspector form.
    ///
    /// This is the pointer-free equivalent of dragging between two Gantt bars.
    fn add_link_from_form(&mut self) {
        let endpoints = link_endpoints(self.link_predecessor, self.link_successor);
        let (predecessor, successor) = match endpoints {
            Ok(pair) => pair,
            Err(message) => {
                self.status_message = Some(message);
                return;
            }
        };
        let Ok(lag) = self.link_lag_buffer.trim().parse::<i64>() else {
            self.status_message = Some("Lag must be a whole number of working days".to_string());
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
                        self.status_message = Some(format!("Cannot add link: {error}"));
                    }
                    Ok(_) => {
                        self.selected_relationship = Some(id);
                        self.last_rel = None;
                        self.status_message = Some(format!(
                            "Added {} link {} → {}",
                            link_type, predecessor, successor
                        ));
                    }
                }
            }
            Err(error) => {
                self.status_message = Some(format!("Cannot add link: {error}"));
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

        if self.name_buffer != activity.name && !self.name_buffer.trim().is_empty() {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(RenameActivity {
                    id,
                    name: self.name_buffer.clone(),
                }),
            );
        }

        if let Ok(duration) = self.duration_buffer.trim().parse::<u32>() {
            if duration != activity.original_duration {
                let _ = self.stack.execute(
                    &mut self.project,
                    Box::new(SetActivityDuration { id, duration }),
                );
            }
        }

        let constraint = match self.constraint_kind {
            0 => Constraint::AsSoonAsPossible,
            1 => Constraint::AsLateAsPossible,
            2 => parse_date(self.constraint_date_buffer.trim())
                .map(Constraint::StartNoEarlierThan)
                .unwrap_or(activity.constraint),
            3 => parse_date(self.constraint_date_buffer.trim())
                .map(Constraint::StartNoLaterThan)
                .unwrap_or(activity.constraint),
            4 => parse_date(self.constraint_date_buffer.trim())
                .map(Constraint::FinishNoEarlierThan)
                .unwrap_or(activity.constraint),
            _ => parse_date(self.constraint_date_buffer.trim())
                .map(Constraint::FinishNoLaterThan)
                .unwrap_or(activity.constraint),
        };

        if constraint != activity.constraint {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityConstraint { id, constraint }),
            );
        }

        if let Some(percent) = parse_percent(self.progress_buffer.trim()) {
            if percent != activity.percent_complete {
                let _ = self.stack.execute(
                    &mut self.project,
                    Box::new(SetActivityProgress {
                        id,
                        percent_complete: percent,
                    }),
                );
            }
        }

        let budget = parse_money(self.budget_buffer.trim());
        if budget != activity.budget_at_completion {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityBudget {
                    id,
                    budget_at_completion: budget,
                }),
            );
        }

        let cost = parse_money(self.actual_cost_buffer.trim());
        if cost != activity.actual_cost {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityActualCost {
                    id,
                    actual_cost: cost,
                }),
            );
        }

        let actual_start = parse_date(self.actual_start_buffer.trim());
        let actual_finish = parse_date(self.actual_finish_buffer.trim());
        if actual_start != activity.actual_start || actual_finish != activity.actual_finish {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityActualDates {
                    id,
                    actual_start,
                    actual_finish,
                }),
            );
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
                self.status_message = Some(error);
            }
        }

        if let Ok(capacity) = self.resource_capacity_buffer.trim().parse::<u32>() {
            if capacity != resource.capacity {
                if let Err(error) = self.stack.execute(
                    &mut self.project,
                    Box::new(SetResourceCapacity { id, capacity }),
                ) {
                    self.status_message = Some(error);
                }
            }
        }

        let kind = resource_kind_from_index(self.resource_kind_index);
        if kind != resource.kind {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(SetResourceKind { id, kind }))
            {
                self.status_message = Some(error);
            }
        }

        self.recompute();
        self.select_resource(id);
    }

    /// Add a new resource to the dictionary.
    fn create_resource(&mut self) {
        let name = self.resource_name_buffer.trim().to_string();
        if name.is_empty() {
            self.status_message = Some("Resource name must not be empty".to_string());
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
                self.recompute();
                self.select_resource(id);
            }
            Err(error) => self.status_message = Some(error),
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
            self.status_message = Some("Calendar name must not be empty".to_string());
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
            Err(error) => self.status_message = Some(error),
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

        let name = self.calendar_name_buffer.trim().to_string();
        if !name.is_empty() && name != named.name {
            if let Err(error) = self
                .stack
                .execute(&mut self.project, Box::new(RenameCalendar { id, name }))
            {
                self.status_message = Some(error);
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
                self.status_message = Some(error);
            }
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
            self.status_message = Some(format!(
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
            Err(error) => self.status_message = Some(error),
        }
    }

    /// Build a calendar from the editor buffers, reporting why it is invalid.
    fn build_calendar(&mut self) -> Option<Calendar> {
        if !self.calendar_weekdays.iter().any(|working| *working) {
            self.status_message = Some("A calendar needs at least one working day".to_string());
            return None;
        }
        let mut calendar = Calendar::new(self.calendar_weekdays).ok()?;
        for entry in self.calendar_holidays_buffer.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some(date) = parse_date(trimmed) else {
                self.status_message = Some(format!("\"{trimmed}\" is not a valid date"));
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
            Err(error) => self.status_message = Some(error),
        }
    }

    /// Assign the given resource to the selected activity, or remove it.
    fn toggle_assignment(&mut self, resource_id: ResourceId) {
        let Some(activity_id) = self.selected else {
            self.status_message = Some("Select an activity first".to_string());
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
            Err(error) => self.status_message = Some(error),
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
            self.status_message = Some("The project cannot be scheduled".to_string());
            return;
        };
        let Ok(result) = level_resources(&self.project, &schedule) else {
            self.status_message = Some("The project cannot be leveled".to_string());
            return;
        };

        let mut applied = 0usize;
        for activity in &result.activities {
            if activity.delay <= 0 {
                continue;
            }
            let constraint = Constraint::StartNoEarlierThan(activity.start);
            if self
                .stack
                .execute(
                    &mut self.project,
                    Box::new(SetActivityConstraint {
                        id: activity.id,
                        constraint,
                    }),
                )
                .is_ok()
            {
                applied += 1;
            }
        }
        self.recompute();
        self.leveling_report = Some(format!("Applied leveling to {applied} activities"));
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
            Err(error) => self.status_message = Some(error),
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
                self.status_message = Some(format!("Report printed to {}", path.display()));
            }
            Err(error) => {
                self.status_message = Some(format!("Cannot write {}: {error}", path.display()));
            }
        }
    }

    /// Write the Markdown report next to the working directory.
    ///
    /// The report itself needs no dialog; "Print report…" is the one that
    /// opens one, for choosing where a PDF goes.
    fn save_report(&mut self) {
        let path = format!("{}.report.md", file_stem(&self.project.name));
        let report = cadence_core::report::project_report(&self.project, today());
        match std::fs::write(&path, report) {
            Ok(()) => self.status_message = Some(format!("Report written to {path}")),
            Err(error) => self.status_message = Some(format!("Cannot write {path}: {error}")),
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
            Err(error) => self.status_message = Some(error),
        }
    }

    fn sorted_activities(&self) -> Vec<Activity> {
        let mut activities = self.project.activities.clone();
        let schedule = self.schedule.as_ref().ok();
        activities.sort_by(|left, right| {
            let ordering = match self.sort_key {
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
            if self.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        activities
    }
}

impl eframe::App for CadenceApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

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

        if ctx.input(|input| input.key_pressed(egui::Key::D) && input.modifiers.ctrl) {
            self.dark = !self.dark;
            apply_theme(&ctx, self.dark);
        }

        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y: undo and redo from any view.
        let undo_requested = ctx.input(|input| {
            input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && !input.modifiers.shift
        });
        let redo_requested = ctx.input(|input| {
            (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl)
                || (input.key_pressed(egui::Key::Z)
                    && input.modifiers.ctrl
                    && input.modifiers.shift)
        });
        if undo_requested {
            let _ = self.stack.undo(&mut self.project);
            self.recompute();
        }
        if redo_requested {
            let _ = self.stack.redo(&mut self.project);
            self.recompute();
        }

        // Ctrl+1..Ctrl+5: switch views without reaching for the mouse.
        let (ctrl_held, pressed_view) = ctx.input(|input| {
            (
                input.modifiers.ctrl,
                [
                    egui::Key::Num1,
                    egui::Key::Num2,
                    egui::Key::Num3,
                    egui::Key::Num4,
                    egui::Key::Num5,
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
                    _ => ViewMode::Calendars,
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
                let activities = self.sorted_activities();
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
                        self.status_message = Some(format!("Gantt printed to {}", path.display()));
                    }
                    Err(error) => {
                        self.status_message =
                            Some(format!("Cannot write {}: {error}", path.display()));
                    }
                }
                self.gantt_print_path = None;
            } else {
                ctx.request_repaint();
            }
        }

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Cadence");
                ui.separator();
                ui.label(&self.project.name);
                if ui.button("Table").clicked() {
                    self.view_mode = ViewMode::Table;
                }
                if ui.button("Gantt").clicked() {
                    self.view_mode = ViewMode::Gantt;
                }
                if ui.button("Resources").clicked() {
                    self.view_mode = ViewMode::Resources;
                }
                if ui.button("Baselines").clicked() {
                    self.view_mode = ViewMode::Baselines;
                }
                if ui.button("Calendars").clicked() {
                    self.view_mode = ViewMode::Calendars;
                }
                if ui
                    .button("Save report")
                    .on_hover_text("Write a Markdown report next to the working directory")
                    .clicked()
                {
                    self.save_report();
                }
                if ui
                    .button("Print report…")
                    .on_hover_text("Save a print-ready PDF of the report via a file dialog")
                    .clicked()
                {
                    self.print_report_dialog();
                }
                if ui
                    .add_enabled(
                        self.view_mode == ViewMode::Gantt,
                        egui::Button::new("Print Gantt…"),
                    )
                    .on_hover_text("Save a print-ready PDF of the Gantt view via a file dialog")
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
                if ui
                    .button("Report a bug…")
                    .on_hover_text("Preview and report a problem to the Cadence issue tracker")
                    .clicked()
                {
                    if self.crash_log.is_empty() {
                        if let Some(log) = crash::read_log() {
                            self.crash_log = log;
                        }
                    }
                    self.crash_notes.clear();
                    self.show_crash_dialog = true;
                }
                let update_label = match &self.update_status {
                    update_check::UpdateStatus::UpdateAvailable { version, .. } => {
                        format!("Update available: {version}")
                    }
                    _ => "Updates".to_string(),
                };
                if ui
                    .button(update_label)
                    .on_hover_text("Check GitHub Releases for a newer Cadence")
                    .clicked()
                {
                    self.show_update_dialog = true;
                }
                if ui.button("Undo").clicked() {
                    let _ = self.stack.undo(&mut self.project);
                    self.recompute();
                }
                if ui.button("Redo").clicked() {
                    let _ = self.stack.redo(&mut self.project);
                    self.recompute();
                }
                if let Some(message) = &self.status_message {
                    ui.label(format!("Status: {message}"));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Toggle theme").clicked() {
                        self.dark = !self.dark;
                        apply_theme(&ctx, self.dark);
                    }
                    if ui
                        .button("Shortcuts")
                        .on_hover_text("Keyboard shortcuts (F1)")
                        .clicked()
                    {
                        self.show_shortcuts_dialog = true;
                    }
                });
            });
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
                                if let Some(id) = self.selected {
                                    ui.label(format!("Activity {}", id));
                                    ui.label("Name");
                                    ui.text_edit_singleline(&mut self.name_buffer);
                                    ui.label("Duration (working days)");
                                    ui.text_edit_singleline(&mut self.duration_buffer);
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
                                        ui.label("Constraint date (YYYY-MM-DD)");
                                        ui.text_edit_singleline(&mut self.constraint_date_buffer);
                                    }

                                    ui.separator();
                                    ui.heading("Calendar");
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

                                    ui.separator();
                                    ui.heading("Progress");
                                    ui.label("Percent complete");
                                    ui.text_edit_singleline(&mut self.progress_buffer);
                                    ui.label("Budget at completion");
                                    ui.text_edit_singleline(&mut self.budget_buffer);
                                    ui.label("Actual cost");
                                    ui.text_edit_singleline(&mut self.actual_cost_buffer);
                                    ui.label("Actual start (YYYY-MM-DD)");
                                    ui.text_edit_singleline(&mut self.actual_start_buffer);
                                    ui.label("Actual finish (YYYY-MM-DD)");
                                    ui.text_edit_singleline(&mut self.actual_finish_buffer);

                                    if ui.button("Apply").clicked() {
                                        self.apply_edits();
                                    }

                                    ui.separator();
                                    ui.heading("Relationships");
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
                                        if let Some(rel) = self
                                            .selected_relationship
                                            .and_then(|id| self.project.relationship(id))
                                        {
                                            ui.label(format!(
                                                "Type: {}",
                                                LINK_TYPE_LABELS[link_type_index(rel.link_type)]
                                            ));
                                        }
                                        ui.label("Lag (working days)");
                                        ui.text_edit_singleline(&mut self.lag_buffer);
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

                let schedule = self.schedule.clone();
                egui::CentralPanel::default().show(ui, |ui| match schedule {
                    Err(error) => {
                        ui.colored_label(egui::Color32::RED, format!("Schedule error: {error}"));
                    }
                    Ok(schedule) => {
                        ui.label(format!(
                            "Project start: {}   Project finish: {}",
                            schedule.project_start, schedule.project_finish
                        ));
                        ui.separator();
                        ScrollArea::vertical().show(ui, |ui| {
                            let mut header_clicked = None;
                            Grid::new("schedule_header")
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    for (label, key) in [
                                        ("Activity", SortKey::Name),
                                        ("Dur", SortKey::Duration),
                                        ("Start", SortKey::Start),
                                        ("Finish", SortKey::Finish),
                                        ("Float", SortKey::Float),
                                        ("Critical", SortKey::Critical),
                                    ] {
                                        if ui.button(label).clicked() {
                                            header_clicked = Some(key);
                                        }
                                    }
                                    ui.end_row();
                                });

                            if let Some(key) = header_clicked {
                                if self.sort_key == key {
                                    self.sort_ascending = !self.sort_ascending;
                                } else {
                                    self.sort_key = key;
                                    self.sort_ascending = true;
                                }
                            }

                            Grid::new("schedule_rows")
                                .striped(true)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    for activity in self.sorted_activities() {
                                        if let Some(result) = schedule.activity(activity.id) {
                                            let selected = self.selected == Some(activity.id);
                                            let response =
                                                ui.selectable_label(selected, &activity.name);
                                            if response.clicked() {
                                                self.select(activity.id);
                                            }
                                            if selected && self.scroll_to_selection {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                            ui.label(activity.original_duration.to_string());
                                            ui.label(result.scheduled_start.to_string());
                                            ui.label(result.scheduled_finish.to_string());
                                            ui.label(result.total_float.to_string());
                                            ui.label(if result.critical { "Yes" } else { "No" });
                                            ui.end_row();
                                        }
                                    }
                                });
                            self.scroll_to_selection = false;
                        });
                    }
                });
            }
            ViewMode::Gantt => {
                egui::CentralPanel::default().show(ui, |ui| {
                    let px = self.pixels_per_day;
                    render_gantt(
                        ui,
                        &mut self.project,
                        &mut self.schedule,
                        &mut self.stack,
                        &mut self.drag_start,
                        &mut self.status_message,
                        px,
                        &mut self.pixels_per_day,
                    );
                });
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
                            self.apply_leveling();
                        }
                        if let Some(report) = &self.leveling_report {
                            ui.label(report);
                        }
                    });

                let schedule = self.schedule.clone();
                let conflicts = schedule
                    .as_ref()
                    .ok()
                    .map(|schedule| resource_conflicts(&self.project, schedule))
                    .unwrap_or_default();
                let resources = self.project.resources.clone();
                let activities = self.sorted_activities();
                let selected_resource = self.selected_resource;

                egui::CentralPanel::default().show(ui, |ui| match schedule {
                    Err(error) => {
                        ui.colored_label(egui::Color32::RED, format!("Schedule error: {error}"));
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
                                            egui::Color32::from_rgb(0xC0, 0x39, 0x2B),
                                            over.to_string(),
                                        );
                                    }
                                    ui.end_row();
                                }
                            });

                        ui.separator();
                        ui.heading("Usage");
                        if let Some(id) = selected_resource {
                            let usage = resource_day_usage(&self.project, &schedule, id);
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
                                                    if units > capacity {
                                                        ui.colored_label(
                                                            egui::Color32::from_rgb(
                                                                0xC0, 0x39, 0x2B,
                                                            ),
                                                            format!("{units} / {capacity}"),
                                                        );
                                                    } else {
                                                        ui.label(format!("{units} / {capacity}"));
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
                            ui.label("No baselines captured yet.");
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
                        ui.label("Status date (YYYY-MM-DD)");
                        ui.text_edit_singleline(&mut self.status_date_buffer);
                    });

                let baselines = self.project.baselines.clone();
                let selected_baseline = self
                    .selected_baseline
                    .and_then(|id| baselines.iter().find(|baseline| baseline.id == id).cloned());
                let status_date = parse_date(self.status_date_buffer.trim());
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
                                egui::Color32::RED,
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
                            if ui.button("Open release page").clicked() {
                                ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
                            }
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
                                    "Ctrl+1 … Ctrl+5",
                                    "Switch view: Table, Gantt, Resources, Baselines, \
                                     Calendars",
                                ),
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
    project: &Project,
    schedule: &Schedule,
    resource_id: ResourceId,
) -> BTreeMap<Date, u32> {
    let histogram = resource_histogram(project, schedule);
    let mut usage = BTreeMap::new();
    for (date, assignments) in &histogram.entries {
        let units = assignments
            .iter()
            .filter(|assignment| assignment.resource_id == resource_id)
            .count() as u32;
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
         Red bars (marked !) are critical; the darker fill is recorded progress. \
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
    pixels_per_day: f32,
    pixels_per_day_out: &mut f32,
) {
    let current_schedule = match schedule.as_ref() {
        Ok(schedule) => schedule.clone(),
        Err(error) => {
            ui.colored_label(egui::Color32::RED, format!("Schedule error: {error}"));
            return;
        }
    };
    let origin = project.start_date;
    let finish = current_schedule.project_finish;
    let total_days = (finish.to_julian_day() - origin.to_julian_day()).max(1) as f32 + 2.0;
    let row_height = 20.0;
    let header_height = 24.0;
    let total_height = project.activities.len() as f32 * row_height + header_height;
    let timeline_width = total_days * pixels_per_day + 40.0;
    let summary = gantt_summary(&current_schedule);
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
    });

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

        // Drag-to-create dependency links.
        if response.drag_started() {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                let row = ((pos.y - rect.min.y - header_height) / row_height).floor() as usize;
                if row < project.activities.len() {
                    *drag_start = Some(row);
                }
            }
        }

        if let Some(start_row) = *drag_start {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                let start_y =
                    rect.min.y + header_height + start_row as f32 * row_height + row_height / 2.0;
                let start_activity = &project.activities[start_row];
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
                    if end_row < project.activities.len() && end_row != start_row {
                        let predecessor = project.activities[start_row].id;
                        let successor = project.activities[end_row].id;
                        let id = project.next_relationship_id();
                        let command =
                            CreateRelationship::new(id, predecessor, successor, LinkType::Fs, 0);
                        if stack.execute(project, Box::new(command)).is_ok() {
                            *schedule =
                                cadence_core::engine::schedule(project).map_err(|e| e.to_string());
                            if schedule.is_err() {
                                let _ = stack.undo(project);
                                *schedule = cadence_core::engine::schedule(project)
                                    .map_err(|e| e.to_string());
                                *status_message =
                                    Some("Cannot add link: cycle detected".to_string());
                            } else {
                                *status_message = None;
                            }
                        } else {
                            *status_message = Some("Cannot add link".to_string());
                        }
                    }
                }
            }
        }

        // Zoom with Ctrl + wheel.
        if response.hovered() {
            let scroll = ui.ctx().input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 && ui.ctx().input(|input| input.modifiers.ctrl) {
                *pixels_per_day_out =
                    (*pixels_per_day_out * (1.0 + scroll * 0.001)).clamp(2.0, 60.0);
            }
        }

        let tick_step = if pixels_per_day < 4.0 {
            10
        } else if pixels_per_day < 10.0 {
            5
        } else {
            1
        };

        // Timeline header.
        for day in 0..=(total_days as i32) {
            let x = rect.min.x + day as f32 * pixels_per_day;
            if x < clip.min.x || x > clip.max.x {
                continue;
            }
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.min.y + 6.0)],
                egui::Stroke::new(1.0, Color32::GRAY),
            );
            if day % tick_step == 0 {
                let date = Date::from_julian_day(origin.to_julian_day() + day).unwrap_or(origin);
                painter.text(
                    Pos2::new(x, rect.min.y + 8.0),
                    Align2::CENTER_TOP,
                    date.to_string(),
                    egui::FontId::proportional(10.0),
                    Color32::GRAY,
                );
            }
        }

        let first_row = ((clip.min.y - rect.min.y - header_height) / row_height)
            .floor()
            .max(0.0) as usize;
        let last_row = ((clip.max.y - rect.min.y - header_height) / row_height)
            .ceil()
            .max(0.0) as usize
            + 1;
        let last_row = last_row.min(project.activities.len());

        for row in first_row..last_row {
            let activity = &project.activities[row];
            let y = rect.min.y + header_height + row as f32 * row_height;
            if let Some(result) = current_schedule.activity(activity.id) {
                let start_days =
                    (result.scheduled_start.to_julian_day() - origin.to_julian_day()) as f32;
                let finish_days =
                    (result.scheduled_finish.to_julian_day() - origin.to_julian_day()) as f32;
                let x0 = rect.min.x + start_days * pixels_per_day;
                let width = ((finish_days - start_days + 1.0) * pixels_per_day).max(6.0);
                let x1 = x0 + width;
                let bar =
                    Rect::from_min_max(Pos2::new(x0, y + 2.0), Pos2::new(x1, y + row_height - 2.0));
                let color = if result.critical {
                    Color32::from_rgb(0xFF, 0x6B, 0x6B)
                } else {
                    Color32::from_rgb(0x9B, 0xC1, 0xFF)
                };
                painter.rect_filled(bar, 2.0, color);

                // Progress overlay: the completed share of the bar is drawn in
                // a stronger tone so that recorded progress is visible.
                if let Some(percent) = activity.percent_complete {
                    if percent > 0 {
                        let fraction = percent.clamp(0, 10_000) as f32 / 10_000.0;
                        let progress = Rect::from_min_max(
                            bar.min,
                            Pos2::new(bar.min.x + bar.width() * fraction, bar.max.y),
                        );
                        let progress_color = if result.critical {
                            Color32::from_rgb(0xC0, 0x39, 0x2B)
                        } else {
                            Color32::from_rgb(0x2F, 0x6F, 0xED)
                        };
                        painter.rect_filled(progress, 2.0, progress_color);
                    }
                }

                let bar_label = if result.critical {
                    format!("! {}", activity.name)
                } else {
                    activity.name.clone()
                };
                painter.text(
                    bar.center(),
                    Align2::CENTER_CENTER,
                    &bar_label,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Dependency arrows.
        for relationship in &project.relationships {
            let pred_activity = project
                .activities
                .iter()
                .position(|a| a.id == relationship.predecessor);
            let succ_activity = project
                .activities
                .iter()
                .position(|a| a.id == relationship.successor);
            let (Some(pred_row), Some(succ_row)) = (pred_activity, succ_activity) else {
                continue;
            };
            if pred_row < first_row && pred_row >= last_row {
                // skip fully off-screen predecessors
            }
            if succ_row < first_row || succ_row >= last_row {
                // skip off-screen successors
                continue;
            }
            let Some(pred_result) = current_schedule.activity(relationship.predecessor) else {
                continue;
            };
            let Some(succ_result) = current_schedule.activity(relationship.successor) else {
                continue;
            };
            let pred_days =
                (pred_result.scheduled_finish.to_julian_day() - origin.to_julian_day()) as f32;
            let succ_days =
                (succ_result.scheduled_start.to_julian_day() - origin.to_julian_day()) as f32;
            let x0 = rect.min.x + (pred_days + 1.0) * pixels_per_day;
            let x1 = rect.min.x + succ_days * pixels_per_day;
            let y0 = rect.min.y + header_height + pred_row as f32 * row_height + row_height / 2.0;
            let y1 = rect.min.y + header_height + succ_row as f32 * row_height + row_height / 2.0;
            let mid_x = (x0 + x1) / 2.0;
            let stroke = egui::Stroke::new(1.0, Color32::GRAY);
            painter.line_segment([Pos2::new(x0, y0), Pos2::new(mid_x, y0)], stroke);
            painter.line_segment([Pos2::new(mid_x, y0), Pos2::new(mid_x, y1)], stroke);
            painter.line_segment([Pos2::new(mid_x, y1), Pos2::new(x1, y1)], stroke);
        }
    });
}

/// Run the Cadence GUI with the given project.
pub fn run(project: Project) -> Result<(), String> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Cadence",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx, true);
            Ok(Box::new(CadenceApp::new(project)))
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
