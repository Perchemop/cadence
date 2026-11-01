# UI Surface Inventory — Cadence (egui/eframe)

Date: 2026-11-01 · Scope: read-only analysis of the user-facing surface of
`crates/ui-egui/src/lib.rs` (4 045 lines), `design.rs`, `crash.rs`,
`update_check.rs`, cross-checked against `crates/commands/src/lib.rs` and
`crates/core/src/model.rs`. No source was modified.

All `lib.rs:NNN` citations refer to `crates/ui-egui/src/lib.rs`; `design.rs:NNN`
to `crates/ui-egui/src/design.rs`; `commands:NNN` to `crates/commands/src/lib.rs`;
`model:NNN` to `crates/core/src/model.rs`.

---

## 1. Views / tabs (ViewMode)

`ViewMode` enum: `lib.rs:127-138`. State field `view_mode`: `lib.rs:226`.
Default view: `Table` (`lib.rs:316`).

| View | Reaching it | Code | Notes |
|---|---|---|---|
| Table | Toolbar tab "Table" (`lib.rs:1547`); **Ctrl+1** (`lib.rs:1400`) | `ViewMode::Table` render `lib.rs:1742-2234` | Default view |
| Gantt | Toolbar tab "Gantt" (`lib.rs:1548`); **Ctrl+2** (`lib.rs:1401`) | `lib.rs:2235-2255` | Zoom keys active only here (`lib.rs:1425`) |
| Resources | Toolbar tab "Resources" (`lib.rs:1549`); **Ctrl+3** (`lib.rs:1402`) | `lib.rs:2256-2445` | |
| Baselines | Toolbar tab "Baselines" (`lib.rs:1550`); **Ctrl+4** (`lib.rs:1403`) | `lib.rs:2446-2603` | |
| Calendars | Toolbar tab "Calendars" (`lib.rs:1551`); **Ctrl+5** (`lib.rs:1404`) | `lib.rs:2604-2696` | |
| Audit | Toolbar tab "Audit" (`lib.rs:1552`); **Ctrl+6** (`lib.rs:1405`) | `lib.rs:2697-2788` | Audit auto-runs on entry (`lib.rs:2699-2701`) |
| Lookahead | Toolbar tab "Lookahead" (`lib.rs:1553`); **Ctrl+7** (`lib.rs:1406`) | `lib.rs:2789-3045` | |
| Diary | Toolbar tab "Diary" (`lib.rs:1554`); **Ctrl+8** (`lib.rs:1407`) | `lib.rs:3047-3166` | |
| Production | Toolbar tab "Production" (`lib.rs:1555`) only | `lib.rs:3168-3173` | **No keyboard shortcut** — `Num9` is not in the key array (`lib.rs:1383-1391`) and the match arm `8 => ViewMode::Production` (`lib.rs:1408`) is unreachable |

Keyboard dispatch block: `lib.rs:1378-1412`. The comment there says
"Ctrl+1..Ctrl+5" but the code wires 8 keys — stale comment.

Programmatic view switches (jump-to-activity): Audit finding click
(`lib.rs:2774`), Lookahead row click (`lib.rs:2855`), Lookahead clash links
(`lib.rs:3036`) all set `view_mode = ViewMode::Table` after `select()`.

---

## 2. Window / global chrome

| Item | Label / content | Behaviour | Code |
|---|---|---|---|
| Native window title | `Cadence` | — | `lib.rs:3892` |
| Theme | Dark by default | `apply_theme` via `design::apply` | `lib.rs:311`, `design.rs:139-198` |
| Fonts | Inter Regular/Medium/SemiBold/Bold, 11 px body, 9.5 small, 11 button, 16 heading, 11 mono | `install_fonts` / `install_style` | `design.rs:73-135` |
| Visual states | Hover fill `#262A33` (dark) / `#EEF1F6` (light); active `#2E333D` / `#E2E8F2`; selection accent `#2F6FED`; striped rows on; 4 px corner radius | applied to every widget | `design.rs:156-195` |
| Semantic colours | critical red `#F87171`/`#B91C1C`, normal blue `#93C5FD`/`#1D4ED8`, ok green, warn yellow, grid hairline | `palette(dark)` | `design.rs:41-69` |

---

## 3. Toolbar (top panel) — `lib.rs:1516-1684`

Panel id `toolbar`, laid out with `horizontal_wrapped` (`lib.rs:1517`).

### 3.1 Identity & file actions

| Control | Type | Label / tooltip | Action (function) | Code |
|---|---|---|---|---|
| Heading | text | "Cadence" | none | `lib.rs:1518` |
| Project label | text | `{project.name}` | none (display only) | `lib.rs:1520` |
| Button | button | "Open…" / tooltip "Open a Cadence project file (Ctrl+O)" | `open_dialog()` (`lib.rs:1184`) → rfd pick → `open_from()` (`lib.rs:1194`) | `lib.rs:1521-1527` |
| Button | button | "Save" / tooltip "Save the project (Ctrl+S)" | `save_or_prompt()` (`lib.rs:1146`) → `save_to()`/`save_as_dialog()` | `lib.rs:1528-1534` |
| Button | button | "Save as…" / tooltip "Save the project to a new file" | `save_as_dialog()` (`lib.rs:1167`) | `lib.rs:1535-1541` |

### 3.2 View tabs — `lib.rs:1544-1569`

Nine `egui::Button`s separated by `ui.separator()`; active tab uses
`.selected(selected).frame(selected)` so the selected tab keeps its fill
(`lib.rs:1563-1568`). Labels exactly: `Table`, `Gantt`, `Resources`,
`Baselines`, `Calendars`, `Audit`, `Lookahead`, `Diary`, `Production`
(`lib.rs:1546-1556`). Clicking sets `view_mode`. No tooltips on tabs.

### 3.3 Layout / table options

| Control | Type | Label / tooltip | Action | Code |
|---|---|---|---|---|
| Split toggle | button (label reflects state) | "Split Gantt: on" / "Split Gantt: off" | flips `show_split_gantt` | `lib.rs:1572-1581` (state `lib.rs:290`, default true `lib.rs:380`) |
| Menu | menu_button | "Columns" / tooltip "Choose which columns show in the activity table (MS Project / P6 column chooser)" | opens column chooser (below) | `lib.rs:1582-1599` |
| ├ heading | text | "Visible columns" | — | `lib.rs:1583` |
| ├ checkbox ×6 | checkbox | "Activity", "Dur", "Start", "Finish", "Float", "Critical" (from `SortKey::label`, `lib.rs:102-111`) | toggles `visible_columns` | `lib.rs:1584-1593` |
| └ button | button | "Reset to default" | restores all 6 columns | `lib.rs:1594-1597` |

