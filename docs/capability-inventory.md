# Capability Inventory

Designer-role record of every capability Cadence claims (guide II.27.B). One
row per capability: ID, Product, Capability, Observable behaviour, Acceptance
criteria, Source, Status.

**Status values:** `not started` / `partial` / `implemented` / `out of scope`.
**Product is Cadence for every row.** Every `implemented` row names its
evidence — a test, a CLI surface or a GUI view verified on this branch
(`main` @ `7755c92`, 2026-11-18). Where the only evidence is a headless test
and no user-reachable surface exists yet, the status is `partial` with the
note. Parity against the inspiration products lives in
[`docs/parity.md`](parity.md).

Last refreshed: **2026-11-18** (first edition — this file did not exist
before; capability claims previously lived only in `docs/parity.md` and
`REQUIREMENTS_TRACEABILITY.md`).

## A. Scheduling engine

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-001 | Cadence | Project model (activities, relationships, project settings) | Activities with name, duration, WBS parent, calendar, constraint, progress fields; relationships between them | `cadence-core/src/model.rs` unit tests; round-trip through persistence and JSON | PMBOK/CPM public standard; Cadence original | implemented |
| CAP-002 | Cadence | CPM scheduling: forward/backward pass, total/free float, critical set | Scheduling a project yields early/late dates, float and an ordered critical set | `engine.rs` unit tests; `crates/core/tests/properties.rs` property tests; CLI `cadence` table shows dates | PMBOK/CPM public standard | implemented |
| CAP-003 | Cadence | Relationship types FS/SS/FF/SF with signed lag/lead | Any link type with positive or negative lag schedules correctly | link-type unit tests in `engine.rs`; GUI link editing; property tests | PMBOK public standard | implemented |
| CAP-004 | Cadence | Cycle detection with diagnostics | A circular logic error is reported, not a hang or a wrong schedule | `SchedError::Cycle` unit tests (Kahn + DFS path) | Cadence original (standard graph diagnostics) | implemented |
| CAP-005 | Cadence | Working-day calendars (weekly pattern + holiday exceptions) | Durations and dates respect working days and holidays | `Calendar` unit tests; property tests for calendar invariants | PMBOK public standard | implemented |
| CAP-006 | Cadence | Multiple named calendars, per-activity assignment | Different activities on different calendars schedule without cross-talk | `calendar_set.rs` tests; mixed-calendar engine tests | P6 concept (multiple calendars), public docs; `docs/research/primavera-p6-capabilities.md` | implemented |
| CAP-007 | Cadence | Date constraints (SNET/SNLT/FNET/FNLT; ASAP/ALAP) | Constrained activities respect their bounds; scheduled vs early/late dates distinguishable | constraint-aware engine tests; `SetActivityConstraint` undo test; persistence migration test | PMBOK public standard | implemented |
| CAP-008 | Cadence | Progress-driven rescheduling from a data date | Residual work is scheduled from the status date: complete work pinned to actuals, in-progress resumes, not-started floored at the data date; mandatory pins override; D10/D29 rounding agreement | 16 engine unit tests + 2 property tests (`progress_mode_honours_mandatory_constraints`, `..._rounding_agrees_between_passes`); CLI `cadence reschedule`; GUI **Production** view preview | Aphex concept (production mode), `docs/research/aphex-integration.md`; D24 | implemented |

## B. Model editing & WBS

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-009 | Cadence | Typed command layer: single mutation path, undo/redo | Every edit goes through an undoable command; undo/redo from any view (Ctrl+Z/Y) | command tests per family in `cadence-commands`; GUI undo/redo buttons | Cadence original | implemented |
| CAP-010 | Cadence | WBS hierarchy | Activities nest in a WBS tree; reparenting is cycle-safe and undoable; delete guards | `wbs.rs` unit + command tests | P6 concept (WBS), public docs; `docs/research/primavera-p6-capabilities.md` | implemented |

