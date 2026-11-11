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

PR #90 (`docs/audit/2026-11-03-exhaustive-design-and-gui-audit.md`) catalogued 43 findings across S1 (blockers), S2 (major) and S3 (minor). The source audit is no longer in the tree (recoverable at commit `a10b740`); the tables below capture the S-id → finding mapping. Latest re-verification: 2026-11-12, against `main` @ `1248382` plus the 2026-11-11 token slice and the 2026-11-12 interaction slice.

**S1 blockers — all closed in the current tree** (the audit was written against an older revision):
- **S1-1 Create/delete activity:** `CreateActivity` wired via Ctrl+N and the **New activity** toolbar button (select + recompute + scroll-to-view); `DeleteActivity` wired in the inspector with a confirmation dialog (`confirm_delete_activity`). The empty state now names the action (S6).
- **S1-2 Production dead tab:** Production view renders an activity picker + date + percent + **Record** issuing `RecordProduction`/`DeleteProduction`; reachable via `Ctrl+9` and listed in the F1 dialog.
- **S1-3 Dirty state + close guard + window title:** `dirty: bool` tracked, cleared on save; viewport title shows `•` when dirty; close is guarded by a Save/Discard/Cancel modal.
- **AC2 — GUI Audit view:** selecting a finding now also sets `scroll_to_selection`, so it selects *and scrolls* to the activity in the Table view — acceptance criterion 2 fully met.

**S2 majors — partially closed:**
- **S2-2 Toolbar Undo/Redo:** now call `recompute()` + `refresh_selection()`, carry shortcut tooltips, and are disabled when the command stack is empty (`can_undo()`/`can_redo()`). Fully closed.
- **S2-3 Status feedback:** `StatusSeverity` (Info/Success/Warning/Error) colours messages by semantic token (`ok`/`warn`/`critical`/`text_muted`); info/success messages now auto-clear after five seconds (S2-3b). Errors and warnings persist.
- **S2-17 Shortcuts dialog:** now lists Ctrl+S, Ctrl+O, Ctrl+N, Production (Ctrl+1..9) and all other bindings. Closed.
- **S2-1 Toolbar/menu bar — partial:** the three-zone toolbar, tab tooltips,
  the zoom readout and the status-bar-only message are all in the tree; **focus
  rings are not** — they exist only on the unmerged branch
  `opencode/focus-rings` (commit `50b9b2d`). Remaining work: revive/reimplement
  the focus rings on `main`.
- **S2-4 Date fields — partial:** the `date_field` helper is live at five sites
  (Baselines, Audit, Lookahead, Diary, Production). The inspector's constraint
  date, actual start, actual finish and reschedule status date are still raw
  text fields whose parse errors land in the status bar.
- **S2-9 Location/WBS inspector fields — still open:** neither inspector
  exposes Location or WBS. The `SetActivityLocation`/`AssignActivityWbs`
  commands have no UI usage, and the Lookahead clash panel tells users to
  "Set a location in the inspector".
- **S2-10 Diary — done:** the date picker, note editing and the activity
  picker all ship; the diary activity is a ComboBox ("None" plus `id — name`
  entries), the Production view's pattern, so notes can stay unlinked.
- **S2-12 Lookahead cache — done:** the window cache is in, and the "new in
  window" comparison precomputes a `BTreeSet` of snapshot activity ids, so
  membership is O(1) rather than an O(n×m) scan per frame.
- **S2-16 Ctrl+wheel zoom — done:** the consume-handler (`lib.rs:6252-6262`)
  was dead under egui 0.35 because ctrl+wheel flowed into egui's zoom path.
  Fixed by disabling the global zoom (`Options.input_options.zoom_modifier =
  Modifiers::NONE`, `zoom_with_keyboard = false`), so the handler receives the
  wheel and zooms the chart; keyboard `Ctrl`+`+`/`-`/`0` no longer double-zoom
  the GUI.
- **S2-20 AccessKit `labelled_by` — partial:** 13 fields are wired; ~15
  visible-label field sites plus all combo boxes remain (F19 in
  `docs/audit/2026-11-07-full-visual-functional-audit.md`).