### 3.4 Reports & printing

| Control | Type | Label / tooltip | Action | Code |
|---|---|---|---|---|
| Button | button | "Save report" / tooltip "Write a Markdown report next to the working directory" | `save_report()` (`lib.rs:1214`) writes `{project}.report.md` to CWD (no dialog) | `lib.rs:1600-1606` |
| Button | button | "Print report…" / tooltip "Save a print-ready PDF of the report via a file dialog" | `print_report_dialog()` (`lib.rs:1122`) → rfd save (filter "PDF document" `*.pdf`, name `{project}.pdf`) → `cadence_core::report::project_report_pdf` | `lib.rs:1607-1613` |
| Button | button, **enabled only in Gantt view** (`add_enabled`, greyed otherwise) | "Print Gantt…" / tooltip "Save a print-ready PDF of the Gantt view via a file dialog" | rfd save (name `{project}-gantt.pdf`) → viewport `Screenshot` command → PDF write in `lib.rs:1480-1514` | `lib.rs:1614-1632` |

### 3.5 Help / feedback / updates

| Control | Type | Label / tooltip | Action | Code |
|---|---|---|---|---|
| Button | button | "Report a bug…" / tooltip "Preview and report a problem to the Cadence issue tracker" | re-reads crash log if empty, clears notes, opens crash dialog | `lib.rs:1633-1645` |
| Button | button, dynamic label | "Updates" or "Update available: {version}" / tooltip "Check GitHub Releases for a newer Cadence" | opens update dialog | `lib.rs:1646-1658` |
| Button | button | "Undo" — **no tooltip** | `stack.undo` + `recompute`; **does not call `refresh_selection()`** (contrast Ctrl+Z path `lib.rs:1364-1373`) | `lib.rs:1659-1662` |
| Button | button | "Redo" — **no tooltip** | `stack.redo` + `recompute`; same refresh gap | `lib.rs:1663-1666` |
| Status | label | "Status: {message}" | last status message (plain, no colour) | `lib.rs:1667-1669` |
| Button (right-aligned) | button | "Toggle theme" — no tooltip | flips `dark`, `apply_theme` | `lib.rs:1670-1674` |
| Button (right-aligned) | button | "Shortcuts" / tooltip "Keyboard shortcuts (F1)" | opens shortcuts dialog | `lib.rs:1675-1681` |

---

## 4. Status bar (bottom panel) — `lib.rs:1690-1740`

| Item | Content | Code |
|---|---|---|
| Summary label | "{start} to {finish}   {n} activities   {n} critical   Zoom {pct}%" (weak, small); shows "Cannot schedule" when the schedule errored | `lib.rs:1694-1722` |
| Status message | "   ·   " separator then the message in dark red `#C0392B` (weak, small) — **styled differently from the toolbar copy** | `lib.rs:1723-1731` |
| Button (right-aligned) | "Toggle theme" — duplicate of the toolbar control | `lib.rs:1732-1737` |

There are **no toasts**; the single `status_message: Option<String>`
(`lib.rs:229`) is rendered in both toolbar and status bar.

---

## 5. Table view (`ViewMode::Table`) — `lib.rs:1742-2234`

### 5.1 Inspector panel (right, id `inspector`, default 260 px) — `lib.rs:1744-1983`

Shown only when an activity is selected; otherwise the single label
"Select an activity in the table." (`lib.rs:1980`).

Headings: "Inspector" (`lib.rs:1750`), "Calendar" (`lib.rs:1776`),
"Field planning" (`lib.rs:1831`), "Progress" (`lib.rs:1845`),
"Relationships" (`lib.rs:1862`), "Add relationship" (`lib.rs:1921`).

| Field / control | Type | Label | Value handling / command | Code |
|---|---|---|---|---|
| Activity id | label | "Activity {id}" | display | `lib.rs:1752` |
| Name | text_edit_singleline | "Name" | `RenameActivity` on Apply | `lib.rs:1753-1754`, `lib.rs:595-606` |
| Duration | text_edit_singleline | "Duration (working days)" | `SetActivityDuration` | `lib.rs:1755-1756`, `lib.rs:608-628` |
| Constraint | ComboBox id `constraint_kind` | "Constraint"; options from `CONSTRAINT_LABELS` (`lib.rs:144-151`): "As soon as possible", "As late as possible", "Start no earlier than", "Start no later than", "Finish no earlier than", "Finish no later than" | `SetActivityConstraint` | `lib.rs:1757-1769`, `lib.rs:630-656` |
| Constraint date | text_edit_singleline, shown only when kind ≥ 2 | "Constraint date (YYYY-MM-DD)" | parsed `parse_date` (`lib.rs:3320`) | `lib.rs:1770-1773` |
| Calendar | ComboBox id `activity_calendar` | (no field label — preceded by "Calendar" heading); options: "Project default" + every named calendar | `SetActivityCalendar` (`lib.rs:1802-1826`) | `lib.rs:1791-1828` |
| Owner | text_edit_singleline | "Owner" | `SetActivityOwner` via `apply_field_planning` | `lib.rs:1832-1833`, `lib.rs:487-529` |
| Promised | checkbox | "Promised in the lookahead" | `SetActivityPromise` (with delay reason) | `lib.rs:1834-1837` |
| Delay reason | text_edit_singleline | "Delay reason (why dates moved)" | same command | `lib.rs:1838-1839` |
| — button | button | "Apply field planning" | `apply_field_planning(id)` | `lib.rs:1840-1842` |
| Percent complete | text_edit_singleline | "Percent complete" (no % sign / 0-100 hint) | `SetActivityProgress`; buffer in percent, stored basis points | `lib.rs:1846-1847`, `lib.rs:658-677` |
| Budget | text_edit_singleline | "Budget at completion" | `SetActivityBudget` (money, empty clears) | `lib.rs:1848-1849`, `lib.rs:679-697` |
| Actual cost | text_edit_singleline | "Actual cost" | `SetActivityActualCost` | `lib.rs:1850-1851`, `lib.rs:699-717` |
| Actual start | text_edit_singleline | "Actual start (YYYY-MM-DD)" | `SetActivityActualDates` | `lib.rs:1852-1853`, `lib.rs:719-747` |
| Actual finish | text_edit_singleline | "Actual finish (YYYY-MM-DD)" | same command | `lib.rs:1854-1855` |
| — button | button | "Apply" | `apply_edits()` (`lib.rs:583`) commits all fields above in order | `lib.rs:1857-1859` |
| Relationship picker | ComboBox id `relationship_select`, placeholder "Select relationship"; items "{pred} -> {succ}" | (heading "Relationships") | sets `selected_relationship`, loads lag buffer | `lib.rs:1863-1892` |
| Type | label | "Type: {label}" (from `LINK_TYPE_LABELS`, `lib.rs:157-162`) | display only | `lib.rs:1899-1903` |
| Lag | text_edit_singleline | "Lag (working days)" | `SetRelationshipLag` via "Apply lag" button | `lib.rs:1904-1917` |
| Predecessor | ComboBox id `link_predecessor`, placeholder "Select activity" | "Predecessor" | buffers only | `lib.rs:1932-1943` |
| Successor | ComboBox id `link_successor`, placeholder "Select activity" | "Successor" | buffers only | `lib.rs:1949-1960` |
| Link type | ComboBox id `link_type` | "Type"; options "Finish → Start (FS)", "Start → Start (SS)", "Finish → Finish (FF)", "Start → Finish (SF)" (`lib.rs:157-162`) | buffers only | `lib.rs:1961-1973` |
| Link lag | text_edit_singleline | "Lag (working days)" | buffers only | `lib.rs:1974-1975` |
| — button | button | "Add link" | `add_link_from_form()` (`lib.rs:534`); creates `CreateRelationship` | `lib.rs:1976-1978` |
| Hint | label | "This form is the keyboard path; dragging between Gantt bars is a shortcut." | — | `lib.rs:1922-1925` |

