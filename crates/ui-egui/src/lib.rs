//! egui/eframe user interface for Cadence.
//!
//! Phase 2 starts here: a small, dependency-light shell that renders the
//! scheduled project in a window. The scheduling engine remains the single
//! source of truth; this crate only displays its results.

use cadence_core::engine::schedule;
use cadence_core::engine::Schedule;
use cadence_core::model::Project;
use egui::{Color32, Context, Grid, ScrollArea, Ui, Visuals};

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

/// The Cadence application state.
pub struct CadenceApp {
    project: Project,
    schedule: Result<Schedule, String>,
    dark: bool,
}

impl CadenceApp {
    /// Build the application from a loaded project.
    pub fn new(project: Project) -> Self {
        let schedule = schedule(&project).map_err(|error| error.to_string());
        Self {
            project,
            schedule,
            dark: true,
        }
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Toggle theme").clicked() {
                        self.dark = !self.dark;
                        apply_theme(&ctx, self.dark);
                    }
                });
            });
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
                    Grid::new("schedule_grid")
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.strong("Activity");
                            ui.strong("Dur");
                            ui.strong("Start");
                            ui.strong("Finish");
                            ui.strong("Float");
                            ui.strong("Critical");
                            ui.end_row();

                            for activity in &self.project.activities {
                                if let Some(result) = schedule.activity(activity.id) {
                                    ui.label(&activity.name);
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