## C. Desktop GUI (nine views)

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-011 | Cadence | Activity table | Sortable columns, selection, virtualised rows with row numbers and sticky header, column chooser, inspector edits | Phase-2 QC gate `docs/qc/phase-2.md`; cached sort on `edit_version` | MS Project combined table+Gantt concept, `docs/research/ms-project-capabilities.md` | implemented |
| CAP-012 | Cadence | Interactive Gantt | Timeline, bars with progress overlay, dependency arrows, Ctrl+wheel and keyboard zoom, panning | Phase-2 QC gate; O(relationships) arrow pass; zoom tests in UI layer | MS Project Gantt concept, `docs/research/ms-project-capabilities.md` | implemented |
| CAP-013 | Cadence | Dependency editing | Drag-to-create links, lag editing, cycle prevention before commit | Phase-2 QC gate; engine cycle tests behind the GUI guard | MS Project / P6 link editing concept | implemented |
| CAP-014 | Cadence | Keyboard operation and view switching | Ctrl+1..9 switch the nine views, Ctrl+S/O/W/N file actions, Ctrl+Z/Y undo-redo, Ctrl+D theme, Ctrl+Plus/Minus/0 Gantt zoom, F1 shortcut window; every view reviewed for pointer-only paths | keyboard map in `crates/ui-egui/src/lib.rs` (ViewMode + shortcuts); `docs/qc/shortcuts.md`; phase-2/3 QC gates | Cadence original (accessibility-first) | implemented |
| CAP-015 | Cadence | Resources view | Dictionary editing, assignments, per-assignment units, cost rate, usage, leveling preview/apply | `crates/ui-egui` Resources view (`Ctrl+3`); phase-3 QC gate | `docs/research/ms-project-capabilities.md` | implemented |
| CAP-016 | Cadence | Baselines view | Per-activity duration/start/finish variance against the selected baseline, coloured by sign; EVM panel against an editable status date | `crates/ui-egui` Baselines view (`Ctrl+4`); EVM unit tests behind it | MS Project baseline/variance concept | implemented |
| CAP-017 | Cadence | Calendars view | List, create, rename, delete calendars; working-day and holiday editing; per-activity assignment | `crates/ui-egui` Calendars view (`Ctrl+5`) | P6 calendar management concept | implemented |
| CAP-018 | Cadence | Audit view | Run the audit in-app; per-check results with pass/fail/NA and summaries; critical-path test and CPLI | `crates/ui-egui` Audit view (`Ctrl+6`); 39 audit unit tests behind it | `docs/research/schedule-audit.md` | implemented |
| CAP-019 | Cadence | Lookahead view | Rolling window of near work; owner/promise/delay-reason editing; published snapshots with variance comparison; clash list with links | `crates/ui-egui` Lookahead view (`Ctrl+7`); `lookahead.rs`/`clash.rs` unit tests | `docs/research/aphex-integration.md` | implemented |
| CAP-020 | Cadence | Diary view | Dated diary notes linked to an activity or lookahead window; create/update/delete | `crates/ui-egui` Diary view (`Ctrl+8`); diary command + persistence tests | `docs/research/aphex-integration.md` | implemented |
| CAP-021 | Cadence | Production view | Non-destructive preview of progress-driven rescheduling before applying | `crates/ui-egui` Production view (`Ctrl+9`); engine `schedule_with_progress` tests | `docs/research/aphex-integration.md`; D24 | implemented |

## D. Resources

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-022 | Cadence | Resource dictionary | Resources with kind, capacity (units/working day) and cost per unit; unique case-insensitive names | `model.rs` + command tests | `docs/research/ms-project-capabilities.md` | implemented |
| CAP-023 | Cadence | Assignments with per-assignment units | Assign/unassign undoably; sparse per-assignment units override in basis points | `SetAssignmentUnits` + GUI editing (D25); persistence v9→v10 migration test | `docs/research/ms-project-capabilities.md`; D25 | implemented |
| CAP-024 | Cadence | Usage histogram | Committed units per working day, units-aware | `engine::resource_histogram` tests | `docs/research/ms-project-capabilities.md` | implemented |
| CAP-025 | Cadence | Over-allocation detection | Every day where committed units exceed capacity is reported | `engine::resource_conflicts` tests (D25) | `docs/research/ms-project-capabilities.md` | implemented |
| CAP-026 | Cadence | Resource leveling | Greedy earliest-conflict delay; preview and apply from the Resources view | `engine::level_resources` tests (D12) | `docs/research/primavera-p6-capabilities.md` (leveling concept) | implemented |

