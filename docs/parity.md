# Capability Parity Tracker

Living list of capabilities. Status values: `todo`, `in progress`, `done`.
Per-capability IDs, observable behaviour and acceptance criteria live in
[`docs/capability-inventory.md`](capability-inventory.md) (first edition
2026-11-18).

## Parity vs the reference products (designer review, 2026-11-18)

Judged against the public-behaviour research notes in `docs/research/` and the
clean-room record in `docs/inspiration.md`. **Microsoft Project** and
**Primavera P6** have approved capability notes; **Aphex**, the **schedule
audit tools** (DCMA/GAO practice, Deltek Acumen Fuse et al.) and the **AI
assistants** (nPlan, Copilot for Project et al.) have approved
future-expansion notes. **Asta and ProjectLibre were named as candidate
references but have no research note** — no parity claim is made against them
until a note is written and approved (guide §2.3). Nothing below is a
like-for-like claim against an enterprise product; it states which *publicly
documented concepts* Cadence implements and where it deliberately does not.

| Capability group | vs reference | Verdict | Evidence |
|---|---|---|---|
| CPM scheduling core | MS Project, P6, PMBOK | **met** for the single-project desktop scope: forward/backward pass, FS/SS/FF/SF with lag/lead, float, critical set, cycles, constraints, working-day and multiple named calendars, property-tested | `engine.rs`, `calendar_set.rs`, `properties.rs` (CAP-001…007) |
| WBS | P6 | **met** at project level (undoable tree, cycle-safe reparenting). P6's enterprise EPS/multi-project scope is out of scope | `wbs.rs` + command tests (CAP-010) |
| Combined table + Gantt workbench | MS Project | **met** for the built views: sortable/virtualised table with column chooser, Gantt with arrows/progress/zoom, dependency drag-to-create, inspector | `ui-egui`, phase-2 QC gate (CAP-011…014) |
| Resources, leveling, histograms | MS Project, P6 | **met** for dictionary, assignments with per-assignment units, cost rate, units-aware histogram, conflict detection and greedy leveling; P6-style resource curves/time-phased rates are **missing** (recorded, not hidden) | `engine.rs`, Resources view (CAP-022…026) |
| Baselines, variance, progress | MS Project | **met** for multiple baselines, variance views, percent/actual-date/cost updating | engine + command + persistence tests, Baselines view (CAP-027…030) |
| Earned value | MS Project, PMBOK | **done**: PV, EV, AC, SV, CV, SPI, CPI at a status date are implemented; EAC/BAC/TCPI/VAC family also computed per PMI standard | `engine::evm_metrics` tests (CAP-029); `evm-eac-tcpi-vac` unit tests |
| Schedule audit | DCMA 14-point, GAO practices, Fuse/Analyzer class tools | **met** for the approved research scope: 18 checks incl. the critical-path test, CPLI and BEI, plus forensic and baseline comparison with machine-readable JSON. The commercial tools expose a far larger metric catalogue — breadth beyond the 18 is future scope, and Cadence exceeds the free-tool baseline by shipping the comparison + JSON without a licence gate | `audit.rs` (39 tests), `compare.rs` (10 tests), Audit view, `cadence audit`/`audit-compare` (CAP-031…034) |
| Lookahead / field planning | Aphex | **met** for the approved research scope: rolling windows, owner/promise/delay-reason tracking, published snapshots with variance, diary/huddle notes, location clash detection, production tracking. Aphex's cloud master→lookahead sync and production forecasting **exceed this scope** and are not claimed | `lookahead.rs`, Lookahead/Diary/Production views (CAP-035…040) |
| Interchange | MS Project (MSPDI/MPX) | **partial**: MSPDI round-trips WBS and named calendars; MPX and CSV import them empty (documented limitation, not a defect); a lossless Cadence JSON exists. | `interchange` modules (CAP-041…046) |
| Scripting / AT surface | — (guide §21.5) | **exceeds the references reviewed**: `--json` schedule output and `--json` audit/comparison documents ship headless for scripts and assistive technology | `interchange/src/schedule.rs`, `audit.rs`, `compare.rs` (CAP-034, 045) |
| AI assistance | nPlan, Copilot for Project, ALICE et al. | **partial, honestly behind the references**: the provider seam, four wire protocols (OpenAI/compatible, Anthropic, Ollama, Gemini), typed validated proposals and undoable command mapping are landed and tested — but there is **no UI, no prompt construction from project data and no review panel**, so no assistant capability is user-reachable yet. The references ship full document-understanding UX; Cadence has not started that (CAP-060). The "assistant proposes, planner disposes" rule is enforced in code on every landed row | `cadence-assistant` (51 headless tests), CAP-056…061 |
| Desktop polish, keyboard, a11y | MS Project desktop | **met at the code-level scope**: nine keyboard-switchable views, WCAG AA contrast tests, AccessKit through eframe, limits recorded in `ACCESSIBILITY.md` | phase-2/3 QC gates, accessibility tests (CAP-014, 052) |
| Crash reporting + update check | — (suite standard, guide §7) | **met**: consent-gated crash report (no token), opt-in update check with cache and fail-soft offline | crash + update modules (CAP-050, 051) |

