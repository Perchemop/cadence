# Research — programme audit tool (schedule quality and health)

Status: **approved by the owner (2026-10-27)** for implementation, as the first
of the three future-expansion notes. This note is the acceptance-criteria source
for the audit slice.

## Behaviour wanted

A built-in tool that **audits a construction programme inside Cadence**: it
runs a battery of schedule-quality and health checks over the current project
and reports, per check and per activity, whether the programme passes, warns or
fails, with the measured value, the threshold, a plain-language explanation and
a suggested fix. It should be usable both while planning (before execution) and
while updating (against a baseline and a status date). Nothing leaves the
machine.

Two entry points, one engine:

- A **GUI Audit view** showing a summary dashboard (per-check score and
  severity) and a findings list where selecting a finding jumps to the activity
  in the table/Gantt. *(Still to build.)*
- A **`cadence audit <file> [--json]`** command for scripted or headless
  checks, writing the same report (text and the `--json` document).
  *(Delivered.)*

## What professional and paid tools provide (public behaviour only)

Summaries below are original wording from public product/help pages; URLs are in
the Sources section. No screenshots, strings, layouts or algorithms are copied.

### Oracle Primavera P6 — Check Schedule

- Built into P6 Professional (from r22): evaluates a programme against most of
  the DCMA 14-point metrics and adds a few of its own (long lags, soft
  constraints, late activities), with configurable thresholds.
- Produces a schedule-check report and the traditional **schedule log**, which
  describes scheduling conditions encountered during the run.
- Public reviews note it is *incomplete* against DCMA 14: it does not implement
  Missed Tasks, the Critical Path Test or the Critical Path Length Index.

### Deltek Acumen Fuse / Acumen

- A schedule-quality and diagnostics workbench with a very large metric library
  (hundreds of metrics) grouped into DCMA, DOE, GAO, NASA and AACE-aligned
  sets, plus the vendor's own "Schedule Quality" playlist.
- **Schedule Cleanser**: a first pass that flags a handful of common defects and
  can correct some automatically.
- **Logic checks**: open ends, leads, lags, redundant logic (two paths to the
  same successor), reverse logic, circular logic, logic on summary activities,
  out-of-sequence progress, open starts/finishes.
- **Logic Trace / Logic Sensitivity**: trace driving paths; nudge an activity by
  a set number of days and see the downstream impact (the same idea as the DCMA
  Critical Path Test).
- **Forensic comparison** of two or more programme updates: logic, calendar,
  duration and progress changes, with delay attribution ("half-step" analysis)
  to produce audit documentation.
- **Ribbon/Phase/Intersection analyzers** and dashboards for trends across
  updates or a portfolio; export to Excel and executive briefing reports.
- A newer AI layer surfaces risks and mitigation suggestions.

### SSI Schedule Analyzer

- A long-established analyst suite for P3/P6 with **Baseline Checker** and
  **Update Checker** (two-schedule comparison), a dashboard, a schedule logger
  per activity, longest-path values for every activity, and a large library of
  reports (200+), with reports that jump back to the activity in the native
  tool.
- SSI Tools' separate integrity module exposes 14 named audits (no predecessor/
  successor; logic on summaries; invalid status; progress out of sequence;
  unnecessary date constraints; negative slack; very high total slack; no
  baseline; negative lag; start/finish late to baseline; child/parent baseline
  date, cost and work mismatches) and an **Audit Builder** so a team can add its
  own checks.

### Steelray Analyzer (for P6 / for Microsoft Project)

- A schedule health and performance analysis with a **scorecard** (green/yellow/
  red), **trend lines** across updates, DCMA 14 and DECM checks, configurable
  thresholds, an explanatory encyclopedia ("The Coach"), and Excel export.

### Barbecana Schedule Inspector

- A Microsoft Project add-in that checks DCMA and AACE (57R-09) best-practice
  metrics, with selectable and customisable tests, colour-coded results,
  printing/copy, and the ability to highlight offending tasks back in Project.

### Web / no-native-licence audit services

- **ScheduleLens**, **SmartPM**, **ScheduleReader** and **Onplana** parse P6
  (XER/XML), Microsoft Project (MPP/XML) and/or Asta Powerproject and produce a
  DCMA-aligned grade, critical-path trace, float distribution, delay/EOT
  analysis and (in some) an AI-written narrative; several are priced per report
  and delete the uploaded file afterwards.
