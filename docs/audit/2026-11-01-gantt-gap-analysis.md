# Gantt gap analysis — Cadence vs Microsoft Project / Primavera P6

Date: 2026-11-01 · Scope: read-only audit · Author: agent session (gap analysis)

**Question this audit answers:** does the Cadence Gantt look and feel like the
Gantt chart in Microsoft Project or Primavera P6? Today it does not: it is a
flat, single-tier timeline of coloured bars with simple elbows — closer to a
diagram than to a scheduler's chart. The scheduling *engine* behind it (CPM,
calendars, constraints, baselines, EVM, audit) is at parity per
`docs/parity.md`; almost none of that data reaches the canvas.

**Method:** every claim below cites `crates/ui-egui/src/lib.rs` (the app, one
file) or `crates/core/src/...` (the model/engine) with line numbers at
commit-time (file is 4045 lines). `render_gantt` spans
`lib.rs:3557–3881`; the Gantt view branch is `lib.rs:2235–2255`; the split
workbench Gantt is `lib.rs:1985–2011`. Research baselines:
`docs/research/ms-project-capabilities.md`,
`docs/research/primavera-p6-capabilities.md`.

Status legend: **present** = works as MSP/P6 would; **partial** = exists but
materially weaker; **missing** = not in the code at all.

---

## 1. Evidence matrix

### 1.1 Timescale

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Day/week/month/quarter/year granularity | **missing** | The header renders only ISO date strings at every `tick_step` (1/5/10 days, chosen purely from px/day) — `lib.rs:3708–3714`, `3716–3736`. No week/month/quarter/year tick anywhere. | A two-tier calendar header: months on top, weeks or days below, quarter/year as you zoom out. |
| Two-tier (major/minor) header | **missing** | One 24 px band (`header_height = 24.0`, `lib.rs:3581`) with one row of `Date::to_string()` labels (`3727–3734`). | Major/minor bands with unit names ("Jan 2026", "W1", "Q2") and separators between periods. |
| Zoom levels | **partial** | Continuous `pixels_per_day` clamped 2–60 px/day via buttons (`3588–3609`), Ctrl+wheel (`3699–3706`), Ctrl+Plus/Minus/0 keys (`1424–1445`). No discrete presets, no "zoom to fit", and zoom is not anchored — the date under the cursor drifts because only the scale changes (`3700–3706`). | Named timescale presets (Day/Week/Month/Quarter/Year) and a "zoom to selected/fit project" action that keeps the focus date stable. |
| Pan behaviours | **partial** | Scrollbars only (`ScrollArea::both()`, `lib.rs:3613`). Drag is reserved for link creation, so there is no drag-to-pan; panning is pointer/wheel-only and not keyboard operable (recorded in `ACCESSIBILITY.md:93`). | Middle-button or empty-space drag panning, keyboard scroll of the timeline, and auto-scroll when dragging near the edges. |
| Timeline range | **partial** | Hard span = project start → finish + 2 days (`3577–3583`); no lead-in or lead-out, so early bars hug the left edge and there is no past/future context. | Padded timescale with configurable start/end and a visible "beyond project" region. |

### 1.2 Row hierarchy

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Summary/parent bars | **missing** | The model has a WBS tree — `Activity.wbs: Option<WbsId>` (`crates/core/src/model.rs:617–619`) and `Project.wbs: WbsTree` (`model.rs:788`) — but the UI crate contains **zero** references to `wbs` (grep across `crates/ui-egui/src` finds none). `render_gantt` iterates a flat `activities` slice (`3747`). | Thick black summary bars spanning their children's dates, with end-caps, rolling up progress and dates. |
| Expand/collapse | **missing** | No outline state anywhere; the table has no WBS column either — columns are Name/Duration/Start/Finish/Float/Critical (`lib.rs:73–124`). | +/- outline toggles on summary rows that collapse both the table and the chart. |
| Indentation | **missing** | No depth/indent concept in either the table rows (`2106–2216`) or Gantt rows (`3747–3792`). | Indented task names under summaries, in both the table and the chart's label column. |
| Row height / grouping | **partial** | Fixed `row_height = 20.0` (`3580`), uniform for all rows; rows follow the table's sort order so the split view stays aligned (`3747`; `PROGRESS.md:906`) — but no group bands, band shading, or per-group height. | Group/filter band rows, taller summary rows, and user-adjustable row height. |

