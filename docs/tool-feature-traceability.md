# Tool–Feature–Function Traceability Matrix

**Purpose:** Map every tool, feature, and function in Cadence to its inspiration source and research note. This document implements the Designer's standing responsibility (Guide II.27.B) and the clean-room protocol (Guide §2.1–2.3).

**Source documents:**
- `docs/inspiration.md` — inspiration products and public concepts
- `docs/scope.md` — in-scope / out-of-scope capabilities
- `docs/capability-inventory.md` — 61 capability rows with acceptance criteria
- `docs/research/*.md` — 11 research notes with original behavioural summaries + public URLs
- `docs/specs/*.spec.md` — feature specifications
- `docs/tickets/*.md` — atomic implementation tickets
- `ROADMAP.md` — release milestones and progress

---

## Legend

| Column | Meaning |
|--------|---------|
| **Capability ID** | From `docs/capability-inventory.md` |
| **Tool/Feature/Function** | User-visible capability or internal function |
| **Inspiration Product** | Product whose public behaviour inspired this capability |
| **Research Note** | File in `docs/research/` with behavioural summary + URLs |
| **Spec** | Specification file in `docs/specs/` (if applicable) |
| **Tickets** | Implementation tickets in `docs/tickets/` |
| **Status** | `implemented` / `partial` / `not started` / `out of scope` |
| **Evidence** | Test file, CLI surface, or GUI view name |

---

## A. Scheduling Engine (CAP-001 to CAP-008)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-001 | Project model (activities, relationships, settings) | PMBOK/CPM public standard | — | — | — | implemented | `cadence-core/src/model.rs` unit tests; round-trip persistence + JSON |
| CAP-002 | CPM scheduling: forward/backward pass, total/free float, critical set | PMBOK/CPM public standard | — | — | — | implemented | `engine.rs` unit tests; `crates/core/tests/properties.rs`; CLI `cadence` table |
| CAP-003 | Relationship types FS/SS/FF/SF with signed lag/lead | PMBOK public standard | — | — | — | implemented | link-type unit tests in `engine.rs`; GUI link editing; property tests |
| CAP-004 | Cycle detection with diagnostics | Cadence original (standard graph diagnostics) | — | — | — | implemented | `SchedError::Cycle` unit tests (Kahn + DFS path) |
| CAP-005 | Working-day calendars (weekly pattern + holiday exceptions) | PMBOK public standard | — | — | — | implemented | `Calendar` unit tests; property tests for calendar invariants |
| CAP-006 | Multiple named calendars, per-activity assignment | Primavera P6 (multiple calendars) | `docs/research/primavera-p6-capabilities.md` | — | — | implemented | `calendar_set.rs` tests; mixed-calendar engine tests |
| CAP-007 | Date constraints (SNET/SNLT/FNET/FNLT; ASAP/ALAP) | PMBOK public standard | — | — | — | implemented | constraint-aware engine tests; `SetActivityConstraint` undo test; persistence migration test |
| CAP-008 | Progress-driven rescheduling from a data date | Aphex (production mode) | `docs/research/aphex-integration.md` | — | — | implemented | 16 engine unit tests + 2 property tests; CLI `cadence reschedule`; GUI Production view preview |

**Key research links:**
- `docs/research/primavera-p6-capabilities.md` — P6 concepts: multiple calendars, WBS hierarchy, activity codes, resource leveling at scale
- `docs/research/aphex-integration.md` — Aphex short-interval planning: rolling windows, weekly work plans, owner/promise tracking, daily diary, production tracking, location clash detection, master→lookahead bridge

---

## B. Model Editing & WBS (CAP-009 to CAP-010)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-009 | Typed command layer: single mutation path, undo/redo | Cadence original | — | — | — | implemented | command tests per family in `cadence-commands`; GUI undo/redo buttons |
| CAP-010 | WBS hierarchy | Primavera P6 (WBS) | `docs/research/primavera-p6-capabilities.md` | — | — | implemented | `wbs.rs` unit + command tests |

---

