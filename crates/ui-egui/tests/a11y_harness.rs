//! AccessKit-tree assertions over the real Cadence app shell.
//!
//! Evidence pipeline phase 2a (guide II.30): these tests drive
//! [`CadenceApp::render`] — the same headless seam the in-crate render tests
//! use — through an `egui_kittest` harness and assert against the AccessKit
//! tree egui emits, not against pixels. The harness renders on the CPU (the
//! default software renderer; the `wgpu`/`snapshot` features are deliberately
//! not enabled in the dev-dependency), so the suite runs wherever
//! `cargo test --workspace` runs — no GPU, no display, no windowing system.
//!
//! Gates covered:
//!
//! 1. **Reachability.** Every piece of chrome a user must operate — the nine
//!    view tabs, the menu bar, Undo/Redo, the status-bar Shortcuts button,
//!    the bottom-right version button, the table filter field and the
//!    "Critical only" checkbox — is exposed to AccessKit with a label
//!    (S2-20: no unreachable controls).
//! 2. **Named inputs.** egui_kittest 0.35.0 does not yet auto-fail on input
//!    widgets without an accessible name (that check landed in a later
//!    release), so this file re-implements the check by hand: after each of
//!    two frames, for both themes, every visible input widget (button,
//!    checkbox, text field, menu item, …) must carry a non-empty accessible
//!    name. The shell ties each text field to its label with
//!    `Response::labelled_by` — these tests lock that discipline in.
//!
//! API notes, verified against the docs.rs/source for egui_kittest 0.35.0,
//! kittest 0.4.0 and egui 0.35.0:
//!
//! - `Harness::builder().with_size(..).build_ui(closure)` enables AccessKit
//!   automatically, runs one bootstrap frame, and owns its `egui::Context` in
//!   the public field `h.ctx`. Cadence fonts and theme are installed on
//!   `h.ctx` right after the build (0.35.0 has no `with_context_setup` yet);
//!   the next frame re-lays out with the real type scale. Accessible names
//!   come from widget text, so the bootstrap frame's default fonts affect no
//!   assertion here. 0.35.0 also has no missing-glyph panic, so the Phosphor
//!   icon glyphs cannot break the run.
//! - Tab and toolbar buttons carry icon-prefixed labels (Phosphor PUA glyph,
//!   space, text), so queries use `get_all_by_label_contains` and filter by
//!   `Role::Button` instead of exact-match `get_by_label`. The one exact
//!   query in this file ("Shortcuts (F1)") has a unique, icon-free label.
//! - `Harness::run` loops while the UI demands *immediate* repaints and
//!   panics past `max_steps`. The fixture shell only requests *delayed*
//!   repaints (the update poller's 250 ms tick), which terminates the loop;
//!   `with_max_steps(8)` is belt-and-braces against a surprise animation.
//!
//! The fixture turns the periodic auto-update check off through the public
//! [`CadenceApp::set_update_checks_enabled`] seam: `cargo test` never touches
//! the network.

use cadence_core::model::{Activity, ActivityId, LinkType, Project, Relationship, Resource};
use cadence_ui_egui::update_check;
use cadence_ui_egui::{design, CadenceApp};
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use time::{Date, Month};

/// The nine view-tab labels of the shell, in display order.
const VIEW_TABS: [&str; 9] = [
    "Table",
    "Gantt",
    "Resources",
    "Baselines",
    "Calendars",
    "Audit",
    "Lookahead",
    "Diary",
    "Production",
];

/// The AccessKit roles a user can act on directly. This mirrors the input set
/// kittest's newer automatic name-check covers; kept explicit here because
/// egui_kittest 0.35.0 predates that check.
const INPUT_ROLES: &[Role] = &[
    Role::Button,
    Role::CheckBox,
    Role::ComboBox,
    Role::MenuItem,
    Role::RadioButton,
    Role::Slider,
    Role::SpinButton,
    Role::Tab,
    Role::TextInput,
];

