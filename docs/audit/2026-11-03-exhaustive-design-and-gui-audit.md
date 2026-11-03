# Exhaustive design and GUI audit — Cadence desktop app

- **Date:** 2026-11-03
- **Scope:** Full visual and interaction design of the egui/eframe desktop shell (`crates/ui-egui/src/lib.rs`, `design.rs`, `crash.rs`, `update_check.rs`) together with the launch path in `crates/app/src/main.rs`. Cross-checked against the command layer (`crates/commands`), the model (`crates/core/src/model.rs`), the brand tokens (`docs/design/brand.md`), the UI pattern matrix (`docs/ui-patterns.md`), accessibility claims (`ACCESSIBILITY.md`), and the prior audits of 2026-10-30 and 2026-11-01.
- **Method:** Exhaustive static analysis of the current tree (recursive repository tree + targeted content retrieval of every UI surface). Findings verified by code presence/absence (including `CreateActivity` / `DeleteActivity` search confirming they remain unused in the UI crate). Judged against Nielsen's heuristics, the project's own brand voice, and the paid-desktop bar (MS Project, Primavera P6, Asta Powerproject) as documented in `docs/ui-patterns.md`. Read-only; no source was modified.
- **Severity:**
  - **S1** — blocker (breaks the product story, causes silent data loss, or renders a core entity unusable)
  - **S2** — major (daily friction, fails paid-app expectations, consistency or feedback failures)
  - **S3** — minor (polish, residual token drift, documentation drift, low-frequency paths)
- **Line citations** refer to `crates/ui-egui/src/lib.rs` unless otherwise noted. Design tokens live in `crates/ui-egui/src/design.rs`.

## Executive summary

Cadence's scheduling engine, command stack, interchange formats and offline guarantees are mature. The design system itself (`design.rs` + `docs/design/brand.md`) is coherent, restrained and professionally intentioned: Inter type scale, 4 px rhythm, semantic colour tokens, WCAG-AA verified accent, hairline structure, and the explicit principle that "the schedule is the hero".

The GUI shell has not yet caught up. Three structural problems dominate:

1. **It is still not a full authoring tool.** Activities cannot be created or deleted from the GUI. The empty state offers no call to action. Production is a dead tab. The app can open, tweak and save an existing programme, but a planner cannot start one from scratch inside the window.
2. **Feedback and chrome contradict the brand.** Every status message renders in error red; success and failure are visually identical. The toolbar is an overloaded, wrapping soup of text buttons that mixes global actions, view tabs, local toggles and transient status. There is no dirty-state indicator and no unsaved-changes guard on close.
3. **The Gantt remains a diagram rather than a working scheduler's chart.** Bars are not selectable, the inspector is unreachable while looking at the chart, drag-to-link is locked to FS lag-0, and several high-value visual conventions (milestones, non-working shading, baseline ghosts, two-tier timescale) are absent.

The prior audits of 2026-10-30 (`docs/qc/gui-audit.md`) and 2026-11-01 (`docs/audit/2026-11-01-ux-critique.md`, surface inventory, Gantt gap analysis, remediation plan) remain substantially accurate. The present document consolidates them, confirms which findings are still open, and adds residual design-system and accessibility observations.

| Severity | Count |
|---|---:|
| S1 blocker | 3 |
| S2 major | 22 |
| S3 minor | 18 |
| **Total** | **43** |

**Single highest-impact defect:** S1-1 — the GUI cannot create or delete an activity. Until this is closed the product cannot honestly claim to be a scheduling application; it is a sophisticated viewer and editor of schedules authored elsewhere.

---

## S1 — Blockers

### S1-1 · Table / whole app · No way to create or delete an activity

- **What is wrong:** `CreateActivity`, `DeleteActivity` and `RestoreActivity` exist in `cadence-commands` and are exercised by the CLI and tests, but a repository-wide search of `crates/ui-egui` returns zero references. The empty state is the plain label **"No activities yet."** (`lib.rs` ~2086) with no button or instructional sentence. Row interaction is `Sense::click()` for selection only; there is no context menu. Resources, baselines and calendars all expose create/delete controls in their inspectors; the core entity of the product does not.
- **Why it fails the bar:** In MS Project, P6 and every professional scheduler the first interaction is typing or inserting a task. Everything else in Cadence (relationships, resources, leveling, baselines, audit, lookahead) presupposes that activities already exist. Brand document (`docs/design/brand.md`) states "the schedule is the hero"; a hero that cannot be authored inside the application is a contradiction.
- **Evidence of persistence:** Confirmed still open by code search on 2026-11-03.
- **Fix:**
  1. Toolbar **"New activity"** (next to Open…) + `Ctrl+N` executing `CreateActivity` with a sensible default name and duration, followed by `select`, `recompute` and scroll-into-view.
  2. Inspector **"Delete activity"** (enabled when a selection exists) with a confirmation dialog.
  3. Row context menu (New below / Delete / Duplicate / Link to selected).
  4. Empty-state CTA that matches brand voice: a sentence plus one primary button ("No activities yet. Add the first activity").

