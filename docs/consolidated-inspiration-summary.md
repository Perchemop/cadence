# Consolidated Inspiration Summary — MS Project, Primavera P6, Aphex, Schedule Audit Tools, AI Assistants

**Status:** Clean-room record of observable behaviours from public sources only. No copied code, UI, icons, strings, layouts, or algorithms. Source URLs provided for every feature.

**Source files:**
- `docs/research/ms-project-capabilities.md`
- `docs/research/primavera-p6-capabilities.md`
- `docs/research/aphex-integration.md`
- `docs/research/schedule-audit.md`
- `docs/research/ai-integration.md`
- `docs/research/gui-inspiration-audit.md`
- `docs/ui-patterns.md`

---

## Table of Contents

1. [Features: Scheduling Core](#1-features-scheduling-core)
2. [Features: WBS & Structure](#2-features-wbs--structure)
3. [Features: Gantt View](#3-features-gantt-view)
4. [Features: Task/Activity Table](#4-features-taskactivity-table)
5. [Features: Baselines & Progress](#5-features-baselines--progress)
6. [Features: Resources](#6-features-resources)
7. [Features: Earned Value](#7-features-earned-value)
8. [Features: Schedule Audit & Quality](#8-features-schedule-audit--quality)
9. [Features: Lookahead & Field Planning](#9-features-lookahead--field-planning)
10. [Features: AI Assistance](#10-features-ai-assistance)
11. [UI Patterns: Workbench Shell](#11-ui-patterns-workbench-shell)
12. [UI Patterns: Views & Navigation](#12-ui-patterns-views--navigation)
13. [UI Patterns: Editing](#13-ui-patterns-editing)
14. [UI Patterns: Status & Feedback](#14-ui-patterns-status--feedback)
15. [Settings & Keyboard Shortcuts](#15-settings--keyboard-shortcuts)
16. [Data Flow & Persistence](#16-data-flow--persistence)
17. [Edge Cases & Error Handling](#17-edge-cases--error-handling)
18. [Sources](#18-sources)

---

## 1. Features: Scheduling Core

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| CPM Forward Pass | MS Project, P6, PMBOK | Engine calculation; visible in dates | Calculates Early Start/Finish dates propagating through predecessor relationships | https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project |
| CPM Backward Pass | MS Project, P6, PMBOK | Engine calculation; visible in dates | Calculates Late Start/Finish dates propagating backwards from project end | https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project |
| Total Float Calculation | MS Project, P6 | Engine; shown in task table | `LS - ES = Total Float` or `LF - EF = Total Float` per activity | https://support.microsoft.com/en-us/project/understanding-float-in-project |
| Free Float Calculation | MS Project | Engine; shown in task table | `ES of successor - EF of activity` - float available without affecting successors | https://support.microsoft.com/en-us/project/understanding-float-in-project |
| Critical Set Identification | MS Project, P6 | Engine; shown in Gantt red, table column | Activities with Total Float = 0 are on the critical path | https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project |
| Cycle Detection | Standard graph algorithms | Engine; error dialog | Returns clear error with path showing which activities form the cycle | Original concept; not MS Project/P6 specific |
| FS/SS/FF/SF Relationships | MS Project, P6, PMBOK | Gantt bars, table columns | Finish-to-Start: successor can't start before predecessor finishes; Start-to-Start: both can start together; Finish-to-Finish: both must finish together; Start-to-Finish: successor can't finish before predecessor starts | https://support.microsoft.com/en-us/project/create-lag-or-leads-in-microsoft-project |
| Signed Lag/Lead | MS Project, P6 | Relationship dialog | Positive lag delays successor; negative lag (lead) advances successor | https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm |
| Working-Day Calendars | MS Project, P6 | Calendar view (`Ctrl+5`) | Weekly Mon-Fri pattern with optional holiday exceptions per calendar | https://support.microsoft.com/en-us/project/create-and-use-resource-calendars-in-project |
| Named Calendars Assignment | P6 | Task inspector, table column | Each activity can be assigned a specific calendar from the dictionary | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| Multiple Calendars | P6 | Calendar management view | Global calendar + per-project calendars + per-activity calendars | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| Date Constraints | MS Project, P6 | Task inspector | SNET, SNLT, FNET, FNLT, ASAP, ALAP constraint types with visible date fields | https://support.microsoft.com/en-us/project/constraints-in-microsoft-project |

---

## 2. Features: WBS & Structure

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| WBS Hierarchy | MS Project, P6 | Table (WBS column), Gantt (indent bars) | Summary tasks roll up subtasks; durations aggregate; relationships cross levels | https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm |
| Summary Tasks | MS Project, P6 | Table, Gantt canvas | Zero-duration parent activities that group children | https://support.microsoft.com/en-us/project/summary-tasks-in-microsoft-project |
| WBS Codes | P6 | WBS table column | User-configurable alphanumeric codes showing project hierarchy | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| Activity Codes | P6 | Custom column in table | Predefined lists categorize activities for filtering/grouping | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| User-Defined Fields (UDFs) | P6 | Custom columns in table | Free-form text, number, date fields attached to activities | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |

---

## 3. Features: Gantt View

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Timeline Header | MS Project, P6 | Top of Gantt | Shows timescale with date ticks; weeks/quarters/zoom levels | https://support.microsoft.com/en-us/project/timescale-features-in-the-gantt-view |
| Task Bars | MS Project, P6 | Gantt canvas | Bars for each activity; width = duration; start/end on timeline | https://support.microsoft.com/en-us/project/gantt-chart-in-microsoft-project |
| Progress Overlay | MS Project, Asta | Bar fill | Darker fill inside bar shows percent complete | https://www.barbecana.com/schedule-inspector-2 |
| Dependency Arrows | MS Project | Between bars | Straight lines with arrowheads showing relationships; drag to create | https://support.microsoft.com/en-us/project/create-dependencies-in-project |
| Critical Path Bars | MS Project, P6 | Gantt bars | Red-colored bars identify critical activities | https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project |
| Today Line | MS Project | Vertical line | Dashed vertical line showing current date on timescale | https://support.microsoft.com/en-us/project/show-the-today-line-in-project |
| Weekly Grid Lines | MS Project | Vertical hairlines | Light vertical lines at week boundaries | https://support.microsoft.com/en-us/project/timescale-features-in-the-gantt-view |
| Milestone Diamonds | MS Project | Gantt | Diamond shape for zero-duration activities | https://support.microsoft.com/en-us/project/milestones-in-microsoft-project |
| Baseline Ghosts | MS Project | Overlay | Faded bars showing baseline dates for variance | https://support.microsoft.com/en-us/project/baselines-in-project |
| Zoom Controls | MS Project | Toolbar buttons, Ctrl+wheel | Zoom in/out on timescale; shows percentage read-out | https://support.microsoft.com/en-us/project/zoom-in-out-in-the-gantt-view |
| Pan View | MS Project | Click+drag on empty space | Pan horizontally/vertically when zoomed | https://support.microsoft.com/en-us/project/pan-the-gantt-view-in-project |
| Drag to Create Links | MS Project | Gantt bars | Click end of one bar, drag to another to create dependency | https://support.microsoft.com/en-us/project/create-dependencies-in-project |
| Lag Label Editing | MS Project | On arrow | Direct inline editing of lag/lead value on dependency line | https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm |

---

## 4. Features: Task/Activity Table

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Column Headers | MS Project | Table top | Click to sort ascending/descending; visual indicator shows sort order | https://support.microsoft.com/en-us/project/sorting-tasks-in-project |
| Row Numbers | MS Project | First column | Implicit sequential task ID numbers | https://support.microsoft.com/en-us/project/task- ids-in-project |
| Virtualised Rows | Cadence custom | Table body | Only visible rows rendered; smooth scrolling through thousands of activities | Original optimization pattern |
| Sticky Header | MS Project | Table top | Column headers remain visible while scrolling vertically | https://support.microsoft.com/en-us/project/scroll-in-the-task-sheet |
| Column Chooser | MS Project, P6 | Toolbar menu | Select which columns appear; saves user preference | https://support.microsoft.com/en-us/project/customize-the-task-sheet-view |
| Selection Highlighting | All | Row background | Selected row highlighted; selection persists across view switches | MS Project pattern |
| Inspector Edit | MS Project | Right panel | Edit activity properties in side panel; changes flow through command layer | https://support.microsoft.com/en-us/project edit-tasks-in-project |
| Filter Bar | P6 | Above table | Group, sort, filter activities by any column | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |

---

## 5. Features: Baselines & Progress

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Multiple Baselines | MS Project | Baselines view (`Ctrl+4`) | Up to 11 baseline sets with start/finish/work/cost fields each | https://learn.microsoft.com/en-us/answers/questions/5248498/set-baseline-options |
| Baseline Comparison | MS Project | Tracking Gantt | Ghosted bars show baseline vs current; variance columns show delta | https://support.microsoft.com/en-us/project/compare-two-versions-of-a-project |
| Variance Colors | MS Project | Cell background | Positive variance = green, negative = red (customizable) | MS Project pattern |
| Progress Entry | MS Project | Table column, inspector | Enter % complete, actual start/finish, remaining duration | https://support.microsoft.com/en-us/project/entering-progress-in-project |
| Status Date | All | Toolbar, Baselines view | Reference date for EVM calculations; can be set by user | https://support.microsoft.com/en-us/project/status-date-in-project |
| Progress Overlay | MS Project, Asta | Gantt bar fill | Light fill overlay shows % complete within bar | https://www.barbecana.com/schedule-inspector-2 |

---

## 6. Features: Resources

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Resource Dictionary | MS Project | Resources view (`Ctrl+3`) | List of resources with kind (work/material/cost), capacity, cost rate | https://support.microsoft.com/en-us/project/resources-in-project |
| Work Resources | MS Project | Resources view | Labor with hours per day, standard/overtime rates | https://support.microsoft.com/en-us/project/work-resources-in-project |
| Material Resources | MS Project | Resources view | Non-labor (concrete, steel) with quantity units | https://support.microsoft.com/en-us/project/material-resources-in-project |
| Per-Assignment Units | MS Project | Assignment table | Override default 100% with custom units (basis points in Cadence) | https://support.microsoft.com/en-us/project/assignment-units-in-project |
| Cost Rates | MS Project | Resource inspector | Standard, overtime, per-use rates configurable by date range | https://support.microsoft.com/en-us/project/cost-rate-tables-in-project |
| Usage Histogram | MS Project | Resource Usage view | Committed units per working day, time-phased | https://support.microsoft.com/en-us/project/resource-usage-view-in-project |
| Over-Allocation | MS Project, P6 | Resource view indicator | Shows days where committed > capacity | https://support.microsoft.com/en-us/project/overallocation-in-project |
| Resource Leveling | P6 | Resource view | Automatic/manual defer activities to resolve conflicts | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| Resource Codes | P6 | Custom column | Categorize resources for grouping/filtering | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |

---

## 7. Features: Earned Value

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| PV (Planned Value) | MS Project, PMBOK | Baselines view | Budgeted cost of work scheduled to date | https://support.microsoft.com/en-us/project/earned-value-in-project |
| EV (Earned Value) | MS Project, PMBOK | Baselines view | Budgeted cost of work actually completed to status date | https://support.microsoft.com/en-us/project/earned-value-in-project |
| AC (Actual Cost) | MS Project, PMBOK | Baselines view | Actual cost incurred to date | https://support.microsoft.com/en-us/project/earned-value-in-project |
| CV (Cost Variance) | MS Project, PMBOK | Baselines view | EV - AC; negative = over budget | https://support.microsoft.com/en-us/project/earned-value-in-project |
| SV (Schedule Variance) | MS Project, PMBOK | Baselines view | EV - PV; negative = behind schedule | https://support.microsoft.com/en-us/project/earned-value-in-project |
| CPI (Cost Performance Index) | MS Project, PMBOK | Baselines view | EV / AC; <1.0 = cost problems | https://support.microsoft.com/en-us/project/earned-value-in-project |
| SPI (Schedule Performance Index) | MS Project, PMBOK | Baselines view | EV / PV; <1.0 = schedule behind | https://support.microsoft.com/en-us/project/earned-value-in-project |
| EAC (Estimate at Completion) | MS Project, PMBOK | Baselines view | Forecasted total final cost | https://support.microsoft.com/en-us/project/earned-value-in-project |
| TCPI (To Complete Performance Index) | PMBOK | Baselines view | (BAC - EV) / (BAC - AC) for CPI target | PMBOK standard |
| VAC (Variance at Completion) | PMBOK | Baselines view | EAC - BAC; expected final variance | PMBOK standard |
| EVM Status Date | All | Baselines view | Date against which all EVM metrics are calculated | PMBOK standard |

---

## 8. Features: Schedule Audit & Quality

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Structure Checks | DCMA 14-point | Audit view (`Ctrl+6`) | Open starts/finishes, logic completeness | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Lead Detection | DCMA 14-point | Audit view | Flag negative lag relationships | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Lag Analysis | DCMA 14-point | Audit view | Flag positive lag > 5% threshold | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Relationship Types | DCMA 14-point | Audit view | Flag non-FS relationships (target 90%+ FS) | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Hard Constraints | DCMA 14-point | Audit view | Flag must-start/on or no-later-than dates | https://www.taradigm.com/how-to-use-check-schedule-report-in-primavera-p6 |
| High Float | DCMA 14-point | Audit view | Flag activities with float > 44 days | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Negative Float | DCMA 14-point | Audit view | Flag activities with negative float | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Long Duration | DCMA 14-point | Audit view | Flag activities > 44 working days | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Invalid Dates | DCMA 14-point | Audit view | Flag dates outside valid range | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Missed Tasks | DCMA 14-point | Audit view | Flag activities finishing after baseline | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Critical Path Test | DCMA 14-point | Audit view | Nudge critical activity; check finish moves | https://learning.deltek.com/bundle/Acumen_Help_System/page/DCMA_14_Point_Assessment_Metric_Descriptions.html |
| CPLI | DCMA 14-point | Audit view | Critical Path Length Index >= 0.95 | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| BEI | DCMA 14-point | Audit view | Baseline Execution Index >= 0.95 | https://docs.onplana.com/projects-tasks/check-schedule-quality/ |
| Out-of-Sequence Progress | SSI Schedule Analyzer | Audit view | Flag progress posted on activity with unfinished predecessor | https://scheduleanalyzer.com/sa_review.htm |
| Redundant Logic | SSI Schedule Analyzer | Audit view | Flag multiple paths to same successor | https://www.ssitools.com/helpandsuport/ssianalysishelp/ssi_schedule_integrity_audits.htm |
| Unnecessary Constraints | P6 | Audit view | Flag hard constraints on non-critical activities | https://www.taradigm.com/how-to-use-check-schedule-report-in-primavera-p6 |
| Comparison Tool | Acumen Fuse | Audit view (`Ctrl+6`) | Two version diff: added/removed activities, duration, relationship, constraint changes | https://www.deltek.com/products/delivery-assurance/ppm/acumen/fuse/ |

---

## 9. Features: Lookahead & Field Planning

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Rolling Window | Aphex | Lookahead view (`Ctrl+7`) | Filter activities to 2-12 week future window | https://www.aphex.co/updates/what-is-a-lookahead-plan |
| Owner Assignment | Aphex | Lookahead view | Assign responsible person to each lookahead activity | https://www.aphex.co/updates/what-is-a-lookahead-plan |
| Promise Flag | Aphex | Lookahead view | Mark activity as promised work; requires delay reason to move | https://www.aphex.co/updates/what-is-a-lookahead-plan |
| Delay Reason | Aphex | Lookahead dialog | Required field when moving a promised activity | https://www.aphex.co/product/plan-together |
| Snapshot Capture | Aphex | Lookahead view | Save named read-only version for comparison | https://www.aphex.co/product/plan-together |
| Snapshot Comparison | Aphex | Lookahead view | Compare current with previous snapshot: start/finish variance | https://blog.aphex.co/post/importingplans |
| Daily Diary | Aphex | Diary view (`Ctrl+8`) | Dated notes linked to activity or window | https://www.aphex.co/product/build-routine |
| Huddle Briefing | Aphex | Diary view | Structured daily coordination from live plan | https://blog.aphex.co/construction-scheduling-software-australia |
| Location Tag | Aphex | Activity inspector | Free-text location field for clash detection | https://www.aphex.co/product/plan-together |
| Location Clash | Aphex | Lookahead view | Flag when two activities share location on overlapping days | https://www.aphex.co/product/plan-together |
| Production Tracking | Aphex | Production view (`Ctrl+9`) | Record actual progress vs plan; chart plan vs actual | https://www.aphex.co/product/simplify-reporting |

---

## 10. Features: AI Assistance

| Feature | Inspiration Product | UI Location | Observable Behaviour | Source URL |
|---------|---------------------|-------------|---------------------|------------|
| Document Understanding | nPlan, Document Crunch | Upload dialog | Read PDF/email and extract schedule-relevant facts | https://www.documentcrunch.com/blog/ai-in-construction-benefits |
| WBS Generation | nPlan, Copilot for Project | Assistant panel | Propose work breakdown from scope description | https://www.nplan.io/ |
| Activity Draft | Copilot for Project | Assistant panel | Propose individual activities with durations | https://learn.microsoft.com/en-us/dynamics365/project-operations/project-management/copilot-features |
| Risk Mitigation | Copilot for Project | Assistant panel | Draft risk assessments from schedule data | https://learn.microsoft.com/en-us/dynamics365/project-operations/responsible-ai/copilot-for-project-faq |
| Status Report Draft | Copilot for Project | Assistant panel | Turn schedule metrics into narrative report | https://learn.microsoft.com/en-us/dynamics365/project-operations/project-management/copilot-features |
| What-If Scenarios | Planera | Assistant panel | Answer questions about float, critical path, risk | https://www.planera.io/post/best-ai-construction-scheduling-tools |
| Optimization | ALICE | Backend | Generate multiple schedule options against constraints | https://www.alicetechnologies.com/home |

---

## 11. UI Patterns: Workbench Shell

| Pattern | Inspiration Product | Implementation | Observable Behaviour | Source |
|---------|---------------------|----------------|---------------------|--------|
| View Tabs | MS Project, P6 | Top toolbar tab strip | Row of named view tabs; one active; click to switch | https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm |
| Splitter Pane | MS Project | Between table and Gantt | Draggable divider; tables scroll with Gantt vertically | https://support.microsoft.com/en-us/project/gantt-chart-in-microsoft-project |
| Status Bar | MS Project | Bottom of window | Task counts, status date, zoom %, activity counts | https://support.microsoft.com/en-us/project/status-bar-in-project |
| Menu Bar | P6 | Top of window | File, Edit, View, Tools, etc. | https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm |

---

## 12. UI Patterns: Views & Navigation

| Pattern | Inspiration Product | Implementation | Observable Behaviour | Source |
|---------|---------------------|----------------|---------------------|--------|
| Keyboard View Switch | MS Project | Ctrl+1 through Ctrl+9 | Number keys switch to corresponding view | MS Project standard |
| View Mode Enum | Cadence design | `ViewMode` enum | Names: Table, Gantt, Resources, Baselines, Calendars, Audit, Lookahead, Diary, Production | docs/tool-feature-traceability.md |
| View Icons | MS Project, P6 | Toolbar icons | Visual icon for each view tab | MS Project toolbar pattern |
| Context Menu | All | Right-click on activity | Edit, assign resources, add notes | Windows standard |

---

## 13. UI Patterns: Editing

| Pattern | Inspiration Product | Implementation | Observable Behaviour | Source |
|---------|---------------------|----------------|---------------------|--------|
| Inspector Panel | MS Project, P6 | Right side panel | Edit activity properties; changes flow through command layer | https://support.microsoft.com/en-us/project edit-tasks-in-project |
| Inline Editing | MS Project | Table cell | Double-click to edit text/number inline | https://support.microsoft.com/en-us/project/edit-tasks-in-project |
| Drag to Link | MS Project | Gantt bars | Drag from bar endpoint to create dependency | https://support.microsoft.com/en-us/project/create-dependencies-in-project |
| Column Editing | MS Project | Table header | Edit column header to rename | MS Project pattern |
| Undo/Redo | MS Project | Ctrl+Z/Y | All edits undoable; history persists | https://support.microsoft.com/en-us/project/undo-redo-in-project |

---

## 14. UI Patterns: Status & Feedback

| Pattern | Inspiration Product | Implementation | Observable Behaviour | Source |
|---------|---------------------|----------------|---------------------|--------|
| Critical Path Highlight | MS Project, P6 | Red bars, `!` marker | Critical activities shown in red | https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project |
| Status Date Line | MS Project | Vertical line | Dashed line on Gantt showing status date | https://support.microsoft.com/en-us/project/status-date-in-project |
| Progress Overlay | MS Project | Inner bar fill | Percentage shown as darker fill inside bar | https://www.barbecana.com/schedule-inspector-2 |
| Float Column | P6 | Table column | Shows total float per activity | https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm |
| Resource Conflict Indicator | MS Project | Resource view | Shows days over-allocated | https://support.microsoft.com/en-us/project/overallocation-in-project |

---

## 15. Settings & Keyboard Shortcuts

| Shortcut | Feature | UI Location | Observable Behaviour | Source |
|----------|---------|-------------|---------------------|--------|
| Ctrl+1 | Table view | Toolbar | Switch to Activity Table view | MS Project, P6 pattern |
| Ctrl+2 | Gantt view | Toolbar | Switch to Gantt view | MS Project, P6 pattern |
| Ctrl+3 | Resources view | Toolbar | Switch to Resources view | MS Project, P6 pattern |
| Ctrl+4 | Baselines view | Toolbar | Switch to Baselines view | MS Project, P6 pattern |
| Ctrl+5 | Calendars view | Toolbar | Switch to Calendars view | P6 pattern |
| Ctrl+6 | Audit view | Toolbar | Switch to Schedule Audit view | Acumen Fuse, P6 pattern |
| Ctrl+7 | Lookahead view | Toolbar | Switch to Lookahead view | Aphex pattern |
| Ctrl+8 | Diary view | Toolbar | Switch to Daily Diary view | Aphex pattern |
| Ctrl+9 | Production view | Toolbar | Switch to Production view | Aphex pattern |
| Ctrl+Z | Undo | Toolbar/keyboard | Undo last command | MS Project standard |
| Ctrl+Y | Redo | Toolbar/keyboard | Redo last undone command | MS Project standard |
| Ctrl+S | Save | File menu | Save project to file | MS Project standard |
| Ctrl+O | Open | File menu | Open project file | MS Project standard |
| Ctrl+N | New | File menu | Create new project | MS Project standard |
| Ctrl+D | Theme | Toolbar | Toggle light/dark theme | Cadence design |
| Ctrl++ | Zoom in | Gantt view | Zoom Gantt timeline in | MS Project pattern |
| Ctrl+0 | Zoom 100% | Gantt view | Reset zoom | MS Project pattern |
| Ctrl+- | Zoom out | Gantt view | Zoom Gantt timeline out | MS Project pattern |
| F1 | Help/Shortcuts | Modal | Show keyboard shortcuts dialog | Windows standard |

**Settings Categories:**
| Setting | Location | Observable Behaviour | Source |
|---------|----------|---------------------|--------|
| Theme | File > Settings | Light/Dark/System follow OS | MS Project pattern |
| Status Date | Baselines view | Editable date field | MS Project pattern |
| Zoom Level | Gantt view toolbar | Shows percentage; persists | MS Project pattern |
| Column Visibility | Toolbelt > Columns | Toggle visible columns | MS Project pattern |
| Keyboard Shortcuts | Help dialog (F1) | List all shortcuts | MS Project standard |

---

## 16. Data Flow & Persistence

| Feature | Implementation | Observable Behaviour | Source |
|---------|----------------|---------------------|--------|
| Save Operation | SQLite transaction | Atomic save: all-or-nothing write | Cadence original design |
| Load Operation | SQLite read | Validates schema version; migrates if needed | Cadence original design |
| Schema Versioning | `PRAGMA user_version` | Current schema v10; older versions migrate up | Cadence original design |
| Import/Export | Interchange handlers | MSPDI, MPX, CSV, JSON formats | `docs/research/ms-project-capabilities.md` |
| Undo Stack | Command pattern | Every edit produces inverse; Ctrl+Z/Y | Cadence original design |
| CLI Output | `--json` flag | Machine-readable JSON document | docs/usage.md |

---

## 17. Edge Cases & Error Handling

| Edge Case | Response | Source |
|-----------|----------|--------|
| Empty project | Show empty table with "New Project" prompt | MS Project pattern |
| Single activity | Show in both table and Gantt normally | Original design |
| Milestone (0 duration) | Diamond shape in Gantt; not affected by duration logic | MS Project pattern |
| Long activities | Horizontal scroll or zoom out | MS Project pattern |
| Cycle relationship | Reject with error showing cycle path | Original graph algorithms |
| Missing resource | Over-allocation flag in Resources view | MS Project pattern |
| Invalid constraint combination | Validation error before save | MS Project pattern |
| Import format mismatch | Clear error message with expected format | Original design |
| Corrupted project file | Graceful error: "File appears to be corrupted" | Original design |

---

## 18. Sources

**Primary Research Documents:**
1. `docs/research/ms-project-capabilities.md` — Microsoft Project capabilities
2. `docs/research/primavera-p6-capabilities.md` — Primavera P6 capabilities  
3. `docs/research/aphex-integration.md` — Aphex short-interval planning
4. `docs/research/schedule-audit.md` — Schedule quality checks (DCMA 14-point)
5. `docs/research/ai-integration.md` — AI assistant concepts
6. `docs/research/gui-inspiration-audit.md` — UI pattern reference

**Public URLs Referenced:**
- MS Project: https://support.microsoft.com/en-us/project/, https://learn.microsoft.com/en-us/project/
- Primavera P6: https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/, https://primavera.oraclecloud.com/
- DCMA 14-point: https://docs.onplana.com/projects-tasks/check-schedule-quality/, https://www.deltek.com/resources/articles/dcma-14-point-assessment/
- AACE: https://web.aacei.org/resources/recommended-practices
- Aphex: https://www.aphex.co/, https://help.aphex.co/docs/
- nPlan: https://www.nplan.io/

**Generated:** 2026-11-28
**Based on:** Branch `main` @ `7755c92` (2026-11-18)