**No field exists for** activity `location` (model `model:683`, command
`SetActivityLocation` `commands:770`), WBS (`model:618`), or activity
create/delete.

### 5.2 Split Gantt panel (right, id `workbench_gantt`, default 420 px, min 300, resizable) — `lib.rs:1985-2011`

Rendered only when `show_split_gantt` is true; calls the same `render_gantt`
(`lib.rs:3556`) as the Gantt view; on change bumps `recompute()`
(`lib.rs:2006-2010`).

### 5.3 Activity table (central panel) — `lib.rs:2019-2233`

| Element | Detail | Code |
|---|---|---|
| Error state | "Schedule error: {error}" in red | `lib.rs:2020-2022` |
| Columns | `#` (fixed 36 px, always visible, not sortable — `lib.rs:2030-2031`), then any of "Activity" (flex), "Dur" 60, "Start" 96, "Finish" 96, "Float" 56, "Critical" 64 (widths `lib.rs:114-123`) | `lib.rs:2030-2048` |
| Sticky header | sort buttons per column; active column appends "▲" / "▼" (`lib.rs:2063-2071`); `#` is a weak Label | `lib.rs:2052-2082` |
| Sorting | click header: new key → ascending; same key → toggle direction (`lib.rs:2221-2229`); sort cache `refresh_sorted` (`lib.rs:1245-1308`) | |
| Filtering | **none** — no text filter, no critical-only filter | — |
| Empty state | "No activities yet." | `lib.rs:2086` |
| Row hit area | whole row width is an invisible click target (`Sense::click`) | `lib.rs:2113-2123` |
| Name cell | Button with `.selected(selected).frame(selected).truncate()` — selection highlight visual | `lib.rs:2138-2152` |
| `#` cell | row index, monospace, weak | `lib.rs:2126-2136` |
| Dur/Start/Finish/Float cells | monospace Labels | `lib.rs:2154-2198` |
| Critical cell | "Yes" / "No" (no colour) | `lib.rs:2200-2208` |
| Virtualisation | `ScrollArea::show_rows`, row height 20 px; `scroll_to_selection` offsets to selection minus 120 px after arrow-key moves | `lib.rs:2091-2098`, reset `lib.rs:2233` |
| Row click | `select(id)` (`lib.rs:437`) loads all inspector buffers | `lib.rs:2230-2232` |

---

## 6. Gantt view — `lib.rs:2235-2255` + `render_gantt` `lib.rs:3556-3881`

| Element | Detail | Code |
|---|---|---|
| Screen-reader summary | "Gantt chart: {n} activities, {m} critical, {start} to {finish}. Red bars (marked !) are critical; the accent fill is recorded progress. The Table view lists the same data as text." | `lib.rs:3543-3554`, drawn `lib.rs:3585`, `WidgetInfo` `lib.rs:3618-3620` |
| Zoom row | label "Zoom"; buttons "−" (tooltip "Zoom out (Ctrl+Minus)"), "+" (tooltip "Zoom in (Ctrl+Plus)"), "Reset" (tooltip "Reset zoom (Ctrl+0)"); live "{pct}%" label | `lib.rs:3586-3610` |
| Zoom range | clamp 2.0–60.0 px/day, reset 14.0 (= 100 %) | `lib.rs:3593`, `lib.rs:3600`, `lib.rs:3607` |
| Canvas | `ScrollArea::both` + `allocate_exact_size` with `Sense::click_and_drag` | `lib.rs:3613-3617` |
| Weekly gridlines | every 5 working days, hairline `grid` colour | `lib.rs:3627-3649` |
| Today line | 1.5 px accent vertical line if today within range | `lib.rs:3650-3658` |
| Timeline header ticks | date labels every `tick_step` days (10 if px/day < 4, 5 if < 10, else 1); 6 px tick marks, gray | `lib.rs:3708-3736` |
| Bars | critical = palette.critical, else palette.normal; 2 px rounding; min width 6 px | `lib.rs:3747-3764` |
| Progress overlay | accent fill proportional to `percent_complete` | `lib.rs:3769-3778` |
| Bar label | "{name}" centered, white, 10 px; critical bars prefixed "! " | `lib.rs:3780-3791` |
| Dependency arrows | gray elbow lines; anchor edges depend on link type (FS/SS/FF/SF) | `lib.rs:3798-3855` |
| Drag-to-link | drag from any point starts a link; yellow preview line from predecessor finish edge to cursor; on release over another row creates an **FS lag-0** relationship | `lib.rs:3660-3697`, creation `lib.rs:3858-3879` |
| Ctrl+wheel zoom | multiplies `pixels_per_day` by `1 + scroll*0.001`, clamped | `lib.rs:3700-3706` |
| Errors | "Schedule error: {error}" (also in split panel) | `lib.rs:3572-3575` |
| Status messages | "Cannot add link: cycle detected" (`lib.rs:3870`), "Cannot add link" (`lib.rs:3876`) | |
| **Not possible** | bars are not clickable/selectable; no hover tooltip per bar; no bar dragging to move dates; drag link type/lag is fixed FS/0 | — |

