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
| Q2 | Should v1 include activity codes / custom fields? | Open — revisit during a later table-design slice |
| Q3 | PDF printing approach (pure-Rust layout vs. system print)? | Decided (D16): a dependency-free PDF writer plus a native save dialog (`rfd`) for the GUI |
| Q4 | Name conflict: "Cadence" is also a large EDA vendor and appears in other software. Has the name been checked against the WIPO Global Brand Database and package registries, and is the registry/domain handle reserved? | **Open — human.** A later takedown or rename is expensive; see `VISION.md` N1/N2 |
| Q5 | Employer / IP policy: has the maintainer's employment agreement been checked for rights over work related to the employer's business or made with company resources (third-party material, trade secrets, patents, trademarks, data collection, AI-model terms, SBOM obligations)? | **Open — human.** It decides whether the work can be licensed and published as it stands; see `VISION.md` N3 |
| Q6 | Admin continuity: is there a second trusted admin, and is MFA enabled on every account with write access? | **Open — human** (`VISION.md` N4/N5); the governance model assumes one maintainer today |
| Q7 | Should the project adopt a DCO (`Signed-off-by`) or a CLA? | Decided for now: neither — the dual licence plus GitHub's inbound=outbound terms is the agreement (`CONTRIBUTING.md`); revisit only if a lawyer or a dual-licensed offering requires it |
