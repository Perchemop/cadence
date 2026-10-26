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
| `cadence [--json]` | Run the demo project and print its schedule |
| `cadence save <file>` | Save the demo project to a project file |
| `cadence load <file> [--json]` | Load a project file and print its schedule |
| `cadence report <file> [out]` | Write a Markdown report (stdout if no file) |
| `cadence print <file> <out.pdf>` | Print the report as a PDF |
| `cadence version` | Show the version and the last cached update check |
| `cadence check-updates` | Ask GitHub Releases for the latest version |
| `cadence gui [file]` | Open the project in the GUI (demo if no file) |
| `cadence export-xml <in> <out>` | Microsoft Project XML (MSPDI) |
| `cadence import-xml <in> <out>` | Import MSPDI |
| `cadence export-mpx <in> <out>` | MPX |
| `cadence import-mpx <in> <out>` | Import MPX |
| `cadence export-csv <in> <out>` | CSV activity table |
| `cadence import-csv <in> <out>` | Import CSV |
| `cadence export-json <in> <out>` | Cadence JSON (lossless) |
| `cadence import-json <in> <out>` | Import Cadence JSON |

## Machine-readable output

`cadence --json` and `cadence load <file> --json` print the schedule as one
JSON document, so that scripts and assistive technology (guide section 21.5,
accessibility of tooling) never have to parse the aligned plain-text table.
The top-level `cadence` member is the document's own schema version (1), and
`kind` says which document it is:

* `kind` is `"schedule"` when the engine succeeded. `project` carries the
  name and the first and last working days; `activities` lists one entry per
  scheduled activity, in project order — `id`, `name`, `duration_days` (in
  working days; 0 is a milestone), the `scheduled`, `early` and `late`
  start/finish dates, `total_float_days` and `free_float_days` in working
  days, and a `critical` boolean; and `critical_path` lists the critical
  activity ids.
* `kind` is `"error"` when the schedule could not be computed; then
  `error.message` says what went wrong, using the same text the CLI writes
  to stderr.

stdout carries only the document, with one trailing newline; everything a
human needs goes to stderr as diagnostics of the form
`ERROR 2026-10-25T12:34:56Z cadence: <message>`. Exit codes: 0 success,
1 error, 2 usage error.

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
| MSPDI (XML) | Activities, constraints, links, resources, assignments, baselines | Durations convert at 8 h/day; lag is quantised to days and progress to whole percents (D14); no WBS or named calendars |
| MPX | Same scope as MSPDI, minus money and baselines | Older Microsoft Project Exchange dialect |
| CSV | One activity per row | Links, resources and progress folded into the row; no baselines, WBS, calendars or resource kinds |
| JSON | Everything Cadence models: WBS tree, named calendars, activities, links, resources, assignments, baselines | Cadence's own schema, versioned in the document, lossless |

Money is stored in minor units and shown as decimal amounts; percentages are
basis points in the model and percentages everywhere else (D13).

## Reports

`cadence report <file>` renders a Markdown report: a schedule table with
dates, float and criticality; per-resource committed and over-allocated days;
earned-value metrics against the most recent baseline; and the baseline list.
`cadence print <file> <out.pdf>` renders the same report as a print-ready PDF
(heading and body text, wrapped and paginated over A4 pages). In the GUI,
**Save report** writes the Markdown next to the working directory,
**Print report…** opens a native save dialog for the report PDF, and
**Print Gantt…** (enabled in the Gantt view) captures the current Gantt
timeline as a single landscape A4 page in a PDF.

## Crash reports

If Cadence panics, the panic hook appends the version, operating system, panic
message and a backtrace to a local crash log in the system temporary
directory. Nothing is uploaded. On the next launch the GUI shows the log in a
**Crash report** window with an optional notes box; **Open GitHub issue** opens
a pre-filled issue in your browser, **Copy details** copies the preview, and
**Dismiss** deletes the local log. The **Report a bug…** toolbar button opens
the same window even when there is no crash log.

## Update checks

Update checks are opt-in. `cadence version` prints the running version and the
last cached answer but never reaches the network. `cadence check-updates` asks
the public GitHub Releases API for the latest release and prints whether an
update is available and where to get it. In the GUI, **Updates** opens a small
window with the same information and a **Check now** button; when a newer
release is known, the toolbar button changes to **Update available: `<version>`**
and the window links to the release page.

The check is one bounded GET request with a `Cadence/<version>` user agent. It
sends no project data, account, identifier or token. A successful answer is
cached in the system temporary directory for six hours, so repeated checks do
not hit the API, and an offline or failed check is reported in the window
without blocking the application.

## Keyboard map (GUI)

| Keys | Action |
|---|---|
| Ctrl+Z | Undo |
| Ctrl+Shift+Z or Ctrl+Y | Redo |
| Ctrl+1 | Activity table |
| Ctrl+2 | Gantt |
| Ctrl+3 | Resources |
| Ctrl+4 | Baselines |
| Ctrl+5 | Calendars |
| Ctrl+D | Toggle light/dark theme |
| Ctrl+Plus / Ctrl+Minus | Zoom the Gantt timeline in or out |
| Ctrl+0 | Reset the Gantt zoom |
| Up / Down | Select the previous/next activity in the Activity table |
| Tab / Shift+Tab | Move focus between controls |
| Space / Enter | Activate the focused control or selectable row |
| F1 | Open the keyboard-shortcuts window |

## Views

* **Table** — the sortable activity list and inspector (details, constraint,
  calendar, progress, resources, relationships). The inspector's **Add
  relationship** form creates dependency links without dragging.
* **Gantt** — timeline with bars, dependency arrows, zoom and a progress
  overlay. Drag between bars to create dependencies, or use the inspector's
  **Add relationship** form; the zoom buttons and `Ctrl+Plus` / `Ctrl+Minus`
  work without a pointer. A text summary above the chart names the activity
  and critical counts, and the Table view lists the same data as text.
* **Resources** — dictionary, capacity editing, assignments, per-day usage,
  leveling preview and apply.
* **Baselines** — capture and delete baselines, variance against the selected
  baseline, and earned-value metrics at a status date.
* **Calendars** — named calendars with working days and holidays, create /
  rename / delete, and per-activity assignment (a calendar cannot be deleted
  while an activity uses it).

Reports live outside the views: `cadence report` writes Markdown and
`cadence print` writes a PDF, both from any project file.

## Examples

`cargo run -p cadence-app --example bridge_report` builds the sample bridge
project, prints its report, and checks a JSON round trip.