---

## 7. Resources view — `lib.rs:2256-2445`

### 7.1 Right panel (id `resource_inspector`, default 300 px)

Headings: "Resource" (`lib.rs:2260`), "Leveling" (`lib.rs:2291`).

| Control | Type | Label | Action | Code |
|---|---|---|---|---|
| Name | text_edit_singleline | "Name" | `RenameResource` / `CreateResource` | `lib.rs:2261-2262`, `lib.rs:764-805` |
| Kind | ComboBox id `resource_kind`; options "Labour", "Equipment", "Material" (`lib.rs:154`) | "Kind" | `SetResourceKind` | `lib.rs:2263-2274` |
| Capacity | text_edit_singleline | "Capacity (units per working day)" | `SetResourceCapacity` / creation (default "1", `lib.rs:335`) | `lib.rs:2275-2276` |
| Button | button | "Add resource" | `create_resource()` (`lib.rs:808`) | `lib.rs:2278-2280` |
| Button | button, only when a resource is selected | "Apply changes" | `apply_resource_edits()` (`lib.rs:764`) | `lib.rs:2281-2284` |
| Button | button, only when selected | "Delete resource" | `delete_selected_resource()` (`lib.rs:972`) | `lib.rs:2285-2288` |
| Button | button | "Preview leveling" | `preview_leveling()` (`lib.rs:1029`) | `lib.rs:2292-2294` |
| Button | button | "Apply leveled dates" | `apply_leveling()` (`lib.rs:1062`) — writes `StartNoEarlierThan` constraints | `lib.rs:2295-2297` |
| Report | label | `leveling_report` text (see §13) | — | `lib.rs:2298-2300` |

### 7.2 Central panel

| Element | Detail | Code |
|---|---|---|
| Heading | "Resource dictionary" | `lib.rs:2318` |
| Empty state | "No resources yet. Add one in the panel on the right." | `lib.rs:2319-2321` |
| Table columns | "Name", "Kind", "Capacity", "Assigned", "Over-allocated days" (Grid `resource_rows`, striped) | `lib.rs:2322-2331` |
| Name cell | `selectable_label`, selected state when chosen; click → `select_resource()` | `lib.rs:2333-2337` |
| Over-allocation cell | "0" plain; > 0 in dark red `#C0392B` | `lib.rs:2349-2360` |
| Error state | "Schedule error: {error}" | `lib.rs:2314-2316` |
| Heading | "Usage" | `lib.rs:2366` |
| Usage table | per-date rows "{units} / {capacity}"; over-capacity red; max height 160 px scroll (Grid `resource_usage_rows`) | `lib.rs:2367-2403` |
| Usage empty | "This resource is not assigned to any scheduled activity." | `lib.rs:2370-2372` |
| Usage idle | "Select a resource to see its daily usage." | `lib.rs:2404-2406` |
| Heading | "Assignments" | `lib.rs:2409` |
| Assignment hint | "Toggle '{resource name}' on an activity to assign or remove it." | `lib.rs:2411-2417` |
| Assignment table | per activity: `selectable_label` (filled = assigned) → selects activity **and** `toggle_assignment()`; plus duration, start, finish (or "-") | `lib.rs:2418-2439` |
| Assignments idle | "Select a resource to assign it to activities." | `lib.rs:2440-2442` |

No sorting or filtering on either table.

---

## 8. Baselines view — `lib.rs:2446-2603`

### 8.1 Right panel (id `baseline_inspector`, default 300 px)

| Control | Type | Label | Action | Code |
|---|---|---|---|---|
| Heading | — | "Baselines" | — | `lib.rs:2450` |
| Name | text_edit_singleline | "Name for the new baseline" (empty ⇒ auto "Baseline {n+1}", `lib.rs:1098-1103`) | `capture_baseline_now()` | `lib.rs:2451-2455`, `lib.rs:1097-1117` |
| Button | button | "Capture baseline" | `CaptureBaseline` command | `lib.rs:2453-2455` |
| List | selectable labels | "{name} — {created_at}"; empty state "No baselines captured yet." | sets `selected_baseline` | `lib.rs:2458-2468` |
| Button | button, **disabled without selection** (`add_enabled`) | "Delete baseline" | `delete_selected_baseline()` → `RemoveBaseline` | `lib.rs:2469-2475`, `lib.rs:1224-1238` |
| Status date | text_edit_singleline | "Status date (YYYY-MM-DD)" (default today, `lib.rs:339`) | drives EVM panel | `lib.rs:2477-2481` |

### 8.2 Central panel

| Element | Detail | Code |
|---|---|---|
| Heading | "Earned value" | `lib.rs:2499` |
| Error state | "Status date must be written as YYYY-MM-DD" in red | `lib.rs:2501-2506` |
| EVM grid (`evm_rows`, striped) | label/value rows: "Budget at completion", "Planned value", "Earned value", "Actual cost", "Schedule variance", "Cost variance", "Schedule performance index", "Cost performance index" ("n/a" when undefined, `lib.rs:3463-3468`) | `lib.rs:2508-2538` |
| Heading | "Variance against the selected baseline" | `lib.rs:2543` |
| No baseline | "Capture or select a baseline to compare against." | `lib.rs:2545-2547` |
| Legend | "Variances are whole calendar days; a positive value is later or longer than planned." | `lib.rs:2549-2551` |
| Variance table columns | "Activity", "Start", "Base start", "ΔStart", "Finish", "Base finish", "ΔFinish", "Dur", "Base dur", "ΔDur" | `lib.rs:2555-2566` |
| Activity cell | `selectable_label` "{id}  {name}", selected when it is the current selection; click selects | `lib.rs:2569-2573` |
| Variance cells | signed text (`variance_text`, `lib.rs:3378-3384`); colours red/green per theme (`variance_color`, `lib.rs:3401-3409`), gray for n/a | `lib.rs:2576-2595` |

---

## 9. Calendars view — `lib.rs:2604-2696`

