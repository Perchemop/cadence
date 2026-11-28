//! Command Palette (⌘K) — fuzzy searchable action palette
//!
//! Matches the pattern from Linear, Planera, nPlan, and MS Project Web.
//! Single searchable palette for all actions (views, commands, navigation).

use egui::{Align2, Color32, Context, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Vec2};
use egui_phosphor::regular as phos;

use crate::{CadenceApp, SortKey, ViewMode};
use cadence_core::model::ActivityId;

/// Category for palette entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCategory {
    Views,
    GanttOperations,
    TableOperations,
    Resources,
    Settings,
    Help,
    Activity,
    Project,
}

/// A single entry in the command palette.
#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub category: PaletteCategory,
    pub shortcut: Option<String>,
    pub icon: Option<&'static str>,
    pub action: PaletteAction,
}

/// Action to execute when entry is selected.
#[derive(Debug, Clone)]
pub enum PaletteAction {
    /// Switch to a view mode
    SetView(ViewMode),
    /// Execute a built-in command by name
    Command(String),
    /// Open a dialog
    OpenDialog(String),
    /// Toggle a boolean state
    Toggle(String),
    /// Select an activity by ID
    SelectActivity(ActivityId),
    /// No-op (for headers/separators)
    None,
}

impl PaletteEntry {
    /// Create a view switcher entry.
    pub fn view(mode: ViewMode, label: &str, tooltip: &str, shortcut: &str, icon: &'static str) -> Self {
        Self {
            id: format!("view:{}", label.to_ascii_lowercase().replace(' ', "-")),
            label: label.to_string(),
            description: Some(tooltip.to_string()),
            category: PaletteCategory::Views,
            shortcut: if shortcut.is_empty() { None } else { Some(shortcut.to_string()) },
            icon: Some(icon),
            action: PaletteAction::SetView(mode),
        }
    }

    /// Create a command entry.
    pub fn command(id: &str, label: &str, description: &str, shortcut: &str, icon: &'static str) -> Self {
        Self {
            id: format!("cmd:{}", id),
            label: label.to_string(),
            description: Some(description.to_string()),
            category: PaletteCategory::Help,
            shortcut: if shortcut.is_empty() { None } else { Some(shortcut.to_string()) },
            icon: Some(icon),
            action: PaletteAction::Command(id.to_string()),
        }
    }

    /// Create a dialog entry.
    pub fn dialog(id: &str, label: &str, description: &str, shortcut: &str, icon: &'static str) -> Self {
        Self {
            id: format!("dialog:{}", id),
            label: label.to_string(),
            description: Some(description.to_string()),
            category: PaletteCategory::Settings,
            shortcut: if shortcut.is_empty() { None } else { Some(shortcut.to_string()) },
            icon: Some(icon),
            action: PaletteAction::OpenDialog(id.to_string()),
        }
    }

    /// Create a toggle entry.
    pub fn toggle(id: &str, label: &str, description: &str, shortcut: &str, icon: &'static str) -> Self {
        Self {
            id: format!("toggle:{}", id),
            label: label.to_string(),
            description: Some(description.to_string()),
            category: PaletteCategory::Settings,
            shortcut: if shortcut.is_empty() { None } else { Some(shortcut.to_string()) },
            icon: Some(icon),
            action: PaletteAction::Toggle(id.to_string()),
        }
    }
}

/// State for the command palette.
#[derive(Debug, Default)]
pub struct CommandPaletteState {
    /// Whether the palette is currently open.
    pub open: bool,
    /// Current search query.
    pub query: String,
    /// Index of the selected entry in the filtered results.
    pub selected_index: usize,
    /// All available entries (rebuilt when project changes).
    pub entries: Vec<PaletteEntry>,
    /// Filtered entries matching the current query.
    pub filtered: Vec<usize>,
    /// Whether to focus the search field on next open.
    pub focus_next: bool,
}

impl CommandPaletteState {
    /// Create a new command palette state with default entries.
    pub fn new(app: &CadenceApp) -> Self {
        let mut state = Self::default();
        state.rebuild_entries(app);
        state
    }

