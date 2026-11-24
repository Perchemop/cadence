# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Rust workspace (edition 2024, rust-version 1.85) with `egui`/`eframe` for native desktop GUI. Crates: `cadence-core` (headless domain + CPM engine), `cadence-commands` (typed command layer + undo/redo), `cadence-persistence` (SQLite project files, schema v10), `cadence-interchange` (MSPDI/MPX/CSV/JSON import-export), `cadence-assistant` (optional AI integration, off by default), `cadence-app` (binary wiring). CI: GitHub Actions (fmt, clippy -D warnings, test, multi-OS release artefacts). No local cargo; verification via `ci.sh` → Actions.

## Users

Primary: Planners and project-controls engineers on construction and capital projects. Secondary: Small consultancies and site teams without enterprise scheduling licences. Tertiary: Anyone who must keep project data on their own machine (offline-first requirement).

Situation: User has a laptop, possibly no internet, needs to produce a defensible schedule in a file they own. Employer will not buy/host a commercial tool (Primavera P6, Microsoft Project Server, etc.).

Job: Create and maintain a construction/capital-project schedule — WBS, dependencies (FS/SS/FF/SF with lag/lead), calendars, constraints, resources, baselines, earned-value metrics — and exchange it via open formats (MSPDI, MPX, CSV, JSON, Markdown, PDF).

## Product Purpose

A free, offline scheduling application that does what professional tools do (CPM scheduling, resources, baselines, EVM) without a subscription, without an account, and without sending project data anywhere. Success means: a planner can open the app, build a schedule, run CPM, assign resources, set baselines, track progress, compute EVM, and export a report — all offline, in a documented SQLite file they control.

## Positioning

Three pillars, each a mechanism a neighbouring product could not truthfully copy:

1. **Genuinely offline.** No telemetry, no account, no network call in scheduling, persistence, reporting, or interchange paths. Crash reporting and update checks are opt-in, user-gated, and token-free.
2. **Open file formats.** A documented, versioned SQLite project file; MSPDI (Microsoft Project XML) and MPX import/export; CSV and a lossless JSON format; Markdown and PDF reports. Nothing is locked behind a proprietary reader.
3. **Original and open source.** Clean-room implementation under MIT OR Apache-2.0, auditable, extensible, embeddable. No reverse-engineered proprietary formats.

## Operating Context

Workflows: Create project → build WBS → add activities → link dependencies (drag-to-create in Gantt) → assign calendars → run CPM → assign resources → level → set baseline → track progress (percent/actual dates/cost) → compute EVM → export report (Markdown/PDF) or interchange (MSPDI/MPX/CSV/JSON).

Environments: Desktop (Linux x86_64/ARM64, Windows x86_64, macOS ARM64). No web, no mobile.

Tools: Native app (`cadence-app`), CLI (`cadence` binary with subcommands: `save`, `load`, `report`, `print`, `export-xml`, `export-csv`, `export-json`, `audit`, `audit-compare`, `check-updates`, `version`, `schedule`).

Documents: Project SQLite file (schema v10), MSPDI XML, MPX, CSV, Cadence JSON, Markdown reports, PDF prints.

Rituals: Weekly schedule review, baseline comparison, earned-value review at status date, audit before submission.

## Capabilities and Constraints

**Confirmed functionality (implemented):**
- CPM engine: forward/backward pass, FS/SS/FF/SF with signed lag/lead, total/free float, critical path, cycle detection (Kahn + DFS path), date constraints (SNET/SNLT/FNET/FNLT), multiple named calendars with holiday exceptions, cross-calendar convention
- WBS: single-rooted tree, cycle-safe reparenting, undoable commands
- Resources: dictionary (Labour/Equipment/Material), capacity (units/day), cost rates, per-assignment units (basis points), histograms, conflict detection, greedy leveling
- Baselines: multiple, variance views, progress update (percent/actual dates/cost)
- Earned value: PV, EV, AC, SV, CV, SPI, CPI, EAC, BAC, TCPI, VAC per PMI standard
- Schedule audit: 18 DCMA/GAO-style checks, CPLI, BEI, forensic + baseline comparison with machine-readable JSON
- Lookahead/field planning: rolling windows, owner/promise/delay-reason tracking, published snapshots with variance, diary/huddle notes, location clash detection, production tracking
- Interchange: MSPDI round-trips WBS + named calendars; MPX/CSV import them empty (documented); lossless Cadence JSON
- CLI `--json` output for schedule, audit, comparison (headless, scriptable, AT-friendly)
- Optional AI integration (backend landed): provider seam (OpenAI-compatible, Anthropic, Ollama, Gemini), typed validated proposals, secret-redacting config, command-stack mapping for undoable apply — **no UI surface yet**
- Desktop UI (egui): nine keyboard-switchable views (Table, Gantt, Resources, Baselines, Audit, Lookahead, Diary, Production, Calendars), virtualised table, dependency drag-to-create, inspector, zoom/pan, column chooser
- Accessibility: AccessKit via eframe, WCAG AA contrast tests, standard widget labels/roles, Gantt canvas WidgetInfo summary — manual screen-reader passes pending
- Crash reporting: local panic log, GUI preview, opt-in pre-filled GitHub issue (no embedded token)
- Update check: opt-in GitHub Releases API, 6-hour cache, fail-soft offline