### 1.3 Bar types and rendering

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Normal task bar | **present** | Rounded rect from scheduled start to finish+1 day, min width 6 px (`3750–3758`), themed colour (`design.rs:49–50`, `62–63`). | — (this one matches). |
| Summary bar | **missing** | Nothing in `render_gantt` aggregates children (see 1.2). | Summary bars with end-caps, drawn over the group's full span. |
| Milestone (zero-duration marker) | **missing** | Zero-duration activities draw a 6 px stub bar (`3755` `.max(6.0)`); the model explicitly says "Zero denotes a milestone" (`model.rs:615`) but nothing special-cases duration 0. | Black diamonds (start/finish milestones) with labels to the side, never clipped to a stub. |
| Critical bar | **partial** | Colour swap to `tokens.critical` (`3759–3763`) plus a `!` text prefix (`3780–3784`). No weight/shape distinction, no legend. | Red critical bars plus explicit "Critical"/"Noncritical" legend, thickness, and (P6) longest-path vs float-critical distinction. |
| Completed / % fill | **partial** | Accent overlay fills the left `percent_complete/10_000` share of the bar (`3766–3778`). No % text on the bar. | Progress fill *plus* the % printed on the bar, and a distinct hatched "remaining" section. |
| Remaining bar | **missing** | The overlay covers the completed share only; no visual remainder treatment (`3766–3778`). | Remaining work drawn as an unfilled/hatched extension, and progress-on-actual-dates (fill bounded by the status date, not the bar length). |
| Baseline "ghost" bars | **missing** | Baselines are fully modelled (`ActivityBaselineSnapshot`, `model.rs:1000–1014`; capture/delete in `lib.rs:1097–1240`) and shown as a variance *table* in the Baselines view (`2447–2600`) — `render_gantt` never reads `project.baselines`. | Tracking-Gantt overlay: baseline bar behind the current bar, slippage visible as red/green overhang. |
| Delayed / behind-schedule bar | **missing** | Actual start/finish exist (`model.rs:666–671`) but are unused on the canvas; nothing compares scheduled vs actual vs status date. | Bars behind schedule flagged with a marker/colour (MSP status-mark indicators, P6 progress-bar-over-remaining). |
| Constraint & deadline indicators | **missing** | Constraints are modelled (`model.rs:522–606`) and editable in the inspector (`lib.rs:1759–1776`) but never drawn; no deadline field exists (deliberate per D15, `ms-project-capabilities.md:96`). | Constraint icons at the constrained edge, deadline arrows/flags, and slack-to-deadline whiskers. |
| Progress lines | **missing** | Not present anywhere in the canvas. | The stepped progress line connecting in-progress bar tips against the status date. |
| Status date line | **missing** | A status date exists only as an EVM input in the Baselines view (`2480–2491`); the Gantt has no status-date marker. | A vertical status-date line against which progress is measured, labelled and moveable. |
| Today line | **present** | 1.5 px accent vertical line when today ∈ [start, finish] (`3650–3658`); no label, and absent entirely if today is outside the project window. | A labelled, always-visible date line with a shaded band and a header flag. |

### 1.4 Dependency arrows

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| All four link types FS/SS/FF/SF | **present** | Anchor mapping `Fs/Ss/Ff/Sf → (pred_from_start, succ_at_finish)` with correct 0/+1-day edge offsets (`3826–3847`). | — (anchoring matches; see routing/lag below). |
| Correct orthogonal routing | **partial** | Three segments: horizontal → vertical at the x-midpoint → horizontal (`3850–3854`). The mid-point route crosses bars and other arrows, produces wrong-reading geometry when the successor sits left of the predecessor (negative lag / out-of-sequence), and ignores row spacing. | Elbow routes that drop into the gutter between rows, avoid bars, and flip direction cleanly for out-of-sequence links. |
| Arrowheads | **missing** | Lines only; no triangle/chevron is drawn at the successor end (`3850–3854`). | A filled arrowhead that makes link direction legible at a glance. |
| Lag labels | **missing** | `Relationship.lag` exists and is editable (`1904–1917`, `1974–1978`) but is never rendered on the canvas. | Lag/lead values printed at the elbow, editable in place. |
| Hover highlight of driving logic | **missing** | Arrows are painted strokes with no `Response`, so they cannot be hovered or clicked (`3803–3855`). | Hover a link (or a Task-Path / driver view) to light up the driving chain into the selected activity. |

