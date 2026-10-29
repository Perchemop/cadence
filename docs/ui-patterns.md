# UI pattern reference matrix — professional desktop schedulers

Clean-room record. Every row below is an **observable behaviour of a publicly
documented product** (help pages, public walkthroughs, videos of shipping
releases), restated in Cadence's own words. Nothing here is code, UI layout,
icons, strings, or internal algorithms copied from another product — only
behavioural concepts that Cadence may or may not adopt, expressed as original
design intent.

Status values: `adopted` (in Cadence), `planned` (in a roadmap slice),
`considering`, `out of scope`.

## MS Project (desktop)

| Behaviour | Reference product does | Cadence status | Notes |
|---|---|---|---|
| Combined task sheet + Gantt in one window | Split view with draggable divider; task table left, Gantt right; both scroll together vertically | planned | Split table + Gantt (the "workbench") in this slice; synced selection is a follow-up |
| View tabs | Row of named view tabs (Gantt Chart, Task Sheet, Resource Sheet, ...), one active | adopted | Tab strip in the top toolbar replaces the flat button row |
| Column headers with sorting | Click a column header to sort the task sheet | adopted | Sort buttons above the virtualised table |
| Row numbers | Implicit task ID numbers in the first column | adopted | `#` column in the virtualised table |
| Fixed row height, word-wrap off | Compact monospace rows, one line per task | adopted | Virtualised rows at 20 px with truncated names |
| Footer status bar | Bottom bar with task counts, status date, zoom | adopted | New status bar: dates, activity count, critical count, zoom |
| Zoom in/out buttons and % readout | Timeline zoom controls next to the timescale | adopted | Gantt zoom buttons already present; % shown in toolbar and status bar |
| Critical path shown as red bars | Critical tasks highlighted in the Gantt | adopted | Red bars, `!` name marker, "Critical" column |
| Today line | Vertical line at today's date on the timescale | adopted | Accent-coloured today line in Gantt |
| Weekly grid lines | Vertical hairlines at week boundaries | adopted | Low-contrast weekly grid |
| Drag to create dependency links | Drag from one task bar to another | adopted | Gantt drag-to-link, now O(relationships) |
| Undo/redo from any view | Global Undo/Redo | adopted | Toolbar + Ctrl+Z/Y |

## Primavera P6

| Behaviour | Reference product does | Cadence status | Notes |
|---|---|---|---|
| Activity table + Gantt in one workspace | "Layouts" combine a configurable column table with a Gantt canvas | planned | Workbench continues P6's configurable-column idea; column chooser is a later slice |
| View/tab switching at the top | Tabbed workspaces with a toolbar per workspace | adopted | View tab strip |
| Groups, sort, filters above the table | Filter/sort/group bar sits over the activity table | considering | Currently sort only; filters/grouping are a later slice |
| Critical path highlighting | Critical activities flagged in table and Gantt | adopted | `Critical` column + red bars |
| Status date line | Vertical status-date line used for earned value | considering | Today line exists; status-date line is part of baselines/EVM later |
| Float column | Total float shown as a table column | adopted | Float column |

## Asta Powerproject

| Behaviour | Reference product does | Cadence status | Notes |
|---|---|---|---|
| Coloured task bars by status/criticality | Bars coloured by progress/criticality | adopted | Critical vs normal bar colours, progress overlay |
| Responsible owner per task | Owner field shown in the table/inspector | adopted | Lookahead fields (owner/promise/delay reason) already shipped |
| Progress as a bar overlay | Percent-complete drawn inside the bar | adopted | Darker fill inside Gantt bars |

## ProjectLibre

| Behaviour | Reference product does | Cadence status | Notes |
|---|---|---|---|
| Gantt + table side by side | Split pane with table on the left | adopted/planned | Same as MS Project row above |
| Timeline at the top of the Gantt | Timescale header above the canvas | adopted | Week grid + date ticks at top |

## Aphex (short-interval / lookahead planning)

| Behaviour | Reference product does | Cadence status | Notes |
|---|---|---|---|
| Rolling weekly lookahead window | Window of N weeks with owner + promise per activity | adopted | `Ctrl+7` Lookahead view (PRs #42, #43, #45) |
| Daily diary / huddle briefing | Day-by-day view of promised work | planned | Remaining lookahead stage |
| Production tracking | Percent complete vs plan per activity | planned | Remaining lookahead stage |
| Location clash detection | Warn when two activities claim the same location | planned | Remaining lookahead stage |

## Layout direction adopted for Cadence

The workbench shell — a scrollable activity table with sticky sortable headers
and virtualised rows on the left, a Gantt canvas on the right separated by a
draggable splitter, view tabs across the top, and a status bar across the
bottom — follows the shared layout language of MS Project, P6, Asta and
ProjectLibre. Cadence's own contribution is the tokenised theme, the monospaced
numeric columns, and the restrained colour semantics defined in
`docs/design/brand.md`.