### 9.1 Right panel (id `calendar_inspector`, default 300 px)

| Control | Type | Label | Action | Code |
|---|---|---|---|---|
| Heading | — | "Calendars" | — | `lib.rs:2608` |
| Name | text_edit_singleline | "Name" | `RenameCalendar` / `CreateCalendar` | `lib.rs:2609-2610` |
| Weekday checkboxes ×7 | checkboxes | "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun" (default Mon–Fri, `lib.rs:342`) | `SetCalendarDefinition` via `build_calendar` (`lib.rs:951-969`) | `lib.rs:2611-2619` |
| Holidays | text_edit_singleline | "Holidays (comma-separated YYYY-MM-DD)" | same | `lib.rs:2620-2621` |
| Button | button | "Add calendar" | `create_calendar()` (`lib.rs:853`) | `lib.rs:2623-2625` |
| Button | button, when selected | "Apply changes" | `apply_calendar_edits()` (`lib.rs:874`) | `lib.rs:2626-2629` |
| Button | button, when selected | "Delete calendar" | `delete_selected_calendar()` (`lib.rs:918`) — refuses while in use | `lib.rs:2630-2633` |
| Note | label | "A calendar cannot be deleted while an activity uses it; holidays and working days can be changed at any time." | — | `lib.rs:2634-2637` |

### 9.2 Central panel

| Element | Detail | Code |
|---|---|---|
| Heading | "Named calendars" | `lib.rs:2643` |
| Empty state | "No named calendars yet; the project uses one shared working week." | `lib.rs:2644-2648` |
| Table columns | "Calendar", "Working days" ("{n}/7"), "Holidays" (count), "Activities" (count) | `lib.rs:2649-2676` |
| Name cell | `selectable_label` → `select_calendar()` | `lib.rs:2658-2664` |
| Detail block | heading = calendar name; "Calendar {id}"; "No holiday exceptions." or one date per line | `lib.rs:2679-2691` |
| Idle | "Select a calendar to see its holidays." | `lib.rs:2692-2694` |

The Inspector's per-activity calendar combo (§5.1) is the only place an
activity is bound to a calendar.

---

## 10. Audit view — `lib.rs:2697-2788`

Central panel only (no side panel).

| Element | Detail | Code |
|---|---|---|
| Heading | "Audit" | `lib.rs:2702` |
| Description | "Schedule-quality checks over the current programme, following public DCMA and GAO practice. Findings are data, not errors: fix what matters and re-run." | `lib.rs:2703-2707` |
| Status date | text_edit_singleline, label "Status date (YYYY-MM-DD, optional)" | `lib.rs:2710-2711` |
| Button | button | "Run audit" — empty ⇒ clear status date; else parse; invalid ⇒ status message "Status date must be YYYY-MM-DD" | `lib.rs:2712-2724` |
| Unschedulable | "The programme cannot be scheduled, so there is nothing to audit; fix the scheduling error shown in the table." | `lib.rs:2726-2732` |
| Summary | "{pass} pass, {fail} fail, {not applicable} not applicable" | `lib.rs:2733-2745` |
| Check rows (ScrollArea) | monospace check code; outcome word "[pass]" / "[FAIL]" / "[not applicable]"; measured value; summary; "Threshold: {t}" | `lib.rs:2747-2766` |
| Activity finding | `selectable_label` "{name} ({id})" → `select()` + jump to Table | `lib.rs:2767-2778` |
| Relationship finding | label "Relationship {id}: {detail}" (not clickable) | `lib.rs:2779-2781` |
| Auto-refresh | re-runs when `audit_report` is None or `edit_version` changed | `lib.rs:2699-2701`, `run_audit` `lib.rs:417-435` |

---

## 11. Lookahead view — `lib.rs:2789-3045`

Central panel only.

| Element | Detail | Code |
|---|---|---|
| Heading | "Lookahead" | `lib.rs:2791` |
| Description | "The rolling short-interval window over the master schedule: …" | `lib.rs:2792-2797` |
| Window start | text_edit_singleline "Window starts (YYYY-MM-DD)" (default today) | `lib.rs:2800-2801` |
| Length | 4 inline `selectable_value` options "2 w", "4 w", "6 w", "12 w" (label "Length") — flat text look, no button chrome | `lib.rs:2802-2809` |
| Invalid date | "The window start must be a date in YYYY-MM-DD format." | `lib.rs:2813-2816` |
| Unschedulable | "The programme cannot be scheduled, so there is no lookahead; fix the scheduling error shown in the table." | `lib.rs:2817-2823` |
| Count label | "{n} activities between {from} and {window_end}" | `lib.rs:2825-2831` |
| Empty state | "Nothing is scheduled in this window yet; move the window or add work in the table." | `lib.rs:2833-2838` |
| Table columns | "Activity", "Start", "Finish", "State", "Owner", "Promised", "Critical" (Grid `lookahead_rows`, striped) | `lib.rs:2840-2850` |
| Activity cell | `selectable_label` "{name} ({id})" → select + jump to Table | `lib.rs:2851-2856` |
| State cell | "In progress" / "Upcoming" | `lib.rs:2859-2863` |
| Owner cell | owner or "-" | `lib.rs:2864` |
| Promised cell | "Promised" or "-" | `lib.rs:2865-2866` |
| Critical cell | "Yes" / "No" | `lib.rs:2867` |
| Publish snapshot heading | "Publish snapshot"; field "Name"; button "Publish" | `lib.rs:2875-2909` |
| Publish messages | "Snapshot name must not be empty." / "Cannot publish snapshot." / "Lookahead snapshot published." | `lib.rs:2881-2906` |
| Published snapshots | heading "Published snapshots"; empty "No snapshots published yet."; rows: selectable name + "{w} weeks from {start}" + "captured {date}" | `lib.rs:2912-2928` |
| Compare section | heading "Compare with snapshot"; snapshot summary line; per-entry "start {±}d / finish {±}d"; "{name} ({id}) — no longer in window" + "removed"; "{name} ({id}) — new in window" + "added" | `lib.rs:2937-2996` |
| Button | button | "Clear comparison" — deselects snapshot | `lib.rs:2998-3000` |
| Location clashes | heading "Location clashes"; empty: "No activities share a location on overlapping days. Set a location in the inspector to include an activity in the check."; rows: strong location, date, hyperlink "{first_name} ({first})", "clashes with", hyperlink "{second_name} ({second})" → jump to Table | `lib.rs:3005-3037` |
| Footer note | "Select an entry to open the activity in the table. The window comes from the master schedule, so editing dates there moves the lookahead too." | `lib.rs:3040-3044` |