### 1.5 Non-working time

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Weekend/holiday shading | **missing** | No shading anywhere; the only calendar use on the canvas is `add_working_days(week_origin, 5)` to place weekly hairlines (`3632–3649`) — a working-day step, so "week" lines drift whenever a holiday shifts the 5-day block. | Grey columns for non-working time derived from the assigned calendar — the single most recognisable texture of a P6/MSP Gantt. |
| Calendar-aware shading | **missing** | Per-activity calendars exist (`model.rs:623–625`, Calendars view `2600+`) but the canvas shades nothing. | Per-row shading when an activity uses a different calendar than the project default. |

### 1.6 Today line

**present** — accent vertical line, `lib.rs:3650–3658`; drawn only when today is
inside [project start, project finish], unlabelled. MSP/P6 label the line in the
header and keep it visible with a marker.

### 1.7 Text on/near bars

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Name on bar | **partial** | Centered, 10 px, white, with `!` prefix when critical (`3780–3791`). Not truncated: long names overflow the bar and are cut only by the canvas clip rect (`3621–3623`). In the full-screen Gantt there is **no label column at all** — names exist only inside bars. | Ellipsised in-bar labels, plus per-field labels placed left/right/above the bar, and a persistent name column beside the chart. |
| Resource on bar | **missing** | `Activity.resources` (`model.rs:620–622`) never rendered on the canvas. | Resource names printed under the bar; resource over-allocation markers. |
| % complete on bar | **missing** | Progress is drawn but the number is not (`3766–3778`). | "45%" printed at the bar tip. |
| Dates on bar | **missing** | Start/finish appear only in the table (`2167–2188`). | Start/finish dates flanking the bar (P6 style) or on hover. |

### 1.8 Interaction

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Click-select bars (sync with table) | **missing** | The canvas takes `Sense::click_and_drag()` (`3614–3617`) but `.clicked()` is never handled and no `selected` state is read anywhere in `render_gantt`; clicking a bar does nothing and there is no selection highlight on the chart. Keyboard Up/Down selection is Table-view only (`1447–1476`). | Click a bar to select the row (table + inspector + highlight all follow), with the selected bar outlined. |
| Drag to move / resize bars | **missing** | No bar hit-testing at all — dates are CPM output; dragging is reserved for link creation. Manual scheduling is a recorded deliberate exclusion (D15, `ms-project-capabilities.md:96`), so this is a philosophy gap, but P6-style *drag re-constrain* (drag to set a SNET date) remains available without breaking the engine. | Drag a bar to move it or grab an edge to change duration, with the schedule recalculating live. |
| Drag to create links | **partial** | Any drag inside the canvas starts a link from the row under the pointer, **ignoring x** (`3661–3668`); the preview is a yellow line from the predecessor's finish edge (`3670–3685`); drop creates an FS lag-0 link with cycle detection + undo (`3858–3879`, `3864`). Link type is not choosable in the gesture (the form is, `1921–1978`). | Edge-grab glyphs at bar ends, a live link-type preview, and the chosen type/lag reflected in the gesture. |
| Tooltips | **missing** | No `on_hover_text` anywhere on the canvas (the only hover texts are the zoom buttons, `3590–3604`). | Rich hover: full name, dates, float, resources, predecessors. |
| Hit-testing | **partial** | Row-only, from the y coordinate (`3663`, `3691`); no per-bar rects, no bar-edge zones, no arrow hit-testing. | Precise per-bar and per-link hit targets with cursors that change at resize handles. |

### 1.9 Scroll synchronisation

**missing** — the table's `ScrollArea` (`2091–2098`) and the Gantt's
`ScrollArea` (`3613`) are independent widgets with no shared vertical offset;
the split workbench (`1985–2011`) only *looks* aligned while both sit at the
top. Scrolling one pane leaves the other behind; there is also no horizontal
sync of a shared timescale between two charts. MSP/P6 pin the table and chart
to one scroll position, always.

### 1.10 Printing / PDF

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Gantt → PDF | **partial** | "Print Gantt…" (Gantt view only, `1614–1630`) issues a viewport `Screenshot` (`1628`), consumes the `Screenshot` event (`1478–1514`), and writes RGB through `Pdf::image_page` (`1499–1500`). Documented limits: the capture is the **whole window** (toolbar, panels, current scroll) and virtualised rows outside the viewport are absent (`PROGRESS.md:329–346`; `printing-the-gantt-view.md:33–35`). | A print engine: fit-to-pages timescale, repeated table columns on every page, header/footer, legend, page numbers, vector output. |