### S1-2 · Production · Dead tab

- **What is wrong:** The entire view is a heading **"Production tracking"** and the sentence **"Record daily actual progress against the schedule."** (`lib.rs` ~3168–3173). No form, no table of entries, no empty state, no error path. The `production_*` buffers and the commands `RecordProduction` / `DeleteProduction` / `RestoreProduction` exist but are never wired. The view is also unreachable by keyboard (`Ctrl+9` is not present in the key array).
- **Why it fails the bar:** A paid desktop application never ships a permanently empty tab. Nine named tabs imply nine functional surfaces; one of them is a placeholder. This reads as prototype-grade.
- **Fix:** Either (a) remove the tab from the strip until the roadmap slice is ready, or (b) implement a minimal real surface: plan-vs-actual rows using existing variance machinery plus an empty state that routes the user to the Table inspector's percent-complete field. Bind `Ctrl+9` and add the shortcut to the F1 dialog.

### S1-3 · App shell · No dirty state, no unsaved-changes prompt, static window title

- **What is wrong:** No `dirty` flag is tracked. The native window title is the constant `"Cadence"`. Closing the window (or quitting) never prompts. `Ctrl+S` exists and works, but nothing tells the user it is required.
- **Why it fails the bar:** Silent loss of edits on close is data loss. Every professional desktop application (and every modern tool in the Linear / VS Code class) marks unsaved state and protects it.
- **Fix:**
  1. `dirty: bool` set on every successful command execution / `recompute`, cleared on successful `save_to`.
  2. Viewport title updated each frame: `"{name}{dirty marker} — Cadence"`.
  3. On `close_requested`, if dirty, cancel the close and show a modal ("Save changes to ⟨name⟩?") with Save / Discard / Cancel, reusing the existing save path.

---

## S2 — Major

### S2-1 · App shell · Overloaded toolbar; status message lives inside it

- The single `horizontal_wrapped` toolbar carries identity, file actions, nine view tabs, Split Gantt, Columns, three report/print actions, bug reporting, updates, Undo/Redo, the live **"Status: …"** string, theme toggle and Shortcuts.
- Status text is also rendered (in a different red) in the bottom status bar.
- Heuristic: aesthetic and minimalist design; consistency and standards.
- **Fix:** Introduce an `egui::menu::bar` (File / Edit / View / Help). Keep a compact tab strip. Move status exclusively to the bottom bar. Remove the toolbar status copy.

### S2-2 · App shell · Toolbar Undo / Redo skip inspector refresh and give no state feedback

- Keyboard path (`Ctrl+Z` / `Ctrl+Y`) correctly calls `refresh_selection()` after the stack operation. Toolbar buttons do not. After a toolbar undo the inspector buffers remain stale, so the next **Apply** recreates the undone edit.
- Buttons are never disabled when the stack is empty.
- **Fix:** Call `refresh_selection()` from both toolbar handlers; expose `can_undo` / `can_redo` (or mirror counts) and use `add_enabled`.

### S2-3 · Status feedback · Every message renders in error red

- Success strings ("Saved to …", "Diary note added.", "Opened …", "Lookahead snapshot published.") and genuine errors share the same hard-coded dark red (`#C0392B` or `Color32::RED`).
- Brand document reserves colour for meaning; the palette already defines `ok`, `warn` and `critical`.
- **Fix:** Replace `status_message: Option<String>` with a typed status (`Ok | Warn | Error`) and a single render helper that selects the correct token. Auto-clear `Ok` messages after a few seconds.

### S2-4 · Dates · Raw `YYYY-MM-DD` text fields; errors raised far from the input

- Eight sites hand-roll the same pattern (constraint date, actual start/finish, status date, lookahead window, diary date, etc.).
- Parse failures frequently appear in the opposite panel or the status bar rather than beside the field.
- **Fix:** One reusable `date_field` helper that draws label + field + inline hint/validation. Prefer `egui_extras::DatePicker` where the dependency is acceptable; otherwise keep text but surface errors inline.

### S2-5 · Table · No filter, find, group or multi-select

- Sorting is present and polarity is indicated (▲/▼). Filtering, grouping, find-in-table and multi-select are absent.
- Paid daily workflow for programmes of even a few hundred activities depends on slicing.
- **Fix:** Filter field + critical-only toggle above the sticky header; selection model that supports shift/ctrl ranges; bulk actions once multi-select exists.