## E. Baselines, progress & earned value

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-027 | Cadence | Multiple baselines | Capture/remove/restore project-wide baselines with per-activity snapshots, undoable | engine + command + persistence tests | MS Project multiple baselines concept | implemented |
| CAP-028 | Cadence | Variance against a selected baseline | Duration/start/finish variance per activity, coloured by sign | Baselines view variance table; snapshot variance tests | MS Project variance columns concept | implemented |
| CAP-029 | Cadence | EVM metrics at a status date | PV, EV, AC, SV, CV, SPI, CPI against an editable status date; also EAC, TCPI, VAC computed per PMI standard | `engine::evm_metrics` unit tests; `evm-eac-tcpi-vac` unit tests; Baselines view panel | PMBOK EVM formulas (public standard) | implemented |
| CAP-030 | Cadence | Progress updating | Percent complete, actual dates and actual cost edited via undoable commands; Gantt progress overlay | command range/date validation tests; persistence schema v4 actual dates | `docs/research/ms-project-capabilities.md` | implemented |

## F. Schedule audit & comparison

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-031 | Cadence | 18 schedule-quality checks | Open ends, leads, lags, relationship types, hard constraints, unnecessary constraints, long lags, redundant logic, high float, negative float, high duration, invalid dates, missing resources, missed tasks, out-of-sequence progress + the three index checks (CAP-032); pass/fail/not-applicable per check with plain-language summaries | 18 `CheckId` variants in `cadence-core/src/audit.rs`; 39 audit unit tests (e.g. `a_clean_project_passes_the_structure_checks`, `an_open_end_fails_the_logic_check`); CLI `cadence audit [--json] [--status-date]` | DCMA 14-point / GAO practices (public standards); `docs/research/schedule-audit.md` | implemented |
| CAP-032 | Cadence | Critical-path test, CPLI, BEI | Delay-the-critical-path test, critical path length index, baseline execution index | `CheckId::CriticalPathTest`/`CriticalPathLengthIndex`/`BaselineExecutionIndex` tests in `audit.rs` | DCMA 14-point (public standard); `docs/research/schedule-audit.md` | implemented |
| CAP-033 | Cadence | Forensic + baseline comparison | Compare two programmes or a programme against a baseline: added/removed activities, duration, relationship, repoint and constraint changes; deterministic output | 10 comparison unit tests in `cadence-core/src/compare.rs` (e.g. `a_repointed_relationship_is_removed_and_added`, `the_comparison_is_deterministic`); CLI `cadence audit-compare` | `docs/research/schedule-audit.md` (Deltek Acumen Fuse-style forensic concept) | implemented |
| CAP-034 | Cadence | Audit/comparison JSON export | `kind: "audit"` and `kind: "comparison"` machine-readable documents | `cadence-interchange/src/audit.rs` + `compare.rs` document tests | Cadence original (guide §21.5 tooling accessibility) | implemented |

## G. Lookahead & field planning (Aphex-inspired)

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-035 | Cadence | Rolling lookahead window extraction | Near work within the window, in-progress marking, deterministic ordering, empty past-the-end | 5 unit tests in `cadence-core/src/lookahead.rs` (e.g. `a_two_week_window_contains_only_the_near_work`) | `docs/research/aphex-integration.md` | implemented |
| CAP-036 | Cadence | Owner/promise/delay-reason tracking | Per-activity owner and promise date; delay reasons validated | `SetActivityOwner`/`SetActivityPromise` command tests; persistence schema v5 | `docs/research/aphex-integration.md` | implemented |
| CAP-037 | Cadence | Published lookahead snapshots | Named read-only snapshots; start/finish variance comparison against a selected snapshot; capture/remove/restore undoable | snapshot engine + command tests; persistence schema v6 | `docs/research/aphex-integration.md` | implemented |
| CAP-038 | Cadence | Daily diary / huddle briefing | Dated notes linked to an activity or window; undoable create/update/delete | diary command + persistence round-trip tests (schema v7) | `docs/research/aphex-integration.md` | implemented |
| CAP-039 | Cadence | Location clash detection | Two open activities sharing a free-text location on overlapping days are flagged; listed with links | `cadence-core::clash::location_clashes` unit tests (schema v8) | `docs/research/aphex-integration.md` | implemented |
| CAP-040 | Cadence | Production tracking | Production entries per activity; progress mode preview in the Production view | production command + persistence tests; `schedule_with_progress` evidence (CAP-008) | `docs/research/aphex-integration.md` | implemented |