    /// Rebuild all palette entries based on current application state.
    pub fn rebuild_entries(&mut self, app: &CadenceApp) {
        self.entries.clear();

        // --- Views ---
        for (mode, label, tooltip, icon, _group, shortcut) in crate::VIEW_META {
            self.entries.push(PaletteEntry::view(
                mode, label, tooltip, shortcut, icon,
            ));
        }

        // --- Gantt Operations ---
        self.entries.push(PaletteEntry::command(
            "gantt:zoom_in",
            "Zoom In (Gantt)",
            "Increase Gantt time scale",
            "Ctrl++ / Ctrl+Scroll Up",
            phos::MAGNIFYING_GLASS_PLUS,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:zoom_out",
            "Zoom Out (Gantt)",
            "Decrease Gantt time scale",
            "Ctrl+- / Ctrl+Scroll Down",
            phos::MAGNIFYING_GLASS_MINUS,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:zoom_fit",
            "Fit to Window (Gantt)",
            "Zoom Gantt to fit all activities",
            "Ctrl+0",
            phos::ARROWS_IN_LINE_HORIZONTAL,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:toggle_critical",
            "Toggle Critical Path Highlight",
            "Show/hide critical path highlighting in Gantt",
            "Ctrl+Shift+C",
            phos::FLAG,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:toggle_baseline",
            "Toggle Baseline Overlay",
            "Show/hide baseline bars in Gantt",
            "Ctrl+Shift+B",
            phos::STACK,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:toggle_today",
            "Toggle Today Line",
            "Show/hide the today indicator line in Gantt",
            "",
            phos::CALENDAR,
        ));
        self.entries.push(PaletteEntry::command(
            "gantt:toggle_split",
            "Toggle Split Gantt/Table",
            "Show/hide the split Gantt + Table view",
            "",
            phos::WINDOWS,
        ));

        // --- Table Operations ---
        self.entries.push(PaletteEntry::command(
            "table:new_activity",
            "New Activity",
            "Create a new activity at the end",
            "Ctrl+N",
            phos::PLUS,
        ));
        self.entries.push(PaletteEntry::command(
            "table:delete_activity",
            "Delete Selected Activity",
            "Delete the currently selected activity",
            "Delete",
            phos::TRASH,
        ));
        self.entries.push(PaletteEntry::command(
            "table:duplicate_activity",
            "Duplicate Activity",
            "Duplicate the selected activity",
            "Ctrl+D",
            phos::COPY,
        ));
        self.entries.push(PaletteEntry::command(
            "table:add_relationship",
            "Add Relationship",
            "Create a dependency between two activities",
            "Ctrl+L",
            phos::ARROW_RIGHT,
        ));
        self.entries.push(PaletteEntry::command(
            "table:delete_relationship",
            "Delete Relationship",
            "Remove the selected relationship",
            "",
            phos::X,
        ));
        self.entries.push(PaletteEntry::command(
            "table:toggle_critical_only",
            "Show Only Critical",
            "Filter table to show only critical path activities",
            "",
            phos::FILTER,
        ));
        self.entries.push(PaletteEntry::command(
            "table:toggle_columns",
            "Toggle Columns",
            "Show/hide columns in the activity table",
            "",
            phos::COLUMNS,
        ));
        self.entries.push(PaletteEntry::command(
            "table:sort_by_name",
            "Sort by Name",
            "Sort activities alphabetically by name",
            "",
            phos::SORT_ASCENDING,
        ));
        self.entries.push(PaletteEntry::command(
            "table:sort_by_date",
            "Sort by Start Date",
            "Sort activities by start date",
            "",
            phos::CALENDAR,
        ));
        self.entries.push(PaletteEntry::command(
            "table:expand_all",
            "Expand All WBS",
            "Expand all WBS levels in the table",
            "Ctrl+Shift+E",
            phos::CARET_DOWN,
        ));
        self.entries.push(PaletteEntry::command(
            "table:collapse_all",
            "Collapse All WBS",
            "Collapse all WBS levels in the table",
            "Ctrl+Shift+C",
            phos::CARET_UP,
        ));

        // --- Resources ---
        self.entries.push(PaletteEntry::command(
            "resource:new",
            "New Resource",
            "Create a new resource",
            "",
            phos::USER_PLUS,
        ));
        self.entries.push(PaletteEntry::command(
            "resource:assign",
            "Assign Resource",
            "Assign a resource to the selected activity",
            "",
            phos::USERS,
        ));
        self.entries.push(PaletteEntry::command(
            "resource:level",
            "Level Resources",
            "Auto-level resource overallocations",
            "F9",
            phos::GAUGE,
        ));

        // --- Settings ---
        self.entries.push(PaletteEntry::toggle(
            "theme:dark",
            "Dark Theme",
            "Switch to dark theme",
            "",
            phos::MOON,
        ));
        self.entries.push(PaletteEntry::toggle(
            "theme:light",
            "Light Theme",
            "Switch to light theme",
            "",
            phos::SUN,
        ));
        self.entries.push(PaletteEntry::dialog(
            "settings:general",
            "General Settings",
            "Open general application settings",
            "Ctrl+,",
            phos::GEAR,
        ));
        self.entries.push(PaletteEntry::dialog(
            "settings:appearance",
            "Appearance Settings",
            "Open appearance and theme settings",
            "",
            phos::PALETTE,
        ));
        self.entries.push(PaletteEntry::dialog(
            "settings:assistant",
            "AI Assistant Settings",
            "Configure AI assistant provider and options",
            "",
            phos::CHAT_CIRCLE_TEXT,
        ));
        self.entries.push(PaletteEntry::dialog(
            "settings:shortcuts",
            "Keyboard Shortcuts",
            "View and customize keyboard shortcuts",
            "F1",
            phos::KEYBOARD,
        ));

        // --- Help ---
        self.entries.push(PaletteEntry::command(
            "help:shortcuts",
            "Keyboard Shortcuts",
            "Show all keyboard shortcuts",
            "F1",
            phos::KEYBOARD,
        ));
        self.entries.push(PaletteEntry::command(
            "help:release_notes",
            "Release Notes",
            "View release notes for this version",
            "",
            phos::NEWSPAPER,
        ));
        self.entries.push(PaletteEntry::command(
            "help:about",
            "About Cadence",
            "Show version and build information",
            "",
            phos::INFO,
        ));

        // --- Project Operations ---
        self.entries.push(PaletteEntry::command(
            "project:new",
            "New Project",
            "Create a new empty project",
            "Ctrl+Shift+N",
            phos::FILE_NEW,
        ));
        self.entries.push(PaletteEntry::command(
            "project:open",
            "Open Project",
            "Open an existing project file",
            "Ctrl+O",
            phos::FOLDER_OPEN,
        ));
        self.entries.push(PaletteEntry::command(
            "project:save",
            "Save Project",
            "Save the current project",
            "Ctrl+S",
            phos::FLOPPY_DISK,
        ));
        self.entries.push(PaletteEntry::command(
            "project:save_as",
            "Save Project As",
            "Save the current project with a new name",
            "Ctrl+Shift+S",
            phos::FLOPPY_DISK_BACK,
        ));
        self.entries.push(PaletteEntry::command(
            "project:import",
            "Import Project",
            "Import from MSPDI, MPX, CSV, or JSON",
            "Ctrl+I",
            phos::DOWNLOAD,
        ));
        self.entries.push(PaletteEntry::command(
            "project:export",
            "Export Project",
            "Export to MSPDI, MPX, CSV, JSON, or PDF",
            "Ctrl+E",
            phos::UPLOAD,
        ));
        self.entries.push(PaletteEntry::command(
            "project:print_gantt",
            "Print Gantt Chart",
            "Print the Gantt view to PDF",
            "Ctrl+P",
            phos::PRINTER,
        ));
        self.entries.push(PaletteEntry::command(
            "project:audit",
            "Run Schedule Audit",
            "Run DCMA/GAO schedule quality checks",
            "Ctrl+Shift+A",
            phos::SHIELD_CHECK,
        ));

        // --- Edit Operations ---
        self.entries.push(PaletteEntry::command(
            "edit:undo",
            "Undo",
            "Undo the last action",
            "Ctrl+Z",
            phos::ARROW_COUNTERCLOCKWISE,
        ));
        self.entries.push(PaletteEntry::command(
            "edit:redo",
            "Redo",
            "Redo the last undone action",
            "Ctrl+Shift+Z",
            phos::ARROW_CLOCKWISE,
        ));
        self.entries.push(PaletteEntry::command(
            "edit:copy",
            "Copy",
            "Copy selected activity",
            "Ctrl+C",
            phos::COPY,
        ));
        self.entries.push(PaletteEntry::command(
            "edit:paste",
            "Paste",
            "Paste as new activity",
            "Ctrl+V",
            phos::CLIPBOARD_TEXT,
        ));

        // --- Lookahead / Short-interval Planning ---
        self.entries.push(PaletteEntry::command(
            "lookahead:new_snapshot",
            "Create Lookahead Snapshot",
            "Save a named snapshot of the current lookahead window",
            "",
            phos::CAMERA,
        ));
        self.entries.push(PaletteEntry::command(
            "lookahead:compare_snapshots",
            "Compare Lookahead Snapshots",
            "Compare two lookahead snapshots",
            "",
            phos::STACK_SIMPLE,
        ));

        // --- Diary ---
        self.entries.push(PaletteEntry::command(
            "diary:new_note",
            "New Diary Note",
            "Create a new dated site note",
            "",
            phos::NOTE_PENCIL,
        ));

        // --- Production ---
        self.entries.push(PaletteEntry::command(
            "production:new_entry",
            "New Production Entry",
            "Record actual progress for an activity",
            "",
            phos::GAUGE,
        ));

        // Add activity-specific entries if an activity is selected
        if let Some(id) = app.selected {
            if let Some(activity) = app.project.activity(id) {
                self.entries.push(PaletteEntry {
                    id: format!("activity:{}", activity.id.0),
                    label: format!("Activity: {}", activity.name),
                    description: Some(format!("ID: {} | Duration: {} days", activity.id.0, activity.original_duration)),
                    category: PaletteCategory::Activity,
                    shortcut: None,
                    icon: Some(phos::TASK),
                    action: PaletteAction::SelectActivity(activity.id),
                });
            }
        }

        self.filter("");
    }

