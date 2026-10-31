# Capability audit — Cadence against its reference products

- **Date:** 2026-10-30
- **Scope:** every distinct capability, feature, button and action named or
  implied in Cadence's clean-room reference material, mapped to what Cadence
  actually ships.
- **References read in full:** `docs/inspiration.md`,
  `docs/research/ms-project-capabilities.md`,
  `docs/research/primavera-p6-capabilities.md`,
  `docs/research/aphex-integration.md`,
  `docs/research/schedule-audit.md`, `docs/research/ai-integration.md`,
  `docs/ui-patterns.md`, `docs/parity.md`, `docs/scope.md`.
- **Method:** behaviour comparison only. Cadence is inspired by *publicly
  documented concepts and observable behaviour*; no proprietary code, resource,
  icon, layout, string or algorithm has been consulted (`docs/inspiration.md`
  §Clean-room protocol). Every "observable behaviour" below is quoted or
  paraphrased in Cadence's own words from the public research notes.
- **Status vocabulary:** `shipped` (implemented **and** reachable from a GUI or
  CLI surface), `partial` (implemented in part, or present only in core with no
  user surface), `missing`, `out-of-scope`.
- **Reference-product keys in the tables:** `MSP` Microsoft Project, `P6`
  Primavera P6, `Asta` Asta Powerproject, `ProjectLibre`, `Aphex`, `Audit` the
  commercial DCMA/GAO/AACE audit tools (Acumen Fuse, SSI, Steelray, Barbecana,
  ScheduleLens/SmartPM/Onplana), `AI` the optional AI tools (nPlan, Copilot,
  Planera, ALICE, Trunk Tools, Document Crunch), `PMBOK` the public standard.

## Summary

Cadence is audited below against **273 distinct capabilities** across 21
domains. Overall: **130 shipped, 37 partial, 86 missing, 20 out-of-scope.**

Counts per reference product (a row naming several products is counted once
under each, so the per-product totals exceed the 273 rows; `Audit` also matches
the named tools/standards DCMA, GAO, AACE, Acumen, SSI, Steelray, Barbecana,
ScheduleLens, SmartPM, Onplana, and `AI` also matches Copilot, nPlan, Planera,
ALICE, Trunk Tools and Document Crunch):

| Reference | shipped | partial | missing | out-of-scope | total |
|---|---:|---:|---:|---:|---:|
| MSP (Microsoft Project) | 85 | 18 | 39 | 7 | 149 |
| P6 (Primavera P6) | 100 | 21 | 38 | 11 | 170 |
| Asta (Asta Powerproject) | 4 | 1 | 1 | 0 | 6 |
| ProjectLibre | 4 | 0 | 1 | 0 | 5 |
| Aphex (lookahead/field planning) | 10 | 6 | 13 | 9 | 38 |
| Audit (DCMA/GAO/AACE tools) | 31 | 8 | 16 | 0 | 55 |
| AI (optional assistant tools) | 0 | 0 | 11 | 0 | 11 |
| PMBOK (public standard) | 9 | 0 | 0 | 0 | 9 |
| **All rows (each row counted once)** | **130** | **37** | **86** | **20** | **273** |

_Counts are an automated tally of the reference and status columns of every
table below._

## Top gaps

Ranked by value to a paid professional scheduler. "Path" is the concrete build:
crate/module, command or GUI view.

| # | Gap | Reference behaviour | Why it matters | Implementation path |
|---|---|---|---|---|
| 1 | **Navigation filters, grouping and highlighters** | P6/MSP let a planner slice a 5,000-activity programme by activity code, resource, float or custom field, and highlight rows. | A professional cannot work an enterprise-size programme without slicing it; Cadence is "sort only" today. | `cadence-ui-egui`: a filter/group-by/highlighter state over the activity list, predicate on `Activity` fields; persist as a layout. Recorded "considering" in `docs/ui-patterns.md:36`. |
| 2 | **WBS tree view and roll-up** | MSP/P6 show the outline as a collapsible tree with rolled-up summary dates, cost and work. | The WBS model exists but is invisible; planners navigate by hierarchy, not a flat table. | New **WBS** view in `cadence-ui-egui` reading `project.wbs` (`crates/core/src/wbs.rs`); roll-up dates/cost in the engine. |
| 3 | **Out-of-sequence progress detection** | DCMA/SSI/Acumen flag progress posted out of sequence; P6 records it in the schedule log. | A core reason owners reject an update; already named as an audit edge case (`docs/research/schedule-audit.md:202`). | New `CheckId` in `crates/core/src/audit.rs`; also feeds the Aphex production-tracking stage. |
| 4 | **Progress-driven rescheduling mode** | MSP/P6 reschedule remaining work from actual dates and a status date. | Updating a live programme is the daily paid task; Cadence records progress but never schedules from it (D15). | Explicit engine mode (`crates/core/src/engine.rs`); already queued in `ROADMAP.md:199` (v0.3.x). |
| 5 | **Remaining duration field** | MSP keeps duration/actual/remaining separately and lets the planner override remaining work. | The planner's most-used edit during updating. | `Activity` field in `crates/core/src/model.rs`, `SetActivityRemainingDuration` command, inspector edit. |
| 6 | **Schedule log** | P6 writes a per-run record of conditions encountered (constraint effects, open ends, out-of-sequence). | The reviewer's audit trail behind the dates; the audit report is not a substitute. | Return conditions from `schedule()`/`level_resources`; a CLI/GUI panel. |
| 7 | **Longest-path criticality and per-activity longest path** | P6 distinguishes float-based criticality from the longest driving chain; SSI prints longest-path values per activity. | Constraints distort float; owners increasingly require longest-path claims. | Add a longest-path pass in `crates/core/src/engine.rs`; recorded post-v1 candidate (`docs/parity.md:35`). |
| 8 | **Driving logic / Task Inspector** | MSP's Task Inspector shows, per task, which predecessor/logic drives its dates. | Turns a critical bar into "why is this date this date". | Engine already holds the relationship graph; compute the driving predecessor; add an inspector pane in `cadence-ui-egui`. |
| 9 | **EAC / ETC / VAC / TCPI** | MSP reports estimate-at-completion, estimate-to-complete, variance-at-completion and to-complete performance index. | Standard earned-value reporting fields; Cadence stops at SPI/CPI. | Extend `EvmMetrics` (`crates/core/src/model.rs:1212`) and `evm_metrics` (`engine.rs:550`). |
| 10 | **Cost rate tables and cost accounts** | MSP/P6 carry standard/overtime/per-use rates, rate-change dates and cost accounts; cost rolls to WBS. | A cost-loaded programme is what many clients actually submit. | Extend the `Resource` model (D11 records the one-unit, no-rate limit) and add WBS cost roll-up. |
| 11 | **Activity codes and user-defined fields** | P6 codes activities from predefined value lists and UDFs, for grouping/sorting/filtering/reporting. | The enterprise unit of categorisation; needed for gap #1 to be useful. | Model + interchange + grouping; recorded as open question Q2 (`docs/research/primavera-p6-capabilities.md:89`). |
| 12 | **Saved layouts / Gantt bar styles / timescale / gridline / legend config** | MSP and P6 let the planner configure and save the whole view (tables, bars, timescale tiers, gridlines, legend). | "Looks and performs like a paid application" is an explicit owner goal. | Persisted view state + Gantt drawing options in `cadence-ui-egui`; column chooser already exists (`parity.md:46`). |
| 13 | **Status-date line and progress lines** | MSP draws a status-date line and progress lines on the Gantt; P6 uses the status-date line for EVM. | Reading schedule health off the chart; recorded "considering" (`docs/ui-patterns.md:38`). | Draw both in the Gantt canvas in `cadence-ui-egui` (a today line already exists). |
| 14 | **Redundant/reverse logic and logic-on-summaries checks** | Acumen and SSI flag redundant logic, reverse logic and logic attached to summary activities. | Common defects that inflate or hide float; the audit note already calls for relationship findings. | New relationship-level `CheckId`s in `crates/core/src/audit.rs` (`Finding::Relationship` already exists). |
| 15 | **Forensic delay attribution** | Acumen does "half-step" delay attribution; AACE RP 29R-03 defines windows/time-impact analyses. | Extension-of-time and claim work is a paid specialty; Cadence's compare lists changes but does not attribute delay. | Extend `crates/core/src/compare.rs` with window/as-built comparison. |
| 16 | **Near-critical paths and float distribution** | AACE RP 92R-17 and the audit tools report near-critical paths and a float distribution. | The highest-value schedule-risk information after the critical path itself. | Enumerate near-critical sets in the engine; add to audit + `--json`. |
| 17 | **Schedule risk analysis (Monte Carlo)** | GAO best practice and AACE RP 57R-09 require a cost/schedule risk analysis; the audit tools trend it. | Required on many public and owner programmes. | A new module/crate over the existing CPM engine; large, so post-v1. |
| 18 | **Production tracking (plan vs actual)** | Aphex records daily actual progress against the lookahead and charts plan versus actual per activity. | The last remaining Aphex-inspired stage; feeds out-of-sequence and missed-task checks. | Remaining lookahead stage (`docs/parity.md:103`, `ROADMAP.md:170`); reuses `percent_complete` and the audit. |
| 19 | **Printable briefing sheet / huddle output** | Aphex briefs crews from the plan and prints group/sort/filter output to PDF. | The daily site ritual; Cadence has the diary but no briefing artifact. | Add a diary/lookahead section to `crates/core/src/report.rs` and the existing PDF/print path. |
| 20 | **Complete master→lookahead metadata bridge** | Aphex imports WBS, calendar, relationships and owner/subcontractor/location metadata from P6/Asta/MSP. | Without structure, the lookahead is a flat list; the bridge is what makes the workflow real. | MSPDI import of WBS and named calendars (only JSON is lossless today, `docs/parity.md:85`). |
| 21 | **Board / timesheet / list presentation views** | Aphex offers map, board, list and Gantt views; timesheet-style entry is common in tracking tools. | Field and progress entry are more natural off a board than a Gantt. | New views in `cadence-ui-egui` over the same model; map is out of scope (GIS). |
| 22 | **Persisted audit threshold profiles and custom checks** | Acumen/SSI let a team save threshold playlists and add their own checks (Audit Builder). | Contractors, owners and PMOs use different limits. | Persist `AuditConfig` (`crates/core/src/audit.rs`) in the project file; a check registry. |
| 23 | **Dashboards, trend lines and Excel export** | Steelray/Acumen trend metrics across updates and export to Excel; MSP has built-in report templates. | Management reporting and progress-over-time. | Trend store + CSV/JSON export; `--json` already exists. |
| 24 | **Optional AI assistance (advice, skeletons, enrichment, status, report draft)** | Copilot/nPlan/Planera/Trunk Tools read documents and start or advance a programme. | A differentiator, but governance-sensitive; already owner-approved and queued. | Full design in `docs/research/ai-integration.md`; off-by-default, reviewable commands. |
| 25 | **Per-assignment resource units, curves and resource calendars** | MSP has per-assignment units and cost-rate curves and resource calendars; P6 has resource codes/roles and resource-dependent activities. | "Half a crane" and time-phased resource availability are standard. | Extend the `Resource`/assignment model (D11) and the histogram/leveling engine. |

