# Full visual & functional audit — Cadence v0.12.1

Date: 2026-11-07 · Version: 0.12.1 · Commit: `db95ad4` (main)
Scope: every user-facing visual or manual surface of the desktop GUI
(`crates/ui-egui`), benchmarked against the inspiration products.
Method: read-only code audit by a coordinated team (guide §12.2) — one
specialist inventoried every surface (`lib.rs`, `design.rs`, `crash.rs`,
`update_check.rs`, `main.rs`), one audited functional defects, and the
supervisor reconciled against the approved inspiration research
(`docs/research/ms-project-capabilities.md`,
`docs/research/primavera-p6-capabilities.md`), `docs/ui-patterns.md`,
`docs/design/brand.md`, and the prior 2026-11-01 audit set.

This supersedes the 2026-11-01 audit where they disagree. Several 2026-11-01
items are now **stale** — see §1.

---

## 1. What actually changed since 2026-11-01 (honest reconciliation)

The 2026-11-01 audit (`docs/audit/2026-11-01-remediation-plan.md`) concluded the
app "does not yet look and feel like MS Project / P6" and named three dominant
reasons. Re-checked against the current tree:

| 2026-11-01 claim | Status now | Evidence |
|---|---|---|
| "The GUI cannot create or delete an activity" | **Fixed** | New activity (Ctrl+N), row context menu, Delete activity/selection with confirm — `lib.rs:2228`, `2557-2562`, `4563-4619` |
| "Production view is a heading plus a sentence" | **Fixed** | Full record/delete/reschedule-from-progress view — `lib.rs:4404-4557` |
| "No unsaved-changes guard" | **Fixed (with a hole)** | Title `•`, close guard `lib.rs:1893-1896`, Save/Discard/Cancel modal; but **Open** bypasses it — F1 below |
| "No menu bar; one wrapping toolbar of ~21 text buttons" | **Fixed** | Three-zone shell: File/Edit/View/Window/Help menus + tab strip + tool cluster — `lib.rs:2144-2441` |
| "Every status message renders in error red" | **Fixed** | `StatusSeverity` with ok/warn/critical/info — `lib.rs:158-216`, `2505-2516` |
| "Gantt is a flat diagram; no milestones/summary bars/baselines/arrowheads" | **Mostly fixed** | Summary bars, WBS indentation, milestones, baseline ghost bars, arrowheads, lag badges all present — `lib.rs:5446-5603` |
| "Adopt egui_extras / egui-phosphor / egui-toast / egui_dock" (Pillar D) | **Not done** | `crates/ui-egui/Cargo.toml` still depends only on `eframe`, `egui`, `rfd`, `serde`, `time`, `ureq` |
| "Filter/search/group in the table" | **Partial** | Filter + Critical-only landed (`lib.rs:2885-2902`); grouping not built |

Three items the older `docs/design/ui-polish-audit.md` records as **Done** are
**not done** in this tree — F4 (diary parse errors), F19 (AccessKit
`labelled_by` coverage), F21 (split-Gantt resize hint). A "Done" tick that does
not survive re-reading the code is a process defect, not just a doc defect.

---

## 2. Verdict

**Functionally, Cadence is now a real authoring tool** (create/edit/delete
activities and links, baselines, calendars, resources, audit, lookahead, diary,
production). The engine is at parity with the reference products. The earlier
"it is only a viewer" verdict no longer holds.

**Visually, it still reads as engineer-built rather than studio-polished**, for
three reasons that dominate:

1. **No iconography and no bought widget layer.** The toolbar, tabs and menus
   are text-only (`brand.md:71` chose this for v1), tables are hand-rolled, and
   date entry is raw `YYYY-MM-DD` text. Paid desktop tools lean on icons,
   `DatePicker`s, striped/resizable grids and toasts. The 2026-11-01 Pillar D
   (permissively-licensed `egui-phosphor`, `egui_extras`, `egui-toast`) was
   never adopted.
2. **The Gantt is missing the single most recognisable professional texture** —
   non-working (weekend/holiday) shading — plus a two-tier timescale and
   expand/collapse outline. Everything else a planner looks for is now present.