- Still open in S2: S2-9, plus the remaining halves of S2-1, S2-4 and S2-20
  above; the table below carries each row's status. The only
  release-checklist item that cannot be closed by code is the manual
  screen-reader verification (NVDA/VoiceOver/Orca).

**S3 minors — mixed; re-verified 2026-11-12.** All 18 S3 findings with their
verified status. This table captures the S-id → finding mapping (the source
audit file is absent from the tree; see the note above):

| ID | Finding | Status | Note |
|----|---------|--------|------|
| S3-1 | Non-token red error colours in Table/Resources/Baselines/Audit | Done (scope) | Routed through `StatusSeverity`/palette tokens; residual literals are being centralized in the current token slice. |
| S3-2 | `status_message` never auto-clears | Done | 5 s TTL with tests (same work as S2-3b). |
| S3-3 | Crash dialog cleared the log before the browser opened | Done | Order fixed: browser opens first, log cleared after the dialog closes. |
| S3-4 | Crash preview clones the whole log every frame | Done | Preview borrows the log via `as_str()`; no per-frame clone. |
| S3-5 | Gantt week-grid loop can spin forever on a zero-working-day calendar | Done (in flight) | Guard/break added in the current slice; verified in the working tree (`lib.rs:6076-6081`). |
| S3-6 | Doc drift: `ACCESSIBILITY.md` still mentions Ctrl+1–5 | Partial | Fixed in `ACCESSIBILITY.md`; stale "Ctrl+1..5" remains in `docs/parity.md` and historical docs. |
| S3-7 | No tooltips on the nine view tabs | Done | `on_hover_text` on each tab button. |
| S3-8 | No tooltips on toolbar Undo/Redo | Done | |
| S3-9 | Column chooser / split-Gantt always visible | Open | |
| S3-10 | Progress range hint | Partial | |
| S3-11 | Empty inspectors: nothing beyond a sentence | Open | |
| S3-12 | Focus rings on dense tables | Partial | No table-specific focus treatment; overlaps the S2-1 focus-ring work. |
| S3-13 | Diary note click-through | Open | |
| S3-14 | Lookahead jump from diary | Open | |
| S3-15 | Disabled Print Gantt control has no tooltip | Done | `on_disabled_hover_text`: "Switch to the Gantt view to print the Gantt chart." |
| S3-16 | Explicit reduced-motion record | In progress | Being added to `ACCESSIBILITY.md` in the current docs refresh. |
| S3-17 | Font symbol coverage for non-Latin imports | Unverified | |
| S3-18 | No window minimum size; toolbar wraps on extreme narrow widths | Done | Minimum window size asserted. |

## Directive-by-directive findings

| Directive | State | Notes |
|---|---|---|
| 1. Branding & chrome | Compliant | Plain window title, token palette, 4 px radii, 1 px hairlines; no proprietary assets. |
| 2. Top-level navigation | Partial | View-tab strip and toolbar present; tab tooltips added (S3-7, done); toolbar is dense but functional (S2-1 partial — focus rings remaining). |
| 3. Buttons & hierarchy | Partial | Text buttons throughout; primary actions carry no accent weight (see below). Toolbar Undo/Redo are disabled when the stack is empty (S2-2b, done). |
| 4. Menus & combos | Compliant | Column menu and combo boxes navigate by keyboard; hover text on the column menu. |
| 5. Table & inspector | Partial | Virtualised rows, monospace numerics, sticky sortable header; the active sort column is now accent-coloured and its tooltip names the order; inspector Location/WBS fields still missing (S2-9). |
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

Findings from the PR #90 backlog, re-verified against the current tree on
2026-11-12 (`main` @ `1248382` plus the 2026-11-11 token slice and the
2026-11-12 interaction slice) and given a
per-row status, ranked by the audit's severity and the project's existing
remediation pillars (P0 authoring, P1 feedback/shell, P2 Gantt as working
chart, P3 density/completeness). S3 minors are tracked in the S3 table above.
Open and partial items are queued so they are not missed; none of them block
the current slices.