## C. Desktop GUI — Ten Views (CAP-011 to CAP-021)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-011 | Activity table (sortable, virtualised, column chooser, inspector) | MS Project (combined table+Gantt) | `docs/research/ms-project-capabilities.md` | — | — | implemented | Phase-2 QC gate `docs/qc/phase-2.md`; cached sort on `edit_version` |
| CAP-012 | Interactive Gantt (timeline, bars, arrows, zoom, pan) | MS Project (Gantt) | `docs/research/ms-project-capabilities.md` | — | — | implemented | Phase-2 QC gate; O(relationships) arrow pass; zoom tests in UI layer |
| CAP-013 | Dependency editing (drag-to-create, lag editing, cycle prevention) | MS Project / P6 link editing | `docs/research/ms-project-capabilities.md` | — | — | implemented | Phase-2 QC gate; engine cycle tests behind GUI guard |
| CAP-014 | Keyboard operation & view switching (Ctrl+1..9, shortcuts) | Cadence original (accessibility-first) | — | — | — | implemented | keyboard map in `crates/ui-egui/src/lib.rs`; `docs/qc/shortcuts.md`; phase-2/3 QC gates |
| CAP-015 | Resources view (dictionary, assignments, usage, leveling preview/apply) | MS Project | `docs/research/ms-project-capabilities.md` | — | — | implemented | `crates/ui-egui` Resources view (`Ctrl+3`); phase-3 QC gate |
| CAP-016 | Baselines view (per-activity variance, EVM panel) | MS Project baseline/variance | `docs/research/ms-project-capabilities.md` | — | — | implemented | `crates/ui-egui` Baselines view (`Ctrl+4`); EVM unit tests |
| CAP-017 | Calendars view (list, create, edit, delete, per-activity assignment) | P6 calendar management | `docs/research/primavera-p6-capabilities.md` | — | — | implemented | `crates/ui-egui` Calendars view (`Ctrl+5`) |
| CAP-018 | Audit view (run audit in-app, per-check results, CPLI, critical-path test) | Schedule audit tools (DCMA/GAO) | `docs/research/schedule-audit.md` | — | — | implemented | `crates/ui-egui` Audit view (`Ctrl+6`); 39 audit unit tests |
| CAP-019 | Lookahead view (rolling window, owner/promise, snapshots, clash list) | Aphex (lookahead/field planning) | `docs/research/aphex-integration.md` | — | — | implemented | `crates/ui-egui` Lookahead view (`Ctrl+7`); `lookahead.rs`/`clash.rs` unit tests |
| CAP-020 | Diary view (dated notes linked to activity/window) | Aphex (daily diary/huddle) | `docs/research/aphex-integration.md` | — | — | implemented | `crates/ui-egui` Diary view (`Ctrl+8`); diary command + persistence tests |
| CAP-021 | Production view (non-destructive preview of progress-driven rescheduling) | Aphex (production mode) | `docs/research/aphex-integration.md` | — | — | implemented | `crates/ui-egui` Production view (`Ctrl+9`); engine `schedule_with_progress` tests |

**Key research links:**
- `docs/research/ms-project-capabilities.md` — MS Project concepts: combined table+Gantt, baseline/variance workflow, earned-value reporting basics
- `docs/research/aphex-integration.md` — Aphex concepts: short-interval/lookahead planning, rolling windows, weekly work plans, owner/promise tracking, daily diary, production tracking, location clash detection, master→lookahead bridge

---

## View → Shortcut → Render Method Mapping (from VIEW_META in lib.rs)

| ViewMode | Tab Label | Icon | Shortcut | ViewGroup | Render Method |
|---|---|---|---|---|---|
| Table | Table | TABLE | Ctrl+1 | Core | `render_table` / `render_split_gantt` |
| Gantt | Gantt | CHART_BAR | Ctrl+2 | Core | `render_gantt` |
| Resources | Resources | USERS | Ctrl+3 | Core | `render_resources` |
| Baselines | Baselines | STACK | Ctrl+4 | PlanTime | `render_baselines` |
| Calendars | Calendars | CALENDAR | Ctrl+5 | PlanTime | `render_calendars` |
| Audit | Audit | SHIELD_CHECK | Ctrl+6 | Quality | `render_audit` |
| Lookahead | Lookahead | BINOCULARS | Ctrl+7 | ProgressSite | `render_lookahead` |
| Diary | Diary | NOTE_PENCIL | Ctrl+8 | ProgressSite | `render_diary` |
| Production | Production | GAUGE | Ctrl+9 | ProgressSite | `render_production` |
| Assistant | Assistant | CHAT_CIRCLE_TEXT | *(none)* | Quality | `render_assistant` |

---

## D. Resources (CAP-022 to CAP-026)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-022 | Resource dictionary (kind, capacity, cost) | MS Project | `docs/research/ms-project-capabilities.md` | — | — | implemented | `model.rs` + command tests |
| CAP-023 | Assignments with per-assignment units (basis points override) | MS Project | `docs/research/ms-project-capabilities.md`; D25 | — | — | implemented | `SetAssignmentUnits` + GUI editing (D25); persistence v9→v10 migration test |
| CAP-024 | Usage histogram (committed units per working day) | MS Project | `docs/research/ms-project-capabilities.md` | — | — | implemented | `engine::resource_histogram` tests |
| CAP-025 | Over-allocation detection | MS Project | `docs/research/ms-project-capabilities.md` | — | — | implemented | `engine::resource_conflicts` tests (D25) |
| CAP-026 | Resource leveling (greedy earliest-conflict delay) | Primavera P6 (leveling concept) | `docs/research/primavera-p6-capabilities.md` | — | — | implemented | `engine::level_resources` tests (D12) |

---

## E. Baselines, Progress & Earned Value (CAP-027 to CAP-030)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-027 | Multiple baselines (capture/remove/restore, undoable) | MS Project multiple baselines | `docs/research/ms-project-capabilities.md` | — | — | implemented | engine + command + persistence tests |
| CAP-028 | Variance against selected baseline (duration/start/finish, coloured) | MS Project variance columns | `docs/research/ms-project-capabilities.md` | — | — | implemented | Baselines view variance table; snapshot variance tests |
| CAP-029 | EVM metrics at status date (PV, EV, AC, SV, CV, SPI, CPI, EAC, TCPI, VAC) | PMBOK EVM formulas (public standard) | — | — | — | implemented | `engine::evm_metrics` unit tests; `evm-eac-tcpi-vac` unit tests; Baselines view panel |
| CAP-030 | Progress updating (percent complete, actual dates, actual cost, Gantt overlay) | MS Project | `docs/research/ms-project-capabilities.md` | — | — | implemented | command range/date validation tests; persistence schema v4 actual dates |

---