## Per-domain capability checklist

### 1. Scheduling and logic

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Work breakdown structure (summary tasks + subtasks) | MSP, P6 | An outline of summary tasks and subtasks; durations, links and constraints hang off the task rows. | partial | Core + commands + JSON only: `crates/core/src/wbs.rs`, `CreateWbsNode`/`MoveWbsNode`/`AssignActivityWbs` (`crates/commands/src/lib.rs:1504`); **no GUI tree or CLI surface**. |
| Finish-to-start dependency | MSP, P6 | FS relationships with editable lag/lead in tables and on the Gantt. | shipped | `crates/core/src/model.rs:152` `LinkType`; `CreateRelationship`; engine `crates/core/src/engine.rs:641`. |
| Start-to-start dependency | MSP, P6 | SS relationships with lag/lead. | shipped | `LinkType::ALL`, `SetRelationshipLag` (`commands/lib.rs:1888`); parity `docs/parity.md:31`. |
| Finish-to-finish dependency | MSP, P6 | FF relationships with lag/lead. | shipped | As above. |
| Start-to-finish dependency | MSP, P6, ProjectLibre | SF relationships with lag/lead. | shipped | As above. |
| Signed lag / lead | MSP, P6 | Positive lag delays; negative lag (a lead) advances the successor. | shipped | `docs/parity.md:31`; drag-to-link lag editing `commands/lib.rs:1888`. |
| Forward / backward pass | MSP, P6, PMBOK | Early and late start/finish per activity. | shipped | `crates/core/src/engine.rs:641`; JSON schedule `crates/interchange/src/schedule.rs:65`. |
| Total float | MSP, P6, PMBOK | Total float per activity, shown as a column. | shipped | `Activity.total_float` (`model.rs:649`); Float column `docs/ui-patterns.md:39`. |
| Free float | P6, PMBOK | Free float per activity. | shipped | `Activity.free_float` (`model.rs:652`); `--json` free_float_days `docs/usage.md:45`. |
| Critical path (float-based) | MSP, P6 | Critical tasks highlighted in the Gantt and table. | shipped | Red bars + Critical column `docs/ui-patterns.md:24`; critical set ordered by early start `docs/parity.md:35`. |
| Longest-path criticality | P6 | Distinguishes total-float criticality from the longest driving chain that survives constraints. | missing | Needs a longest-path pass in `crates/core/src/engine.rs`; recorded post-v1 candidate `docs/scope.md:37`, `docs/parity.md:35`. |
| Milestones (zero-duration) | MSP, P6 | Zero-duration start/finish milestones. | shipped | `Activity.original_duration = 0` (`model.rs:615`); `docs/usage.md:91`. |
| Cycle detection / circular logic | P6, Audit | Circular relationships are detected rather than hung on. | shipped | `SchedError::Cycle` (Kahn + DFS) `docs/parity.md:36`. |
| Manually scheduled tasks | MSP | User-controlled task/summary dates alongside automatic ones. | out-of-scope | Cadence is deliberately logic-driven (D15, `docs/research/ms-project-capabilities.md:96`). |
| Activity types (task/resource-dependent/LOE/milestones/WBS summary) | P6 | Distinct activity types with different scheduling behaviour. | partial | Only a generic activity plus a zero-duration milestone; no LOE/resource-dependent types (`Activity`, `model.rs:610`). |
| Driving logic / Task Inspector | MSP | Per-task pane showing which predecessor drives the dates. | missing | Compute driving predecessor from the existing graph; add an inspector pane in `cadence-ui-egui`. |
| Network Diagram / PERT view | MSP | Box-and-arrow network view of the logic. | missing | A new `cadence-ui-egui` view over the relationship graph. |
| Task Path highlighting | MSP | Highlight the driving chain to/from a selected task. | missing | Engine graph walk + Gantt highlight; depends on longest-path work. |
| Cross-calendar linkage convention | P6 | Linked activities on different calendars schedule on the correct working days. | shipped | Convention D6 (`DECISIONS.md:34`), `docs/architecture.md`. |
| Multiple projects / EPS | P6 | Enterprise project structure holding many projects. | out-of-scope | `docs/research/primavera-p6-capabilities.md:94`, `docs/scope.md:25`. |

