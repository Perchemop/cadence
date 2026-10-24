# v0.1.0 release — legal and quality checklist

The guide's §2.5 exhaustive legal checklist and §9 finished-product gates,
run before publishing the v0.1.0 GitHub Release (2026-10-24).

| Area | Check | Status |
|------|-------|--------|
| Copyright | No proprietary source, assets, or substantial copied text in tree | clean — all 15k+ lines original; interchange formats implemented from public documentation (D14) |
| Clean-room | No RE tools/output used as implementation input | clean — inspiration recorded in `docs/inspiration.md`; formats from public specs and observed behaviour only |
| Trademarks | Product name/logo original; no vendor marks in UI or installers | clean — "Cadence" checked at discovery (`docs/product-name.md`); no third-party marks in UI |
| Trade dress | UI is original expression | clean — design tokens, layout and copy original (`docs/design/brand.md`) |
| Patents | Prefer public standards and non-infringing techniques | clean — CPM, calendars and PDF 1.4 are public standards; no known patented techniques used |
| Licence | App licence clear; third-party attribution complete | clean — dual MIT OR Apache-2.0; `ATTRIBUTION.md` lists direct and dev-only dependencies |
| Secrets | No keys, tokens or credentials in git or release binaries | clean — scan of tracked files found no matches; no tokens in the binary |
| Privacy | Crash reports send only what user consents to | clean — crash log is local-only (nothing sent); update check makes no network requests; README privacy section states this |
| GitHub API | No embedded privileged bot token in client | clean — update check is a stub; release publishing uses the workflow's own token |
| Research files | Public research = summaries + URLs | clean — `docs/research/*` are original summaries with public URLs, human-gated |
| Claims | README/roadmap make qualified claims only | clean — v0.1.0 notes state "first functional test build; limited scope" |
| Export/crypto | No crypto or export-controlled code | n/a — none present |
| Contributors | CONTRIBUTING.md states licence terms | done — `CONTRIBUTING.md` (MIT OR Apache-2.0 for PRs) |

## Quality gates (§9)

| Gate | Status |
|---|---|
| Every in-scope capability implemented and verified | done (`REQUIREMENTS_TRACEABILITY.md`) |
| Main workflows end-to-end | done (schedule → baseline → report → print → interchange) |
| Persistence and integrity tested | done (schema v4 round-trips, migrations, undo/redo) |
| No fake functionality | done (QC gates phase-1..5) |
| CI green; primary platform hands-on | done (Linux primary; Windows/macOS CI) |
| Additional platforms as scoped | done (Windows, macOS binaries in this release) |
| Crash reporting and update-check as designed | stubs per guide §6.1 — local crash log and `cadence version`; network features deferred to 0.2.x |
| ROADMAP and CHANGELOG accurate | done |
| No secrets; licences listed; README accurate | done |
| Clean-room rules respected | done |
