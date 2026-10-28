# Research — Primavera P6 capability inventory

Status: draft, not human-approved as *new* research. Primavera P6 is already a
reference product in `docs/inspiration.md`; this note is the clean-room
capability inventory the parity review reads (guide §II.27.3). Original
summaries with public URLs only — no Oracle documentation text is reproduced,
and no proprietary format is reverse engineered.

## Purpose and method

P6 is the enterprise scheduling product construction planners compare Cadence
against. This inventory records what P6 *does*, from Oracle's public help and
public walkthroughs, so parity claims are traceable. Cadence deliberately does
not target the enterprise database, the P6 file formats beyond the documented
XML dialect, or portfolio management.

## Capabilities observed

### Enterprise structure

- **EPS → Project → WBS → Activities** hierarchy: an enterprise project
  structure holds many projects; each project has a WBS tree; activities live
  under WBS nodes.
- **Activity codes** (global / EPS / project) and **user-defined fields**:
  categorise activities from a predefined list of values for grouping,
  sorting, filtering and reporting; UDFs carry free-form attributes.

### Scheduling

- **Activity types**: task-dependent, resource-dependent, level of effort,
  start/finish milestones, and WBS summary activities.
- **Relationships**: FS/SS/FF/SF with lag; the scheduling engine honours
  calendar assignments per activity.
- **Constraints**: a wider set than Cadence's (early/late start/finish,
  mandatory start/finish), with the schedule log documenting how they applied.
- **Calendars**: global, resource and project calendars with working-time
  patterns and exceptions; activities may use any calendar.
- **Critical path / longest path**: P6 distinguishes total-float-based
  criticality from the *longest path* (the driving chain to the project
  finish), which survives constraints that distort float.

### Scheduling quality and logs

- **Schedule log**: a per-run record of the scheduling conditions the engine
  encountered (out-of-sequence progress, constraint effects, open ends).
- **Check Schedule / Schedule Health Score**: built-in DCMA-style checks with
  configurable thresholds and a health score; public reviews note it omits
  Missed Tasks, the Critical Path Test and CPLI — which Cadence's audit now
  includes.
- **Schedule Comparison (formerly Claim Digger) in P6 Visualizer**: a
  two-schedule comparison report (activities added/deleted, changed dates,
  durations, relationships, constraints) between a project and a baseline or
  an update.

### Resources and leveling

- Resource dictionary with roles, resource codes and calendars;
  resource-dependent activities; **leveling** with priority rules, leveling
  dates and "level within total float" options; over-allocation indicators.

### Earned value and periods

- Earned value on resource assignments rolling up to activities and WBS; a
  designated **earned value baseline**; **store period performance** freezes
  each financial period's actuals so cumulative metrics (SV, CV, SPI, CPI,
  EAC) survive later schedule changes.

### Reporting and formats

- **P6 Visualizer**: Gantt charts, timescaled logic diagrams and schedule
  comparisons without a full P6 licence.
- **Formats**: the proprietary XER database export, the documented P6 XML
  (an MSPDI-compatible export exists), and spreadsheets via reports.
- **APIs**: enterprise REST/SDK surfaces for EPPM (integration territory).

## Where Cadence already stands

| Capability | Cadence |
|---|---|
| WBS → activities hierarchy | shipped (single project, no EPS) |
| FS/SS/FF/SF with lag, per-activity calendars | shipped |
| Constraints | shipped (ASAP/ALAP, SNET/SNLT/FNET/FNLT); mandatory start/finish deliberately out |
| Multiple calendars (global + named) | shipped |
| Critical set | shipped (total-float based); longest-path extraction is a recorded post-v1 candidate |
| Schedule log / health checks | shipped as the programme audit (14 checks, GUI + CLI + JSON) — including the three checks P6's built-in tool omits |
| Two-schedule comparison | audit tool's forensic stage, pending |
| Resource dictionary, leveling | shipped (one-unit model D11; greedy leveling D12) |
| Earned value against a baseline | shipped (subset; no period-performance store) |
| Activity codes / UDFs | not built; open question Q2 |
| XER import/export | deliberately out (proprietary format, clean-room rule); MSPDI XML and Cadence JSON are the documented paths |

## Deliberately out of scope

- The proprietary XER format and Oracle database schemas (reverse engineering
  is forbidden).
- EPS, portfolios, multi-project resource pools, role-based enterprise
  resource management, and the EPPM API surface — enterprise/cloud territory
  outside Cadence's offline, single-file model (`VISION.md` non-goals).
- Store-period-performance accounting (a cost-system feature).

## Sources

- Earned value overview (Oracle Primavera Cloud help): https://primavera.oraclecloud.com/help/en/user/233373.htm
- Schedule health score results (Oracle): https://primavera.oraclecloud.com/help/en/user/149131.htm
- Activity codes (P6 EPPM user guide): https://docs.oracle.com/cd/F51301_01/English/User_Guides/p6_eppm_user/6589.htm
- P6 Visualizer overview (P6 Professional user guide): https://docs.oracle.com/cd/G18296_01/English/User_Guides/p6_pro_user/66418.htm
- Check Schedule walkthrough: https://tensix.com/using-the-p6-check-schedule-tool
- Schedule Comparison (formerly Claim Digger) walkthrough: https://tensix.com/primavera-p6s-schedule-comparison-feature-explained
- Interpreting the P6 schedule log: http://www.melaniecalverley.com/interpreting-the-primavera-p6-schedule-log.html
- Using the Check Schedule Report: https://www.taradigm.com/how-to-use-check-schedule-report-in-primavera-p6