### S2-6 · Gantt · Chart view loses table, inspector and selection

- Bars are not hit-tested for click. Selection cannot be set from the Gantt view. No inspector panel is shown. Split table exists only inside Table view.
- **Fix:** Hit-test bars on click; keep a persistent inspector (or restore the split) in Gantt view; synchronise selection both ways.

### S2-7 · Gantt · Drag-to-link is always FS lag-0

- Drag creates a finish-to-start relationship with zero lag regardless of modifier keys or a type picker.
- Arrow geometry already branches correctly on link type for *display*; creation does not.
- **Fix:** Default remains FS; modifier or small pop-up after release allows SS/FF/SF and lag.

### S2-8 · Relationships · Links can be added but never removed or retyped from the GUI

- No use of `DeleteRelationship`. Lag can be edited; type cannot. The relationship picker is display + lag only.
- **Fix:** Delete button (with confirm) next to the selected relationship; type ComboBox that issues an update command.

### S2-9 · Inspector · Missing Location and WBS fields

- Model and commands support `SetActivityLocation` and WBS. Lookahead view even instructs the user to "set a location in the inspector". The field does not exist.
- **Fix:** Add the fields; they are already first-class in the model.

### S2-10 · Diary · Activity ID is a raw number; non-numeric input fails silently

- No activity picker; the field is a narrow text edit expecting a numeric ID. Parse failure produces no visible error in some paths.
- Notes cannot be edited after creation (`UpdateDailyNote` exists but is unused).
- **Fix:** ComboBox or searchable picker for the activity; surface parse errors; wire edit/delete consistently.

### S2-11 · Resources · Expensive work every frame

- Resource conflicts, histogram, full clone of the resource list and sorting run on every frame.
- **Fix:** Cache against `edit_version`.

### S2-12 · Lookahead · O(n²) snapshot comparison every frame

- Same class of problem as Resources.
- **Fix:** Cache the comparison window; use a map keyed by identity.

### S2-13 · Status bar · Critical-activity recount every frame

- O(activities) lookup per paint.
- **Fix:** Compute once inside `recompute()` and store the count.

### S2-14 · File actions · "Save report" writes to opaque CWD without dialog

- Unlike Print report / Print Gantt / Save as…, "Save report" drops a Markdown file next to the process working directory with no confirmation or path choice.
- **Fix:** Use the same save-file dialog pattern as the PDF actions, or at least confirm the path.

### S2-15 · Leveling · Applies N separate undo entries

- A mid-batch failure can leave the schedule in a partial state.
- **Fix:** Composite command (or explicit transaction) so undo is atomic.

### S2-16 · Ctrl+wheel · Zooms and scrolls simultaneously

- Scroll delta is not fully consumed when zooming.
- **Fix:** Consume the event when the zoom modifier is active.

### S2-17 · Shortcuts dialog · Incomplete and slightly stale

- Lists Ctrl+1…8 but omits Ctrl+S / Ctrl+O and Production. Comment above the key array still mentions only five views.
- **Fix:** Keep the dialog in lock-step with the actual bindings; add the missing entries.

### S2-18 · Confirmation dialogs · Inconsistent presence

- Delete baseline / delete resource have enabled-state guards; delete activity (once added) and destructive leveling need explicit confirmation. Pattern should be uniform.

### S2-19 · Production keyboard · Unreachable

- Already covered under S1-2; listed here for completeness of the keyboard map.

### S2-20 · Text-field labelling · AccessKit association incomplete

- Many `ui.label` + `text_edit_singleline` pairs are not linked via `labelled_by`. Screen-reader users hear the control without its purpose.
- **Fix:** Systematic use of labelled_by (or the equivalent helper) for every form field.

### S2-21 · Gantt zoom · One-frame lag after button press

- New `pixels_per_day` is applied after the current paint in some paths.
- **Fix:** Apply the value before the canvas is drawn.

### S2-22 · Theme toggle · Duplicated control

- Present in both toolbar and status bar with identical behaviour and no differentiation.

---

## S3 — Minor / polish