**Technical constraints:**
- Pure Rust, no unsafe in app crates (only `cadence-core` may use isolated unsafe with SAFETY comments)
- No panics in production code (`unwrap`/`expect`/`panic!` forbidden at crate roots)
- Headless engine: `cadence-core` has no I/O, no UI deps, fully testable via `cargo test`
- Command-centric: every mutation via typed `Command` with inverse for undo/redo
- SQLite persistence: transactional save/load, in-place migrations, FK enabled
- Offline-first: zero network in core paths; network only in opt-in crash-report/update-check
- Clean-room: no decompilation, no RE dumps, no proprietary source/assets, original implementation only

**Explicitly undecided / open (from VISION.md):**
- N1: Name conflict — "Cadence" vs Cadence Design Systems (EDA); WIPO/package registry check needed
- N2: Canonical home URL and handles (domain, GitHub org, package name) not reserved
- N3: Employer IP policy check for rights over this work
- N4: Second trusted admin with MFA for bus factor
- N5: MFA enabled on owner account and write-access accounts

## Brand Commitments

Name: **Cadence** (open question N1 — potential conflict with Cadence Design Systems).
Voice: Professional, precise, honest about limitations. No marketing fluff. "Free, offline, open" — not "best" or "enterprise-grade".
Assets: None committed. No logo, wordmark, palette, typography, or icon set locked.
Identity constraints: Must remain original (clean-room). No trade dress copying from MS Project, Primavera P6, Aphex, or others.
References made binding: MIT OR Apache-2.0 dual licence (deliberate "OR" for Rust ecosystem + patent grant). ATTRIBUTION.md lists all third-party licences.

## Evidence on Hand

Real content/data:
- Demo project (3 activities, FS link) built into binary
- `bridge_report` example project in `docs/`
- Property-based engine tests (`crates/core/tests/properties.rs`) as behavioural evidence
- 39 audit tests, 10 comparison tests, 51 assistant tests, headless workflow tests
- Release artefacts: prebuilt binaries for 4 targets + SHA-256 + CycloneDX SBOM per release (v0.3.0…v0.14.0 tagged)
- Screenshots and rendered evidence published to GitHub Releases (not in repo)

Absences future work must not fabricate:
- No fake testimonials, customers, benchmarks, pricing, or licensing claims
- No "drop-in replacement" or compatibility certification claims for commercial tools
- No cloud/mobile/web features (explicit non-goals)
- No proprietary format reverse-engineering

## Product Principles

1. **Offline-first, user-owned.** The schedule file belongs to the user; the app never phones home.
2. **Correctness over breadth.** A correct CPM engine with documented limits beats a feature-rich one with silent errors.
3. **Open interchange, not lock-in.** Every format is documented; the lossless JSON is the canonical round-trip.
4. **Command-centric integrity.** Every mutation is intentional, undoable, and testable.
5. **Honest about gaps.** Limitations (MPX/CSV calendar loss, AI UI missing, resource curves missing) are documented, not hidden.

## Accessibility & Inclusion

Target: WCAG 2.2 Level AA where feasible for native desktop (egui + AccessKit). Known gaps recorded in `ACCESSIBILITY.md`: per-bar Gantt nodes not exposed to AT, live-region status text partial, manual screen-reader passes (NVDA/Narrator/VoiceOver) not yet completed. Keyboard: all nine views switchable via `Ctrl+1`–`Ctrl+9`, undo/redo `Ctrl+Z`/`Ctrl+Y`, table navigation, Gantt zoom `Ctrl++`/`Ctrl+-`/`Ctrl+0`, shortcut help `F1`. Colour: contrast tests enforce AA for both themes. Reduced motion: in-app preference (platform signal unavailable on desktop). Target size: ≥24pt interactive targets.