3. **A data-loss blocker sits in the most common file action.** Opening a file
   silently discards unsaved edits (F1). One bug like this undoes a lot of
   polish credibility.

The foundations remain good (token palette, Inter, WCAG-checked accent, strict
type scale). This is a **composition + completeness + affor­dance** problem, not
a rewrite.

---

## 3. Visual audit — "how it looks"

Each item: what the user sees, vs. the reference class. `[R]` = raised in the
2026-11-01 set and still true unless marked.

### 3.1 Shell (top bar, tabs, status bar)

- **Menu bar + tab strip + tool cluster** — present and a large improvement.
  Still all-text; menu items are wide. Reference: Project-class apps pair a
  command bar with icons. — V1
- **Status bar (V2 / F14).** A single non-wrapping row (`lib.rs:2449-2540`):
  project range + activity count + critical count + full EVM string + status
  message + save state + zoom % + Shortcuts button. At the 1000 px minimum it
  clips the right-hand cluster. Fix: `horizontal_wrapped` (the toolbar already
  wraps) and drop EVM from the bar at narrow widths.
- **Bare "143%" zoom (V2).** The zoom figure has no tooltip saying it is Gantt
  zoom (`lib.rs:2525-2529`); it also disagrees with the Gantt's own % label.
- **Release-notes button (V3 / F18).** The version button and dialog are fine in
  shape, but the dialog carries a run of ~30 spaces (`lib.rs:4886`), stale
  hard-coded token metrics ("release cycle (0.12.0): ~45K") that contradict the
  running 0.12.1 and the `docs/tokens.json` ledger (`lib.rs:4911-4916`), and
  claims a "zoom slider in the status bar" that does not exist
  (`lib.rs:4900-4901`).
- **Theme & recent files are not persisted (V4 / F18).** Both live in memory
  (`lib.rs:374, 557-559`), so the app forgets the chosen theme and the recent-
  files list on every restart — but the release notes say it "remembers the ten
  most recent projects" (`lib.rs:4905-4907`). Unpolished and untrue.

### 3.2 Gantt

- **Present and credible now:** normal bars, summary bars, WBS indentation
  (`lib.rs:5594-5603`), milestones (diamond), critical colouring, progress fill,
  baseline ghost bars (`lib.rs:5446-5466`), dependency arrows **with heads**
  (`lib.rs:5519-5527`), signed lag badges (`lib.rs:5533-5540`), a 2 px today
  line with a top marker, weekly hairlines, per-link geometry.
- **Missing: non-working shading (V5).** No weekend/holiday columns anywhere
  (grep: zero `weekend`/`non_working` in the crate). This is the texture that
  makes a Gantt read as "P6/Project" at a glance, and it also fixes the
  working-day weekly hairlines that drift across holidays.
- **Missing: two-tier timescale (V6).** One header band with ISO date strings;
  no months-over-weeks, quarters/years on zoom-out (grep: zero `quarter`,
  `timescale`).
- **Missing: expand/collapse outline (V7).** WBS depth indents the labels, but
  there are no +/- toggles and no grouping (grep: zero `collapse`/`expand`/
  `outline`).
- **Missing: progress lines, a Gantt status-date line, constraint/deadline
  indicators, drag-to-pan (V8).** Constraints and a status date are modelled and
  edited elsewhere but never drawn on the canvas; the only canvas drag is
  drag-to-link.
- **Non-token colours (V9 / F12).** `Color32::YELLOW` drag-preview line
  (`lib.rs:5339`), `GRAY` arrows/ticks (`5396-5429, 5699-5714`), `WHITE`
  milestone outline and summary end-caps (`5557-5574`) — near-invisible on the
  light theme's white panel; `variance_color` literals (`5030-5038`). Violates
  `brand.md` ("colour is reserved for meaning") and the token discipline.

### 3.3 Table, inspector, other grids

- **Table** is virtualised with sortable headers, a filter bar, multi-select,
  context menu, empty states. Solid.
- **Monospace rule not applied outside the table (V10 / F20).** `brand.md:54`
  says dates/floats/counts are monospaced; the Resources, Baselines, Calendars
  and Diary grids render them in the proportional face
  (`lib.rs:3420-3484, 3671-3752, 3822-3830, 4344`).