1. Non-token red error colours remain in Table, Resources, Baselines and Audit panels (Gantt was corrected earlier).
2. `status_message` is never auto-cleared; stale success text lingers indefinitely.
3. Crash-dialog "Open GitHub issue" clears the log before the browser has opened.
4. Crash preview clones the entire log every frame.
5. Gantt week-grid loop can theoretically spin on a zero-working-day calendar (defensive advance-or-break needed).
6. Doc drift: `ACCESSIBILITY.md` still mentions Ctrl+1–5 in one place; F1 dialog and keyboard map need Production and file shortcuts.
7. No tooltips on the nine view tabs.
8. Undo / Redo toolbar buttons lack tooltips (keyboard equivalents are documented elsewhere).
9. Column chooser and Split Gantt are Table-centric but always visible.
10. Progress percentage field lacks an explicit 0–100 hint or unit.
11. No visual distinction between "no selection" empty inspectors beyond a single sentence.
12. Hyperlink colour and selection accent are correctly shared, but focus rings could be made more explicit on dense tables.
13. Diary notes list has no click-through to the linked activity.
14. Lookahead clash links correctly jump to Table; the same pattern is not offered from Diary.
15. "Print Gantt…" is correctly disabled outside Gantt view, but the disabled state could carry a tooltip explaining why.
16. No reduced-motion special casing beyond "do nothing custom" (acceptable, but worth an explicit comment).
17. Font fallback ordering is correct (Inter first); verify symbol coverage for the few non-Latin strings that may appear in imported files.
18. Window minimum size is not asserted; extreme narrow widths still wrap the toolbar into many lines.

---

## Design system residual observations

**What is already excellent**
- Token definitions in `design.rs` match `docs/design/brand.md` exactly (bg, panel, text, text-muted, accent, critical, normal, ok, warn, grid, deep).
- Type scale, spacing rhythm and corner radii are applied consistently through `install_style` and `apply`.
- Selection and hyperlink correctly use the single accent.
- Critical path is never colour-only (`!` prefix + legend + Critical column).
- Striped tables and hairline grids reinforce the dense professional aesthetic.

**What still drifts**
- Status and error text outside the Gantt ignore the semantic tokens.
- A few hard-coded fills for hover/active states exist alongside the tokenised ones; they should be derived from the palette for future theme variants.
- No elevation / shadow language is defined (egui windows already provide a little); keep it minimal per brand.

**Voice**
- Most labels and empty states already follow the brand ("plain, specific, unhysterical"). The empty activity state and the Production stub are the largest remaining voice violations.

---

## Accessibility residual (post-ACCESSIBILITY.md)

The document is honest and high-quality. Residual gaps that affect design:

- Individual Gantt bars remain non-nodes; Table is the declared alternative — acceptable provided the Table path stays complete.
- Status messages are not live regions; once status becomes typed and coloured, consider a short polite announcement for errors only.
- Focus visibility inside long scrolling inspectors can still drop below the fold on short windows; the control remains reachable, but a scroll-into-view on focus would improve the experience.
- Manual screen-reader verification on Windows (NVDA), macOS (VoiceOver) and Linux (Orca) remains a release-checklist item; it cannot be fully automated.

---

## Relationship to prior audits

| Prior document | Status of its findings |
|---|---|
| `docs/qc/gui-audit.md` (2026-10-30) | Critical save/open and several HIGH items fixed; open items 1–17 largely still open or only partially addressed |
| `docs/audit/2026-11-01-ui-surface-inventory.md` | Still an accurate map of the surface |
| `docs/audit/2026-11-01-ux-critique.md` | S1 and most S2 findings confirmed still present |
| `docs/audit/2026-11-01-gantt-gap-analysis.md` | Structural Gantt gaps remain |
| `docs/audit/2026-11-01-remediation-plan.md` | Pillars A–C and the P0–P3 slices remain the correct prioritisation |

This audit does not replace the surface inventory; it re-ranks and re-validates the experience against the current tree.

---

## Recommended next slices (aligned with existing remediation plan)

**P0 — authoring blockers (must ship before any claim of "scheduling application")**
- S1-1 Create / delete activities
- S1-2 Production (implement or hide)
- S1-3 Dirty state + close guard

**P1 — feedback and shell**
- Typed, token-coloured status + auto-clear
- Menu bar + cleaned toolbar / tab strip
- Toolbar Undo/Redo parity with keyboard
- Unified date field

**P2 — Gantt as working chart**
- Bar selection + inspector presence
- Non-FS drag-to-link
- Non-working shading, milestones, baseline ghosts (from Gantt gap analysis)

**P3 — density and completeness**
- Table filter / multi-select
- Relationship delete / retype
- Location + WBS fields
- Diary picker + note editing
- Performance caches
- Remaining token and doc drift

---

## Closing judgement

The foundations — engine, commands, tokens, type, accessibility intent, offline contract — are unusually solid for a young project. The remaining work is composition and completeness, not invention. Closing the three S1 items would transform the application from "excellent viewer of schedules authored elsewhere" into "the place a planner builds and owns the programme". That is the gap between the current GUI and the promise made in `VISION.md` and `docs/design/brand.md`.

All findings above are reproducible from the current tree. Severity assignments prioritise user impact and product-story integrity over pure code elegance.