**Dead end:** the inspector has no Location field, so the "Set a location in
the inspector" instruction cannot be followed in the GUI (see §19).

---

## 12. Diary view — `lib.rs:3047-3166`

Central panel only.

| Element | Detail | Code |
|---|---|---|
| Heading | "Daily diary" | `lib.rs:3049` |
| Description | "Dated site records linked to the plan: …" | `lib.rs:3050-3055` |
| Date | text_edit_singleline "Date (YYYY-MM-DD)" (default today) | `lib.rs:3058-3059` |
| Activity link | singleline, width 56, label "Activity id (optional)" (raw numeric id, no picker) | `lib.rs:3060-3064` |
| Window link | singleline, width 96, label "Window start (optional)" | `lib.rs:3065-3069` |
| Note | multiline TextEdit, 3 rows, label "Note" | `lib.rs:3071-3074` |
| Button | button | "Add note" → `CreateDailyNote` | `lib.rs:3075-3116` |
| Messages | "Diary note added." / "Cannot add the note; check the content." / "The date (and window start, if given) must be YYYY-MM-DD." | `lib.rs:3103-3113` |
| Notes list | heading "Notes"; sorted newest first; empty "No diary notes yet."; rows: monospace date, content, "→ activity {id}" when linked, "→ window from {date}" when linked, per-row button "Delete" | `lib.rs:3118-3153` |
| Delete | `DeleteDailyNote` → "Diary note deleted." | `lib.rs:3155-3165` |

No note editing (`UpdateDailyNote` exists at `commands:1639` but is unused),
no filter, no click-through from a note to its activity.

---

## 13. Production view — `lib.rs:3168-3173` — **STUB**

The entire view:

```rust
ui.heading("Production tracking");
ui.label("Record daily actual progress against the schedule.");
```

Nothing else. Gap inventory for this view:

- Dead state buffers: `production_activity_buffer`, `production_date_buffer`,
  `production_percent` declared `lib.rs:280-282`, initialised `lib.rs:370-372`,
  never read by any widget.
- Ready but unused commands: `RecordProduction` (`commands:1454`),
  `UndoRecordProduction` (`commands:1535`), `DeleteProduction`
  (`commands:1591`), `RestoreProduction` (`commands:1616`).
- Ready model: `ProductionEntry` (`model:1253`), `project.production_entries`
  (`model:803`), `next_production_entry_id` (`model:899`).
- No form, no table of recorded entries, no empty state, no error state.
- Unreachable via keyboard (see §1).

---

## 14. Dialogs / windows

All three use `egui::Window` and block view shortcuts while open
(`dialog_open`, `lib.rs:1421-1422` — note the view-tab shortcut block above it
is **not** gated, §19).

### 14.1 Crash report — `lib.rs:3176-3222`, helpers `crash.rs`

- Opens **automatically at startup** when a crash log exists
  (`lib.rs:303-304`, `show_crash_dialog`), and via "Report a bug…".
- Window "Crash report", non-collapsible, resizable, close "X".
- Copy: "Cadence found a crash log from a previous run. Nothing has been sent anywhere."
- "Preview" heading + scrollable monospace log (max height 220).
- Field: multiline "Optional notes (what were you doing?)" (`crash_notes`).
- Buttons: "Open GitHub issue" (builds pre-filled URL `crash.rs:82-88`,
  opens browser tab, clears log + notes, closes), "Copy details"
  (copies `issue_body` to clipboard), "Dismiss" (clears log, closes).
- Crash log location `$TMP/cadence-crash.log` (`crash.rs:20-22`); written by
  the panic hook in `crates/app/src/crash_report.rs:31-49`.

### 14.2 Updates — `lib.rs:3224-3266`, logic `update_check.rs`

- Window "Updates", non-collapsible, non-resizable, close "X".
- Version line "Cadence {version}".
- State-dependent copy: NotChecked → "No update check has run yet.";
  UpToDate → "You are running the latest release."; UpdateAvailable →
  "Update available: {version}" + button "Open release page" (opens URL in new
  tab); Unavailable → "Update check unavailable: {message}".
- "Checking…" shown while the background thread runs; otherwise button
  "Check now" → `start_update_check()` (`lib.rs:386-396`).
- Privacy note: "The check sends one GET request to the public GitHub Releases API. …".
- Button label in toolbar reflects availability (§3.5). Status polled every
  250 ms via `request_repaint_after` (`lib.rs:1322-1335`).

### 14.3 Keyboard shortcuts — `lib.rs:3268-3316`

- Window "Keyboard shortcuts", opened by F1 or the "Shortcuts" button.
- Striped grid of 10 rows (keys → action):

| Keys (as displayed) | Action (as displayed) |
|---|---|
| Ctrl+Z | Undo |
| Ctrl+Y or Ctrl+Shift+Z | Redo |
| Ctrl+1 … Ctrl+8 | Switch view: Table, Gantt, Resources, Baselines, Calendars, Audit, Lookahead, Diary |
| Ctrl+D | Toggle light/dark theme |
| Ctrl+Plus / Ctrl+Minus | Zoom the Gantt timeline in or out |
| Ctrl+0 | Reset the Gantt zoom |
| Tab / Shift+Tab | Move focus between controls |
| Space / Enter | Activate the focused button or row |
| Up / Down | Select the previous/next activity in the Table view |
| F1 | Open this window |

- Footer: "Every task has a pointer-free path: …".
- **Ctrl+S and Ctrl+O exist in code but are missing from this reference** (§19).

---

## 15. File dialogs (native `rfd`)

| Trigger | Dialog | Filter / default name | Code |
|---|---|---|---|
| "Open…" / Ctrl+O | pick_file | "Cadence project" `*.cadence` | `lib.rs:1184-1191` |
| "Save" with no path / "Save as…" | save_file | "Cadence project" `*.cadence`, default `{project}.cadence` or current file name | `lib.rs:1167-1181` |
| "Print report…" | save_file | "PDF document" `*.pdf`, default `{project}.pdf` | `lib.rs:1122-1129` |
| "Print Gantt…" | save_file | "PDF document" `*.pdf`, default `{project}-gantt.pdf` | `lib.rs:1622-1626` |
| "Save report" | none — writes `{project}.report.md` into the **current working directory** silently | — | `lib.rs:1214-1221` |