### 2. WBS, outline and activity codes

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| WBS hierarchy model (create/move/rename/delete) | MSP, P6 | A tree of outline nodes with cycle-safe reparenting. | shipped | `crates/core/src/wbs.rs`; `CreateWbsNode`, `MoveWbsNode` (`commands/lib.rs:1504`, `:1619`); `docs/parity.md:30`. |
| Activity membership in a WBS node | MSP, P6 | Each activity belongs to a WBS node. | partial | `Activity.wbs` (`model.rs:619`), `AssignActivityWbs`; **no GUI/CLI to set it**. |
| WBS display as a tree view | MSP, P6, ProjectLibre | Collapsible outline in the table/Gantt. | missing | New **WBS** view in `cadence-ui-egui` reading `project.wbs`. |
| WBS summary roll-up (dates, work, cost) | MSP, P6 | Parent rows roll up child dates/cost/work. | missing | Roll-up in `crates/core/src/engine.rs` + WBS view; no current code. |
| WBS outline numbering / auto codes | MSP, P6 | Automatic outline numbers per node. | missing | Derive in `crates/core/src/wbs.rs` + display. |
| WBS in interchange | MSP, P6 | The WBS tree travels with the project. | partial | JSON only (`docs/parity.md:85`); MSPDI/MPX/CSV import it empty. |
| Activity codes (global/EPS/project) | P6 | Categorise activities from predefined value lists for grouping/sorting/filtering. | missing | Model + interchange + grouping; open question Q2 (`docs/research/primavera-p6-capabilities.md:89`). |
| User-defined fields (UDFs) | P6 | Free-form custom attributes on activities. | missing | Model + interchange + column chooser entry. |

### 3. Calendars and shifts

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Multiple named calendars | MSP, P6 | Global and named calendars activities may select. | shipped | `crates/core/src/calendar_set.rs`; `docs/parity.md:33`. |
| Working-week patterns | MSP, P6 | A Mon–Fri (or other) working-week pattern. | shipped | `Calendar::new` (`model.rs:231`). |
| Holiday exceptions | MSP, P6 | Dated non-working exceptions. | shipped | `Calendar::with_holidays` (`model.rs:257`); `docs/parity.md:32`. |
| Per-activity calendar assignment | MSP, P6 | An activity uses any calendar. | shipped | `Activity.calendar` (`model.rs:625`), `SetActivityCalendar` (`commands/lib.rs:1803`). |
| Calendar management UI | MSP, P6 | Create/rename/delete calendars and edit working days/holidays. | shipped | Calendars view `docs/parity.md:87`, `docs/usage.md:190`. |
| Project default calendar | MSP, P6 | A project-wide default calendar. | shipped | `project.calendar`, `resolve_calendar` (`model.rs:926`). |
| Shifts / multiple shifts per day | P6, Aphex | Working-time patterns finer than one shift per day. | missing | Extend `Calendar` (`model.rs:216`) with shift blocks; Aphex edge case `docs/research/aphex-integration.md:104`. |
| Resource calendars | MSP, P6 | A calendar per resource. | missing | Add `calendar` to `Resource` (`model.rs:78`) and honour it in the histogram/leveling. |
| Calendars in interchange | MSP, P6 | Calendars travel between tools. | partial | JSON only (`docs/parity.md:85`); MSPDI/MPX/CSV cannot express them (D14). |

### 4. Constraints and deadlines

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| As-soon-as-possible (ASAP) | MSP, P6 | Default: schedule as early as logic allows. | shipped | `Constraint::AsSoonAsPossible` (`model.rs:533`); `docs/parity.md:34`. |
| As-late-as-possible (ALAP) | MSP, P6 | Schedule as late as logic allows; a reporting preference (D10). | shipped | `Constraint::AsLateAsPossible`; D10 `DECISIONS.md:81`. |
| Start/finish no-earlier-than (SNET/FNET) | MSP, P6 | Lower-bound date constraints. | shipped | `Constraint::{StartNoEarlierThan,FinishNoEarlierThan}`; D10 `DECISIONS.md:67`. |
| Start/finish no-later-than (SNLT/FNLT) | MSP, P6 | Upper-bound date constraints; tighten float. | shipped | As above; constraint violation is an explicit error (`ConstraintViolated`). |
| Must-start-on / must-finish-on (hard) | MSP, P6 | Exact-date inflexible constraints. | missing | Deliberately deferred (D10 `DECISIONS.md:72`); would need a `Constraint` variant the engine can resolve. |
| Mandatory start/finish | P6 | Hard mandatory dates honoured by the scheduler. | out-of-scope | Recorded out (`docs/research/primavera-p6-capabilities.md:82`). |
| Deadline field (slack without constraining) | MSP | A deadline shows negative slack without moving dates. | missing | Add `Activity.deadline` (`model.rs:610`) and surface slack; research `docs/research/ms-project-capabilities.md:96` says "may be revisited". |
| Constraint-effect reporting in a schedule log | P6 | The log documents how constraints applied. | partial | The audit's hard-constraint check reports constrained activities (`audit.rs:560`); no per-run log. |
| Constraint-conflict error | MSP, P6 | An impossible upper bound is reported, not silently absorbed. | shipped | `SchedError::ConstraintViolated` (D10 `DECISIONS.md:77`). |

### 5. Resources

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Resource dictionary | MSP, P6, Asta | A named list of resources with types. | shipped | `Resource` (`model.rs:78`), `CreateResource`; `docs/parity.md:54`. |
| Resource kinds (labour/equipment/material) | MSP, P6 | Work, material and cost resource types. | shipped | `ResourceKind` (`model.rs:129`); `SetResourceKind` (`commands/lib.rs:1147`). |
| Capacity (units per working day) | MSP, P6 | Resource availability driving over-allocation. | shipped | `Resource.capacity` (`model.rs:78`); `SetResourceCapacity` (`commands/lib.rs:1120`). |
| Assign / unassign resources | MSP, P6 | Resources assigned to activities. | shipped | `Activity.resources` (`model.rs:621`); `AddResourceToActivity`/`RemoveResourceFromActivity` (`commands/lib.rs:851`). |
| Per-assignment units factor | MSP | A resource assignment may consume less than one unit (e.g. 0.5). | missing | D11 records the one-unit model (`DECISIONS.md:88`); ROADMAP `ROADMAP.md:201` plans per-assignment units. |
| Cost-rate tables (standard/overtime/per-use) | MSP, P6 | Rate tables with time-phased rate changes. | missing | D11 (`DECISIONS.md:94`); ROADMAP `ROADMAP.md:201`. |
| Resource availability tables over time | MSP, P6 | Time-phased resource availability. | missing | Extend `Resource` (`model.rs:78`) with availability windows. |
| Over-allocation detection | MSP, P6 | Indicators where a resource is consumed beyond capacity. | shipped | `engine::resource_conflicts` (`engine.rs:325`); `docs/parity.md:58`. |
| Resource leveling | MSP, P6 | Automatic/manual leveling to remove over-allocation. | shipped | `engine::level_resources` (`engine.rs:396`), greedy; D12; `docs/parity.md:59`. |
| Leveling options (order/priority/granularity/split/within-float) | MSP, P6 | Leveling respects priority order, granularity and may split tasks. | partial | Greedy earliest-conflict delay only (D12); no options, prioritisation or task splitting. |
| Resource histogram (committed units/day) | MSP, P6 | Time-phased usage grid. | shipped | `engine::resource_histogram` (`engine.rs:231`); `docs/parity.md:57`. |
| Histogram chart in the GUI | MSP, P6 | A bar-chart usage view. | partial | Resources view shows per-day usage **as a table** (`docs/usage.md:186`); a bar chart is open polish (`ROADMAP.md:208`). |
| Task Usage / Resource Usage time-phased grids | MSP, P6 | Per-task and per-resource time-phased grids. | partial | Resource usage table exists; no task-usage grid. |
| Driving resource per activity | MSP | Which resource assignment drives the activity's duration. | missing | Derive in the engine; no current code. |
| Resource curves / profile shapes | Asta | Distribution curves for how work is spread. | missing | Model + histogram support. |
| Resource codes and roles | P6 | Roles and resource codes for grouping. | missing | Model + grouping. |
| Resource-dependent activities | P6 | The resource, not the fixed duration, drives the activity. | missing | Needs duration-type fields on `Activity` (`model.rs:610`). |
| Multi-project resource pools | P6 | Shared enterprise resource pools. | out-of-scope | `docs/research/primavera-p6-capabilities.md:96`. |