**Where Cadence exceeds the references reviewed** (recorded honestly, not as
marketing): headless `--json` surfaces for scripts and assistive technology;
keyless local-Ollama and OpenAI-compatible-server support behind the provider
seam (D30); the forensic/baseline comparison shipped alongside the audit at no
licence gate; and progress-driven rescheduling as an explicit, property-tested
engine mode (D24) rather than a hidden recalculation.

**Corrections made in this refresh** (over-claims or stale rows found and
fixed): the audit row said "14 core checks" — the code has **18** `CheckId`
variants; the "first tagged release" row said *in progress* — releases
`v0.3.0`…`v0.14.0` are tagged; the AI row said *approved — queued* — three
backend slices have landed; the crate-layout row omitted `ui-egui`,
`interchange` and `assistant`; the persistence row now records schema v10.

## Phase 0 — Foundation

| Capability | Status | Notes |
|---|---|---|
| Cargo workspace + crate layout | done | core / commands / persistence / ui-egui / interchange / assistant / app |
| GitHub Actions CI (fmt, clippy, test) | done | `.github/workflows/ci.yml` |
| Typed command layer + undo/redo | done | `cadence-commands` |
| Product name locked | done | `docs/product-name.md` |
| SQLite project file persistence | done | relational schema (current `user_version` 10), transactional save/load, CLI `save`/`load` |
| Minimal project model | done | Project, Activity, Relationship |

## Vertical slice (guide §12.4)

| Capability | Status | Notes |
|---|---|---|
| Create project | done | `Project::new`, command layer |
| Two activities | done | demo builds three |
| FS link with lag | done | plus SS/FF/SF and leads |
| Run CPM | done | forward/backward pass, float, critical set |
| Show dates in a table | done | aligned CLI table via `cadence` binary |

## Phase 1 — Scheduling engine

| Capability | Status | Notes |
|---|---|---|
| WBS hierarchy | done | tree with cycle-safe reparenting, undoable commands |
| SS / FF / SF relationships | done | signed lag/lead supported |
| Working-day calendar arithmetic | done | Mon–Fri pattern + holiday exceptions |
| Multiple calendars (working time, exceptions) | done | named calendars, per-activity selection; cross-calendar convention in `architecture.md` |
| Constraints (ASAP/ALAP, dates) | done | SNET/SNLT/FNET/FNLT date bounds, scheduled vs early/late dates, undoable command |
| Total float / free float / critical path | done | critical activity set ordered by early start; longest-path extraction later |
| Cycle detection & diagnostics | done | Kahn + DFS path, reported as `SchedError::Cycle` |
| Property-based engine tests | done | `crates/core/tests/properties.rs` (proptest) |
| Phase-1 QC gate | done | `docs/qc/phase-1.md` |
| Serde round-trip for model/schedule | done | project, link types, schedule |

## Phase 2 — Gantt & tables

| Capability | Status | Notes |
|---|---|---|
| Interactive Gantt | done | timeline, bars, arrows, zoom/pan; O(relationships) arrow pass via a cached id→row index, schedule borrowed instead of cloned per frame |
| Activity table | done | sortable columns, selection, inspector edits via commands; cached sort keyed on `edit_version`, virtualised rows with `ScrollArea::show_rows`, row numbers, sticky header; **column chooser** lets users pick visible columns (MS Project / P6 layouts) |
| Dependency editing | done | drag-to-create links, lag editing, cycle prevention |
| Zoom / pan | done | Ctrl+wheel zoom in the Gantt view, scroll areas for panning |
| Phase-2 QC gate | done | `docs/qc/phase-2.md` |

## Phase 3 — Resources

| Capability | Status | Notes |
|---|---|---|
| Resource dictionary | done | `Resource` with `kind`, `capacity` (units per working day) and `cost_per_unit`, unique case-insensitive names |
| Resource assignments | done | `Activity::resources`, undoable assign/unassign commands |
| Per-assignment units | done | sparse `Activity::assignment_units` override in basis points; `SetAssignmentUnits`, GUI editing (D25) |
| Resource cost rates | done | `Resource::cost_per_unit`; planned-cost roll-up in the report; `SetResourceCostRate` (D25). Time-phased rate tables deferred |
| Usage histograms | done | `engine::resource_histogram`: committed units per working day, units-aware |
| Over-allocation detection | done | `engine::resource_conflicts` reports every day committed units exceed capacity (D25) |
| Resource leveling | done | `engine::level_resources`, greedy earliest-conflict delay (D12) |
| Resource panel in the UI | done | dictionary, editing, assignments, units, cost rate, usage, leveling preview/apply |