### 1.11 Accessibility

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| AccessKit labels for bars/rows | **partial** | The whole canvas is a single `WidgetInfo::labeled` node carrying the `gantt_summary` string (`3618–3620`; `3543–3554`). Individual bars are **not** AccessKit nodes — recorded as a known limit (`ACCESSIBILITY.md:58–61`, `90–91`). Zoom is keyboard-operable (`1424–1445`); panning is not (`ACCESSIBILITY.md:93`). The PDF is an untagged image (`ACCESSIBILITY.md:112–113`). | Per-row/per-bar accessible nodes (at minimum a list of activity names + dates), so the chart is navigable without switching to the Table view. |

### 1.12 Visual polish

| Item | Status | Evidence | What MSP/P6 shows that Cadence does not |
|---|---|---|---|
| Grid contrast | **present** | Deliberately low-contrast grid tokens for both themes (`3625–3631`; `design.rs:27, 48, 61`), leaving colour for meaning. | — (structure/colour separation is fine; MSP separates period bands a bit more strongly). |
| Bars aligned to day boundaries | **present** | x = (julian-day − origin) × px/day; width = (finish − start + 1) × px/day (`3750–3756`). | — |
| Label truncation | **missing** | `painter.text` with no measurement or ellipsis (`3785–3791`); labels collide with neighbours and the clip edge. | In-bar ellipsis and clean overflow into the label gutter. |
| Bar-label contrast | **partial** | Bar text is hardcoded `Color32::WHITE` (`3790`), including on the dark-theme `normal` bar `0x93C5FD` (`design.rs:50`) — white on light blue is far below WCAG 3:1 for the glyph, and the contrast tests cover variance/accent colours but not bar text (`3935–3980`). The drag preview is hardcoded yellow (`3681`), also untested. | Labels always legible on every bar colour (MSP switches label colour or places labels outside the bar). |
| Colour semantics | **partial** | Critical = red, normal = blue, progress = accent (`design.rs:41–68`); critical also encoded as `!` text (`3780–3784`) — good. But the legend is only the `gantt_summary` sentence (`3543–3554`). | An on-canvas legend explaining every colour, marker and line. |

---

## 2. Ranking by "does this look and feel like MSP/P6?"

Ranked by combined weight of (a) visual signature — what makes a screenshot
instantly read as a professional scheduler — and (b) daily-use feel. 1 = fix
first.

| # | Gap (status) | Why it dominates the impression |
|---|---|---|
| 1 | WBS outline rows: summary bars, indentation, expand/collapse (**missing**) | The defining structure of both tools; Cadence's flat list is the #1 reason it reads as a toy. The model already carries the WBS — this is UI-only work. |
| 2 | Two-tier timescale header (month over week/day; quarter/year when zoomed out) + zoom presets (**missing**) | The second thing the eye checks; a strip of ISO dates every N days is the #1 tell that this is not MSP/P6. |
| 3 | Weekend/holiday shading from the calendar (**missing**) | The grey columns are the texture of every construction Gantt; also fixes the drifting "weekly" hairlines (`3645–3648`). |
| 4 | Milestone diamonds for zero-duration activities (**missing**) | Milestones are how programmes communicate; 6 px stub bars hide them entirely. |
| 5 | Click-to-select on bars + selection highlight + tooltips (**missing**) | The chart currently ignores clicks; users immediately conclude "the Gantt isn't interactive". |
| 6 | Baseline ghost bars (Tracking-Gantt overlay) (**missing**) | Baselines exist end-to-end in the model but are invisible on the chart — the highest-value data already paid for. |
| 7 | Arrowheads + elbow routing + lag labels (**partial**) | Headless mid-routed elbows look unfinished; lag is entered but never shown. |
| 8 | Table↔timeline vertical scroll sync + a name column in the Gantt view (**missing**) | The split workbench desyncs on first scroll, breaking the "task sheet + chart" layout it imitates. |
| 9 | Bar text: ellipsised labels, % complete, resource names; in-Gantt label column (**partial/missing**) | Bars are the content; MSP/P6 bars carry 2–3 fields of text, Cadence's carry one. |
| 10 | Constraint & deadline indicators on the timeline (**missing**) | Constraints are scheduled and audited yet invisible where planners expect them. |
| 11 | Status date line + progress line + behind-schedule treatment (**missing**) | The tracking overlay that turns a plan into a status report; status date already exists as EVM input. |
| 12 | Printing as a real chart output (canvas crop, all rows, header/footer/legend) instead of a window screenshot (**partial**) | For a construction audience the printed Gantt *is* the deliverable; a whole-window screenshot is a stopgap. |