- **Assignments grid has no header row and a duplicated cell (V11).**
  `lib.rs:3503-3515`: four trailing numeric columns are unlabelled, and
  `original_duration` is drawn twice in a row.
- **Date entry is raw text (V12).** Every date is a `YYYY-MM-DD` text field with
  inline validation, not a calendar picker. Reference class uses pickers.
- **Dialogs (V13 / F13).** Four confirm windows, plus crash/update/shortcuts/
  release-notes. None closes on **Esc**; Enter does not press the primary
  button; and while a modal is open the global shortcuts still fire behind it
  (Ctrl+Z/S/O/N/D, Ctrl+1-9 — `lib.rs:1933-1988` run before `dialog_open`
  `lib.rs:2039-2040`). Worst case described in F13.
- **Terminology drift (V14 / F17).** "Relationships" vs "dependency link" vs
  "link"; raw basis points "5000 bp" where the user typed "50"; relationship
  picker shows `1 -> 2` while the add-link form shows names; "New activity
  below" appends at the end; "Add to selection" actually toggles.

---

## 4. Functional audit — "how it works" (severity-ranked)

Findings are verified against the current tree (commit `db95ad4`). IDs `F#` are
stable for follow-up. Line refs are `crates/ui-egui/src/lib.rs` unless noted.

### S1 — blocker

- **F1. Opening a file silently discards unsaved changes.** `open_from`
  (`1664-1684`) never checks `self.dirty`; entry points Ctrl+O (`1943-1945`),
  File ▸ Open… (`2154-2156`), File ▸ Recent files (`2164-2175`). Unsaved edits
  are replaced and `dirty` is cleared, with only a green "Opened …" toast. The
  close guard exists but Open bypasses it. *Fix:* stash `pending_open` and reuse
  the Save/Discard/Cancel modal.

### S2 — major

- **F2. Assignment units field is uneditable.** The buffer is overwritten from
  the model every frame (`3564-3574` behind the editor `3519-3541`), so typed
  input is destroyed on the next repaint and `SetAssignmentUnits` can never fire
  with a new value. *Fix:* resync only when not focused and the model changed.
- **F3. Production ▸ Record fails silently on rejected commands.** `if
  execute(...).is_ok() { … }` with no `else` (`4433-4462`); out-of-range percent
  or a stale activity id does nothing visible. *Fix:* `Err(error) => set_status`.
- **F4. Diary: non-numeric id silently detaches the note; wrong error for an
  unknown id.** `4273-4278` maps `abc` to `None`; a valid-but-unknown id gets
  "Cannot add the note; check the content." (`4300-4302`) pointing at the wrong
  field. *(Recorded "Done" in ui-polish-audit as S2-10 — it is not.)*
- **F5. Open leaves stale, destructive selection/state.** `open_from`
  (`1664-1684`) does not clear `multi_selected` (`717-718`); ids restart at 1 per
  project, so project-A rows stay highlighted in project-B and the inspector
  offers "Delete N selected" (`2553-2560`) the user never chose. `editing_note`,
  `leveling_report`, `reschedule_preview`, diary buffers survive too.

### S3 — minor (polish / consistency)

- **F6/F7. Per-frame clone/recompute on hot paths.** Whole-programme clone in
  the default Table view (`2846`); Resources (`3388-3389`), Baselines
  (`3604-3653`), Lookahead (`4197`), Diary (`4315-4329`), Calendars
  (`2606, 3797`), crash log (`4731`). *Fix:* the `edit_version`-keyed
  `Arc`-snapshot cache already used for `conflicts_cache`.
- **F8. "Show in Gantt" doesn't scroll to the bar** (`3230-3234`); the flag also
  leaks and scrolls the table later (`3247`).
- **F9. Selecting a row rewrites the link form** (`827-830`).
- **F10. Silent command errors** in apply-lag (`2763-2774`), relationship retype
  (`952`), calendar assignment (`2630-2652`), delete-selection (`927`), confirm
  deletes (`4576, 4662`), note/production delete (`4395, 4483`).