/// The fixture project from the in-crate headless render tests: three
/// activities ("Survey"/2, "Build"/3, "Finish"/1), one FS relationship
/// 1 → 2 with lag 0, and one "Crew" resource assigned to activity 1.
fn build_project() -> Project {
    let mut project = Project::new(
        "AccessKit harness",
        Date::from_calendar_date(2026, Month::January, 5).expect("date"),
    );
    let survey = ActivityId::new(1);
    let build = ActivityId::new(2);
    let finish = ActivityId::new(3);
    project.activities.push(Activity::new(survey, "Survey", 2));
    project.activities.push(Activity::new(build, "Build", 3));
    project.activities.push(Activity::new(finish, "Finish", 1));
    let relationship_id = project.next_relationship_id();
    project.relationships.push(Relationship::new(
        relationship_id,
        survey,
        build,
        LinkType::Fs,
        0,
    ));
    let resource_id = project.next_resource_id();
    project.resources.push(Resource::new(resource_id, "Crew"));
    if let Some(activity) = project.activity_mut(survey) {
        activity.resources.push(resource_id);
    }
    project
}

/// The app under test: the real shell over the fixture project, with no
/// project path (nothing is read from or written to a project file) and the
/// auto-update check disabled (the harness is strictly offline).
fn build_app() -> CadenceApp {
    let mut app = CadenceApp::with_path(build_project(), None);
    app.set_update_checks_enabled(false);
    app
}

/// A kittest harness over the real shell: 1024×768, AccessKit enabled by the
/// builder, Cadence fonts + theme installed on the harness context, then one
/// post-install frame so queries see the themed shell.
fn build_harness(app: &mut CadenceApp, dark: bool) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(1024.0, 768.0))
        // Generous headroom past the bootstrap frames; `run` stops as soon as
        // no *immediate* repaint is requested, which the fixture shell never
        // needs.
        .with_max_steps(8)
        .build_ui(|ui| app.render(ui));
    // Fonts and theme must be on the harness's own context, which the builder
    // has already used for its bootstrap frame; the `run` below re-lays out.
    design::install_fonts(&harness.ctx);
    design::apply(&harness.ctx, dark);
    harness.run();
    harness
}

/// Role + accessible label of every node whose label contains `needle`.
///
/// `needle` must be `'static`: kittest's queries borrow the label for as long
/// as the tree. All needles in this file are literals or the crate version.
fn labels_containing(
    harness: &egui_kittest::Harness<'_>,
    needle: &'static str,
) -> Vec<(Role, Option<String>)> {
    harness
        .get_all_by_label_contains(needle)
        .map(|node| {
            let accesskit = node.accesskit_node();
            (accesskit.role(), accesskit.label())
        })
        .collect()
}

/// Labels of the `Role::Button` nodes whose accessible label contains
/// `needle`.
fn button_labels_containing(
    harness: &egui_kittest::Harness<'_>,
    needle: &'static str,
) -> Vec<String> {
    labels_containing(harness, needle)
        .into_iter()
        .filter(|(role, _)| *role == Role::Button)
        .map(|(_, label)| label.unwrap_or_default())
        .collect()
}

/// Accessible labels of every node with the given role (`None` when unnamed).
fn role_labels(harness: &egui_kittest::Harness<'_>, role: Role) -> Vec<Option<String>> {
    harness
        .get_all_by_role(role)
        .map(|node| node.accesskit_node().label())
        .collect()
}

/// The AccessKit `Disabled` state of every node whose label contains
/// `needle`.
fn disabled_states(harness: &egui_kittest::Harness<'_>, needle: &'static str) -> Vec<bool> {
    harness
        .get_all_by_label_contains(needle)
        .map(|node| node.accesskit_node().is_disabled())
        .collect()
}

/// Descriptions of every visible input widget that has no accessible name.
fn unnamed_inputs(harness: &egui_kittest::Harness<'_>) -> Vec<String> {
    let root = harness.root();
    root.children_recursive()
        .map(|node| node.accesskit_node())
        .filter(|node| INPUT_ROLES.contains(&node.role()))
        .filter(|node| !node.is_hidden())
        .filter(|node| node.label().map_or(true, |label| label.trim().is_empty()))
        .map(|node| format!("{:?} without an accessible name", node.role()))
        .collect()
}

