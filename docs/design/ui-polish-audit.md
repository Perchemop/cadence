# UI polish audit and plan

A working audit of Cadence's desktop UI against the design directives in
[`brand.md`](brand.md), with the polish opportunities ranked by impact and
effort and the shippable slices that close them. Records the current state,
the gaps, and what each change touches.

## Method

Observations are from the running v0.10.0 build and a read of
`crates/ui-egui/src/lib.rs`. Behavioural comparisons with other products restate
publicly documented behaviour only (see [`../ui-patterns.md`](../ui-patterns.md));
nothing here copies another product's layout, icons, strings or trade dress.

## Directive-by-directive findings

| Directive | State | Notes |
|---|---|---|
| 1. Branding & chrome | Compliant | Plain window title, token palette, 4 px radii, 1 px hairlines; no proprietary assets. |
| 2. Top-level navigation | Partial | View-tab strip and toolbar present; toolbar button hierarchy thin; tab labels carry no shortcut hint. |
| 3. Buttons & hierarchy | Partial | Text buttons throughout; primary actions carry no accent weight (see below). |
| 4. Menus & combos | Compliant | Column menu and combo boxes navigate by keyboard; hover text on the column menu. |
| 5. Table & inspector | Partial | Virtualised rows, monospace numerics, sticky sortable header; active sort column was not visually distinguished. |
| 6. Gantt | Compliant | Critical/normal bars, milestones, summary bars, baseline ghosts, arrows with signed lag-label badges, and a 2 px today line with a shape marker — all present. |
| 7. Status bar | Partial → improved | Now shows the project range, activity count, critical count, earned-value metrics when a baseline exists, save state, and zoom. |
| 8. Dialogs | Compliant | Shortcuts, updates and crash-report windows share one button order and modality. |
| 9. Icons | Deferred | Text-only labels per brand; an original icon set is a later slice. Criticality/variance/progress/selection never rely on colour alone. |
| 10. Type, spacing, tokens | Compliant | Inter type scale, 4 px grid, monospace numerics, light/dark parity, WCAG AA accent verified by tests. |
| 11. Keyboard & discoverability | Partial → improved | F1 map, Ctrl+Z/Y, Ctrl+1..9, Ctrl+S/O/N/D, Gantt zoom; toolbar tooltips now name the shortcuts. |

## Prioritised opportunities

| # | Opportunity | Impact | Effort | Status |
|---|---|---|---|---|
| S1 | Status bar: earned-value metrics, critical count, save indicator, zoom | High | Low | Done |
| S2 | Toolbar: tooltips naming the keyboard shortcut on every action | High | Low | Done |
| S3 | Table: accent-colour the active sort column and label its order | Medium | Low | Done |
| S4 | Gantt: thicker today line with a shape marker; signed, badged lag labels | Medium | Medium | Done |
| S5 | Split view: a drag-to-resize hint on the split toggle | Medium | Low | Done |
| S6 | Empty states: a plain sentence and a shortcut per view | Medium | Low | Done |
| S7 | Discoverability: an F1 shortcuts footer in the status bar | Medium | Low | Done |

Deferred, with reason: original icon set (S9 in the directives; text-only per
`brand.md` for v1) and per-button visual shortcut badges (an egui text button
carries one text style, so a badge needs a custom widget; the shortcut is stated
in every button's tooltip and in the F1 footer instead).

## Slices in this change

**S1 — status bar.** The bottom bar gains a critical-activity count in the
critical colour, the most recent baseline's earned-value metrics (`PV EV AC SV
CV SPI CPI`) when a baseline exists (or `No baseline`), a save indicator that
reads as a word (`✓ saved` / `● unsaved`), and the zoom percentage beside the
theme toggle. Metrics are recomputed on edit and undo/redo via `recompute`, not
per frame, so the table and Gantt hot paths are untouched.

**S2 — toolbar tooltips.** Every primary toolbar action now carries
`on_hover_text` naming its shortcut: Open (`Ctrl+O`), Save (`Ctrl+S`), New
activity (`Ctrl+N`), Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Toggle theme (`Ctrl+D`),
and the shortcut window (`F1`), alongside the existing report, print, updates
and column-chooser hints.

**S3 — active sort column.** The active sort column's header renders in the
accent colour and its tooltip states the sort order, so the current sort is
readable without decoding a small glyph.

**S4 — Gantt legibility.** The today line is drawn at a 2 px weight with a small
downward marker at the top, so the marker is a shape cue and not colour alone.
Dependency lag labels are signed (`+2d` / `-1d`) and drawn on a small panel-
coloured badge so they stay readable where they cross the grid hairlines.

**S5 — split-view hint.** The **Split Gantt** toggle explains that the divider
between the table and the chart can be dragged to resize it, so the resize
handle is discoverable without trial and error.

**S6 — empty states.** Table, Baselines, Published snapshots and Diary now
state, in the brand's plain voice, what the empty surface is for and the one
action that fills it (with the shortcut where one exists). The Lookahead and
Resources views already did this.

**S7 — F1 footer.** The status bar carries a small **F1: shortcuts** control, so
the keyboard reference is one click from anywhere, matching the F1 binding, and
the feature is discoverable without hunting the toolbar.

## Acceptance criteria

- The status bar shows the critical count, the save state and the zoom at all
  times, and earned-value metrics whenever a baseline exists.
- Every primary toolbar action has a hover tooltip that names its shortcut.
- The active sort column is distinguishable by colour and by its tooltip text,
  not by colour alone.
- The today marker is distinguishable by shape as well as colour; lag labels
  carry their sign in the text.
- Every primary toolbar action has a hover tooltip that names its shortcut, and
  the F1 shortcuts reference is reachable from the status bar.
- No new dependency; the table and Gantt hot paths are unchanged.
- Light and dark themes keep WCAG AA contrast; AccessKit labels are unchanged.