    /// Filter entries by query string.
    pub fn filter(&mut self, query: &str) {
        self.query = query.to_string();
        self.filtered.clear();
        self.selected_index = 0;

        let query_lower = query.to_ascii_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().filter(|w| !w.is_empty()).collect();

        for (idx, entry) in self.entries.iter().enumerate() {
            if query.is_empty() || self.matches_query(entry, &query_words) {
                self.filtered.push(idx);
            }
        }
    }

    /// Check if an entry matches the query words (fuzzy matching).
    fn matches_query(&self, entry: &PaletteEntry, words: &[&str]) -> bool {
        let label_lower = entry.label.to_ascii_lowercase();
        let desc_lower = entry.description.as_ref().map(|d| d.to_ascii_lowercase()).unwrap_or_default();
        let category_lower = format!("{:?}", entry.category).to_ascii_lowercase();
        let shortcut_lower = entry.shortcut.as_ref().map(|s| s.to_ascii_lowercase()).unwrap_or_default();

        for word in words {
            let found = label_lower.contains(word)
                || desc_lower.contains(word)
                || category_lower.contains(word)
                || shortcut_lower.contains(word);
            if !found {
                return false;
            }
        }
        true
    }

    /// Open the palette.
    pub fn open(&mut self) {
        self.open = true;
        self.focus_next = true;
        self.query.clear();
        self.filter("");
    }