Cancelling any dialog changes nothing (`lib.rs:1127-1129` documented).

---

## 16. All keyboard shortcuts (implemented)

| Shortcut | Context | Action | Code |
|---|---|---|---|
| Ctrl+D | everywhere | toggle dark/light theme | `lib.rs:1337-1340` |
| Ctrl+S | everywhere (even with text focus) | `save_or_prompt()` | `lib.rs:1344-1346` |
| Ctrl+O | everywhere (even with text focus) | `open_dialog()` | `lib.rs:1347-1349` |
| Ctrl+Z | outside text fields | undo (`stack.undo`, recompute, `refresh_selection`) | `lib.rs:1353-1368` |
| Ctrl+Y | outside text fields | redo | `lib.rs:1357-1373` |
| Ctrl+Shift+Z | outside text fields | redo | `lib.rs:1357-1373` |
| Ctrl+1 … Ctrl+8 | everywhere — **not gated on open dialogs** | switch to Table/Gantt/Resources/Baselines/Calendars/Audit/Lookahead/Diary | `lib.rs:1379-1412` |
| Ctrl+9 | — | **not wired** (Production unreachable) | `lib.rs:1383-1391` |
| F1 | everywhere | shortcuts dialog | `lib.rs:1415-1417` |
| Ctrl+Plus / Ctrl+Equals | Gantt view, outside text fields, no dialog | zoom in ×1.25 | `lib.rs:1425-1445` |
| Ctrl+Minus | same | zoom out ÷1.25 | `lib.rs:1425-1445` |
| Ctrl+0 | same | reset zoom to 14 px/day | `lib.rs:1442-1444` |
| Ctrl+wheel | hovering Gantt canvas | smooth zoom | `lib.rs:3700-3706` |
| ArrowUp / ArrowDown | Table view, outside text fields, no popup, no dialog | select previous/next activity, scroll into view | `lib.rs:1449-1476` |
| Tab / Shift+Tab, Space / Enter | everywhere | egui built-in focus navigation & activation (documented in dialog) | egui defaults; `lib.rs:3293-3294` |
| Ctrl+Z inside a text field | text editing | egui built-in field undo (deliberately not app undo) | `lib.rs:1351-1356` |

---

## 17. Status messages (complete list of user-visible strings)

Rendered in the toolbar ("Status: …") and status bar (red). Set points:

Validation errors (inspector): "The activity name must not be empty."
(`lib.rs:592`); "Duration must be a whole number of working days."
(`lib.rs:623-625`); "A date constraint needs a date in YYYY-MM-DD format."
(`lib.rs:635-637`); "Progress must be a percentage between 0 and 100."
(`lib.rs:661-663`); "Budget must be a number." (`lib.rs:682`); "Actual cost
must be a number." (`lib.rs:702`); "Actual start must be a date in YYYY-MM-DD
format." (`lib.rs:722-724`); "Actual finish must be a date in YYYY-MM-DD
format." (`lib.rs:730-732`).

Links: "A link needs two different activities" (`lib.rs:191`); "Choose both a
predecessor and a successor" (`lib.rs:192`); "Lag must be a whole number of
working days" (`lib.rs:544`); "Added {link_type} link {pred} → {succ}"
(`lib.rs:570-573`); "Cannot add link: {error}" (`lib.rs:565`, `lib.rs:578`).

Resources: "Resource name must not be empty" (`lib.rs:811`); "Select an
activity first" (`lib.rs:992`); command errors surface verbatim
(`lib.rs:778-800`, `lib.rs:832`, `lib.rs:985`).

Leveling (report pane, not status bar): "The project cannot be scheduled"
(`lib.rs:1031`, `lib.rs:1064`); "The project cannot be leveled"
(`lib.rs:1068`); "Leveling failed: {error}" (`lib.rs:1057`); "{n}
over-allocated days remain after leveling" (`lib.rs:1042-1045`); "The schedule
is already free of over-allocation" (`lib.rs:1047-1048`); "Leveling delays {n}
activities, for example activity {id} by {d} working days ({a} to {b})"
(`lib.rs:1050-1054`, `describe_delay` `lib.rs:3333-3338`); "Applied leveling
to {n} activities" (`lib.rs:1093`).

Calendars: "Calendar name must not be empty" (`lib.rs:856`); "A calendar needs
at least one working day" (`lib.rs:953`); "\"{text}\" is not a valid date"
(`lib.rs:963`); "{n} activity(ies) use this calendar — clear their calendar
first" (`lib.rs:928-935`).

Files/reports: "Saved to {path}" (`lib.rs:1157`); "Cannot save: {error}"
(`lib.rs:1161`); "Opened {path}" (`lib.rs:1206`); "Cannot open: {error}"
(`lib.rs:1209`); "Report written to {path}" (`lib.rs:1218`); "Cannot write
{path}: {error}" (`lib.rs:1136`, `lib.rs:1219`); "Report printed to {path}"
(`lib.rs:1133`); "Gantt printed to {path}" (`lib.rs:1503`); "Cannot write
{path}: {error}" (`lib.rs:1507`).

Gantt drag: "Cannot add link: cycle detected" (`lib.rs:3870`); "Cannot add
link" (`lib.rs:3876`).

Audit: "Status date must be YYYY-MM-DD" (`lib.rs:2721-2723`).

Lookahead: "Snapshot name must not be empty." (`lib.rs:2882-2883`); "Cannot
publish snapshot." (`lib.rs:2904-2905`); "Lookahead snapshot published."
(`lib.rs:2901-2902`).

Diary: "Diary note added." (`lib.rs:3103`); "Cannot add the note; check the
content." (`lib.rs:3105-3107`); "The date (and window start, if given) must be
YYYY-MM-DD." (`lib.rs:3110-3113`); "Diary note deleted." (`lib.rs:3163`).

≈ 45 distinct message templates. Every `stack.execute` failure also surfaces
its command error string verbatim (e.g. engine cycle errors).

---

## 18. Visual states summary