- **ScheduleGate** (GitHub) is a public open-source example of a DCMA
  14-point-style analyser for Integrated Master Schedules.

## The standards the tools measure against (public)

### DCMA 14-point assessment

A tool-agnostic first pass on programme construction and performance. The
commonly used checks and default thresholds:

| # | Check | Flags | Default pass |
|---|---|---|---|
| 1 | Logic | Open activities with no predecessor, no successor, or neither (zero-duration start/finish milestones are exempt) | ≤ 5% |
| 2 | Leads | Relationships with negative lag | none |
| 3 | Lags | Relationships with positive lag | ≤ 5% |
| 4 | Relationship types | Non-finish-to-start relationships | ≥ 90% FS |
| 5 | Hard constraints | Must-start/finish or start/finish-no-later-than with a date | ≤ 5% |
| 6 | High float | Total float above ~44 working days | ≤ 5% |
| 7 | Negative float | Total float below zero | none |
| 8 | High duration | Activities longer than ~44 working days | ≤ 5% |
| 9 | Invalid dates | Forecast dates before, or actual dates after, the status date | none |
| 10 | Resources | Non-milestone activities with no resource assigned (when resource-loaded) | none |
| 11 | Missed tasks | Activities that finished/will finish after their baseline finish | ≤ 5% |
| 12 | Critical path test | Nudge one critical activity by ~600 working days and reschedule | finish moves by the full nudge |
| 13 | CPLI | Critical-path length index (path + float to deadline) ÷ path | ≥ 0.95 |
| 14 | BEI | Baseline execution index (activities done ÷ baseline-due) | ≥ 0.95 |

### GAO Schedule Assessment Guide — ten best practices

Capturing all activities; sequencing all activities; assigning resources;
establishing durations; verifying horizontal and vertical traceability;
confirming the critical path; ensuring reasonable total float; conducting a
schedule risk analysis; updating with actual progress and logic; maintaining a
baseline schedule.

### AACE International recommended practices

Useful reference points for specific checks: RP 49R-06 (identifying the
critical path), RP 92R-17 (near-critical paths), RP 53R-06 (schedule update
review), RP 29R-03 (forensic schedule analysis), RP 57R-09 (integrated cost and
schedule risk analysis with Monte Carlo).

## How Cadence could implement it (original design direction)

- A pure module in `cadence-core` (`audit.rs`): `audit(project, &schedule,
  &AuditConfig) -> AuditReport`. The engine already exposes everything the
  core checks need: logic, relationship types and lag, constraints, early/late
  and scheduled dates, total/free float, criticality, resources, baselines,
  actual dates, percent complete and EVM. No model change is required for the
  first version; a status date already exists for EVM.
- Checks are **data**, not a monolith: a stable check ID, a category
  (structure, logic, float, constraints, dates, resources, baseline/
  performance, critical path), a default threshold, a severity, and a function
  that returns the offending activities with their measured values.
- Defaults follow the DCMA thresholds above but every threshold is
  configurable, because contractors, owners and PMOs use different limits.
  Cadence has no separate milestone object, so the logic check's start/finish
  exemption applies to zero-duration activities: an open start or open finish
  is legitimate for a zero-duration activity, and every other open activity
  needs logic on both sides.
- Findings are actionable: check ID, activity ID and name, measured value,
  threshold, one-sentence explanation and a suggested fix; the GUI links a
  finding to the activity, and the CLI/report prints the same data.
- Report shape: summary per check (count, percentage, pass/warn/fail against
  the threshold) plus the activity-level evidence, and an overall grade. The
  same document is emitted as human text, Markdown/PDF and the machine-readable
  `--json` schema.
- Works on the internal model, so it can audit a program created in Cadence and
  any program imported from MSPDI, MPX, CSV or Cadence JSON.
- Phased delivery, smallest first: (1) structure/logic/float/constraint/date
  checks *(delivered)*; (2) resource and baseline-performance checks, missed
  tasks and BEI *(delivered)*; (3) critical-path checks — the critical path
  test and CPLI are delivered, near-critical paths are not yet; (4) the
  forensic comparison of two versions and of a baseline against the current
  programme is delivered (`cadence audit-compare`, `cadence-core::compare`);
  (5) custom checks and threshold profiles remain open.