| ID | Finding | Severity | Fix hint | Pillar | Status |
|----|---------|----------|----------|--------|--------|
| S1-1c | Row context menu (New below / Delete / Duplicate / Link to selected) on table rows | S1 | Add a right-click context menu to table rows | P0 | Done |
| S2-1 | Overloaded toolbar; status message duplicated inside toolbar | S2 | `egui::menu::bar` (File/Edit/View/Help); compact tab strip; status only in bottom bar; remaining: focus rings (revive/reimplement — only on the unmerged branch `opencode/focus-rings`, 50b9b2d) | P1 | Partial |
| S2-2b | Toolbar Undo/Redo disabled when stack is empty | S2 | `add_enabled(stack.can_undo())` / `can_redo()` | P1 | Done |
| S2-3b | Success/info messages auto-clear after five seconds; errors and warnings persist | S2 | Timed auto-clear via `maybe_clear_status` using `Instant` + `STATUS_MESSAGE_TTL` | P1 | Done |
| S2-4 | Raw `YYYY-MM-DD` text fields; parse errors far from the input | S2 | Reusable `date_field` helper with inline validation; inspector dates (constraint, actual start/finish, reschedule status) still raw | P1 | Partial |
| S2-5 | Table: no filter, find, group or multi-select | S2 | Filter field + critical-only toggle; shift/ctrl range selection | P3 | Done |
| S2-6 | Gantt: bars not hit-testable; inspector unreachable from chart | S2 | Hit-test bars on click; persistent inspector; two-way selection sync | P2 | Done |
| S2-7 | Drag-to-link always FS lag-0 regardless of modifier/type | S2 | Default FS; modifier or pop-up for SS/FF/SF + lag | P2 | Done |
| S2-8 | Relationships can be added but never removed or retyped in GUI | S2 | Delete button + confirm; type ComboBox issuing update command | P3 | Done |
| S2-9 | Inspector missing Location and WBS fields | S2 | Add fields (first-class in model/commands, but no UI usage today; the Lookahead clash panel points at the missing inspector fields) | P3 | Open |
| S2-10 | Diary: raw numeric Activity ID; notes not editable after creation | S2 | Date picker + note editing + activity picker (Production's ComboBox pattern); notes can stay unlinked | P3 | Done |
| S2-11 | Resources: conflicts/histogram/sort recompute every frame | S2 | Cache against `edit_version` | P3 | Done |
| S2-12 | Lookahead: O(n²) snapshot comparison every frame | S2 | Window cache plus a `BTreeSet` of snapshot activity ids — O(1) membership per frame | P3 | Done |
| S2-13 | Status bar: critical-activity recount every frame | S2 | Compute once in `recompute()`, store the count | P1 | Done |
| S2-14 | Save report writes to CWD without a dialog | S2 | Use the save-file dialog pattern (like Print report) | P1 | Done |
| S2-15 | Leveling applies N separate undo entries (partial state on mid-batch failure) | S2 | Composite/transactional command for atomic undo | P3 | Done |
| S2-16 | Ctrl+wheel zooms and scrolls simultaneously | S2 | `Options.input_options.zoom_modifier = Modifiers::NONE` + `zoom_with_keyboard = false`, so the existing Ctrl+wheel handler zooms the chart and keyboard zoom does not double-apply | P1 | Done |
| S2-18 | Confirmation dialogs inconsistent across destructive actions | S2 | Uniform confirm pattern for delete activity / destructive leveling / calendar delete (undoable; refused while in use) | P1 | Done |
| S2-20 | AccessKit: many label+field pairs not linked via `labelled_by` | S2 | Systematic `labelled_by` for every form field (13 wired; ~15 visible-label field sites + all combos remain — F19 in `docs/audit/2026-11-07-full-visual-functional-audit.md`) | P1 | Partial |
| S2-21 | Gantt zoom applies one frame after button press | S2 | Apply `pixels_per_day` before drawing the canvas | P2 | Done |
| S2-22 | Theme toggle duplicated in toolbar and status bar | S2 | Differentiate or remove one | P1 | Done |
| — | Accessibility: manual screen-reader verification (NVDA/VoiceOver/Orca) | release | Release-checklist item; cannot be fully automated | — | — |

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
