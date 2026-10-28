# Research — Microsoft Project capability inventory

Status: draft, not human-approved as *new* research. Microsoft Project is
already a reference product in `docs/inspiration.md`; this note is the
clean-room capability inventory the parity review reads (guide §II.27.3). It
records observable behaviour with public URLs — original summaries only. Where
Cadence already ships a capability, `docs/parity.md` carries the evidence.

## Purpose and method

Microsoft Project is the desktop scheduling product Cadence is most often
compared with. This inventory enumerates what it *does*, from official
documentation and public pages, so that parity claims about "what MS Project
does" are traceable. Nothing here is copied: no UI, strings, icons, layouts or
algorithms — behaviour descriptions and URLs only.

## Capabilities observed

### Scheduling

- **Work breakdown structure**: summary tasks and subtasks form the outline;
  durations, links and constraints hang off the task rows.
- **Dependency logic**: finish-to-start, start-to-start, finish-to-finish and
  start-to-finish links with lead/lag, editable in tables and on the Gantt.
- **Constraints and deadlines**: flexible constraints (start/finish no earlier
  than) and inflexible ones (must start/finish on, no later than), plus
  deadline fields that show slack without constraining the schedule.
- **Task calendars and milestones**: per-task calendars and zero-duration
  milestones.
- **Critical path**: highlighted in the Gantt; the Tracking Gantt overlays the
  baseline; Task Path and the Task Inspector pane show driving logic per task.
- **Manually scheduled tasks** ("user-controlled" summary/task dates) alongside
  automatically scheduled ones.

### Baselines and tracking

- **Multiple baselines** (up to eleven sets of baseline start/finish/work/cost
  fields) plus interim plans.
- **Progress entry**: percent complete, actual start/finish, actual and
  remaining duration, status date.
- **Variance surfaces**: Tracking Gantt overlay, variance tables, progress
  lines.

### Resources and cost

- **Resource types**: work, material and cost resources; availability and
  cost-rate tables (standard, overtime, per-use); resource calendars.
- **Over-allocation**: indicators in views, with leveling done automatically
  or manually, with options for order, granularity and whether leveling can
  split tasks.
- **Usage views**: Task Usage and Resource Usage time-phased grids.

### Earned value

- Budgeted and earned values (BCWS/BCWP), actual cost (ACWP), schedule and
  cost variance, SPI/CPI, estimate fields (EAC/ETC/VAC) and TCPI — measured
  against a baseline and a status date.

### Views, reporting and comparison

- **Views**: Gantt variants (including Tracking Gantt), Network Diagram,
  Timeline, calendar view, task/resource usage.
- **Reporting**: built-in dashboards and report templates; visual reports to
  Excel/Visio; print and PDF.
- **Compare Projects**: a two-version comparison report listing the
  differences between the current project and an earlier or later version,
  with colour coding and indicators; the report contains only differences.

### Interchange

- Proprietary `.mpp` (binary; documented only by the vendor), **MSPDI XML**
  (documented dialect), legacy MPX, CSV/Excel export, and
  Project Server/Online storage (subscription).

### Assistive/AI layer

- **Copilot in Project** (subscription plans): generates a preliminary
  WBS/task plan from a name and description, drafts risk assessments and
  status reports, and takes plain-language commands. See the AI research note
  for how Cadence would approach this differently.

## Where Cadence already stands

| Capability | Cadence |
|---|---|
| WBS, dependency logic, constraints, milestones, per-activity calendars | shipped |
| Multiple baselines, variance views, status date | shipped |
| Earned value (PV/EV/AC, SV/CV, SPI/CPI, variances) | shipped (subset) |
| Resources, over-allocation detection, greedy leveling | shipped (one-unit model, D11) |
| Gantt + table, keyboard-first UI, light/dark | shipped |
| MSPDI / MPX / CSV / JSON interchange | shipped (MPP deliberately out — clean-room rule, see `docs/scope.md`) |
| Critical path highlighting, Tracking-Gantt-style overlay | shipped (Gantt critical bars, progress fill) |
| Compare Projects (two-version diff) | audit tool's forensic stage, pending |
| Audit/schedule-health checks | shipped core + CLI + GUI view |
| Copilot-style assistance | researched, queued (`docs/research/ai-integration.md`) |
| Manually scheduled tasks, deadline-only fields | deliberately out: Cadence is logic-driven (D15); deadlines may be revisited |

## Deliberately out of scope

- The proprietary `.mpp` file format (would require reverse engineering —
  forbidden by the clean-room rules in `CONTRIBUTING.md`).
- Project Server/Online, enterprise resource pools, portfolios, Teams
  integration — cloud/enterprise territory, out of scope for a single-file
  offline application (`VISION.md` non-goals).

## Sources

- Critical path: https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project
- Compare two versions of a project: https://support.microsoft.com/en-us/project/compare-two-versions-of-a-project
- Set baseline options: https://learn.microsoft.com/en-us/answers/questions/5248498/set-baseline-options
- Copilot for project overview and responsible-AI FAQ: https://learn.microsoft.com/en-us/dynamics365/project-operations/project-management/copilot-features , https://learn.microsoft.com/en-us/dynamics365/project-operations/responsible-ai/copilot-for-project-faq
- Feature overview (secondary, third-party): https://businessanalyst.techcanvass.com/features-of-microsoft-project
- Third-party audit add-in for Microsoft Project: https://www.barbecana.com/schedule-inspector-2
