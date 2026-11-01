# UX/UI critique — Cadence desktop app

- **Date:** 2026-11-01
- **Scope:** the 9 views (Table, Gantt, Resources, Baselines, Calendars, Audit,
  Lookahead, Diary, Production) and the app shell (toolbar, status bar,
  inspector/property panels, dialogs, empty states). Read-only analysis; no
  source was modified.
- **Method:** exhaustive read of `crates/ui-egui/src/lib.rs` (4,045 lines),
  `crates/ui-egui/src/design.rs`, `crates/app/src/main.rs`, cross-checked
  against `docs/qc/gui-audit.md`, `docs/qc/app-audit.md`,
  `docs/qc/capability-audit.md`, `docs/parity.md`, `docs/ui-patterns.md` and
  `docs/design/brand.md`. Findings verified in the code before ranking; this
  critique goes beyond the earlier audits by judging the experience against
  Nielsen's 10 heuristics and paid-desktop expectations (Microsoft Project,
  Primavera P6, Linear) rather than defect-hunting alone.
- **Severity:** **S1** blocker (breaks the product story or loses data) ·
  **S2** major (visible daily friction, fails the paid-app bar) ·
  **S3** minor (polish, consistency, tokens).

All `lib.rs:` line numbers refer to `crates/ui-egui/src/lib.rs` at the time of
writing (egui/eframe 0.35). Every finding cites the exact user-visible label.

## Executive summary

Cadence's engine and command layer are far ahead of its shell. The shell is
one overloaded toolbar, four fixed-width inspectors that share a confusing
create-or-edit pattern, raw ISO date text fields with validation errors
raised far from the input, status messages that always render in error red,
one dead tab (Production), and — decisively — **no way to create or delete an
activity anywhere in the GUI**. Nothing here is about aesthetics: the token
system in `design.rs` is good and mostly applied. The gaps are structural:
missing commands in the shell, inconsistent interaction models between views,
and feedback that contradicts itself.

| Severity | Count |
|---|---:|
| S1 blocker | 3 |
| S2 major | 20 |
| S3 minor | 24 |
| **Total** | **47** |

**The single most damaging issue:** S1-1 — the GUI cannot create or delete
activities. A scheduler whose window is a read-and-tweak surface over work
that must be authored elsewhere (CLI, imported file) is a viewer, not the
paid desktop tool the brand document commits to ("The schedule is the hero",
`docs/design/brand.md:19`). Fix: wire the existing `CreateActivity` /
`DeleteActivity` commands into a toolbar button, an inspector section and a
row context menu (details in S1-1).

---

## S1 — Blockers

### S1-1 · Table (whole app) · No way to create or delete an activity

- **What is wrong:** `CreateActivity` and `DeleteActivity` exist in
  `cadence-commands` but are referenced nowhere in `crates/ui-egui/src/`
  (verified by grep; also recorded as open item 7 in
  `docs/qc/gui-audit.md:47`). The empty state — label **"No activities yet."**
  (`lib.rs:2086`) — offers no call to action, because no surface can add one.
  There is no row context menu either: the row interaction
  (`lib.rs:2117-2123`) is `Sense::click()` for selection only.
- **Why it fails the bar:** in MSP and P6 the first interaction is typing a
  task into the sheet. Everything else in Cadence (links, resources, levels,
  baselines, audit) presumes activities exist. The shell can delete
  resources, baselines and calendars (`lib.rs:2285, 2630, 2471`) but not the
  entity the whole app is about — a consistency failure *and* a capability
  hole that makes the app unusable from scratch.
- **Fix (code-level):**
  1. Add a **"New activity"** toolbar button (next to "Open…", `lib.rs:1522`)
     and Ctrl+N that executes
     `CreateActivity { id: self.project.next_activity_id(), name: "New activity", duration: 1 }`,
     then `self.select(id); self.recompute(); self.scroll_to_selection = true;`.
  2. Add a **"Delete activity"** button at the top of the Inspector
     (`lib.rs:1750`), enabled when `self.selected.is_some()`, executing
     `DeleteActivity { id }` after a confirmation dialog (see S2-18).
  3. Add a right-click context menu in the row loop
     (`lib.rs:2100-2215`): `if response.context_menu(|ui| { … })` with
     New activity below / Delete / Duplicate / Link to selected.
  4. Change the empty state at `lib.rs:2086` to a sentence plus one button:
     "No activities yet. **Add the first activity**" wired to the same
     command. Brand voice: states are sentences, errors say what to do next
     (`docs/design/brand.md:76`).

### S1-2 · Production · The view is a dead tab

- **What is wrong:** the entire Production view is
  heading **"Production tracking"** plus one sentence, **"Record daily actual
  progress against the schedule."** (`lib.rs:3168-3173`). No controls, no
  empty state, no roadmap pointer. It is also unreachable by keyboard (see
  S2-19) and missing from the F1 map.
- **Why it fails the bar:** a paid app never ships an empty tab. It reads as
  prototype-grade in the most visible way possible — the ninth of nine tabs
  does nothing, and tab labels imply parity of function.