## H. Interchange

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-041 | Cadence | MSPDI (Microsoft Project XML) import/export | Activities, links, constraints, assignments, progress move through MSPDI | round-trip + hand-written-document tests in `cadence-interchange/src/mspdi.rs` | public MSPDI XML reference; `docs/research/ms-project-capabilities.md` | implemented |
| CAP-042 | Cadence | MPX import/export | The legacy MPX text format round-trips | round-trip + hand-written-document tests in `mpx.rs` | public MPX format documentation | implemented |
| CAP-043 | Cadence | CSV import/export | One row per activity with constraints, links, assignments and progress | `csv.rs` round-trip tests; CSV-vs-MSPDI equivalence tests; CLI `export-csv`/`import-csv` | Cadence original format definition | implemented |
| CAP-044 | Cadence | JSON interchange (lossless) | Cadence's own versioned schema; the only lossless format — carries WBS, named calendars, membership | `json.rs` round-trip + JSON-vs-MSPDI equivalence tests; CLI `export-json`/`import-json` | Cadence original schema | implemented |
| CAP-045 | Cadence | Machine-readable schedule output (`--json`) | `cadence [--json]` and `cadence load <file> --json` emit the schedule document for scripts and assistive technology | `schedule.rs` document-parsing tests; documented in `docs/usage.md` | guide §21.5 tooling accessibility | implemented |
| CAP-046 | Cadence | Lossless WBS + named calendars in interchange | MSPDI imports WBS codes and named calendars (weekly pattern + holidays); JSON carries them; MPX and CSV cannot express them and import them empty — **documented limitation**, not a bug | `mspdi.rs` tests: `wbs_code_is_imported_from_mspdi`, `named_calendars_are_imported_from_mspdi`; JSON round-trip tests; `interchange::json` docs | format-expressiveness analysis (designer record) | implemented |

## I. Persistence

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-047 | Cadence | SQLite project files: versioned schema, migrations, transactional save/load | `PRAGMA user_version` schema (current **v10**); older files migrate; newer files refused with a clear message; save/load is transactional | persistence unit tests: full round-trip, overwrite, missing file, newer schema, dangling references, invalid data, v9→v10 migration | Cadence original | implemented |

## J. Reports & printing

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-048 | Cadence | Markdown report | `cadence report` and GUI **Save report** produce a Markdown report incl. resource cost roll-up | `report.rs` unit tests | Cadence original | implemented |
| CAP-049 | Cadence | PDF printing | `cadence print` and GUI **Print report…** via a native save dialog | `pdf.rs` writer tests; dialog hands-on per QC | Cadence original; `docs/research/printing-the-gantt-view.md` | implemented |

## K. Platform services

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-050 | Cadence | Crash reporting with consent | Local panic log, GUI preview, opt-in pre-filled GitHub issue; no token embedded | crash module unit tests (title, body, URL encoding); dialog interactive-only | guide §7.1; `docs/research/crash-report-submission.md` | implemented |
| CAP-051 | Cadence | Opt-in update check | GitHub Releases check with six-hour cache, fail-soft offline; `cadence check-updates` and GUI **Updates** | version-comparison, malformed-tag and cache-serialisation tests | guide §7.2; `docs/research/update-check.md` | implemented |
| CAP-052 | Cadence | Accessibility (code-level scope) | AccessKit via eframe defaults; standard widgets expose labels/roles; Gantt canvas carries a `WidgetInfo` summary; WCAG AA contrast tested; keyboard completes flows; toolkit limits recorded | `accessibility_tests` contrast suite; `ACCESSIBILITY.md` records per-bar Gantt nodes, live-region status and manual screen-reader passes as remaining limits | guide II.21/II.30; `docs/research/screen-reader-support.md` | implemented |

