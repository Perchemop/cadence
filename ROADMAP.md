# Roadmap

Cadence is a clean-room construction and capital-project scheduling
application: WBS, full dependency logic, CPM engine, working-day calendars,
resources with leveling, baselines, earned-value metrics, a Gantt/table GUI,
reports in Markdown and PDF, and open interchange (MSPDI, MPX, CSV, JSON).

Status is updated **only from evidence**: a capability is counted when it is
implemented and verified (CI run on the platforms, or hands-on local test
where CI cannot see it).

## Release milestones

### v0.1.0 — first functional test build (this tag)

The whole Phase 0–4 plan plus Phase 5 apart from the items below. Verified by
CI run on the release commit: 203+ tests per platform (Linux, Windows, macOS),
clippy with warnings denied, release binary packaged for all three.

| Area | Readiness | Evidence |
|---|---|---|
| CPM engine (passes, float, critical set, cycles, constraints) | 100% | `engine.rs` unit tests + property tests |
| Working-day calendars (multi-calendar, holidays) | 100% | unit + property tests |
| WBS hierarchy | 100% | unit + command tests |
| Command layer (typed, undo/redo) | 100% | command tests per family |
| SQLite project files (schema v4, migrations) | 100% | persistence round-trip/migration tests |
| Resources, histograms, leveling | 100% | engine + command + property tests |
| Baselines, variance, EVM | 100% | engine tests, GUI Baselines view |
| Progress updating | 100% | command + persistence tests |
| Gantt + table GUI | 100% of built views | Phase-2/3 QC checklists (interactive parts hands-on) |
| Reports (Markdown + PDF) and printing | 100% | report + PDF writer tests; dialog hands-on |
| Interchange MSPDI / MPX / CSV / JSON | 100% of documented subsets | round-trip + property tests |
| CI: multi-OS verify + release artefacts | 100% | runs on every push; release.yml on tags |
| Accessibility | ~60% | WCAG AA contrast verified; keyboard map added; screen-reader/keyboard-only audit outstanding |
| Crash reporting | stub | local panic hook + crash log (guide §6.1 allows a stub) |
| Update check | stub | version reported; network check deferred to 0.2.x |

### v0.1.x — polish the first release

- [ ] Fix anything the first release surfaces (bug reports via GitHub Issues).
- [ ] Windows and macOS release binaries proven on real machines.

### v0.2.0 — suite infrastructure

- [ ] Crash reporting: GitHub Issue creation with user consent, UI preview
      before send (guide §7.1); local crash log already in place.
- [ ] Update check: query GitHub Releases API, compare versions, surface in
      the GUI; fail soft offline (guide §7.2).
- [ ] Print the Gantt view: capture the rendered view and emit an image page
      through the existing PDF writer.
- [ ] Accessibility: AccessKit integration for screen readers; full
      keyboard-only operation audit.

### v0.3.x — deeper scheduling

- [ ] Progress-driven rescheduling mode (actual dates feeding the engine —
      D15 records why this must be an explicit mode).
- [ ] Per-assignment resource units and cost rate tables (D11 records the
      current one-unit model).
- [ ] Calendar editing in the GUI (calendars are modelled and persisted; the
      GUI surface is not built).

### Later

- [ ] Resource histograms rendered in the GUI (data and engine support exist).
- [ ] Suite shared crates (`crash-report`, `update-check`) extracted once a
      second application joins the suite (guide §7.3).

## Out of scope (recorded in `docs/scope.md`)

Mobile platforms; any type of telemetry; proprietary file formats.