- **F11. Resource editor silently skips invalid input** (`1206-1230`), and
  `create_resource` coerces a bad capacity to 1 (`1258-1281`).
- **F12. Non-token colours** — see V9.
- **F13. Dialog keyboard behaviour** — see V13.
- **F14. Status bar overflow + unexplained zoom** — see V2.
- **F15. "Update now" is a mislabel** (`4781-4788`): identical to "Open release
  page"; nothing is downloaded.
- **F16. Crash dialog:** misleading empty-state text (`4722-4725`); whole log
  percent-encoded into the issue URL (`crash.rs:82-88`); log cleared before the
  browser is confirmed (`4738-4745`).
- **F17. Terminology/label drift** — see V14.
- **F18. Release-notes content bugs** — see V3/V4.
- **F19. AccessKit `labelled_by` missing on most fields** (constraint, calendar,
  add-link, resource editor, baseline name, calendar editor, diary fields,
  production percent, reschedule date, assignment units —
  `2584, 2619, 2792, 3337, 3597, 3766, 4250, 4431, 4497, 3525`). *(Recorded
  "Done" as S2-20 — it is not.)*
- **F20. Monospace rule not applied outside the table** — see V10.
- **F21. Split-Gantt toggle has no resize hint** (`2262-2269`). *(Recorded
  "Done" as S5 — it is not.)* The toggle also only affects the Table view
  (`2261, 2845-2880`), so it appears dead elsewhere.
- **F22. Deletes without confirmation:** resource (`3363`), baseline (`3624`),
  diary note (`4358`), production entry (`4473`).
- **F23. Gantt week-grid loop hardening** (`5291-5294`): `unwrap_or(week_origin)`
  could spin forever; currently unreachable (calendars reject zero working
  days), so hardening only.
- **F24. Update checks:** network call on the first frame of every launch with
  no opt-out (`1915-1931`); the 24 h timer clobbers an in-flight manual check
  (`674-684`).
- **F25. Dead code / stale doc:** unused `production_percent` (`471, 638`),
  unused `_pixels_per_day` (`5194`), and `save_or_prompt`'s doc comment
  describes the report writer (`1609-1619`).

---

## 5. Benchmark vs the inspiration products

`docs/ui-patterns.md` is the behavioural matrix; `docs/parity.md` is the
capability tracker. Reconciled against the current tree:

| Behaviour | MS Project / P6 | Cadence now | Gap |
|---|---|---|---|
| Combined table + Gantt split | Split with draggable divider, synced | Adopted (draggable divider); **scroll sync still open** | minor |
| View tabs | Named view tabs | Adopted (9 tabs) | — |
| Column sort/filter/group | Sort + filter + group above the sheet | Sort + filter adopted; **group missing** | minor |
| WBS outline + summary bars | Outline +/- with summary bars | Summary bars + indent adopted; **no expand/collapse** | medium |
| Two-tier timescale | Month-over-week, quarter/year | **Missing** | medium |
| Non-working shading | Grey weekend/holiday columns | **Missing** | high |
| Critical path | Red bars + legend | Adopted (colour + `!`); **no legend** | minor |
| Drag to create links | Drag bar-to-bar | Adopted (with type picker) | — |
| Drag to move/resize bars | Drag bar/edge to reschedule | **Missing** | high |
| Milestones | Diamonds, labelled | Adopted | — |
| Baseline overlay | Tracking Gantt ghosts | Adopted (ghost bars) | — |
| Progress line / status date on chart | Stepped progress line | **Missing** | medium |
| Constraint/deadline indicators | Icons at constrained edge | **Missing** | minor |
| Inspector pane | Right-hand contextual properties | Adopted | — |
| Menu/command bar + icons | Icons in command bar | Menus adopted; **text-only** | medium |
| Yes/No dialogs, Esc cancels | Standard | **Esc does not cancel** (F13) | high |
| Data pickers | Date pickers | **Raw text dates** | medium |
| Usage views | Task/Resource Usage grids | Resource usage adopted; Task Usage not | minor |

---

## 6. Cross-cutting gaps

