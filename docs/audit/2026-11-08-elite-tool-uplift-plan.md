# Elite-tool uplift plan — from "engineer-built" to "studio-polished"

Date: 2026-11-08 · Version: 0.12.1 (main @ `e24efd5`)
Method: coordinated parallel-agent investigation (guide §12.2) — three
specialists ran concurrently against the 2026-11-07 full visual & functional
audit, `docs/ui-patterns.md`, `docs/parity.md`, `docs/design/brand.md`, and the
inspiration research. Clean-room only: public behaviour + URLs, original
implementation, no copied trade dress.

This plan **supersedes nothing**; it refines the P2/P3 tranches of
`docs/audit/2026-11-07-full-visual-functional-audit.md` with concrete technical
approaches and a re-ranked priority order.

---

## 1. Verdict

Cadence's **scheduling engine is at parity** with the reference products (CPM,
WBS, calendars, resources, baselines, EVM, lookahead, production). The gap to an
elite paid tool is **composition, interaction density, and visual texture** —
not engine capability. Three root causes dominate:

1. **Canvas texture** — the Gantt lacks non-working shading and a two-tier
   timescale, the two cues that make a chart read as "P6/Project" at a glance.
2. **Interaction density** — bars cannot be dragged, the WBS cannot be
   collapsed, and there is no progress/status line: the chart is read-only in
   the places users most expect to author.
3. **Polish conventions** — iconography, date pickers, token colours outside the
   table, dialog keyboard behaviour (now largely fixed in P1), status-bar layout.

---

## 2. Re-ranked priority (impact × effort)

| # | Item | Audit ref | Impact | Effort | Slice |
|---|---|---|---|---|---|
| 1 | **Non-working day shading** from the assigned calendar | V5 | Very high | Low | P2-visual |
| 2 | **Token colours on canvas/grids** (kill `YELLOW`/`GRAY`/`WHITE` literals) | V9/F12 | High | Low | P2-visual |
| 3 | **Phosphor iconography** (`egui-phosphor`, MIT) | V1/V16 | High | Medium | P2-visual |
| 4 | **WBS expand/collapse outline** (+/- toggles, collapsed-state persistence) | V7 | High | Medium | P2-depth |
| 5 | **Drag-to-move/resize Gantt bars** (`RescheduleActivity`/`SetDate` commands) | V8 | High | High | P3-depth |
| 6 | **Two-tier timescale** (month/quarter over week/day) | V6 | Medium | Medium | P2-visual |
| 7 | **Monospace numerics in every grid** + right alignment | V10/F20 | Medium | Low | P2-visual |
| 8 | **Status-bar wrap + zoom tooltip** | V2/F14 | Medium | Low | P2-visual |
| 9 | **DatePicker** (`egui_extras`) replacing raw `YYYY-MM-DD` | V12 | Medium | Medium | P2-visual |
| 10 | **Progress line + status-date line** on the chart | V8 | Medium | Medium | P3-depth |
| 11 | **EVM inspector surfacing** (existing data, better placed) | F14 | Medium | Low | P2-visual |
| 12 | **Constraint/deadline indicators** on bars | V8 | Low-medium | Low | P3-depth |
| 13 | **Theme + recent-files persistence** | V4 | Medium | Medium | P3-depth |
| 14 | **Golden-render / screenshot pipeline** (honest visual review) | V15 | Enabling | Medium | P4-infra |

**Sequencing rule.** Items 1–2 first (cheapest, highest texture gain), then 7–8
and 11 (cheap polish), then 3 and 6 (iconography, timescale), then 4–5
(the structural interaction work), then 9–10 and 12–13. Item 14 can run in
parallel as enabling infrastructure.

---

## 3. Technical approaches (clean-room)

### 3.1 Non-working day shading (V5) — highest impact

- **Domain:** no new types; `Calendar::is_working(date)` already exists and the
  engine uses it. Resolve each activity's calendar (`activity.calendar` →
  `project.calendars` → `project.calendar`).