### 6. Cost and earned value

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Budget at completion (BAC) per activity | MSP, P6, PMBOK | Planned total cost per activity. | shipped | `Activity.budget_at_completion` (`model.rs:659`); `SetActivityBudget` (`commands/lib.rs:581`). |
| Actual cost (ACWP) | MSP, P6 | Actual cost incurred to date. | shipped | `Activity.actual_cost` (`model.rs:662`); `SetActivityActualCost` (`commands/lib.rs:654`). |
| Planned value (PV / BCWS) | MSP, P6, PMBOK | Budgeted cost of work scheduled. | shipped | `EvmMetrics.planned_value` (`model.rs:1216`); `engine.rs:550`. |
| Earned value (EV / BCWP) | MSP, P6, PMBOK | Budgeted cost of work performed. | shipped | `EvmMetrics.earned_value` (`model.rs:1218`). |
| Cost variance (CV) | MSP, P6, PMBOK | EV − AC. | shipped | `EvmMetrics.cost_variance` (`model.rs:1222`). |
| Schedule variance (SV) | MSP, P6, PMBOK | EV − PV. | shipped | `EvmMetrics.schedule_variance` (`model.rs:1224`). |
| SPI / CPI | MSP, P6, PMBOK | Schedule and cost performance indices. | shipped | `EvmMetrics.{cost,schedule}_performance_index` (`model.rs:1226`); `docs/parity.md:68`. |
| EAC / ETC / VAC | MSP, P6 | Estimate at completion, to complete and variance at completion. | missing | Extend `EvmMetrics` (`model.rs:1212`); no current fields. |
| TCPI | MSP | To-complete performance index. | missing | Extend `EvmMetrics`. |
| Earned-value baseline designation | P6 | A designated EV baseline for the metrics. | partial | The most recent baseline drives the checks (`docs/usage.md:77`); no user-designated EV baseline. |
| Store period performance | P6 | Freeze each financial period's actuals so cumulative metrics survive changes. | out-of-scope | `docs/research/primavera-p6-capabilities.md:99` (a cost-system feature). |
| Cost accounts | MSP, P6 | Activities charged to cost accounts. | missing | Model + roll-up; no current code. |
| Cost roll-up to WBS | MSP, P6 | Parent nodes total child cost. | missing | Engine roll-up + WBS view. |
| EVM surfaced in GUI and reports | MSP, P6 | Earned-value views and reports. | shipped | Baselines view EVM at a status date (`docs/usage.md:189`); report earned-value section (`crates/core/src/report.rs:15`). |

### 7. Baselines

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Multiple baselines | MSP (up to 11), P6 | Several named sets of baseline dates/work/cost. | shipped | `Project.baselines` (`model.rs:794`); `docs/parity.md:66`. |
| Baseline capture / remove / restore | MSP, P6 | Save, delete and bring back a baseline. | shipped | `CaptureBaseline`/`RemoveBaseline`/`RestoreBaseline` (`commands/lib.rs:1197`); undoable. |
| Baseline variance (duration/start/finish) | MSP, P6 | Per-activity variance against the selected baseline. | shipped | Baselines view `docs/parity.md:67`; `duration_variance`/`start_variance` (`ui-egui/lib.rs:3443`). |
| Tracking Gantt overlay of the baseline | MSP | The Tracking Gantt draws the baseline behind the current bars. | partial | Cadence draws a progress fill, not a baseline bar overlay (`docs/research/ms-project-capabilities.md:92`). |
| Interim plans | MSP | A dated snapshot of start/finish separate from the named baselines. | missing | Reuse the baseline machinery with an interim kind. |
| Baseline comparison (two-version diff) | MSP, P6 | Compare Projects / Schedule Comparison lists all differences. | shipped | `cadence audit-compare` CLI + `crates/core/src/compare.rs`; `docs/parity.md:101`. |
| Baselines in interchange | MSP, P6 | Baselines travel between tools. | partial | MSPDI carries them; MPX/CSV do not (`docs/usage.md:109`). |

### 8. Progress updating

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Percent complete | MSP, P6 | Progress entered as a percentage. | shipped | `Activity.percent_complete` basis points (`model.rs:665`); `SetActivityProgress` (`commands/lib.rs:613`). |
| Actual start / actual finish | MSP, P6 | Recorded real dates beside the plan. | shipped | `Activity.actual_start/actual_finish` (`model.rs:668`); `SetActivityActualDates` (`commands/lib.rs:687`). |
| Actual cost entry | MSP, P6 | Actual cost posted against the activity. | shipped | `SetActivityActualCost` (`commands/lib.rs:654`). |
| Remaining duration field | MSP, P6 | A separate, editable remaining-duration value. | missing | Add `Activity.remaining_duration` (`model.rs:610`) + command + inspector edit. |
| Out-of-sequence progress detection | P6, Audit | Flags progress posted on an activity whose predecessors are unfinished. | missing | New `CheckId` in `crates/core/src/audit.rs:70`; named edge case `docs/research/schedule-audit.md:202`. |
| Progress lines on the Gantt | MSP | Trend lines connecting progress points. | missing | Draw in the Gantt canvas (`cadence-ui-egui`); status date needed first. |
| Progress overlay on bars | MSP, Asta | Percent complete drawn inside the bar. | shipped | Darker fill in Gantt bars (`docs/ui-patterns.md:47`); `docs/parity.md:69`. |
| Progress-driven rescheduling | MSP, P6 | Remaining work is rescheduled from actual dates/status date. | missing | Explicit engine mode; D15 (`DECISIONS.md:199`), queued `ROADMAP.md:199`. |
| Status date | MSP, P6 | A date that anchors EVM and updating. | shipped | Editable status date in the Baselines view (`ui-egui/lib.rs:2453`); `--status-date` (`main.rs:33`). |
| Progress in interchange | MSP, P6 | Progress travels between tools. | shipped | MSPDI/CSV/JSON carry percent, actual dates, actual cost (`docs/usage.md:109`). |

