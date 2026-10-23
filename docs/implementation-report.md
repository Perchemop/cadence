# Implementation report

Evidence-based summary of what Cadence now implements, how it is verified, and
what is deliberately left open. Every claim below points at a test, a CI run,
or a document; nothing here asserts behaviour that CI has not exercised.

## Scope completed

Phases 0–4 of `IMPLEMENTATION_PLAN.md` are complete and Phase 5 is complete
except for the two items recorded as deliberate deferrals below.

| Phase | Delivered |
|---|---|
| 0 — Foundation | Workspace, CI (the free task runner), command layer, SQLite project files, MIT OR Apache-2.0 dual licence |
| 1 — Engine | WBS hierarchy, FS/SS/FF/SF with signed lag and leads, working-day calendars with holiday exceptions, multiple named calendars, ASAP/ALAP/SNET/SNLT/FNET/FNLT constraints, total and free float, critical set, cycle detection with a path diagnostic, property-based engine tests |
| 2 — Gantt & tables | egui/eframe shell with design tokens and light/dark themes, sortable activity table, inspector edits routed through commands, Gantt view with timeline, bars, dependency arrows, zoom and virtualised rows, drag-to-create dependencies |
| 3 — Resources | Resource dictionary with kinds and capacities, assignments, usage histograms, over-allocation detection per working day, greedy resource leveling, functional Resources view |
| 4 — Baselines & progress | Multiple baselines with activity snapshots, variance view, earned-value metrics (PV, EV, AC, SV, CV, SPI, CPI), progress updating (percent complete, budget, actual cost, actual dates), Gantt progress overlay |
| 5 — Interop & polish | Microsoft Project XML, MPX, CSV and JSON import/export, Markdown reports (CLI and GUI), keyboard map, WCAG-verified colour contrast, performance gates, user documentation and an example project |

## How the agreed scope is implemented

Per-requirement implementation and verification lives in
[`REQUIREMENTS_TRACEABILITY.md`](../REQUIREMENTS_TRACEABILITY.md). Summary:

- **Domain-first, headless.** `cadence-core` holds the model and the engine and
  performs no I/O. The command layer (`cadence-commands`) is the single
  mutation path and every command produces its inverse; undo/redo is tested for
  each command family.
- **Derived data is never trusted.** Schedules are recomputed from logic,
  calendars and constraints; cached values are not persisted (D9) and are
  cleared when an edit invalidates them.
- **Data integrity.** Project files are relational SQLite with
  `PRAGMA user_version` (currently 4) and in-place migrations for every older
  version. Saves are a single transaction; dangling WBS, calendar and
  relationship references are rejected before writing. Round-trip, downgrade,
  migration, duplicate-name and invalid-data cases are unit-tested.
- **Interchange is a documented subset per format.** JSON is lossless and
  versioned (schema 2, carrying the WBS tree, named calendars and membership);
  MSPDI, MPX and CSV carry the plan and convert at documented boundaries
  (D14). All four readers and writers are in-tree — no XML or JSON dependency
  was added.
- **Original visual design.** Design tokens, both themes, and every screen
  were built from scratch; no commercial layout, string or asset was copied.

## Verification evidence

Latest green run: **`39924358416`** on `f3e9be1`, four checks all successful.

| Check | Platform | Outcome |
|---|---|---|
| Verify (`cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`) | ubuntu-latest | success |
| Verify (`cargo test --workspace`) | ubuntu-latest | success — 203 tests |
| Verify (`cargo test --workspace`) | windows-latest | success — 203 tests |
| Verify (`cargo test --workspace`) | macos-latest | success — 203 tests |
| Package (`cargo build --release -p cadence-app`) | ubuntu-latest | success — `cadence-linux-x86_64` artefact uploaded |

What those 203 tests include:

- **Unit and integration tests** for the model, engine, calendar arithmetic,
  WBS tree, command layer (including every inverse), SQLite persistence and all
  four interchange formats — each format's reader tested against
  hand-written documents and each writer tested through a round trip.