- **Canvas:** pre-compute a `HashSet<Date>` of non-working days for the visible
  window once per frame; fill those column rectangles with a token surface tint
  (8–12 % opacity) **before** bars/arrows; a darker stripe distinguishes
  holidays from weekends.
- **Bonus:** base the weekly hairlines on working-day windows so they stop
  drifting across holidays.
- **Reference (public behaviour only):** Microsoft "set a project calendar" /
  "hide weekends in Gantt chart" support pages; Oracle Primavera public feature
  pages on calendar display.

### 3.2 Token colours on canvas and grids (V9/F12)

- Replace `Color32::YELLOW` (drag preview), `GRAY` (arrows/ticks), `WHITE`
  (milestone outline, summary end-caps) with `design::palette` tokens resolved
  for the active theme; move `variance_color` literals into `design.rs`.
- Apply monospace + right alignment to dates/floats/counts in Resources,
  Baselines, Calendars and Diary grids (V10/F20) via a shared cell helper.

### 3.3 Iconography (V1/V16)

- Adopt `egui-phosphor` (MIT) pinned to the egui 0.35-compatible release;
  register the font in `design.rs` and use icons in the menu bar, tab strip,
  tool cluster and inspector section headers, each with an `on_hover_text`
  label so the text meaning is never lost.
- Fall back to text-only if the pinned version is unavailable; never block on
  the dependency.

### 3.4 WBS expand/collapse outline (V7)

- **UI-only state:** `HashSet<WbsId> collapsed_wbs` in the app state, persisted
  with preferences. No project mutation, so no new domain command is required.
- Filter the flattened activity list through a "visible" predicate; draw only
  visible rows in table and Gantt; keep the summary bar visible with children
  hidden; +/- glyph in the first column.

### 3.5 Drag-to-move/resize bars (V8)

- **New reversible commands:** `RescheduleActivity { id, new_start, new_finish,
  mode }` and `SetDate { id, field, value }`, both re-running `schedule()` and
  pushing an undo entry.
- **Hit testing:** distance to bar body < 6 px → move; distance to edge < 6 px →
  resize; convert pixel delta to working days with the activity's calendar;
  clamp at constraints and surface a status warning; preview line uses a token
  colour.

### 3.6 Two-tier timescale (V6)

- Pure presentation driven by `pixels_per_day`: day labels at high zoom, week
  numbers over month names at mid zoom, months/quarters/years at low zoom;
  two stacked header bands above the canvas.

### 3.7 Progress line + status date (V8)

- `SetStatusDate(date)` (reversible) drives a second vertical line and EVM
  recompute; a stepped progress line runs from `actual_start` to the
  percent-complete position for in-progress activities.

---

## 4. What is already done (avoid re-doing)

- P0 correctness (F1/F2/F3/F4/F5) — merged `fba8d98`.
- P1 dialog & feedback polish (F13/F15/F16/F18/F21) — merged `0443f4c`.
- F22 confirm-deletes (resource/baseline/diary/production) — in flight.
- F17 terminology drift — remaining P1.

---

## 5. Clean-room and legal notes

- Every recommendation is implemented from **public documented behaviour** and
  the project's own spec; no proprietary source, icons, layout or strings.
- Third-party assets are permissively licensed (MIT/Apache-2.0/ISC) and must be
  recorded in `ATTRIBUTION.md`; `egui-phosphor` and `egui_extras` are MIT.
- No pixel-copying of any reference product; iconography and layout remain
  original.

---

## 6. Records

- Full defect audit: `docs/audit/2026-11-07-full-visual-functional-audit.md`
- Behavioural matrix: `docs/ui-patterns.md`
- Capability tracker: `docs/parity.md`
- Design source of truth: `docs/design/brand.md`
- UI/UX uplift deep-dive: `docs/design/elite-ui-ux-uplift.md`
- Inspiration research: `docs/research/ms-project-capabilities.md`,
  `docs/research/primavera-p6-capabilities.md`