### 9. Views and user interface

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Combined table + Gantt split view | MSP, P6, Asta, ProjectLibre | Task table on the left, Gantt on the right, synced vertically. | shipped | Split Gantt toggle (`docs/ui-patterns.md:17`); `show_split_gantt` (`ui-egui/lib.rs:291`). |
| View tabs | MSP, P6 | A row of named view tabs, one active. | shipped | Tab strip (`docs/ui-patterns.md:18`); `ViewMode` (`ui-egui/lib.rs:134`). |
| Column chooser | MSP, P6 | Choose which columns a table shows. | shipped | `Columns` menu (`ui-egui/lib.rs:1577`); `docs/parity.md:46`. |
| Sortable column headers | MSP, P6 | Click a header to sort. | shipped | Sort buttons + cached sort (`docs/parity.md:46`); `SortKey` (`ui-egui/lib.rs:91`). |
| Filters over the activity table | MSP, P6 | Filter bar above the table. | missing | "Considering", sort only (`docs/ui-patterns.md:36`); add predicate state in `cadence-ui-egui`. |
| Group-by over the activity table | MSP, P6, Aphex | Group rows by code/resource/field. | missing | Same view-state work; recorded "considering" (`docs/ui-patterns.md:36`). |
| Highlighters | MSP, P6 | Rule-based row/bar highlighting. | missing | Same view-state work; no current code. |
| Saved layouts | P6 | A saved combination of columns, filters, sort and grouping. | missing | Persist view state in the project file. |
| Gantt bar styles customisation | MSP | Configure bar colours/shapes by rule. | missing | Gantt drawing options in `cadence-ui-egui`; only critical/normal/progress bars today. |
| Gantt timescale configuration | MSP | Multiple timescale tiers (years/quarters/months/weeks/days). | partial | Week grid + date ticks and zoom only (`docs/ui-patterns.md:54`); no tier configuration. |
| Gantt gridline configuration | MSP | Configurable vertical/horizontal gridlines. | partial | A low-contrast weekly grid only (`docs/ui-patterns.md:26`). |
| Gantt legend | MSP, P6 | A legend explaining bar colours. | missing | AccessKit summary names a legend (`DECISIONS.md:453`); no visible legend widget. |
| Today line | MSP, P6 | A vertical line at today's date. | shipped | Accent today line (`docs/ui-patterns.md:25`). |
| Status-date line | MSP, P6 | A vertical line at the status date. | missing | "Considering" (`docs/ui-patterns.md:38`); draw in the Gantt canvas. |
| Dependency arrows | MSP, P6 | Arrows linking dependent bars. | shipped | O(relationships) arrow pass (`docs/parity.md:45`). |
| Drag to create dependency links | MSP | Drag from one bar to another. | shipped | Drag-to-link (`docs/ui-patterns.md:27`); keyboard alternative too. |
| Zoom / pan | MSP, P6 | Timeline zoom and canvas panning. | shipped | Ctrl+wheel/+keyboard zoom, scroll pan (`docs/parity.md:48`). |
| Row numbers | MSP | Implicit task ID in the first column. | shipped | `#` column (`docs/ui-patterns.md:20`). |
| Footer status bar | MSP | Task counts, dates, zoom. | shipped | Status bar (`docs/ui-patterns.md:22`). |
| Inspector / details pane | MSP, P6 | Edit selected activity details. | shipped | Table inspector (`docs/usage.md:178`). |
| Light / dark theme | ProjectLibre, MSP | Theme switching. | shipped | Ctrl+D (`docs/usage.md:168`); `docs/parity.md:88`. |
| Selection sync table↔Gantt | MSP, P6 | Selecting a row selects its bar and vice versa. | partial | Shared selection exists; documented as a follow-up (`docs/ui-patterns.md:17`). |
| Vertical/table toolbar | P6 | A toolbar per workspace. | shipped | Toolbar per view (`docs/parity.md:90`). |
| Calendar (month) view | MSP | A calendar-grid view of tasks. | missing | New `cadence-ui-egui` view. |
| Timeline view | MSP | A high-level timeline of key tasks. | missing | New view over milestones/summary activities. |
| Board view | Aphex | Kanban-style board of work items. | missing | New view; recorded as an Aphex capability (`docs/research/aphex-integration.md:28`). |
| Map view | Aphex | Spatial map of work locations. | out-of-scope | A real GIS layer is out of scope for v1 (`docs/research/aphex-integration.md:71`). |
| List view | Aphex | A flat list of work items. | partial | The Table view is list-like (sortable rows). |
| Timesheet view | Aphex, tracking tools | Time-phased progress/entry grid. | missing | New view (see gap #21). |
| Print preview | MSP, P6 | Preview before printing. | partial | Reports/Gantt go straight to PDF (no preview dialog). |

### 10. Tracking and status

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Schedule log (per-run conditions) | P6 | Records out-of-sequence progress, constraint effects and open ends encountered during a schedule run. | missing | Return conditions from `crates/core/src/engine.rs`; the audit is not a substitute. |
| Schedule health score / grade | P6, Steelray | A single health score/grade from the checks. | partial | Audit gives pass/fail/n-a per check and a tally, but no score/grade (`crates/core/src/audit.rs:199`). |
| Scorecard green/yellow/red | Steelray, Audit | Colour-coded per-check scorecard. | partial | GUI Audit view shows outcomes per check (`docs/usage.md:193`); no colour grade. |
| Status report drafting | MSP (Copilot), AI | Draft a narrative status report from the schedule. | missing | AI research scope item 5 (`docs/research/ai-integration.md:25`). |

### 11. Lookahead / field planning (Aphex-inspired)

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Rolling lookahead window | Aphex | Plan upcoming work over a rolling window (commonly 3–12 weeks). | shipped | `crates/core/src/lookahead.rs:59`; GUI Lookahead view `Ctrl+7` (`docs/parity.md:103`). |
| Configurable window length | Aphex | A configurable window length. | shipped | 2/4/6/12 weeks in the Lookahead view (`docs/usage.md:199`); acceptance says 2–12 (`docs/research/aphex-integration.md:54`). |
| Weekly work plans | Aphex | Keep the near-term plan live as work unfolds. | shipped | Lookahead view + owner/promise tracking (`docs/parity.md:70`). |
| Publish immutable snapshot | Aphex | Freeze/publish a snapshot for comparison. | shipped | `CaptureLookaheadSnapshot` (`commands/lib.rs:1267`), schema v6 (`docs/parity.md:70`). |
| Compare lookahead vs previous / master | Aphex | Compare the latest published lookahead with previous versions and the master. | shipped | Snapshot start/finish variance comparison (`docs/parity.md:70`). |
| Owner per activity | Aphex, Asta | Task owners assigned per task. | shipped | `Activity.owner` (`model.rs:674`); `SetActivityOwner` (`commands/lib.rs:730`). |
| Promise flag | Aphex | A "promise" on imminent tasks. | shipped | `Activity.promised` (`model.rs:677`); `SetActivityPromise` (`commands/lib.rs:803`). |
| Delay reason required to move a promise | Aphex | A delay reason is required to move a promised task. | partial | The reason is stored (`Activity.delay_reason`, `model.rs:680`) but moving dates does not *require* one (no enforcement in `commands`). |
| Variation tracking | Aphex | Track variations against promised work. | missing | Add a variation record to the promise model; no current code. |
| Daily diary | Aphex | Dated site records linked to planned work. | shipped | `DailyNote` (`model.rs:1178`); Diary view `Ctrl+8`, schema v7 (`docs/parity.md:71`). |
| Daily meetings / huddles | Aphex | Structured daily coordination from the live plan. | partial | The Diary view records the day; no meeting artifacts/agenda. |
| Briefings / printable briefing sheet | Aphex | Brief crews from the plan; print group/sort/filter output. | missing | Add a briefing/lookahead section to `crates/core/src/report.rs` + print path. |
| Production tracking (plan vs actual) | Aphex | Track production against the plan per activity. | missing | Remaining lookahead stage (`ROADMAP.md:170`, `docs/parity.md:103`); reuses percent-complete + audit. |
| Staging diagrams (auto-built) | Aphex | Map work with auto-built diagrams. | missing | New view; large, dependency-drawing territory. |
| Resource planning (forecast labour/plant/materials) | Aphex | Forecast and allocate labour, plant and materials. | partial | Resource dictionary + histogram + leveling exist; no forward allocation/forecast view. |
| Location / area tags | Aphex | Optional location per activity. | shipped | `Activity.location` (`model.rs:683`), `SetActivityLocation` (`commands/lib.rs:766`), schema v8. |
| Location clash detection | Aphex | Flag two activities planned on the same day in the same location. | shipped | `crates/core/src/clash.rs::location_clashes`; listed in the Lookahead view (`docs/parity.md:72`). |
| Master→lookahead bridge (metadata import) | Aphex | Import WBS, calendar, relationships and owner/subcontractor/location attributes from P6/Asta/MSP. | partial | MSPDI/JSON import activities/links; WBS/calendars only via JSON (`docs/parity.md:85`). |
| Published reality fed back to the master | Aphex | Feed what actually happened back to the plan. | missing | Timeline: implied by production tracking; no current code. |
| Blockers on tasks | Aphex | Flag blockers before publishing. | missing | Add a blocker field/flag to `Activity` (`model.rs:610`). |
| Mandatory fields before publishing | Aphex | Mandatory fields gate publishing. | missing | Add validation to `CaptureLookaheadSnapshot` (`commands/lib.rs:1267`). |
| Multi-user editing on one plan | Aphex | Several users edit one plan without overwriting. | out-of-scope | `docs/scope.md:25` (single-user, offline). |
| Comments and notifications | Aphex | In-app comments and notifications. | out-of-scope | Notifications are cloud/collab territory (`docs/scope.md:25`). |

### 12. Schedule-quality auditing (DCMA 14 / GAO / AACE)

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| DCMA 1 — Logic (open ends) | Audit, P6 | Activities with no predecessor, no successor, or neither. | shipped | `CheckId::LogicOpenEnds` (`audit.rs:71`), `AUDIT-LOGIC`. |
| DCMA 2 — Leads | Audit, P6 | Relationships with negative lag. | shipped | `CheckId::Leads` (`audit.rs:73`), `AUDIT-LEADS`. |
| DCMA 3 — Lags | Audit, P6 | Relationships with positive lag. | shipped | `CheckId::Lags` (`audit.rs:75`), `AUDIT-LAGS`. |
| DCMA 4 — Relationship types | Audit, P6 | Share of finish-to-start relationships. | shipped | `CheckId::RelationshipTypes` (`audit.rs:77`), `AUDIT-RELTYPES`. |
| DCMA 5 — Hard constraints | Audit, P6 | Must/no-later-than date constraints. | shipped | `CheckId::HardConstraints` (`audit.rs:79`), `AUDIT-HARDCONST`. |
| DCMA 6 — High float | Audit, P6 | Total float above ~44 working days. | shipped | `CheckId::HighFloat` (`audit.rs:81`), `AUDIT-HIGHFLOAT`. |
| DCMA 7 — Negative float | Audit, P6 | Total float below zero. | shipped | `CheckId::NegativeFloat` (`audit.rs:83`), `AUDIT-NEGFLOAT`. |
| DCMA 8 — High duration | Audit, P6 | Activities longer than ~44 working days. | shipped | `CheckId::HighDuration` (`audit.rs:85`), `AUDIT-HIGHDUR`. |
| DCMA 9 — Invalid dates | Audit, P6 | Forecast before, or actual after, the status date. | shipped | `CheckId::InvalidDates` (`audit.rs:87`), `AUDIT-INVALIDDATES`. |
| DCMA 10 — Resources | Audit, P6 | Non-milestone activities with no resource. | shipped | `CheckId::MissingResources` (`audit.rs:89`), `AUDIT-RESOURCES`. |
| DCMA 11 — Missed tasks | Audit, P6 | Activities finishing after their baseline finish. | shipped | `CheckId::MissedTasks` (`audit.rs:91`), `AUDIT-MISSED`. |
| DCMA 12 — Critical path test | Audit, P6 | Nudge a critical activity and reschedule; the finish must move. | shipped | `CheckId::CriticalPathTest` 600-day nudge (`audit.rs:95`), `AUDIT-CPT`. |
| DCMA 13 — CPLI | Audit, P6 | Critical-path length index vs the deadline. | shipped | `CheckId::CriticalPathLengthIndex` (`audit.rs:97`), `AUDIT-CPLI`. |
| DCMA 14 — BEI | Audit, P6 | Baseline execution index. | shipped | `CheckId::BaselineExecutionIndex` (`audit.rs:93`), `AUDIT-BEI`. |
| Configurable thresholds | Audit, P6, Steelray | Every threshold is configurable. | shipped | `AuditConfig` (`audit.rs:60`); in code, not persisted (see gap #22). |
| Explanations per check | Steelray ("The Coach") | A plain-language explanation per check. | shipped | Each `CheckResult` carries a summary sentence (`audit.rs:215`); `docs/usage.md:61`. |
| GUI audit dashboard + findings that jump to the activity | Audit, Steelray, Barbecana | A scorecard plus offending tasks highlighted in the tool. | shipped | GUI **Audit** view `Ctrl+6`; findings scroll to the activity (`docs/usage.md:82`). |
| Machine-readable audit output | Audit, Acumen | Export the audit for scripting/reporting. | shipped | `cadence audit --json` (`kind: "audit"`), `docs/usage.md:78`. |
| Redundant logic | Audit (Acumen, SSI) | Two paths to the same successor. | missing | New `Finding::Relationship` check in `crates/core/src/audit.rs`. |
| Reverse logic | Audit (Acumen) | A relationship that runs backwards through the network. | missing | New relationship-level check in `audit.rs`. |
| Circular logic | Audit (Acumen) | A logic loop. | shipped | Detected by the engine (`SchedError::Cycle`, `docs/parity.md:36`). |
| Logic on summary activities | Audit (Acumen, SSI) | Logic attached to a roll-up/summary. | missing | Relationship-level check (WBS logic rule) in `audit.rs`; edge case `docs/research/schedule-audit.md:195`. |
| Out-of-sequence progress | Audit (Acumen, SSI), P6 | Progress posted out of sequence. | missing | New check in `audit.rs`; see gap #3. |
| Negative slack | Audit (SSI) | Negative slack = negative float. | shipped | `CheckId::NegativeFloat` (`audit.rs:83`). |
| Very high total slack | Audit (SSI) | Very high total float. | shipped | `CheckId::HighFloat` (`audit.rs:81`). |
| Unnecessary date constraints | Audit (SSI) | Constraints that are not needed. | partial | Hard-constraint check flags constraints (`audit.rs:560`); no "unnecessary" logic. |
| No baseline | Audit (SSI) | Flags a programme with no baseline. | partial | Baseline checks report `n/a` when absent (`docs/usage.md:70`); not a distinct finding. |
| Child/parent baseline mismatches | Audit (SSI) | Parent/child baseline date/cost/work mismatches. | missing | WBS-level check; needs roll-up first. |
| Longest-path value per activity | Audit (SSI), P6 | Longest-path duration printed per activity. | missing | Engine longest-path pass (gap #7). |
| Logic Trace / Logic Sensitivity | Audit (Acumen) | Trace driving paths; nudge an activity and see downstream impact. | partial | The critical-path test is the same idea (`audit.rs:95`); no interactive trace. |
| Forensic comparison of two updates | Audit (Acumen, SSI), P6 | List logic/calendar/duration/progress changes between updates. | shipped | `cadence audit-compare` + `crates/core/src/compare.rs:1`. |
| Delay attribution ("half-step") | Audit (Acumen), AACE 29R-03 | Attribute the delay between versions to causes. | missing | Extend `crates/core/src/compare.rs`; gap #15. |
| Ribbon / phase / intersection analysers | Audit (Acumen) | Trend and overlap analysis across updates. | missing | New analysis module. |
| Trend lines across updates | Audit (Steelray, Acumen) | Metric trends run over several updates. | missing | Needs a stored metric history; gap #23. |
| Schedule Cleanser / auto-fix | Audit (Acumen) | Flags and can correct common defects. | missing | A command-driven fix pass over findings; no current code. |
| Custom checks / Audit Builder | Audit (SSI) | Teams add their own checks. | missing | A check registry over `AuditConfig`; gap #22. |
| Excel / spreadsheet export of the audit | Audit (Acumen, Steelray) | Export results to a workbook. | partial | JSON export only (`docs/usage.md:78`); CSV export is activity-level. |
| Near-critical paths | AACE 92R-17, Audit | Report near-critical paths. | missing | Engine enumeration + audit/JSON; a recorded open item (`docs/research/schedule-audit.md:168`). |
| Schedule risk analysis (Monte Carlo) | GAO, AACE 57R-09 | Probabilistic cost/schedule risk analysis. | missing | A new module/crate over the CPM engine; out of the v1 audit slice. |
| GAO — capture all activities | Audit, GAO | Every scope item is an activity. | partial | Coverable via activity completeness checks; no explicit check. |
| GAO — sequence all activities | Audit, GAO | All activities have logic. | shipped | Logic/open-ends check (`audit.rs:71`). |
| GAO — assign resources | Audit, GAO | Every activity is resourced. | shipped | Missing-resources check (`audit.rs:89`). |
| GAO — establish durations | Audit, GAO | Reasonable durations. | shipped | High-duration check (`audit.rs:85`). |
| GAO — horizontal & vertical traceability | Audit, GAO | Logic matches the WBS and the scope. | partial | Logic checks exist; no WBS-traceability check. |
| GAO — confirm the critical path | Audit, GAO | The critical path is valid. | shipped | CPT and CPLI (`audit.rs:95`, `:97`). |
| GAO — reasonable total float | Audit, GAO | Float stays within bounds. | shipped | High/negative-float checks (`audit.rs:81`, `:83`). |
| GAO — update with actual progress and logic | Audit, GAO | Updates carry progress and revised logic. | shipped | Invalid-dates and missed-tasks checks (`audit.rs:87`, `:91`). |
| GAO — maintain a baseline | Audit, GAO | A baseline is kept and compared. | shipped | Baseline checks + BEI (`audit.rs:91`, `:93`). |

### 13. Interchange

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| MSPDI (Microsoft Project XML) import/export | MSP, P6 | Read/write the documented MSPDI dialect. | shipped | `crates/interchange/src/mspdi.rs` (D14); `cadence export-xml`/`import-xml`. |
| MPX import/export | MSP | Read/write the legacy MPX record format. | shipped | `crates/interchange/src/mpx.rs`; `cadence export-mpx`/`import-mpx`. |
| CSV import/export | MSP, P6 | One activity per row with links/resources/progress folded in. | shipped | `crates/interchange/src/csv.rs`; `cadence export-csv`/`import-csv`. |
| JSON import/export (Cadence schema, lossless) | Cadence, P6 (comparison) | A versioned, lossless document. | shipped | `crates/interchange/src/json.rs`; `docs/parity.md:81`. |
| Schedule `--json` output | P6, MSP tooling | The derived schedule as a machine-readable document. | shipped | `crates/interchange/src/schedule.rs:65`; `docs/usage.md:33`. |
| WBS in interchange | MSP, P6 | The outline travels between tools. | partial | JSON only (`docs/parity.md:85`). |
| Calendars in interchange | MSP, P6 | Calendars travel between tools. | partial | JSON only (`docs/parity.md:85`). |
| Activity codes / UDFs in interchange | P6 | Codes and custom fields travel. | missing | Model + MSPDI/JSON custom-field mapping. |
| Baselines in interchange | MSP, P6 | Baselines travel. | partial | MSPDI only (`docs/usage.md:109`). |
| Excel export | MSP, P6 | Native Excel workbook exchange. | partial | CSV export only (`docs/usage.md:111`). |
| `.mpp` import/export | MSP | The proprietary binary project format. | out-of-scope | Would need reverse engineering (`docs/research/ms-project-capabilities.md:100`). |
| XER import/export | P6 | The proprietary XER database export. | out-of-scope | `docs/research/primavera-p6-capabilities.md:94`. |
| P6 XML import | P6 | P6's XML export (MSPDI-compatible). | partial | MSPDI-compatible subset is read via `mspdi.rs`; no P6-specific XML features. |
| Project Server / Online storage | MSP | Cloud project storage. | out-of-scope | `docs/research/ms-project-capabilities.md:102`. |

### 14. Reporting, printing and distribution

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Markdown report | MSP, P6 | A reviewable, diffable schedule report. | shipped | `crates/core/src/report.rs:35`; `cadence report`. |
| PDF report print | MSP, P6 | Render the report to PDF. | shipped | `crates/core/src/pdf.rs`, `project_report_pdf` (`report.rs:75`); `cadence print`. |
| Print the Gantt view | MSP, P6 | Print the chart as an image page. | shipped | GUI **Print Gantt…** (`docs/usage.md:126`). |
| Per-resource report section | MSP, P6 | Committed and over-allocated days per resource. | shipped | Resources section (`report.rs:13`); `docs/usage.md:120`. |
| Earned-value report section | MSP, P6 | EVM metrics against a baseline/status date. | shipped | Earned-value section (`report.rs:14`); `docs/usage.md:121`. |
| Built-in report templates / dashboards | MSP, P6 | Report templates and interactive dashboards. | missing | A template catalogue over `report.rs`; none exists. |
| Visual reports to Excel/Visio | MSP | Export to Excel/Visio visual reports. | missing | CSV export is the only spreadsheet path; no visual reports. |
| Executive briefing report | Audit (Acumen) | An executive summary report. | missing | A report template over the audit JSON. |
| Share by link / distribution | Aphex, MSP | Share a read-only view by secure link. | out-of-scope | A hosted link is a cloud feature (`docs/research/aphex-integration.md:77`). |

### 15. Collaboration

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Multi-user concurrent editing | Aphex, P6 | Several users edit one plan without overwriting. | out-of-scope | `docs/scope.md:25`; offline single-file model (`VISION.md`). |
| Cloud sync / hosted component | MSP, P6, Aphex | Projects live in the cloud. | out-of-scope | `docs/scope.md:26`. |
| Task owners / subcontractors | Aphex, Asta | A responsible owner (and subcontractor) per task. | partial | A single free-text `owner` exists (`model.rs:674`); no subcontractor/company concept. |
| Comments / notifications | Aphex | Per-item comments and notifications. | out-of-scope | Collaboration territory (`docs/scope.md:25`). |
| Portfolios / programme roll-up | P6, MSP | Manage many projects as a portfolio. | out-of-scope | `docs/research/primavera-p6-capabilities.md:96`. |

### 16. Configuration and preferences

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Light / dark theme toggle | MSP, ProjectLibre | Switch the whole UI theme. | shipped | `Ctrl+D` (`docs/usage.md:168`); `docs/parity.md:88`. |
| Preferences / settings dialog | MSP, P6 | A central options dialog. | missing | No settings surface; theme is the only user preference. |
| Persisted audit threshold profiles | Audit, P6 | Save named threshold sets. | missing | `AuditConfig` is in code only (`audit.rs:60`); gap #22. |
| Currency / units display options | MSP, P6 | Choose currency, unit and duration displays. | missing | No display preferences; money is minor units shown as decimals (D13). |
| Locale / number formatting | MSP, P6 | Locale-aware formatting. | missing | Dates are ISO `YYYY-MM-DD` only (`interchange/lib.rs:112`). |
| Per-project defaults | P6 | Defaults stored with the project. | partial | The project carries a default calendar; little else (`model.rs:768`). |

### 17. Accessibility

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Screen-reader support (AccessKit) | MSP, P6 | Standard widgets exposed to assistive technology. | shipped | AccessKit via eframe; `docs/parity.md:89`; D23 `DECISIONS.md:445`. |
| Keyboard-only operation | MSP, P6 | Every action reachable without a pointer. | shipped | Pointer-free link creation, Gantt zoom, `F1` map (`docs/parity.md:89`). |
| WCAG AA contrast | MSP, P6 | Text meets AA contrast in both themes. | shipped | `accessibility_tests` (`docs/parity.md:89`); tests at `ui-egui/lib.rs:3878`. |
| Per-bar Gantt AccessKit nodes | MSP, P6 | Each bar is a discrete accessible node. | partial | Deliberately not added; a summary node + Table alternative instead (`DECISIONS.md:454`). |
| Live-region status text | MSP, P6 | Status changes announced. | missing | Recorded gap (`DECISIONS.md:466`); status text is not a live region. |
| Manual screen-reader release passes | MSP, P6 | Verified on each platform's screen reader. | missing | Recorded as a manual, per-platform gap (`docs/parity.md:89`). |

### 18. Keyboard

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Undo / redo | MSP, P6 | Global undo/redo from any view. | shipped | `Ctrl+Z`/`Ctrl+Y` (`docs/usage.md:159`); `CommandStack`. |
| View switching keys | MSP, P6 | Numeric keys switch views. | shipped | `Ctrl+1…Ctrl+8` (`docs/usage.md:161`); `ui-egui/lib.rs:1376`. |
| Save / open keys | MSP, P6 | `Ctrl+S`/`Ctrl+O`. | shipped | `ui-egui/lib.rs:1340`; `docs/usage.md:155`. |
| Gantt zoom keys | MSP, P6 | Keyboard zoom in/out/reset. | shipped | `Ctrl+Plus`/`Ctrl+Minus`/`Ctrl+0` (`docs/parity.md:88`). |
| Row navigation | MSP, P6 | Arrow keys move the selection. | shipped | `Up`/`Down` (`docs/usage.md:171`). |
| Shortcut reference window | MSP, P6 | An in-app shortcut list. | shipped | `F1` (`docs/usage.md:174`). |
| Configurable keybindings | MSP, P6 | Remap shortcuts. | missing | A keymap config layer; shortcuts are hard-coded (`ui-egui/lib.rs:1376`). |

### 19. CLI and scripting

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Headless schedule table | MSP, P6 (tooling) | Print the schedule without a GUI. | shipped | `cadence`/`cadence load` (`main.rs:30`, `:32`). |
| Save / load project | MSP, P6 | Persist and reopen a project. | shipped | `cadence save`/`load` (`main.rs:31`, `:32`). |
| Audit / compare commands | Audit, P6 | Run quality checks headless. | shipped | `cadence audit`, `cadence audit-compare` (`main.rs:33`, `:35`). |
| Report / print commands | MSP, P6 | Write Markdown/PDF headless. | shipped | `cadence report`, `cadence print` (`main.rs:37`, `:38`). |
| Interchange commands | MSP, P6 | Import/export all formats headless. | shipped | `export-*`/`import-*` (`main.rs:42`–`:49`). |
| Update / version commands | MSP, P6 | Report version; check for updates. | shipped | `cadence version`, `cadence check-updates` (`main.rs:39`, `:40`). |
| Documented exit codes | MSP, P6 (tooling) | 0 success, 1 error, 2 usage. | shipped | `main.rs:52`. |
| Macros / scripting / plugin system | MSP, P6 | Automate via macros or a plugin API. | out-of-scope | `docs/scope.md:29`. |
| Batch / multi-file operations | MSP, P6 | Operate on many projects at once. | missing | A multi-file command mode; each command takes one file today. |

### 20. API and integrations

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Enterprise REST / SDK (EPPM) | P6 | An API for enterprise integration. | out-of-scope | `docs/research/primavera-p6-capabilities.md:97`. |
| ArcGIS / Power BI integrations | Aphex | Spatial and dashboard integrations. | out-of-scope | `docs/research/aphex-integration.md:83`. |
| Update check (GitHub Releases) | MSP, P6 | Notify of a newer release. | shipped | `cadence check-updates`, GUI **Updates**; D22 `DECISIONS.md:414`. |
| Crash reporting (opt-in issue) | MSP, P6 | Pre-filled issue with consent. | shipped | `crates/app/src/crash_report.rs`; `docs/usage.md:129`. |
| Project-data network API | P6, Aphex | Exchange programme data over a network. | out-of-scope | Consent-gated, recorded-not-planned (`docs/research/aphex-integration.md:83`). |

### 21. AI assistance (optional, off-by-default, queued)

| Capability | Reference product(s) | Observable behaviour (clean-room) | Cadence status | Evidence |
|---|---|---|---|---|
| Programme advice (grounded Q&A) | Planera, nPlan, Copilot | Answer questions about float, logic and the critical path, cited to the data. | missing | Design only; owner-approved, queued (`docs/parity.md:102`; `docs/research/ai-integration.md:13`). |
| Draft / skeleton WBS and activities | Copilot, nPlan | Generate a preliminary WBS/task plan from a description or documents. | missing | Design only (`docs/research/ai-integration.md:17`). |
| Activity enrichment | nPlan, AI | Propose descriptions, durations, resources or missing logic. | missing | Design only (`docs/research/ai-integration.md:21`). |
| Status updates from notes | Copilot, Aphex | Turn site notes/emails into proposed progress updates. | missing | Design only (`docs/research/ai-integration.md:23`). |
| Report drafting | Copilot | Draft a narrative status report from the schedule. | missing | Design only (`docs/research/ai-integration.md:25`). |
| Document reading (PDF/email/text/CSV) | Trunk Tools, Document Crunch | Read and cite the project document set. | missing | Local index design only (`docs/research/ai-integration.md:71`). |
| Provider-agnostic, bring-your-own key | nPlan, Copilot | A configurable model provider. | missing | One small trait design only (`docs/research/ai-integration.md:76`). |
| Reviewable, undoable command proposals | Copilot, Planera | Proposed changes are a review diff, never a silent edit. | missing | Design only (`docs/research/ai-integration.md:83`). |
| Risk forecasting from past schedules | nPlan | ML duration/risk forecasts. | missing | No model/dataset; the design is a general assistant, not a vendor risk model (`docs/research/ai-integration.md:144`). |
| Generative schedule optimisation | ALICE | Optimise many schedule options against constraints. | missing | Out of the current AI note's scope (not named as a Cadence direction). |
| AI inside the diary | Aphex | Sense-making over site records. | missing | The AI note places the assistant beside the diary; queued (`docs/research/ai-integration.md:52`). |

## Deliberately out of scope

What Cadence should **not** build, drawn from `docs/scope.md` and each research
note's out-list. Each is a deliberate decision, not an omission.

- **Proprietary file formats** — `.mpp` (would need reverse engineering),
  P6 XER and Oracle database schemas. Clean-room rule
  (`docs/research/ms-project-capabilities.md:100`,
  `docs/research/primavera-p6-capabilities.md:94`).
- **Enterprise structure** — EPS, portfolios, multi-project resource pools,
  role-based enterprise resource management, the EPPM REST/SDK surface
  (`docs/research/primavera-p6-capabilities.md:96`).
- **Cloud and multi-user** — Project Server/Online, cloud sync, hosted
  read-only share links, concurrent multi-user editing, comments/notifications
  (`docs/scope.md:25`, `docs/research/aphex-integration.md:77`).
- **Cost-system accounting** — store-period-performance freezing of financial
  actuals (`docs/research/primavera-p6-capabilities.md:99`).
- **Mobile applications** — iOS/Android (`docs/scope.md:27`).
- **ERP / cost-accounting integration** (`docs/scope.md:28`).
- **Macros, scripting, plugins** (`docs/scope.md:29`).
- **Manually scheduled tasks and mandatory constraints** — Cadence is
  deliberately logic-driven (D15, `DECISIONS.md:199`; D10, `DECISIONS.md:72`).
- **A real GIS / map layer** — light location tags and clash detection only
  (`docs/research/aphex-integration.md:71`).
- **An Aphex connector or compatibility claim** — no vendor connector
  (`docs/research/aphex-integration.md:122`).
- **ArcGIS / Power BI integrations and a hosted API** — recorded, not planned
  (`docs/research/aphex-integration.md:83`).
- **Telemetry** — none (`ROADMAP.md:246`, `VISION.md`).
- **AI touching the CPM core** — AI may explain a computed finding but never
  produce or change a date; no automatic network call; no training on user
  content (`docs/research/ai-integration.md:59`).
- **WASM / web build** — deferred candidate (`docs/scope.md:35`).
- **Advanced resource leveling and longest-path criticality** — deferred
  post-v1 candidates (`docs/scope.md:37`, `:38`).

## Method and caveats

- **Clean-room.** This is a behaviour comparison only. Cadence is inspired by
  publicly documented concepts and observable behaviour; no proprietary code,
  resource, icon, layout, string or algorithm was consulted
  (`docs/inspiration.md` §Clean-room protocol). The "observable behaviour"
  column restates public research notes in Cadence's own words.
- **Statuses are evidence-based.** `shipped` means implemented **and** reachable
  from a GUI or CLI surface; `partial` means implemented in part, or present in
  core with no user surface (stated in the Evidence cell); `missing` names the
  build path and whether `ROADMAP.md` already plans it; `out-of-scope` cites the
  scope decision.
- **No capability is invented.** Where Cadence has no equivalent, the row says
  so; where a Cadence-only capability exists (the extended DCMA audit, the
  idempotent offline CLI), it is named as such.
- **Counts** in the Summary are an automated tally of the reference and status
  columns; a row that names several reference products is counted under each.
- **Line references** were taken at the time of writing (2026-10-30) and are
  for orientation; the surrounding section is authoritative.
- **Queued ≠ shipped.** The audit tool, the Aphex-inspired lookahead and the AI
  assistant were owner-approved on 2026-10-27 (`docs/parity.md:93`); only what
  is actually implemented is marked `shipped` or `partial` here.
