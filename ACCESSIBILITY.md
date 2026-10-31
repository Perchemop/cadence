# Accessibility

Cadence ships as a desktop application (egui/eframe) and a command-line tool.
This document records the target, the evidence behind the current claims, and
the limits that remain.

## Target standard

**WCAG 2.2 AA**, applied to what an egui desktop application can meaningfully
support. Web-native requirements that have no native-window equivalent are
recorded as not applicable rather than ignored silently.

## Keyboard support

Every task has a pointer-free path. The global shortcuts are listed in the
application's **Keyboard shortcuts** window (toolbar **Shortcuts** or `F1`) and
in `docs/usage.md`.

- **View switching** — `Ctrl+1`–`Ctrl+5`; every view button is also reachable
  with `Tab` and activates with `Space` or `Enter`.
- **Activity table** — `Tab` reaches the sortable column headers; `Tab` reaches
  the row labels and `Space`/`Enter` selects a row. `Up`/`Down` select the
  previous/next activity in the current sort order and scroll it into view. The
  inspector exposes name, duration, constraint, calendar, progress, cost,
  resources and relationships as standard text fields, combo boxes, checkboxes
  and buttons.
- **Dependency links** — the inspector's **Add relationship** form chooses a
  predecessor, successor, link type and lag, then adds the link. Dragging
  between Gantt bars is a shortcut for the same command, not the only path.
- **Gantt** — the zoom buttons are ordinary buttons, and `Ctrl+Plus`,
  `Ctrl+Minus` and `Ctrl+0` zoom in, out and reset without a pointer. The chart
  carries a text summary and legend. Horizontal and vertical panning of the
  canvas remains pointer/wheel-driven; the Table view presents the same
  schedule data as a keyboard-navigable text table.
- **Resources** — the dictionary rows, capacity fields, assignment toggles,
  leveling preview/apply buttons and per-day usage table are all standard
  widgets. The leveling report is text.
- **Baselines** — capture, naming, selection, deletion and the variance/EVM
  grids are standard widgets; variance values are text, not colour-only.
- **Calendars** — named-calendar selection, the seven working-day checkboxes,
  the holidays field, create/apply/delete and per-activity assignment are all
  keyboard reachable.
- **Dialogs** — the crash, update and keyboard-shortcuts windows use standard
  egui controls. Save/open paths use the platform file dialog, which is
  keyboard operable on each supported desktop.

The keyboard-only pass was performed by reviewing every interactive path in
`crates/ui-egui/src/lib.rs` against the tasks described in `docs/usage.md`. No
task now requires dragging or clicking a custom-painted area.

## Screen-reader support

`eframe` 0.35 enables AccessKit by default (the `accesskit` feature is in its
default feature set) and `egui-winit` forwards the tree to the platform
adapter. Standard egui widgets therefore expose their role, label, enabled
state and selected state.

The Gantt chart is the one custom-painted surface. Its canvas response carries
an AccessKit label through `Response::widget_info`, summarising the activity
and critical counts and the project dates. Individual bars are not separate
AccessKit nodes; the Table view is the semantic alternative, with one row per
activity and text columns for duration, start, finish, float and criticality.
Status messages are visible text prefixed with `Status:`; they are not
announced as a live region.

Code-level screen-reader semantics are in place, but a manual pass with a
specific screen reader on each release platform (NVDA on Windows, VoiceOver on
macOS, Orca on Linux) is a release checklist item rather than something CI can
assert. The platform adapter's coverage is a toolkit/OS property, so no claim of
complete screen-reader support is made for unverified combinations.

## Contrast

Text and structural colours are tested at WCAG AA thresholds on both themes:

- variance text meets the 4.5:1 body-text minimum on the light and dark
  backgrounds (`variance_text_meets_wcag_aa_on_both_themes`);
- white text on the selection accent meets 4.5:1
  (`white_text_on_the_selection_accent_is_readable`);
- the accent meets the 3:1 non-text minimum against both backgrounds
  (`the_accent_is_distinguishable_on_both_themes`).

These tests live in the `accessibility_tests` module of
`crates/ui-egui/src/lib.rs` and run in every CI verification job.

## Known limitations

Recorded honestly, including what is not done:

- **Gantt bar semantics.** The canvas has a summary label but individual bars
  are not separate accessibility nodes. The Table view provides the same data
  as a text table and is the screen-reader path for per-activity detail.
- **Gantt panning.** Zoom is keyboard operable; panning the timeline canvas is
  pointer/wheel-driven. The Table view avoids panning entirely.
- **Live announcements.** Status text is visible and self-describing
  (`Status: …`) but is not exposed as a live region, so a screen reader will
  not interrupt to read it automatically. Errors that block an operation are
  also shown in the relevant view.
- **Combo-box popups.** egui 0.35 does not implement native listbox-style
  arrow-key navigation inside a `ComboBox` popup. The button and the items are
  focusable (Tab or the spatial focus arrows reach them, and Space/Enter
  activates), so the path is still keyboard-only, but long lists are slower
  than a native select. This is a toolkit limitation, not a pointer-only path.
- **Focus visibility in long panels.** The inspector is wrapped in a scroll
  area so it does not clip permanently, and focus follows construction order.
  On very short windows a focused control below the fold may need the panel
  scrolled into view; the control itself remains reachable with `Tab`.
- **Colour-only signals.** Critical Gantt bars are marked with a leading `!`
  and explained in the chart legend, so the meaning is not carried by colour
  alone. The progress fill is visual; the exact percentage is in the
  inspector's **Progress** section and in the report output.
- **PDF output.** The Gantt PDF page is an image and is not a tagged PDF. The
  Markdown report and the Table view are the accessible text alternatives.
- **Toolkit limits.** egui does not expose every native accessibility
  property. Support is limited to what AccessKit and the platform adapter
  provide; this is why manual screen-reader passes remain a release check.

## Supported environments

- Linux, Windows and macOS desktop builds published from releases.
- The CLI on the same three platforms; its plain-text reports are usable with a
  screen reader by construction.
- No web or mobile build is planned; no claims are made for those.

## Reporting an accessibility problem

Use the **Accessibility report** issue template (label `accessibility`), not the
general bug template. The template asks for what matters here: what you were
trying to do, which assistive technology and OS, and what Cadence did instead.

Accessibility reports are treated as correctness reports, not opinions. If a
feature is unusable by a whole group of people, that is a defect in the feature.