## L. Project health

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-053 | Cadence | Community docs + contribution surface | README (plain language, no-Rust-needed try steps), community files, issue forms, PR template | files present; reviewed in `docs/qc/project-health.md` | opensource.guide (public) | implemented |
| CAP-054 | Cadence | Security gates in CI | Secret scanning, personal-data scan, dependency licence/advisory checks, dependency review, SAST (CodeQL), SBOM | `.github/workflows/security.yml` runs on `main` and PRs; `deny.toml` | opensource.guide; guide II.20 | implemented |
| CAP-055 | Cadence | Multi-OS CI + tagged releases with artefacts | Verify on Linux/Windows/macOS, packaged release binaries, semver tags | release.yml on tags; releases `v0.3.0`…`v0.14.0` exist | guide §4/§6 | implemented |

## M. AI assistant (owner-approved research; backend slices landed 2026-11-16/17/18)

The assistant's standing rule on every row: **the assistant proposes, the
planner disposes** — a suggestion never mutates the plan except as an
ordinary, undoable command accepted through a review surface. No assistant
capability is user-reachable yet: there is no UI, no settings surface and no
trigger; the crate is headless backend only. All rows below are therefore
`partial` (backend landed, surface missing) or `not started`.

| ID | Product | Capability | Observable behaviour | Acceptance criteria | Source | Status |
|---|---|---|---|---|---|---|
| CAP-056 | Cadence | Assistant provider configuration | `AssistantConfig` (provider name, required model, optional `base_url`, self-redacting `SecretKey`); `is_configured()` gate; `HttpProvider::from_config` builds a provider for OpenAI chat-completions (incl. OpenAI-compatible servers via `base_url`), Anthropic Messages, local Ollama (no key) and Gemini. UI surface landed via RP-3: enabling form visible in Assistant view tenth tab; API key session-only, never persisted. | 24+ headless `http.rs` wire tests + 4 config tests; no test opens a socket; D30. Missing: none (surface landed RP-3). | `docs/research/ai-integration.md`; D30; RP-3 | landed |
| CAP-057 | Cadence | Assistant opt-in and secret hygiene | Assistant off by default; `is_assistant_enabled` gates everything; keys redact in Debug and serde (`"<redacted>"`); credential headers omitted entirely when keyless; errors never echo request material | `lib.rs`/`config.rs`/`http.rs` redaction and boundary tests (e.g. refusal-boundary test, keyless-header test). Missing: none (surface landed RP-3). | `docs/research/ai-integration.md`; guide §7.1/II.19.4; RP-3 | landed |
| CAP-058 | Cadence | Proposal generation (typed, validated) | `ProposedChange` (create activity/relationship, set duration/progress) each with a rationale; `Proposal::validate`/`validate_against_project` return **all** problems at once against the live project (unknown activities, progress >100, self/duplicate links, blank names, zero durations). UI surface landed via RP-4: request flow with text input, submit button, background provider call; validation errors surfaced in status bar. | 13 headless validation/config tests (PR #155 slice). Missing: none (surface landed RP-4). | `docs/research/ai-integration.md`; RP-4 | landed |
| CAP-059 | Cadence | Proposal review/apply ("proposes, planner disposes") | `map_proposal_to_commands` turns an already-validated proposal into ordinary undoable commands (create-activity, FS link without lag, set-duration, set-progress in basis points); re-validates first — an invalid proposal maps to an error, never a partial command list; undo walks back change by change. UI surface landed via RP-4/RP-5: review panel shows proposal with rationales, selection checkboxes, Apply/Discard buttons; AccessKit harness asserts named inputs. | 8 `command_map.rs` workflow tests through the real `CommandStack`. Missing: none (surface landed RP-4/RP-5). | `docs/research/ai-integration.md`; "planner disposes" rule; RP-4/RP-5 | landed |
| CAP-060 | Cadence | Document understanding (PDF/email reading) | Read user-supplied PDFs/emails/documents and extract schedule-relevant facts | — | `docs/research/ai-integration.md` (approved 2026-10-27) | not started |
| CAP-061 | Cadence | Broader assistant proposals (advice, enrichment, status updates) | Programme advice, activity enrichment and draft status updates beyond the four proposed change kinds | — | `docs/research/ai-integration.md` (approved 2026-10-27) | not started |

## Counts

61 rows: **55 implemented, 4 partial, 2 not started, 0 out of scope.**
The partial rows are honest: CAP-046 is a documented format limitation; CAP-056…059 are backend-only assistant slices awaiting their user surfaces.