- **No visual-evidence pipeline (V15).** There is no `egui_kittest`/golden-render
  job and not one PNG in the repo; `.github/workflows/` has no screenshot step.
  Guide II.27.12 requires visual evidence, and §12.1 requires README screenshots
  at v1.0 and every major release. Every visual claim in every audit to date
  rests on reading render code, not on a rendered image. This is the root cause
  of "we keep auditing looks without ever seeing them".
- **Widget layer unchanged (V16).** `ui-egui` depends only on `egui`/`eframe`.
  The approved, permissive uplift (`egui-phosphor` icons, `egui_extras` tables +
  DatePicker, `egui-toast`) is documented in
  `docs/audit/2026-11-01-design-uplift-research.md` but not adopted. egui is
  pinned at 0.35; the ecosystem has moved to 0.37, so the 0.35-compatible
  versions must be used until a migration slice.
- **Preferences not persisted (V4).** Theme and recent files are in-memory.

---

## 7. Remediation plan (proposed slices)

Each slice = branch → implement → `ci.sh run` → CI green → merge. Ordered by
user-visible harm.

**P0 — correctness (bugs a user hits immediately)**
1. F1 Open discards unsaved work — route through the Save/Discard/Cancel modal.
2. F5 Clear selection/transient state on Open.
3. F2 Make the assignment-units field editable.
4. F3/F4/F10/F11 Surface every rejected command instead of failing silently.

**P1 — dialog & feedback polish**
5. F13 Esc cancels dialogs, Enter accepts, gate shortcuts behind modals.
6. F22 Confirm all destructive deletes.
7. F17 One term per concept; human units; honest verbs.
8. F15/F18 Fix "Update now", release-notes spacing, stale metrics, zoom-slider
   claim; drive notes from `docs/tokens.json`.
9. F16 Crash-dialog copy, URL-size cap, clear-after-open.

**P2 — visual uplift (the "looks" pillars)**
10. V9 Token colours everywhere on the canvas and grids.
11. V10 Monospace in every numeric grid; V11 header row + fix duplicate cell.
12. V5 Non-working shading derived from the assigned calendar.
13. V6 Two-tier timescale + V7 expand/collapse outline in table and chart.
14. V1 Adopt `egui-phosphor` icons (MIT) for toolbar/tabs/inspector.
15. V12 `egui_extras` `DatePicker`; adopt `egui_extras` tables + `egui-toast`.
16. V2 Status-bar wrap; tooltip the zoom.

**P3 — depth & persistence**
17. V4 Persist theme + recent files.
18. F6/F7 Cache per-frame recomputations.
19. F8 Show-in-Gantt scroll; F9 decouple selection from the link form.
20. Progress line, Gantt status-date line, constraint indicators, drag-to-pan.

**P4 — infrastructure (enables honest visual review)**
21. V15 Headless render golden tests in CI + README screenshots (guide
    §12.1, II.27.12).
22. Consider egui 0.35 → 0.37 migration to unlock current ecosystem crates.

---

## 8. What could not be verified without a display

- Any actual pixel rendering: theme parity of the `WHITE`/`GRAY`/`YELLOW`
  literals, status-bar clipping at exactly 1000 px, today-line marker, lag
  badges, drag-line visibility.
- Native `rfd` dialogs and the screenshot-dependent **Print Gantt…** path
  (`2105-2138`) — if no `Screenshot` event arrives the button repaints forever.
- Screen-reader behaviour of the F19 gaps (needs NVDA/Orca).
- Perceived performance of F6/F7 on a genuine 10k-activity programme; drag
  feel and hit tolerance.
- Whether the crash issue URL is rejected at realistic log sizes.

Adding the P4 render-golden job converts most of these into CI-verified facts.

---

## 9. Records

- Behavioural matrix: `docs/ui-patterns.md`
- Capability status: `docs/parity.md`
- Inspiration research: `docs/research/ms-project-capabilities.md`,
  `docs/research/primavera-p6-capabilities.md`
- Design source of truth: `docs/design/brand.md`
- Prior audits: `docs/audit/2026-11-01-*.md`, `docs/qc/gui-audit.md`,
  `docs/design/ui-polish-audit.md`
- Clean-room: only public, documented behaviour and URLs were used; no
  proprietary code, UI, strings, icons or algorithms.
