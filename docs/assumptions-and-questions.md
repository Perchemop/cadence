# Assumptions and Open Questions

## Facts

- Product name is **Cadence** (locked).
- The local machine is resource-constrained; GitHub Actions performs all
  heavy builds and tests (public repo required).
- Discovery answers (2026-10-21): dual primary audience; v1 excludes
  collaboration/cloud/mobile/ERP/macros; Linux-only CI for now; original
  a11y-aware design.

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
| Q1 | Native file format versioning / forward-compat strategy? | Open — decide before first tagged release |
| Q2 | Should v1 include activity codes / custom fields? | Open — revisit during Phase 2 table design |
| Q3 | PDF printing approach (pure-Rust layout vs. system print)? | Open — Phase 5 decision |