## Acceptance criteria (first version)

1. `cadence audit <file>` prints a summary and per-activity findings; `--json`
   emits a documented machine-readable document; the process exits non-zero
   only on an error, not merely on findings (findings are data).
2. The GUI shows the same report in an **Audit** view and selecting a finding
   selects and scrolls to the activity.
3. Every check has a stable ID, a documented default threshold and an
   explanation; thresholds are configurable without rebuilding.
4. The audit is a pure function of the project and schedule: no network, no
   clock dependency beyond the explicit status date, deterministic ordering.
5. A programme with an obvious defect (open end, hard constraint, negative
   float, lead) is detected by the corresponding check and cited with the
   activity; a clean programme passes.

## Edge cases to design for

- Empty projects, single-activity projects and milestones (zero duration).
- Multiple calendars: float and duration thresholds must be measured in each
  activity's own working days (the engine already does).
- ALAP activities and negative float caused by a real deadline versus a hard
  constraint.
- Imported programs with no baseline, no status date or no resources: the
  affected checks must report "not applicable", not a false failure.
- Summary/WBS logic: Cadence's WBS is separate from activities, so "logic on
  summaries" is about relationships that logically belong on a parent roll-up,
  not a distinct task type; define the rule explicitly.
- Redundant and reverse logic are relationship-level findings, not
  activity-level ones; the report needs a relationship finding kind too.
- Very large programmes: the audit must stay linear-ish and not freeze the UI
  (run off the UI thread with the same pattern as resource leveling).
- Percent complete and actual dates may be inconsistent (progress posted out of
  sequence); report it as a finding, never silently rewrite it.

## Clean-room notes

- Only public documentation, public standards and observable behaviour were
  used; no decompilation, no proprietary source, no copied strings, icons,
  layouts or presets.
- DCMA, GAO and AACE are public standards/bodies; the checks are restated in
  Cadence's own words with our own thresholds defaults and our own report
  design.
- Do not reproduce vendor metric libraries, report templates or dashboards.

## Sources

- DCMA 14-point overview and metrics: https://www.deltek.com/resources/articles/dcma-14-point-assessment/
- DCMA 14-point check list and thresholds (neutral summary): https://docs.onplana.com/projects-tasks/check-schedule-quality/
- DCMA metric descriptions: https://learning.deltek.com/bundle/Acumen_Help_System/page/DCMA_14_Point_Assessment_Metric_Descriptions.html
- GAO Schedule Assessment Guide (GAO-16-89G): https://www.gao.gov/products/gao-16-89g
- GAO ten best practices summary: https://www.humphreys-assoc.com/schedule-assessment-guide-best-practices-for-project-schedules
- Oracle Primavera P6 Check Schedule / schedule health score: https://primavera.oraclecloud.com/help/en/user/149131.htm , https://tensix.com/using-the-p6-check-schedule-tool , https://www.taradigm.com/how-to-use-check-schedule-report-in-primavera-p6
- Oracle P6 schedule log: http://www.melaniecalverley.com/interpreting-the-primavera-p6-schedule-log.html
- Deltek Acumen Fuse: https://www.deltek.com/products/delivery-assurance/ppm/acumen/fuse/ and diagnostics process https://help.deltek.com/product/acumen/8.7/ga/The%20Schedule%20Diagnostics%20Process.html
- SSI Schedule Analyzer: https://scheduleanalyzer.com/sa_review.htm and https://www.ssitools.com/helpandsuport/ssianalysishelp/ssi_schedule_integrity_audits.htm
- Steelray Analyzer for P6: https://steelray.com/AnalyzerP6/AnalyzerForP6.php
- Barbecana Schedule Inspector: https://www.barbecana.com/schedule-inspector-2
- Web/no-licence services: https://schedulelens.com/ , https://smartpm.com/ , https://scheduletracker.app/dcma-14-point-schedule-assessment
- Open-source example: https://github.com/gjunqueira-sys/ScheduleGate
- AACE recommended practices: https://web.aacei.org/resources/recommended-practices and https://www.long-intl.com/articles/schedule-analysis-method-2
