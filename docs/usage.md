# Using Cadence

Cadence is a command-line and desktop application for Critical Path Method
scheduling. This page covers the commands, the file formats, the keyboard
map, and the report output. Design decisions behind all of it live in
[`DECISIONS.md`](../DECISIONS.md).

## Projects and the command line

A project lives in a single SQLite file (schema v4, see D9). Every command
takes the file path; with no arguments `cadence` runs a built-in demo project.

| Command | Effect |
|---|---|
| `cadence` | Run the demo project and print its schedule |
| `cadence save <file>` | Save the demo project to a project file |
| `cadence load <file>` | Load a project file and print its schedule |
| `cadence report <file> [out]` | Write a Markdown report (stdout if no file) |
| `cadence gui [file]` | Open the project in the GUI (demo if no file) |
| `cadence export-xml <in> <out>` | Microsoft Project XML (MSPDI) |
| `cadence import-xml <in> <out>` | Import MSPDI |
| `cadence export-mpx <in> <out>` | MPX |
| `cadence import-mpx <in> <out>` | Import MPX |
| `cadence export-csv <in> <out>` | CSV activity table |
| `cadence import-csv <in> <out>` | Import CSV |
| `cadence export-json <in> <out>` | Cadence JSON (lossless) |
| `cadence import-json <in> <out>` | Import Cadence JSON |

## Scheduling model

* Activities carry a duration in **working days** (zero denotes a milestone),
  a constraint (ASAP, ALAP, SNET, SNLT, FNET, FNLT), optional WBS node,
  optional named calendar, cost fields, and recorded progress (D15).
* Relationships are FS, SS, FF and SF with a signed lag in working days; a
  negative lag is a lead.
* The engine runs a forward and backward pass, reports early/late dates,
  total and free float, and the critical set. Cached schedule fields are
  derived data and are never persisted (D9).
* Resources have a kind (Labour, Equipment, Material) and a capacity in units
  per working day. Each assignment consumes one unit per working day (D11).
* Leveling greedily delays the latest-starting activity on the earliest
  over-allocated day until the schedule is feasible (D12).

## File formats

| Format | Scope | Notes |
|---|---|---|
| Project file (SQLite) | Everything, versioned | The native format; migrations in place |
| MSPDI (XML) | Activities, constraints, links, resources, assignments, baselines | Durations convert at 8 h/day; lag is quantised to days (D14) |
| MPX | Same scope as MSPDI | Older Microsoft Project Exchange dialect |
| CSV | One activity per row | Links, resources and progress folded into the row; no baselines |
| JSON | Everything Cadence models, lossless | Cadence's own schema, versioned in the document |

Money is stored in minor units and shown as decimal amounts; percentages are
basis points in the model and percentages everywhere else (D13).

## Reports

`cadence report <file>` renders a Markdown report: a schedule table with
dates, float and criticality; per-resource committed and over-allocated days;
earned-value metrics against the most recent baseline; and the baseline list.
Reports are the printable path — open the file in a browser and print or
export it to PDF there.

## Keyboard map (GUI)

| Keys | Action |
|---|---|
| Ctrl+Z | Undo |
| Ctrl+Shift+Z or Ctrl+Y | Redo |
| Ctrl+1 | Activity table |
| Ctrl+2 | Gantt |
| Ctrl+3 | Resources |
| Ctrl+4 | Baselines |
| Ctrl+D | Toggle light/dark theme |

## Examples

`cargo run -p cadence-app --example bridge_report` builds the sample bridge
project, prints its report, and checks a JSON round trip.