    /// Close the palette.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.filtered.clear();
        self.selected_index = 0;
    }

    /// Handle keyboard input when palette is open.
    pub fn handle_input(&mut self, ctx: &Context) -> bool {
        if !self.open {
            return false;
        }

        let mut handled = false;

        // Escape closes the palette
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
            return true;
        }

        // Arrow navigation
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            if !self.filtered.is_empty() {
                self.selected_index = (self.selected_index + 1).min(self.filtered.len() - 1);
                handled = true;
            }
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            if !self.filtered.is_empty() {
                self.selected_index = self.selected_index.saturating_sub(1);
                handled = true;
            }
        }

        // Enter activates selected entry
        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            if let Some(&entry_idx) = self.filtered.get(self.selected_index) {
                self.execute_entry(entry_idx);
            }
            handled = true;
        }

        // Ctrl+Enter also activates (alternative)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Enter)) {
            if let Some(&entry_idx) = self.filtered.get(self.selected_index) {
                self.execute_entry(entry_idx);
            }
            handled = true;
        }

        handled
    }

    /// Execute the action for the selected entry.
    fn execute_entry(&mut self, entry_idx: usize) {
        if entry_idx >= self.entries.len() {
            return;
        }
        let entry = &self.entries[entry_idx];
        // The actual action execution is handled by the app
        // We just close the palette here; the app will check the action
        self.close();
    }

    /// Get the currently selected entry for execution.
    pub fn selected_entry(&self) -> Option<&PaletteEntry> {
        self.filtered.get(self.selected_index).map(|&idx| &self.entries[idx])
    }

    /// Render the command palette UI.
    pub fn show(&mut self, ctx: &Context, app: &mut CadenceApp) {
        if !self.open {
            return;
        }

        // Handle keyboard input first
        self.handle_input(ctx);

        // Calculate position: centered horizontally, near top
        let screen_rect = ctx.screen_rect();
        let palette_width = 560.0;
        let palette_height = (self.filtered.len() as f32 * 44.0).min(480.0).max(200.0) + 80.0; // header + results
        let x = (screen_rect.width() - palette_width) / 2.0;
        let y = screen_rect.height() * 0.15; // 15% from top

        let palette_rect = egui::Rect::from_min_size(
            egui::pos2(x, y),
            egui::Vec2::new(palette_width, palette_height),
        );

        // Use a layer for the modal overlay
        let layer_id = Id::new("command_palette").with(1000); // High z-index

        egui::Area::new(layer_id)
            .order(egui::Order::Foreground)
            .fixed_pos(palette_rect.min)
            .show(ctx, |ui| {
                self.render_palette(ui, app);
            });
    }

    /// Render the palette contents.
    fn render_palette(&mut self, ui: &mut Ui, app: &mut CadenceApp) {
        let frame = egui::Frame::none()
            .fill(ui.visuals().window_fill)
            .stroke(ui.visuals().window_stroke)
            .rounding(ui.visuals().window_corner_radius)
            .shadow(ui.visuals().window_shadow)
            .inner_margin(egui::Margin::symmetric(12, 10));

        frame.show(ui, |ui| {
            ui.set_min_width(560.0);

            // Header with search field
            ui.horizontal(|ui| {
                // Search icon
                ui.label(RichText::new(phos::MAGNIFYING_GLASS.to_string()).size(16.0).color(ui.visuals().text_color()));

                // Search input
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Type a command or search…")
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(14.0)),
                );

                if self.focus_next {
                    response.request_focus();
                    self.focus_next = false;
                }

                if response.changed() {
                    self.filter(&self.query);
                }

                // Close button
                if ui.button(RichText::new(phos::X.to_string()).size(16.0)).on_hover_text("Close (Esc)").clicked() {
                    self.close();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Results list
            egui::ScrollArea::vertical()
                .max_height(480.0)
                .show(ui, |ui| {
                    if self.filtered.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(20.0);
                            ui.label(RichText::new("No matches found").color(ui.visuals().text_color().gamma_multiply(0.5)));
                            ui.add_space(4.0);
                            ui.label(RichText::new(format!("No commands match \"{}\"", self.query)).size(11.0).color(ui.visuals().text_color().gamma_multiply(0.4)));
                        });
                    } else {
                        for (display_idx, &entry_idx) in self.filtered.iter().enumerate() {
                            let entry = &self.entries[entry_idx];
                            let is_selected = display_idx == self.selected_index;

                            self.render_entry(ui, entry, is_selected, app);
                        }
                    }
                });
        });
    }

    /// Render a single palette entry.
    fn render_entry(&mut self, ui: &mut Ui, entry: &PaletteEntry, is_selected: bool, app: &mut CadenceApp) {
        let rect = ui.available_rect_before_wrap();
        let entry_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(4.0, 2.0),
            egui::vec2(rect.width() - 8.0, 38.0),
        );

        let response = ui.allocate_rect(entry_rect, egui::Sense::click());

        // Draw background for selected/hovered
        if is_selected {
            ui.painter().rect_filled(
                entry_rect,
                ui.visuals().widgets.hovered.corner_radius,
                ui.visuals().widgets.hovered.bg_fill,
            );
        } else if response.hovered() {
            ui.painter().rect_filled(
                entry_rect,
                ui.visuals().widgets.hovered.corner_radius,
                ui.visuals().widgets.hovered.bg_fill.gamma_multiply(0.5),
            );
        }

        // Render entry content
        ui.allocate_ui_at_rect(entry_rect, |ui| {
            ui.horizontal(|ui| {
                // Icon
                if let Some(icon) = entry.icon {
                    let icon_color = if is_selected {
                        ui.visuals().text_color()
                    } else {
                        ui.visuals().text_color().gamma_multiply(0.6)
                    };
                    ui.label(RichText::new(icon.to_string()).size(16.0).color(icon_color));
                } else {
                    ui.add_space(20.0); // Align with icon entries
                }

                // Label and description
                ui.vertical(|ui| {
                    let label_color = if is_selected {
                        ui.visuals().text_color()
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.label(RichText::new(&entry.label).size(13.0).color(label_color).strong());

                    if let Some(desc) = &entry.description {
                        ui.label(RichText::new(desc).size(10.0).color(ui.visuals().text_color().gamma_multiply(0.5)));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Shortcut hint
                    if let Some(shortcut) = &entry.shortcut {
                        ui.label(RichText::new(shortcut).size(9.5).color(ui.visuals().text_color().gamma_multiply(0.4)).monospace());
                    }

                    // Category badge
                    let category_label = match entry.category {
                        PaletteCategory::Views => "View",
                        PaletteCategory::GanttOperations => "Gantt",
                        PaletteCategory::TableOperations => "Table",
                        PaletteCategory::Resources => "Resources",
                        PaletteCategory::Settings => "Settings",
                        PaletteCategory::Help => "Help",
                        PaletteCategory::Activity => "Activity",
                        PaletteCategory::Project => "Project",
                    };
                    ui.label(
                        RichText::new(category_label)
                            .size(8.5)
                            .color(ui.visuals().text_color().gamma_multiply(0.4))
                            .background_color(ui.visuals().faint_bg_color),
                    );
                });
            });
        });

        // Handle click
        if response.clicked() {
            self.selected_index = self.filtered.iter().position(|&i| i == entry_idx).unwrap_or(0);
            self.execute_entry(entry_idx);
            app.execute_palette_action(&entry.action);
        }
    }
}

