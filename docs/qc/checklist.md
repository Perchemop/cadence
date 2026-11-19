# Accessibility Smoke Test Checklist

Per II.21.4: run this checklist whenever the UI is touched. Record the result in the PR description (per II.30.8).

## Keyboard-only walk-through

- [ ] Every flow (create an item, edit it, traverse main views, open/close dialogs) is completable from the keyboard alone — no pointer required.
- [ ] `Tab` cycles focus in a sensible order through all interactive controls (menu bar items, view tabs, tool buttons, inspector fields, combo boxes).
- [ ] `Shift+Tab` reverses focus order.
- [ ] `Enter` activates the focused control or selectable row (buttons, checkboxes, table rows).
- [ ] `Esc` dismisses the topmost open dialog (cancel semantics).
- [ ] `Ctrl+Z` / `Ctrl+Shift+Z` / `Ctrl+Y` undo/redo work from any view.
- [ ] `Ctrl+1`–`Ctrl+9` switch the nine views.
- [ ] Focus is never trapped: `Esc` always exits the current dialog/modal, and `Tab` can always progress to the next control.

## Dialog Esc dismissal

- [ ] Every modal dialog (confirm, save, open, audit, etc.) can be dismissed with `Esc`.
- [ ] `Esc` gives focus back to the control that opened the dialog, or to the main window.
- [ ] No dialog keeps focus inside it after `Esc` press (no focus trapping).

## Screen-reader spot check

- [ ] **Narrator** (Windows): launch Cadence, tab through main views, verify roles/names are announced for:
  - Top bar menu items
  - View tabs (Ctrl+1..9 equivalents)
  - Activity table headers and rows
  - Gantt chart summary label
  - Inspector fields (name, duration, constraint, progress, resources, relationships)
  - Buttons (Save, Close, etc.)
  - Focus ring announcement when Tab moves to a focused control
- [ ] **NVDA** (Windows): same checks as Narrator above.
- [ ] **VoiceOver** (macOS): same checks as Narrator above.
- [ ] **Orca** (Linux): same checks as Narrator above.

> **Honest limitation (per ACCESSIBILITY.md §Known limitations):** Code-level AccessKit semantics are in place (eframe 0.35 enables it by default, egui-winit forwards the tree). Standard egui widgets expose their role, label, enabled state and selected state. However, a manual pass with a specific screen reader on each release platform is a **release checklist item, not something CI can assert**. The platform adapter's coverage is a toolkit/OS property — no claim of complete screen-reader support is made for unverified combinations. The Gantt chart canvas carries a summary `WidgetInfo` label but individual bars are not separate AccessKit nodes.

## Reflow / zoom

- [ ] UI reflows gracefully at **200% zoom** — no clipping or overlapping of controls.
- [ ] UI is usable at **narrow window width** — side panel collapses before the canvas shrinks; no control becomes unreachable.
- [ ] At 200% zoom, text remains readable and contrast thresholds are still met.

## High-contrast (both themes)

- [ ] WCAG AA contrast (4.5:1 for body text, 3:1 for UI boundaries and focus rings) verified on the **light theme**.
- [ ] WCAG AA contrast (4.5:1 for body text, 3:1 for UI boundaries and focus rings) verified on the **dark theme**.
- [ ] Keyboard focus ring (colour `focus.ring`) is ≥ 3:1 non-text contrast against both the panel background and the control background on both themes.
- [ ] Text and structural colours are tested at WCAG AA thresholds on both themes (the contrast suite runs in every CI verification job — `accessibility_tests` module of `crates/ui-egui/src/lib.rs`).

## Recording instruction

Record the result of this checklist in the **PR description** (not just a passing/failing flag). Include:

- Which OS was tested on (Linux/Windows/macOS).
- Which screen reader(s) were used (if any), and the outcome.
- Any failures or limitations observed (be specific — e.g. "Gantt bar not announced", "focus ring vanishes on pointer press", etc.).
- Pass/fail summary per section (keyboard, dialogs, screen-reader, reflow, contrast).

If any section cannot be completed (e.g. screen reader not available on the test OS), note that explicitly rather than omitting it.

---

**Source:** II.21.4 ("Run the accessibility smoke test in docs/qc/checklist.md whenever the UI is touched"), II.30.8 (evidence plan), ACCESSIBILITY.md (keyboard map, screen-reader limits, contrast tests, known limitations).