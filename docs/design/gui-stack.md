# GUI stack record (guide II.30.2)

Read-only detection of the toolkit Cadence actually builds against, recorded so
every GUI/UX slice starts from facts, not assumptions. `cargo` is never run on
the maintainer's machine; the versions below are read from `Cargo.lock` (the
resolved version — the only one that matters) and the crates' manifests.

## Toolkit and resolved versions

| Piece | Declared | Resolved (`Cargo.lock`) | Notes |
|---|---|---|---|
| `egui` | `0.35` | **0.35.0** | Immediate-mode GUI |
| `eframe` | `0.35` | **0.35.0** | Windowing, event loop, AccessKit on by default |
| `egui_extras` | `0.35.0` + `datepicker` | 0.35.0 | First-party widgets: table (`ScrollArea::show_rows`), `DatePickerButton` |
| `egui-phosphor` | `0.13.0` | 0.13.0 | Phosphor icon glyph font (MIT) |
| `jiff` | `0.2` | 0.2.x | Calendar dates for `DatePickerButton`; converted to `time::Date` at the field boundary |

One design path only: the project's own token module plus first-party
`egui`/`egui_extras` and the `egui-phosphor` icon font. No shadcn-egui, spaceui,
egui-material3, Fluent, or web/CSS design system is used, and none will be added
to this native GUI (guide II.30.1, II.30.14).

## Theme module — the single token source

- `crates/ui-egui/src/design.rs` — the semantic palette (`design::palette(dark)`
  → `Palette`), the type scale, spacing rhythm, the accent token (`design::ACCENT`),
  `install_fonts` (Inter + Phosphor), and `apply(ctx, dark)` which maps the
  tokens onto `egui::Visuals`/`Style`.
- Widget code reads roles (`palette.critical`, `palette.text_muted`, …); raw
  colour literals outside `design.rs` are a regression (guide II.30.6).

## Shell shape — what the harness can and cannot drive

- Entry point: `eframe::run_native` builds the app inside its creator closure
  with `CadenceApp::new(project)` / `CadenceApp::with_path(project, path)`, and
  `impl eframe::App for CadenceApp` provides `fn ui(&mut self, ui, _frame)`,
  which delegates to the public inherent method
  `CadenceApp::render(&mut self, ui: &mut Ui)`.
- The shell now renders into a plain `&mut Ui`: the whole UI body lives in
  `CadenceApp::render`, so it is **headless-drivable**. Phase 1 of the II.30
  evidence pipeline is in place — the `headless_render_tests` module drives it
  with a bare `egui::Context` (`begin_pass` / `CentralPanel::show` /
  `end_pass`) across every view mode, both themes, and two passes, asserting the
  shell paints. No GPU, no new dependency; runs in the normal `cargo test`
  suite on all four CI platforms.
- `eframe::CreationContext` and `eframe::Frame` still cannot be fabricated, so
  the windowed path still builds the app inside `eframe::run_native`; only the
  rendering body is shared.
- Evidence pipeline status: **phase 2a done** — `egui_kittest = "0.35.0"`
  is now a dev-dependency of `crates/ui-egui` (the `wgpu` and `snapshot`
  features stay **off**), and `crates/ui-egui/tests/a11y_harness.rs` drives
  the real shell (`CadenceApp::render`) through a kittest harness and
  asserts the AccessKit tree: all nine view tabs reachable by name; Undo/Redo
  present, labelled and exposed as `Disabled` while the stack is empty; the
  menu bar (File/Edit/View/Window/Help); the Shortcuts (`F1`) button; the
  version button; the Filter text input reachable via its visible label; the
  "Critical only" checkbox labelled; plus a hand-rolled named-input check
  (every visible input widget must carry a non-empty accessible name) across
  two frames and both themes. No GPU, no new runtime dependency; runs in the
  normal `cargo test` suite on all four CI platforms. The fixture turns the
  auto-update check off via `CadenceApp::set_update_checks_enabled(false)`
  so the tests are strictly offline. Phase 2b (rendered PNG artefacts via
  `wgpu` + `snapshot`, needing software Vulkan/lavapipe in CI, with a
  documented fallback to phase 2a's AccessKit evidence) remains queued.

## Window / shell layout model

- Top bar: menu bar (`File / Edit / View / Window / Help`) → view-tab strip
  (`Ctrl+1`–`Ctrl+9`) → tool cluster.
- Bottom: status bar (project dates, activity/critical counts, zoom, typed
  status message, `F1` shortcuts).
- Right: activity inspector side panel. Every inspector is a right panel
  (`Panel::right`): `inspector` for the Table view, plus the Gantt, Resources,
  Baselines and Calendars inspectors; there is no left panel.
- Centre: the active view (Table, Gantt, Resources, Baselines, Calendars,
  Audit, Lookahead, Diary, Production).
- Optional second right panel: split Gantt beside the Table view
  (`Panel::right("workbench_gantt")`).
- Fixed three-pane shell; no docking/tiling is a product goal, so `egui_dock`
  and `egui_tiles` are deliberately **not** adopted (guide II.30.4).

## Accessibility in place

- AccessKit is enabled by `eframe`'s default features; interactive widgets carry
  labels, and Gantt has a canvas summary. The contrast asserted in unit tests is
  specific, not palette-wide: the `variance_color` red/green pairs at the 4.5:1
  body-text minimum, and the accent at the 3:1 non-text minimum with white on
  the accent at 4.5:1 — each measured against synthetic worst-case backgrounds
  (white, near-black) rather than the real palette surfaces. Broader
  palette-surface contrast tests, a high-contrast theme, a 24 pt target-size
  check and focus-ring assertions are tracked follow-ups. Limits that cannot be
  met in egui are recorded in `ACCESSIBILITY.md` rather than glossed over.
- **Evidence (II.30.8).** Phase 2a ships AccessKit-tree assertions over the
  shell in `crates/ui-egui/tests/a11y_harness.rs` (kittest 0.35.0
  dev-dependency, no GPU, all four CI platforms); phase 2b rendered PNG
  artefacts (`wgpu` + `snapshot`, needing lavapipe) remain queued with a
  documented fallback to phase 2a's AccessKit evidence.

## How this record is kept current

Re-check with the read-only commands in guide II.30.2 whenever the GUI stack
changes; update the resolved versions here, and record any dependency move as
its own green slice (guide II.30.3) with a `CHANGELOG.md` note.