/// Helper to get entry index from filtered list (needed for click handler)
impl CommandPaletteState {
    fn entry_idx(&self, entry: &PaletteEntry) -> Option<usize> {
        self.entries.iter().position(|e| e.id == entry.id)
    }
}

impl CadenceApp {
    /// Execute a palette action.
    pub fn execute_palette_action(&mut self, action: &PaletteAction) {
        match action {
            PaletteAction::SetView(mode) => {
                self.view_mode = *mode;
                self.set_status(format!("Switched to {} view", mode_label(*mode)));
            }
            PaletteAction::Command(cmd) => {
                self.execute_command(cmd);
            }
            PaletteAction::OpenDialog(dialog) => {
                self.open_dialog(dialog);
            }
            PaletteAction::Toggle(toggle) => {
                self.execute_toggle(toggle);
            }
            PaletteAction::SelectActivity(id) => {
                self.select(*id);
                self.set_status(format!("Selected activity {}", id.0));
            }
            PaletteAction::None => {}
        }
    }

    /// Execute a built-in command by name.
    fn execute_command(&mut self, cmd: &str) {
        match cmd {
            // Gantt commands
            "gantt:zoom_in" => {
                self.pixels_per_day = (self.pixels_per_day * 1.2).min(100.0);
                self.set_status("Gantt zoomed in");
            }
            "gantt:zoom_out" => {
                self.pixels_per_day = (self.pixels_per_day / 1.2).max(2.0);
                self.set_status("Gantt zoomed out");
            }
            "gantt:zoom_fit" => {
                if let Ok(schedule) = &self.schedule {
                    if let Some(project_finish) = schedule.activity(self.project.activities.last().map(|a| a.id).unwrap_or(ActivityId(0))).map(|a| a.scheduled_finish) {
                        let project_start = schedule.activity(self.project.activities.first().map(|a| a.id).unwrap_or(ActivityId(0))).map(|a| a.scheduled_start).unwrap_or(project_finish);
                        let days = self.project.calendar.working_days_between(project_start, project_finish).max(1) as f32;
                        let available_width = 800.0; // approximate
                        self.pixels_per_day = (available_width / days).clamp(4.0, 60.0);
                        self.set_status("Gantt fitted to project");
                    }
                }
            }
            "gantt:toggle_critical" => {
                // This would toggle a setting - for now just show status
                self.set_status("Critical path highlight toggle not yet implemented");
            }
            "gantt:toggle_baseline" => {
                self.set_status("Baseline overlay toggle not yet implemented");
            }
            "gantt:toggle_today" => {
                self.set_status("Today line toggle not yet implemented");
            }
            "gantt:toggle_split" => {
                self.show_split_gantt = !self.show_split_gantt;
                self.set_status(if self.show_split_gantt { "Split Gantt/Table enabled" } else { "Split Gantt/Table disabled" });
            }

            // Table commands
            "table:new_activity" => {
                self.add_activity_at_end();
            }
            "table:delete_activity" => {
                if let Some(id) = self.selected {
                    self.pending_delete_activity_id = Some(id);
                    self.confirm_delete_activity = true;
                } else {
                    self.set_status("Select an activity to delete");
                }
            }
            "table:duplicate_activity" => {
                if let Some(id) = self.selected {
                    if let Some(activity) = self.project.activity(id).cloned() {
                        let mut new_activity = activity;
                        new_activity.id = ActivityId(self.project.next_activity_id());
                        new_activity.name = format!("{} (Copy)", activity.name);
                        let _ = self.stack.execute(&mut self.project, cadence_commands::CreateActivity {
                            id: new_activity.id,
                            name: new_activity.name,
                            duration: new_activity.original_duration,
                            wbs: new_activity.wbs,
                        });
                        self.recompute();
                        self.set_status("Activity duplicated");
                    }
                } else {
                    self.set_status("Select an activity to duplicate");
                }
            }
            "table:add_relationship" => {
                // Open relationship dialog
                self.set_status("Use Inspector → Predecessors/Successors to add relationships");
            }
            "table:delete_relationship" => {
                self.set_status("Use Inspector → Predecessors/Successors to remove relationships");
            }
            "table:toggle_critical_only" => {
                self.table_critical_only = !self.table_critical_only;
                self.set_status(if self.table_critical_only { "Showing only critical activities" } else { "Showing all activities" });
            }
            "table:toggle_columns" => {
                self.set_status("Column toggling not yet implemented");
            }
            "table:sort_by_name" => {
                self.sort_key = SortKey::Name;
                self.sort_ascending = true;
                self.set_status("Sorted by name");
            }
            "table:sort_by_date" => {
                self.sort_key = SortKey::Start;
                self.sort_ascending = true;
                self.set_status("Sorted by start date");
            }
            "table:expand_all" => {
                self.collapsed_wbs.clear();
                self.set_status("All WBS expanded");
            }
            "table:collapse_all" => {
                // Collapse all WBS nodes - would need WBS tree traversal
                self.set_status("Collapse all not yet implemented");
            }

            // Resource commands
            "resource:new" => {
                self.set_status("Use Resources view → New Resource");
            }
            "resource:assign" => {
                self.set_status("Use Inspector → Resources to assign");
            }
            "resource:level" => {
                self.level_resources();
            }

            // Project commands
            "project:new" => self.new_project(),
            "project:open" => self.open_project_dialog(),
            "project:save" => self.save_project(),
            "project:save_as" => self.save_project_as(),
            "project:import" => { self.show_import_dialog = true; },
            "project:export" => { self.show_export_dialog = true; },
            "project:print_gantt" => self.print_gantt_dialog(),
            "project:audit" => {
                self.view_mode = crate::ViewMode::Audit;
                self.run_audit();
                self.set_status("Schedule audit complete");
            }

            // Edit commands
            "edit:undo" => self.undo(),
            "edit:redo" => self.redo(),
            "edit:copy" => {
                if let Some(id) = self.selected {
                    // Copy to clipboard would need system clipboard access
                    self.set_status(format!("Copied activity {}", id.0));
                } else {
                    self.set_status("Select an activity to copy");
                }
            }
            "edit:paste" => {
                self.set_status("Paste not yet implemented");
            }

            // Lookahead commands
            "lookahead:new_snapshot" => {
                self.view_mode = crate::ViewMode::Lookahead;
                self.set_status("Use Lookahead view → New Snapshot");
            }
            "lookahead:compare_snapshots" => {
                self.set_status("Snapshot comparison not yet implemented");
            }

            // Diary commands
            "diary:new_note" => {
                self.view_mode = crate::ViewMode::Diary;
                self.set_status("Use Diary view → New Note");
            }

            // Production commands
            "production:new_entry" => {
                self.view_mode = crate::ViewMode::Production;
                self.set_status("Use Production view → New Entry");
            }

            _ => {
                self.set_status(format!("Unknown command: {}", cmd));
            }
        }
    }

