# Scope

## In scope for v1 (first major release)

- Hierarchical Work Breakdown Structure (WBS)
- Activities with full dependency logic: FS / SS / FF / SF, lag and lead
- Critical Path Method engine: forward/backward pass, total float, free
  float, critical path, cycle diagnostics
- Multiple calendars (working-week patterns, exceptions, per-activity
  selection)
- Constraints (ASAP/ALAP, date constraints)
- High-performance interactive Gantt chart with activity table and
  dependency editing
- Light and dark themes, keyboard-first UX, accessible design
- Resources: dictionary, assignments, usage histograms, basic leveling
- Multiple baselines and variance analysis
- Progress updating and basic earned-value metrics (SPI/CPI)
- Persistence: SQLite native project file; open interchange JSON, CSV, and a
  simple XML format
- Printing/PDF export and basic reports
- Example projects and documentation

## Out of scope for v1

- Multi-user collaboration / concurrent editing
- Cloud sync or any hosted component
- Mobile applications (iOS/Android)
- ERP / cost-accounting system integration
- Macros, scripting, or a plugin system
- File import from proprietary formats (MS Project, P6) — export to open
  formats only; native import may come post-v1

## Explicitly deferred (candidates for post-v1)

- Web build via WASM
- Multi-OS CI matrix
- Longest-path extraction as an alternative to total-float criticality
- Resource leveling beyond the basic algorithm

### Future expansion (approved 2026-10-27, queued)

The owner approved these on 2026-10-27, in this order, so their research notes
may now drive implementation; no release date is committed. They remain outside
the v1 "finished" bar until their first slice lands.

- **Programme audit tool** — audit a construction programme for schedule
  quality and health (DCMA/GAO-style checks) inside Cadence; see
  `docs/research/schedule-audit.md`.
- **Optional AI integration** — off-by-default, consent-gated assistance that
  reads user-supplied PDFs/emails/documents and proposes advice, skeleton
  schedules, activity enrichment and status updates; see
  `docs/research/ai-integration.md`.
- **Lookahead / field planning** — rolling short-interval windows, weekly work
  plans, diary and production tracking, inspired by publicly documented Aphex
  behaviour; see `docs/research/aphex-integration.md`.
