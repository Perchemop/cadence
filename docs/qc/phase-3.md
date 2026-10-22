# Phase 3 Quality Gate

Checklist for the Phase-3 resources slices, the Phase-4 baselines, EVM and
progress slices that landed with them, and the Phase-5 interchange slice.

## Functionality

- [x] Resource dictionary: kinds, capacities, unique case-insensitive names,
      undoable create/rename/set-capacity/set-kind/delete (P3-1).
- [x] Resource assignments on activities with correctly-inverted
      assign/unassign commands (P3-1).
- [x] Usage histograms: committed resource units per working day (P3-2).
- [x] Over-allocation detection reporting each day over capacity (P3-3).
- [x] Greedy resource leveling that preserves dependencies and constraint
      bounds, with a bounded iteration budget (P3-3).
- [x] Functional resource panel: dictionary table, editing, assignment
      toggles, per-day usage against capacity, leveling preview and an
      undoable apply action (P3-3).
- [x] Multiple baselines with per-activity snapshots and undoable
      capture/remove/restore (P4-1).
- [x] Earned value metrics: PV, EV, AC, SV, CV, SPI and CPI in minor units and
      basis points (P4-3).
- [x] Baselines view: capture and delete baselines, a per-activity variance
      table against the selected baseline, and an earned-value panel driven by
      an editable status date (P4-1, P4-3).
- [x] Progress updating: percent complete in basis points, budget at
      completion, actual cost and recorded actual start/finish dates, all
      editable in the inspector through undoable commands (P4-2).
- [x] Gantt bars draw a progress overlay over the completed share (P4-2).
- [x] Microsoft Project XML (MSPDI) import and export, including resources,
      assignments, baselines and actual dates (P5-1).
- [x] MPX import and export (P5-1).
- [x] CSV activity table import and export, with predecessor and resource
      lists, progress and actual dates (P5-1b).
- [x] SQLite schema v4 with in-place v1, v2 and v3 migrations (P3-1, P4-2).

## Engineering

- [x] `cargo fmt --check --all` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes.
- [x] CI green on `opencode/ci` (run 39844511576, 166 tests passed), pending
      merge to `main`.

## Known limitations (carried forward)

- Leveling is a greedy heuristic, not an optimiser: it can delay an activity
  more than strictly necessary, and it reports remaining conflicts rather
  than failing when the iteration budget runs out (D12).
- An assignment always consumes exactly one resource unit per working day
  (D11); per-assignment units and cost rate tables are not modelled.
- MSPDI durations assume the Microsoft Project eight-hour default day, and
  lag is quantised to whole working days (D14).
- The MSPDI reader ignores namespaces and prefixed element names.
- Recorded actual dates do not drive the schedule; they are kept as facts for
  variance and earned-value comparison (D15).