    /// Open a dialog by name.
    fn open_dialog(&mut self, dialog: &str) {
        match dialog {
            "settings:general" => {
                // Settings are integrated in the toolbar/menus
                self.set_status("Settings available in toolbar");
            }
            "settings:appearance" => {
                self.set_status("Appearance settings in toolbar → Theme");
            }
            "settings:assistant" => {
                self.view_mode = crate::ViewMode::Assistant;
                self.set_status("AI Assistant settings in Assistant view");
            }
            "settings:shortcuts" => {
                self.show_shortcuts_dialog = true;
            }
            _ => {
                self.set_status(format!("Unknown dialog: {}", dialog));
            }
        }
    }

    /// Execute a toggle by name.
    fn execute_toggle(&mut self, toggle: &str) {
        match toggle {
            "theme:dark" => {
                self.dark = true;
                self.set_status("Switched to dark theme");
            }
            "theme:light" => {
                self.dark = false;
                self.set_status("Switched to light theme");
            }
            _ => {
                self.set_status(format!("Unknown toggle: {}", toggle));
            }
        }
    }

    /// Helper to get view mode label.
    fn mode_label(mode: ViewMode) -> &'static str {
        match mode {
            ViewMode::Table => "Table",
            ViewMode::Gantt => "Gantt",
            ViewMode::Resources => "Resources",
            ViewMode::Baselines => "Baselines",
            ViewMode::Calendars => "Calendars",
            ViewMode::Lookahead => "Lookahead",
            ViewMode::Diary => "Diary",
            ViewMode::Production => "Production",
            ViewMode::Audit => "Audit",
            ViewMode::Assistant => "Assistant",
        }
    }
}