- **Fix (code-level):** either (a) remove Production from the tab list at
  `lib.rs:1546-1556` until the roadmap slice lands
  (`docs/parity.md:103`), or (b) make it real with data the model already
  has: per-activity plan-vs-actual rows (scheduled vs `actual_start` /
  `actual_finish`, `percent_complete`) — the `VarianceRow` machinery at
  `lib.rs:3497` is reusable — plus an empty state that links percent entry
  to where it actually lives: *"No progress recorded yet. Select an activity
  in the Table view and enter Percent complete."* with a button that sets
  `self.view_mode = ViewMode::Table`.

### S1-3 · App shell · No dirty state, no unsaved-changes prompt, static window title

- **What is wrong:** grep finds no dirty/unsaved tracking anywhere. The
  window title is the constant `"Cadence"` (`lib.rs:3891-3892`), the project
  name sits in the toolbar (`lib.rs:1520`) but nothing ever indicates
  modification, and eframe closes without a prompt. Ctrl+S exists
  (`lib.rs:1344`); nothing tells you it is needed.
- **Why it fails the bar:** the gui-audit ranked "no save" CRITICAL and fixed
  it; the missing dirty marker + close prompt is the remaining half of that
  hole. Every paid desktop app (and every Linear-grade modern app) marks
  unsaved state and protects it on close. Silent loss on close is data loss.
- **Fix (code-level):**
  1. Add `dirty: bool`; set it in `recompute()` (`lib.rs:398-401`) and after
     every `stack.execute`; clear it in `save_to` (`lib.rs:1154-1164`).
  2. Reflect it in the title each frame:
     `ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{}{} — Cadence", self.project.name, if self.dirty { " •" } else { "" })))`.
  3. Guard close: in `ui()`, check
     `ctx.input(|i| i.viewport().close_requested())`; if `self.dirty`, issue
     `egui::ViewportCommand::CancelClose` and show a modal
     **"Save changes to ⟨project name⟩?"** with Save / Discard / Cancel,
     reusing `save_or_prompt` (`lib.rs:1146`).

---

## S2 — Major

### S2-1 · App shell · No menu bar; one overloaded toolbar; the status message lives in it

- **Label(s):** everything in one `horizontal_wrapped` row,
  `lib.rs:1516-1684`: "Cadence", project name, "Open…", "Save", "Save as…",
  nine view tabs, "Split Gantt: on/off", "Columns", "Save report",
  "Print report…", "Print Gantt…", "Report a bug…", "Updates", "Undo",
  "Redo", **"Status: ⟨message⟩"** (`lib.rs:1667-1669`), "Toggle theme",
  "Shortcuts".
- **Heuristics:** consistency and standards; aesthetic and minimalist design.
- **Why it fails:** global file commands, view tabs, view-local toggles,
  reporting, support, undo, *and transient status text* share one row that
  wraps unpredictably. The status message is also duplicated in the status
  bar (`lib.rs:1723-1731`). Paid desktop shells separate File/Edit/View/Help
  menus, a tab strip, and a status bar.
- **Fix:** add `egui::menu::bar` above the toolbar: **File** (Open…/Save/
  Save as…/Export CSV|JSON|XML — see S2-14/Print report…/Print Gantt…),
  **Edit** (Undo/Redo), **View** (tabs, Split Gantt, Columns), **Help**
  (Shortcuts/Report a bug…/Updates). Keep the tabs as their own
  `SelectableLabel` strip. Delete the toolbar status copy at
  `lib.rs:1667-1669`.

### S2-2 · App shell · Toolbar Undo/Redo skip inspector refresh and give no state feedback

- **Labels:** "Undo", "Redo" (`lib.rs:1659-1666`).
- **What is wrong:** the keyboard path calls `refresh_selection()` after
  undo/redo (`lib.rs:1364-1373`); the toolbar buttons do `let _ =
  self.stack.undo(...)` without it. After a toolbar undo, the inspector
  buffers still hold the undone values, so the next **"Apply"**
  (`lib.rs:1857`) re-creates the undone edit as a new command — the exact
  bug class the gui-audit fixed for Ctrl+Z (`docs/qc/gui-audit.md:19`).
  The buttons are also never disabled when the stack is empty.
- **Heuristic:** consistency; user control.
- **Fix:** call `self.refresh_selection()` in both toolbar handlers; expose
  `can_undo()/can_redo()` from `CommandStack` (or track a mirror count in
  the app) and render with `add_enabled(can_undo, …)`.

### S2-3 · App shell/status bar · Every status message renders in error red — success reads as failure

- **Labels affected:** "Saved to ⟨path⟩" (`lib.rs:1157`), "Report printed to
  ⟨path⟩" (`lib.rs:1133`), "Diary note added." (`lib.rs:3103`),
  "Lookahead snapshot published." (`lib.rs:2902`), "Opened ⟨path⟩"
  (`lib.rs:1206`) — all drawn with hardcoded
  `Color32::from_rgb(0xC0, 0x39, 0x2B)` at `lib.rs:1723-1731` and duplicated
  at `lib.rs:1667-1669`.
