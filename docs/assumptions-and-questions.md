# Assumptions and Open Questions

## Facts

- Product name is **Cadence** (locked).
- The local machine is resource-constrained; GitHub Actions performs all
  heavy builds and tests (public repo required).
- Discovery answers (2026-10-21): dual primary audience; v1 excludes
  collaboration/cloud/mobile/ERP/macros; original a11y-aware design.

## Answers to the suite-ready discovery questions (2026-10-24)

| # | Question | Answer |
|---|---|---|
| 4 (suite) | Part of a multi-app suite? | **No — single app.** Suite shared crates (`crash-report`, `update-check`) are deferred until a second app exists; v0.1.0 ships in-app stubs instead (ROADMAP notes the extraction point) |
| 10 (primary platform) | Primary local OS for the first working build | **Linux** (the machine used daily); other platforms via CI |
| 11 (targets) | Platforms after local success | Windows, macOS (both in CI and releases); web/WASM not planned for v1 |
| 12 (offline) | Fully offline? | Yes for scheduling, persistence, reports and interchange; crash submission and update checks are opt-in, fail-soft network features planned for 0.2.x |
| 16 (releases) | Frequent semver GitHub Releases with detailed notes? | **Confirmed** — release.yml publishes on `v*.*.*` tags; v0.1.0 released |
| 17 (roadmap) | Public ROADMAP.md with readiness % and milestones? | **Confirmed** — `ROADMAP.md` maintained with evidence-based percentages |
| 18 (shared crates) | Shared crash/update crates now? | Deferred (single app); stubs live in the app binary |

## Assumptions (unconfirmed — risk noted)

| # | Assumption | Risk if wrong |
|---|---|---|
| A1 | Mon–Fri working week with holiday exceptions covers the typical v1 user's calendars | Low — calendar model is extensible (working-minutes can be added later) |
| A2 | SQLite is sufficient for projects of thousands of activities | Low — measured in Phase 2 performance pass |
| A3 | egui/eframe can deliver the "premium feel" bar for a data-dense Gantt | Medium — would require reconsidering the GUI stack (decision logged in DECISIONS.md) |
| A4 | Basic (non-optimising) resource leveling is enough for v1 | Medium — users of large projects may expect P6-grade leveling |

## Open questions

| # | Question | Status |
|---|---|---|
| Q1 | Native file format versioning / forward-compat strategy? | Decided (D9): relational SQLite, `PRAGMA user_version`; newer versions rejected |
| Q2 | Should v1 include activity codes / custom fields? | Open — revisit during Phase 2 table design |
| Q3 | PDF printing approach (pure-Rust layout vs. system print)? | Open — Phase 5 decision |