| State | Where | Implementation |
|---|---|---|
| Selected tab / selected name cell | toolbar tabs, table name button | `.selected().frame()` — accent fill, `lib.rs:1563-1568`, `lib.rs:2140-2147` |
| Selected row | resource/assignment/baseline/calendar/lookahead/audit/clash `selectable_label`s | egui selection accent (`design.rs:185-187`) |
| Hover | all widgets | `widgets.hovered` fill `lib`/`design.rs:174-178` |
| Active/pressed | all widgets | `design.rs:179-183` |
| Disabled | "Print Gantt…" outside Gantt view (`lib.rs:1614-1618`); "Delete baseline" without selection (`lib.rs:2469-2472`) | `add_enabled` (greyed) |
| Conditional visibility | "Apply changes"/"Delete …" buttons only with a selection; "Constraint date" only for dated constraints; lag block only with a selected relationship; calendar detail only with a selection | `lib.rs:1770`, `lib.rs:1894`, `lib.rs:2281`, `lib.rs:2626`, `lib.rs:2679` |
| Error text | schedule errors and status messages red (`Color32::RED` / `#C0392B`); variance red/green per theme; over-allocation red | `lib.rs:2021`, `lib.rs:1729`, `lib.rs:2356-2359`, `lib.rs:2390-2395`, `lib.rs:2502-2505`, `variance_color` `lib.rs:3401-3409` |
| Drag feedback | yellow preview line while linking | `lib.rs:3679-3683` |
| Progress | accent overlay on Gantt bars | `lib.rs:3769-3778` |
| Focus | Tab/Space/Enter via egui defaults; selection stroke accent | `design.rs:185-187`, `lib.rs:3293-3294` |

Context menus (right-click): **none anywhere** — `context_menu` never used.
Sliders / DragValues: none. Tooltips: 13 `on_hover_text` calls (10 toolbar +
3 Gantt zoom).

---

## 19. Gaps & inconsistencies

1. **Production view is a stub** (`lib.rs:3168-3173`): heading + one sentence.
   `RecordProduction`/`DeleteProduction` commands (`commands:1454`,
   `commands:1591`) and the `production_*` buffers (`lib.rs:280-282`) are dead
   code; no form, no entries table, no empty/error state. The view is also
   **unreachable by keyboard** (Ctrl+9 not wired, `lib.rs:1383-1391`).
2. **No way to create or delete an activity in the GUI.** `CreateActivity`,
   `DeleteActivity`, `RestoreActivity` (`commands:177`, `commands:218`,
   `commands:297`) are unused by the UI; projects can only be opened from
   files made by the CLI. The empty table ("No activities yet.",
   `lib.rs:2086`) has no affordance to add the first activity.
3. **Links can be added but never removed or edited in type.** No
   `DeleteRelationship` usage (`commands:457`); no UI to change a link's type;
   lag only via the inspector. Gantt drag always creates FS lag-0
   (`lib.rs:3864`).
4. **Location clashes dead end** (`lib.rs:3008-3011`): the Lookahead view says
   "Set a location in the inspector", but the inspector has no Location field;
   `SetActivityLocation` (`commands:770`) and `Activity.location`
   (`model:683`) are unused by the GUI, so the feature can only fire on
   imported data.
5. **Toolbar Undo/Redo skip `refresh_selection()`** (`lib.rs:1659-1666`),
   unlike the Ctrl+Z path (`lib.rs:1364-1373`) — inspector buffers can show
   stale values after toolbar undo/redo.
6. **"Split Gantt: on/off" affects only the Table view**; toggling it in the
   Gantt view visibly does nothing, and the button has no tooltip
   (`lib.rs:1572-1581`).
7. **Ctrl+1..8 act behind open dialogs** despite the comment "A dialog owns
   the keyboard while it is open" (`lib.rs:1419-1422`): the view-switch block
   (`lib.rs:1379-1412`) and F1 (`lib.rs:1415-1417`) run before/unconditioned by
   `dialog_open`. The comment "Ctrl+1..Ctrl+5" (`lib.rs:1378`) is also stale.
8. **Shortcuts dialog omits Ctrl+S / Ctrl+O** (`lib.rs:3279-3300`) even though
   they are implemented (`lib.rs:1344-1349`); it also documents Ctrl+1 … Ctrl+8
   while 9 views exist and Production has no key.
9. **No filtering/searching anywhere**: the activity table has sorting only;
   Resources, Baselines, Calendars, Lookahead, Diary tables have neither sort
   nor filter; no WBS grouping/filter despite `Activity.wbs` (`model:618`).
10. **Gantt interaction limits**: bars are not clickable/selectable (selection
    only via Table or arrow keys), no per-bar tooltip, bar labels can overflow
    narrow bars (centered white text, `lib.rs:3785-3791`), and there is no way
    to move a bar by dragging (dates only via inspector).
11. **Published lookahead snapshots cannot be deleted** — `RemoveLookaheadSnapshot`
    (`commands:1308`) unused; snapshot list has no per-item delete.
    Diary notes cannot be edited — `UpdateDailyNote` (`commands:1639`) unused.
12. **Duplicated, inconsistent status presentation**: toolbar copy is plain
    ("Status: …", `lib.rs:1667-1669`) while the status-bar copy is red
    (`lib.rs:1723-1731`); both render simultaneously; no toasts, no auto-clear,
    no history.
13. **"Save report" writes into the CWD without a dialog** (`lib.rs:1214-1221`)
    while "Print report…" opens one — inconsistent file handling for two
    adjacent buttons.
14. **Lookahead "Length" options ("2 w" … "12 w") are bare `selectable_value`
    texts** (`lib.rs:2803-2809`) — no button/checkbox chrome, so they do not
    look interactive; same pattern as the Lookahead/audit jump labels.
15. **Percent-complete and money fields lack unit hints** ("Percent complete"
    has no % or 0-100 hint, `lib.rs:1846`; money shows bare decimals,
    `format_money` `lib.rs:3412-3415`); diary "Activity id (optional)" expects
    a raw numeric id with no picker or validation feedback beyond status text.
16. **Unused inspector capacity**: no fields for `Activity.wbs` or activity
    deletion; the "Add relationship" hint (`lib.rs:1922-1925`) is the only
    documentation that dragging exists; the Resources "Assignments" grid
    conflates selection with assignment toggling (a click both selects the
    activity and flips assignment, `lib.rs:2424-2428`).

*Inventory totals: 9 views; 21 toolbar buttons + 1 menu (7 items); 2 status-bar
controls; 4 side panels with 33 inputs + 14 buttons; 9 data tables/grids; 3
dialogs; 4 native file dialogs; ~45 status-message templates; 16 keyboard
bindings (+2 egui built-ins documented); 13 tooltips; 0 context menus.*