- **Why it fails:** colour is the app's meaning system
  (`docs/design/brand.md:23`); the palette already defines `ok`, `warn` and
  `critical` (`design.rs:49-53`) and none is used here. A user who saves and
  sees red reasonably believes the save failed.
- **Fix:** replace `status_message: Option<String>` with
  `status: Option<(StatusKind, String)>` (`Ok | Warn | Error`); one render
  helper picks `palette(dark).ok / warn / critical`; auto-clear `Ok` after
  ~5 s (also closes gui-audit open item 13, stale message).

### S2-4 · All views · Dates are raw `YYYY-MM-DD` text fields with errors raised far from the input

- **Labels:** "Constraint date (YYYY-MM-DD)" (`lib.rs:1771`), "Actual start
  (YYYY-MM-DD)" / "Actual finish (YYYY-MM-DD)" (`lib.rs:1852-1855`),
  "Status date (YYYY-MM-DD)" (`lib.rs:2479`), "Window starts (YYYY-MM-DD)"
  (`lib.rs:2800`), Diary "Date (YYYY-MM-DD)" / "Window start (optional)"
  (`lib.rs:3058, 3065`), Audit "Status date (YYYY-MM-DD, optional)"
  (`lib.rs:2710`).
- **What is wrong:** eight sites hand-roll the same text-field-and-parse
  pattern. Errors are raised away from the field: the Baselines parse
  failure **"Status date must be written as YYYY-MM-DD"** prints in the
  *centre* panel (`lib.rs:2501-2506`) while the field is in the *right
  inspector*; Audit/Lookahead/Diary errors go to the status bar
  (`lib.rs:2720-2723, 3110-3114`).
- **Heuristic:** help users recognise, diagnose and recover from errors.
- **Why it fails:** paid schedulers offer date pickers/spinners; at minimum
  inline validation. Typing `2026-13-40` and reading a red sentence on the
  other side of the window is prototype-grade.
- **Fix:** one helper
  `fn date_field(ui, id, label, buffer) -> Result<Date, String>` that
  renders label + field + an inline hint ("Use YYYY-MM-DD, e.g. 2026-11-02")
  beside the field, and reuse it at all eight sites. For actual
  start/finish, add ±day buttons or a calendar popup. Keep the status-bar
  message for *command* rejections only.

### S2-5 · Table · No filter, group-by, highlight or multi-select — the daily paid workflow

- **Surface:** `lib.rs:2019-2233` — sort-only table.
- **What is wrong:** no filter row, no grouping, no rule-based highlight, no
  find, and selection is single (`Sense::click()` row interact,
  `lib.rs:2117-2123`). `docs/ui-patterns.md:36` records filters/grouping as
  "considering"; capability-audit gap #1 ranks it the top gap.
- **Why it fails:** a planner cannot work a 5,000-activity programme — or
  honestly a 300-activity one — through a sort-only list. MSP/P6/Linear all
  treat slice-and-highlight as table basics.
- **Fix:** add `filter_text: String` and `filter_critical_only: bool`; apply
  the predicate inside `refresh_sorted()` (`lib.rs:1245-1308`); render a
  filter field above the sticky header row (`lib.rs:2052`). Change selection
  to `BTreeSet<ActivityId>` (or keep `selected` + a shift-click range) and
  add bulk actions (assign resource, set constraint, delete) once
  multi-select exists.

### S2-6 · Gantt · The Gantt-only view loses the table, the inspector and selection

- **Surface:** `lib.rs:2235-2255` — canvas + "Zoom −/+/Reset" row only.
- **What is wrong:** clicking a bar does nothing (the canvas response is
  `Sense::click_and_drag()` but click is unused, `lib.rs:3614-3697`);
  `self.selected` is never settable from here; no inspector panel exists in
  this view; the split table exists only inside Table view
  (`lib.rs:1985-2011`). Selection sync is admitted as a follow-up in
  `docs/ui-patterns.md:17`.
- **Why it fails:** in MSP/P6 the Gantt workspace keeps a task table beside
  the chart; selecting a bar shows its properties. Here the primary editing
  surface (inspector) is unreachable while looking at the chart.
- **Fix:** hit-test bar rows on `response.clicked()` /
  `drag_started()` (row math already exists at `lib.rs:3663`) and call
  `self.select(id)`; draw a selection stroke around the selected bar in the
  row loop (`lib.rs:3747-3793`, compare `self.selected ==
  activity.id`); add the right-hand Inspector panel to the Gantt view too
  (same builder as `lib.rs:1744`).

### S2-7 · Gantt · Timeline header prints a full ISO date on every tick

- **Surface:** `lib.rs:3717-3736`.
- **What is wrong:** at ≥10 px/day (`tick_step == 1`) a 10-character
  `2026-11-02` label is drawn centred on *every* day at 10 px — labels
  collide into an unreadable strip. There is no month/week tiering
  (`docs/parity.md:236` records timescale tiers as "partial").
- **Why it fails:** the timescale is the most-seen surface in a Gantt;
  overlapping labels look unfinished on day one.