## F. Schedule Audit & Comparison (CAP-031 to CAP-034)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-031 | 18 schedule-quality checks (open ends, leads, lags, hard constraints, etc.) | DCMA 14-point / GAO practices (public standards) | `docs/research/schedule-audit.md` | — | — | implemented | 18 `CheckId` variants in `cadence-core/src/audit.rs`; 39 audit unit tests; CLI `cadence audit [--json] [--status-date]` |
| CAP-032 | Critical-path test, CPLI, BEI | DCMA 14-point (public standard) | `docs/research/schedule-audit.md` | — | — | implemented | `CheckId::CriticalPathTest`/`CriticalPathLengthIndex`/`BaselineExecutionIndex` tests in `audit.rs` |
| CAP-033 | Forensic + baseline comparison (added/removed activities, duration, relationship, repoint, constraint changes) | Deltek Acumen Fuse-style forensic concept | `docs/research/schedule-audit.md` | — | — | implemented | 10 comparison unit tests in `cadence-core/src/compare.rs`; CLI `cadence audit-compare` |
| CAP-034 | Audit/comparison JSON export (`kind: "audit"` and `kind: "comparison"`) | Cadence original (Guide §21.5 tooling accessibility) | — | — | — | implemented | `cadence-interchange/src/audit.rs` + `compare.rs` document tests |

**18 CheckId values (from audit.rs):**
| CheckId | Code | Description |
|---|---|---|
| LogicOpenEnds | AUDIT-LOGIC | Open activities missing a predecessor or successor |
| Leads | AUDIT-LEADS | Relationships with a negative lag |
| Lags | AUDIT-LAGS | Relationships with a positive lag |
| RelationshipTypes | AUDIT-RELTYPES | Share of finish-to-start relationships |
| HardConstraints | AUDIT-HARDCONST | Open activities held by a hard date constraint |
| HighFloat | AUDIT-HIGHFLOAT | Open activities with more than the high-float threshold |
| NegativeFloat | AUDIT-NEGFLOAT | Activities with negative total float |
| HighDuration | AUDIT-HIGHDUR | Activities longer than the high-duration threshold |
| InvalidDates | AUDIT-INVDATES | Actual dates after, or forecast dates before, the status date |
| MissingResources | AUDIT-MISSRES | Open activities with a duration but no resource |
| MissedTasks | AUDIT-MISSED | Activities not finished by their baseline finish |
| BaselineExecutionIndex | AUDIT-BEI | Baseline execution index |
| CriticalPathTest | AUDIT-CPT | Delay one critical activity and check the finish moves with it |
| CriticalPathLengthIndex | AUDIT-CPLI | Critical path length index against the baseline deadline |
| OutOfSequenceProgress | AUDIT-OOS | Progress posted on an activity whose FS predecessor is not finished |
| RedundantLogic | AUDIT-REDUNDANT | A successor reached by more than one path through the logic |
| UnnecessaryConstraints | AUDIT-UNNECCONST | A hard date constraint on an activity not on the critical path |
| LongLags | AUDIT-LONGLAGS | A relationship with a lag longer than the long-lag threshold |

**Key research links:**
- `docs/research/schedule-audit.md` — Schedule-quality and health auditing of construction programmes: DCMA 14-point, GAO ten best practices, AACE RPs; forensic comparison concept from Deltek Acumen Fuse, Steelray Analyzer, etc.

---

## G. Lookahead & Field Planning — Aphex-Inspired (CAP-035 to CAP-040)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-035 | Rolling lookahead window extraction | Aphex (short-interval planning) | `docs/research/aphex-integration.md` | — | — | implemented | 5 unit tests in `cadence-core/src/lookahead.rs` |
| CAP-036 | Owner/promise/delay-reason tracking | Aphex (weekly work plans) | `docs/research/aphex-integration.md` | — | — | implemented | `SetActivityOwner`/`SetActivityPromise` command tests; persistence schema v5 |
| CAP-037 | Published lookahead snapshots (named, read-only, variance comparison) | Aphex (published snapshots) | `docs/research/aphex-integration.md` | — | — | implemented | snapshot engine + command tests; persistence schema v6 |
| CAP-038 | Daily diary / huddle briefing | Aphex (daily diary) | `docs/research/aphex-integration.md` | — | — | implemented | diary command + persistence round-trip tests (schema v7) |
| CAP-039 | Location clash detection | Aphex (location clash detection) | `docs/research/aphex-integration.md` | — | — | implemented | `cadence-core::clash::location_clashes` unit tests (schema v8) |
| CAP-040 | Production tracking (entries per activity, progress mode preview) | Aphex (production tracking) | `docs/research/aphex-integration.md` | — | — | implemented | production command + persistence tests; `schedule_with_progress` evidence |

**Key research links:**
- `docs/research/aphex-integration.md` — Aphex short-interval/lookahead planning: rolling windows, weekly work plans, owner/promise tracking, daily diary and briefings, production tracking, location clash detection, master→lookahead bridge

---

## H. Interchange (CAP-041 to CAP-046)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-041 | MSPDI (MS Project XML) import/export | Public MSPDI XML reference | `docs/research/ms-project-capabilities.md` | — | — | implemented | round-trip + hand-written-document tests in `cadence-interchange/src/mspdi.rs` |
| CAP-042 | MPX import/export (legacy MPX text format) | Public MPX format documentation | — | — | — | implemented | round-trip + hand-written-document tests in `mpx.rs` |
| CAP-043 | CSV import/export | Cadence original format definition | — | — | — | implemented | `csv.rs` round-trip tests; CSV-vs-MSPDI equivalence tests; CLI `export-csv`/`import-csv` |
| CAP-044 | JSON interchange (lossless, versioned schema) | Cadence original schema | — | — | — | implemented | `json.rs` round-trip + JSON-vs-MSPDI equivalence tests; CLI `export-json`/`import-json` |
| CAP-045 | Machine-readable schedule output (`--json`) | Guide §21.5 tooling accessibility | — | — | — | implemented | `schedule.rs` document-parsing tests; documented in `docs/usage.md` |
| CAP-046 | Lossless WBS + named calendars in interchange (MSPDI imports them; JSON carries them; MPX/CSV cannot) | Format-expressiveness analysis (designer record) | — | — | — | implemented | `mspdi.rs` tests: `wbs_code_is_imported_from_mspdi`, `named_calendars_are_imported_from_mspdi`; JSON round-trip tests |