## Phase 5 — Polish & interop

| Capability | Status | Notes |
|---|---|---|
| Microsoft Project XML (MSPDI) import/export | done | `cadence-interchange::mspdi` (D14) |
| MPX import/export | done | `cadence-interchange::mpx` (D14) |
| CSV activity table import/export | done | `cadence-interchange::csv`: one row per activity with constraints, links, assignments and progress; CLI `export-csv`/`import-csv` |
| JSON interchange | done | `cadence-interchange::json`: Cadence's own versioned schema and the only format that is lossless — it carries the WBS tree, named calendars and activity membership as well as everything the other formats do; CLI `export-json`/`import-json` |
| Machine-readable CLI schedule output (`--json`) | done | `cadence-interchange::schedule`, guide §21.5 tooling accessibility |
| Crash reporting | done | local panic log, GUI preview, opt-in pre-filled GitHub issue; no token embedded |
| Update check | done | opt-in GitHub Releases check with a six-hour cache, fail-soft offline; `cadence check-updates` and GUI **Updates** |
| WBS and named calendars in interchange | done for JSON | MSPDI, MPX and CSV cannot express them and import them empty (documented in `interchange::json`) |
| Printing / PDF and reports | done | Markdown reports (`cadence report`, **Save report**) and PDF printing (`cadence print`, **Print report…** via a native save dialog); D16 reversed |
| Calendar management in the UI | done | Calendars view: list, create, rename, delete, working-day and holiday editing, per-activity assignment |
| Keyboard map | done | Ctrl+Z/Y undo-redo, `Ctrl+1`–`Ctrl+9` view switching, Ctrl+D theme, Up/Down table selection, Ctrl+Plus/Minus/0 Gantt zoom, F1 shortcut window; every view reviewed for pointer-only paths |
| Accessibility audit | done for the code-level scope | AccessKit is enabled by eframe's defaults; standard widgets expose labels/roles, the Gantt canvas has a `WidgetInfo` summary, and WCAG AA contrast is tested (`accessibility_tests`); per-bar Gantt nodes, live-region status text and manual screen-reader passes are recorded in `ACCESSIBILITY.md` |
| Performance pass | done for the hot paths | cached sorted activity list, borrowed schedule in table and Gantt, O(relationships) dependency arrows, virtualised table rows; table + Gantt workbench split, view tabs and status bar added |
| Documentation, example projects, releases | done | `docs/usage.md` and the `bridge_report` example; releases are tagged through `v0.14.0` (release notes per tag) |

## Future expansion (approved 2026-10-27, queued)

The owner approved all three research notes on 2026-10-27 (audit tool first).
Approval lets the acceptance criteria drive implementation; it does not promise
a release date.

| Capability | Status | Notes |
|---|---|---|
| Programme audit tool | done | All five stages shipped: 18 checks (`CheckId` in `cadence-core/src/audit.rs`), `cadence audit [--json] [--status-date]`, the GUI **Audit** view, the critical-path test/CPLI, and the forensic + baseline comparison (`cadence audit-compare`, `kind: "comparison"` JSON alongside `kind: "audit"`). Note: `docs/research/schedule-audit.md` |
| Optional AI integration | in progress — backend landed | Off-by-default, consent-gated, reviewable undoable proposals. Landed: the `cadence-assistant` crate — provider seam (mock/no-op), typed validated `Proposal`s, secret-redacting config (PR #155, 2026-11-16); four HTTP wires OpenAI/compatible, Anthropic, Ollama, Gemini (PR #174, D30, 2026-11-17); command-stack mapping so an accepted proposal applies and undoes like a manual edit (PR #175, 2026-11-18). **Still missing — no user-reachable surface:** settings UI, prompt construction from project data, review panel; document understanding not started. The assistant proposes, the planner disposes, on every row. Note: `docs/research/ai-integration.md` |
| Lookahead / field planning (Aphex-inspired) | done | Shipped: the rolling-window extraction, the GUI **Lookahead** view (`Ctrl+7`), owner/promise/delay-reason tracking (schema v5), published snapshots (schema v6) with variance comparison, highlighting and delete, the **Diary** view (`Ctrl+8`, schema v7), location clash detection (schema v8) and production tracking (the **Production** view, `Ctrl+9`). Note: `docs/research/aphex-integration.md` |