- **Fix:** two-tier header: month names in a top band, day-of-month (or
  week starts) below; skip labels that would collide
  (`if x - last_label_x < 40.0 { continue }`); abbreviate
  (`format!("{} {}", date.day(), month_short)`).

### S2-8 · Gantt · Milestones and short bars are invisible slivers with spilled labels

- **Surface:** `lib.rs:3755-3791`.
- **What is wrong:** `width = … .max(6.0)` renders a zero-duration milestone
  as a 6-px sliver; the name is centred on the bar (`lib.rs:3785-3791`) and
  spills over neighbours at small widths. No diamond glyph — the universal
  milestone convention in MSP/P6/Asta.
- **Fix:** when `activity.original_duration == 0`, draw a rotated square
  (diamond) at the milestone date in `tokens.critical`/accent with the label
  to the *right* of the point; skip the progress overlay for milestones.

### S2-9 · Inspector/Lookahead/Diary · Entities the engine can remove or update, the UI cannot

- **What is wrong:**
  - Relationships: the inspector has **"Add link"** (`lib.rs:1976`) and
    **"Apply lag"** (`lib.rs:1906`) but no way to delete a link.
  - Lookahead snapshots: **"Publish"** (`lib.rs:2879`) exists, but a
    published snapshot can never be removed from the UI although
    remove/restore commands ship (`docs/parity.md:70`).
  - Diary notes: **"Add note"** (`lib.rs:3075`) and per-row **"Delete"**
    (`lib.rs:3150`) exist, but a note can never be edited although update
    commands ship (`docs/parity.md:71`).
- **Why it fails:** create-without-delete/update is the classic CRUD
  half-implementation; paid apps close the loop on every entity they let you
  make.
- **Fix:** "Delete link" button beside "Apply lag" gated on
  `self.selected_relationship`; a small "Remove" button per snapshot row in
  `lib.rs:2916-2928`; clicking a note loads it into the top form with the
  button relabelled "Update note".

### S2-10 · Calendars · Holidays edited as one comma-separated text line; detail hides the working week

- **Labels:** "Holidays (comma-separated YYYY-MM-DD)" (`lib.rs:2620`),
  grid cell `format!("{}/7", …)` (`lib.rs:2665`).
- **What is wrong:** the exception list is a single text field; the detail
  panel (`lib.rs:2679-2691`) shows the id and holidays but *not* which
  weekdays are working — the one thing you most need to verify. Parse
  failures ("\"{trimmed}\" is not a valid date", `lib.rs:963`) surface in
  the status bar, not beside the field.
- **Why it fails:** P6/MSP edit exceptions as a dated list with per-row
  add/remove and always show the week pattern. A comma-joined ISO list is
  engineer-visible, not user-visible.
- **Fix:** render one row per holiday with a small remove button, plus an
  "Add holiday" date field (helper from S2-4); show the seven weekday
  checkboxes read-only in the detail panel; surface `build_calendar` errors
  inline.

### S2-11 · Resources · Dual-purpose create/edit panel, unfiltered all-activities assignment grid, no chart

- **Labels:** "Add resource" (`lib.rs:2278`), "Apply changes" (`lib.rs:2282`),
  "Delete resource" (`lib.rs:2285`), "Assignments" (`lib.rs:2409`), the
  instruction "Toggle '⟨name⟩' on an activity to assign or remove it."
  (`lib.rs:2411-2417`).
- **What is wrong:** one panel is simultaneously the create form and the
  edit form (same Name/Kind/Capacity fields serve both, button visibility is
  the only signal). The assignment grid (`lib.rs:2418-2439`) lists *every*
  activity with click-to-toggle — no filter, and a click both selects and
  silently toggles. Usage is a 160-px text grid (`lib.rs:2379-2402`); the
  histogram-as-chart polish is still open (`docs/qc/capability-audit.md:169`).
- **Why it fails:** dual-purpose forms train users to fear overwriting a
  selected resource while trying to add one; bulk assignment without
  filtering does not scale past a demo; a "usage histogram" that is a text
  table fails the reference-product bar outright.
- **Fix:** split the panel: read-only summary of the selected resource +
  explicit **"New resource"** button that clears the form into create mode;
  add a filter field above the assignment grid; draw per-day bars (a simple
  `painter.rect_filled` row per date, capacity line in `palette.grid`)
  above the usage grid.

### S2-12 · Resources · Leveling applies N permanent constraints with no composite undo and no confirmation

- **Labels:** "Preview leveling" (`lib.rs:2292`), "Apply leveled dates"
  (`lib.rs:2295`).
- **What is wrong:** `apply_leveling` (`lib.rs:1062-1094`) pushes one
  `SetActivityConstraint` per delayed activity. Undo requires N presses;
  removing the leveling afterwards means finding and clearing every SNET by
  hand. Nothing warns that Apply converts a *computed* level into *hard
  constraints*. (`docs/qc/gui-audit.md:49` records the composite-undo
  finding as open.)
- **Why it fails:** a destructive-ish bulk transformation with silent,
  hard-to-reverse effects is the definition of unsafe in a scheduling tool.