**Interchange format handlers (crates/interchange/src/):**
| Format | Export Function | Import Function | CLI Commands |
|--------|----------------|----------------|--------------|
| Microsoft Project XML (MSPDI) | `mspdi.rs::export()` | `mspdi.rs::import()` | `export-xml`, `import-xml` |
| MPX | `mpx.rs::export()` | `mpx.rs::import()` | `export-mpx`, `import-mpx` |
| CSV | `csv.rs::export()` | `csv.rs::import()` | `export-csv`, `import-csv` |
| Cadence JSON | `json.rs::export()` | `json.rs::import()` | `export-json`, `import-json` |
| Schedule JSON | `schedule.rs::schedule_export()` | — | `cadence --json` |
| Audit JSON | `audit.rs::audit_export()` | — | `cadence audit --json` |
| Comparison JSON | `compare.rs::comparison_export()` | — | `cadence audit-compare --json` |

---

## I. Persistence (CAP-047)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-047 | SQLite project files: versioned schema (v10), migrations, transactional save/load | Cadence original | — | — | — | implemented | persistence unit tests: full round-trip, overwrite, missing file, newer schema, dangling references, invalid data, v9→v10 migration |

---

## J. Reports & Printing (CAP-048 to CAP-049)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-048 | Markdown report (resource cost roll-up) | Cadence original | — | — | — | implemented | `report.rs` unit tests |
| CAP-049 | PDF printing (native save dialog) | Cadence original | `docs/research/printing-the-gantt-view.md` | — | — | implemented | `pdf.rs` writer tests; dialog hands-on per QC |

---

## K. Platform Services (CAP-050 to CAP-052)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-050 | Crash reporting with consent (local panic log, GUI preview, opt-in GitHub issue) | Guide §7.1; crash-report pattern | `docs/research/crash-report-submission.md` | — | — | implemented | crash module unit tests (title, body, URL encoding); dialog interactive-only |
| CAP-051 | Opt-in update check (GitHub Releases, six-hour cache, fail-soft offline) | Guide §7.2; update-check pattern | `docs/research/update-check.md` | — | — | implemented | version-comparison, malformed-tag and cache-serialisation tests |
| CAP-052 | Accessibility (AccessKit via eframe; WCAG AA contrast; keyboard completes flows) | Guide II.21/II.30; accessibility standards | `docs/research/screen-reader-support.md` | `cadence-accessibility.spec.md` | — | implemented | `accessibility_tests` contrast suite; `ACCESSIBILITY.md` records limits |

---

## L. Project Health (CAP-053 to CAP-055)

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-053 | Community docs + contribution surface (README, community files, issue forms, PR template) | opensource.guide (public) | — | — | — | implemented | files present; reviewed in `docs/qc/project-health.md` |
| CAP-054 | Security gates in CI (secret scanning, personal-data scan, dependency licence/advisory, SAST, SBOM) | opensource.guide; Guide II.20 | — | — | — | implemented | `.github/workflows/security.yml` runs on `main` and PRs; `deny.toml` |
| CAP-055 | Multi-OS CI + tagged releases with artefacts | Guide §4/§6 | — | — | — | implemented | release.yml on tags; releases `v0.3.0`…`v0.14.0` exist |

---

## M. AI Assistant (CAP-056 to CAP-061) — Owner-Approved Research; Backend Slices Landed

