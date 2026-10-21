//! egui/eframe user interface for Cadence.
//!
//! Phase 2 starts here: a small, dependency-light shell that renders the
//! scheduled project in a window. The scheduling engine remains the single
//! source of truth; this crate only displays its results and routes edits
//! through the command layer.

use cadence_commands::{CommandStack, RenameActivity, SetActivityConstraint, SetActivityDuration};
use cadence_core::engine::schedule;
use cadence_core::engine::Schedule;
use cadence_core::model::{Activity, ActivityId, Constraint, Project};
use egui::{Color32, ComboBox, Context, Grid, ScrollArea, Ui, Visuals};
use time::Date;

/// Accent colour used for selection and highlights in both themes.
pub const ACCENT: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xED);

const CONSTRAINT_LABELS: [&str; 6] = [
    "As soon as possible",
    "As late as possible",
    "Start no earlier than",
    "Start no later than",
    "Finish no earlier than",
    "Finish no later than",
];

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

/// The Cadence application state.
pub struct CadenceApp {
    project: Project,
    schedule: Result<Schedule, String>,
    dark: bool,
    stack: CommandStack,
    selected: Option<ActivityId>,
    sort_key: SortKey,
    sort_ascending: bool,
    name_buffer: String,
    duration_buffer: String,
    constraint_kind: usize,
    constraint_date_buffer: String,
}

impl CadenceApp {
    /// Build the application from a loaded project.
    pub fn new(project: Project) -> Self {
        let schedule = schedule(&project).map_err(|error| error.to_string());
        Self {
            project,
            schedule,
            dark: true,
            stack: CommandStack::new(),
            selected: None,
            sort_key: SortKey::Name,
            sort_ascending: true,
            name_buffer: String::new(),
            duration_buffer: String::new(),
            constraint_kind: 0,
            constraint_date_buffer: String::new(),
        }
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
            2 => self
                .constraint_date_buffer
                .trim()
                .parse::<Date>()
                .ok()
                .map(Constraint::StartNoEarlierThan)
                .unwrap_or(activity.constraint),
            3 => self
                .constraint_date_buffer
                .trim()
                .parse::<Date>()
                .ok()
                .map(Constraint::StartNoLaterThan)
                .unwrap_or(activity.constraint),
            4 => self
                .constraint_date_buffer
                .trim()
                .parse::<Date>()
                .ok()
                .map(Constraint::FinishNoEarlierThan)
                .unwrap_or(activity.constraint),
            _ => self
                .constraint_date_buffer
                .trim()
                .parse::<Date>()
                .ok()
                .map(Constraint::FinishNoLaterThan)
                .unwrap_or(activity.constraint),
        };

        if constraint != activity.constraint {
            let _ = self.stack.execute(
                &mut self.project,
                Box::new(SetActivityConstraint { id, constraint }),
            );
        }

        self.recompute();
        self.select(id);
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

        if ctx.input(|input| input.key_pressed(egui::Key::D) && input.modifiers.ctrl) {
            self.dark = !self.dark;
            apply_theme(&ctx, self.dark);
        }

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Cadence");
                ui.separator();
                ui.label(&self.project.name);
                if ui.button("Undo").clicked() {
                    let _ = self.stack.undo(&mut self.project);
                    self.recompute();
                }
                if ui.button("Redo").clicked() {
                    let _ = self.stack.redo(&mut self.project);
                    self.recompute();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Toggle theme").clicked() {
                        self.dark = !self.dark;
                        apply_theme(&ctx, self.dark);
                    }
                });
            });
        });

        egui::Panel::right("inspector")
            .default_width(260.0)
            .show(ui, |ui| {
                ui.heading("Inspector");
                if let Some(id) = self.selected {
                    ui.label(format!("Activity {}", id));
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.name_buffer);
                    ui.label("Duration (working days)");
                    ui.text_edit_singleline(&mut self.duration_buffer);
                    ui.label("Constraint");
                    ComboBox::from_id_salt("constraint_kind")
                        .selected_text(CONSTRAINT_LABELS[self.constraint_kind])
                        .show_ui(ui, |ui| {
                            for (index, label) in CONSTRAINT_LABELS.iter().enumerate() {
                                ui.selectable_value(&mut self.constraint_kind, index, *label);
                            }
                        });
                    if self.constraint_kind >= 2 {
                        ui.label("Constraint date (YYYY-MM-DD)");
                        ui.text_edit_singleline(&mut self.constraint_date_buffer);
                    }
                    if ui.button("Apply").clicked() {
                        self.apply_edits();
                    }
                } else {
                    ui.label("Select an activity in the table.");
                }
            });

        egui::CentralPanel::default().show(ui, |ui| match &self.schedule {
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
                                    if ui.selectable_label(selected, &activity.name).clicked() {
                                        self.select(activity.id);
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
                });
            }
        });
    }
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