- **Fix:** wrap the batch in one composite command (as gui-audit item 8
  recommends); before applying, show a confirm dialog listing how many
  activities will receive constraints ("Leveling will add start-no-earlier-
  than constraints to N activities. Apply?"); offer a one-click
  "Remove leveling constraints" that clears exactly the constraints the
  batch added (tag them or store the id list).

### S2-13 · Audit · Findings have no colour semantics; relationship findings are not clickable; jumps abandon the view

- **Labels:** `[{word}]` where word ∈ "pass" / "FAIL" / "not applicable"
  (`lib.rs:2751-2758`), "Run audit" (`lib.rs:2712`).
- **What is wrong:** outcomes render as plain text — no green/red chips,
  although `palette.ok`/`palette.critical` exist. Activity findings are
  clickable (`lib.rs:2769-2777`) but relationship findings
  ("Relationship {id}: {detail}", `lib.rs:2780`) are not. Clicking a finding
  switches to Table view (`lib.rs:2774`), losing the audit you were reading.
  No "failures only" filter; no export from the view (JSON is CLI-only).
- **Why it fails:** the commercial audit tools this feature imitates are
  scorecards — colour-coded, with in-place highlighting. capability-audit
  row "Scorecard green/yellow/red" is already "partial" for exactly this.
- **Fix:** colour the outcome word with `palette.ok`/`palette.critical`/
  `text_muted`; make relationship findings clickable (select the
  relationship + jump); instead of switching views, open the Table view
  *with the split Gantt on and the finding highlighted*, or scroll-to and
  flash the row while returning to the Audit tab with a "back to findings"
  affordance; add a "Show failures only" toggle over the report loop.

### S2-14 · App shell · No import/export in the GUI; "Save report" writes to an opaque working directory

- **Labels:** "Save report" (`lib.rs:1601`, implementation `lib.rs:1214-1221`
  writes `"{}.report.md"` next to the *current working directory*), absence
  of any CSV/JSON/XML/MSPDI import-export entry point.
- **What is wrong:** interchange is CLI-only (`docs/parity.md:78-82`); a
  desktop user cannot get data in or out without a terminal. "Save report"
  silently writes to the process CWD — for a launched-from-shortcut desktop
  app, a directory the user cannot name.
- **Why it fails:** paid desktop apps own their file story: File ▸ Import…
  ▸ Export…, every write through a dialog or a shown path.
- **Fix:** give "Save report" the same `rfd::FileDialog` treatment as
  "Print report…" (`lib.rs:1122-1129`); add File-menu items **"Import…"
  (pick CSV/JSON/XML, route to `cadence-interchange::import_*`, merge or
  replace with a confirm dialog)** and **"Export data…"** with a format
  picker.

### S2-15 · Gantt · Drag interactions: the one power gesture is undiscoverable, off-palette, and drag-to-move is absent

- **Surface:** `lib.rs:3660-3697` (drag-to-link), `lib.rs:3681` (feedback
  line `Color32::YELLOW`).
- **What is wrong:** drag-to-create-link is mentioned only in inspector
  helper text ("This form is the keyboard path; dragging between Gantt bars
  is a shortcut.", `lib.rs:1922-1925`) and the F1 window; the in-progress
  link is drawn in raw yellow, near-invisible on the light theme. There is
  no cursor/hover affordance that bars are draggable, and dragging a bar to
  *move* work — the core Gantt interaction in MSP/P6/Asta — does not exist
  (the engine could support it via a `StartNoEarlierThan` SNET command).
- **Fix:** draw the pending link in `palette.warn` (visible on both themes);
  show a hover tooltip on bar ends ("Drag to another bar to link; Ctrl for
  Start-to-Start"); implement drag-to-move as bar-body drag →
  `SetActivityConstraint(StartNoEarlierThan(shifted_start))` with the shift
  previewed during drag, and modifier-drag for other link types (closes
  gui-audit open item 1).

### S2-16 · Table/Resources/Gantt · Schedule errors are a bare red sentence with no next step

- **Label:** "Schedule error: {error}" (`lib.rs:2020-2022`, `2314-2316`,
  `3572-3575`).
- **What is wrong:** when the engine rejects the schedule (cycle, violated
  constraint), three views print the engine's raw error string in
  `Color32::RED` and stop. No guidance, no way to see the offending
  activity/relationship, no "last good schedule" behind it.
- **Why it fails:** brand voice: "Errors say what to do next"
  (`docs/design/brand.md:76`). A cycle error without the cycle's members is
  undiagnosable in anything but a toy project.
- **Fix:** surface the structured error (`SchedError::Cycle` already carries
  a recovered path): render "Cannot schedule: circular link A → B → C."
  plus clickable activity links and a "Select the first link" action; keep
  the previous good schedule rendered dimmed instead of replacing the whole
  view with the error.

### S2-17 · Diary · The activity link is a raw numeric text field that fails silently

- **Labels:** "Activity id (optional)" (`lib.rs:3060-3064`), "Add note"
  (`lib.rs:3075`).
- **What is wrong:** the note's activity link is typed as a number.
  Non-numeric input parses to `None` and the note is created *silently
  unlinked* (`lib.rs:3077-3082`) — the gui-audit's open item 12. Worse, an
  out-of-range number (e.g. 999) is accepted and links to an activity that
  does not exist.
- **Heuristic:** error prevention (Nielsen #5).
- **Fix:** replace the text field with the same `ComboBox` pattern the
  Add-relationship form already uses (`lib.rs:1933-1943`), listing
  activities by name with id in parentheses; validate an entered id exists
  before `CreateDailyNote`.

### S2-18 · Shell-wide · Destructive actions have no confirmation and no undo affordance

- **Labels:** "Delete resource" (`lib.rs:2285-2288`), "Delete calendar"
  (`lib.rs:2630-2633`), "Delete baseline" (`lib.rs:2470-2475`), Diary
  "Delete" (`lib.rs:3150-3152`).
- **What is wrong:** every delete fires immediately. The commands are
  undoable via Ctrl+Z, but nothing tells the user that — the only feedback
  is a status message that, per S2-3, renders in red as if something went
  wrong ("Diary note deleted.", `lib.rs:3163`).
- **Why it fails:** paid apps either confirm destructive actions or pair
  them with a visible Undo. Cadence does neither consistently (baselines and
  calendars are project-critical artifacts).
- **Fix:** for baselines and calendars show a modal confirm naming the
  object ("Delete baseline 'Foundation' from 2026-09-01?"); for all four,
  add an **"Undo"** button to the success status message that calls
  `self.stack.undo(...)` + `refresh_selection()`.

### S2-19 · App shell · Production has no keyboard shortcut and the F1 map is wrong

- **Surface:** `lib.rs:1382-1391` (key array `Num1..=Num8`),
  `lib.rs:1397-1411` (match with an unreachable `8 => ViewMode::Production`
  arm), `lib.rs:3283-3286` (F1 text "Ctrl+1 … Ctrl+8" listing eight views).
- **What is wrong:** the array holds eight keys, so index 8 never occurs:
  Ctrl+9 does nothing, Production is mouse-only, and the Shortcuts dialog
  documents a nine-view app as eight. (`docs/qc/gui-audit.md:62` already
  records the ACCESSIBILITY.md drift for Ctrl+1–8.)
- **Fix:** add `egui::Key::Num9` to the array, keep the match arm, and
  update the F1 row to "Ctrl+1 … Ctrl+9 — Table, Gantt, Resources,
  Baselines, Calendars, Audit, Lookahead, Diary, Production".

### S2-20 · Lookahead vs Baselines · The same concept (variance) is rendered two different ways

- **Surface:** Baselines variance grid — coloured signed numbers via
  `variance_color`/`variance_text` (`lib.rs:2576-2595`); Lookahead compare —
  one label `"start {:+}d / finish {:+}d"` (`lib.rs:2961-2969`), uncoloured,
  untabular.
- **Why it fails:** consistency (Nielsen #4): users learn "red = later than
  promised" in Baselines, then Lookahead shows the same fact as grey text in
  a sentence. The Lookahead compare section is also a plain sequence of
  `ui.horizontal` rows, not a grid, so values do not align.
- **Fix:** reuse `variance_color` + `variance_text` in the snapshot compare
  and put rows in a `Grid` with columns Activity / Start Δ / Finish Δ /
  Status ("added" / "removed" / name) matching the Baselines grid's header
  style.

---

## S3 — Minor

### S3-1 · Shell · Three of four inspectors are fixed-width; only the embedded Gantt resizes
`lib.rs:1744-1746, 2257-2259, 2447-2449, 2605-2607` use `.default_size(...)`
without `.resizable(true)` while the workbench Gantt has it
(`lib.rs:1991`). Add `.resizable(true)` to all four.

### S3-2 · Inspector · Internal ids surface as user-facing labels
"Activity {id}" header (`lib.rs:1752`), "rel {id}" (`lib.rs:1867`),
"{pred} -> {succ}" (`lib.rs:1878`), "Added {type} link {pred} → {succ}"
(`lib.rs:570-573`). Show "Activity 5 — Survey" and relationship rows as
"Survey → Excavate (FS +2d)"; messages should name activities, not ids.

### S3-3 · Table · The `#` column shows row position while the app's real identifiers appear elsewhere
`lib.rs:2130-2135` prints `index + 1`; but Diary and clash links reference
*activity ids* (e.g. `lib.rs:3145`, `lib.rs:3020`). Either show the
activity id in `#` or label the column "Row".

### S3-4 · Table/Gantt · Criticality marked "Yes"/"No" in the table but "! name" on bars
`lib.rs:2200-2208` vs `lib.rs:3780-3784`. Render the Critical cell with
`palette.critical` (and/or a "!") so the two surfaces share one semantic.

### S3-5 · Monospace token applied inconsistently
Table dates/floats are monospace (`lib.rs:2172-2196`) but Lookahead dates
(`lib.rs:2857-2858`), calendar holiday list (`lib.rs:2688`), clash dates
(`lib.rs:3018`) and EVM money (`lib.rs:2512-2537`) are proportional —
violating "Data is monospaced" (`docs/design/brand.md:25`). Add
`.monospace()` (or `RichText::monospace`) at those sites.

### S3-6 · Off-palette colours throughout
`Color32::RED` (`lib.rs:2021, 2315, 2503`), `YELLOW` drag line
(`lib.rs:3681`), `GRAY` ticks/arrows/zero-variance (`lib.rs:3407, 3724,
3733, 3851`), `WHITE` bar text (`lib.rs:3790`). GRAY at 10 px on the light
panel is below AA. Replace with `palette` tokens; the gui-audit's open item
5 (`docs/qc/gui-audit.md:41`) covers the reds — this extends it to all
literal colours.

### S3-7 · Shell · Controls enabled in views where they do nothing
"Split Gantt: on/off" (`lib.rs:1571-1581`) is global but only rendered in
Table view; "Print Gantt…" is disabled outside Gantt with no explanation
(`lib.rs:1614-1618`). Move Split Gantt + Print Gantt into their views'
local toolbars, or add `.on_disabled_hover_text("Available in the Gantt
view")`.

### S3-8 · Shell · Duplicate controls and headings
"Toggle theme" twice (`lib.rs:1671, 1733`); heading "Earned value" in both
the inspector (`lib.rs:2478`) and the centre panel (`lib.rs:2499`). Keep one
of each; the inspector copy can become the "Status date" field group only.

### S3-9 · Status bar · Success messages never clear
Stale "Saved to …" lingers indefinitely (gui-audit open item 13,
`lib.rs:544` region). Covered by the S2-3 rework.

### S3-10 · Shell · No busy state for work that blocks the UI thread
`run_audit` runs inline when the view opens (`lib.rs:2698-2701`),
`open_from` loads synchronously (`lib.rs:1194-1212`). On a 10,000-activity
programme both freeze the window with no spinner. Minimum: show
"Running audit…" via `ctx.request_repaint_after` + a deferred compute
channel like the update check (`lib.rs:386-396`).

### S3-11 · Table · Truncated names are unrecoverable
The name button uses `.truncate()` (`lib.rs:2140-2148`) with no
`.on_hover_text(&activity.name)`; the full name is inaccessible. Add the
hover tooltip.

### S3-12 · Gantt · Weekly grid lines drift off week boundaries
`lib.rs:3632-3648` advances by five *working* days from the project
calendar, so a Monday holiday moves the "weekly" line to Tuesday — and the
`unwrap_or(week_origin)` can spin forever on a zero-working-day calendar
(gui-audit open item 17). Advance by calendar weeks
(`date.weekday().num_days_from_sunday()`) instead.

### S3-13 · Gantt · Dependency arrows have no arrowheads
`lib.rs:3851-3854` draws three line segments; direction is ambiguous at a
glance. Add a small triangle head at `(x1, y1)`.

### S3-14 · Gantt · Zoom feels one frame late and Ctrl+wheel also scrolls
Zoom writes `pixels_per_day_out` mid-render while the frame already read the
old value (`lib.rs:2238`, gui-audit item 11); Ctrl+wheel does not consume
the scroll, so the canvas zooms *and* pans (`lib.rs:3700-3706`, gui-audit
item 9). Apply the new zoom before drawing and zero the consumed scroll.

### S3-15 · Shell · Theme and window geometry do not persist
`dark: true` hard-coded (`lib.rs:311`, `lib.rs:3896`); `eframe::NativeOptions::default()`
(`lib.rs:3890`) with no size or position persistence; no saved layouts for
columns/sort (`visible_columns`, `sort_key` die with the process). Persist
via eframe storage (`save`) and apply on start.

### S3-16 · Shell · No recent files, no drag-and-drop open
Ctrl+O/Open… only. Paid desktop apps offer File ▸ Open Recent and accept a
dropped `.cadence` file (`ctx.input(|i| i.raw.dropped_files.clone())`).

### S3-17 · Gantt · Accent colour overloaded
The accent is selection/links/focus (`docs/design/brand.md:38`) but also
draws the today line (`lib.rs:3656`) and the progress overlay
(`lib.rs:3776`). Use `palette.text_muted`/a dedicated today token for the
line and `palette.ok` for progress so accent stays selection-only.

### S3-18 · Baselines · Grid headers are not visually distinct from data
Variance headers "Activity / Start / Base start / ΔStart …" are plain
labels inside the striped grid (`lib.rs:2559-2566`); rows under them read
as one undifferentiated list. Render headers with `RichText::strong()` and
a bottom hairline.

### S3-19 · Copy · "activity(ies)" and id-speak
"{} activity(ies) use this calendar — clear their calendar first"
(`lib.rs:928-936`). Use "3 activities use this calendar. Set their calendar
to something else first." (brand voice: plain sentences).

### S3-20 · Crash dialog · The log is cleared before the browser is confirmed
"Open GitHub issue" calls `crash::clear_log()` immediately after
`open_url` (`lib.rs:3199-3206`); if the browser fails to open, the report
is gone (gui-audit item 14). Clear only after an explicit "Dismiss", and
label the button "Copy details and open issue".

### S3-21 · Formatting · Money and percentages lose their units
`format_money` (`lib.rs:3412-3415`) renders "1000.00" — no currency, no
thousands separators; `select()` writes percent as a bare "33.33"
(`lib.rs:452-455`) with no "%". Suffix money with a currency label (project
setting, default per locale) and percent with "%".

### S3-22 · Accessibility · Text fields are not label-associated
`ui.label("Name")` + `text_edit_singleline` pairs throughout the inspector
(`lib.rs:1750-1978`) are not linked via `labelled_by` (gui-audit item 6).
Wire each label's `Response` to its field.

### S3-23 · Lookahead · Window controls lack quick presets
"Window starts (YYYY-MM-DD)" + "2 w / 4 w / 6 w / 12 w" (`lib.rs:2800-2809`):
no "This week / Next week" buttons; the active length is only indicated by
selection styling and its label "2 w" is terse. Add preset buttons and
label the control "Window length".

### S3-24 · Lookahead/Diary · Long undifferentiated scrolls
The Lookahead view is one continuous column of seven sections
(`lib.rs:2789-3045`) with no structure; the Diary likewise. Add
collapsing sections (`egui::CollapsingHeader`) or a two-column layout, and
group diary notes under date headings (the sort at `lib.rs:3135` already
orders them for it).

---

## Nielsen heuristic map

| # | Heuristic | Strongest violations |
|---|---|---|
| 1 | Visibility of system status | S2-3 (success in red), S3-9 (stale status), S3-10 (no busy state), S1-3 (no dirty marker) |
| 2 | Match system & real world | S2-8 (milestone slivers, no diamonds), S3-21 (unit-less money/percent), S3-2 (id-speak) |
| 3 | User control & freedom | S2-12 (leveling without composite undo), S2-18 (no undo affordance), S2-9 (undeletable links/snapshots) |
| 4 | Consistency & standards | S2-20, S2-11 (dual-purpose forms), S3-4, S3-5, S2-1 (one toolbar for everything) |
| 5 | Error prevention | S2-17 (silent unlinked note), S2-12 (irreversible-feeling leveling), S2-4 (free-text dates) |
| 6 | Recognition over recall | S2-17 (type an id), S2-15 (hidden drag gesture), S2-4 (format memorised, not shown) |
| 7 | Flexibility & efficiency | S2-5 (no filter/multi-select), S2-19 (dead shortcut), S3-16 (no recent files) |
| 8 | Aesthetic & minimalist | S2-7 (overlapping ticks), S2-1, S3-24, S3-8 |
| 9 | Recover from errors | S2-16 (bare engine error), S2-4 (errors far from input), S2-3 |
| 10 | Help & documentation | S2-19 (F1 wrong), S2-15 (undiscoverable drag), S3-7 (silent disabled controls) |

## Feature-level affordances absent from the UI where the engine/commands already support them

| Capability | Engine/commands | UI today | Fix surface |
|---|---|---|---|
| Create / delete activities | `CreateActivity`, `DeleteActivity` | none | S1-1 |
| Delete relationships | delete command in `cadence-commands` | none | S2-9 |
| Remove published lookahead snapshots | remove/restore commands (`docs/parity.md:70`) | none | S2-9 |
| Edit daily notes | update commands (`docs/parity.md:71`) | delete only | S2-9 |
| WBS tree & assignment | `wbs.rs`, `CreateWbsNode`/`MoveWbsNode`/`AssignActivityWbs` | no view at all (`docs/qc/capability-audit.md` gap #2) | new view |
| Baseline bars on the Gantt | baseline snapshots in model | progress fill only (`docs/qc/capability-audit.md:203`) | Gantt draw pass |
| Status-date line | status date is editable (`lib.rs:2479`) | today line only (`lib.rs:3650-3658`) | Gantt draw pass |
| Audit thresholds | `AuditConfig` (`audit.rs:60`) | hard-coded defaults | Audit view form |
| Histogram chart | `resource_histogram` (`engine.rs:231`) | text grid | Resources view |
| Drag-to-move bars | SNET constraint command | drag creates links only | S2-15 |
| Import/export from the GUI | full interchange crate | CLI only | S2-14 |
| Driving-logic / Task Inspector | relationship graph in engine | absent (`docs/qc/capability-audit.md:107`) | inspector pane (post-v1) |

## What already meets the bar

Credit where due, so the fixes preserve it: the token system and WCAG AA
contrast tests (`design.rs:189-198`, `lib.rs:3903-3980`); the restrained
brand voice in empty states ("Nothing is scheduled in this window yet; move
the window or add work in the table.", `lib.rs:2834-2837`; "Nothing has
been sent anywhere.", `lib.rs:3184-3186`); the sorted/virtualised table with
sticky headers and sort-direction arrows (`lib.rs:2052-2098`); the status
bar's schedule facts (`lib.rs:1690-1740`); the keyboard-alt-path thinking in
the inspector (`lib.rs:1922-1925`); and the Baselines variance grid's
colour semantics — which the Lookahead compare should copy, not the reverse.