/// The nine view tabs must each be reachable by name through the button role
/// (the tab strip is styled buttons, not an egui tab widget), exactly once.
#[test]
fn the_shell_exposes_view_tabs_to_accesskit() {
    let mut app = build_app();
    let harness = build_harness(&mut app, true);

    for name in VIEW_TABS {
        let tabs = button_labels_containing(&harness, name);
        assert_eq!(
            tabs.len(),
            1,
            "expected exactly one {name:?} view tab in the AccessKit tree, found: {tabs:?}"
        );
    }

    // The plain role query must also succeed without panicking: the shell
    // exposes at least as many buttons as it has view tabs.
    let buttons = harness.get_all_by_role(Role::Button).count();
    assert!(
        buttons >= VIEW_TABS.len(),
        "the shell exposes only {buttons} buttons for {len} view tabs",
        len = VIEW_TABS.len(),
    );
}

/// Toolbar, menu bar, status bar and version chrome must all be labelled —
/// including Undo/Redo, which sit disabled-but-labelled while the command
/// stack is empty (a screen reader must still find and announce them, with
/// the disabled state exposed, not silently dropped).
#[test]
fn the_toolbar_buttons_carry_labels() {
    let mut app = build_app();
    let harness = build_harness(&mut app, true);

    // Undo/Redo: present, button-role, labelled, and exposed as disabled
    // while there is nothing to undo/redo.
    for name in ["Undo (Ctrl+Z)", "Redo (Ctrl+Y)"] {
        let matches = labels_containing(&harness, name);
        assert!(
            !matches.is_empty(),
            "no AccessKit node carries {name:?}; the toolbar undo/redo pair is unreachable"
        );
        for (role, label) in &matches {
            assert_eq!(*role, Role::Button, "{name:?} should be a button");
            assert!(
                label.as_deref().is_some_and(|l| l.contains(name)),
                "the {name:?} button node is missing its label"
            );
        }
        let states = disabled_states(&harness, name);
        assert!(
            !states.is_empty() && states.iter().all(|disabled| *disabled),
            "{name:?} with an empty command stack must be exposed as disabled \
             (accesskit Disabled property), got: {states:?}"
        );
    }

    // Menu bar: File, Edit, View, Window, Help (menu buttons; the query is
    // role-agnostic because egui's menu-button role mapping is an
    // implementation detail).
    for menu in ["File", "Edit", "View", "Window", "Help"] {
        let matches = labels_containing(&harness, menu);
        assert!(
            !matches.is_empty(),
            "menu bar item {menu:?} is missing from the AccessKit tree"
        );
    }

    // Status bar: the F1 footer is queried by its exact, icon-free label —
    // the documented single-match query, which panics when the node is
    // absent (or duplicated).
    let _shortcuts = harness.get_by_label("Shortcuts (F1)");

    // Bottom-right version label is a button (it opens the release notes).
    let version = labels_containing(&harness, update_check::current_version());
    assert!(
        !version.is_empty(),
        "the bottom-right version label {:?} is not exposed to AccessKit",
        update_check::current_version(),
    );
}

/// Every input widget a user can act on must carry an accessible name, on
/// both themes and across two frames (egui sometimes needs a second pass for
/// layout). Additionally, the table filter text input must be reachable
/// through its visible label — the shell ties it to the "Filter:" label with
/// `Response::labelled_by` — and the "Critical only" checkbox keeps its label.
#[test]
fn every_input_widget_has_an_accessible_name() {
    for dark in [true, false] {
        let mut app = build_app();
        let mut harness = build_harness(&mut app, dark);

        for frame in 1..=2 {
            harness.run();
            let unnamed = unnamed_inputs(&harness);
            assert!(
                unnamed.is_empty(),
                "frame {frame} (dark={dark}): input widgets without an accessible name: \
                 {unnamed:#?}"
            );
        }

        // The filter field: a text input must exist, and its accessible name
        // must come from the visible "Filter:" label.
        let text_inputs = role_labels(&harness, Role::TextInput);
        assert!(
            !text_inputs.is_empty(),
            "no text input is exposed to AccessKit (dark={dark})"
        );
        assert!(
            text_inputs
                .iter()
                .any(|label| label.as_deref().is_some_and(|l| l.contains("Filter"))),
            "the table filter text input is not labelled \"Filter:\" \
             (text-input labels: {text_inputs:?})"
        );

        // ...and the critical-only checkbox keeps its visible label.
        let checkboxes = role_labels(&harness, Role::CheckBox);
        assert!(
            checkboxes
                .iter()
                .any(|label| label.as_deref() == Some("Critical only")),
            "the \"Critical only\" checkbox lost its label (checkbox labels: {checkboxes:?})"
        );
    }
}