- **Property tests** (proptest): calendar arithmetic round trips, arbitrary
  DAGs scheduling onto working days, ALAP preserving bounds and logic,
  constraint bounds respected or reported, leveling invariants (delays only,
  delay equals the calendar distance, conflict-free results reachable through
  ordinary constraints), and round-trip properties for all four interchange
  formats within their documented scope.
- **Data-integrity tests**: undo/redo restoring exact state, failed commands
  leaving no change, schema migrations, and files with newer versions being
  refused.
- **Accessibility tests**: WCAG AA (4.5:1) contrast for every colour the app
  draws text in, on both themes, plus white-on-accent selections.
- **Performance gates**: 10,000-activity schedule under 1 s, 10,000 committed
  resource days analysed under 2 s, 100 parallel activities leveled under 3 s.
  The budgets are set well above the slowest of the three CI runners — they
  exist to catch an algorithmic regression, not to measure hardware, which is
  what the first multi-OS run demonstrated: macOS ran the resource analysis in
  852 ms against a 750 ms budget while Linux ran the same code far quicker.
- **Report tests**: exact table rows, resource usage, earned-value sections,
  and an unschedulable project reported honestly instead of silently.

Every slice pushed in this project was verified the same way; the run IDs
above are recorded per slice in `PROGRESS.md`.

## Guide quality gates (`guide.md` section 9)

| Gate | Status | Evidence |
|---|---|---|
| Every in-scope capability has a status | met | `docs/parity.md`, `REQUIREMENTS_TRACEABILITY.md` |
| Main workflows work end-to-end | met for CLI/report/interchange | table, Gantt, resources, baselines, EVM, reports, import/export; interactive GUI behaviour is checked by the Phase-2/3/5 QC checklists, not automated |
| Persistence and data integrity tested | met | persistence and command test suites |
| No fake functionality | met | every visible control performs a real command; partial features are documented rather than stubbed |
| Full test suite and clean build via CI | met | 203 tests × 3 platforms, release packaging |
| No secrets committed | met | tracked files scanned for private keys, cloud and service credentials; no matches. Credentials live outside the repository and reach CI through GitHub secrets |
| Dependencies and licences listed | met | `ATTRIBUTION.md` (direct and dev-only), `Cargo.lock` for transitive |
| README accurate | met | points at `docs/usage.md`, parity and architecture |
| Known limitations documented | met | see below and `docs/qc/phase-5.md` |
| Clean-room discipline | met | `docs/inspiration.md` records only publicly documented behaviour; design, model and copy are original; `guide.md` itself is not tracked by git |

## Known limitations and deliberate deferrals

These are decided, recorded in `DECISIONS.md`, and not defects:

- **Progress-driven rescheduling** — recorded actual dates are facts and never
  feed the scheduling inputs, so variance stays meaningful (D15).
- **Per-assignment resource units and cost rates** — each assignment consumes
  exactly one unit per working day (D11).
- **Screen-reader audit and full keyboard-only operation** (P5-3) — requires a
  human with assistive technology; enabling AccessKit would add a dependency
  tree and is a decision for the maintainer. The **Print report…** save
  dialog is likewise interactive-only.
- **Sub-percent progress in MSPDI, fractional lag, non-8-hour days** —
  documented format-level quantisation (D14).
- **MPX and CSV carry no WBS or named calendars** — those dialects cannot
  express them; JSON can, and does (schema 2).
- **PDF printing scope** — the report prints as text pages; printing the
  Gantt view as an image page is supported by the writer (image pages) but
  not yet wired to a UI action.

## Next steps (human decisions)

1. Review the pull request and merge `opencode/ci` into `main`.
2. Decide whether to cut the first tagged release; the packaging job already
   produces a Linux binary, and the same workflow pattern extends to Windows
   and macOS binaries.
3. If a premium GUI is wanted: a print dialog (D16 notwithstanding), a
   screen-reader pass, and the full keyboard-only audit.
4. Repeat the Chat → Task → Pull Request cycle for anything else in scope.
