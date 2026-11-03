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

## Cross-check with PR #90 exhaustive audit (2026-11-03)

PR #90 (`docs/audit/2026-11-03-exhaustive-design-and-gui-audit.md`) catalogued 43 findings across S1 (blockers), S2 (major) and S3 (minor). Each was re-verified against the current tree.

**S1 blockers — all closed in the current tree** (the audit was written against an older revision):
- **S1-1 Create/delete activity:** `CreateActivity` wired via Ctrl+N and the **New activity** toolbar button (select + recompute + scroll-to-view); `DeleteActivity` wired in the inspector with a confirmation dialog (`confirm_delete_activity`). The empty state now names the action (S6).
- **S1-2 Production dead tab:** Production view renders an activity picker + date + percent + **Record** issuing `RecordProduction`/`DeleteProduction`; reachable via `Ctrl+9` and listed in the F1 dialog.
- **S1-3 Dirty state + close guard + window title:** `dirty: bool` tracked, cleared on save; viewport title shows `•` when dirty; close is guarded by a Save/Discard/Cancel modal.

**S2 majors — partially closed:**
- **S2-2 Toolbar Undo/Redo:** now call `recompute()` + `refresh_selection()` and carry shortcut tooltips. Still open: buttons not disabled when the stack is empty (`can_undo`/`can_redo` exist in `cadence-commands`, not yet used).
- **S2-3 Status feedback:** `StatusSeverity` (Info/Success/Warning/Error) colours messages by semantic token (`ok`/`warn`/`critical`/`text_muted`). Still open: no auto-clear of success messages.
- **S2-17 Shortcuts dialog:** now lists Ctrl+S, Ctrl+O, Ctrl+N, Production (Ctrl+1..9) and all other bindings. Closed.
- Still open: S2-1 (status lives in toolbar; menu bar), S2-4 (date field helper), S2-5 (table filter/multi-select), S2-6 (Gantt bar selection/inspector), S2-7 (drag-to-link FS lag-0 only), S2-8 (relationship delete/retype), S2-9 (Location/WBS inspector fields), S2-10 (Diary picker + note edit), S2-11 (per-frame resource work), S2-12 (per-frame lookahead comparison), S2-13 (per-frame critical count), S2-14 (Save report to CWD), S2-15 (leveling undo atomicity), S2-16 (Ctrl+wheel scroll), S2-18 (confirmation consistency), S2-20 (AccessKit labelled_by), S2-21 (Gantt zoom lag), S2-22 (duplicate theme toggle).

**S3 minors — all open** (doc drift, red non-token colours, crash preview, grid loop, tab tooltips, progress hint, focus rings, window min size, etc.), catalogued in the future-slices table below.

## Directive-by-directive findings

| Directive | State | Notes |
|---|---|---|
| 1. Branding & chrome | Compliant | Plain window title, token palette, 4 px radii, 1 px hairlines; no proprietary assets. |
| 2. Top-level navigation | Partial | View-tab strip and toolbar present; tab labels carry no shortcut hint (S3-7); toolbar is dense but functional (S2-1, future slice). |
| 3. Buttons & hierarchy | Partial | Text buttons throughout; primary actions carry no accent weight (see below). Toolbar Undo/Redo are enabled when the stack is empty (S2-2b, future slice). |
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

## Future slices (from PR #90 backlog)

Findings verified still-open against the current tree, ranked by the audit's
severity and the project's existing remediation pillars (P0 authoring, P1
feedback/shell, P2 Gantt as working chart, P3 density/completeness). None of
these block the current slices; they are queued so they are not missed.

| ID | Finding | Severity | Fix hint | Pillar |
|----|---------|----------|----------|--------|
| S1-1c | Row context menu (New below / Delete / Duplicate / Link to selected) on table rows | S1 | Add a right-click context menu to table rows | P0 |
| S2-1 | Overloaded toolbar; status message duplicated inside toolbar | S2 | `egui::menu::bar` (File/Edit/View/Help); compact tab strip; status only in bottom bar | P1 |
| S2-2b | Toolbar Undo/Redo not disabled when stack is empty | S2 | `add_enabled(stack.can_undo())` / `can_redo()` | P1 |
| S2-3b | Success messages never auto-clear; linger indefinitely | S2 | Timed auto-clear for `StatusSeverity::Success`/`Info` | P1 |
| S2-4 | Raw `YYYY-MM-DD` text fields; parse errors far from the input | S2 | Reusable `date_field` helper with inline validation | P1 |
| S2-5 | Table: no filter, find, group or multi-select | S2 | Filter field + critical-only toggle; shift/ctrl range selection | P3 |
| S2-6 | Gantt: bars not hit-testable; inspector unreachable from chart | S2 | Hit-test bars on click; persistent inspector; two-way selection sync | P2 |
| S2-7 | Drag-to-link always FS lag-0 regardless of modifier/type | S2 | Default FS; modifier or pop-up for SS/FF/SF + lag | P2 |
| S2-8 | Relationships can be added but never removed or retyped in GUI | S2 | Delete button + confirm; type ComboBox issuing update command | P3 |
| S2-9 | Inspector missing Location and WBS fields | S2 | Add fields (already first-class in model/commands) | P3 |
| S2-10 | Diary: raw numeric Activity ID; notes not editable after creation | S2 | Activity picker; surface parse errors; wire UpdateDailyNote/Delete | P3 |
| S2-11 | Resources: conflicts/histogram/sort recompute every frame | S2 | Cache against `edit_version` | P3 |
| S2-12 | Lookahead: O(n²) snapshot comparison every frame | S2 | Cache comparison window, keyed by identity | P3 |
| S2-13 | Status bar: critical-activity recount every frame | S2 | Compute once in `recompute()`, store the count | P1 |
| S2-14 | Save report writes to CWD without a dialog | S2 | Use the save-file dialog pattern (like Print report) | P1 |
| S2-15 | Leveling applies N separate undo entries (partial state on mid-batch failure) | S2 | Composite/transactional command for atomic undo | P3 |
| S2-16 | Ctrl+wheel zooms and scrolls simultaneously | S2 | Consume the scroll event when zoom modifier is active | P1 |
| S2-18 | Confirmation dialogs inconsistent across destructive actions | S2 | Uniform confirm pattern for delete activity / destructive leveling | P1 |
| S2-20 | AccessKit: many label+field pairs not linked via `labelled_by` | S2 | Systematic `labelled_by` for every form field | P1 |
| S2-21 | Gantt zoom applies one frame after button press | S2 | Apply `pixels_per_day` before drawing the canvas | P2 |
| S2-22 | Theme toggle duplicated in toolbar and status bar | S2 | Differentiate or remove one | P1 |
| S3-1 | Non-token red error colours in Table/Resources/Baselines/Audit | S3 | Route through `StatusSeverity`/palette tokens | P3 |
| S3-6 | `ACCESSIBILITY.md` still mentions Ctrl+1–5 | S3 | Update to Ctrl+1..9 + Production + file shortcuts | P3 |
| S3-7 | No tooltips on the nine view tabs | S3 | Add `on_hover_text` to each tab button | P3 |
| S3-18 | No window minimum size; toolbar wraps on extreme narrow widths | S3 | Assert a minimum window size | P3 |
| — | Accessibility: manual screen-reader verification (NVDA/VoiceOver/Orca) | release | Release-checklist item; cannot be fully automated | — |

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