Just below the line: zoom-to-fit and anchored zoom; drag-to-move/re-constrain
bars (in tension with D15); AccessKit per-bar nodes; bar-label contrast fix on
dark theme; a visible legend; drag-link type picker (currently FS/0 only,
`3864`); labelled today line; horizontal row separators.

---

## 3. Ranked top 12 with fix direction

1. **WBS outline rows with summary bars** — flatten the `WbsTree` into
   `sorted_cache` rows with depth, indent the Name column, draw a summary bar
   (thin dark rect + end-caps) spanning each node's children's min/max dates,
   and add collapse state that filters both table and chart rows.
2. **Two-tier timescale header** — draw two header bands: major tier
   (year/quarter/month chosen from px/day) above the existing minor tier of
   days/weeks; add discrete zoom presets (Day/Week/Month/Quarter/Year) instead
   of raw px/day, anchored so the date under the cursor stays put.
3. **Non-working-time shading** — for each day column, ask
   `project.calendar.is_working(date)` (plus per-activity calendars later) and
   fill non-working columns with the theme's `deep` colour before bars; replace
   the working-day-step hairlines with real calendar-week lines.
4. **Milestone diamonds** — special-case `original_duration == 0`: draw a
   rotated-square marker at the finish date centred on the row, label to the
   right, keep the min-6 px bar for everything else.
5. **Canvas hit-testing + selection + tooltips** — compute each visible bar's
   `Rect` (already available at `3757–3758`), on click set the app's
   `selected` id and re-render table/inspector state, outline the selected bar,
   and `on_hover_text` name/dates/float/resources.
6. **Baseline ghost bars** — read the selected `Baseline` snapshots in
   `render_gantt` and draw a hollow/ghost rect behind each bar from
   baseline_start→baseline_finish, tinted red/green by the sign of the finish
   variance (`variance_color` already exists, `lib.rs:3401`).
7. **Arrowheads, routing, lag labels** — add a filled triangle at `(x1, y1)`;
   route via a channel in the row gutter (or offset the vertical segment by
   row-pair hash) instead of the raw midpoint; print `lag` near the elbow when
   non-zero; flip the vertical-drop side when `x1 < x0`.
8. **Scroll sync + Gantt label column** — lift the vertical scroll offset into
   shared app state and feed both `ScrollArea`s (`ScrollArea` supports
   programmatic offsets, as the table already does at `2092–2097`); in the
   full Gantt view reserve a left gutter rendering the Name cell per row.
9. **Bar text improvements** — measure the label, ellipsize inside the bar,
   print `percent_complete%` at the tip and resource names under the bar; move
   labels outside the bar when they do not fit (and fix the white-on-light-blue
   dark-theme label contrast while there).
10. **Constraint/deadline markers** — draw a small glyph (⌐/flag) at the
    constrained edge of any activity with a date constraint, with the
    constraint date as the tooltip.
11. **Status date + progress line** — render the (existing) EVM status date as
    a labelled vertical line; optionally draw the stepped progress line through
    in-progress bars' percent-points at the status date; tint bars that finish
    after the status date unstarted/behind.
12. **Real Gantt printing** — replace the whole-window `ViewportCommand::
    Screenshot` with a painter-to-image pass: render the canvas (all rows, full
    timescale, header, legend, title/footer) into an offscreen `ColorImage` at
    print DPI and feed `Pdf::image_page` — the PDF writer needs no change
    (`printing-the-gantt-view.md` already sketches this).

---

## 4. Notes and caveats

- **Deliberate exclusions respected:** manual (user-controlled) scheduling and
  deadline-only fields are recorded as out of scope (D15,
  `docs/research/ms-project-capabilities.md:96`); this audit flags the *feel*
  gap but not a change of decision. Longest-path vs float-critical (P6) and
  multiple-baseline overlay selection are engine-side candidates noted in
  `docs/parity.md`.
- **Performance guards to preserve while fixing:** the O(relationships) arrow
  pass with the cached id→row map (`3795–3802`), the clip-rect-culled weekly
  grid (`3632–3649`), the visible-row window (`3738–3745`), and borrowed
  schedule (`3560`, `3570–3576`) are the reasons the canvas survives large
  programmes (`docs/parity.md:45`, `PROGRESS.md:885–906`). Any summary-bar,
  shading, or hit-testing work should extend these patterns, not replace them.
- **Quick wins with outsized effect:** milestone diamonds (#4), arrowheads
  (#7a), bar click-select (#5) and bar-label ellipsis (#9a) are each small,
  local changes to `render_gantt` that together close a large share of the
  visual gap before the bigger hierarchy/timescale work lands.