| Capability ID | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| CAP-056 | Assistant provider configuration (`AssistantConfig`, `HttpProvider::from_config`, `SecretKey`) | nPlan, Copilot for Project, Planera, ALICE, Trunk Tools, Document Crunch (AI assistance) | `docs/research/ai-integration.md` | `review-panel.spec.md` | RP-1, RP-2 | partial (backend only) | 24+ headless `http.rs` wire tests + 4 config tests; no test opens socket; D30. Missing: settings UI |
| CAP-057 | Assistant opt-in and secret hygiene (off by default, `SecretKey` redacts in Debug/serde) | `docs/research/ai-integration.md`; Guide §7.1/II.19.4 | `docs/research/ai-integration.md` | `review-panel.spec.md` | RP-2 | partial (backend only) | `lib.rs`/`config.rs`/`http.rs` redaction and boundary tests. Missing: user-facing consent/enable surface |
| CAP-058 | Proposal generation (typed, validated: create activity/relationship, set duration/progress, each with `rationale`) | `docs/research/ai-integration.md` | `docs/research/ai-integration.md` | `review-panel.spec.md` | RP-1 | partial (backend only) | 13 headless validation/config tests (PR #155 slice). Missing: prompt construction, document ingestion, UI trigger |
| CAP-059 | Proposal review/apply ("proposes, planner disposes") — `map_proposal_to_commands` | `docs/research/ai-integration.md`; "planner disposes" rule | `docs/research/ai-integration.md` | `review-panel.spec.md` | RP-3, RP-4, RP-5, RP-6 | partial (backend only) | 8 `command_map.rs` workflow tests through real `CommandStack`. Missing: review panel UI |
| CAP-060 | Document understanding (PDF/email reading) | `docs/research/ai-integration.md` (approved 2026-10-27) | `docs/research/ai-integration.md` | — | — | not started | — |
| CAP-061 | Broader assistant proposals (advice, enrichment, status updates) | `docs/research/ai-integration.md` (approved 2026-10-27) | `docs/research/ai-integration.md` | — | — | not started | — |

**Assistant crate public API (crates/assistant/src/lib.rs):**
| Module | Public Functions/Types |
|--------|------------------------|
| `config` | `AssistantConfig`, `SecretKey`, `is_known_provider`, `is_assistant_enabled` |
| `http` | `HttpProvider`, `Wire`, `AssistantError` variants, `CONNECT_TIMEOUT`, `READ_TIMEOUT`, `WRITE_TIMEOUT` |
| `prompt` | `build_proposal_prompt`, `parse_proposal_response`, `ACTIVITY_SNAPSHOT_CAP` |
| `proposal` | `Proposal`, `ProposalError`, `ProposedChange` variants, `ProposedChange::validate_against_project`, `ProposedChange::rationale` |
| `provider` | `AssistantProvider` trait, `AssistantError` variants |
| `settings` | `settings_path`, `load_settings`, `store_settings` |
| `command_map` | `map_proposal_to_commands` |
| `lib.rs` | `AssistantSuggestion` struct |

**Specification & tickets for Assistant Review Panel (the next slice):**
- Spec: `docs/specs/review-panel.spec.md` — full system architecture, flow, interface contracts, non-goals, constraints, open questions Q1–Q8
- Tickets: RP-1 (prompt builder/parser), RP-2 (settings storage), RP-3 (UI review panel), RP-4 (apply path), RP-5 (error handling/a11y), RP-6 (integration/polish)

---

## N. Design System & UI Polish (from Design Audit & ROADMAP)

| Area | Tool/Feature/Function | Inspiration Product | Research Note | Spec | Tickets | Status | Evidence |
|---|---|---|---|---|---|---|---|
| Typeface | Inter (SIL OFL 1.1) | — | — | `cadence-design-tokens.spec.md` | — | implemented | `docs/design/brand.md`; font loading in `crates/ui-egui` |
| Token palette | Documented token palette & type scale | — | — | `cadence-design-tokens.spec.md` | — | implemented | `cadence-ui-egui::design_tokens`; `docs/design/brand.md` |
| Widget styling | Professional widget styling in both themes | — | — | `cadence-design-tokens.spec.md` | — | implemented | `cadence-ui-egui::design` chrome module |
| Gantt polish | Weekly grid, today line, dependency arrowheads, lag labels, milestone diamonds, WBS summary bars, two-tier timescale, baseline ghosts | MS Project / P6 / Asta Powerproject / ProjectLibre / Aphex | `docs/ui-patterns.md` | — | — | implemented | Gantt rendering in `crates/ui-egui` |
| Workbench shell | View tabs across top, status bar, optional split Gantt panel | MS Project / P6 style | `docs/ui-patterns.md` | — | — | implemented | Shell in `crates/ui-egui/src/lib.rs` |
| Column chooser | `Columns` menu in toolbar | MS Project column customization / P6 layouts | `docs/ui-patterns.md` | — | — | implemented | Activity table column chooser |
| Focus rings | Contrast-tested `focus_ring` palette token | Guide II.30 / WCAG AA | `docs/audit/2026-11-03-exhaustive-design-and-gui-audit.md` | `cadence-accessibility.spec.md` | — | implemented | Focus ring applied to focused-widget stroke |
| AccessKit labels | Every field and combo box has `labelled_by` | Guide II.21/II.30 | `docs/research/screen-reader-support.md` | `cadence-accessibility.spec.md` | — | implemented | `a11y_harness` named-input check |

---

## O. Research Notes — Full Inventory

| Research File | Inspiration Products | Capabilities Covered | Status |
|---|---|---|---|
| `docs/research/ms-project-capabilities.md` | Microsoft Project | CAP-011, CAP-012, CAP-013, CAP-015, CAP-016, CAP-022, CAP-023, CAP-024, CAP-025, CAP-027, CAP-028, CAP-030, CAP-041 | Published |
| `docs/research/primavera-p6-capabilities.md` | Primavera P6 | CAP-006, CAP-010, CAP-017, CAP-026 | Published |
| `docs/research/schedule-audit.md` | DCMA 14-point, GAO ten practices, AACE RPs, Deltek Acumen Fuse, Steelray, Barbecana, ScheduleLens/SmartPM/Onplana | CAP-031, CAP-032, CAP-033, CAP-034 | Published (owner-approved 2026-10-27) |
| `docs/research/aphex-integration.md` | Aphex | CAP-008, CAP-019, CAP-020, CAP-021, CAP-035, CAP-036, CAP-037, CAP-038, CAP-039, CAP-040 | Published (owner-approved 2026-10-27) |
| `docs/research/ai-integration.md` | nPlan, Copilot for Project, Planera, ALICE, Trunk Tools, Document Crunch | CAP-056, CAP-057, CAP-058, CAP-059, CAP-060, CAP-061 | Published (owner-approved 2026-10-27) |
| `docs/research/gui-inspiration-audit.md` | MS Project, P6, Asta Powerproject, ProjectLibre, Aphex | Design patterns, UI patterns | Published |
| `docs/research/printing-the-gantt-view.md` | — | CAP-049 | Published |
| `docs/research/crash-report-submission.md` | — | CAP-050 | Published |
| `docs/research/update-check.md` | — | CAP-051 | Published |
| `docs/research/screen-reader-support.md` | — | CAP-052 | Published |

---

## P. Specifications — Full Inventory

| Spec File | Related Capabilities | Status |
|---|---|---|
| `docs/specs/cadence-accessibility.spec.md` | CAP-052 | Published |
| `docs/specs/cadence-design-tokens.spec.md` | Design system, token palette, type scale | Published |
| `docs/specs/cadence-settings-architecture.spec.md` | Settings architecture | Published |
| `docs/specs/cadence-settings-implementation.spec.md` | Settings implementation | Published |
| `docs/specs/cadence-settings.spec.md` | Settings UI | Published |
| `docs/specs/cadence-toolbar-spec.md` | Toolbar | Published |
| `docs/specs/cadence-toolbar.spec.md` | Toolbar (duplicate?) | Published |
| `docs/specs/cadence-view-sync-architecture.md` | View sync architecture | Published |
| `docs/specs/cadence-view-sync.md` | View sync | Published |
| `docs/specs/cadence-view-sync.spec.md` | View sync spec | Published |
| `docs/specs/review-panel.spec.md` | CAP-056 to CAP-059 (Assistant review panel) | Drafted for owner review |

---

## Q. Tickets — Full Inventory

| Ticket | Title | Related Capabilities | Status |
|---|---|---|---|
| `docs/tickets/RP-1.md` | Assistant prompt builder, response parser and off-thread call seam | CAP-056, CAP-058 | Not started (spec drafted) |
| `docs/tickets/RP-2.md` | Assistant settings storage (key session-only) | CAP-057 | Not started |
| `docs/tickets/RP-3.md` | Assistant review panel UI (tenth centre view tab) | CAP-056, CAP-058, CAP-059 | Not started |
| `docs/tickets/RP-4.md` | Assistant apply path (map_proposal_to_commands + CommandStack) | CAP-059 | Not started |
| `docs/tickets/RP-5.md` | Assistant error handling & accessibility | CAP-057, CAP-059 | Not started |
| `docs/tickets/RP-6.md` | Assistant integration & polish | CAP-056–CAP-059 | Not started |

---

## R. CLI Commands — Complete Inventory (19 commands)

All implemented in `crates/app/src/main.rs` with manual pattern matching on `std::env::args()`:

| # | Command | Syntax | Description |
|---|---------|--------|-------------|
| 1 | **demo** (default) | `cadence [--json]` | Run built-in demo project and print schedule |
| 2 | **save** | `cadence save <file>` | Save demo project to SQLite file |
| 3 | **load** | `cadence load <file> [--json]` | Load project file and print schedule |
| 4 | **audit** | `cadence audit <file> [--json] [--status-date YYYY-MM-DD]` | Audit programme schedule quality |
| 5 | **audit-compare** | `cadence audit-compare <before> <after> [--json]` | Forensic comparison of two programme versions |
| 6 | **reschedule** | `cadence reschedule <file> [--json] [--status-date YYYY-MM-DD]` | Reschedule remaining work from recorded progress |
| 7 | **report** | `cadence report <file> [out]` | Write Markdown report (stdout if no output file) |
| 8 | **print** | `cadence print <file> <out.pdf>` | Print project report to PDF |
| 9 | **version** | `cadence version` | Show version and last cached update check |
| 10 | **check-updates** | `cadence check-updates` | Query GitHub Releases for latest version |
| 11 | **gui** | `cadence gui [file]` | Open project in GUI (demo if no file) |
| 12 | **export-xml** | `cadence export-xml <in> <out>` | Export project as Microsoft Project XML (MSPDI) |
| 13 | **import-xml** | `cadence import-xml <in> <out>` | Import Microsoft Project XML into project file |
| 14 | **export-mpx** | `cadence export-mpx <in> <out>` | Export project as MPX format |
| 15 | **import-mpx** | `cadence import-mpx <in> <out>` | Import MPX file into project file |
| 16 | **export-csv** | `cadence export-csv <in> <out>` | Export project as CSV activity table |
| 17 | **import-csv** | `cadence import-csv <in> <out>` | Import CSV activity table into project file |
| 18 | **export-json** | `cadence export-json <in> <out>` | Export project as Cadence JSON |
| 19 | **import-json** | `cadence import-json <in> <out>` | Import Cadence JSON into project file |

---

## S. Internal Command Pattern (NOT CLI commands)

**30+ command types in `crates/commands/src/lib.rs`** — implements the `Command` trait for UI undo/redo:

| Family | Command Types |
|--------|---------------|
| **Activity** | CreateActivity, DeleteActivity, RestoreActivity, DeleteActivities, RestoreActivities, RenameActivity |
| **WBS** | CreateWbsNode, DeleteWbsNode, RestoreWbsNode, RenameWbsNode, MoveWbsNode, AssignActivityWbs |
| **Relationship** | CreateRelationship, DeleteRelationship, RestoreRelationship, SetRelationshipLag, SetRelationshipType |
| **Duration/Constraints** | SetActivityDuration, SetConstraints, SetActivityConstraint, SetMandatoryConstraint, RescheduleActivity, RescheduleActivityUndo |
| **Progress/Cost** | SetActivityProgress, SetActivityActualCost, SetActivityActualDates, SetActivityBudget |
| **Owner/Location/Promise** | SetActivityOwner, SetActivityLocation, SetActivityPromise |
| **Resource** | CreateResource, DeleteResource, RestoreResource, RenameResource, SetResourceCapacity, SetResourceKind, SetResourceCostRate |
| **Assignment** | AddResourceToActivity, RemoveResourceFromActivity, RestoreResourceOnActivity, SetAssignmentUnits, SetAssignmentUnitsUndo |
| **Calendar** | CreateCalendar, DeleteCalendar, RestoreCalendar, RenameCalendar, SetCalendarDefinition, SetActivityCalendar |
| **Baseline** | CaptureBaseline, RemoveBaseline, RestoreBaseline |
| **Lookahead** | CaptureLookaheadSnapshot, RemoveLookaheadSnapshot, RestoreLookaheadSnapshot |
| **Diary** | CreateDailyNote, DeleteDailyNote, RestoreDailyNote, UpdateDailyNote |
| **Production** | RecordProduction, UndoRecordProduction, DeleteProduction, RestoreProduction |
| **Global/View/Tool Settings** | SetGlobalSettings, SetGlobalSettingsUndo, SetViewSettings, SetViewSettingsUndo, SetToolSettings, SetToolSettingsUndo |
| **Status Date** | SetStatusDate |

---

## T. Design Tokens — Public API (crates/ui-egui/src/design_tokens.rs)

| Category | Public Types/Functions |
|----------|------------------------|
| **Colour** | `ColorTokens` struct (29 semantic roles), `LIGHT_COLORS`, `DARK_COLORS`, `ACCENT`, `ACCENT_ON`, `FOCUS_RING` |
| **Typography** | `TypographyToken`, `FontWeight`, `TypographyScale` |
| **Spacing** | `BASE_UNIT`, `SpacingToken` (10 sizes), `Density` (4 levels) |
| **Radius** | `RadiusToken` (4 sizes), `RadiusSet` |
| **Elevation** | `ElevationToken` (3 levels), `ElevationSet` |
| **Motion** | `DurationToken` (5 levels), `EasingToken` (4 types), `MotionSpec`, `MotionSet` |
| **Contrast** | `ContrastRatio`, `validate_contrast`, `contrast_ratio`, `relative_luminance`, `wcag_min_ratio`, `meets_wcag`, `SEMANTIC_CONTRAST_PAIRS` |
| **Theme** | `Theme` trait, `LightTheme`, `DarkTheme`, `ThemeMode` (Light/Dark/System), `ThemeRegistry`, `init_theme_registry`, `theme_registry` |
| **Chrome** | `chrome::primary`, `chrome::ghost`, `chrome::tab`, `chrome::secondary`, `chrome::danger` |
| **Utilities** | `variance_color`, `translucent`, `apply`, `apply_visuals`, `install_fonts`, `export_tokens_json`, `light_tokens`, `dark_tokens` |

---

## U. Accessibility Features (CAP-052) — Complete Implementation

| Feature | Implementation | Evidence |
|---------|---------------|----------|
| **WCAG AA contrast** | `validate_contrast` tests both themes against 13 semantic pairs; `contrast_ratio` + `relative_luminance` pure functions | `crates/ui-egui/tests/contrast_tests.rs` |
| **Focus rings** | `FOCUS_RING` token = `ACCENT`; applied to focused-widget stroke in chrome buttons; contrast-tested | `design_tokens.rs:1253`, `theme.rs` chrome module |
| **Keyboard completes all flows** | Ctrl+1..9 view switching; Ctrl+S/O/W/N file actions; Ctrl+Z/Y undo/redo; Ctrl+D theme; Ctrl+Plus/Minus/0 Gantt zoom; F1 shortcuts dialog | `lib.rs` keyboard handlers, `docs/qc/shortcuts.md` |
| **AccessKit labels** | Every input widget (text field, checkbox, menu item, button) tied to visible label via `Response::labelled_by` | `a11y_harness.rs` named-input check |
| **Reduced motion** | `DurationToken::ms_reduced` respects platform reduced-motion; documented limitation in `ACCESSIBILITY.md` | `design_tokens.rs:622` |
| **Target size ≥ 24pt** | Chrome buttons enforce minimum size; density variants change spacing not below 24pt floor | `theme.rs` chrome module |
| **Colour never only signal** | Error/warning/success carry text + icon; disabled explains why; selection has fill + marker | Design tokens + UI patterns |
| **Reflow at 200% / narrow window** | Shell collapses side panel before shrinking canvas; no control clips or becomes unreachable | View layout logic in `lib.rs` |

**Accessibility tests (crates/ui-egui/tests/):**
| Test File | Purpose |
|-----------|---------|
| `contrast_tests.rs` | WCAG 2.2 AA contrast validation (both themes) |
| `a11y_harness.rs` | AccessKit tree assertions over real shell (17 checks) |
| `design_token_tests.rs` | Token completeness, density scaling, theme parity, serialization |
| `token_validation.rs` | Colour roles distinct, contrast, theme parity, spacing, chrome buttons |

---

## V. Traceability Summary

### By Inspiration Product

| Inspiration Product | Capabilities Traced | Research Notes |
|---|---|---|
| Microsoft Project | 15 (CAP-011, 012, 013, 015, 016, 022, 023, 024, 025, 027, 028, 030, 041, 043, 044) | `ms-project-capabilities.md` |
| Primavera P6 | 4 (CAP-006, 010, 017, 026) | `primavera-p6-capabilities.md` |
| Aphex | 10 (CAP-008, 019, 020, 021, 035, 036, 037, 038, 039, 040) | `aphex-integration.md` |
| PMBOK/CPM (public standard) | 7 (CAP-001, 002, 003, 004, 005, 007, 029) | — |
| DCMA/GAO/AACE (public standards) | 4 (CAP-031, 032, 033, 034) | `schedule-audit.md` |
| AI tools (nPlan, Copilot, Planera, ALICE, Trunk, Document Crunch) | 6 (CAP-056–061) | `ai-integration.md` |
| Cadence original | 20+ (command layer, JSON interchange, SQLite project files, reports, platform services, project health, design system, etc.) | — |

### By Status

| Status | Count | Capabilities |
|---|---|---|
| **implemented** | 55 | CAP-001 to CAP-055 (except CAP-046 which is documented limitation) |
| **partial** | 4 | CAP-056 to CAP-059 (Assistant backend landed, UI surfaces missing) |
| **not started** | 2 | CAP-060, CAP-061 (AI document understanding, broader proposals) |
| **out of scope** | 0 | — |

**Total: 61 capabilities tracked**

---

## W. Clean-Room Compliance Notes

Per Guide §2.1–2.3 and II.19:

1. **All capabilities derived from public sources only:**
   - PMBOK/CPM formulas and terminology (public standard)
   - MS Project public documentation, help pages, public tutorials
   - Primavera P6 public documentation, help pages
   - DCMA 14-point assessment guide (public)
   - GAO Schedule Assessment Guide (public)
   - AACE International Recommended Practices (public)
   - Aphex public marketing, help docs, public demo videos

2. **No proprietary material consulted:**
   - No decompilation of MS Project, P6, Aphex binaries
   - No reverse-engineering reports used as implementation input
   - No proprietary icons, fonts, exact layouts, string tables, presets copied

3. **Original implementation:**
   - All data models, commands, UI layouts, interaction design are original
   - Capability inventory (`docs/capability-inventory.md`) records source URL for every claimed behaviour
   - Research notes (`docs/research/*.md`) contain original behavioural summaries + public URLs only

4. **Human gate enforced:**
   - Research notes for future-expansion capabilities (Aphex, Schedule Audit, AI) required owner approval before implementation (Guide §2.3)
   - All three approved 2026-10-27 per ROADMAP.md

---

## X. Quick Reference: Capability → Test/Evidence Location

| Capability Range | Primary Test Location | GUI View (if applicable) |
|---|---|---|
| CAP-001–008 | `crates/core/src/engine.rs`, `crates/core/tests/properties.rs` | CLI `cadence` table |
| CAP-009–010 | `crates/commands/src/`, `crates/core/src/wbs.rs` | — |
| CAP-011–013 | `crates/ui-egui/src/` (Table, Gantt, dependency editing) | `Ctrl+1` Table, `Ctrl+2` Gantt |
| CAP-014 | `crates/ui-egui/src/lib.rs` (ViewMode, shortcuts) | All views |
| CAP-015 | `crates/ui-egui/src/` (Resources view) | `Ctrl+3` Resources |
| CAP-016 | `crates/ui-egui/src/` (Baselines view) | `Ctrl+4` Baselines |
| CAP-017 | `crates/ui-egui/src/` (Calendars view) | `Ctrl+5` Calendars |
| CAP-018 | `crates/ui-egui/src/` (Audit view) | `Ctrl+6` Audit |
| CAP-019–021 | `crates/ui-egui/src/` (Lookahead, Diary, Production) | `Ctrl+7` Lookahead, `Ctrl+8` Diary, `Ctrl+9` Production |
| CAP-022–026 | `crates/core/src/engine.rs` (resource_*), `crates/commands/src/` | `Ctrl+3` Resources |
| CAP-027–030 | `crates/core/src/engine.rs` (evm_*), `crates/ui-egui/src/` | `Ctrl+4` Baselines |
| CAP-031–034 | `crates/core/src/audit.rs`, `crates/core/src/compare.rs` | `Ctrl+6` Audit |
| CAP-035–040 | `crates/core/src/lookahead.rs`, `clash.rs`, `diary.rs`, `production.rs` | `Ctrl+7` Lookahead, `Ctrl+8` Diary, `Ctrl+9` Production |
| CAP-041–046 | `crates/interchange/src/` (mspdi.rs, mpx.rs, csv.rs, json.rs, audit.rs, compare.rs) | CLI `cadence` import/export |
| CAP-047 | `crates/persistence/src/` | — |
| CAP-048–049 | `crates/core/src/report.rs`, `pdf.rs` | GUI **Save report**, **Print report…** |
| CAP-050–051 | `crates/app/src/crash_report.rs`, `crates/update-check/src/` | GUI crash preview, **Updates** window |
| CAP-052 | `crates/ui-egui/tests/contrast_tests.rs`, `crates/ui-egui/tests/a11y_harness.rs` | All views (keyboard/AccessKit) |
| CAP-053–055 | Repository root files, `.github/workflows/` | — |
| CAP-056–059 | `crates/assistant/src/` (prompt.rs, config.rs, http.rs, proposal.rs, command_map.rs, settings.rs) | (tenth view: Assistant — not yet built) |

---

*Generated: 2026-11-26 | Based on branch `main` @ `7755c92` (2026-11-18) and latest `docs/` tree + comprehensive codebase exploration via subagents*