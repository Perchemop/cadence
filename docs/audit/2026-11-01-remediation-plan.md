# End-to-end audit — verdict and remediation plan

Date: 2026-11-01 · Scope: whole GUI (Cadence desktop app, egui 0.35)
Inputs (all in `docs/audit/`):
- `2026-11-01-ui-surface-inventory.md` — every view, menu, button, field, shortcut (653 lines)
- `2026-11-01-ux-critique.md` — 47 severity-ranked findings (3×S1, 20×S2, 24×S3)
- `2026-11-01-gantt-gap-analysis.md` — the Gantt vs MS Project / P6 (12 ranked gaps)
- `2026-11-01-design-uplift-research.md` — free/OSS crates + design systems (egui 0.35-compatible)

## Verdict

**Does Cadence look and feel like its paid counterparts (MS Project, Primavera P6)? Not yet.**

Three reasons dominate, in order:

1. **It is not yet an authoring tool.** The GUI can tweak an existing programme but cannot
   create or delete an activity, and one whole view (Production) is a heading plus a sentence.
   MS Project / P6 are where the programme is *built*; this is a viewer-with-edits. (S1)
2. **The Gantt is a diagram, not a scheduler's chart.** Flat rows, single-tier date strip,
   no weekend shading, no milestones, no summary bars, no baseline ghosts, click-deaf,
   headless elbows. The *engine* is at parity; almost none of the data reaches the canvas.
3. **The shell reads as a prototype.** No menu bar; one wrapping toolbar of ~21 *text*
   buttons carries file ops, nine "tabs" (separated by dividers), reports, undo/redo and an
   inline "Status:" label; every status message renders in error red; no unsaved-changes guard.

The foundations are good (token palette, Inter, strict type scale, WCAG-checked accent,
solid engine + tests), so this is a composition-and-completeness problem, not a rewrite.

## Pillars

| Pillar | What "done" means |
|---|---|
| **A. Make it an authoring tool** | Create/delete/rename activities and relationships from the GUI; a real Production view; dirty-state + save-on-close guard |
| **B. Make the Gantt a scheduler's chart** | WBS outline + summary bars, two-tier timescale, non-working shading, milestones, baselines, arrows w/ heads + lag, selection, status/progress lines, real printing |
| **C. Make the shell professional** | Menu bar + grouped icon toolbar + real status bar; token-driven colour semantics; toasts; consistent Undo/Redo; clearer file actions |
| **D. Modernise the widget layer (0.35-safe)** | `egui_extras` tables + DatePicker, `egui-phosphor` icons, `egui-toast`; optional egui 0.37 migration |

## Prioritised slices

Each slice = branch → implement → CI green → squash-merge (the standing loop).

### P0 — authoring blockers (S1)
- **S1-1 Create/delete activities in the GUI.** Wire `CreateActivity` / `DeleteActivity` /
  `RestoreActivity`: "New activity" toolbar button + `Ctrl+N`, inspector "Delete activity"
  with confirm, row context menu, empty-state CTA. *(lib.rs ~2086)*
- **S1-2 Production view is a dead tab.** Either complete it (wire `RecordProduction` /
  `DeleteProduction` to the existing buffers) or remove the tab. Complete it, and bind `Ctrl+9`.
  *(lib.rs ~3168)*
- **S1-3 Unsaved-changes guard.** Dirty flag on every command; window title shows `*`; prompt
  on close. *(lib.rs ~1317, ~1670)*

### P1 — the Gantt (owner priority)
- **G1 quick wins** — non-working shading, milestones, bar click-select + highlight + tooltip,
  arrowheads + gutter routing + lag labels, label ellipsis + % + contrast fix. *(in progress)*
- **G2 structure** — WBS outline rows + summary bars + indentation + expand/collapse; two-tier
  timescale (year/quarter/month over week/day) + zoom presets (anchored); baseline ghost bars;
  a Name gutter in the Gantt view; table↔timeline vertical scroll sync.
- **G3 tracking + print** — constraint indicators, status-date line, progress line,
  behind-schedule treatment; real canvas→PDF printing (all rows, header/footer/legend).

### P2 — shell and semantics
- **Menu bar** (File / Edit / View / Insert / Project / Report / Help) replacing the button soup;
  keep a compact icon toolbar for the daily commands.
- **Status colour semantics** — use `ok` / `warn` / `critical` tokens; currently every message is red.
- **Undo/Redo consistency** — toolbar buttons must call `refresh_selection()` like `Ctrl+Z`. *(~1659)*
- **Toasts** (`egui-toast`) instead of one reused `status_message` rendered twice.
- **Gantt-view affordances** — "Split Gantt" toggle only affects Table view; make it honest.
- **Shortcuts dialog** — add `Ctrl+S`/`Ctrl+O`, nine views, `Ctrl+9`; fix dialog keyboard scoping. *(~1421, ~3270)*

### P3 — depth and polish
- Table **filter / search / group** (paid daily workflow); optional `wbs` column.
- Delete relationships; edit/delete diary notes (`UpdateDailyNote`); delete lookahead snapshots
  (`RemoveLookaheadSnapshot`); a **Location** field (`SetActivityLocation`) so the Lookahead
  "set a location in the inspector" instruction is true.
- Date inputs → `egui_extras::DatePicker` instead of raw `YYYY-MM-DD` text.
- "Save report" writes to an opaque CWD → propose a path / confirm.
- Resource leveling: composite undo + confirm before planting N constraints.
- Gantt legend, labelled today line, horizontal row separators, AccessKit per-bar nodes.

### D — widget layer
Adopt (verified egui-0.35-compatible, permissive licences):
`egui_extras@0.35.0`, `egui-phosphor@0.13.0`, `egui_dock@0.20.1`, `egui-toast@0.21.0`,
`egui-file-dialog@0.14.1`; consider `egui_dnd@0.16.0`, `egui_tiles@0.16.0`.
Open decision: **migrate egui 0.35 → 0.37** (PRs #21/#22) to unlock current ecosystem
versions — larger UI API migration; not required for the plan above.

## Verification loop

Every slice is verified by the repository CI (fmt, clippy `-D warnings`, tests on 4 targets,
security, lockfile, PII) via `~/.config/opencode/scripts/ci.sh`. No local `cargo`.
Because the live window cannot be screenshotted in this session (no wlr-screencopy / portal
Screenshot; native Wayland), visual claims rest on the deterministic render code and on a
proposed future **headless render test** (offscreen `ColorImage` + golden) rather than on guesses.

## Re-judge

After P0–P2 land, ask again: *does it look and feel like MS Project / P6?* If not, iterate on
the remaining gaps. The bar is: a planner can build and edit a programme end-to-end in the GUI,
and a screenshot of the Gantt reads instantly as a professional scheduler's chart.
