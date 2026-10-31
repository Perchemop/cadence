# GUI audit — egui desktop shell

Date: 2026-10-30. Scope: `crates/ui-egui/src/{lib.rs,design.rs,crash.rs,update_check.rs}`
and `crates/app/src/main.rs` (the GUI launch path). Method: exhaustive read of the
source, cross-checked against `docs/design/brand.md`, `docs/ui-patterns.md` and
`ACCESSIBILITY.md`. Read-only; fixes are listed separately with their commit.

Severity: **CRITICAL** = data loss / crash / unusable; **HIGH** = broken feature
or wrong result; **MEDIUM** = UX, accessibility or consistency bug; **LOW/NIT** = polish.

## Fixed in this audit

| Sev | Location | Defect | Fix |
|---|---|---|---|
| CRITICAL | `lib.rs` (whole file), `main.rs:557` | The GUI had **no save or open**. `gui_file`/`gui_demo` loaded a project and enabled editing, but no `save_project`/`load_project` call existed in `ui-egui`: every edit was discarded on close with no dirty prompt. | Added `project_path`, `save_or_prompt`/`save_to`/`save_as_dialog`/`open_dialog`/`open_from`, toolbar **Open… / Save / Save as…**, Ctrl+S/Ctrl+O, and `run_with_path`; `gui_file` passes the path. |
| CRITICAL | `lib.rs` `apply_edits` | `parse_money`/`parse_date` return `None` for **both** empty and unparseable text, and `apply_edits` treated `None` as "clear", so typing `abc` into Budget / Actual cost / Actual start / Actual finish silently wiped the stored value. | Distinguish empty (clear) from invalid (report + abort) via `parse_optional_money`/`parse_optional_date`; every rejected command now sets `status_message` instead of being dropped by `let _ =`. |
| HIGH | `lib.rs` `apply_calendar_edits` | Calendar edits executed `SetCalendarDefinition` but never rescheduled, leaving stale dates everywhere the calendar is used. | Recompute when a rename or definition actually changed. |
| HIGH | `lib.rs` undo/redo | Ctrl+Z/Ctrl+Y fired while a text field was focused (egui's `TextEdit` also handles them → double undo). | Guard on `!ctx.text_edit_focused()`. |
| HIGH | `lib.rs` undo/redo | Undo/redo left the inspector buffers stale; an immediate Apply re-created the undone edit as a new command. | Added `refresh_selection()` to reload the inspector from the model after undo/redo. |
| HIGH | `lib.rs` status bar | The bottom panel was added **after** the per-view `CentralPanel`, which already claimed the space, so the footer rendered collapsed. | Moved the status bar before the view match. |
| HIGH | `lib.rs` `render_gantt` | A drag-created link set `*schedule` directly and never bumped `edit_version`, so cached sort (Start/Finish/Float/Critical) and the Audit view stayed stale. | `render_gantt` now returns whether it changed the schedule; both call sites call `recompute()`. |
| HIGH | `lib.rs` `apply_field_planning` | Always pushed two commands even when nothing changed, and ignored errors. | Only push on change; report errors. |
| MEDIUM | `lib.rs` toolbar | The toolbar was a non-wrapping `ui.horizontal`, so controls fell off-screen on a narrow window. | `horizontal_wrapped`. |
| MEDIUM | `lib.rs` diary | A non-numeric activity id silently attached the note to no activity. | Still open (see below) — noted, not yet fixed. |
| HIGH | `lib.rs` `render_gantt` | Gantt dependency arrows ignored link type and always drew finish→start, contradicting SS/FF/SF logic. | Arrow geometry now branches on `relationship.link_type`. |
| MEDIUM | `lib.rs` Gantt | Bar colours, progress overlay and error text were hardcoded literals that ignored the theme and diverged from the token palette. | Sourced from `design::palette(dark)` / `DesignTokens`; the progress overlay now uses the accent. |
| MEDIUM | `lib.rs` table headers | Sort headers gave no direction cue. | The active column label now shows ▲ / ▼. |
| MEDIUM | `lib.rs` table | Hiding the Name column removed the only pointer selection target. | Each row is now a full-width click target that selects the activity regardless of visible columns. |

## Open findings (ranked)

1. **HIGH — Gantt drag-to-create is always FS** (same block): acceptable default but
   worth a modifier for other types.
2. **MEDIUM — Resources view recomputes and clones per frame** (`lib.rs:1986-2051,
   3242`): `resource_conflicts`, `resource_histogram`, `project.resources.clone()`
   and `sorted().to_vec()` run every frame. Cache on `edit_version`.
3. **MEDIUM — Lookahead recomputes per frame with an O(n²) snapshot compare**
   (`lib.rs:2507,2633-2679`): cache the window; compare with a `BTreeMap`.
4. **MEDIUM — Status bar recounts critical activities every frame** (`lib.rs:3129`):
   O(activities) with a schedule lookup per row. Compute alongside `recompute()`.
5. **MEDIUM — Non-token red error text outside the Gantt** (`lib.rs:1737,1998,2040,
   2186,3287`): the Gantt now uses `palette.critical`; the top-bar status message and
   the Table/Resources/Baselines/Audit views still hardcode reds that fall below
   WCAG AA on the light panel.
6. **MEDIUM — Text fields have no AccessKit label association** (`lib.rs:1476-1577`):
   `ui.label("Name")` + `text_edit_singleline` is not linked. Pass `labelled_by`.
7. **MEDIUM — GUI cannot create or delete activities** (`lib.rs:1317-1460`):
    `CreateActivity`/`DeleteActivity` are unused here. Add toolbar controls.
8. **MEDIUM — Leveling applies N undo entries** (`lib.rs:958`): wrap in one composite
    command so a mid-batch failure cannot leave a partial state.
9. **MEDIUM — Ctrl+wheel zooms and scrolls at once** (`lib.rs:3414`): consume the
    scroll delta for zoom.
10. **MEDIUM — `select()` rewrites the link form** (`lib.rs:451`): clicking a row sets
    `link_predecessor` and may clear `link_successor`. Separate selection from form.
11. **MEDIUM — Gantt zoom is one frame behind** (`lib.rs:3280`): apply the new
    `pixels_per_day` before drawing.
12. **MEDIUM — Diary activity id** (`lib.rs:2760`): a non-numeric id silently attaches
    the note to no activity; report a parse error.
13. **LOW — `status_message` is never cleared** (`lib.rs:544`): stale errors linger.
14. **LOW — Crash "Open GitHub issue" clears the log before the browser opens**
    (`lib.rs:2876`); crash preview clones the whole log every frame (`lib.rs:2869`).
15. **LOW — Doc drift**: `ACCESSIBILITY.md:19` says Ctrl+1–5 (app has Ctrl+1–8);
    `ACCESSIBILITY.md:43` claimed a GUI project file dialog before this audit (now
    true); `docs/ui-patterns.md` Diary/clash rows are corrected.
16. **LOW — Duplicate "Toggle theme"** in toolbar and status bar (`lib.rs:1449,3044`).
17. **LOW — Gantt week-grid loop** (`lib.rs:3347`): `unwrap_or(week_origin)` would spin
    forever on a zero-working-day calendar; advance by a day or break on error.

## Accessibility and design consistency

- The audit found no keyboard dead-ends: every view has a pointer-free path (F1
  reference is accurate apart from the Ctrl+1–8 wording).
- Contrast: error text uses non-token reds (item 7); the accent is AA-verified by
  `design::accessibility_tests`.
- The GUI now saves and opens projects, closing the largest gap between the
  documented capability and the shipped app.
